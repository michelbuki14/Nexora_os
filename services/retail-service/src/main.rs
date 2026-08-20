//! nexora-retail-service — serverless function stub.

use axum::{
    routing::get,
    Router,
};
use utoipa::OpenApi;

/// Minimal OpenAPI doc for the nexora-retail-service.
#[derive(OpenApi)]
#[openapi(
    paths(),
    info(
        title = "nexora-retail-service",
        version = "0.1.0",
    ),
)]
pub struct RetailServiceApiDoc;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is alive"),
    ),
    tag = "health",
)]
async fn health() -> &'static str {
    "OK"
}

pub fn router() -> Router {
    Router::new().route("/health", get(health))
}

#[tokio::main]
async fn main() {
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "0.0.0.0:3005".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, router()).await.unwrap();
}
