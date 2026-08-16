//! Nexora OS Audit Service — append-only audit log for tenant actions.
//!
//! Domain boundary: Audit events — append-only record of all tenant actions
//! for compliance, forensic analysis, and security auditing.
//!
//! Service startup: config → logging → DB pool → JWT validator → auth
//! middleware → RLS middleware → routes. Handlers extract `DbConn` and
//! `AuthContext` from request extensions (set by the RLS middleware), so the
//! protected router carries no axum state of its own.

mod handlers;
mod models;
mod openapi;
mod routes;

use nexora_common::{
    auth_middleware::AuthState, config::Config, db::connect, health::health_router,
    jwt::JwtValidator, logging::init_logging, tenant_context::RlsState,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("config load failed: {e}");
        Config::load().unwrap()
    });
    init_logging(&config.tracing)?;

    let config = std::sync::Arc::new(config);

    // Database pool (shared across middleware).
    let pool = connect(&config.database).await?;

    // JWT validator for auth middleware.
    let jwt_validator = std::sync::Arc::new(JwtValidator::new(config.auth.clone()));

    // CORS (permissive for dev; tighten for prod via config).
    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    // Health probes at root (no auth, no RLS). `health_router` pins state to
    // `Arc<Config>` internally, returning `Router<()>`.
    let health_routes = health_router(config.clone());

    // RLS state for middleware.
    let rls_state = RlsState {
        pool: pool.clone(),
        config: config.clone(),
    };

    // Auth state for middleware.
    let auth_state = AuthState {
        validator: jwt_validator.clone(),
    };

    // Protected API routes (auth + RLS). Handlers extract `DbConn`/`AuthContext`
    // from request extensions, so this router is `Router<()>` and merges with the
    // health router directly — no `with_state` indirection needed.
    let api_routes = crate::routes::protected_audit_routes(rls_state, auth_state);

    let app = health_routes
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    info!(%addr, "Nexora OS Audit Service started");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
}
