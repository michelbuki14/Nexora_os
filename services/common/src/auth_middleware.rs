//! Authentication middleware bridging JWT claims to AuthContext.

use crate::{jwt::JwtValidator, tenant_context::AuthContext, ulid::Ulid, AosError};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, warn};

/// Extractor yielding the [`AuthContext`] when present, `None` for unauthenticated requests.
///
/// Axum cannot implement `FromRequestParts` for `Option<AuthContext>` directly: the
/// `S` state parameter does not appear on a local type in the impl head (E0210). A
/// dedicated newtype absorbs the optional case and gives handlers an unambiguous
/// authorization shape (see [`Self::0`]).
#[derive(Debug, Clone)]
pub struct OptionalAuthContext(pub Option<AuthContext>);

/// State for auth middleware.
#[derive(Clone)]
pub struct AuthState {
    pub validator: Arc<JwtValidator>,
}

/// Derive a stable [`Ulid`] from a Keycloak `sub` claim.
///
/// Keycloak's `sub` is a UUID (e.g. `b93af0e4-...`), which cannot be parsed as
/// a ULID. Rather than reject real tokens, hash the subject into a ULID so the
/// `user_id` field stays a valid `Ulid` while the raw subject remains available
/// as [`AuthContext::keycloak_sub`]. Deterministic for a given subject.
fn user_id_from_sub(sub: &str) -> Ulid {
    use sha2::{Digest, Sha256};
    let mut bytes = [0u8; 16];
    let digest = Sha256::digest(sub.as_bytes());
    bytes.copy_from_slice(&digest[..16]);
    Ulid::from_bytes(bytes)
}

/// Middleware that validates JWT and creates AuthContext.
///
/// This runs after jwt_middleware (which adds the validator to extensions)
/// and before rls_middleware (which needs AuthContext in extensions).
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AosError> {
    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AosError::Unauthorized("Missing Authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AosError::Unauthorized("Invalid Authorization header format".into()))?;

    // Validate token using the JWKS validator
    let mut claims = state.validator.validate(token).await.map_err(|e| {
        warn!("JWT validation failed: {}", e);
        AosError::Unauthorized("Invalid token".into())
    })?;

    // Build AuthContext from claims
    let tenant_id = claims
        .tenant_id
        .as_ref()
        .ok_or_else(|| AosError::Unauthorized("Token missing tenant_id claim".into()))?
        .clone();
    let org_id = claims
        .org_id
        .as_ref()
        .ok_or_else(|| AosError::Unauthorized("Token missing org_id claim".into()))?
        .clone();
    let user_id = claims.sub.clone();
    // Clone `sub` before moving it into `keycloak_sub` so the subsequent
    // `claims.has_role(...)` borrow is not a borrow-after-move (E0382).
    let keycloak_sub = claims.sub.clone();
    let is_system = claims.has_role("PLATFORM_ADMIN") || claims.has_role("SUPER_ADMIN");
    let roles = std::mem::take(&mut claims.roles);
    let permissions = std::mem::take(&mut claims.permissions);

    let auth_ctx = AuthContext {
        tenant_id: tenant_id
            .parse()
            .map_err(|_| AosError::Unauthorized("Invalid tenant_id format".into()))?,
        org_id: org_id
            .parse()
            .map_err(|_| AosError::Unauthorized("Invalid org_id format".into()))?,
        user_id: user_id_from_sub(&user_id),
        keycloak_sub,
        is_system,
        roles,
        permissions,
    };

    debug!(
        tenant_id = %auth_ctx.tenant_id,
        user_id = %auth_ctx.user_id,
        is_system = auth_ctx.is_system,
        "AuthContext created from JWT"
    );

    // Store AuthContext in extensions for downstream middleware/handlers
    request.extensions_mut().insert(auth_ctx);

    Ok(next.run(request).await)
}

/// Extractor for optional auth (endpoints that work with or without a logged-in user).
///
/// Returns the [`AuthContext`] that earlier middleware inserted into extensions, or
/// `None` when no auth context is present (health checks, public endpoints).
#[axum::async_trait]
impl<S> FromRequestParts<S> for OptionalAuthContext {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuthContext(
            parts.extensions.get::<AuthContext>().cloned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::AosClaims;

    fn test_claims() -> AosClaims {
        AosClaims {
            sub: "01ARZ3NDEKTSV4RRFFQ69G5FAV".into(),
            iss: "https://keycloak.example.com/realms/aos".into(),
            aud: vec!["aos-api".into()],
            exp: u64::MAX,
            iat: 0,
            jti: "test-jti".into(),
            azp: Some("aos-client".into()),
            scope: Some("openid profile email".into()),
            roles: vec!["HR_ADMIN".into(), "EMPLOYEE".into()],
            tenant_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAW".into()),
            org_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAX".into()),
            permissions: vec!["employee.read".into(), "employee.write".into()],
        }
    }

    #[test]
    fn auth_context_from_claims() {
        let claims = test_claims();
        let auth_ctx = AuthContext {
            tenant_id: claims.tenant_id.as_ref().unwrap().parse().unwrap(),
            org_id: claims.org_id.as_ref().unwrap().parse().unwrap(),
            user_id: claims.sub.parse().unwrap(),
            keycloak_sub: claims.sub.clone(),
            is_system: claims.has_role("PLATFORM_ADMIN") || claims.has_role("SUPER_ADMIN"),
            roles: claims.roles.clone(),
            permissions: claims.permissions.clone(),
        };

        assert_eq!(
            auth_ctx.tenant_id.to_string(),
            claims.tenant_id.as_deref().unwrap()
        );
        assert_eq!(
            auth_ctx.org_id.to_string(),
            claims.org_id.as_deref().unwrap()
        );
        assert_eq!(auth_ctx.user_id.to_string(), claims.sub);
        assert!(!auth_ctx.is_system);
        assert!(auth_ctx.has_role("HR_ADMIN"));
        assert!(auth_ctx.has_permission("employee.read"));
    }

    #[test]
    fn auth_context_system_flag() {
        let mut claims = test_claims();
        claims.roles.push("PLATFORM_ADMIN".into());

        let auth_ctx = AuthContext {
            tenant_id: claims.tenant_id.as_ref().unwrap().parse().unwrap(),
            org_id: claims.org_id.as_ref().unwrap().parse().unwrap(),
            user_id: claims.sub.parse().unwrap(),
            keycloak_sub: claims.sub.clone(),
            is_system: claims.has_role("PLATFORM_ADMIN") || claims.has_role("SUPER_ADMIN"),
            roles: claims.roles,
            permissions: claims.permissions,
        };

        assert!(auth_ctx.is_system);
    }
}
