//! Government service handlers — real SQLx operations under the request's RLS context.

use crate::models::{
    CreateLicenseRequest, CreatePermitRequest, LicenseRow, LicenseResponse, PermitRow,
    PermitResponse, RecordRow, RecordResponse, VALID_LICENSE_STATUSES, VALID_PERMIT_STATUSES,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use nexora_common::{ulid::new_ulid, AuthContext, DbConn, NexoraError};
use tracing::debug;

const PERMIT_SELECT: &str = r#"
    SELECT ulid, applicant_ulid, permit_type, description, status, submitted_at
    FROM permits
"#;

const LICENSE_SELECT: &str = r#"
    SELECT ulid, holder_ulid, license_type, holder_name, status, issued_at, expires_at
    FROM licenses
"#;

const RECORD_SELECT: &str = r#"
    SELECT ulid, record_type, subject_name, issued_at, jurisdiction
    FROM civil_records
"#;

/// Create a new permit application.
#[utoipa::path(
    post,
    path = "/api/v1/permits",
    tag = "Government",
    request_body = CreatePermitRequest,
    responses(
        (status = 201, description = "Permit created", body = PermitResponse),
        (status = 400, description = "Validation failed", body = nexora_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn create_permit(
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreatePermitRequest>,
) -> Result<impl IntoResponse, NexoraError> {
    validate_permit_request(&req)?;
    let mut conn = db.acquire().await?;
    let permit_ulid = new_ulid();
    sqlx::query(
        r#"
        INSERT INTO permits (ulid, tenant_id, org_id, applicant_ulid, permit_type, description, status)
        VALUES (
            $1,
            (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            $2, $3, $4, $5
        )
        "#,
    )
    .bind(&permit_ulid)
    .bind(auth.user_id.to_string())
    .bind(&req.permit_type)
    .bind(&req.description)
    .bind(&req.status)
    .execute(conn.as_mut())
    .await?;

    debug!(permit = %permit_ulid, user = %auth.user_id, "permit created");
    Ok((
        StatusCode::CREATED,
        Json(PermitResponse {
            permit_id: permit_ulid,
            applicant_id: auth.user_id.to_string(),
            permit_type: req.permit_type,
            description: req.description,
            status: req.status,
            submitted_at: None,
        }),
    ))
}

/// List permits visible to the caller.
#[utoipa::path(
    get,
    path = "/api/v1/permits",
    tag = "Government",
    responses(
        (status = 200, description = "Permits visible to caller", body = [PermitResponse]),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_permits(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<PermitResponse>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<PermitRow> = sqlx::query_as(&format!("{PERMIT_SELECT}\nORDER BY ulid"))
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(PermitResponse::from).collect()))
}

/// Get a single permit by ULID.
#[utoipa::path(
    get,
    path = "/api/v1/permits/{id}",
    tag = "Government",
    params(("id" = String, Path, description = "Permit ULID")),
    responses(
        (status = 200, description = "Permit", body = PermitResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
        (status = 404, description = "Not found", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn get_permit(
    _auth: AuthContext,
    db: DbConn,
    Path(ulid): Path<String>,
) -> Result<Json<PermitResponse>, NexoraError> {
    let mut conn = db.acquire().await?;
    let row: Option<PermitRow> = sqlx::query_as(&format!("{PERMIT_SELECT}\nWHERE ulid = $1"))
        .bind(&ulid)
        .fetch_optional(conn.as_mut())
        .await?;
    let row = row.ok_or_else(|| NexoraError::NotFound(format!("permit {ulid} not found")))?;
    Ok(Json(row.into()))
}

/// Create a new license.
#[utoipa::path(
    post,
    path = "/api/v1/licenses",
    tag = "Government",
    request_body = CreateLicenseRequest,
    responses(
        (status = 201, description = "License created", body = LicenseResponse),
        (status = 400, description = "Validation failed", body = nexora_common::ErrorResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn create_license(
    auth: AuthContext,
    db: DbConn,
    Json(req): Json<CreateLicenseRequest>,
) -> Result<impl IntoResponse, NexoraError> {
    validate_license_request(&req)?;
    let mut conn = db.acquire().await?;
    let license_ulid = new_ulid();
    let issued = chrono::Utc::now();
    let expires = issued + chrono::Duration::days(365);
    sqlx::query(
        r#"
        INSERT INTO licenses (ulid, tenant_id, org_id, holder_ulid, license_type, holder_name, status, issued_at, expires_at)
        VALUES (
            $1,
            (SELECT id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            (SELECT org_id FROM tenants WHERE ulid = current_setting('nexora.current_tenant_id', true)),
            $2, $3, $4, $5, $6, $7
        )
        "#,
    )
    .bind(&license_ulid)
    .bind(auth.user_id.to_string())
    .bind(&req.license_type)
    .bind(&req.holder_name)
    .bind(&req.status)
    .bind(issued)
    .bind(expires)
    .execute(conn.as_mut())
    .await?;

    debug!(license = %license_ulid, user = %auth.user_id, "license created");
    Ok((
        StatusCode::CREATED,
        Json(LicenseResponse {
            license_id: license_ulid,
            holder_id: auth.user_id.to_string(),
            license_type: req.license_type,
            holder_name: req.holder_name,
            status: req.status,
            issued_at: Some(issued.to_rfc3339()),
            expires_at: Some(expires.to_rfc3339()),
        }),
    ))
}

/// List licenses visible to the caller.
#[utoipa::path(
    get,
    path = "/api/v1/licenses",
    tag = "Government",
    responses(
        (status = 200, description = "Licenses visible to caller", body = [LicenseResponse]),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_licenses(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<LicenseResponse>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<LicenseRow> = sqlx::query_as(&format!("{LICENSE_SELECT}\nORDER BY ulid"))
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(LicenseResponse::from).collect()))
}

/// Get a single license by ULID.
#[utoipa::path(
    get,
    path = "/api/v1/licenses/{id}",
    tag = "Government",
    params(("id" = String, Path, description = "License ULID")),
    responses(
        (status = 200, description = "License", body = LicenseResponse),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
        (status = 404, description = "Not found", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn get_license(
    _auth: AuthContext,
    db: DbConn,
    Path(ulid): Path<String>,
) -> Result<Json<LicenseResponse>, NexoraError> {
    let mut conn = db.acquire().await?;
    let row: Option<LicenseRow> = sqlx::query_as(&format!("{LICENSE_SELECT}\nWHERE ulid = $1"))
        .bind(&ulid)
        .fetch_optional(conn.as_mut())
        .await?;
    let row = row.ok_or_else(|| NexoraError::NotFound(format!("license {ulid} not found")))?;
    Ok(Json(row.into()))
}

/// List civil records visible to the caller.
#[utoipa::path(
    get,
    path = "/api/v1/records",
    tag = "Government",
    responses(
        (status = 200, description = "Records visible to caller", body = [RecordResponse]),
        (status = 401, description = "Unauthorized", body = nexora_common::ErrorResponse),
    ),
    security(("bearer" = []))
)]
pub async fn list_records(
    _auth: AuthContext,
    db: DbConn,
) -> Result<Json<Vec<RecordResponse>>, NexoraError> {
    let mut conn = db.acquire().await?;
    let rows: Vec<RecordRow> = sqlx::query_as(&format!("{RECORD_SELECT}\nORDER BY issued_at DESC LIMIT 100"))
        .fetch_all(conn.as_mut())
        .await?;
    Ok(Json(rows.into_iter().map(RecordResponse::from).collect()))
}

fn validate_permit_request(req: &CreatePermitRequest) -> Result<(), NexoraError> {
    if req.permit_type.trim().is_empty() {
        return Err(NexoraError::Validation("permit_type is required".into()));
    }
    if req.description.trim().is_empty() {
        return Err(NexoraError::Validation("description is required".into()));
    }
    if !VALID_PERMIT_STATUSES.contains(&req.status.as_str()) {
        return Err(NexoraError::Validation(format!(
            "invalid status {:?}: expected one of {VALID_PERMIT_STATUSES:?}",
            req.status
        )));
    }
    Ok(())
}

fn validate_license_request(req: &CreateLicenseRequest) -> Result<(), NexoraError> {
    if req.license_type.trim().is_empty() {
        return Err(NexoraError::Validation("license_type is required".into()));
    }
    if req.holder_name.trim().is_empty() {
        return Err(NexoraError::Validation("holder_name is required".into()));
    }
    if !VALID_LICENSE_STATUSES.contains(&req.status.as_str()) {
        return Err(NexoraError::Validation(format!(
            "invalid status {:?}: expected one of {VALID_LICENSE_STATUSES:?}",
            req.status
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permit_validation() {
        assert!(validate_permit_request(&CreatePermitRequest {
            permit_type: "building".into(),
            description: "Construct a shed".into(),
            status: "draft".into(),
        })
        .is_ok());
        assert!(validate_permit_request(&CreatePermitRequest {
            permit_type: "building".into(),
            description: "Construct a shed".into(),
            status: "exploded".into(),
        })
        .is_err());
    }

    #[test]
    fn license_validation() {
        assert!(validate_license_request(&CreateLicenseRequest {
            license_type: "driver".into(),
            holder_name: "Ada".into(),
            status: "pending".into(),
        })
        .is_ok());
        assert!(validate_license_request(&CreateLicenseRequest {
            license_type: "driver".into(),
            holder_name: "Ada".into(),
            status: "flying".into(),
        })
        .is_err());
    }
}
