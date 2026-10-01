//! Nexora OS Retail Service - Consumer commerce (products, orders, inventory).
//!
//! Domain boundary: retail catalog, orders, and inventory. Owns its data model
//! and enforces row-level security via the request-scoped `DbConn`.

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

    let pool = nexora_common::db::connect(&config.database).await?;
    let jwt_validator = Arc::new(JwtValidator::new(config.auth.clone()));

    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let health_routes = health_router(config.clone());

    let rls_state = RlsState {
        pool: pool.clone(),
        config: config.clone(),
    };
    let auth_state = AuthState {
        validator: jwt_validator.clone(),
    };

    // Execution order: auth -> RLS -> handler.
    let api_routes: Router<()> = Router::new()
        .route(
            "/products",
            post(handlers::create_product).get(handlers::list_products),
        )
        .route("/products/:id", get(handlers::get_product))
        .route(
            "/orders",
            post(handlers::create_order).get(handlers::list_orders),
        )
        .route("/orders/:id", get(handlers::get_order))
        .route("/inventory", get(handlers::list_inventory))
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
        .route("/admin/products", post(handlers::create_product))
        .layer(middleware::from_fn_with_state(
            rls_state.clone(),
            rls_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            RbacState::require("retail.admin"),
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
    info!(%addr, "Nexora OS Retail Service started");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
}
