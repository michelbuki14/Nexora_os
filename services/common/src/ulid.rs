//! ULID utilities for sortable, non-sequential identifiers.

pub use ulid::Ulid;

/// Generate a new ULID string.
pub fn new_ulid() -> String {
    Ulid::new().to_string()
}

/// Parse ULID from string.
pub fn parse_ulid(value: &str) -> Result<Ulid, ulid::DecodeError> {
    value.parse()
}

/// Generate a deterministic ULID from a UUID-like byte array.
pub fn ulid_from_bytes(bytes: [u8; 16]) -> Ulid {
    Ulid::from_bytes(bytes)
}
