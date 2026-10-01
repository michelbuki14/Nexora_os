//! Government service request/response models and DB row mapping.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Permit status values — mirrors the `permits.status` CHECK constraint.
pub const VALID_PERMIT_STATUSES: [&str; 4] = ["draft", "submitted", "approved", "rejected"];
/// License status values — mirrors the `licenses.status` CHECK constraint.
pub const VALID_LICENSE_STATUSES: [&str; 4] = ["active", "expired", "revoked", "pending"];

/// Create a new permit application.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePermitRequest {
    pub permit_type: String,
    pub description: String,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "draft".to_string()
}

/// Permit representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct PermitResponse {
    pub permit_id: String,
    pub applicant_id: String,
    pub permit_type: String,
    pub description: String,
    pub status: String,
    pub submitted_at: Option<String>,
}

/// A `permits` row.
#[derive(Debug, sqlx::FromRow)]
pub struct PermitRow {
    pub ulid: String,
    pub applicant_ulid: String,
    pub permit_type: String,
    pub description: String,
    pub status: String,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<PermitRow> for PermitResponse {
    fn from(row: PermitRow) -> Self {
        Self {
            permit_id: row.ulid,
            applicant_id: row.applicant_ulid,
            permit_type: row.permit_type,
            description: row.description,
            status: row.status,
            submitted_at: row.submitted_at.map(|d| d.to_rfc3339()),
        }
    }
}

/// Create a new license.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateLicenseRequest {
    pub license_type: String,
    pub holder_name: String,
    #[serde(default = "default_license_status")]
    pub status: String,
}

fn default_license_status() -> String {
    "pending".to_string()
}

/// License representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct LicenseResponse {
    pub license_id: String,
    pub holder_id: String,
    pub license_type: String,
    pub holder_name: String,
    pub status: String,
    pub issued_at: Option<String>,
    pub expires_at: Option<String>,
}

/// A `licenses` row.
#[derive(Debug, sqlx::FromRow)]
pub struct LicenseRow {
    pub ulid: String,
    pub holder_ulid: String,
    pub license_type: String,
    pub holder_name: String,
    pub status: String,
    pub issued_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<LicenseRow> for LicenseResponse {
    fn from(row: LicenseRow) -> Self {
        Self {
            license_id: row.ulid,
            holder_id: row.holder_ulid,
            license_type: row.license_type,
            holder_name: row.holder_name,
            status: row.status,
            issued_at: row.issued_at.map(|d| d.to_rfc3339()),
            expires_at: row.expires_at.map(|d| d.to_rfc3339()),
        }
    }
}

/// Civil record representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct RecordResponse {
    pub record_id: String,
    pub record_type: String,
    pub subject_name: String,
    pub issued_at: String,
    pub jurisdiction: String,
}

/// A `civil_records` row.
#[derive(Debug, sqlx::FromRow)]
pub struct RecordRow {
    pub ulid: String,
    pub record_type: String,
    pub subject_name: String,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub jurisdiction: String,
}

impl From<RecordRow> for RecordResponse {
    fn from(row: RecordRow) -> Self {
        Self {
            record_id: row.ulid,
            record_type: row.record_type,
            subject_name: row.subject_name,
            issued_at: row.issued_at.to_rfc3339(),
            jurisdiction: row.jurisdiction,
        }
    }
}
