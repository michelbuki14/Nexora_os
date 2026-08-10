//! Tenant service handlers — real SQLx operations under the request's RLS context.
//!
//! All database access runs through the request-scoped [`DbConn`] extractor so
//! queries execute inside the tenant's RLS transaction set up by `rls_middleware`
//! (see `aos_common::tenant_context`). Handlers never touch the raw pool on
//! RLS-protected paths.

use crate::models::{CreateTenantRequest, TenantResponse, TenantRow, VALID_TIERS};
use aos_common::{ulid::new_ulid, AosError, AuthContext, DbConn};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::debug;

/// Column list shared by tenant SELECTs. Joins `organizations` so the
/// organization ULID (not just its UUID) is returned to callers. RLS applies to
/// both sides of the join, so a caller only ever resolves organizations it can
/// see.
const TENANT_SELECT: &str = r#"
    SELECT t.ulid, o.ulid AS org_ulid, t.name, t.slug, t.status, t.tier
    FROM tenants t
    JOIN organizations o ON o.id = t.org_id
"#;

/// Create a new organization + tenant pair (system operation).
#[utoipa::path(
    post,
    path = "/api/v1/admin/tenants",
    tag = "Tenant",
    request_body = CreateTenantRequest,
    responses(
        (status = 201, description = "Tenant created", body = TenantResponse),
        (status = 400, description = "Validation failed", body = aos_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = aos_common::ErrorResponse),
        (status = 403, description = "Forbidden", body = aos_common::ErrorResponse),
        (status = 500, description = "Internal error", body = aos_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn create_tenant(
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateTenantRequest>,
) -> Result<impl IntoResponse, AosError> {
    validate_create_request(&req)?;

    // Creating a tenant is a cross-tenant operation: the new tenant's ULID can
    // never match the caller's RLS context, so the `tenants` WITH CHECK policy
    // would reject it. Fail fast with a clean 403 instead of a DB error.
    if !auth.is_system {
        return Err(AosError::Forbidden(
            "tenant creation requires system privileges".into(),
        ));
    }

    let mut conn = db.acquire().await?;

    let org_ulid = new_ulid();
    let tenant_ulid = new_ulid();

    // Create the owning organization first, returning its UUID for the tenant.
    let org_id: sqlx::types::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO organizations (ulid, name, slug, tier)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(&org_ulid)
    .bind(&req.name)
    .bind(&req.slug)
    .bind(&req.tier)
    .fetch_one(conn.as_mut())
    .await?;

    // Then the tenant referencing it. Runs inside the request's RLS transaction
    // where `aos.is_system` = 'true' permits the insert.
    sqlx::query(
        r#"
        INSERT INTO tenants (ulid, org_id, name, slug, tier)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(&tenant_ulid)
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.slug)
    .bind(&req.tier)
    .execute(conn.as_mut())
    .await?;

    debug!(tenant = %tenant_ulid, org = %org_ulid, "tenant created");

    Ok((
        StatusCode::CREATED,
        Json(TenantResponse {
            tenant_id: tenant_ulid,
            org_id: org_ulid,
            name: req.name,
            slug: req.slug,
            status: "active".to_string(),
            tier: req.tier,
        }),
    ))
}

/// List tenants visible to the caller. Under RLS this returns only the caller's
/// own tenant row, or every tenant when the caller has system privileges.
#[utoipa::path(
    get,
    path = "/api/v1/tenants",
    tag = "Tenant",
    responses(
        (status = 200, description = "Tenants visible to caller", body = [TenantResponse]),
        (status = 401, description = "Unauthorized", body = aos_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_tenants(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<TenantResponse>>, AosError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<TenantRow> = sqlx::query_as(&format!("{TENANT_SELECT}\nORDER BY t.created_at"))
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(TenantResponse::from).collect()))
}

/// Get a single tenant by ULID. Only visible when it belongs to the caller's
/// RLS context (or the caller is a system user); otherwise 404.
#[utoipa::path(
    get,
    path = "/api/v1/tenants/{id}",
    tag = "Tenant",
    params(("id" = String, Path, description = "Tenant ULID")),
    responses(
        (status = 200, description = "Tenant", body = TenantResponse),
        (status = 401, description = "Unauthorized", body = aos_common::ErrorResponse),
        (status = 404, description = "Not found", body = aos_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn get_tenant(
    _auth: AuthContext,
    db: DbConn,
    Path(ulid): Path<String>,
) -> Result<Json<TenantResponse>, AosError> {
    let mut conn = db.acquire().await?;
    let row: Option<TenantRow> = sqlx::query_as(&format!("{TENANT_SELECT}\nWHERE t.ulid = $1"))
        .bind(&ulid)
        .fetch_optional(conn.as_mut())
        .await?;
    let row = row.ok_or_else(|| AosError::NotFound(format!("tenant {ulid} not found")))?;
    Ok(Json(row.into()))
}

/// Validate tenant create input before touching the database.
fn validate_create_request(req: &CreateTenantRequest) -> Result<(), AosError> {
    if req.name.trim().is_empty() {
        return Err(AosError::Validation("name is required".into()));
    }
    if !is_valid_slug(&req.slug) {
        return Err(AosError::Validation(format!(
            "invalid slug {:?}: expected 1-64 chars of [a-z0-9-] starting with a letter or digit",
            req.slug
        )));
    }
    if !VALID_TIERS.contains(&req.tier.as_str()) {
        return Err(AosError::Validation(format!(
            "invalid tier {:?}: expected one of {VALID_TIERS:?}",
            req.tier
        )));
    }
    Ok(())
}

/// Slugs are URL-safe tenant identifiers: lowercase ASCII alphanumerics and
/// hyphens, never starting or ending with a hyphen.
fn is_valid_slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_validation() {
        assert!(is_valid_slug("acme"));
        assert!(is_valid_slug("acme-ltd-2024"));
        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("-acme"));
        assert!(!is_valid_slug("acme ltd"));
        assert!(!is_valid_slug("ACME"));
        assert!(!is_valid_slug(&"a".repeat(65)));
    }

    #[test]
    fn tier_validation() {
        let ok = CreateTenantRequest {
            name: "Acme".into(),
            slug: "acme".into(),
            tier: "enterprise".into(),
        };
        assert!(validate_create_request(&ok).is_ok());

        let bad_tier = CreateTenantRequest {
            name: "Acme".into(),
            slug: "acme".into(),
            tier: "platinum".into(),
        };
        assert!(validate_create_request(&bad_tier).is_err());

        let bad_slug = CreateTenantRequest {
            name: "Acme".into(),
            slug: "bad slug".into(),
            tier: "free".into(),
        };
        assert!(validate_create_request(&bad_slug).is_err());
    }
}
