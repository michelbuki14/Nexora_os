//! OpenTelemetry tracing initialization and utilities.

use crate::{config::TracingConfig, AosError, AosResult};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::Registry;
use tracing_subscriber::util::SubscriberInitExt;

/// Initialize OpenTelemetry tracing.
///
/// Returns the tracer provider so the caller can shut it down gracefully.
/// The OTLP layer is installed only if no global subscriber is set yet; when
/// `init_logging` has already installed a subscriber, callers must compose
/// layers instead (see the service bootstrap in Phase 1 gateway hardening).
pub fn init_tracing(config: &TracingConfig) -> AosResult<Option<sdktrace::SdkTracerProvider>> {
    if !config.enabled {
        return Ok(None);
    }

    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])
        .build();

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.otlp_endpoint)
        .with_timeout(std::time::Duration::from_secs(config.export_timeout_secs))
        .build()
        .map_err(|e| AosError::Internal(format!("Failed to build OTLP span exporter: {e}")))?;

    // Batch export settings use SDK defaults; tune via Config if needed.
    let tracer_provider = sdktrace::SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    let tracer = tracer_provider.tracer(config.service_name.clone());

    // Register the provider so `opentelemetry::global::tracer()` works anywhere.
    global::set_tracer_provider(tracer_provider.clone());

    let otel_layer = OpenTelemetryLayer::new(tracer).with_error_records_to_exceptions(true);
    let subscriber = Registry::default().with(otel_layer);
    if let Err(e) = subscriber.try_init() {
        tracing::warn!("OTLP tracing layer not installed (subscriber already set): {e}");
    }

    Ok(Some(tracer_provider))
}

/// Shutdown tracing provider gracefully.
pub fn shutdown_tracing(provider: Option<sdktrace::SdkTracerProvider>) {
    if let Some(provider) = provider {
        if let Err(e) = provider.shutdown() {
            tracing::error!("Failed to shutdown tracer provider: {}", e);
        }
    }
}

/// Add span attributes from request context.
pub fn enrich_span_with_request(
    span: &tracing::Span,
    method: &str,
    path: &str,
    tenant_id: Option<&str>,
    user_id: Option<&str>,
) {
    span.record("http.method", method);
    span.record("http.path", path);
    if let Some(tid) = tenant_id {
        span.record("tenant.id", tid);
    }
    if let Some(uid) = user_id {
        span.record("user.id", uid);
    }
}

/// Create a span for database operations.
pub fn db_span(operation: &str, table: &str) -> tracing::Span {
    tracing::info_span!(
        "db",
        db.operation = %operation,
        db.table = %table,
    )
}

/// Create a span for external service calls.
pub fn external_span(service: &str, operation: &str) -> tracing::Span {
    tracing::info_span!(
        "external",
        external.service = %service,
        external.operation = %operation,
    )
}
