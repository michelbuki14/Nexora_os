//! Health check endpoints and readiness/liveness probes.

use crate::Config;
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub service: String,
    pub version: String,
    pub environment: String,
    pub instance_id: String,
    pub timestamp: String,
    pub checks: Vec<HealthCheck>,
}

/// Overall health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual health check result.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

/// Readiness check response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub checks: Vec<ReadinessCheck>,
}

/// Readiness check detail.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReadinessCheck {
    pub name: String,
    pub ready: bool,
    pub message: Option<String>,
}

/// Health check trait for dependency checks.
#[async_trait::async_trait]
pub trait HealthCheckable: Send + Sync {
    async fn check(&self) -> HealthCheck;
}

/// Database health check.
pub struct DatabaseHealthCheck {
    pool: sqlx::PgPool,
}

impl DatabaseHealthCheck {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl HealthCheckable for DatabaseHealthCheck {
    async fn check(&self) -> HealthCheck {
        let start = std::time::Instant::now();
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => HealthCheck {
                name: "database".to_string(),
                status: HealthStatus::Healthy,
                message: None,
                latency_ms: Some(start.elapsed().as_millis() as u64),
            },
            Err(e) => HealthCheck {
                name: "database".to_string(),
                status: HealthStatus::Unhealthy,
                message: Some(e.to_string()),
                latency_ms: Some(start.elapsed().as_millis() as u64),
            },
        }
    }
}

/// Redis health check.
pub struct RedisHealthCheck {
    client: redis::Client,
}

impl RedisHealthCheck {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl HealthCheckable for RedisHealthCheck {
    async fn check(&self) -> HealthCheck {
        let start = std::time::Instant::now();
        match self.client.get_async_connection().await {
            Ok(mut conn) => match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                Ok(_) => HealthCheck {
                    name: "redis".to_string(),
                    status: HealthStatus::Healthy,
                    message: None,
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                },
                Err(e) => HealthCheck {
                    name: "redis".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: Some(e.to_string()),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                },
            },
            Err(e) => HealthCheck {
                name: "redis".to_string(),
                status: HealthStatus::Unhealthy,
                message: Some(e.to_string()),
                latency_ms: Some(start.elapsed().as_millis() as u64),
            },
        }
    }
}

/// Create health check router.
pub fn health_router(config: Arc<Config>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/health/ready", get(readiness_handler))
        .route("/health/live", get(liveness_handler))
        .with_state(config)
}

/// Health endpoint - overall service health.
async fn health_handler(
    axum::extract::State(config): axum::extract::State<Arc<Config>>,
) -> axum::Json<HealthResponse> {
    let checks = vec![HealthCheck {
        name: "self".to_string(),
        status: HealthStatus::Healthy,
        message: None,
        latency_ms: Some(0),
    }];

    let overall = if checks.iter().all(|c| c.status == HealthStatus::Healthy) {
        HealthStatus::Healthy
    } else if checks.iter().any(|c| c.status == HealthStatus::Unhealthy) {
        HealthStatus::Unhealthy
    } else {
        HealthStatus::Degraded
    };

    axum::Json(HealthResponse {
        status: overall,
        service: config.service.name.clone(),
        version: config.service.version.clone(),
        environment: format!("{:?}", config.service.environment).to_lowercase(),
        instance_id: config.service.instance_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        checks,
    })
}

/// Readiness endpoint - ready to serve traffic.
async fn readiness_handler(
    axum::extract::State(config): axum::extract::State<Arc<Config>>,
) -> axum::Json<ReadinessResponse> {
    let ready = config.service.environment != crate::config::Environment::Development;
    axum::Json(ReadinessResponse {
        ready,
        checks: vec![ReadinessCheck {
            name: "configuration".to_string(),
            ready,
            message: if ready {
                None
            } else {
                Some("development mode".to_string())
            },
        }],
    })
}

/// Liveness endpoint - process is alive.
async fn liveness_handler() -> axum::Json<ReadinessResponse> {
    axum::Json(ReadinessResponse {
        ready: true,
        checks: vec![ReadinessCheck {
            name: "process".to_string(),
            ready: true,
            message: None,
        }],
    })
}
