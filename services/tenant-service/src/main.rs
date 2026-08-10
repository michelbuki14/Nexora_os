//! AOS Tenant Service - Multi-tenancy core domain.
//!
//! Domain boundary: Organizations, Tenants.
//!
//! This service owns the tenant data model and enforces row-level security.
//! Handlers extract the request-scoped `DbConn` (an RLS transaction) and
//! `AuthContext` from request extensions — never the raw pool — so every query
//! runs inside the caller's tenant context.

mod handlers;
mod models;

use aos_common::{
    auth_middleware::{auth_middleware, AuthState},
    config::Config,
    health::health_router,
    jwt::JwtValidator,
    logging::init_logging,
    rbac::{require_permission_middleware, RbacState},
    tenant_context::{rls_middleware, RlsState},
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("config load failed: {e}");
        Config::default()
    });
    init_logging(&config.tracing)?;

    let config = Arc::new(config);

    // Database pool
    let pool = aos_common::db::connect(&config.database).await?;

    // JWT validator
    let jwt_validator = Arc::new(JwtValidator::new(config.auth.clone()));

    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    // Health probes at root
    let health_routes = health_router(config.clone());

    // RLS state for middleware
    let rls_state = RlsState {
        pool: pool.clone(),
        config: config.clone(),
    };
    let auth_state = AuthState {
        validator: jwt_validator.clone(),
    };

    // Protected tenant routes. Axum composes `.layer()` calls bottom-to-top: the
    // last layer added is the outermost and runs first. `auth_middleware` must
    // run before `rls_middleware` (it sets the `AuthContext` that RLS reads), so
    // auth is added last. Execution order: auth → RLS → handler.
    let api_routes: Router<()> = Router::new()
        .route(
            "/api/v1/tenants",
            post(handlers::create_tenant).get(handlers::list_tenants),
        )
        .route("/api/v1/tenants/:id", get(handlers::get_tenant))
        .layer(middleware::from_fn_with_state(
            rls_state.clone(),
            rls_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    // Admin routes with additional RBAC. Execution order: auth → RBAC → RLS.
    let admin_routes: Router<()> = Router::new()
        .route("/api/v1/admin/tenants", post(handlers::create_tenant))
        .layer(middleware::from_fn_with_state(
            rls_state.clone(),
            rls_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            RbacState::require("tenant.create"),
            require_permission_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    let app = health_routes
        .merge(api_routes)
        .merge(admin_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    info!(%addr, "AOS Tenant Service started");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
}
