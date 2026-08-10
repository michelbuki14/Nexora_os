//! Multi-tenancy support with tenant context and isolation.

use crate::{ulid::Ulid, AosResult};
use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::request::Parts,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Tenant context for request-scoped tenant information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: Ulid,
    pub org_id: Ulid,
    pub name: String,
    pub slug: String,
    pub status: TenantStatus,
    pub tier: TenantTier,
    pub features: HashMap<String, bool>,
    pub quotas: TenantQuotas,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tenant status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    Active,
    Suspended,
    PendingVerification,
    Deleted,
}

/// Tenant subscription tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantTier {
    Free,
    Starter,
    Professional,
    Enterprise,
    Custom,
}

/// Tenant resource quotas.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantQuotas {
    pub max_users: Option<u32>,
    pub max_api_calls_per_month: Option<u64>,
    pub max_storage_gb: Option<u64>,
    pub max_workflows: Option<u32>,
    pub custom: HashMap<String, u64>,
}

impl TenantContext {
    /// Check if tenant has a feature enabled.
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }

    /// Check if tenant is active.
    pub fn is_active(&self) -> bool {
        matches!(self.status, TenantStatus::Active)
    }

    /// Get quota for a resource.
    pub fn get_quota(&self, resource: &str) -> Option<u64> {
        self.quotas.custom.get(resource).copied()
    }
}

/// Extract tenant from request (set by auth middleware).
#[derive(Clone)]
pub struct TenantExtractor(pub TenantContext);

#[async_trait]
impl<S> FromRequestParts<S> for TenantExtractor
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<TenantContext>()
            .cloned()
            .map(TenantExtractor)
            .ok_or((
                axum::http::StatusCode::FORBIDDEN,
                "Tenant context not found",
            ))
    }
}

/// Middleware to resolve tenant from request.
pub async fn tenant_resolution_middleware(
    tenant_service: Arc<dyn TenantResolver>,
    mut req: Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, (axum::http::StatusCode, &'static str)> {
    // Try to get tenant from header (for API calls)
    let tenant_id = req
        .headers()
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<Ulid>().ok());

    // Try to get from JWT claims (set by auth middleware)
    let claims_tenant = req
        .extensions()
        .get::<crate::jwt::AosClaims>()
        .and_then(|c| c.tenant_id.as_ref())
        .and_then(|s| s.parse::<Ulid>().ok());

    let tenant_id = tenant_id.or(claims_tenant);

    if let Some(tid) = tenant_id {
        match tenant_service.resolve_tenant(tid).await {
            Ok(Some(ctx)) if ctx.is_active() => {
                req.extensions_mut().insert(ctx);
            }
            Ok(Some(_)) => {
                return Err((axum::http::StatusCode::FORBIDDEN, "Tenant suspended"));
            }
            Ok(None) => {
                return Err((axum::http::StatusCode::NOT_FOUND, "Tenant not found"));
            }
            Err(e) => {
                tracing::error!("Tenant resolution failed: {}", e);
                return Err((
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Tenant resolution failed",
                ));
            }
        }
    }

    Ok(next.run(req).await)
}

/// Trait for tenant resolution.
#[async_trait]
pub trait TenantResolver: Send + Sync {
    async fn resolve_tenant(&self, tenant_id: Ulid) -> AosResult<Option<TenantContext>>;
    async fn resolve_by_slug(&self, slug: &str) -> AosResult<Option<TenantContext>>;
}
