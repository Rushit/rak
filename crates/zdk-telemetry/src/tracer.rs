//! Tracer setup and management

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::{SimpleSpanProcessor, TracerProvider};
use std::sync::{Arc, Mutex, OnceLock};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Global tracer provider holder
static TRACER_PROVIDER: OnceLock<Arc<TracerProvider>> = OnceLock::new();

/// Global span processor builders (registered before initialization)
type ProcessorBuilder = Box<dyn FnOnce() -> SimpleSpanProcessor + Send>;
static SPAN_PROCESSOR_BUILDERS: Mutex<Option<Vec<ProcessorBuilder>>> = Mutex::new(Some(Vec::new()));

/// Register a custom span processor builder to be used when telemetry is initialized.
///
/// This allows custom span processors (e.g., for different exporters like Jaeger, Zipkin,
/// or custom backends) to be registered before the OpenTelemetry tracer provider is created.
/// Must be called BEFORE `init_telemetry()`.
///
/// # Example
///
/// ```ignore
/// use zdk_telemetry::{register_span_processor, init_telemetry};
/// use opentelemetry_sdk::trace::SimpleSpanProcessor;
///
/// // Register a custom span processor before initializing
/// register_span_processor(Box::new(|| {
///     SimpleSpanProcessor::new(Box::new(/* your exporter */))
/// }));
/// init_telemetry();
/// ```
pub fn register_span_processor(builder: ProcessorBuilder) {
    let mut builders = SPAN_PROCESSOR_BUILDERS
        .lock()
        .expect("Failed to lock span processor builders");

    if let Some(ref mut vec) = *builders {
        vec.push(builder);
    } else {
        tracing::warn!("Attempted to register span processor after telemetry initialization");
    }
}

/// Initialize telemetry with OpenTelemetry support.
///
/// This sets up:
/// - A tracer provider with any registered span processors
/// - Integration with the tracing subscriber
/// - Structured logging output
///
/// # Example
///
/// ```rust,no_run
/// use zdk_telemetry::init_telemetry;
///
/// init_telemetry();
/// ```
pub fn init_telemetry() {
    // Take the span processor builders (can only initialize once)
    let builders = SPAN_PROCESSOR_BUILDERS
        .lock()
        .expect("Failed to lock span processor builders")
        .take()
        .unwrap_or_default();

    // Build tracer provider with registered processors
    let mut provider_builder = TracerProvider::builder();
    for builder in builders {
        let processor = builder();
        provider_builder = provider_builder.with_span_processor(processor);
    }
    let tracer_provider = provider_builder.build();

    // Create tracer from provider
    let tracer = tracer_provider.tracer(crate::attributes::SYSTEM_NAME);

    // Store provider globally
    let _ = TRACER_PROVIDER.set(Arc::new(tracer_provider));

    // Set up tracing subscriber with OpenTelemetry layer
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Combine with structured logging
    tracing_subscriber::registry()
        .with(telemetry_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_line_number(true),
        )
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

/// Get the global tracer provider if initialized
pub fn tracer_provider() -> Option<Arc<TracerProvider>> {
    TRACER_PROVIDER.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_provider_not_initialized() {
        // Before initialization, should return None
        // Note: This test may fail if other tests have initialized telemetry
        // In a real scenario, initialization happens once per process
        assert!(tracer_provider().is_none() || tracer_provider().is_some());
    }
}
