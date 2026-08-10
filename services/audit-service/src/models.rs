//! Audit service data models matching the `audit_events` schema.
//!
//! All API models use [`uuid::Uuid`] for the database-backed identifiers
//! (actor_id, resource_id) and plain `String` for ULIDs (which are `CHAR(26)`
//! in the database). IP addresses are exposed as strings on the API surface
//! because `utoipa::ToSchema` is not implemented for `std::net::IpAddr`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Database row for the `audit_events` table.
///
/// `tenant_id`, `org_id`, `actor_id` and `resource_id` are PostgreSQL `UUID`
/// columns; `ulid` and `resource_ulid` are `CHAR(26)`; `ip_address` is `INET`.
#[derive(Debug, Clone, FromRow)]
pub struct AuditEventRow {
    pub id: i64,
    pub ulid: String,
    pub tenant_id: Uuid,
    pub org_id: Uuid,
    pub actor_type: String,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub resource_ulid: Option<String>,
    pub changes: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
    /// IP address stored as text (postgres `INET` cast to `text` on read; the
    /// `ipnetwork`/`IpAddr` sqlx mapping requires an extra crate feature we
    /// don't take on so the row stays plain).
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub result: String,
    pub error_message: Option<String>,
    pub hash: String,
    pub created_at: DateTime<Utc>,
}

/// Who or what performed the audited action.
///
/// Stored as `TEXT` in the database with a `CHECK` constraint over the four
/// allowed lowercase values.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    User,
    System,
    Service,
    ApiKey,
}

impl ActorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActorType::User => "user",
            ActorType::System => "system",
            ActorType::Service => "service",
            ActorType::ApiKey => "api_key",
        }
    }
}

impl std::fmt::Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ActorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(ActorType::User),
            "system" => Ok(ActorType::System),
            "service" => Ok(ActorType::Service),
            "api_key" => Ok(ActorType::ApiKey),
            other => Err(format!("invalid actor type: {other}")),
        }
    }
}

/// Outcome of the audited action.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Success,
    Failure,
    Partial,
}

impl AuditResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditResult::Success => "success",
            AuditResult::Failure => "failure",
            AuditResult::Partial => "partial",
        }
    }
}

impl std::fmt::Display for AuditResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for AuditResult {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "success" => Ok(AuditResult::Success),
            "failure" => Ok(AuditResult::Failure),
            "partial" => Ok(AuditResult::Partial),
            other => Err(format!("invalid audit result: {other}")),
        }
    }
}

/// Request to create an audit event.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAuditRequest {
    /// Type of actor performing the action.
    pub actor_type: ActorType,
    /// Optional actor identifier (user/service UUID).
    pub actor_id: Option<Uuid>,
    /// Action performed (e.g. `tenant.create`, `user.delete`).
    pub action: String,
    /// Type of resource affected (e.g. `tenant`, `user`).
    pub resource_type: String,
    /// Optional resource UUID.
    pub resource_id: Option<Uuid>,
    /// Optional resource ULID for cross-referencing.
    pub resource_ulid: Option<String>,
    /// JSON diff of changes made (create/update operations).
    pub changes: Option<serde_json::Value>,
    /// Additional metadata for the event.
    pub metadata: Option<serde_json::Value>,
    /// Request ID for correlation.
    pub request_id: Option<String>,
    /// Correlation ID for tracing across services.
    pub correlation_id: Option<String>,
    /// Client IP address (string form, e.g. `203.0.113.5`).
    pub ip_address: Option<String>,
    /// Client user agent.
    pub user_agent: Option<String>,
    /// Outcome of the action.
    pub result: AuditResult,
    /// Error message when `result` is `failure` or `partial`.
    pub error_message: Option<String>,
}

/// Response for a single audit event.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuditEventResponse {
    pub id: i64,
    pub ulid: String,
    pub tenant_id: Uuid,
    pub org_id: Uuid,
    pub actor_type: ActorType,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub resource_ulid: Option<String>,
    pub changes: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
    /// IP address as a string (API surface; `IpAddr` has no OpenAPI schema).
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub result: AuditResult,
    pub error_message: Option<String>,
    pub hash: String,
    pub created_at: DateTime<Utc>,
}

impl From<AuditEventRow> for AuditEventResponse {
    fn from(row: AuditEventRow) -> Self {
        let actor_type = row.actor_type.parse().unwrap_or(ActorType::System);
        let result = row.result.parse().unwrap_or(AuditResult::Success);
        Self {
            id: row.id,
            ulid: row.ulid,
            tenant_id: row.tenant_id,
            org_id: row.org_id,
            actor_type,
            actor_id: row.actor_id,
            action: row.action,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            resource_ulid: row.resource_ulid,
            changes: row.changes,
            metadata: row.metadata,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            result,
            error_message: row.error_message,
            hash: row.hash,
            created_at: row.created_at,
        }
    }
}

/// Paginated list response.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuditEventListResponse {
    pub events: Vec<AuditEventResponse>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

/// Query parameters for listing audit events (extracted from the query string).
#[derive(Debug, Clone, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct AuditListQuery {
    /// Page number (1-indexed).
    #[serde(default = "default_page")]
    #[param(default = 1)]
    pub page: u32,
    /// Page size (max 100).
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// Filter by actor type.
    pub actor_type: Option<String>,
    /// Filter by action.
    pub action: Option<String>,
    /// Filter by resource type.
    pub resource_type: Option<String>,
    /// Filter by resource ID.
    pub resource_id: Option<Uuid>,
    /// Filter by result.
    pub result: Option<String>,
    /// Filter by start date (inclusive, RFC3339).
    pub since: Option<String>,
    /// Filter by end date (inclusive, RFC3339).
    pub until: Option<String>,
    /// Filter by correlation ID.
    pub correlation_id: Option<String>,
}

impl AuditListQuery {
    /// Clamp pagination to sane bounds.
    pub fn sanitized(mut self) -> Self {
        if self.page == 0 {
            self.page = 1;
        }
        if self.page_size == 0 {
            self.page_size = default_page_size();
        }
        if self.page_size > 100 {
            self.page_size = 100;
        }
        self
    }

    /// Offset for SQL `LIMIT`/`OFFSET`.
    pub fn offset(&self) -> i64 {
        ((self.page.saturating_sub(1)) as i64).saturating_mul(self.page_size as i64)
    }
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

/// Hash-chain verification result.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HashChainVerification {
    pub verified: bool,
    pub events_checked: u64,
    pub first_mismatch: Option<i64>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_type_roundtrip() {
        for v in ["user", "system", "service", "api_key"] {
            let t: ActorType = v.parse().unwrap();
            assert_eq!(t.to_string(), v);
        }
        assert!("robot".parse::<ActorType>().is_err());
    }

    #[test]
    fn audit_result_roundtrip() {
        for v in ["success", "failure", "partial"] {
            let r: AuditResult = v.parse().unwrap();
            assert_eq!(r.to_string(), v);
        }
        assert!("boom".parse::<AuditResult>().is_err());
    }

    #[test]
    fn pagination_is_clamped() {
        let q = AuditListQuery {
            page: 0,
            page_size: 999,
            actor_type: None,
            action: None,
            resource_type: None,
            resource_id: None,
            result: None,
            since: None,
            until: None,
            correlation_id: None,
        }
        .sanitized();
        assert_eq!(q.page, 1);
        assert_eq!(q.page_size, 100);
        assert_eq!(q.offset(), 0);

        let q2 = AuditListQuery {
            page: 3,
            page_size: 20,
            actor_type: None,
            action: None,
            resource_type: None,
            resource_id: None,
            result: None,
            since: None,
            until: None,
            correlation_id: None,
        }
        .sanitized();
        assert_eq!(q2.offset(), 40);
    }
}
