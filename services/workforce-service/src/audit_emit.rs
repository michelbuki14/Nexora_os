//! Workforce audit emission helpers.
//!
//! Every workforce lifecycle action emits two records in the SAME database
//! transaction via a single `&mut PgConnection`:
//!
//!   1. A hash-chained row in `audit_events` — tamper-evident, replayed by
//!      `verify-hash-chain`. Same canonical-payload algorithm as audit-service.
//!   2. An `outbox_events` row with `aggregate_type='workforce'` — consumed by
//!      the future outbox dispatcher job (payroll feed, notifications). Never
//!      published directly; the dispatcher job polls `published_at IS NULL`.
//!
//! Dual-write on the same connection means both rows are inside the
//! `rls_middleware` transaction and roll back together on any error.
//!
//! Security-monitoring events (salary.changed, privilege.escalated,
//! employee.exported, mass_employee_update, suspicious_document_access) route
//! through this same helper — the `action` string IS the signal.

use nexora_common::{
    audit::{compute_chain_hash, serialize_canonical, GENESIS_HASH},
    config::Config,
    error::{NexoraError, NexoraResult},
    tenant_context::AuthContext,
    ulid::new_ulid,
};
use serde_json::Value;
use sqlx::PgConnection;

/// Emit a workforce audit event + outbox event in `conn` (must be in a tx).
///
/// `resource_type`: e.g. `"employee"`, `"department"`, `"compensation"`.
/// `resource_ulid`: the external ULID of the affected row.
/// `action`: e.g. `"employee.lifecycle.created"`, `"salary.changed"`.
/// `changes`: JSON diff / before-after. Redact sensitive fields before passing:
///   never log national_id, raw compensation amounts, or document bytes.
///
/// `resource_uuid`: the UUID primary key of the affected row, required for
/// `outbox_events.aggregate_id` (UUID NOT NULL). Pass `uuid::Uuid::nil()` for
/// virtual resources (e.g. bulk export events that have no single row UUID).
// The signature is intentionally wide: it mirrors the audit_events row 1:1 and
// every caller passes all fields. Bundling into a context struct would churn
// ~10 call sites for no behavior change.
#[allow(clippy::too_many_arguments)]
pub async fn emit_workforce_event(
    conn: &mut PgConnection,
    config: &Config,
    auth: &AuthContext,
    tenant_uuid: uuid::Uuid,
    org_uuid: uuid::Uuid,
    resource_type: &str,
    resource_ulid: &str,
    action: &str,
    changes: Value,
) -> NexoraResult<()> {
    emit_workforce_event_with_uuid(
        conn,
        config,
        auth,
        tenant_uuid,
        org_uuid,
        resource_type,
        resource_ulid,
        uuid::Uuid::nil(),
        action,
        changes,
    )
    .await
}

/// Like `emit_workforce_event` but with an explicit `resource_uuid` for the
/// outbox `aggregate_id` column.
#[allow(clippy::too_many_arguments)]
pub async fn emit_workforce_event_with_uuid(
    conn: &mut PgConnection,
    config: &Config,
    auth: &AuthContext,
    tenant_uuid: uuid::Uuid,
    org_uuid: uuid::Uuid,
    resource_type: &str,
    resource_ulid: &str,
    resource_uuid: uuid::Uuid,
    action: &str,
    changes: Value,
) -> NexoraResult<()> {
    let event_ulid = new_ulid();
    let actor_id = auth.user_id.to_string();
    let tenant_ulid = auth.tenant_id.to_string();

    // --- 1. Compute hash chain ------------------------------------------
    let prev_hash: String = sqlx::query_scalar(
        "SELECT hash FROM audit_events WHERE tenant_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(tenant_uuid)
    .fetch_optional(&mut *conn)
    .await?
    .unwrap_or_else(|| GENESIS_HASH.to_string());

    let payload_value = serde_json::json!({
        "ulid": event_ulid,
        "tenant_id": tenant_ulid,
        "actor_id": actor_id,
        "action": action,
        "resource_type": resource_type,
        "resource_ulid": resource_ulid,
    });
    let canonical = serialize_canonical(&payload_value)?;
    let hash = compute_chain_hash(config, &tenant_uuid, &prev_hash, &canonical);

    // --- 2. Insert audit_events row ------------------------------------
    sqlx::query(
        r#"INSERT INTO audit_events
               (ulid, tenant_id, org_id, actor_type, actor_id,
                action, resource_type, resource_ulid,
                changes, result, hash)
           VALUES
               ($1, $2, $3, 'user', $4,
                $5, $6, $7,
                $8, 'success', $9)"#,
    )
    .bind(&event_ulid)
    .bind(tenant_uuid)
    .bind(org_uuid)
    .bind(&actor_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_ulid)
    .bind(&changes)
    .bind(&hash)
    .execute(&mut *conn)
    .await?;

    // --- 3. Insert outbox_events row -----------------------------------
    // aggregate_id is UUID NOT NULL; use nil UUID for virtual resources.
    let outbox_ulid = new_ulid();
    sqlx::query(
        r#"INSERT INTO outbox_events
               (ulid, aggregate_type, aggregate_id, aggregate_ulid, event_type,
                payload, tenant_id, org_id)
           VALUES
               ($1, 'workforce', $2, $3, $4,
                $5, $6, $7)"#,
    )
    .bind(&outbox_ulid)
    .bind(resource_uuid) // aggregate_id UUID NOT NULL
    .bind(resource_ulid) // aggregate_ulid CHAR(26)
    .bind(action)
    .bind(&changes)
    .bind(tenant_uuid)
    .bind(org_uuid)
    .execute(&mut *conn)
    .await?;

    Ok(())
}
