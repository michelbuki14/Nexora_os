//! Nexora OS Common - Shared library for Nexora OS services.
//!
//! This crate provides foundational types, configuration, error handling,
//! telemetry, and utilities used across all AOS microservices.

pub mod audit;
pub mod auth_middleware;
pub mod config;
pub mod db;
pub mod error;
pub mod health;
pub mod jwt;
pub mod logging;
pub mod money;
pub use rust_decimal::Decimal;
pub mod rbac;
pub mod s3;
pub mod tenant;
pub mod tenant_context;
pub mod time;
pub mod tracing;
pub mod ulid;
pub mod validation;
pub mod idempotency;

pub use auth_middleware::{auth_middleware, AuthState, OptionalAuthContext};
pub use config::*;
pub use db::*;
pub use error::*;
pub use health::*;
pub use jwt::*;
pub use logging::*;
pub use money::*;
pub use rbac::{require_permission, require_permission_middleware, AuthContextExt, RbacState};
pub use tenant::*;
pub use tenant_context::{add_rls_middleware, AuthContext, DbConn, DbConnGuard, RlsState};
pub use time::*;
pub use ulid::*;
pub use validation::*;
pub use idempotency::{IdempotencyState, add_idempotency_middleware, IDEMPOTENCY_KEY_HEADER};
