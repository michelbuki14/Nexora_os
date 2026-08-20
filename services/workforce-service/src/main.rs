//! Nexora OS Workforce Service — Phase 1.
//!
//! Provides org structure, employee identity/employment/compensation,
//! secure document management, and lifecycle audit events.
//!
//! Port default: 3002  (gateway 3000, audit 3001).

use std::{net::SocketAddr, sync::Arc};

use nexora_common::{
    auth_middleware::AuthState, config::Config, db::connect, health::health_router,
    jwt::JwtValidator, logging::init_logging, s3::build_s3_client, tenant_context::RlsState,
};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;

use nexora_workforce_service::{routes::workforce_router, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("config load failed, using defaults: {e}");
        Config::load().unwrap()
    });
    init_logging(&config.tracing)?;

    let config = Arc::new(config);
    let pool = connect(&config.database).await?;
    let jwt = Arc::new(JwtValidator::new(config.auth.clone()));
    let s3_client = Arc::new(build_s3_client(&config.s3));

    let rls_state = RlsState {
        pool: pool.clone(),
        config: config.clone(),
    };
    let auth_state = AuthState {
        validator: jwt.clone(),
    };

    let state = AppState {
        config: config.clone(),
        s3_client: s3_client.clone(),
        s3_cfg: config.s3.clone(),
    };

    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        .allow_headers(Any)
        .allow_origin(Any);

    let request_id = MakeRequestUuid;
    let health = health_router(config.clone());
    let wf_routes: axum::Router<AppState> =
        workforce_router(rls_state, auth_state, config.as_ref().clone());

    let app = health
        .merge(wf_routes.with_state(state))
        .layer(SetRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
            request_id,
        ))
        .layer(PropagateRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(config.server.request_timeout_secs),
        ))
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "Nexora OS Workforce Service started");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
