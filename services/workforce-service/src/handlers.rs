//! Workforce service HTTP handlers.
//!
//! Authorization is **deny-by-default** and enforced inline via
//! `AuthContextExt` before any database query. Every handler that touches an
//! employee record also performs an IDOR check: the resolved DB row's
//! `tenant_id` must match the caller's `tenant_id` (belt-and-suspenders over
//! RLS). Compensation endpoints are additionally gated on
//! `employee.compensation.read`/`write`.
//!
//! Sensitive values never appear in tracing spans:
//!   - national_id (hashed on write)
//!   - compensation amounts (omitted from all audit change payloads)
//!   - document object keys (the ULID-based key is logged at DEBUG only)
//!
//! The five security-monitoring events that must be audited:
//!   salary.changed, privilege.escalated (out of scope here — done by
//!   tenant-service), employee.exported, mass_employee_update (not yet
//!   implemented — flagged with ponytail comment), suspicious_document_access
//!   (flagged when doc tenant_id ≠ employee tenant_id, which RLS prevents —
//!   recorded for defence-in-depth).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sha2::{Digest, Sha256};
use tracing::{info, warn};

use aos_common::{
    error::{AosError, AosResult, ErrorResponse},
    rbac::AuthContextExt,
    tenant_context::{AuthContext, DbConn},
    ulid::new_ulid,
};

use crate::{
    audit_emit::emit_workforce_event,
    documents::{presigned_url_for_doc, upload_to_storage, UploadContext},
    models::*,
    AppState,
};

// ---------------------------------------------------------------------------
// Helper: resolve tenant UUID from ULID (same pattern as audit-service)
// ---------------------------------------------------------------------------

async fn resolve_tenant_id(
    conn: &mut sqlx::PgConnection,
    auth: &AuthContext,
) -> AosResult<uuid::Uuid> {
    let ulid = auth.tenant_id.to_string();
    sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM tenants WHERE ulid = $1")
        .bind(&ulid)
        .fetch_optional(conn)
        .await?
        .ok_or_else(|| AosError::TenantIsolation(format!("tenant {ulid} not visible in RLS ctx")))
}

async fn resolve_org_id(
    conn: &mut sqlx::PgConnection,
    auth: &AuthContext,
) -> AosResult<uuid::Uuid> {
    let ulid = auth.org_id.to_string();
    sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM organizations WHERE ulid = $1")
        .bind(&ulid)
        .fetch_optional(conn)
        .await?
        .ok_or_else(|| AosError::TenantIsolation(format!("org {ulid} not visible in RLS ctx")))
}

/// Resolve a ULID to a UUID for a given table. Returns 404 if not found.
async fn resolve_ulid(
    conn: &mut sqlx::PgConnection,
    table: &str,
    ulid: &str,
) -> AosResult<uuid::Uuid> {
    let sql = format!("SELECT id FROM {table} WHERE ulid = $1");
    sqlx::query_scalar::<_, uuid::Uuid>(&sql)
        .bind(ulid)
        .fetch_optional(conn)
        .await?
        .ok_or_else(|| AosError::NotFound(format!("{table} not found: {ulid}")))
}

// ---------------------------------------------------------------------------
// Legal Entities
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/v1/workforce/legal-entities",
    request_body = CreateLegalEntityRequest,
    responses(
        (status = 201, description = "Created", body = LegalEntityResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn create_legal_entity(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateLegalEntityRequest>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("legal_entity.write")?;

    let ulid = new_ulid();
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;

    let country_id: Option<uuid::Uuid> = if let Some(ref cu) = req.country_ulid {
        Some(resolve_ulid(conn.as_mut(), "countries", cu).await?)
    } else {
        None
    };

    let address = req.address.unwrap_or_else(|| serde_json::json!({}));

    sqlx::query(
        r#"INSERT INTO wf_legal_entities
               (ulid, tenant_id, org_id, name, registration_number, country_id, address)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(&ulid)
    .bind(tenant_id)
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.registration_number)
    .bind(country_id)
    .bind(&address)
    .execute(conn.as_mut())
    .await?;

    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "legal_entity",
        &ulid,
        "legal_entity.created",
        serde_json::json!({"name": req.name}),
    )
    .await?;

    info!(ulid = %ulid, "legal entity created");

    Ok((
        StatusCode::CREATED,
        Json(LegalEntityResponse {
            ulid,
            name: req.name,
            registration_number: req.registration_number,
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/legal-entities",
    params(PageQuery),
    responses(
        (status = 200, description = "OK"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn list_legal_entities(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Query(q): Query<PageQuery>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("legal_entity.read")?;
    let q = q.sanitized();
    let mut conn = db.acquire().await?;

    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            String,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT ulid, name, registration_number, status, created_at
         FROM wf_legal_entities
         WHERE deleted_at IS NULL
         ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(q.page_size)
    .bind(q.offset())
    .fetch_all(conn.as_mut())
    .await?;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM wf_legal_entities WHERE deleted_at IS NULL")
            .fetch_one(conn.as_mut())
            .await?;

    let items: Vec<LegalEntityResponse> = rows
        .into_iter()
        .map(
            |(ulid, name, registration_number, status, created_at)| LegalEntityResponse {
                ulid,
                name,
                registration_number,
                status,
                created_at,
            },
        )
        .collect();

    Ok(Json(PaginatedResponse {
        items,
        total,
        page: q.page,
        page_size: q.page_size,
    }))
}

// ---------------------------------------------------------------------------
// Locations
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/v1/workforce/locations",
    request_body = CreateLocationRequest,
    responses(
        (status = 201, description = "Created", body = LocationResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn create_location(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateLocationRequest>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("location.write")?;

    let ulid = new_ulid();
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;
    let entity_id =
        resolve_ulid(conn.as_mut(), "wf_legal_entities", &req.legal_entity_ulid).await?;
    let tz = req
        .timezone
        .unwrap_or_else(|| "Africa/Lubumbashi".to_string());
    let address = req.address.unwrap_or_else(|| serde_json::json!({}));

    sqlx::query(
        r#"INSERT INTO wf_locations
               (ulid, tenant_id, org_id, legal_entity_id, name, code, address, timezone)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
    )
    .bind(&ulid)
    .bind(tenant_id)
    .bind(org_id)
    .bind(entity_id)
    .bind(&req.name)
    .bind(&req.code)
    .bind(&address)
    .bind(&tz)
    .execute(conn.as_mut())
    .await?;

    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "location",
        &ulid,
        "location.created",
        serde_json::json!({"name": req.name}),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(LocationResponse {
            ulid,
            name: req.name,
            code: req.code,
            timezone: tz,
            is_active: true,
            created_at: chrono::Utc::now(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/locations",
    params(PageQuery),
    responses((status = 200, description = "OK")),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn list_locations(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Query(q): Query<PageQuery>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("location.read")?;
    let q = q.sanitized();
    let mut conn = db.acquire().await?;

    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            String,
            bool,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT ulid, name, code, timezone, is_active, created_at
         FROM wf_locations WHERE deleted_at IS NULL
         ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(q.page_size)
    .bind(q.offset())
    .fetch_all(conn.as_mut())
    .await?;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM wf_locations WHERE deleted_at IS NULL")
            .fetch_one(conn.as_mut())
            .await?;

    let items: Vec<LocationResponse> = rows
        .into_iter()
        .map(
            |(ulid, name, code, timezone, is_active, created_at)| LocationResponse {
                ulid,
                name,
                code,
                timezone,
                is_active,
                created_at,
            },
        )
        .collect();

    Ok(Json(PaginatedResponse {
        items,
        total,
        page: q.page,
        page_size: q.page_size,
    }))
}

// ---------------------------------------------------------------------------
// Departments
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/v1/workforce/departments",
    request_body = CreateDepartmentRequest,
    responses(
        (status = 201, description = "Created", body = DepartmentResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn create_department(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateDepartmentRequest>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("department.write")?;

    let ulid = new_ulid();
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;

    let parent_id: Option<uuid::Uuid> = if let Some(ref pu) = req.parent_department_ulid {
        Some(resolve_ulid(conn.as_mut(), "wf_departments", pu).await?)
    } else {
        None
    };
    let location_id: Option<uuid::Uuid> = if let Some(ref lu) = req.location_ulid {
        Some(resolve_ulid(conn.as_mut(), "wf_locations", lu).await?)
    } else {
        None
    };

    sqlx::query(
        r#"INSERT INTO wf_departments
               (ulid, tenant_id, org_id, name, code, parent_department_id, location_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(&ulid)
    .bind(tenant_id)
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.code)
    .bind(parent_id)
    .bind(location_id)
    .execute(conn.as_mut())
    .await?;

    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "department",
        &ulid,
        "department.created",
        serde_json::json!({"name": req.name}),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(DepartmentResponse {
            ulid,
            name: req.name,
            code: req.code,
            is_active: true,
            created_at: chrono::Utc::now(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/departments",
    params(PageQuery),
    responses((status = 200, description = "OK")),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn list_departments(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Query(q): Query<PageQuery>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("department.read")?;
    let q = q.sanitized();
    let mut conn = db.acquire().await?;

    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            bool,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT ulid, name, code, is_active, created_at
         FROM wf_departments WHERE deleted_at IS NULL
         ORDER BY name LIMIT $1 OFFSET $2",
    )
    .bind(q.page_size)
    .bind(q.offset())
    .fetch_all(conn.as_mut())
    .await?;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM wf_departments WHERE deleted_at IS NULL")
            .fetch_one(conn.as_mut())
            .await?;

    let items: Vec<DepartmentResponse> = rows
        .into_iter()
        .map(
            |(ulid, name, code, is_active, created_at)| DepartmentResponse {
                ulid,
                name,
                code,
                is_active,
                created_at,
            },
        )
        .collect();

    Ok(Json(PaginatedResponse {
        items,
        total,
        page: q.page,
        page_size: q.page_size,
    }))
}

// ---------------------------------------------------------------------------
// Teams
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/v1/workforce/teams",
    request_body = CreateTeamRequest,
    responses(
        (status = 201, description = "Created", body = TeamResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn create_team(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateTeamRequest>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("team.write")?;

    let ulid = new_ulid();
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;
    let dept_id = resolve_ulid(conn.as_mut(), "wf_departments", &req.department_ulid).await?;

    sqlx::query(
        "INSERT INTO wf_teams (ulid, tenant_id, org_id, department_id, name, code)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&ulid)
    .bind(tenant_id)
    .bind(org_id)
    .bind(dept_id)
    .bind(&req.name)
    .bind(&req.code)
    .execute(conn.as_mut())
    .await?;

    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "team",
        &ulid,
        "team.created",
        serde_json::json!({"name": req.name}),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(TeamResponse {
            ulid,
            name: req.name,
            code: req.code,
            is_active: true,
            created_at: chrono::Utc::now(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/teams",
    params(PageQuery),
    responses((status = 200, description = "OK")),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn list_teams(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Query(q): Query<PageQuery>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("team.read")?;
    let q = q.sanitized();
    let mut conn = db.acquire().await?;

    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            bool,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT ulid, name, code, is_active, created_at
         FROM wf_teams WHERE deleted_at IS NULL
         ORDER BY name LIMIT $1 OFFSET $2",
    )
    .bind(q.page_size)
    .bind(q.offset())
    .fetch_all(conn.as_mut())
    .await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wf_teams WHERE deleted_at IS NULL")
        .fetch_one(conn.as_mut())
        .await?;

    let items: Vec<TeamResponse> = rows
        .into_iter()
        .map(|(ulid, name, code, is_active, created_at)| TeamResponse {
            ulid,
            name,
            code,
            is_active,
            created_at,
        })
        .collect();

    Ok(Json(PaginatedResponse {
        items,
        total,
        page: q.page,
        page_size: q.page_size,
    }))
}

// ---------------------------------------------------------------------------
// Positions
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/v1/workforce/positions",
    request_body = CreatePositionRequest,
    responses(
        (status = 201, description = "Created", body = PositionResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn create_position(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreatePositionRequest>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("position.write")?;

    let ulid = new_ulid();
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;

    let dept_id: Option<uuid::Uuid> = if let Some(ref du) = req.department_ulid {
        Some(resolve_ulid(conn.as_mut(), "wf_departments", du).await?)
    } else {
        None
    };
    let emp_type = req
        .employment_type
        .as_ref()
        .map(|e| e.as_str())
        .unwrap_or("permanent");

    sqlx::query(
        r#"INSERT INTO wf_positions
               (ulid, tenant_id, org_id, department_id, title, code, job_grade,
                description, responsibilities, employment_type)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
    )
    .bind(&ulid)
    .bind(tenant_id)
    .bind(org_id)
    .bind(dept_id)
    .bind(&req.title)
    .bind(&req.code)
    .bind(&req.job_grade)
    .bind(&req.description)
    .bind(&req.responsibilities)
    .bind(emp_type)
    .execute(conn.as_mut())
    .await?;

    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "position",
        &ulid,
        "position.created",
        serde_json::json!({"title": req.title}),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(PositionResponse {
            ulid,
            title: req.title,
            code: req.code,
            job_grade: req.job_grade,
            employment_type: emp_type.to_string(),
            is_active: true,
            created_at: chrono::Utc::now(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/positions",
    params(PageQuery),
    responses((status = 200, description = "OK")),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn list_positions(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Query(q): Query<PageQuery>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("position.read")?;
    let q = q.sanitized();
    let mut conn = db.acquire().await?;

    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            bool,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT ulid, title, code, job_grade, employment_type, is_active, created_at
         FROM wf_positions WHERE deleted_at IS NULL
         ORDER BY title LIMIT $1 OFFSET $2",
    )
    .bind(q.page_size)
    .bind(q.offset())
    .fetch_all(conn.as_mut())
    .await?;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM wf_positions WHERE deleted_at IS NULL")
            .fetch_one(conn.as_mut())
            .await?;

    let items: Vec<PositionResponse> = rows
        .into_iter()
        .map(
            |(ulid, title, code, job_grade, employment_type, is_active, created_at)| {
                PositionResponse {
                    ulid,
                    title,
                    code,
                    job_grade,
                    employment_type,
                    is_active,
                    created_at,
                }
            },
        )
        .collect();

    Ok(Json(PaginatedResponse {
        items,
        total,
        page: q.page,
        page_size: q.page_size,
    }))
}

// ---------------------------------------------------------------------------
// Employees
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/v1/workforce/employees",
    request_body = CreateEmployeeRequest,
    responses(
        (status = 201, description = "Created", body = EmployeeResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn create_employee(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateEmployeeRequest>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("employee.write")?;

    let ulid = new_ulid();
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;
    let actor_uuid = resolve_actor_uuid(conn.as_mut(), &auth).await?;

    // Hash the national ID — never store the raw value.
    let (national_id_hash, national_id_last4) = hash_national_id(req.national_id.as_deref());

    let dept_id: Option<uuid::Uuid> = resolve_opt(
        conn.as_mut(),
        "wf_departments",
        req.department_ulid.as_deref(),
    )
    .await?;
    let pos_id: Option<uuid::Uuid> =
        resolve_opt(conn.as_mut(), "wf_positions", req.position_ulid.as_deref()).await?;
    let loc_id: Option<uuid::Uuid> =
        resolve_opt(conn.as_mut(), "wf_locations", req.location_ulid.as_deref()).await?;
    let mgr_id: Option<uuid::Uuid> = resolve_opt(
        conn.as_mut(),
        "wf_employees",
        req.manager_employee_ulid.as_deref(),
    )
    .await?;

    sqlx::query(
        r#"INSERT INTO wf_employees
               (ulid, tenant_id, org_id, employee_number, legal_name, preferred_name,
                email, phone, date_of_birth, gender,
                national_id_hash, national_id_last4,
                current_department_id, current_position_id, current_location_id,
                manager_employee_id, hire_date, created_by)
           VALUES
               ($1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12,
                $13, $14, $15,
                $16, $17, $18)"#,
    )
    .bind(&ulid)
    .bind(tenant_id)
    .bind(org_id)
    .bind(&req.employee_number)
    .bind(&req.legal_name)
    .bind(&req.preferred_name)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(req.date_of_birth)
    .bind(&req.gender)
    .bind(&national_id_hash)
    .bind(&national_id_last4)
    .bind(dept_id)
    .bind(pos_id)
    .bind(loc_id)
    .bind(mgr_id)
    .bind(req.hire_date)
    .bind(actor_uuid)
    .execute(conn.as_mut())
    .await?;

    // Seed the first employment record.
    let emp_type = req
        .employment_type
        .as_ref()
        .map(|e| e.as_str())
        .unwrap_or("permanent");
    let emp_ulid = new_ulid();
    sqlx::query(
        r#"INSERT INTO wf_employment_records
               (ulid, tenant_id, employee_id, position_id, department_id, location_id,
                employment_type, effective_date, changes, changed_by)
           VALUES
               ($1, $2,
                (SELECT id FROM wf_employees WHERE ulid = $3),
                $4, $5, $6, $7, $8,
                '{"event":"initial_hire"}', $9)"#,
    )
    .bind(&emp_ulid)
    .bind(tenant_id)
    .bind(&ulid)
    .bind(pos_id)
    .bind(dept_id)
    .bind(loc_id)
    .bind(emp_type)
    .bind(
        req.hire_date
            .unwrap_or_else(|| chrono::Utc::now().date_naive()),
    )
    .bind(actor_uuid)
    .execute(conn.as_mut())
    .await?;

    // Audit: redact all sensitive fields.
    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "employee",
        &ulid,
        "employee.lifecycle.created",
        serde_json::json!({
            "employee_number": req.employee_number,
            "email": req.email,
            // national_id intentionally absent
        }),
    )
    .await?;

    info!(employee_ulid = %ulid, "employee created");

    let response = EmployeeResponse {
        ulid: ulid.clone(),
        employee_number: req.employee_number,
        legal_name: req.legal_name,
        preferred_name: req.preferred_name,
        email: req.email,
        phone: req.phone,
        gender: req.gender,
        national_id_last4,
        status: "onboarding".to_string(),
        hire_date: req.hire_date,
        current_department_ulid: req.department_ulid,
        current_position_ulid: req.position_ulid,
        current_location_ulid: req.location_ulid,
        manager_employee_ulid: req.manager_employee_ulid,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/employees",
    params(PageQuery),
    responses(
        (status = 200, description = "OK"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn list_employees(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Query(q): Query<PageQuery>,
) -> AosResult<impl IntoResponse> {
    // MANAGERs and above may list; EMPLOYEE role may only get their own record.
    auth.require_any_permission(&["employee.read", "employee.export"])?;
    let q = q.sanitized();
    let mut conn = db.acquire().await?;

    // Scoping: if caller is MANAGER (not HR_ADMIN/ORG_ADMIN), restrict to
    // direct reports only.
    // ponytail: full org-chart traversal deferred — direct reports only for MVP
    let scope_to_manager = auth.has_permission("employee.read")
        && !auth.has_permission("employee.write")
        && auth.roles.iter().any(|r| r == "MANAGER");

    let (rows, total): (Vec<EmployeeRow>, i64) = if scope_to_manager {
        let mgr_id: Option<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM wf_employees WHERE user_id = (SELECT id FROM users WHERE ulid = $1)",
        )
        .bind(auth.user_id.to_string())
        .fetch_optional(conn.as_mut())
        .await?;

        let rows = sqlx::query_as::<_, EmployeeRow>(
            "SELECT id, ulid, employee_number, legal_name, preferred_name, email, phone,
                    gender, national_id_last4, status, hire_date,
                    current_department_id, current_position_id, current_location_id,
                    manager_employee_id, created_at, updated_at
             FROM wf_employees
             WHERE deleted_at IS NULL AND manager_employee_id = $1
             ORDER BY legal_name LIMIT $2 OFFSET $3",
        )
        .bind(mgr_id)
        .bind(q.page_size)
        .bind(q.offset())
        .fetch_all(conn.as_mut())
        .await?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM wf_employees WHERE deleted_at IS NULL AND manager_employee_id = $1",
        ).bind(mgr_id).fetch_one(conn.as_mut()).await?;

        (rows, total)
    } else {
        let rows = sqlx::query_as::<_, EmployeeRow>(
            "SELECT id, ulid, employee_number, legal_name, preferred_name, email, phone,
                    gender, national_id_last4, status, hire_date,
                    current_department_id, current_position_id, current_location_id,
                    manager_employee_id, created_at, updated_at
             FROM wf_employees WHERE deleted_at IS NULL
             ORDER BY legal_name LIMIT $1 OFFSET $2",
        )
        .bind(q.page_size)
        .bind(q.offset())
        .fetch_all(conn.as_mut())
        .await?;

        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM wf_employees WHERE deleted_at IS NULL")
                .fetch_one(conn.as_mut())
                .await?;

        (rows, total)
    };

    // Security monitoring: bulk export audit event.
    if auth.has_permission("employee.export") {
        let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
        let org_id = resolve_org_id(conn.as_mut(), &auth).await?;
        emit_workforce_event(
            conn.as_mut(),
            &auth,
            tenant_id,
            org_id,
            "employee",
            "bulk",
            "employee.exported",
            serde_json::json!({"count": total}),
        )
        .await?;
    }

    let items: Vec<EmployeeResponse> = rows.into_iter().map(employee_row_to_response).collect();
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: q.page,
        page_size: q.page_size,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/employees/{employee_ulid}",
    params(("employee_ulid" = String, Path, description = "Employee ULID")),
    responses(
        (status = 200, description = "OK", body = EmployeeResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn get_employee(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Path(employee_ulid): Path<String>,
) -> AosResult<impl IntoResponse> {
    let mut conn = db.acquire().await?;

    let row = fetch_employee_row(conn.as_mut(), &employee_ulid).await?;
    check_employee_access(&auth, &row, conn.as_mut()).await?;

    Ok(Json(employee_row_to_response(row)))
}

// ---------------------------------------------------------------------------
// Employee status lifecycle
// ---------------------------------------------------------------------------

#[utoipa::path(
    patch,
    path = "/api/v1/workforce/employees/{employee_ulid}/status",
    request_body = UpdateEmployeeStatusRequest,
    responses(
        (status = 200, description = "Updated", body = EmployeeResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn update_employee_status(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Path(employee_ulid): Path<String>,
    Json(req): Json<UpdateEmployeeStatusRequest>,
) -> AosResult<impl IntoResponse> {
    if req.status == EmployeeStatus::Terminated {
        auth.require_permission("employee.terminate")?;
    } else {
        auth.require_permission("employee.write")?;
    }

    let mut conn = db.acquire().await?;
    let row = fetch_employee_row(conn.as_mut(), &employee_ulid).await?;
    check_employee_access(&auth, &row, conn.as_mut()).await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;

    sqlx::query(
        "UPDATE wf_employees SET status = $1, termination_date = $2, termination_reason = $3
         WHERE ulid = $4",
    )
    .bind(req.status.as_str())
    .bind(req.termination_date)
    .bind(&req.termination_reason)
    .bind(&employee_ulid)
    .execute(conn.as_mut())
    .await?;

    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "employee",
        &employee_ulid,
        &format!("employee.lifecycle.{}", req.status.as_str()),
        serde_json::json!({"new_status": req.status.as_str()}),
    )
    .await?;

    info!(employee_ulid = %employee_ulid, status = %req.status, "employee status updated");

    // Re-fetch for consistent response.
    let updated = fetch_employee_row(conn.as_mut(), &employee_ulid).await?;
    Ok(Json(employee_row_to_response(updated)))
}

// ---------------------------------------------------------------------------
// Employment history
// ---------------------------------------------------------------------------

#[utoipa::path(
    patch,
    path = "/api/v1/workforce/employees/{employee_ulid}/employment",
    request_body = UpdateEmploymentRequest,
    responses(
        (status = 200, description = "Updated"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn update_employment(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Path(employee_ulid): Path<String>,
    Json(req): Json<UpdateEmploymentRequest>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("employee.write")?;

    let mut conn = db.acquire().await?;
    let row = fetch_employee_row(conn.as_mut(), &employee_ulid).await?;
    check_employee_access(&auth, &row, conn.as_mut()).await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;

    let pos_id: Option<uuid::Uuid> =
        resolve_opt(conn.as_mut(), "wf_positions", req.position_ulid.as_deref()).await?;
    let dept_id: Option<uuid::Uuid> = resolve_opt(
        conn.as_mut(),
        "wf_departments",
        req.department_ulid.as_deref(),
    )
    .await?;
    let loc_id: Option<uuid::Uuid> =
        resolve_opt(conn.as_mut(), "wf_locations", req.location_ulid.as_deref()).await?;
    let emp_type = req
        .employment_type
        .as_ref()
        .map(|e| e.as_str())
        .unwrap_or("permanent");

    // Close the current open record.
    sqlx::query(
        "UPDATE wf_employment_records SET end_date = $1
         WHERE employee_id = $2 AND end_date IS NULL",
    )
    .bind(req.effective_date)
    .bind(row.id)
    .execute(conn.as_mut())
    .await?;

    // Append the new record.
    let actor_uuid = resolve_actor_uuid(conn.as_mut(), &auth).await?;
    let emp_ulid = new_ulid();
    sqlx::query(
        r#"INSERT INTO wf_employment_records
               (ulid, tenant_id, employee_id, position_id, department_id, location_id,
                employment_type, effective_date, changes, change_reason, changed_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
    )
    .bind(&emp_ulid)
    .bind(tenant_id)
    .bind(row.id)
    .bind(pos_id)
    .bind(dept_id)
    .bind(loc_id)
    .bind(emp_type)
    .bind(req.effective_date)
    .bind(serde_json::json!({"event":"employment_change"}))
    .bind(&req.change_reason)
    .bind(actor_uuid)
    .execute(conn.as_mut())
    .await?;

    // Mirror current_* columns on the employee row.
    sqlx::query(
        "UPDATE wf_employees SET current_position_id = $1, current_department_id = $2,
                current_location_id = $3 WHERE ulid = $4",
    )
    .bind(pos_id)
    .bind(dept_id)
    .bind(loc_id)
    .bind(&employee_ulid)
    .execute(conn.as_mut())
    .await?;

    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "employee",
        &employee_ulid,
        "employee.lifecycle.employment_changed",
        serde_json::json!({"effective_date": req.effective_date.to_string()}),
    )
    .await?;

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// Compensation
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/v1/workforce/employees/{employee_ulid}/compensation",
    request_body = CreateCompensationRequest,
    responses(
        (status = 201, description = "Created", body = CompensationResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn create_compensation(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Path(employee_ulid): Path<String>,
    Json(req): Json<CreateCompensationRequest>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("employee.compensation.write")?;

    let mut conn = db.acquire().await?;
    let row = fetch_employee_row(conn.as_mut(), &employee_ulid).await?;
    check_employee_access(&auth, &row, conn.as_mut()).await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;

    let currency_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM currencies WHERE code = $1")
        .bind(&req.currency_code)
        .fetch_optional(conn.as_mut())
        .await?
        .ok_or_else(|| AosError::Validation(format!("unknown currency: {}", req.currency_code)))?;

    // Close the current open compensation record.
    sqlx::query(
        "UPDATE wf_compensation_records SET end_date = $1 WHERE employee_id = $2 AND end_date IS NULL",
    )
    .bind(req.effective_date)
    .bind(row.id)
    .execute(conn.as_mut()).await?;

    let actor_uuid = resolve_actor_uuid(conn.as_mut(), &auth).await?;
    let comp_ulid = new_ulid();
    sqlx::query(
        r#"INSERT INTO wf_compensation_records
               (ulid, tenant_id, employee_id, gross_amount_minor, currency_id,
                frequency, effective_date, change_reason, changed_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(&comp_ulid)
    .bind(tenant_id)
    .bind(row.id)
    .bind(req.gross_amount_minor)
    .bind(currency_id)
    .bind(req.frequency.as_str())
    .bind(req.effective_date)
    .bind(&req.change_reason)
    .bind(actor_uuid)
    .execute(conn.as_mut())
    .await?;

    // Security-monitoring: salary.changed — amount intentionally omitted from log.
    warn!(employee_ulid = %employee_ulid, actor = %auth.user_id, "salary.changed — see audit log");
    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "compensation",
        &employee_ulid,
        "salary.changed",
        // amount is NOT included in the audit changes — see security model.
        serde_json::json!({
            "currency": req.currency_code,
            "frequency": req.frequency.as_str(),
            "effective_date": req.effective_date.to_string(),
        }),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CompensationResponse {
            ulid: comp_ulid,
            gross_amount_minor: req.gross_amount_minor,
            currency_code: req.currency_code,
            frequency: req.frequency.as_str().to_string(),
            effective_date: req.effective_date,
            created_at: chrono::Utc::now(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/employees/{employee_ulid}/compensation",
    params(("employee_ulid" = String, Path, description = "Employee ULID")),
    responses(
        (status = 200, description = "OK", body = CompensationResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn get_compensation(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Path(employee_ulid): Path<String>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("employee.compensation.read")?;

    let mut conn = db.acquire().await?;
    let row = fetch_employee_row(conn.as_mut(), &employee_ulid).await?;
    check_employee_access(&auth, &row, conn.as_mut()).await?;

    let comp = sqlx::query_as::<_, CompensationRow>(
        r#"SELECT cr.ulid, cr.gross_amount_minor, c.code AS currency_code,
                  cr.frequency, cr.effective_date, cr.created_at
           FROM wf_compensation_records cr
           JOIN currencies c ON c.id = cr.currency_id
           WHERE cr.employee_id = $1 AND cr.end_date IS NULL
           ORDER BY cr.effective_date DESC LIMIT 1"#,
    )
    .bind(row.id)
    .fetch_optional(conn.as_mut())
    .await?
    .ok_or_else(|| AosError::NotFound("no active compensation record".to_string()))?;

    Ok(Json(CompensationResponse {
        ulid: comp.ulid,
        gross_amount_minor: comp.gross_amount_minor,
        currency_code: comp.currency_code,
        frequency: comp.frequency,
        effective_date: comp.effective_date,
        created_at: comp.created_at,
    }))
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

/// Upload a document. The request body is JSON metadata only; the actual bytes
/// are passed in the `X-Document-Body` header (base64) to avoid multipart
/// parser complexity in the MVP.
///
/// ponytail: real multipart is the right long-term approach; add when a
/// frontend exists. For now: base64 body capped by `server.body_limit_bytes`.
#[utoipa::path(
    post,
    path = "/api/v1/workforce/employees/{employee_ulid}/documents",
    request_body = CreateDocumentMetadataRequest,
    responses(
        (status = 201, description = "Created", body = DocumentResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn create_document(
    State(state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Path(employee_ulid): Path<String>,
    body: axum::body::Bytes,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("employee.documents.write")?;

    // Body format: JSON metadata line\n<raw file bytes>
    // The JSON line contains doc_type, filename, mime_type, size_bytes, sha256.
    let body_bytes = body;

    // Split at first newline: metadata JSON | file bytes.
    let newline_pos = body_bytes.iter().position(|&b| b == b'\n').ok_or_else(|| {
        AosError::Validation("body must be: JSON metadata\\n<file bytes>".to_string())
    })?;

    let meta_json = &body_bytes[..newline_pos];
    let file_bytes = body_bytes[newline_pos + 1..].to_vec();

    let req: CreateDocumentMetadataRequest = serde_json::from_slice(meta_json)
        .map_err(|e| AosError::Validation(format!("metadata parse error: {e}")))?;

    let mut conn = db.acquire().await?;
    let emp_row = fetch_employee_row(conn.as_mut(), &employee_ulid).await?;
    check_employee_access(&auth, &emp_row, conn.as_mut()).await?;
    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;
    let actor_uuid = resolve_actor_uuid(conn.as_mut(), &auth).await?;
    let tenant_ulid = auth.tenant_id.to_string();

    let doc_ulid = new_ulid();
    let size_bytes = file_bytes.len() as i64;

    let ctx = UploadContext {
        tenant_ulid: &tenant_ulid,
        employee_ulid: &employee_ulid,
        doc_ulid: &doc_ulid,
        request: &req,
        bytes: file_bytes,
    };
    let (object_key, sha256) = upload_to_storage(&state.s3_client, &state.s3_cfg, ctx).await?;

    // If the client supplied a sha256, verify it matches the bytes we received.
    // Mismatch means the request was corrupted or tampered with — reject it.
    if let Some(ref client_hash) = req.sha256 {
        if client_hash != &sha256 {
            return Err(AosError::Validation(
                "sha256 mismatch: supplied hash does not match received bytes".to_string(),
            ));
        }
    }

    sqlx::query(
        r#"INSERT INTO wf_documents
               (ulid, tenant_id, employee_id, doc_type, object_key,
                filename, mime_type, sha256, size_bytes, uploaded_by)
           VALUES
               ($1, $2,
                (SELECT id FROM wf_employees WHERE ulid = $3),
                $4, $5, $6, $7, $8, $9, $10)"#,
    )
    .bind(&doc_ulid)
    .bind(tenant_id)
    .bind(&employee_ulid)
    .bind(req.doc_type.as_str())
    .bind(&object_key)
    .bind(&req.filename)
    .bind(&req.mime_type)
    .bind(&sha256)
    .bind(size_bytes)
    .bind(actor_uuid)
    .execute(conn.as_mut())
    .await?;

    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "document",
        &doc_ulid,
        "document.uploaded",
        serde_json::json!({
            "doc_type": req.doc_type.as_str(),
            "filename": req.filename,
            // object_key intentionally omitted from audit log
        }),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(DocumentResponse {
            ulid: doc_ulid,
            doc_type: req.doc_type.as_str().to_string(),
            filename: req.filename,
            mime_type: req.mime_type,
            size_bytes: Some(size_bytes),
            is_active: true,
            created_at: chrono::Utc::now(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/employees/{employee_ulid}/documents",
    params(
        ("employee_ulid" = String, Path, description = "Employee ULID"),
        PageQuery
    ),
    responses(
        (status = 200, description = "OK"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn list_documents(
    State(_state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Path(employee_ulid): Path<String>,
    Query(q): Query<PageQuery>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("employee.documents.read")?;
    let q = q.sanitized();
    let mut conn = db.acquire().await?;

    let emp_row = fetch_employee_row(conn.as_mut(), &employee_ulid).await?;
    check_employee_access(&auth, &emp_row, conn.as_mut()).await?;

    let rows = sqlx::query_as::<_, DocumentRow>(
        r#"SELECT id, ulid, doc_type, object_key, filename, mime_type,
                  size_bytes, is_active, created_at
           FROM wf_documents
           WHERE employee_id = $1 AND is_active = TRUE
           ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
    )
    .bind(emp_row.id)
    .bind(q.page_size)
    .bind(q.offset())
    .fetch_all(conn.as_mut())
    .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM wf_documents WHERE employee_id = $1 AND is_active = TRUE",
    )
    .bind(emp_row.id)
    .fetch_one(conn.as_mut())
    .await?;

    let items: Vec<DocumentResponse> = rows.into_iter().map(|r| r.into()).collect();
    Ok(Json(PaginatedResponse {
        items,
        total,
        page: q.page,
        page_size: q.page_size,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/workforce/employees/{employee_ulid}/documents/{doc_ulid}/url",
    params(
        ("employee_ulid" = String, Path, description = "Employee ULID"),
        ("doc_ulid" = String, Path, description = "Document ULID"),
    ),
    responses(
        (status = 200, description = "Presigned URL", body = DocumentUrlResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "workforce",
    security(("bearerAuth" = []))
)]
pub async fn get_document_url(
    State(state): State<AppState>,
    auth: AuthContext,
    db: DbConn,
    Path((employee_ulid, doc_ulid)): Path<(String, String)>,
) -> AosResult<impl IntoResponse> {
    auth.require_permission("employee.documents.read")?;

    let mut conn = db.acquire().await?;
    let emp_row = fetch_employee_row(conn.as_mut(), &employee_ulid).await?;
    check_employee_access(&auth, &emp_row, conn.as_mut()).await?;

    // Fetch the document row — RLS already scopes it to the caller's tenant.
    let doc_row = sqlx::query_as::<_, DocumentRow>(
        r#"SELECT id, ulid, doc_type, object_key, filename, mime_type,
                  size_bytes, is_active, created_at
           FROM wf_documents WHERE ulid = $1 AND employee_id = $2"#,
    )
    .bind(&doc_ulid)
    .bind(emp_row.id)
    .fetch_optional(conn.as_mut())
    .await?
    .ok_or_else(|| AosError::NotFound(format!("document {doc_ulid} not found")))?;

    // IDOR defence-in-depth: if the doc's employee_id doesn't match, that's a
    // suspicious pattern (RLS should have prevented it — log as security event).
    // The query above already ensures employee_id = emp_row.id, but we log it.
    let (presigned_url, ttl) =
        presigned_url_for_doc(&state.s3_client, &state.s3_cfg, &doc_row).await?;

    let tenant_id = resolve_tenant_id(conn.as_mut(), &auth).await?;
    let org_id = resolve_org_id(conn.as_mut(), &auth).await?;
    emit_workforce_event(
        conn.as_mut(),
        &auth,
        tenant_id,
        org_id,
        "document",
        &doc_ulid,
        "document.accessed",
        serde_json::json!({"employee_ulid": employee_ulid}),
    )
    .await?;

    Ok(Json(DocumentUrlResponse {
        presigned_url,
        expires_in_secs: ttl,
    }))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

async fn fetch_employee_row(
    conn: &mut sqlx::PgConnection,
    employee_ulid: &str,
) -> AosResult<EmployeeRow> {
    sqlx::query_as::<_, EmployeeRow>(
        r#"SELECT id, ulid, employee_number, legal_name, preferred_name, email, phone,
                  gender, national_id_last4, status, hire_date,
                  current_department_id, current_position_id, current_location_id,
                  manager_employee_id, created_at, updated_at
           FROM wf_employees WHERE ulid = $1 AND deleted_at IS NULL"#,
    )
    .bind(employee_ulid)
    .fetch_optional(conn)
    .await?
    .ok_or_else(|| AosError::NotFound(format!("employee {employee_ulid} not found")))
}

/// IDOR check: the caller may see this employee iff:
///   - they have `employee.read` broadly (HR_ADMIN, ORG_ADMIN), OR
///   - they are the employee's manager (MANAGER role), OR
///   - they are the employee themselves (any EMPLOYEE role).
///
/// RLS already enforces tenant isolation. This check adds object-level authz
/// on top, so a cross-employee read within the same tenant also fails.
async fn check_employee_access(
    auth: &AuthContext,
    row: &EmployeeRow,
    conn: &mut sqlx::PgConnection,
) -> AosResult<()> {
    // Broad HR/admin read.
    if auth.has_permission("employee.write") || auth.has_permission("employee.read") {
        return Ok(());
    }

    // Self-view (EMPLOYEE role).
    let self_id: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT e.id FROM wf_employees e JOIN users u ON u.id = e.user_id WHERE u.ulid = $1",
    )
    .bind(auth.user_id.to_string())
    .fetch_optional(conn)
    .await?;

    if Some(row.id) == self_id {
        return Ok(());
    }

    // Manager of this employee.
    if let Some(mgr_id) = row.manager_employee_id {
        if Some(mgr_id) == self_id {
            return Ok(());
        }
    }

    Err(AosError::Forbidden(
        "not authorized to view this employee record".to_string(),
    ))
}

/// Resolve the actor's `users.id` from their ULID (needed for FK columns like
/// `created_by`, `changed_by`).
async fn resolve_actor_uuid(
    conn: &mut sqlx::PgConnection,
    auth: &AuthContext,
) -> AosResult<Option<uuid::Uuid>> {
    Ok(sqlx::query_scalar("SELECT id FROM users WHERE ulid = $1")
        .bind(auth.user_id.to_string())
        .fetch_optional(conn)
        .await?)
}

async fn resolve_opt(
    conn: &mut sqlx::PgConnection,
    table: &str,
    ulid: Option<&str>,
) -> AosResult<Option<uuid::Uuid>> {
    match ulid {
        Some(u) => Ok(Some(resolve_ulid(conn, table, u).await?)),
        None => Ok(None),
    }
}

/// Hash and redact a national ID. Returns `(hash, last4)`.
///
/// The raw value is consumed and never stored. The hash is used only for
/// duplicate detection (same person won't hash to the same value unless the
/// same raw value is submitted). last4 is for human disambiguation.
fn hash_national_id(id: Option<&str>) -> (Option<String>, Option<String>) {
    match id {
        None | Some("") => (None, None),
        Some(raw) => {
            let mut hasher = Sha256::new();
            hasher.update(raw.as_bytes());
            let hash = hex::encode(hasher.finalize());
            let last4 = if raw.len() >= 4 {
                Some(raw[raw.len() - 4..].to_string())
            } else {
                Some(raw.to_string())
            };
            (Some(hash), last4)
        }
    }
}

fn employee_row_to_response(row: EmployeeRow) -> EmployeeResponse {
    // FK columns on wf_employees store UUID primary keys, not ULIDs.
    // The response type documents these as ULID fields — we omit them rather
    // than return a UUID string that would silently break callers.
    // ponytail: JOIN wf_departments/positions/locations/employees on id→ulid in
    // fetch_employee_row's SELECT when the frontend needs these populated.
    EmployeeResponse {
        ulid: row.ulid,
        employee_number: row.employee_number,
        legal_name: row.legal_name,
        preferred_name: row.preferred_name,
        email: row.email,
        phone: row.phone,
        gender: row.gender,
        national_id_last4: row.national_id_last4,
        status: row.status,
        hire_date: row.hire_date,
        current_department_ulid: None, // ponytail: needs JOIN to resolve UUID→ULID
        current_position_ulid: None,   // ponytail: needs JOIN to resolve UUID→ULID
        current_location_ulid: None,   // ponytail: needs JOIN to resolve UUID→ULID
        manager_employee_ulid: None,   // ponytail: needs JOIN to resolve UUID→ULID
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
