//! Standardized error types for AOS services.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

/// Result type alias for AOS services.
pub type AosResult<T> = Result<T, AosError>;

/// Application error types with HTTP status mapping.
#[derive(Debug, Error, ToSchema)]
#[serde(tag = "error", content = "details")]
pub enum AosError {
    #[error("Internal server error: {0}")]
    #[schema(example = json!({"error": "Internal", "details": "database connection failed"}))]
    Internal(String),

    #[error("Configuration error: {0}")]
    #[schema(example = json!({"error": "Config", "details": "missing DATABASE_URL"}))]
    Config(String),

    #[error("Not found: {0}")]
    #[schema(example = json!({"error": "NotFound", "details": "tenant not found"}))]
    NotFound(String),

    #[error("Conflict: {0}")]
    #[schema(example = json!({"error": "Conflict", "details": "tenant already exists"}))]
    Conflict(String),

    #[error("Validation failed: {0}")]
    #[schema(example = json!({"error": "Validation", "details": "email is required"}))]
    Validation(String),

    #[error("Unauthorized: {0}")]
    #[schema(example = json!({"error": "Unauthorized", "details": "invalid token"}))]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    #[schema(example = json!({"error": "Forbidden", "details": "insufficient permissions"}))]
    Forbidden(String),

    #[error("Rate limited: {0}")]
    #[schema(example = json!({"error": "RateLimited", "details": "too many requests"}))]
    RateLimited(String),

    #[error("Bad request: {0}")]
    #[schema(example = json!({"error": "BadRequest", "details": "invalid JSON"}))]
    BadRequest(String),

    #[error("Service unavailable: {0}")]
    #[schema(example = json!({"error": "ServiceUnavailable", "details": "database offline"}))]
    ServiceUnavailable(String),

    #[error("Gateway timeout: {0}")]
    #[schema(example = json!({"error": "GatewayTimeout", "details": "upstream timeout"}))]
    GatewayTimeout(String),

    #[error("External service error: {0}")]
    #[schema(example = json!({"error": "ExternalService", "details": "payment provider timeout"}))]
    ExternalService(String),

    #[error("Migration error: {0}")]
    #[schema(example = json!({"error": "Migration", "details": "failed to apply migration"}))]
    Migration(String),

    #[error("Serialization error: {0}")]
    #[schema(example = json!({"error": "Serialization", "details": "invalid JSON"}))]
    Serialization(String),

    #[error("Tenant isolation violation: {0}")]
    #[schema(example = json!({"error": "TenantIsolation", "details": "cross-tenant access denied"}))]
    TenantIsolation(String),

    #[error("Idempotency conflict: {0}")]
    #[schema(example = json!({"error": "IdempotencyConflict", "details": "duplicate request"}))]
    IdempotencyConflict(String),

    #[error("Insufficient funds: {0}")]
    #[schema(example = json!({"error": "InsufficientFunds", "details": "wallet balance too low"}))]
    InsufficientFunds(String),

    #[error("Compliance violation: {0}")]
    #[schema(example = json!({"error": "Compliance", "details": "sanctions screening failed"}))]
    Compliance(String),
}

impl AosError {
    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            AosError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AosError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AosError::NotFound(_) => StatusCode::NOT_FOUND,
            AosError::Conflict(_) => StatusCode::CONFLICT,
            AosError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AosError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AosError::Forbidden(_) => StatusCode::FORBIDDEN,
            AosError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            AosError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AosError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AosError::GatewayTimeout(_) => StatusCode::GATEWAY_TIMEOUT,
            AosError::ExternalService(_) => StatusCode::BAD_GATEWAY,
            AosError::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AosError::Serialization(_) => StatusCode::BAD_REQUEST,
            AosError::TenantIsolation(_) => StatusCode::FORBIDDEN,
            AosError::IdempotencyConflict(_) => StatusCode::CONFLICT,
            AosError::InsufficientFunds(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AosError::Compliance(_) => StatusCode::FORBIDDEN,
        }
    }

    /// Get a stable error code for client handling.
    pub fn error_code(&self) -> &'static str {
        match self {
            AosError::Internal(_) => "INTERNAL_ERROR",
            AosError::Config(_) => "CONFIG_ERROR",
            AosError::NotFound(_) => "NOT_FOUND",
            AosError::Conflict(_) => "CONFLICT",
            AosError::Validation(_) => "VALIDATION_ERROR",
            AosError::Unauthorized(_) => "UNAUTHORIZED",
            AosError::Forbidden(_) => "FORBIDDEN",
            AosError::RateLimited(_) => "RATE_LIMITED",
            AosError::BadRequest(_) => "BAD_REQUEST",
            AosError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            AosError::GatewayTimeout(_) => "GATEWAY_TIMEOUT",
            AosError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            AosError::Migration(_) => "MIGRATION_ERROR",
            AosError::Serialization(_) => "SERIALIZATION_ERROR",
            AosError::TenantIsolation(_) => "TENANT_ISOLATION_VIOLATION",
            AosError::IdempotencyConflict(_) => "IDEMPOTENCY_CONFLICT",
            AosError::InsufficientFunds(_) => "INSUFFICIENT_FUNDS",
            AosError::Compliance(_) => "COMPLIANCE_VIOLATION",
        }
    }

    /// Create a validation error from validator crate errors.
    pub fn from_validation_errors(errors: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |e| {
                    let msg = e.message.as_deref().unwrap_or("invalid value");
                    format!("{field}: {msg}")
                })
            })
            .collect();
        AosError::Validation(messages.join("; "))
    }
}

/// Standardized error response body.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub error_code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
    pub timestamp: String,
}

impl IntoResponse for AosError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();

        let body = ErrorResponse {
            error: self.variant_name().to_string(),
            error_code: error_code.to_string(),
            message,
            details: None,
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        (status, Json(body)).into_response()
    }
}

impl AosError {
    fn variant_name(&self) -> &'static str {
        match self {
            AosError::Internal(_) => "Internal",
            AosError::Config(_) => "Config",
            AosError::NotFound(_) => "NotFound",
            AosError::Conflict(_) => "Conflict",
            AosError::Validation(_) => "Validation",
            AosError::Unauthorized(_) => "Unauthorized",
            AosError::Forbidden(_) => "Forbidden",
            AosError::RateLimited(_) => "RateLimited",
            AosError::BadRequest(_) => "BadRequest",
            AosError::ServiceUnavailable(_) => "ServiceUnavailable",
            AosError::GatewayTimeout(_) => "GatewayTimeout",
            AosError::ExternalService(_) => "ExternalService",
            AosError::Migration(_) => "Migration",
            AosError::Serialization(_) => "Serialization",
            AosError::TenantIsolation(_) => "TenantIsolation",
            AosError::IdempotencyConflict(_) => "IdempotencyConflict",
            AosError::InsufficientFunds(_) => "InsufficientFunds",
            AosError::Compliance(_) => "Compliance",
        }
    }
}

impl From<anyhow::Error> for AosError {
    fn from(err: anyhow::Error) -> Self {
        AosError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for AosError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AosError::NotFound("record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    AosError::Conflict("unique constraint violation".to_string())
                } else if db_err.is_foreign_key_violation() {
                    AosError::Conflict("foreign key violation".to_string())
                } else {
                    AosError::Internal(db_err.to_string())
                }
            }
            _ => AosError::Internal(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for AosError {
    fn from(err: serde_json::Error) -> Self {
        AosError::Serialization(err.to_string())
    }
}

impl From<validator::ValidationErrors> for AosError {
    fn from(err: validator::ValidationErrors) -> Self {
        AosError::from_validation_errors(err)
    }
}

impl From<uuid::Error> for AosError {
    fn from(err: uuid::Error) -> Self {
        AosError::Internal(format!("UUID error: {}", err))
    }
}
