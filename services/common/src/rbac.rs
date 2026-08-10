//! RBAC (Role-Based Access Control) policy middleware.
//!
//! Authorization is deny-by-default. Protected routes must configure an
//! explicit permission in [`RbacState`] or check [`AuthContextExt`] inline.

use crate::{tenant_context::AuthContext, AosError};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, warn};

/// State for a permission-protected router.
#[derive(Clone)]
pub struct RbacState {
    pub required_permission: &'static str,
    pub permission_resolver: Option<Arc<dyn PermissionResolver>>,
}

impl RbacState {
    pub const fn require(required_permission: &'static str) -> Self {
        Self {
            required_permission,
            permission_resolver: None,
        }
    }
}

/// Trait for custom authorization logic beyond static permissions.
#[async_trait::async_trait]
pub trait PermissionResolver: Send + Sync {
    async fn resolve(&self, context: &AuthContext) -> Result<bool, AosError>;
}

/// Middleware enforcing the permission configured in [`RbacState`].
pub async fn require_permission_middleware(
    State(state): State<RbacState>,
    request: Request,
    next: Next,
) -> Result<Response, AosError> {
    let auth_ctx = request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| AosError::Unauthorized("Authentication required".into()))?;

    let granted = if let Some(resolver) = &state.permission_resolver {
        resolver.resolve(auth_ctx).await?
    } else {
        auth_ctx.has_permission(state.required_permission)
    };

    if !granted {
        warn!(
            user_id = %auth_ctx.user_id,
            tenant_id = %auth_ctx.tenant_id,
            permission = state.required_permission,
            "Permission denied"
        );
        return Err(AosError::Forbidden(format!(
            "Permission required: {}",
            state.required_permission
        )));
    }

    debug!(
        user_id = %auth_ctx.user_id,
        permission = state.required_permission,
        "Permission granted"
    );
    Ok(next.run(request).await)
}

/// Helper to apply permission middleware to a router.
pub fn require_permission(
    router: axum::Router<RbacState>,
    state: RbacState,
) -> axum::Router<RbacState> {
    router.layer(axum::middleware::from_fn_with_state(
        state,
        require_permission_middleware,
    ))
}

/// Inline authorization checks for handlers and service methods.
pub trait AuthContextExt {
    fn require_permission(&self, perm: &str) -> Result<(), AosError>;
    fn require_any_permission(&self, perms: &[&str]) -> Result<(), AosError>;
    fn require_all_permissions(&self, perms: &[&str]) -> Result<(), AosError>;
    fn require_role(&self, role: &str) -> Result<(), AosError>;
    fn require_any_role(&self, roles: &[&str]) -> Result<(), AosError>;
}

impl AuthContextExt for AuthContext {
    fn require_permission(&self, perm: &str) -> Result<(), AosError> {
        self.has_permission(perm)
            .then_some(())
            .ok_or_else(|| AosError::Forbidden(format!("Permission required: {perm}")))
    }

    fn require_any_permission(&self, perms: &[&str]) -> Result<(), AosError> {
        perms
            .iter()
            .any(|p| self.has_permission(p))
            .then_some(())
            .ok_or_else(|| {
                AosError::Forbidden(format!(
                    "One of these permissions required: {}",
                    perms.join(", ")
                ))
            })
    }

    fn require_all_permissions(&self, perms: &[&str]) -> Result<(), AosError> {
        perms
            .iter()
            .all(|p| self.has_permission(p))
            .then_some(())
            .ok_or_else(|| {
                AosError::Forbidden(format!(
                    "All of these permissions required: {}",
                    perms.join(", ")
                ))
            })
    }

    fn require_role(&self, role: &str) -> Result<(), AosError> {
        self.has_role(role)
            .then_some(())
            .ok_or_else(|| AosError::Forbidden(format!("Role required: {role}")))
    }

    fn require_any_role(&self, roles: &[&str]) -> Result<(), AosError> {
        roles
            .iter()
            .any(|r| self.has_role(r))
            .then_some(())
            .ok_or_else(|| {
                AosError::Forbidden(format!("One of these roles required: {}", roles.join(", ")))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ulid::Ulid;

    fn context() -> AuthContext {
        AuthContext {
            tenant_id: Ulid::new(),
            org_id: Ulid::new(),
            user_id: Ulid::new(),
            keycloak_sub: "kc".into(),
            is_system: false,
            roles: vec!["HR_ADMIN".into(), "MANAGER".into(), "tenant_admin".into()],
            permissions: vec![
                "employee.read".into(),
                "employee.write".into(),
                "payroll.read".into(),
            ],
        }
    }

    #[test]
    fn inline_checks_are_deny_by_default() {
        let ctx = context();
        assert!(ctx.require_permission("employee.read").is_ok());
        assert!(ctx.require_permission("payroll.approve").is_err());
        assert!(ctx
            .require_any_permission(&["employee.read", "payroll.approve"])
            .is_ok());
        assert!(ctx
            .require_all_permissions(&["employee.read", "employee.write"])
            .is_ok());
        assert!(ctx
            .require_all_permissions(&["employee.read", "payroll.approve"])
            .is_err());
        assert!(ctx.require_role("HR_ADMIN").is_ok());
        assert!(ctx
            .require_any_role(&["HR_ADMIN", "MANAGER", "tenant_admin"])
            .is_ok());
    }
}
