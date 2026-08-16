//! Audit service handlers — append-only create, paginated list, get-by-ID,
//! and authorized hash-chain verification.
//!
//! All database access runs through the request-scoped [`DbConn`] extractor so
//! queries execute under the tenant's RLS context. Handlers also extract
//! [`AuthContext`] for tenant/organization identity and authorization.

use crate::models::{
    AuditEventListResponse, AuditEventResponse, AuditEventRow, AuditListQuery, CreateAuditRequest,
    HashChainVerification,
};
use nexora_common::audit::{compute_chain_hash, GENESIS_HASH};
use nexora_common::{AosError, AuthContext, DbConn};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use tracing::warn;

/// Column list shared by all `audit_events` SELECTs. `ip_address` is cast to
/// `text` so the row maps to `Option<String>` (no `IpAddr` sqlx feature).
const SELECT_COLUMNS: &str = r#"
    SELECT id, ulid, tenant_id, org_id, actor_type, actor_id, action,
           resource_type, resource_id, resource_ulid, changes, metadata,
           request_id, correlation_id, ip_address::text AS ip_address, user_agent,
           result, error_message, hash, created_at
"#;

/// Create a new append-only audit event.
#[utoipa::path(
    post,
    path = "/api/v1/audit/events",
    tag = "Audit",
    request_body = CreateAuditRequest,
    responses(
        (status = 201, description = "Audit event created", body = AuditEventResponse),
        (status = 400, description = "Validation failed", body = nexora_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
        (status = 500, description = "Internal error", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn create_audit_event(
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Json(req): Json<CreateAuditRequest>,
) -> Result<impl IntoResponse, AosError> {
    // Reject empty action / resource_type early — these are NOT NULL in DB.
    if req.action.trim().is_empty() {
        return Err(AosError::Validation("action is required".into()));
    }
    if req.resource_type.trim().is_empty() {
        return Err(AosError::Validation("resource_type is required".into()));
    }
    if let Some(ref ip) = req.ip_address {
        ip.parse::<std::net::IpAddr>()
            .map_err(|e| AosError::Validation(format!("invalid ip_address: {e}")))?;
    }

    let mut conn = db.acquire().await?;

    // Resolve the tenant's UUID given its ULID (from the auth context). This
    // query runs under the RLS context already set by `rls_middleware`, so a
    // caller scoped to tenant A cannot resolve tenant B's UUID.
    let tenant_id = resolve_tenant_id(&mut conn, &auth).await?;

    // Resolve the organization UUID from the auth-context org ULID.
    let org_id = resolve_org_id(&mut conn, &auth).await?;

    // Compute the SHA-256 chain hash: prev_hash || canonical_payload.
    let prev_hash = latest_hash_for_tenant(conn.as_mut(), tenant_id).await?;
    let payload = canonical_payload(&auth, &tenant_id, &org_id, &req);
    let hash = compute_chain_hash(&prev_hash, &payload);

    let event_ulid = nexora_common::ulid::new_ulid();
    // Bind `ip_address` as text and let Postgres cast it to `INET`. sqlx's
    // `IpAddr` mapping requires the `ipnetwork` feature which we don't take on.
    let ip_address = req.ip_address.as_ref().map(|addr| addr.to_string());
    let metadata = req.metadata.unwrap_or(serde_json::Value::Null);

    let row = sqlx::query_as::<_, AuditEventRow>(
        r#"
        INSERT INTO audit_events (
            ulid, tenant_id, org_id, actor_type, actor_id, action,
            resource_type, resource_id, resource_ulid, changes,
            metadata, request_id, correlation_id, ip_address,
            user_agent, result, error_message, hash
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            $11, $12, $13, $14::inet, $15, $16, $17, $18
        )
        RETURNING
            id, ulid, tenant_id, org_id, actor_type, actor_id, action,
            resource_type, resource_id, resource_ulid, changes, metadata,
            request_id, correlation_id, ip_address::text AS ip_address, user_agent,
            result, error_message, hash, created_at
        "#,
    )
    .bind(event_ulid)
    .bind(tenant_id)
    .bind(org_id)
    .bind(req.actor_type.as_str())
    .bind(req.actor_id)
    .bind(req.action)
    .bind(req.resource_type)
    .bind(req.resource_id)
    .bind(req.resource_ulid)
    .bind(req.changes)
    .bind(metadata)
    .bind(req.request_id)
    .bind(req.correlation_id)
    .bind(ip_address)
    .bind(req.user_agent)
    .bind(req.result.as_str())
    .bind(req.error_message)
    .bind(&hash)
    .fetch_one(conn.as_mut())
    .await?;

    let response = AuditEventResponse::from(row);
    Ok((StatusCode::CREATED, Json(response)))
}

/// List audit events for the caller's tenant with optional filters and pagination.
#[utoipa::path(
    get,
    path = "/api/v1/audit/events",
    tag = "Audit",
    params(AuditListQuery),
    responses(
        (status = 200, description = "Paginated audit events", body = AuditEventListResponse),
        (status = 400, description = "Validation failed", body = nexora_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_audit_events(
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Query(query): Query<AuditListQuery>,
) -> Result<Json<AuditEventListResponse>, AosError> {
    let query = query.sanitized();

    // Validate filter strings against the DB CHECK constraints before binding.
    if let Some(ref a) = query.actor_type {
        validate_actor_type(a)?;
    }
    if let Some(ref r) = query.result {
        validate_result(r)?;
    }

    // Parse RFC3339 date strings for since/until filters.
    let since = query
        .since
        .as_deref()
        .map(|s| s.parse::<chrono::DateTime<chrono::Utc>>())
        .transpose()
        .map_err(|e| AosError::Validation(format!("invalid since date: {e}")))?;
    let until = query
        .until
        .as_deref()
        .map(|s| s.parse::<chrono::DateTime<chrono::Utc>>())
        .transpose()
        .map_err(|e| AosError::Validation(format!("invalid until date: {e}")))?;

    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant_id(&mut conn, &auth).await?;

    let sql = format!(
        "{SELECT_COLUMNS}
        FROM audit_events
        WHERE tenant_id = $1
          AND ($2::text IS NULL OR actor_type = $2)
          AND ($3::text IS NULL OR action = $3)
          AND ($4::text IS NULL OR resource_type = $4)
          AND ($5::uuid IS NULL OR resource_id = $5)
          AND ($6::text IS NULL OR result = $6)
          AND ($7::timestamptz IS NULL OR created_at >= $7)
          AND ($8::timestamptz IS NULL OR created_at <= $8)
          AND ($9::text IS NULL OR correlation_id = $9)
        ORDER BY created_at DESC
        LIMIT $10 OFFSET $11"
    );
    let rows: Vec<AuditEventRow> = sqlx::query_as::<_, AuditEventRow>(&sql)
        .bind(tenant_id)
        .bind(query.actor_type.clone())
        .bind(query.action.clone())
        .bind(query.resource_type.clone())
        .bind(query.resource_id)
        .bind(query.result.clone())
        .bind(since)
        .bind(until)
        .bind(query.correlation_id.clone())
        .bind(query.page_size as i64)
        .bind(query.offset())
        .fetch_all(conn.as_mut())
        .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM audit_events
        WHERE tenant_id = $1
          AND ($2::text IS NULL OR actor_type = $2)
          AND ($3::text IS NULL OR action = $3)
          AND ($4::text IS NULL OR resource_type = $4)
          AND ($5::uuid IS NULL OR resource_id = $5)
          AND ($6::text IS NULL OR result = $6)
          AND ($7::timestamptz IS NULL OR created_at >= $7)
          AND ($8::timestamptz IS NULL OR created_at <= $8)
          AND ($9::text IS NULL OR correlation_id = $9)
        "#,
    )
    .bind(tenant_id)
    .bind(query.actor_type.clone())
    .bind(query.action.clone())
    .bind(query.resource_type.clone())
    .bind(query.resource_id)
    .bind(query.result.clone())
    .bind(since)
    .bind(until)
    .bind(query.correlation_id.clone())
    .fetch_one(conn.as_mut())
    .await?;

    let events = rows.into_iter().map(AuditEventResponse::from).collect();
    Ok(Json(AuditEventListResponse {
        events,
        total,
        page: query.page,
        page_size: query.page_size,
    }))
}

/// Get a single audit event by its serial ID.
#[utoipa::path(
    get,
    path = "/api/v1/audit/events/{id}",
    tag = "Audit",
    params(("id" = i64, Path, description = "Audit event serial ID")),
    responses(
        (status = 200, description = "Audit event", body = AuditEventResponse),
        (status = 404, description = "Not found", body = nexora_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn get_audit_event(
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbConn>,
    Path(event_id): Path<i64>,
) -> Result<Json<AuditEventResponse>, AosError> {
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant_id(&mut conn, &auth).await?;

    let sql = format!(
        "{SELECT_COLUMNS}
        FROM audit_events
        WHERE id = $1 AND tenant_id = $2"
    );
    let row = sqlx::query_as::<_, AuditEventRow>(&sql)
        .bind(event_id)
        .bind(tenant_id)
        .fetch_optional(conn.as_mut())
        .await?
        .ok_or_else(|| AosError::NotFound(format!("audit event {event_id} not found")))?;

    Ok(Json(AuditEventResponse::from(row)))
}

/// Verify the hash-chain integrity of audit events for the caller's tenant,
/// walking from the first event through the requested event.
#[utoipa::path(
    get,
    path = "/api/v1/audit/events/verify/{id}",
    tag = "Audit",
    params(("id" = i64, Path, description = "Audit event serial ID to verify up to")),
    responses(
        (status = 200, description = "Hash-chain verification result", body = HashChainVerification),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn verify_hash_chain(
    auth: AuthContext,
    db: DbConn,
    Path(event_id): Path<i64>,
) -> Result<Json<HashChainVerification>, AosError> {
    let mut conn = db.acquire().await?;
    let tenant_id = resolve_tenant_id(&mut conn, &auth).await?;

    let sql = format!(
        "{SELECT_COLUMNS}
        FROM audit_events
        WHERE tenant_id = $1
        ORDER BY id ASC"
    );
    let rows: Vec<AuditEventRow> = sqlx::query_as::<_, AuditEventRow>(&sql)
        .bind(tenant_id)
        .fetch_all(conn.as_mut())
        .await?;

    if rows.is_empty() {
        return Ok(Json(HashChainVerification {
            verified: true,
            events_checked: 0,
            first_mismatch: None,
            message: "no audit events for tenant".to_string(),
        }));
    }

    let mut prev_hash = GENESIS_HASH.to_string();
    let mut events_checked: u64 = 0;
    let mut first_mismatch: Option<i64> = None;

    for row in &rows {
        events_checked += 1;
        let payload = canonical_payload_from_row(row);
        let computed = compute_chain_hash(&prev_hash, &payload);
        if computed != row.hash {
            if first_mismatch.is_none() {
                first_mismatch = Some(row.id);
            }
            warn!(
                event_id = row.id,
                tenant_id = %row.tenant_id,
                "audit hash-chain mismatch detected"
            );
            break;
        }
        prev_hash = row.hash.clone();
        // Stop after we've covered the requested event.
        if row.id == event_id {
            break;
        }
    }

    let verified = first_mismatch.is_none();
    let message = if verified {
        format!("chain verified: {events_checked} event(s) checked")
    } else {
        format!(
            "chain broken at event {} after {events_checked} event(s)",
            first_mismatch.unwrap_or(-1)
        )
    };

    Ok(Json(HashChainVerification {
        verified,
        events_checked,
        first_mismatch,
        message,
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve the tenant `UUID` (the `tenants.id` column) from the auth-context
/// tenant ULID. Runs under RLS so only the caller's own tenant resolves.
async fn resolve_tenant_id(
    conn: &mut sqlx::PgConnection,
    auth: &AuthContext,
) -> Result<uuid::Uuid, AosError> {
    let tenant_ulid = auth.tenant_id.to_string();
    let row: Option<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM tenants WHERE ulid = $1")
        .bind(&tenant_ulid)
        .fetch_optional(conn)
        .await?;
    row.map(|(id,)| id).ok_or_else(|| {
        AosError::TenantIsolation(format!(
            "tenant {tenant_ulid} not visible in current RLS context"
        ))
    })
}

/// Resolve the organization `UUID` from the auth-context org ULID.
async fn resolve_org_id(
    conn: &mut sqlx::PgConnection,
    auth: &AuthContext,
) -> Result<uuid::Uuid, AosError> {
    let org_ulid = auth.org_id.to_string();
    let row: Option<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM organizations WHERE ulid = $1")
        .bind(&org_ulid)
        .fetch_optional(conn)
        .await?;
    row.map(|(id,)| id).ok_or_else(|| {
        AosError::TenantIsolation(format!(
            "organization {org_ulid} not visible in current RLS context"
        ))
    })
}

/// Latest hash in the chain for this tenant (genesis if none yet).
async fn latest_hash_for_tenant(
    conn: &mut sqlx::PgConnection,
    tenant_id: uuid::Uuid,
) -> Result<String, AosError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT hash FROM audit_events WHERE tenant_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(conn)
    .await?;
    Ok(row
        .map(|(h,)| h)
        .unwrap_or_else(|| GENESIS_HASH.to_string()))
}

/// Deterministic canonical payload for hash computation on a create request.
///
/// Fields are serialized in a fixed order so the hash is reproducible by the
/// verifier ([`canonical_payload_from_row`]). Every field here must be one the
/// stored row can reproduce on read; including a value that is never persisted
/// (e.g. the caller's tenant ULID, which the row has no column for) makes the
/// chain unverifiable. The `tenant_id`/`org_id` UUIDs already bind a chain to
/// a tenant, so no separate actor-tenant field is needed.
fn canonical_payload(
    _auth: &AuthContext,
    tenant_id: &uuid::Uuid,
    org_id: &uuid::Uuid,
    req: &CreateAuditRequest,
) -> String {
    use serde_json::json;
    let payload = json!({
        "tenant_id": tenant_id.to_string(),
        "org_id": org_id.to_string(),
        "actor_type": req.actor_type.as_str(),
        "actor_id": req.actor_id,
        "action": req.action,
        "resource_type": req.resource_type,
        "resource_id": req.resource_id.map(|u| u.to_string()),
        "resource_ulid": req.resource_ulid,
        "changes": req.changes,
        "metadata": req.metadata,
        "request_id": req.request_id,
        "correlation_id": req.correlation_id,
        "ip_address": req.ip_address,
        "user_agent": req.user_agent,
        "result": req.result.as_str(),
        "error_message": req.error_message,
    });
    serialize_canonical(&payload)
}

/// Canonical payload reconstructed from a stored row, used during verification.
fn canonical_payload_from_row(row: &AuditEventRow) -> String {
    use serde_json::json;
    let payload = json!({
        "tenant_id": row.tenant_id.to_string(),
        "org_id": row.org_id.to_string(),
        "actor_type": row.actor_type,
        "actor_id": row.actor_id,
        "action": row.action,
        "resource_type": row.resource_type,
        "resource_id": row.resource_id.map(|u| u.to_string()),
        "resource_ulid": row.resource_ulid,
        "changes": row.changes,
        "metadata": row.metadata,
        "request_id": row.request_id,
        "correlation_id": row.correlation_id,
        "ip_address": row.ip_address,
        "user_agent": row.user_agent,
        "result": row.result,
        "error_message": row.error_message,
    });
    serialize_canonical(&payload)
}

/// Serialize a `serde_json::Value` to a string with deterministic key ordering.
fn serialize_canonical(value: &serde_json::Value) -> String {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::CompactFormatter;
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    let _ = value.serialize(&mut ser);
    String::from_utf8_lossy(&buf).to_string()
}

fn validate_actor_type(s: &str) -> Result<(), AosError> {
    match s {
        "user" | "system" | "service" | "api_key" => Ok(()),
        other => Err(AosError::Validation(format!("invalid actor_type: {other}"))),
    }
}

fn validate_result(s: &str) -> Result<(), AosError> {
    match s {
        "success" | "failure" | "partial" => Ok(()),
        other => Err(AosError::Validation(format!("invalid result: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ActorType, AuditEventRow, AuditResult};
    use nexora_common::{ulid::Ulid, AuthContext};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn validators_reject_unknown() {
        assert!(validate_actor_type("user").is_ok());
        assert!(validate_actor_type("robot").is_err());
        assert!(validate_result("success").is_ok());
        assert!(validate_result("boom").is_err());
    }

    #[test]
    fn canonical_serialization_is_stable() {
        use serde_json::json;
        let v = json!({"b": 1, "a": 2});
        assert_eq!(serialize_canonical(&v), serialize_canonical(&v));
    }

    /// The hash-chain is only verifiable if create-time and verify-time canonical
    /// payloads are byte-identical for the same logical event. This test catches
    /// field divergences (e.g. a field present at create but not persisted, or a
    /// different serialization order).
    #[test]
    fn canonical_payload_matches_stored_row() {
        // Build an AuthContext matching a real tenant/org.
        let tenant_ulid = Ulid::new();
        let org_ulid = Ulid::new();
        let tenant_uuid = Uuid::new_v4();
        let org_uuid = Uuid::new_v4();
        let auth = AuthContext {
            tenant_id: tenant_ulid,
            org_id: org_ulid,
            user_id: Ulid::new(),
            keycloak_sub: "kc-test".into(),
            is_system: false,
            roles: vec!["ADMIN".into()],
            permissions: vec!["audit.write".into()],
        };

        // Create request matching what a handler would receive.
        let req = CreateAuditRequest {
            actor_type: ActorType::User,
            actor_id: Some("01HXZ_ACTOR_ULID_TEST0001".into()),
            action: "tenant.create".into(),
            resource_type: "tenant".into(),
            resource_id: Some(Uuid::new_v4()),
            resource_ulid: Some("01ARZ3NDEKTSV4RRFFQ69G5FAW".into()),
            changes: Some(serde_json::json!({"name": "Acme"})),
            metadata: Some(serde_json::json!({"source": "api"})),
            request_id: Some("req-123".into()),
            correlation_id: Some("corr-456".into()),
            ip_address: Some("203.0.113.5".into()),
            user_agent: Some("test-agent".into()),
            result: AuditResult::Success,
            error_message: None,
        };

        // Compute the create-time payload.
        let create_payload = canonical_payload(&auth, &tenant_uuid, &org_uuid, &req);

        // Build a row mirroring exactly what the handler stores.
        let row = AuditEventRow {
            id: 1,
            ulid: "01ARZ3NDEKTSV4RRFFQ69G5FAV".into(),
            tenant_id: tenant_uuid,
            org_id: org_uuid,
            actor_type: "user".into(),
            actor_id: req.actor_id,
            action: req.action.clone(),
            resource_type: req.resource_type.clone(),
            resource_id: req.resource_id,
            resource_ulid: req.resource_ulid.clone(),
            changes: req.changes.clone(),
            metadata: req.metadata.clone().unwrap_or(serde_json::Value::Null),
            request_id: req.request_id.clone(),
            correlation_id: req.correlation_id.clone(),
            ip_address: req.ip_address.clone(),
            user_agent: req.user_agent.clone(),
            result: req.result.as_str().into(),
            error_message: req.error_message.clone(),
            hash: String::new(), // not used in payload
            created_at: Utc::now(),
        };

        // Compute the verify-time payload from the stored row.
        let verify_payload = canonical_payload_from_row(&row);

        // They MUST be identical for the hash chain to verify.
        assert_eq!(
            create_payload, verify_payload,
            "create and verify canonical payloads diverged; hash-chain will report false mismatches"
        );
    }
}
