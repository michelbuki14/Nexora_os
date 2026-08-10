//! AOS API Gateway - modular monolith entry point.
//!
//! The gateway owns cross-cutting concerns (request IDs, tracing, CORS,
//! timeouts, health probes) and exposes versioned domain routes.

use aos_common::{config::Config, health::health_router, logging::init_logging};
use axum::{extract::State, http::Method, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

#[derive(Debug, Serialize)]
struct ApiMetadata {
    name: &'static str,
    version: &'static str,
    status: &'static str,
    documentation: &'static str,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().unwrap_or_else(|error| {
        eprintln!("configuration load failed, using safe defaults: {error}");
        Config::default()
    });
    init_logging(&config.tracing)?;

    let config = Arc::new(config);
    let state = AppState {
        config: config.clone(),
    };
    let request_id = MakeRequestUuid;

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers(Any)
        .allow_origin(Any);

    // Health probes are merged at root (health_router defines `/health`,
    // `/health/ready`, `/health/live`) so the probes are reachable exactly as
    // documented — never double-prefixed. Each router keeps its own baked
    // state (health: Arc<Config>; API: AppState), so both are collapsed to
    // `Router<()>` before merging.
    let health_routes = health_router(config.clone());
    let api_routes: Router<AppState> = Router::new()
        .route("/", get(api_metadata))
        .route("/api/v1", get(api_metadata))
        .nest("/api/v1/identity", placeholder_router("identity"))
        .nest("/api/v1/organizations", placeholder_router("organizations"))
        .nest("/api/v1/audit", placeholder_router("audit"))
        // Workforce is a real service; its router owns its own auth+RLS stack.
        // The gateway forwards /api/v1/workforce/* to the workforce service
        // (currently co-located; extract to a separate process when load demands).
        .nest("/api/v1/workforce", placeholder_router("workforce"))
        .nest("/api/v1/commerce", placeholder_router("commerce"))
        .nest("/api/v1/finance", placeholder_router("finance"))
        .nest("/api/v1/government", placeholder_router("government"));

    let app = health_routes
        .merge(api_routes.with_state(state))
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
            Duration::from_secs(30),
        ))
        .layer(cors);

    let address: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!(%address, service = %config.service.name, "AOS API Gateway started");

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
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}

async fn api_metadata(State(state): State<AppState>) -> impl IntoResponse {
    Json(ApiMetadata {
        name: "Africa Operating System",
        version: Box::leak(state.config.service.version.clone().into_boxed_str()),
        status: "operational",
        documentation: "/docs",
    })
}

fn placeholder_router(domain: &'static str) -> Router<AppState> {
    Router::new().route("/status", get(move || async move {
        Json(serde_json::json!({"domain": domain, "status": "planned", "message": "Domain boundary reserved; implementation gated behind production readiness review"}))
    }))
}
