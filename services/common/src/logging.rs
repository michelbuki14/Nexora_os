//! Structured logging configuration.

use crate::config::TracingConfig;
use std::path::Path;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Initialize structured JSON logging.
pub fn init_logging(config: &TracingConfig) -> crate::AosResult<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,nexora=debug,tower_http=debug"));

    let fmt_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_filter(env_filter);

    if config.enabled && !config.otlp_endpoint.is_empty() {
        // OTLP will be configured separately via tracing module
        tracing_subscriber::registry().with(fmt_layer).init();
    } else {
        tracing_subscriber::registry().with(fmt_layer).init();
    }

    Ok(())
}

/// Initialize file-based logging for production.
pub fn init_file_logging(log_dir: &Path, service_name: &str) -> crate::AosResult<()> {
    let file_appender = rolling::daily(log_dir, format!("{service_name}.log"));
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(non_blocking);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(file_layer.with_filter(env_filter))
        .init();

    Ok(())
}

/// Create a structured span for request tracking.
pub fn request_span(
    request_id: &str,
    method: &str,
    path: &str,
    tenant_id: Option<&str>,
) -> tracing::Span {
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        http.method = %method,
        http.path = %path,
    );

    if let Some(tenant) = tenant_id {
        span.record("tenant_id", tenant);
    }

    span
}
