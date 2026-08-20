//! Payroll audit emission helpers.
//!
//! Mirrors `workforce-service::audit_emit`: every payroll lifecycle action writes
//! two rows on the SAME `&mut PgConnection` (therefore inside the request's RLS
//! transaction, so both roll back together):
//!
//!   1. A hash-chained row in `audit_events` — tamper-evident, replayed by
//!      `verify-hash-chain`. Identical canonical-payload algorithm to
//!      audit-service, so chains stay verifiable across services.
//!   2. An `outbox_events` row with `aggregate_type='payroll'`, consumed by the
//!      outbox dispatcher (bank file export, payslip delivery, notifications).
//!
//! Payroll approvals/locks are financially material: `changes` must carry the
//! decision metadata (totals, actor, notes) but never raw personal identifiers.

use nexora_common::{
    audit::{compute_chain_hash, serialize_canonical, GENESIS_HASH},
    config::Config,
    error::NexoraResult,
    tenant_context::AuthContext,
    ulid::new_ulid,
};
use serde_json::Value;
use sqlx::PgConnection;

/// Emit a payroll audit event + outbox event inside the caller's transaction.
///
/// `resource_type`: `"payroll_run"`, `"payslip"`, `"payroll_configuration"`.
/// `resource_ulid`: external ULID of the affected row.
/// `resource_uuid`: primary key for `outbox_events.aggregate_id` (UUID NOT NULL);
///   pass `uuid::Uuid::nil()` for virtual resources.
/// `action`: `"payroll.run.created"`, `"payroll.run.approved"`, ...
#[allow(clippy::too_many_arguments)]
pub async fn emit_payroll_event(
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

    // --- 1. Hash chain --------------------------------------------------
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

    // --- 2. audit_events ------------------------------------------------
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

    // --- 3. outbox_events -----------------------------------------------
    let outbox_ulid = new_ulid();
    sqlx::query(
        r#"INSERT INTO outbox_events
               (ulid, aggregate_type, aggregate_id, aggregate_ulid, event_type,
                payload, tenant_id, org_id)
           VALUES
               ($1, 'payroll', $2, $3, $4,
                $5, $6, $7)"#,
    )
    .bind(&outbox_ulid)
    .bind(resource_uuid)
    .bind(resource_ulid)
    .bind(action)
    .bind(&changes)
    .bind(tenant_uuid)
    .bind(org_uuid)
    .execute(&mut *conn)
    .await?;

    Ok(())
}
