//! Nexora OS Payroll Service - DRC-compliant payroll engine.
//!
//! Domain: IPR progressive tax, CNSS contributions, SMIG compliance,
//! payroll run lifecycle (draft -> review -> approved -> locked),
//! payslip generation + PDF rendering.
//!
//! Handlers extract the request-scoped `DbConn` (an RLS transaction) and
//! `AuthContext` from request extensions — never the raw pool.

use nexora_payroll_service::{routes, AppState};
use nexora_common::{
    auth_middleware::{auth_middleware, AuthState},
    config::Config,
    db,
    health::health_router,
    jwt::JwtValidator,
    logging::init_logging,
    tenant_context::{rls_middleware, RlsState},
};
use std::{net::SocketAddr, sync::Arc};
use axum::middleware;
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

    let config = Arc::new(config);

    // Database pool
    let pool = db::connect(&config.database).await?;

    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers(Any)
        .allow_origin(Any);

    // Health probes at root
    let health_routes = health_router(config.clone());

    // RLS + auth state for middleware
    let rls_state = RlsState {
        pool: pool.clone(),
        config: config.clone(),
    };
    let auth_state = AuthState {
        validator: Arc::new(JwtValidator::new(config.auth.clone())),
    };

    let app_state = AppState {
        config: config.clone(),
        s3_cfg: config.s3.clone(),
    };

    // Payroll API routes (auth -> RLS -> handler).
    let api_routes = routes::router()
        .layer(middleware::from_fn_with_state(rls_state.clone(), rls_middleware))
        .layer(middleware::from_fn_with_state(auth_state.clone(), auth_middleware))
        .with_state(app_state);

    let app = health_routes
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    info!(%addr, "Nexora OS Payroll Service started");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
}
