//! Nexora OS API Gateway - modular monolith entry point.
//!
//! The gateway owns cross-cutting concerns (request IDs, tracing, CORS,
//! timeouts, health probes) and exposes versioned domain routes. Routes for
//! live services (workforce) are reverse-proxied to the upstream service;
//! not-yet-built verticals render an honest "planned" boundary.

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{HeaderName, Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Json, Router,
};
use nexora_common::{config::Config, health::health_router, logging::init_logging};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    http: reqwest::Client,
    body_limit: usize,
}

#[derive(Debug, Serialize)]
struct ApiMetadata {
    name: &'static str,
    version: String,
    status: &'static str,
    documentation: &'static str,
}

/// Hop-by-hop headers that must not be forwarded by a proxy (RFC 7230 §6.1).
/// reqwest sets Host/Content-Length itself from the URL + body.
fn is_hop_by_hop(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "host"
            | "connection"
            | "content-length"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

/// Reverse-proxy core. Forwards `req` to `base_url + upstream_path + query`,
/// stripping the gateway prefix (`strip`). Authorization header is forwarded so
/// the upstream's own auth+RLS stack still enforces.
async fn proxy_to(
    http: &reqwest::Client,
    base_url: &str,
    strip: &str,
    body_limit: usize,
    req: Request<Body>,
) -> Response {
    let path = req.uri().path();
    let upstream_path = if strip.is_empty() {
        path.to_string()
    } else {
        path.trim_start_matches(strip).to_string()
    };
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    let url = format!("{base_url}{upstream_path}{query}");

    // Copy request headers, dropping hop-by-hop ones. Body is buffered; for
    // document uploads (JSON\nbytes) this is fine within the body limit.
    let connection_headers = req
        .headers()
        .get("connection")
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .map(|name| name.trim().to_ascii_lowercase())
                .collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();
    let mut upstream_req = http.request(req.method().clone(), &url);
    for (name, value) in req.headers() {
        if !is_hop_by_hop(name) && !connection_headers.contains(name.as_str()) {
            upstream_req = upstream_req.header(name, value);
        }
    }

    let body = match to_bytes(req.into_body(), body_limit).await {
        Ok(b) => b,
        Err(e) => {
            warn!(%e, limit = body_limit, "gateway request body exceeds configured limit");
            return Response::builder()
                .status(StatusCode::PAYLOAD_TOO_LARGE)
                .body(Body::from("request body exceeds configured limit"))
                .expect("response");
        }
    };
    upstream_req = upstream_req.body(body);

    match upstream_req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            // Cap upstream response bodies: refuse to buffer more than the
            // configured request body limit (ponytail: symmetric cap keeps
            // gateway memory bounded; raise server.body_limit_bytes if a
            // legitimate upstream response legitimately exceeds it).
            if let Some(content_length) = resp.content_length() {
                if content_length > body_limit as u64 {
                    warn!(
                        content_length,
                        limit = body_limit,
                        "upstream response exceeds limit"
                    );
                    return Response::builder()
                        .status(StatusCode::PAYLOAD_TOO_LARGE)
                        .body(Body::from("upstream response exceeds configured limit"))
                        .expect("response");
                }
            }
            let bytes = match resp.bytes().await {
                Ok(b) => {
                    if b.len() > body_limit {
                        warn!(
                            len = b.len(),
                            limit = body_limit,
                            "upstream response exceeds limit"
                        );
                        return Response::builder()
                            .status(StatusCode::PAYLOAD_TOO_LARGE)
                            .body(Body::from("upstream response exceeds configured limit"))
                            .expect("response");
                    }
                    b
                }
                Err(e) => {
                    warn!(%e, "failed to read upstream response body");
                    return Response::builder()
                        .status(StatusCode::BAD_GATEWAY)
                        .body(Body::from("upstream read error"))
                        .expect("response");
                }
            };
            let connection_headers = headers
                .get("connection")
                .and_then(|value| value.to_str().ok())
                .map(|value| {
                    value
                        .split(',')
                        .map(|name| name.trim().to_ascii_lowercase())
                        .collect::<std::collections::HashSet<_>>()
                })
                .unwrap_or_default();

            let mut builder = Response::builder().status(status);
            for (name, value) in &headers {
                if !is_hop_by_hop(name) && !connection_headers.contains(name.as_str()) {
                    builder = builder.header(name, value);
                }
            }
            builder
                .body(Body::from(bytes))
                .expect("valid upstream response")
        }
        Err(e) => {
            warn!(%e, %url, "upstream request failed");
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("upstream error: {e}")))
                .expect("response")
        }
    }
}

/// `/api/v1/workforce/*` -> workforce service (bare paths: /employees, ...).
async fn workforce_proxy(State(state): State<AppState>, req: Request<Body>) -> Response {
    proxy_to(
        &state.http,
        &state.config.gateway.workforce_base_url,
        "/api/v1/workforce",
        state.body_limit,
        req,
    )
    .await
}

/// `/api/v1/audit/*` -> audit service (bare paths: /events, ...).
async fn audit_proxy(State(state): State<AppState>, req: Request<Body>) -> Response {
    proxy_to(
        &state.http,
        &state.config.gateway.audit_base_url,
        "/api/v1/audit",
        state.body_limit,
        req,
    )
    .await
}

/// `/api/v1/tenants*` -> tenant service. It already serves `/api/v1/tenants*`,
/// so paths are forwarded unchanged (no prefix strip).
async fn tenant_proxy(State(state): State<AppState>, req: Request<Body>) -> Response {
    proxy_to(
        &state.http,
        &state.config.gateway.tenant_base_url,
        "",
        state.body_limit,
        req,
    )
    .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Fail closed: a misconfigured gateway must not start silently against
    // localhost defaults — that would proxy to nowhere in production.
    let config = Config::load().map_err(|error| {
        eprintln!("configuration load failed: {error}");
        anyhow::anyhow!("configuration load failed: {error}")
    })?;
    init_logging(&config.tracing)?;
    let request_timeout = Duration::from_secs(config.server.request_timeout_secs);
    let body_limit = config.server.body_limit_bytes;

    let config = Arc::new(config);
    let state = AppState {
        config: config.clone(),
        http: reqwest::Client::builder()
            .timeout(request_timeout)
            .build()
            .expect("reqwest client"),
        body_limit,
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
        .route("/api/v1", any(api_metadata))
        .nest("/api/v1/identity", placeholder_router("identity"))
        .nest("/api/v1/organizations", placeholder_router("organizations"))
        // Live services are reverse-proxied to their upstream, which owns its
        // own auth+RLS stack (Authorization is forwarded). Not-yet-built
        // verticals render an honest "planned" boundary instead.
        // Use Router fallback to catch all sub-paths within each prefix.
        .nest("/api/v1/audit", Router::new().fallback(audit_proxy))
        .nest("/api/v1/workforce", Router::new().fallback(workforce_proxy))
        .nest("/api/v1/tenants", Router::new().fallback(tenant_proxy))
        // Admin tenant management endpoint (tenant-service serves at /api/v1/admin/tenants)
        .nest(
            "/api/v1/admin/tenants",
            Router::new().fallback(tenant_proxy),
        )
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
            request_timeout,
        ))
        .layer(cors);

    let address: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!(%address, service = %config.service.name, "Nexora OS API Gateway started");

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
        version: state.config.service.version.clone(),
        status: "operational",
        documentation: "/docs",
    })
}

fn placeholder_router(domain: &'static str) -> Router<AppState> {
    Router::new().route("/status", any(move || async move {
        Json(serde_json::json!({"domain": domain, "status": "planned", "message": "Domain boundary reserved; implementation gated behind production readiness review"}))
    }))
}
