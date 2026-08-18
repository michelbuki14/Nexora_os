//! Library surface of the workforce service, exported for integration tests.

pub mod audit_emit;
pub mod documents;
pub mod models;

// Internal modules used only by the binary.
#[doc(hidden)]
pub mod handlers;
#[doc(hidden)]
pub mod openapi;
#[doc(hidden)]
pub mod routes;

use aws_sdk_s3::Client as S3Client;
use nexora_common::config::{Config, S3Config};
use std::sync::Arc;

/// Shared application state — re-exported for tests that instantiate handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub s3_client: Arc<S3Client>,
    pub s3_cfg: S3Config,
}

/// Extension type for passing Config to audit emission
pub type ConfigExt = std::sync::Arc<Config>;

impl AppState {
    pub fn placeholder() -> Self {
        let cfg = Config::load().unwrap();
        let s3_cfg = cfg.s3.clone();
        let client = nexora_common::s3::build_s3_client(&s3_cfg);
        Self {
            config: Arc::new(cfg),
            s3_client: Arc::new(client),
            s3_cfg,
        }
    }
}
