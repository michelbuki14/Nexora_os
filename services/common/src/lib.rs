//! AOS Common - Shared library for Africa Operating System services.
//!
//! This crate provides foundational types, configuration, error handling,
//! telemetry, and utilities used across all AOS microservices.

pub mod audit;
pub mod auth_middleware;
pub mod config;
pub mod s3;
pub mod db;
pub mod error;
pub mod health;
pub mod jwt;
pub mod logging;
pub mod money;
pub mod rbac;
pub mod tenant;
pub mod tenant_context;
pub mod time;
pub mod tracing;
pub mod ulid;
pub mod validation;

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
