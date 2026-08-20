//! Database pool construction and connection management.
//!
//! Centralizes PostgreSQL pool creation so every service and the migration
//! runner uses the same sizing and timeout policy. Migrations themselves are
//! owned by the `nexora-migrate` binary, not by service startup, so exactly one
//! process applies schema changes (see `services/migrate`).

use crate::{DatabaseConfig, NexoraError, NexoraResult};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use std::time::Duration;

/// Build a PostgreSQL connection pool from AOS configuration.
///
/// Applies pool sizing and timeout values from `DatabaseConfig`; all values
/// fall back to sensible defaults when unset.
pub async fn connect(config: &DatabaseConfig) -> NexoraResult<PgPool> {
    connect_url(config.url.expose(), config.max_connections, config).await
}

/// Build a pool from a raw connection URL plus explicit sizing/timeouts.
///
/// `url` is used directly (e.g. by tooling and tests that construct a URL from
/// environment variables rather than a parsed `Config`).
pub async fn connect_url(
    url: &str,
    max_connections: u32,
    config: &DatabaseConfig,
) -> NexoraResult<PgPool> {
    let options = parse_options(url)?;

    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(config.max_lifetime_secs))
        .connect_with(options)
        .await
        .map_err(|e| NexoraError::Internal(format!("database connection failed: {e}")))
}

/// Parse a connection string into `PgConnectOptions`, surfacing malformed
/// values as a config error rather than a panic. Public so tooling and tests
/// can validate URLs without opening a connection.
pub fn parse_options(url: &str) -> NexoraResult<PgConnectOptions> {
    PgConnectOptions::from_str(url)
        .map_err(|e| NexoraError::Config(format!("invalid DATABASE_URL: {e}")))
}

/// Cheap round-trip proving the pool is connected and healthy.
pub async fn ping(pool: &PgPool) -> NexoraResult<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| NexoraError::Internal(format!("database ping failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn garbage_url_is_a_config_error_not_a_panic() {
        assert!(matches!(
            parse_options("not a postgres url"),
            Err(NexoraError::Config(_))
        ));
    }

    #[test]
    fn valid_url_parses() {
        let opts = parse_options("postgres://nexora@localhost:5432/nexora").expect("valid url");
        assert_eq!(opts.get_host(), "localhost");
        assert_eq!(opts.get_port(), 5432);
        assert_eq!(opts.get_database(), Some("nexora"));
        assert_eq!(opts.get_username(), "nexora");
    }
}
