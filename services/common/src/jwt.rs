//! JWT token validation and claims extraction.

use crate::{config::AuthConfig, AosError, AosResult};
use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use jsonwebtoken::{
    decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// JWT claims for AOS tokens.
///
/// Keycloak emits `aud` as a single string when there is one audience and as
/// an array when there are several, so `aud` uses a custom deserializer that
/// normalizes both forms into a `Vec<String>`. The actual audience check is
/// performed by `Validation::set_audience` during decoding, independent of
/// this field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AosClaims {
    pub sub: String,
    pub iss: String,
    #[serde(default, deserialize_with = "deserialize_aud")]
    pub aud: Vec<String>,
    pub exp: u64,
    #[serde(default)]
    pub iat: u64,
    #[serde(default)]
    pub jti: String,
    pub azp: Option<String>,
    pub scope: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    pub tenant_id: Option<String>,
    pub org_id: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Deserialize `aud` accepting either a single string or an array of strings.
fn deserialize_aud<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use std::collections::BTreeSet;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Aud {
        Single(String),
        Many(Vec<String>),
    }

    match Option::<Aud>::deserialize(deserializer)? {
        None => Ok(Vec::new()),
        Some(Aud::Single(s)) => Ok(vec![s]),
        Some(Aud::Many(v)) => Ok(v.into_iter().collect::<BTreeSet<_>>().into_iter().collect()),
    }
}

impl AosClaims {
    /// Check if token is expired.
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.exp < now
    }

    /// Check if token has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if token has a specific permission.
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Check if token belongs to a tenant.
    pub fn belongs_to_tenant(&self, tenant_id: &str) -> bool {
        self.tenant_id.as_deref() == Some(tenant_id)
    }

    /// Check if token belongs to an organization.
    pub fn belongs_to_org(&self, org_id: &str) -> bool {
        self.org_id.as_deref() == Some(org_id)
    }
}

/// JWT validator with JWKS caching.
#[derive(Clone)]
pub struct JwtValidator {
    config: AuthConfig,
    jwks: Arc<RwLock<Option<JwkSet>>>,
    last_fetch: Arc<RwLock<Option<SystemTime>>>,
}

impl JwtValidator {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            jwks: Arc::new(RwLock::new(None)),
            last_fetch: Arc::new(RwLock::new(None)),
        }
    }

    /// Get or fetch JWKS.
    async fn get_jwks(&self) -> AosResult<JwkSet> {
        let now = SystemTime::now();
        let should_refresh = {
            let last = self.last_fetch.read().await;
            last.is_none_or(|t| {
                now.duration_since(t).unwrap()
                    > Duration::from_secs(self.config.jwks_cache_ttl_secs)
            })
        };

        if should_refresh {
            debug!("Fetching JWKS from {}", self.config.jwks_url);
            let response = reqwest::get(&self.config.jwks_url)
                .await
                .map_err(|e| AosError::ExternalService(format!("Failed to fetch JWKS: {}", e)))?;
            let jwks = response
                .json::<JwkSet>()
                .await
                .map_err(|e| AosError::ExternalService(format!("Invalid JWKS: {}", e)))?;

            *self.jwks.write().await = Some(jwks.clone());
            *self.last_fetch.write().await = Some(now);
            Ok(jwks)
        } else {
            self.jwks
                .read()
                .await
                .clone()
                .ok_or_else(|| AosError::Internal("JWKS not available".to_string()))
        }
    }

    /// Validate a JWT token and extract claims.
    pub async fn validate(&self, token: &str) -> AosResult<AosClaims> {
        let header = decode_header(token)
            .map_err(|e| AosError::Unauthorized(format!("Invalid token header: {}", e)))?;

        let kid = header
            .kid
            .ok_or_else(|| AosError::Unauthorized("Missing key ID".to_string()))?;

        let jwks = self.get_jwks().await?;
        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| AosError::Unauthorized("Key not found in JWKS".to_string()))?;

        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(|e| AosError::Internal(format!("Invalid JWK: {}", e)))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.config.audience]);
        validation.set_issuer(&[&self.config.issuer]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = 30;

        let token_data: TokenData<AosClaims> = decode(token, &decoding_key, &validation)
            .map_err(|e| AosError::Unauthorized(format!("Token validation failed: {}", e)))?;

        if token_data.claims.is_expired() {
            return Err(AosError::Unauthorized("Token expired".to_string()));
        }

        Ok(token_data.claims)
    }
}

/// Extract JWT from Authorization header.
pub struct JwtExtractor(pub AosClaims);

#[async_trait]
impl<S> FromRequestParts<S> for JwtExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header"))?;

        let token = auth_header.strip_prefix("Bearer ").ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid Authorization header format",
        ))?;

        // The validator should be in extensions (added by middleware)
        let validator = parts.extensions.get::<Arc<JwtValidator>>().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "JWT validator not configured",
        ))?;

        let claims = validator.validate(token).await.map_err(|e| {
            warn!("JWT validation failed: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid token")
        })?;

        Ok(JwtExtractor(claims))
    }
}

/// Middleware to add JWT validator to request extensions.
pub async fn jwt_middleware(
    validator: Arc<JwtValidator>,
    mut req: Request,
    next: axum::middleware::Next,
) -> Result<Response, (StatusCode, &'static str)> {
    req.extensions_mut().insert(validator);
    Ok(next.run(req).await)
}

use axum::response::Response;
