//! Audit hash-chain primitives shared by the audit service and integration tests.
//!
//! Audit events are append-only and linked into a per-tenant HMAC-SHA-256 hash chain:
//! `hash_i = HMAC-SHA-256(signing_key, prev_hash || canonical_payload_i)`.
//! The previous hash for the very first event in a tenant's chain is [`GENESIS_HASH`].
//! Keeping the primitive here (rather than service-local) means the tamper-detection
//! integration tests exercise exactly the same code path the audit service uses in production.

use crate::config::Config;
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Empty previous-hash used for the very first event in a tenant's chain.
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// HMAC-SHA-256 keyed hash chain: HMAC(key, prev_hash || payload).
///
/// Uses a per-tenant signing key derived from the tenant's secret to prevent
/// cross-tenant hash collisions and enable tamper detection.
pub fn compute_chain_hash(config: &Config, tenant_id: &uuid::Uuid, prev_hash: &str, payload: &str) -> String {
    // Derive a per-tenant signing key from the config's audit signing secret
    // In production, this would come from a vault/HSM. For now, we derive from
    // a config secret combined with tenant ID.
    let signing_key = derive_signing_key(config, tenant_id);

    let mut mac = Hmac::<Sha256>::new_from_slice(&signing_key)
        .expect("HMAC key must be valid length");
    mac.update(prev_hash.as_bytes());
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Derive a per-tenant signing key from the global audit signing secret.
///
/// In production, this should use a proper KDF (HKDF) with a vault-managed
/// master key. This implementation uses a simple HMAC-based derivation.
pub fn derive_signing_key(config: &Config, tenant_id: &uuid::Uuid) -> Vec<u8> {
    // Use the audit signing secret from config, or fall back to a dev-only default
    let master_secret = config.audit.signing_secret.expose().as_bytes();
    let tenant_bytes = tenant_id.as_bytes();

    let mut mac = Hmac::<Sha256>::new_from_slice(master_secret)
        .expect("master secret must be valid length for HMAC");
    mac.update(b"tenant-signing-key-derivation");
    mac.update(tenant_bytes);
    mac.finalize().into_bytes().to_vec()
}

/// Verify the hash chain integrity for a tenant up to a given event.
/// Returns the first mismatched event ID if the chain is broken.
pub fn verify_chain(
    config: &Config,
    tenant_id: &uuid::Uuid,
    events: &[AuditEventForVerification],
) -> Result<Option<i64>, crate::NexoraError> {
    let mut prev_hash = GENESIS_HASH.to_string();

    for event in events {
        let computed = compute_chain_hash(config, tenant_id, &prev_hash, &event.canonical_payload);
        if computed != event.hash {
            return Ok(Some(event.id));
        }
        prev_hash = event.hash.clone();
    }

    Ok(None)
}

/// Minimal event structure for chain verification.
pub struct AuditEventForVerification {
    pub id: i64,
    pub hash: String,
    pub canonical_payload: String,
}

/// Serialize a JSON value with keys sorted (deterministic / canonical).
///
/// This matches the `serialize_canonical` in workforce-service's audit_emit.rs.
/// Keeping both in sync is important: if the format diverges the hash chain breaks at verify time.
pub fn serialize_canonical(value: &serde_json::Value) -> Result<String, crate::NexoraError> {
    let sorted = sort_keys(value);
    serde_json::to_string(&sorted)
        .map_err(|e| crate::NexoraError::Internal(format!("canonical serialization failed: {e}")))
}

fn sort_keys(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            let mut keys: Vec<&str> = map.keys().map(|s| s.as_str()).collect();
            keys.sort_unstable();
            for k in keys {
                sorted.insert(k.to_string(), sort_keys(&map[k]));
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => serde_json::Value::Array(arr.iter().map(sort_keys).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use uuid::Uuid;

    #[test]
    fn genesis_hash_is_64_zeros() {
        assert_eq!(GENESIS_HASH.len(), 64);
        assert!(GENESIS_HASH.chars().all(|c| c == '0'));
    }

    #[test]
    fn chain_hash_is_deterministic() {
        let config = Config::default();
        let tenant_id = Uuid::new_v4();
        let h1 = compute_chain_hash(&config, &tenant_id, "abc", "payload");
        let h2 = compute_chain_hash(&config, &tenant_id, "abc", "payload");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn chain_hash_changes_with_inputs() {
        let config = Config::default();
        let tenant_id = Uuid::new_v4();
        let a = compute_chain_hash(&config, &tenant_id, "abc", "payload");
        let b = compute_chain_hash(&config, &tenant_id, "abd", "payload");
        let c = compute_chain_hash(&config, &tenant_id, "abc", "payload2");
        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn chain_hash_differs_per_tenant() {
        let config = Config::default();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let h_a = compute_chain_hash(&config, &tenant_a, "abc", "payload");
        let h_b = compute_chain_hash(&config, &tenant_b, "abc", "payload");
        // Same inputs, different tenants -> different hashes (keyed by tenant)
        assert_ne!(h_a, h_b);
    }
}