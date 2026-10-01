//! Nexora OS Fintech Service - Consumer payments, wallets, and transactions.
//!
//! Domain boundary: consumer-facing financial products (wallets, payments,
//! transactions, card artifacts). This service owns the fintech data model and
//! enforces row-level security via the request-scoped `DbConn` (an RLS
//! transaction) and `AuthContext` extracted from request extensions.

mod handlers;
mod models;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use nexora_common::{
    auth_middleware::{auth_middleware, AuthState},
    config::Config,
    health::health_router,
    jwt::JwtValidator,
    logging::init_logging,
    rbac::{require_permission_middleware, RbacState},
    tenant_context::{rls_middleware, RlsState},
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
        Config::load().unwrap()
    });
    init_logging(&config.tracing)?;

    let config = Arc::new(config);

    // Database pool
    let pool = nexora_common::db::connect(&config.database).await?;

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

    // Protected fintech routes. Execution order: auth -> RLS -> handler.
    let api_routes: Router<()> = Router::new()
        .route(
            "/wallets",
            post(handlers::create_wallet).get(handlers::list_wallets),
        )
        .route("/wallets/:id", get(handlers::get_wallet))
        .route(
            "/transactions",
            post(handlers::create_transaction).get(handlers::list_transactions),
        )
        .route("/transactions/:id", get(handlers::get_transaction))
        .route("/payments", get(handlers::list_payments))
        .layer(middleware::from_fn_with_state(
            rls_state.clone(),
            rls_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    // Admin routes with additional RBAC.
    let admin_routes: Router<()> = Router::new()
        .route("/admin/wallets", post(handlers::create_wallet))
        .layer(middleware::from_fn_with_state(
            rls_state.clone(),
            rls_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            RbacState::require("fintech.admin"),
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
    info!(%addr, "Nexora OS Fintech Service started");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
}
