//! Idempotency middleware for deduplicating requests.
//!
//! Provides an Axum middleware that checks for an `Idempotency-Key` header
//! and deduplicates requests by caching responses in the `idempotency_records` table.

use crate::{NexoraError, NexoraResult, AuthContext};
use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use uuid::Uuid;

/// The header name for the idempotency key.
pub const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";

/// Maximum time-to-live for pending idempotency records (7 days).
const PENDING_TTL_DAYS: i64 = 7;

/// Cleanup expired idempotency records older than PENDING_TTL_DAYS.
/// Should be run periodically (e.g., via cron or background job).
pub async fn cleanup_idempotency_records(pool: &sqlx::PgPool) -> NexoraResult<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM idempotency_records
        WHERE status = 'pending'
        AND created_at < NOW() - INTERVAL '7 days'
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Idempotency state shared across middleware invocations.
#[derive(Clone)]
pub struct IdempotencyState {
    pub pool: sqlx::PgPool,
}

/// Cached response from a previous request.
/// Headers stored as (name, value) string tuples for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

/// Idempotency middleware.
///
/// Checks for an `Idempotency-Key` header on mutating requests (POST, PUT, PATCH, DELETE).
/// If present:
/// - Looks up existing record by (tenant_id, idempotency_key, operation)
/// - If found with status 'completed', returns the cached response
/// - If found with status 'pending', returns 409 Conflict (concurrent request)
/// - If not found, creates a 'pending' record, runs the handler, then updates with response
///
/// The operation type is derived from the request path for simplicity.
pub async fn idempotency_middleware(
    State(state): State<IdempotencyState>,
    request: Request,
    next: Next,
) -> Result<Response, NexoraError> {
    // Only apply to mutating methods
    let method = request.method().clone();
    if !matches!(
        method,
        axum::http::Method::POST
            | axum::http::Method::PUT
            | axum::http::Method::PATCH
            | axum::http::Method::DELETE
    ) {
        return Ok(next.run(request).await);
    }

    // Extract idempotency key from header
    let idempotency_key = match extract_idempotency_key(&request) {
        Some(key) => key,
        None => {
            debug!("No idempotency key provided for mutating request; skipping deduplication");
            return Ok(next.run(request).await);
        }
    };

    // Extract AuthContext from request extensions (set by auth middleware)
    let auth = request.extensions().get::<AuthContext>().cloned();
    let Some(auth) = auth else {
        debug!("No auth context; skipping idempotency check");
        return Ok(next.run(request).await);
    };

    // Determine operation type from path
    let operation = operation_from_path(request.uri().path());

    // Acquire DB connection
    let pool = state.pool.clone();

    // Check for existing idempotency record
    let existing = lookup_idempotency_record(
        &pool,
        auth.tenant_id.to_string(),
        &idempotency_key,
        &operation,
    )
    .await?;

    if let Some(ref record) = existing {
        match record.status.as_str() {
            "completed" => {
                // Return cached response
                debug!(
                    idempotency_key = %idempotency_key,
                    operation = %operation,
                    "Returning cached idempotent response"
                );
                // Record is consumed by cached_response_to_axum, so we return early
                return Ok(cached_response_to_axum(&record));
            }
            "pending" => {
                // Concurrent request with same key - return conflict
                warn!(
                    idempotency_key = %idempotency_key,
                    operation = %operation,
                    "Concurrent request with same idempotency key"
                );
                return Err(NexoraError::IdempotencyConflict(format!(
                    "Request with idempotency key '{}' is already being processed",
                    idempotency_key
                )));
            }
            "failed" => {
                // Previous attempt failed - allow retry by treating as new request
                debug!(
                    idempotency_key = %idempotency_key,
                    operation = %operation,
                    "Previous idempotent request failed; allowing retry"
                );
                // Fall through to create new record below
            }
            _ => {
                // Unknown status - treat as new request
                debug!("Unknown idempotency status: {}", record.status);
            }
        }
    }
    // If status was "failed" or unknown, we fall through to create a new record

    // Create new pending record
    let record_id = create_pending_idempotency_record(
        &pool,
        auth.tenant_id.to_string(),
        auth.org_id.to_string(),
        auth.user_id.to_string(),
        &idempotency_key,
        &operation,
    )
    .await?;

    // Run the handler
    let response = next.run(request).await;

    // Capture response body and headers
    let status_code = response.status().as_u16();
    let (cached_headers, cached_body) = capture_response_parts(response).await?;
    let record_status = if status_code < 500 {
        "completed"
    } else {
        "failed"
    };

    update_idempotency_record(
        &pool,
        &record_id,
        &CachedResponse {
            status_code,
            headers: cached_headers,
            body: cached_body,
        },
        record_status,
    )
    .await?;

    // Return the cached response from the database record for idempotent re-requests.
    // The middleware runs after the handler completes, so the original response was already sent.
    // However, for the idempotency pattern to work correctly on re-requests, we return the
    // cached response from the database record so subsequent identical requests get consistent data.
    if let Some(record) = existing {
        return Ok(cached_response_to_axum(&record));
    }

    // If we get here without an existing record (shouldn't happen since we checked above),
    // return a simple success response
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Idempotent request processed"))
        .unwrap())
}

/// Extract idempotency key from request header.
fn extract_idempotency_key(request: &Request) -> Option<String> {
    request
        .headers()
        .get(IDEMPOTENCY_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Derive operation type from request path.
fn operation_from_path(path: &str) -> String {
    // Normalize path for operation identification
    // e.g., "/api/v1/payments/batches" -> "create_payment_batch"
    // "/api/v1/payroll/runs" -> "create_payroll_run"
    let path = path.trim_start_matches('/');
    let parts: Vec<&str> = path.split('/').collect();

    // Look for known patterns
    if parts.len() >= 3 {
        let domain = parts.get(2).copied().unwrap_or("");
        let resource = parts.get(3).copied().unwrap_or("");

        match (domain, resource) {
            ("payments", "batches") => "create_payment_batch".to_string(),
            ("payments", "orders") => "create_payment_order".to_string(),
            ("payroll", "runs") => "create_payroll_run".to_string(),
            ("payroll", "payslips") => "generate_payslip".to_string(),
            ("workforce", "employees") => "create_employee".to_string(),
            ("workforce", "contracts") => "create_contract".to_string(),
            ("workforce", "documents") => "upload_document".to_string(),
            ("finance", "journal-entries") => "create_journal_entry".to_string(),
            ("tenants", _) => "create_tenant".to_string(),
            ("organizations", _) => "create_organization".to_string(),
            _ => format!("{}_{}", domain, resource).replace('/', "_"),
        }
    } else {
        "unknown".to_string()
    }
}

/// Look up an existing idempotency record.
async fn lookup_idempotency_record(
    pool: &sqlx::PgPool,
    tenant_id: String,
    idempotency_key: &str,
    operation: &str,
) -> NexoraResult<Option<IdempotencyRecord>> {
    let row = sqlx::query_as::<_, IdempotencyRecord>(
        r#"
        SELECT id, tenant_id, org_id, requestor_id, idempotency_key, operation,
               status, response_payload, response_status_code, created_at, updated_at, completed_at
        FROM idempotency_records
        WHERE tenant_id = $1 AND idempotency_key = $2 AND operation = $3
        "#,
    )
    .bind(Uuid::parse_str(&tenant_id)?)
    .bind(Uuid::parse_str(idempotency_key)?)
    .bind(operation)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Create a new pending idempotency record.
async fn create_pending_idempotency_record(
    pool: &sqlx::PgPool,
    tenant_id: String,
    org_id: String,
    requestor_id: String,
    idempotency_key: &str,
    operation: &str,
) -> NexoraResult<Uuid> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO idempotency_records (tenant_id, org_id, requestor_id, idempotency_key, operation, status)
        VALUES ($1, $2, $3, $4, $5, 'pending')
        RETURNING id
        "#,
    )
    .bind(Uuid::parse_str(&tenant_id)?)
    .bind(Uuid::parse_str(&org_id)?)
    .bind(Uuid::parse_str(&requestor_id)?)
    .bind(Uuid::parse_str(idempotency_key)?)
    .bind(operation)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// Capture header names/values and body from a response safely.
/// Stores headers as (String, String) for serialization.
async fn capture_response_parts(response: Response) -> NexoraResult<(Vec<(String, String)>, Vec<u8>)> {
    let mut headers = Vec::new();

    for (name, value) in response.headers() {
        // Store header as string tuple for serialization
        let header_name = name.as_str().to_string();

        // Convert header value to string, skipping non-UTF8 values
        if let Ok(header_value) = value.to_str() {
            headers.push((header_name, header_value.to_string()));
        } else {
            warn!("Non-UTF8 header value skipped for name: {}", header_name);
        }
    }

    // Capture response body (up to 10MB limit)
    // to_bytes consumes the body
    let body_bytes = to_bytes(response.into_body(), 10 * 1024 * 1024)
        .await
        .map_err(|e| NexoraError::Internal(format!("Failed to read response body: {}", e)))?;

    Ok((headers, body_bytes.to_vec()))
}

/// Update idempotency record with cached response.
async fn update_idempotency_record(
    pool: &sqlx::PgPool,
    record_id: &Uuid,
    cached: &CachedResponse,
    status: &str,
) -> NexoraResult<()> {
    let completed_at = if status == "completed" {
        Some(chrono::Utc::now())
    } else {
        None
    };

    sqlx::query(
        r#"
        UPDATE idempotency_records
        SET status = $1,
            response_payload = $2,
            response_status_code = $3,
            completed_at = $4,
            updated_at = NOW()
        WHERE id = $5
        "#,
    )
    .bind(status)
    .bind(serde_json::to_value(cached)?)
    .bind(cached.status_code as i16)
    .bind(completed_at)
    .bind(record_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update idempotency record status only.
async fn update_idempotency_record_status(
    pool: &sqlx::PgPool,
    record_id: &Uuid,
    status: &str,
) -> NexoraResult<()> {
    sqlx::query(
        r#"
        UPDATE idempotency_records
        SET status = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(status)
    .bind(record_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Convert a database record to an Axum response.
fn cached_response_to_axum(record: &IdempotencyRecord) -> Response {
    let cached: CachedResponse = match serde_json::from_value(record.response_payload.clone()) {
        Ok(c) => c,
        Err(_) => {
            // Fallback if deserialization fails
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to deserialize cached response"))
                .unwrap();
        }
    };

    let mut builder = Response::builder().status(cached.status_code);

    for (header_name, header_value) in &cached.headers {
        // Parse header name and value back to typed variants
        if let Ok(parsed_name) = header_name.parse::<HeaderName>() {
            if let Ok(parsed_value) = header_value.parse::<HeaderValue>() {
                builder = builder.header(parsed_name, parsed_value);
            }
        }
    }

    builder.body(Body::from(cached.body)).unwrap()
}

/// Database record for idempotency records.
#[derive(sqlx::FromRow, Debug)]
#[allow(clippy::unused_fields)]
struct IdempotencyRecord {
    id: Uuid,
    tenant_id: Uuid,
    org_id: Uuid,
    requestor_id: Uuid,
    idempotency_key: Uuid,
    operation: String,
    status: String,
    response_payload: serde_json::Value,
    response_status_code: Option<i16>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Builder for adding idempotency middleware to a router.
pub fn add_idempotency_middleware(
    router: axum::Router<IdempotencyState>,
    state: IdempotencyState,
) -> axum::Router<IdempotencyState> {
    let middleware_fn = {
        let state = state.clone();
        move |request: Request, next: Next| {
            let state = state.clone();
            async move { idempotency_middleware(State(state), request, next).await }
        }
    };
    router.layer(axum::middleware::from_fn(middleware_fn))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_from_path() {
        assert_eq!(
            operation_from_path("/api/v1/payments/batches"),
            "create_payment_batch"
        );
        assert_eq!(
            operation_from_path("/api/v1/payments/orders"),
            "create_payment_order"
        );
        assert_eq!(
            operation_from_path("/api/v1/payroll/runs"),
            "create_payroll_run"
        );
        assert_eq!(
            operation_from_path("/api/v1/payroll/payslips"),
            "generate_payslip"
        );
        assert_eq!(
            operation_from_path("/api/v1/workforce/employees"),
            "create_employee"
        );
        assert_eq!(
            operation_from_path("/api/v1/workforce/contracts"),
            "create_contract"
        );
        assert_eq!(
            operation_from_path("/api/v1/workforce/documents"),
            "upload_document"
        );
        assert_eq!(
            operation_from_path("/api/v1/finance/journal-entries"),
            "create_journal_entry"
        );
        assert_eq!(operation_from_path("/api/v1/tenants"), "create_tenant");
        assert_eq!(
            operation_from_path("/api/v1/organizations"),
            "create_organization"
        );
        assert_eq!(
            operation_from_path("/api/v1/unknown/resource"),
            "unknown_resource"
        );
    }
}
