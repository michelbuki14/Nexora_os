//! Audit hash-chain primitives shared by the audit service and integration tests.
//!
//! Audit events are append-only and linked into a per-tenant SHA-256 hash chain:
//! `hash_i = SHA-256(prev_hash || canonical_payload_i)`. The previous hash for the
//! first event in a tenant's chain is [`GENESIS_HASH`]. Keeping the primitive here
//! (rather than service-local) means the tamper-detection integration tests exercise
//! exactly the same code path the audit service uses in production.

use sha2::{Digest, Sha256};

/// Empty previous-hash used for the very first event in a tenant's chain.
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// SHA-256(prev_hash || payload) — the chain link between two audit events.
pub fn compute_chain_hash(prev_hash: &str, payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_hash_is_64_zeros() {
        assert_eq!(GENESIS_HASH.len(), 64);
        assert!(GENESIS_HASH.chars().all(|c| c == '0'));
    }

    #[test]
    fn chain_hash_is_deterministic() {
        let h1 = compute_chain_hash("abc", "payload");
        let h2 = compute_chain_hash("abc", "payload");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn chain_hash_changes_with_inputs() {
        let a = compute_chain_hash("abc", "payload");
        let b = compute_chain_hash("abd", "payload");
        let c = compute_chain_hash("abc", "payload2");
        assert_ne!(a, b);
        assert_ne!(a, c);
    }
}
