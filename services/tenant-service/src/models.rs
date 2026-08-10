//! Tenant service request/response models and DB row mapping.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Valid tenant tiers — mirrors the `tenants.tier` CHECK constraint.
pub const VALID_TIERS: [&str; 5] = ["free", "starter", "professional", "enterprise", "custom"];

/// Create a new tenant (system/admin operation).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
    #[serde(default = "default_tier")]
    pub tier: String,
}

fn default_tier() -> String {
    "free".to_string()
}

/// Tenant representation returned by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct TenantResponse {
    pub tenant_id: String,
    pub org_id: String,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub tier: String,
}

/// A `tenants` row joined with its organization's ULID.
#[derive(Debug, sqlx::FromRow)]
pub struct TenantRow {
    pub ulid: String,
    pub org_ulid: String,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub tier: String,
}

impl From<TenantRow> for TenantResponse {
    fn from(row: TenantRow) -> Self {
        Self {
            tenant_id: row.ulid,
            org_id: row.org_ulid,
            name: row.name,
            slug: row.slug,
            status: row.status,
            tier: row.tier,
        }
    }
}
