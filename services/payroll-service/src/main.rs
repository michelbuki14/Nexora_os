use nexora_payroll_service::{PayrollState, run_migrations};
use nexora_common::Config;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,nexora_payroll_service=debug".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(Config::load()?);
    let state = PayrollState::new(config.clone()).await?;

    // Run migrations
    run_migrations(&state.pool).await?;

    // Build router
    let app = nexora_payroll_service::routes::router(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port)).await?;
    tracing::info!("Payroll service listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}