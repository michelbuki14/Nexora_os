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
pub type NexoraResult<T> = Result<T, NexoraError>;

/// Application error types with HTTP status mapping.
#[derive(Debug, Error, ToSchema)]
#[serde(tag = "error", content = "details")]
pub enum NexoraError {
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

impl NexoraError {
    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            NexoraError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NexoraError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NexoraError::NotFound(_) => StatusCode::NOT_FOUND,
            NexoraError::Conflict(_) => StatusCode::CONFLICT,
            NexoraError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            NexoraError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            NexoraError::Forbidden(_) => StatusCode::FORBIDDEN,
            NexoraError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            NexoraError::BadRequest(_) => StatusCode::BAD_REQUEST,
            NexoraError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            NexoraError::GatewayTimeout(_) => StatusCode::GATEWAY_TIMEOUT,
            NexoraError::ExternalService(_) => StatusCode::BAD_GATEWAY,
            NexoraError::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NexoraError::Serialization(_) => StatusCode::BAD_REQUEST,
            NexoraError::TenantIsolation(_) => StatusCode::FORBIDDEN,
            NexoraError::IdempotencyConflict(_) => StatusCode::CONFLICT,
            NexoraError::InsufficientFunds(_) => StatusCode::UNPROCESSABLE_ENTITY,
            NexoraError::Compliance(_) => StatusCode::FORBIDDEN,
        }
    }

    /// Get a stable error code for client handling.
    pub fn error_code(&self) -> &'static str {
        match self {
            NexoraError::Internal(_) => "INTERNAL_ERROR",
            NexoraError::Config(_) => "CONFIG_ERROR",
            NexoraError::NotFound(_) => "NOT_FOUND",
            NexoraError::Conflict(_) => "CONFLICT",
            NexoraError::Validation(_) => "VALIDATION_ERROR",
            NexoraError::Unauthorized(_) => "UNAUTHORIZED",
            NexoraError::Forbidden(_) => "FORBIDDEN",
            NexoraError::RateLimited(_) => "RATE_LIMITED",
            NexoraError::BadRequest(_) => "BAD_REQUEST",
            NexoraError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            NexoraError::GatewayTimeout(_) => "GATEWAY_TIMEOUT",
            NexoraError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            NexoraError::Migration(_) => "MIGRATION_ERROR",
            NexoraError::Serialization(_) => "SERIALIZATION_ERROR",
            NexoraError::TenantIsolation(_) => "TENANT_ISOLATION_VIOLATION",
            NexoraError::IdempotencyConflict(_) => "IDEMPOTENCY_CONFLICT",
            NexoraError::InsufficientFunds(_) => "INSUFFICIENT_FUNDS",
            NexoraError::Compliance(_) => "COMPLIANCE_VIOLATION",
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
        NexoraError::Validation(messages.join("; "))
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

impl IntoResponse for NexoraError {
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

impl NexoraError {
    fn variant_name(&self) -> &'static str {
        match self {
            NexoraError::Internal(_) => "Internal",
            NexoraError::Config(_) => "Config",
            NexoraError::NotFound(_) => "NotFound",
            NexoraError::Conflict(_) => "Conflict",
            NexoraError::Validation(_) => "Validation",
            NexoraError::Unauthorized(_) => "Unauthorized",
            NexoraError::Forbidden(_) => "Forbidden",
            NexoraError::RateLimited(_) => "RateLimited",
            NexoraError::BadRequest(_) => "BadRequest",
            NexoraError::ServiceUnavailable(_) => "ServiceUnavailable",
            NexoraError::GatewayTimeout(_) => "GatewayTimeout",
            NexoraError::ExternalService(_) => "ExternalService",
            NexoraError::Migration(_) => "Migration",
            NexoraError::Serialization(_) => "Serialization",
            NexoraError::TenantIsolation(_) => "TenantIsolation",
            NexoraError::IdempotencyConflict(_) => "IdempotencyConflict",
            NexoraError::InsufficientFunds(_) => "InsufficientFunds",
            NexoraError::Compliance(_) => "Compliance",
        }
    }
}

impl From<anyhow::Error> for NexoraError {
    fn from(err: anyhow::Error) -> Self {
        NexoraError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for NexoraError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => NexoraError::NotFound("record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    NexoraError::Conflict("unique constraint violation".to_string())
                } else if db_err.is_foreign_key_violation() {
                    NexoraError::Conflict("foreign key violation".to_string())
                } else {
                    NexoraError::Internal(db_err.to_string())
                }
            }
            _ => NexoraError::Internal(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for NexoraError {
    fn from(err: serde_json::Error) -> Self {
        NexoraError::Serialization(err.to_string())
    }
}

impl From<validator::ValidationErrors> for NexoraError {
    fn from(err: validator::ValidationErrors) -> Self {
        NexoraError::from_validation_errors(err)
    }
}

impl From<uuid::Error> for NexoraError {
    fn from(err: uuid::Error) -> Self {
        NexoraError::Internal(format!("UUID error: {}", err))
    }
}
