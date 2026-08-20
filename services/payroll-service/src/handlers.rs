//! Payroll service HTTP handlers.
//!
//! Every handler:
//! - is protected by [`nexora_common::rbac::AuthContextExt`] permission checks
//!   (deny-by-default; callers without the permission get 403)
//! - gets its RLS-scoped DB connection from the [`DbConn`] extension installed by
//!   [`nexora_common::tenant_context::rls_middleware`]
//! - emits a hash-chained audit record + outbox event via [`audit_emit::emit_payroll_event`]
//!   inside the same request transaction, so both roll back together
//!
//! Wire types use ULIDs (the external address the rest of the platform knows).
//! UUID primary keys stay internal to the DB layer.

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use nexora_common::{
    error::{ErrorResponse, NexoraError, NexoraResult},
    rbac::AuthContextExt,
    tenant_context::{AuthContext, DbConn},
    ulid::new_ulid,
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::{audit_emit::emit_payroll_event, models::*, pdf, AppState};

// ============================================================================
// Helpers
// ============================================================================

/// Resolve a tenant ULID (from [`AuthContext`]) to its internal UUID.
///
/// Guards against RLS-context drift: even though RLS already restricts rows,
/// we also validate the ULID exists so a missing/revoked tenant returns 404
/// instead of silently returning an empty set.
async fn resolve_tenant(ctx: &AuthContext, db: &mut sqlx::PgConnection) -> NexoraResult<Uuid> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM tenants WHERE ulid = $1")
        .bind(ctx.tenant_id.to_string())
        .fetch_optional(&mut *db)
        .await?
        .ok_or_else(|| NexoraError::NotFound(format!("tenant {} not found", ctx.tenant_id)))
}

/// Resolve an org ULID to its internal UUID.
async fn resolve_org(ctx: &AuthContext, db: &mut sqlx::PgConnection) -> NexoraResult<Uuid> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM organizations WHERE ulid = $1")
        .bind(ctx.org_id.to_string())
        .fetch_optional(&mut *db)
        .await?
        .ok_or_else(|| NexoraError::NotFound(format!("organization {} not found", ctx.org_id)))
}

/// Resolve an employee ULID to its internal UUID (scoped by tenant via RLS).
async fn resolve_employee_ulid(db: &mut sqlx::PgConnection, ulid: &str) -> NexoraResult<Uuid> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM wf_employees WHERE ulid = $1")
        .bind(ulid)
        .fetch_optional(&mut *db)
        .await?
        .ok_or_else(|| NexoraError::NotFound(format!("employee {ulid} not found")))
}

/// Resolve a ULID to its UUID for any table whose ULID column is CHAR(26).
async fn resolve_ulid(db: &mut sqlx::PgConnection, table: &str, ulid: &str) -> NexoraResult<Uuid> {
    sqlx::query_scalar::<_, Uuid>(&format!("SELECT id FROM {table} WHERE ulid = $1"))
        .bind(ulid)
        .fetch_optional(&mut *db)
        .await?
        .ok_or_else(|| NexoraError::NotFound(format!("{table} {ulid} not found")))
}

/// Parse a ULID from the URL path; return 400 on malformed input.
fn parse_ulid(s: &str) -> NexoraResult<String> {
    if s.chars().count() != 26 {
        return Err(NexoraError::BadRequest(format!("invalid ULID: {s}")));
    }
    // Basic format check: must be Crockford base32 (upper-case letters + digits,
    // no padding). We treat anything 26 chars as a valid ULID candidate and let
    // the DB be the final arbiter.
    Ok(s.to_string())
}

// ============================================================================
// Payroll runs
// ============================================================================

/// Create a new payroll run in `draft` status.
#[utoipa::path(
    post,
    path = "/api/v1/payroll/runs",
    request_body = CreatePayrollRunRequest,
    responses(
        (status = 201, description = "Created", body = PayrollRunResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
    ),
    tag = "payroll",
    security(("bearerAuth" = []))
)]
pub async fn create_payroll_run(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Json(req): Json<CreatePayrollRunRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.create")?;

    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    // Validate the requested config version exists for the tenant.
    let config_exists: (i64,) = sqlx::query_scalar::<_, (i64,)>(
        "SELECT 1 FROM payroll_configurations WHERE tenant_id = $1 AND config_version = $2 LIMIT 1",
    )
    .bind(tenant_id)
    .bind(&req.config_version)
    .fetch_one(&mut **conn)
    .await?
    .into();

    let now = chrono::Utc::now();
    let ulid = new_ulid();

    let row = sqlx::query(
        r#"INSERT INTO payroll_runs
               (ulid, tenant_id, org_id, config_version, period_start, period_end,
                status, total_gross_cdf, total_net_cdf, total_employer_cost_cdf,
                employee_count, initiated_by, metadata, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, 'draft', 0, 0, 0, 0, $7, '{}', $8, $8)"#,
    )
    .bind(&ulid)
    .bind(tenant_id)
    .bind(org_id)
    .bind(&req.config_version)
    .bind(req.period_start)
    .bind(req.period_end)
    .bind(auth.user_id.to_string())
    .bind(now)
    .execute(&mut **conn)
    .await?;

    // Emit audit + outbox in the same transaction.
    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_run",
        &ulid,
        uuid::Uuid::nil(),
        "payroll.run.created",
        serde_json::json!({
            "config_version": req.config_version,
            "period_start": req.period_start,
            "period_end": req.period_end,
        }),
    )
    .await?;

    let resp = PayrollRunResponse {
        ulid,
        config_version: req.config_version,
        period_start: req.period_start,
        period_end: req.period_end,
        status: PayrollRunStatus::Draft,
        total_gross: nexora_common::money::Money::zero(nexora_common::money::CurrencyCode::cdf()),
        total_net: nexora_common::money::Money::zero(nexora_common::money::CurrencyCode::cdf()),
        total_employer_cost: nexora_common::money::Money::zero(
            nexora_common::money::CurrencyCode::cdf(),
        ),
        employee_count: 0,
        reviewed_at: None,
        approved_at: None,
        locked_at: None,
        created_at: now,
        updated_at: now,
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

/// List payroll runs for the current tenant.
#[utoipa::path(
    get,
    path = "/api/v1/payroll/runs",
    params(PageQuery),
    responses(
        (status = 200, description = "OK", body = PageResponse<PayrollRunResponse>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payroll",
    security(("bearerAuth" = []))
)]
pub async fn list_payroll_runs(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Query(q): Query<PageQuery>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_any_permission(&["payroll.read", "payroll.calculate"])?;

    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let q = q.sanitized();

    let where_clause = if let Some(ref status) = q.status {
        let parsed: Result<PayrollRunStatus, _> = status.parse();
        match parsed {
            Ok(s) => format!(" AND status = '{}'", s.as_str()),
            Err(_) => {
                return Err(NexoraError::BadRequest(format!(
                    "invalid status filter: {status}"
                )))
            }
        }
    } else {
        String::new()
    };

    let total: (i64,) = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM payroll_runs WHERE tenant_id = $1{}",
        where_clause
    ))
    .bind(tenant_id)
    .fetch_one(&mut **conn)
    .await?;

    let offset = q.offset();
    let rows = sqlx::query(&format!(
        "SELECT ulid, config_version, period_start, period_end, status, \
             total_gross_cdf, total_net_cdf, total_employer_cost_cdf, employee_count, \
             reviewed_at, approved_at, locked_at, created_at, updated_at \
             FROM payroll_runs \
             WHERE tenant_id = $1{} \
             ORDER BY created_at DESC \
             LIMIT $2 OFFSET $3",
        where_clause
    ))
    .bind(tenant_id)
    .bind(q.page_size as i32)
    .bind(offset as i32)
    .fetch_all(&mut **conn)
    .await?;

    let items: Vec<PayrollRunResponse> = rows
        .into_iter()
        .map(|r| {
            let status_str: String = r.get("status");
            PayrollRunResponse {
                ulid: r.get("ulid"),
                config_version: r.get("config_version"),
                period_start: r.get("period_start"),
                period_end: r.get("period_end"),
                status: PayrollRunStatus::from_str(&status_str).unwrap_or(PayrollRunStatus::Draft),
                total_gross: nexora_common::money::Money::new(
                    r.get::<nexora_common::Decimal, _>("total_gross_cdf"),
                    nexora_common::money::CurrencyCode::cdf(),
                ),
                total_net: nexora_common::money::Money::new(
                    r.get::<nexora_common::Decimal, _>("total_net_cdf"),
                    nexora_common::money::CurrencyCode::cdf(),
                ),
                total_employer_cost: nexora_common::money::Money::new(
                    r.get::<nexora_common::Decimal, _>("total_employer_cost_cdf"),
                    nexora_common::money::CurrencyCode::cdf(),
                ),
                employee_count: r.get("employee_count"),
                reviewed_at: r.try_get("reviewed_at").unwrap_or(None),
                approved_at: r.try_get("approved_at").unwrap_or(None),
                locked_at: r.try_get("locked_at").unwrap_or(None),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }
        })
        .collect();

    Ok(Json(PageResponse {
        items,
        total: total.0,
        page: q.page,
        page_size: q.page_size,
    }))
}

/// Get a single payroll run by ULID.
#[utoipa::path(
    get,
    path = "/api/v1/payroll/runs/{ulid}",
    params(("ulid" = String, Path, description = "Payroll run ULID")),
    responses(
        (status = 200, description = "OK", body = PayrollRunResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payroll",
    security(("bearerAuth" = []))
)]
pub async fn get_payroll_run(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.read")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;

    let row = sqlx::query(
        "SELECT ulid, config_version, period_start, period_end, status, \
         total_gross_cdf, total_net_cdf, total_employer_cost_cdf, employee_count, \
         reviewed_at, approved_at, locked_at, created_at, updated_at \
         FROM payroll_runs WHERE ulid = $1 AND tenant_id = $2",
    )
    .bind(&ulid)
    .bind(tenant_id)
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!("payroll run {ulid} not found")))?;

    let status_str: String = row.get("status");
    Ok(Json(PayrollRunResponse {
        ulid: row.get("ulid"),
        config_version: row.get("config_version"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        status: PayrollRunStatus::from_str(&status_str).unwrap_or(PayrollRunStatus::Draft),
        total_gross: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("total_gross_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        total_net: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("total_net_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        total_employer_cost: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("total_employer_cost_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        employee_count: row.get("employee_count"),
        reviewed_at: row.try_get("reviewed_at").unwrap_or(None),
        approved_at: row.try_get("approved_at").unwrap_or(None),
        locked_at: row.try_get("locked_at").unwrap_or(None),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

/// Calculate payroll for a draft run (draft -> review transition).
#[utoipa::path(
    post,
    path = "/api/v1/payroll/runs/{ulid}/calculate",
    request_body = CalculateRunRequest,
    responses(
        (status = 200, description = "Calculated", body = CalculationSummaryResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payroll",
    security(("bearerAuth" = []))
)]
pub async fn calculate_payroll_run(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
    Json(req): Json<CalculateRunRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.calculate")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    // Fetch the run.
    let run = sqlx::query(
        "SELECT id, ulid, status, config_version, period_start, period_end, \
         total_gross_cdf, total_net_cdf, employee_count, metadata \
         FROM payroll_runs WHERE ulid = $1 AND tenant_id = $2",
    )
    .bind(&ulid)
    .bind(tenant_id)
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!("payroll run {ulid} not found")))?;

    let run_id: Uuid = run.get("id");
    let status_str: String = run.get("status");
    let current_status = PayrollRunStatus::from_str(&status_str)
        .map_err(|_| NexoraError::BadRequest(format!("invalid run status: {status_str}")))?;

    if current_status != PayrollRunStatus::Draft {
        return Err(NexoraError::BadRequest(format!(
            "can only calculate draft runs, got {current_status}"
        )));
    }

    // Fetch the config version.
    let config_row = sqlx::query(
        "SELECT config_version, ipr_bands, cnss_employee_rate, cnss_employer_rate, \
         cnss_ceiling, smig_daily, smig_monthly_26, smig_monthly_30, is_active \
         FROM country_tax_configs WHERE tenant_id = $1 AND config_version = $2 AND is_active = true",
    )
    .bind(tenant_id)
    .bind(run.get("config_version"))
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!(
        "active config version {} not found for tenant",
        run.get("config_version")
    )))?;

    // Parse config from DB row.
    let ipr_bands: Vec<crate::calc::IprBracket> =
        serde_json::from_value(config_row.get::<serde_json::Value, _>("ipr_bands"))
            .map_err(|e| NexoraError::Internal(format!("failed to parse ipr_bands: {e}")))?;

    let cnss_config = crate::calc::CnssConfig {
        employee_rate: config_row.get("cnss_employee_rate"),
        employer_rate: config_row.get("cnss_employer_rate"),
        ceiling: config_row.get("cnss_ceiling"),
    };

    let smig_config = crate::calc::SmigConfig {
        daily: config_row.get("smig_daily"),
        monthly_26: config_row.get("smig_monthly_26"),
        monthly_30: config_row.get("smig_monthly_30"),
    };

    let config = crate::calc::PayrollConfig {
        config_version: config_row.get("config_version"),
        ipr_brackets: ipr_bands,
        cnss: cnss_config,
        smig: smig_config,
        currency: nexora_common::money::CurrencyCode::cdf(),
    };

    // Fetch employees for this run.
    let employee_ids: Vec<Uuid> = {
        let mut ids = Vec::new();
        // Get employees from the run's employee scope (if any) or all active employees.
        let metadata: serde_json::Value =
            run.try_get("metadata").unwrap_or(serde_json::Value::Null);
        if let Some(employee_ulids) = metadata.get("employee_ulids").and_then(|v| v.as_array()) {
            for ulid_val in employee_ulids {
                if let Some(ulid_str) = ulid_val.as_str() {
                    let emp_id = resolve_employee_ulid(&mut **conn, ulid_str).await?;
                    ids.push(emp_id);
                }
            }
        } else {
            // Get all active employees for the tenant.
            let emp_rows = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM wf_employees WHERE tenant_id = $1 AND status = 'active'",
            )
            .bind(tenant_id)
            .fetch_all(&mut **conn)
            .await?;
            ids = emp_rows;
        }
        ids
    };

    // For each employee, fetch their compensation and calculate.
    let default_days = req.default_days_worked.unwrap_or(26);
    let mut results = Vec::new();
    let mut smig_violations = Vec::new();
    let mut total_gross =
        nexora_common::money::Money::zero(nexora_common::money::CurrencyCode::cdf());
    let mut total_net =
        nexora_common::money::Money::zero(nexora_common::money::CurrencyCode::cdf());
    let mut total_employer =
        nexora_common::money::Money::zero(nexora_common::money::CurrencyCode::cdf());
    let mut smig_compliant_count = 0;

    for emp_id in &employee_ids {
        // Get employee compensation from wf_compensation_records.
        let comp = sqlx::query(
            "SELECT gross_amount_minor, currency_id, frequency FROM wf_compensation_records \
             WHERE employee_id = $1 AND end_date IS NULL ORDER BY effective_date DESC LIMIT 1",
        )
        .bind(*emp_id)
        .fetch_optional(&mut **conn)
        .await?
        .ok_or_else(|| NexoraError::Internal(format!("no compensation for employee {emp_id}")))?;

        let gross_minor: i64 = comp.get("gross_amount_minor");
        // Default monthly gross in minor units (assuming CDF with 0 decimal places for simplicity).
        let gross = nexora_common::money::Money::new(
            nexora_common::Decimal::from(gross_minor),
            nexora_common::money::CurrencyCode::cdf(),
        );

        let input = crate::calc::PayrollInput {
            employee_id: *emp_id,
            gross_pay: gross,
            overtime_hours: None,
            overtime_rate: None,
            allowances: vec![],
            deductions: vec![],
            days_worked: default_days,
        };

        let result = crate::calc::calculate_payroll(input, &config)
            .map_err(|e| NexoraError::Internal(format!("calculation error: {e}")))?;

        results.push(result.clone());

        total_gross = total_gross.checked_add(&result.gross_pay).unwrap();
        total_net = total_net.checked_add(&result.net_pay).unwrap();
        total_employer = total_employer.checked_add(&result.employer_cost).unwrap();

        if result.smig_compliant {
            smig_compliant_count += 1;
        } else {
            // Get employee ULID for the violation list.
            let emp_ulid: String =
                sqlx::query_scalar("SELECT ulid FROM wf_employees WHERE id = $1")
                    .bind(*emp_id)
                    .fetch_one(&mut **conn)
                    .await?;
            smig_violations.push(emp_ulid);
        }

        // Write payslip.
        let payslip_ulid = new_ulid();
        sqlx::query(
            r#"INSERT INTO payslips
                   (ulid, tenant_id, org_id, payroll_run_id, employee_id, version,
                    gross_pay_cdf, taxable_pay_cdf, ipr_deduction_cdf, cnss_employee_cdf,
                    cnss_employer_cdf, net_pay_cdf, employer_cost_cdf, metadata, created_at)
               VALUES ($1, $2, $3, $4, $5, 1, $6, $7, $8, $9, $10, $11, $12, '{}', NOW())"#,
        )
        .bind(&payslip_ulid)
        .bind(tenant_id)
        .bind(org_id)
        .bind(run_id)
        .bind(*emp_id)
        .bind(result.gross_pay.amount)
        .bind(result.taxable_gross.amount)
        .bind(result.ipr_deduction.amount)
        .bind(result.cnss_employee.amount)
        .bind(result.cnss_employer.amount)
        .bind(result.net_pay.amount)
        .bind(result.employer_cost.amount)
        .execute(&mut **conn)
        .await?;

        // Write payroll items (line ledger).
        let items = [
            ("BASIC", "Earning", result.gross_pay, true, true),
            ("IPR", "Tax", result.ipr_deduction, false, false),
            ("CNSS_EMP", "Deduction", result.cnss_employee, false, false),
        ];
        for (code, item_type, amount, taxable, cnssable) in items {
            sqlx::query(
                r#"INSERT INTO payroll_items
                       (ulid, tenant_id, org_id, payslip_id, item_type, item_code,
                        description, amount_cdf, is_taxable, is_cnssable, metadata, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, '{}', NOW())"#,
            )
            .bind(new_ulid())
            .bind(tenant_id)
            .bind(org_id)
            .bind(payslip_ulid)
            .bind(item_type)
            .bind(code)
            .bind(format!("{code} for payroll run {ulid}"))
            .bind(amount.amount)
            .bind(taxable)
            .bind(cnssable)
            .execute(&mut **conn)
            .await?;
        }
    }

    let smig_non_compliant = results.len() - smig_compliant_count;

    // Update the run totals.
    sqlx::query(
        "UPDATE payroll_runs SET status = 'review', total_gross_cdf = $1, \
         total_net_cdf = $2, total_employer_cost_cdf = $3, employee_count = $4, \
         updated_at = NOW() WHERE id = $5",
    )
    .bind(total_gross.amount)
    .bind(total_net.amount)
    .bind(total_employer.amount)
    .bind(results.len() as i32)
    .bind(run_id)
    .execute(&mut **conn)
    .await?;

    // Audit + outbox.
    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_run",
        &ulid,
        run_id,
        "payroll.run.calculated",
        serde_json::json!({
            "employee_count": results.len(),
            "smig_compliant_count": smig_compliant_count,
            "smig_non_compliant_count": smig_non_compliant,
            "smig_violations": smig_violations,
        }),
    )
    .await?;

    let run_resp = get_payroll_run_response_from_row(&mut *conn, run_id, tenant_id).await?;

    Ok(Json(CalculationSummaryResponse {
        run: run_resp,
        payslips_written: results.len(),
        smig_compliant_count,
        smig_non_compliant_count: smig_non_compliant,
        smig_violations,
    }))
}

/// Review a payroll run (review -> approved or review -> draft for edits).
#[utoipa::path(
    post,
    path = "/api/v1/payroll/runs/{ulid}/review",
    request_body = ReviewPayrollRunRequest,
    responses(
        (status = 200, description = "Reviewed", body = PayrollRunResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payroll",
    security(("bearerAuth" = []))
)]
pub async fn review_payroll_run(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
    Json(req): Json<ReviewPayrollRunRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.review")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    let run = sqlx::query("SELECT id, status FROM payroll_runs WHERE ulid = $1 AND tenant_id = $2")
        .bind(&ulid)
        .bind(tenant_id)
        .fetch_optional(&mut **conn)
        .await?
        .ok_or_else(|| NexoraError::NotFound(format!("payroll run {ulid} not found")))?;

    let run_id: Uuid = run.get("id");
    let status_str: String = run.get("status");
    let current_status = PayrollRunStatus::from_str(&status_str)
        .map_err(|_| NexoraError::BadRequest(format!("invalid run status: {status_str}")))?;

    // review -> approved (forward) or review -> draft (reopen for edits)
    let new_status = match (current_status, req.notes.as_deref()) {
        (PayrollRunStatus::Review, _) => {
            // Default: approve on review unless notes indicate reopening.
            if req
                .notes
                .as_deref()
                .map_or(false, |n| n.contains("reopen") || n.contains("revise"))
            {
                PayrollRunStatus::Draft
            } else {
                PayrollRunStatus::Approved
            }
        }
        _ => {
            return Err(NexoraError::BadRequest(format!(
                "cannot review run in {current_status} status"
            )))
        }
    };

    sqlx::query(
        "UPDATE payroll_runs SET status = $1, reviewed_by = $2, reviewed_at = NOW(), \
         updated_at = NOW() WHERE id = $3",
    )
    .bind(new_status.as_str())
    .bind(auth.user_id.to_string())
    .bind(run_id)
    .execute(&mut **conn)
    .await?;

    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_run",
        &ulid,
        run_id,
        &format!("payroll.run.{new_status}"),
        serde_json::json!({
            "from_status": current_status.as_str(),
            "notes": req.notes,
            "new_status": new_status.as_str(),
        }),
    )
    .await?;

    let resp = get_payroll_run_response_from_row(&mut *conn, run_id, tenant_id).await?;
    Ok(Json(resp))
}

/// Approve a payroll run (approved -> locked transition).
#[utoipa::path(
    post,
    path = "/api/v1/payroll/runs/{ulid}/approve",
    request_body = ApprovePayrollRunRequest,
    responses(
        (status = 200, description = "Approved/Locked", body = PayrollRunResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payroll",
    security(("bearerAuth" = []))
)]
pub async fn approve_payroll_run(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
    Json(req): Json<ApprovePayrollRunRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.approve")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    let run = sqlx::query(
        "SELECT id, status, total_gross_cdf, total_net_cdf, employee_count \
         FROM payroll_runs WHERE ulid = $1 AND tenant_id = $2",
    )
    .bind(&ulid)
    .bind(tenant_id)
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!("payroll run {ulid} not found")))?;

    let run_id: Uuid = run.get("id");
    let status_str: String = run.get("status");
    let current_status = PayrollRunStatus::from_str(&status_str)
        .map_err(|_| NexoraError::BadRequest(format!("invalid run status: {status_str}")))?;

    if current_status != PayrollRunStatus::Approved {
        return Err(NexoraError::BadRequest(format!(
            "can only lock approved runs, got {current_status}"
        )));
    }

    // Lock the run.
    sqlx::query(
        "UPDATE payroll_runs SET status = 'locked', approved_by = $1, approved_at = NOW(), \
         locked_at = NOW(), updated_at = NOW() WHERE id = $2",
    )
    .bind(auth.user_id.to_string())
    .bind(run_id)
    .execute(&mut **conn)
    .await?;

    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_run",
        &ulid,
        run_id,
        "payroll.run.locked",
        serde_json::json!({
            "total_gross": run.get::<nexora_common::Decimal, _>("total_gross_cdf"),
            "total_net": run.get::<nexora_common::Decimal, _>("total_net_cdf"),
            "employee_count": run.get("employee_count"),
            "approver_notes": req.approver_notes,
        }),
    )
    .await?;

    let resp = get_payroll_run_response_from_row(&mut *conn, run_id, tenant_id).await?;
    Ok(Json(resp))
}

/// Lock/confirm a payroll run (approved -> locked).
///
/// Alias for approve — keeping both for backward compat with API consumers that
/// use /lock directly. The lifecycle is: draft -> review -> approved -> locked.
#[utoipa::path(
    post,
    path = "/api/v1/payroll/runs/{ulid}/lock",
    request_body = TransitionRequest,
    responses(
        (status = 200, description = "Locked", body = PayrollRunResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payroll",
    security(("bearerAuth" = []))
)]
pub async fn lock_payroll_run(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
    Json(_req): Json<TransitionRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.lock")?;

    // Reuse approve logic — lock is the terminal transition.
    approve_payroll_run(
        State(state),
        Extension(auth),
        Extension(db),
        Path(ulid),
        Json(ApprovePayrollRunRequest {
            approver_notes: None,
        }),
    )
    .await
}

/// Cancel a payroll run (draft/review -> cancelled).
#[utoipa::path(
    post,
    path = "/api/v1/payroll/runs/{ulid}/cancel",
    request_body = TransitionRequest,
    responses(
        (status = 200, description = "Cancelled", body = PayrollRunResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payroll",
    security(("bearerAuth" = []))
)]
pub async fn cancel_payroll_run(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
    Json(req): Json<TransitionRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.calculate")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    let run = sqlx::query("SELECT id, status FROM payroll_runs WHERE ulid = $1 AND tenant_id = $2")
        .bind(&ulid)
        .bind(tenant_id)
        .fetch_optional(&mut **conn)
        .await?
        .ok_or_else(|| NexoraError::NotFound(format!("payroll run {ulid} not found")))?;

    let run_id: Uuid = run.get("id");
    let status_str: String = run.get("status");
    let current_status = PayrollRunStatus::from_str(&status_str)
        .map_err(|_| NexoraError::BadRequest(format!("invalid run status: {status_str}")))?;

    // Only draft or review can be cancelled.
    if current_status != PayrollRunStatus::Draft && current_status != PayrollRunStatus::Review {
        return Err(NexoraError::BadRequest(format!(
            "cannot cancel run in {current_status} status"
        )));
    }

    sqlx::query("UPDATE payroll_runs SET status = 'cancelled', updated_at = NOW() WHERE id = $1")
        .bind(run_id)
        .execute(&mut **conn)
        .await?;

    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_run",
        &ulid,
        run_id,
        "payroll.run.cancelled",
        serde_json::json!({
            "from_status": current_status.as_str(),
            "reason": req.notes,
        }),
    )
    .await?;

    let resp = get_payroll_run_response_from_row(&mut *conn, run_id, tenant_id).await?;
    Ok(Json(resp))
}

// ============================================================================
// Payslips
// ============================================================================

/// Get a single payslip by ULID.
#[utoipa::path(
    get,
    path = "/api/v1/payroll/payslips/{ulid}",
    params(("ulid" = String, Path, description = "Payslip ULID")),
    responses(
        (status = 200, description = "OK", body = PayslipResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payslips",
    security(("bearerAuth" = []))
)]
pub async fn get_payslip(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payslip.read")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;

    let row = sqlx::query(
        "SELECT p.ulid, p.payroll_run_id, p.employee_id, p.version, \
         p.gross_pay_cdf, p.taxable_pay_cdf, p.ipr_deduction_cdf, \
         p.cnss_employee_cdf, p.cnss_employer_cdf, p.net_pay_cdf, \
         p.employer_cost_cdf, p.created_at, \
         e.ulid as employee_ulid, e.legal_name, e.employee_number \
         FROM payslips p \
         JOIN wf_employees e ON e.id = p.employee_id \
         WHERE p.ulid = $1 AND p.tenant_id = $2",
    )
    .bind(&ulid)
    .bind(tenant_id)
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!("payslip {ulid} not found")))?;

    let run_ulid: String = sqlx::query_scalar("SELECT ulid FROM payroll_runs WHERE id = $1")
        .bind(row.get("payroll_run_id"))
        .fetch_one(&mut **conn)
        .await?;

    Ok(Json(PayslipResponse {
        ulid: row.get("ulid"),
        payroll_run_ulid: run_ulid,
        employee_ulid: row.get("employee_ulid"),
        employee_name: row.get("legal_name"),
        employee_number: row.get("employee_number"),
        version: row.get("version"),
        gross_pay: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("gross_pay_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        taxable_pay: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("taxable_pay_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        ipr_deduction: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("ipr_deduction_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        cnss_employee: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("cnss_employee_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        cnss_employer: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("cnss_employer_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        net_pay: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("net_pay_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        employer_cost: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("employer_cost_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        smig_compliant: row.get::<nexora_common::Decimal, _>("net_pay_cdf")
            >= nexora_common::money::Money::new(
                nexora_common::Decimal::new(182000, 0),
                nexora_common::money::CurrencyCode::cdf(),
            )
            .amount,
        created_at: row.get("created_at"),
    }))
}

/// List payslips for a payroll run.
#[utoipa::path(
    get,
    path = "/api/v1/payroll/runs/{run_ulid}/payslips",
    params(("run_ulid" = String, Path, description = "Payroll run ULID")),
    responses(
        (status = 200, description = "OK", body = PageResponse<PayslipResponse>),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payslips",
    security(("bearerAuth" = []))
)]
pub async fn list_payslips(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(run_ulid): Path<String>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payslip.read")?;

    let run_ulid = parse_ulid(&run_ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;

    let run_id: Uuid =
        sqlx::query_scalar("SELECT id FROM payroll_runs WHERE ulid = $1 AND tenant_id = $2")
            .bind(&run_ulid)
            .bind(tenant_id)
            .fetch_optional(&mut **conn)
            .await?
            .ok_or_else(|| NexoraError::NotFound(format!("payroll run {run_ulid} not found")))?
            .into();

    let rows = sqlx::query(
        "SELECT p.ulid, p.employee_id, p.version, \
         p.gross_pay_cdf, p.taxable_pay_cdf, p.ipr_deduction_cdf, \
         p.cnss_employee_cdf, p.cnss_employer_cdf, p.net_pay_cdf, \
         p.employer_cost_cdf, p.created_at, \
         e.ulid as employee_ulid, e.legal_name, e.employee_number \
         FROM payslips p \
         JOIN wf_employees e ON e.id = p.employee_id \
         WHERE p.payroll_run_id = $1 AND p.tenant_id = $2 \
         ORDER BY p.created_at DESC",
    )
    .bind(run_id)
    .bind(tenant_id)
    .fetch_all(&mut **conn)
    .await?;

    let items: Vec<PayslipResponse> = rows
        .into_iter()
        .map(|r| PayslipResponse {
            ulid: r.get("ulid"),
            payroll_run_ulid: run_ulid.clone(),
            employee_ulid: r.get("employee_ulid"),
            employee_name: r.get("legal_name"),
            employee_number: r.get("employee_number"),
            version: r.get("version"),
            gross_pay: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("gross_pay_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            taxable_pay: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("taxable_pay_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            ipr_deduction: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("ipr_deduction_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            cnss_employee: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("cnss_employee_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            cnss_employer: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("cnss_employer_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            net_pay: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("net_pay_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            employer_cost: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("employer_cost_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            smig_compliant: r.get::<nexora_common::Decimal, _>("net_pay_cdf")
                >= nexora_common::money::Money::new(
                    nexora_common::Decimal::new(182000, 0),
                    nexora_common::money::CurrencyCode::cdf(),
                )
                .amount,
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(PageResponse {
        items,
        total: items.len() as i64,
        page: 1,
        page_size: items.len() as i64,
    }))
}

/// Generate PDF payslips for a payroll run.
#[utoipa::path(
    post,
    path = "/api/v1/payroll/runs/{run_ulid}/generate-payslips",
    request_body = GeneratePayslipsRequest,
    responses(
        (status = 200, description = "Payslips generated", body = GeneratePayslipsResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payslips",
    security(("bearerAuth" = []))
)]
pub async fn generate_payslips(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(run_ulid): Path<String>,
    Json(req): Json<GeneratePayslipsRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payslip.deliver")?;

    let run_ulid = parse_ulid(&run_ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    let run = sqlx::query(
        "SELECT id, config_version, period_start, period_end FROM payroll_runs \
         WHERE ulid = $1 AND tenant_id = $2",
    )
    .bind(&run_ulid)
    .bind(tenant_id)
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!("payroll run {run_ulid} not found")))?;

    let run_id: Uuid = run.get("id");
    let config_version: String = row.get("config_version");
    let period_start: NaiveDate = row.get("period_start");
    let period_end: NaiveDate = row.get("period_end");

    // Get the payslips for this run.
    let payslip_rows = sqlx::query(
        "SELECT p.id, p.ulid, p.employee_id, p.gross_pay_cdf, p.net_pay_cdf, \
         p.cnss_employee_cdf, p.cnss_employer_cdf, p.ipr_deduction_cdf, \
         p.employer_cost_cdf, p.version, \
         e.ulid as employee_ulid, e.legal_name, e.employee_number \
         FROM payslips p \
         JOIN wf_employees e ON e.id = p.employee_id \
         WHERE p.payroll_run_id = $1 AND p.tenant_id = $2",
    )
    .bind(run_id)
    .bind(tenant_id)
    .fetch_all(&mut **conn)
    .await?;

    // Filter by employee IDs if specified.
    let filtered_rows: Vec<_> = if let Some(ref emp_ids) = req.employee_ids {
        let id_set: std::collections::HashSet<Uuid> = emp_ids.iter().cloned().collect();
        payslip_rows
            .into_iter()
            .filter(|r| id_set.contains(&r.get::<Uuid, _>("employee_id")))
            .collect()
    } else {
        payslip_rows
    };

    let mut generated = 0;
    for payslip_row in filtered_rows {
        let data = pdf::PayslipPdfData {
            gross_pay: nexora_common::money::Money::new(
                payslip_row.get::<nexora_common::Decimal, _>("gross_pay_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            net_pay: nexora_common::money::Money::new(
                payslip_row.get::<nexora_common::Decimal, _>("net_pay_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            cnss_employee: nexora_common::money::Money::new(
                payslip_row.get::<nexora_common::Decimal, _>("cnss_employee_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            cnss_employer: nexora_common::money::Money::new(
                payslip_row.get::<nexora_common::Decimal, _>("cnss_employer_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            ipr_deduction: nexora_common::money::Money::new(
                payslip_row.get::<nexora_common::Decimal, _>("ipr_deduction_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            employer_cost: nexora_common::money::Money::new(
                payslip_row.get::<nexora_common::Decimal, _>("employer_cost_cdf"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            employee_name: payslip_row.get("legal_name"),
            employee_number: payslip_row.get("employee_number"),
            period_start,
            period_end,
            config_version: config_version.clone(),
        };

        let pdf_bytes = pdf::generate_payslip_pdf(&data)
            .map_err(|e| NexoraError::Internal(format!("PDF generation failed: {e}")))?;

        // In production, upload to S3 here. For now, just mark as generated.
        // We'd store the object key in the payslip record.
        // For this MVP, we just count successful generations.
        generated += 1;
    }

    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_run",
        &run_ulid,
        run_id,
        "payroll.payslips.generated",
        serde_json::json!({
            "payslip_count": generated,
            "employee_count": filtered_rows.len(),
        }),
    )
    .await?;

    Ok(Json(GeneratePayslipsResponse {
        pdf_bytes_count: generated,
        employee_count: filtered_rows.len(),
    }))
}

/// Download a payslip PDF.
#[utoipa::path(
    get,
    path = "/api/v1/payroll/payslips/{ulid}/pdf",
    params(("ulid" = String, Path, description = "Payslip ULID")),
    responses(
        (status = 200, description = "OK", body = Vec<u8>, content_type = "application/pdf"),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "payslips",
    security(("bearerAuth" = []))
)]
pub async fn download_payslip_pdf(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payslip.deliver")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;

    let row = sqlx::query(
        "SELECT p.id, p.ulid, p.employee_id, p.gross_pay_cdf, p.net_pay_cdf, \
         p.cnss_employee_cdf, p.cnss_employer_cdf, p.ipr_deduction_cdf, \
         p.employer_cost_cdf, p.version, \
         pr.config_version, pr.period_start, pr.period_end, \
         e.ulid as employee_ulid, e.legal_name, e.employee_number \
         FROM payslips p \
         JOIN payroll_runs pr ON pr.id = p.payroll_run_id \
         JOIN wf_employees e ON e.id = p.employee_id \
         WHERE p.ulid = $1 AND p.tenant_id = $2",
    )
    .bind(&ulid)
    .bind(tenant_id)
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!("payslip {ulid} not found")))?;

    let config_version: String = row.get("config_version");
    let period_start: NaiveDate = row.get("period_start");
    let period_end: NaiveDate = row.get("period_end");

    let data = pdf::PayslipPdfData {
        gross_pay: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("gross_pay_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        net_pay: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("net_pay_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        cnss_employee: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("cnss_employee_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        cnss_employer: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("cnss_employer_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        ipr_deduction: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("ipr_deduction_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        employer_cost: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("employer_cost_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        employee_name: row.get("legal_name"),
        employee_number: row.get("employee_number"),
        period_start,
        period_end,
        config_version,
    };

    let pdf_bytes = pdf::generate_payslip_pdf(&data)
        .map_err(|e| NexoraError::Internal(format!("PDF generation failed: {e}")))?;

    // Return the PDF as a binary response.
    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/pdf")],
        axum::body::Body::from(pdf_bytes),
    )
        .into_response())
}

// ============================================================================
// Payroll Components (allowances/deductions)
// ============================================================================

/// Create a payroll component (allowance or deduction).
#[utoipa::path(
    post,
    path = "/api/v1/payroll/components",
    request_body = CreatePayrollComponentRequest,
    responses(
        (status = 201, description = "Created", body = PayrollComponentResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
    ),
    tag = "components",
    security(("bearerAuth" = []))
)]
pub async fn create_payroll_component(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Json(req): Json<CreatePayrollComponentRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.config.write")?;

    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    let ulid = new_ulid();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"INSERT INTO payroll_components
               (ulid, tenant_id, org_id, code, description, component_type,
                default_amount, is_taxable, is_cnssable, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10, $10)"#,
    )
    .bind(&ulid)
    .bind(tenant_id)
    .bind(org_id)
    .bind(&req.code)
    .bind(&req.description)
    .bind(req.component_type.as_str())
    .bind(req.default_amount.as_ref().map(|m| &m.amount))
    .bind(req.is_taxable)
    .bind(req.is_cnssable)
    .bind(now)
    .execute(&mut **conn)
    .await?;

    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_component",
        &ulid,
        uuid::Uuid::nil(),
        "payroll.component.created",
        serde_json::json!({
            "code": req.code,
            "component_type": req.component_type,
            "is_taxable": req.is_taxable,
            "is_cnssable": req.is_cnssable,
        }),
    )
    .await?;

    let resp = PayrollComponentResponse {
        ulid,
        code: req.code,
        description: req.description,
        component_type: req.component_type,
        default_amount: req.default_amount,
        is_taxable: req.is_taxable,
        is_cnssable: req.is_cnssable,
        is_active: true,
        created_at: now,
        updated_at: now,
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

/// Get a single payroll component by ULID.
#[utoipa::path(
    get,
    path = "/api/v1/payroll/components/{ulid}",
    params(("ulid" = String, Path, description = "Component ULID")),
    responses(
        (status = 200, description = "OK", body = PayrollComponentResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "components",
    security(("bearerAuth" = []))
)]
pub async fn get_payroll_component(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.config.read")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;

    let row = sqlx::query(
        "SELECT ulid, code, description, component_type, default_amount, \
         is_taxable, is_cnssable, is_active, created_at, updated_at \
         FROM payroll_components WHERE ulid = $1 AND tenant_id = $2",
    )
    .bind(&ulid)
    .bind(tenant_id)
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!("component {ulid} not found")))?;

    let resp = PayrollComponentResponse {
        ulid: row.get("ulid"),
        code: row.get("code"),
        description: row.get("description"),
        component_type: ComponentType::from_str(&row.get::<String, _>("component_type"))
            .unwrap_or(ComponentType::Allowance),
        default_amount: row.try_get("default_amount").unwrap_or(None),
        is_taxable: row.get("is_taxable"),
        is_cnssable: row.get("is_cnssable"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };
    Ok(Json(resp))
}

/// List payroll components for the current tenant.
#[utoipa::path(
    get,
    path = "/api/v1/payroll/components",
    responses(
        (status = 200, description = "OK", body = PageResponse<PayrollComponentResponse>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "components",
    security(("bearerAuth" = []))
)]
pub async fn list_payroll_components(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_any_permission(&["payroll.config.read", "payroll.config.write"])?;

    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;

    let rows = sqlx::query(
        "SELECT ulid, code, description, component_type, default_amount, \
         is_taxable, is_cnssable, is_active, created_at, updated_at \
         FROM payroll_components WHERE tenant_id = $1 ORDER BY code",
    )
    .bind(tenant_id)
    .fetch_all(&mut **conn)
    .await?;

    let items: Vec<PayrollComponentResponse> = rows
        .into_iter()
        .map(|r| PayrollComponentResponse {
            ulid: r.get("ulid"),
            code: r.get("code"),
            description: r.get("description"),
            component_type: ComponentType::from_str(&r.get::<String, _>("component_type"))
                .unwrap_or(ComponentType::Allowance),
            default_amount: r.try_get("default_amount").unwrap_or(None),
            is_taxable: r.get("is_taxable"),
            is_cnssable: r.get("is_cnssable"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
        .collect();

    Ok(Json(PageResponse {
        items,
        total: items.len() as i64,
        page: 1,
        page_size: items.len() as i64,
    }))
}

/// Update a payroll component.
#[utoipa::path(
    put,
    path = "/api/v1/payroll/components/{ulid}",
    request_body = UpdatePayrollComponentRequest,
    responses(
        (status = 200, description = "Updated", body = PayrollComponentResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "components",
    security(("bearerAuth" = []))
)]
pub async fn update_payroll_component(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
    Json(req): Json<UpdatePayrollComponentRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.config.write")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    let now = chrono::Utc::now();

    sqlx::query(
        r#"UPDATE payroll_components SET
               code = COALESCE($2, code),
               description = COALESCE($3, description),
               component_type = COALESCE($4, component_type),
               default_amount = COALESCE($5, default_amount),
               is_taxable = COALESCE($6, is_taxable),
               is_cnssable = COALESCE($7, is_cnssable),
               is_active = COALESCE($8, is_active),
               updated_at = $9
           WHERE ulid = $1 AND tenant_id = $10"#,
    )
    .bind(&ulid)
    .bind(req.code)
    .bind(req.description)
    .bind(req.component_type.as_str())
    .bind(req.default_amount.as_ref().map(|m| &m.amount))
    .bind(req.is_taxable)
    .bind(req.is_cnssable)
    .bind(req.is_active)
    .bind(now)
    .bind(tenant_id)
    .execute(&mut **conn)
    .await?;

    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_component",
        &ulid,
        uuid::Uuid::nil(),
        "payroll.component.updated",
        serde_json::json!({
            "code": req.code,
            "description": req.description,
            "component_type": req.component_type,
            "is_taxable": req.is_taxable,
            "is_cnssable": req.is_cnssable,
            "is_active": req.is_active,
        }),
    )
    .await?;

    // Fetch updated component.
    let row = sqlx::query(
        "SELECT ulid, code, description, component_type, default_amount, \
         is_taxable, is_cnssable, is_active, created_at, updated_at \
         FROM payroll_components WHERE ulid = $1 AND tenant_id = $2",
    )
    .bind(&ulid)
    .bind(tenant_id)
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!("component {ulid} not found")))?;

    let resp = PayrollComponentResponse {
        ulid: row.get("ulid"),
        code: row.get("code"),
        description: row.get("description"),
        component_type: ComponentType::from_str(&row.get::<String, _>("component_type"))
            .unwrap_or(ComponentType::Allowance),
        default_amount: row.try_get("default_amount").unwrap_or(None),
        is_taxable: row.get("is_taxable"),
        is_cnssable: row.get("is_cnssable"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };
    Ok(Json(resp))
}

/// Delete (deactivate) a payroll component.
#[utoipa::path(
    delete,
    path = "/api/v1/payroll/components/{ulid}",
    params(("ulid" = String, Path, description = "Component ULID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "components",
    security(("bearerAuth" = []))
)]
pub async fn delete_payroll_component(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(ulid): Path<String>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.config.write")?;

    let ulid = parse_ulid(&ulid)?;
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    let result = sqlx::query(
        "UPDATE payroll_components SET is_active = false, updated_at = NOW() \
         WHERE ulid = $1 AND tenant_id = $2",
    )
    .bind(&ulid)
    .bind(tenant_id)
    .execute(&mut **conn)
    .await?;

    if result.rows_affected() == 0 {
        return Err(NexoraError::NotFound(format!("component {ulid} not found")));
    }

    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_component",
        &ulid,
        ulid,
        "payroll.component.deleted",
        serde_json::json!({
            "component_ulid": ulid,
        }),
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Payroll Configurations
// ============================================================================

/// Create a payroll configuration version.
#[utoipa::path(
    post,
    path = "/api/v1/payroll/configs",
    request_body = CreatePayrollConfigRequest,
    responses(
        (status = 201, description = "Created", body = PayrollConfigResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
    ),
    tag = "configs",
    security(("bearerAuth" = []))
)]
pub async fn create_payroll_config(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Json(req): Json<CreatePayrollConfigRequest>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.config.write")?;

    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;
    let org_id = resolve_org(&auth, &mut **conn).await?;

    let now = chrono::Utc::now();

    // Insert into country_tax_configs.
    let config_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO country_tax_configs
               (tenant_id, org_id, country_code, config_version,
                ipr_bands, cnss_employee_rate, cnss_employer_rate,
                cnss_ceiling, smig_daily, smig_monthly_26, smig_monthly_30,
                is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, true, $12, $12)
           RETURNING id"#,
    )
    .bind(tenant_id)
    .bind(org_id)
    .bind(&req.country_code)
    .bind(&req.config_version)
    .bind(serde_json::to_value(&req.ipr_brackets).unwrap())
    .bind(req.cnss.employee_rate)
    .bind(req.cnss.employer_rate)
    .bind(req.cnss.ceiling)
    .bind(req.smig.daily)
    .bind(req.smig.monthly_26)
    .bind(req.smig.monthly_30)
    .bind(now)
    .fetch_one(&mut **conn)
    .await?;

    emit_payroll_event(
        &mut **conn,
        state.config.as_ref(),
        &auth,
        tenant_id,
        org_id,
        "payroll_configuration",
        &req.config_version,
        config_id,
        "payroll.config.created",
        serde_json::json!({
            "config_version": req.config_version,
            "country_code": req.country_code,
            "ipr_bracket_count": req.ipr_brackets.len(),
        }),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(PayrollConfigResponse {
            config_version: req.config_version,
            country_code: req.country_code,
            ipr_bands: serde_json::to_value(&req.ipr_brackets).unwrap(),
            cnss_employee_rate: req.cnss.employee_rate,
            cnss_employer_rate: req.cnss.employer_rate,
            cnss_ceiling: nexora_common::money::Money::new(
                req.cnss.ceiling,
                nexora_common::money::CurrencyCode::cdf(),
            ),
            smig_daily: nexora_common::money::Money::new(
                req.smig.daily,
                nexora_common::money::CurrencyCode::cdf(),
            ),
            smig_monthly_26: nexora_common::money::Money::new(
                req.smig.monthly_26,
                nexora_common::money::CurrencyCode::cdf(),
            ),
            smig_monthly_30: nexora_common::money::Money::new(
                req.smig.monthly_30,
                nexora_common::money::CurrencyCode::cdf(),
            ),
            is_active: true,
        }),
    ))
}

/// Get a payroll configuration by version.
#[utoipa::path(
    get,
    path = "/api/v1/payroll/configs/{version}",
    params(("config_version" = String, Path, description = "Config version")),
    responses(
        (status = 200, description = "OK", body = PayrollConfigResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "configs",
    security(("bearerAuth" = []))
)]
pub async fn get_payroll_config(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(version): Path<String>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.config.read")?;

    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;

    let r = sqlx::query(
        "SELECT config_version, country_code, ipr_bands, cnss_employee_rate, \
         cnss_employer_rate, cnss_ceiling, smig_daily, smig_monthly_26, \
         smig_monthly_30, is_active \
         FROM country_tax_configs \
         WHERE tenant_id = $1 AND config_version = $2 AND is_active = true",
    )
    .bind(tenant_id)
    .bind(&version)
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound(format!("config version {version} not found")))?;

    Ok(Json(PayrollConfigResponse {
        config_version: r.get("config_version"),
        country_code: r.get("country_code"),
        ipr_bands: r.get("ipr_bands"),
        cnss_employee_rate: r.get("cnss_employee_rate"),
        cnss_employer_rate: r.get("cnss_employer_rate"),
        cnss_ceiling: nexora_common::money::Money::new(
            r.get::<nexora_common::Decimal, _>("cnss_ceiling"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        smig_daily: nexora_common::money::Money::new(
            r.get::<nexora_common::Decimal, _>("smig_daily"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        smig_monthly_26: nexora_common::money::Money::new(
            r.get::<nexora_common::Decimal, _>("smig_monthly_26"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        smig_monthly_30: nexora_common::money::Money::new(
            r.get::<nexora_common::Decimal, _>("smig_monthly_30"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        is_active: r.get("is_active"),
    }))
}

/// List payroll configurations for the current tenant.
#[utoipa::path(
    get,
    path = "/api/v1/payroll/configs",
    responses(
        (status = 200, description = "OK", body = PageResponse<PayrollConfigResponse>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "configs",
    security(("bearerAuth" = []))
)]
pub async fn list_payroll_configs(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
) -> NexoraResult<impl IntoResponse> {
    auth.require_permission("payroll.config.read")?;

    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant(&auth, &mut **conn).await?;

    let rows = sqlx::query(
        "SELECT config_version, country_code, ipr_bands, cnss_employee_rate, \
         cnss_employer_rate, cnss_ceiling, smig_daily, smig_monthly_26, \
         smig_monthly_30, is_active \
         FROM country_tax_configs \
         WHERE tenant_id = $1 AND is_active = true ORDER BY config_version",
    )
    .bind(tenant_id)
    .fetch_all(&mut **conn)
    .await?;

    let total_count = rows.len() as i64;
    let items: Vec<PayrollConfigResponse> = rows
        .into_iter()
        .map(|r| PayrollConfigResponse {
            config_version: r.get("config_version"),
            country_code: r.get("country_code"),
            ipr_bands: r.get("ipr_bands"),
            cnss_employee_rate: r.get("cnss_employee_rate"),
            cnss_employer_rate: r.get("cnss_employer_rate"),
            cnss_ceiling: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("cnss_ceiling"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            smig_daily: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("smig_daily"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            smig_monthly_26: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("smig_monthly_26"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            smig_monthly_30: nexora_common::money::Money::new(
                r.get::<nexora_common::Decimal, _>("smig_monthly_30"),
                nexora_common::money::CurrencyCode::cdf(),
            ),
            is_active: r.get("is_active"),
        })
        .collect();

    Ok(Json(PageResponse {
        items,
        total: total_count,
        page: 1,
        page_size: items.len() as i64,
    }))
}

// ============================================================================
// Helper: build a PayrollRunResponse from a DB row
// ============================================================================

async fn get_payroll_run_response_from_row(
    conn: &mut sqlx::PgConnection,
    run_id: Uuid,
    _tenant_id: Uuid,
) -> NexoraResult<PayrollRunResponse> {
    let row = sqlx::query(
        "SELECT ulid, config_version, period_start, period_end, status, \
         total_gross_cdf, total_net_cdf, total_employer_cost_cdf, employee_count, \
         reviewed_at, approved_at, locked_at, created_at, updated_at \
         FROM payroll_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| NexoraError::NotFound("payroll run not found".to_string()))?;

    let status_str: String = row.get("status");
    Ok(PayrollRunResponse {
        ulid: row.get("ulid"),
        config_version: row.get("config_version"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        status: PayrollRunStatus::from_str(&status_str).unwrap_or(PayrollRunStatus::Draft),
        total_gross: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("total_gross_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        total_net: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("total_net_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        total_employer_cost: nexora_common::money::Money::new(
            row.get::<nexora_common::Decimal, _>("total_employer_cost_cdf"),
            nexora_common::money::CurrencyCode::cdf(),
        ),
        employee_count: row.get("employee_count"),
        reviewed_at: row.try_get("reviewed_at").unwrap_or(None),
        approved_at: row.try_get("approved_at").unwrap_or(None),
        locked_at: row.try_get("locked_at").unwrap_or(None),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
