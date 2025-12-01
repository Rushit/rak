# ZDK Observability Guide

**Created:** 2025-11-19 19:00  
**Status:** Complete

## Overview

ZDK provides comprehensive observability through OpenTelemetry integration and structured logging. This matches the Go ZDK's telemetry approach while leveraging Rust's powerful `tracing` ecosystem.

## Features

### 1. OpenTelemetry Integration

ZDK integrates with OpenTelemetry to provide distributed tracing:

- **LLM Call Tracing**: Automatic tracing of all LLM requests and responses
- **Tool Execution Tracing**: Detailed traces of tool calls with arguments and results
- **Custom Span Processors**: Support for custom span processors (e.g., stdout, Jaeger, cloud exporters)
- **Standardized Attributes**: Uses the same span attributes as Go ZDK for compatibility

### 2. Structured Logging

All logging uses the `tracing` crate for structured, contextual logging:

- **Contextual Fields**: Logs include `invocation_id`, `session_id`, `user_id`, etc.
- **Log Levels**: Proper use of ERROR, WARN, INFO, DEBUG, TRACE levels
- **Environment Control**: Use `RUST_LOG` env var to control verbosity
- **JSON Support**: Can output logs in JSON format for log aggregation systems

### 3. Request/Response Logging

HTTP middleware automatically logs:

- Request method and path
- Response status code
- Request duration
- All structured for easy filtering and analysis

### 4. Health Check Endpoints

Production-ready health monitoring:

- `/health` - Basic liveness check
- `/readiness` - Readiness check for dependencies

## Usage

### Basic Setup

```rust
use rak_telemetry::init_telemetry;

#[tokio::main]
async fn main() {
    // Initialize telemetry with OpenTelemetry support
    init_telemetry();
    
    // Your application code...
}
```

### With Environment Filter

```rust
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // More control over log filtering
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    
    // Your application code...
}
```

### Custom Span Processor

```rust
use rak_telemetry::{init_telemetry, register_span_processor};
use opentelemetry_sdk::trace::SimpleSpanProcessor;
use opentelemetry_sdk::export::trace::stdout;

// Register a custom span processor BEFORE init_telemetry()
register_span_processor(Box::new(|| {
    SimpleSpanProcessor::new(Box::new(stdout::SpanExporter::default()))
}));

init_telemetry();
```

## OpenTelemetry Span Attributes

### LLM Call Spans

ZDK traces LLM calls with these attributes (matching Go ZDK):

```
gen_ai.system = "gcp.vertex.agent"
gen_ai.request.model = "gemini-2.0-flash-exp"
gen_ai.request.top_p = 0.95
gen_ai.request.max_tokens = 1024
gcp.vertex.agent.llm_request = "{...}"
gcp.vertex.agent.llm_response = "{...}"
gcp.vertex.agent.invocation_id = "inv-123"
gcp.vertex.agent.session_id = "sess-456"
gcp.vertex.agent.event_id = "event-789"
```

### Tool Call Spans

Tool executions are traced with:

```
gen_ai.operation.name = "execute_tool"
gen_ai.tool.name = "calculator"
gen_ai.tool.description = "Evaluates mathematical expressions"
gen_ai.tool.call.id = "call-123"
gcp.vertex.agent.tool_call_args = "{\"expression\": \"2+2\"}"
gcp.vertex.agent.tool_response = "{\"result\": 4}"
gcp.vertex.agent.invocation_id = "inv-123"
gcp.vertex.agent.session_id = "sess-456"
gcp.vertex.agent.event_id = "event-789"
```

## Structured Logging Examples

### Agent Lifecycle

```rust
tracing::info!(
    invocation_id = %invocation_id,
    session_id = %session_id,
    agent = %agent_name,
    "Starting LLM agent execution"
);
```

### LLM Calls

```rust
tracing::debug!(
    invocation_id = %invocation_id,
    session_id = %session_id,
    model = %model_name,
    iteration = iteration,
    "Calling LLM"
);
```

### Tool Execution

```rust
tracing::debug!(
    invocation_id = %invocation_id,
    session_id = %session_id,
    tool_name = %tool_name,
    tool_id = %tool_id,
    "Executing tool"
);
```

### Errors

```rust
tracing::error!(
    error = %err,
    invocation_id = %invocation_id,
    session_id = %session_id,
    "LLM call failed"
);
```

## Environment Variables

### Log Level Control

```bash
# Show all logs
RUST_LOG=trace

# Show debug and above
RUST_LOG=debug

# Show info and above (default for production)
RUST_LOG=info

# Module-specific levels
RUST_LOG=rak_agent=debug,rak_runner=info

# Enable telemetry-specific logging
RUST_LOG=rak_telemetry=trace
```

### Running Examples with Logging

```bash
# Quickstart with debug logging
RUST_LOG=debug cargo run --example quickstart

# Telemetry example
RUST_LOG=debug cargo run --example telemetry_usage

# Server with info logging (production-like)
RUST_LOG=info cargo run --bin zdk-server
```

## HTTP Request Logging

The server automatically logs all HTTP requests:

```
2025-11-19T19:00:00.123456Z  INFO tower_http::trace: request{method=POST uri=/api/v1/sessions}
2025-11-19T19:00:00.234567Z  INFO tower_http::trace: response{status=200 latency_ms=111}
```

## Health Check Endpoints

### Liveness Check

```bash
curl http://localhost:8080/health
# Response: OK (200)
```

Used by:
- Kubernetes liveness probes
- Load balancer health checks
- Monitoring systems

### Readiness Check

```bash
curl http://localhost:8080/readiness
# Response: READY (200)
```

Used by:
- Kubernetes readiness probes
- Service mesh health checks
- Deployment orchestration

## Integration with Observability Platforms

### Prometheus Metrics (Future)

While not yet implemented, ZDK is designed to integrate with Prometheus for metrics:

- Request rates and latency
- LLM call counts and durations
- Tool execution metrics
- Error rates

### Jaeger/Zipkin (Future)

OpenTelemetry spans can be exported to distributed tracing platforms:

```rust
// Example configuration (when implemented)
use opentelemetry_jaeger::JaegerPipeline;

let tracer = JaegerPipeline::new()
    .with_service_name("zdk-rust-app")
    .install_simple()?;
```

### Cloud Logging

For cloud deployments, logs can be structured as JSON:

```rust
tracing_subscriber::fmt()
    .json()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

## Comparison with Go ZDK

| Feature | Go ZDK | ZDK | Status |
|---------|--------|----------|--------|
| OpenTelemetry Tracing | ✅ | ✅ | Complete |
| Structured Logging | ✅ | ✅ | Complete |
| LLM Call Tracing | ✅ | ✅ | Complete |
| Tool Call Tracing | ✅ | ✅ | Complete |
| Custom Span Processors | ✅ | ✅ | Complete |
| HTTP Request Logging | ✅ | ✅ | Complete |
| Health Check Endpoints | ✅ | ✅ | Complete |
| Prometheus Metrics | ❌ | ❌ | Not in scope |
| Distributed Tracing Export | Partial | Partial | Planned |

## Best Practices

### 1. Always Include Context

```rust
// Good: Includes invocation and session IDs
tracing::debug!(
    invocation_id = %inv_id,
    session_id = %sess_id,
    "Processing request"
);

// Bad: No context
tracing::debug!("Processing request");
```

### 2. Use Appropriate Log Levels

- **ERROR**: Something failed that requires attention
- **WARN**: Something unexpected but handled
- **INFO**: Important business events (agent started, completed)
- **DEBUG**: Detailed diagnostic information
- **TRACE**: Very detailed (function entry/exit)

### 3. Structured Fields, Not String Interpolation

```rust
// Good: Structured fields
tracing::info!(user_id = %user_id, "User logged in");

// Bad: String interpolation
tracing::info!("User {} logged in", user_id);
```

### 4. Don't Log Sensitive Data

Never log:
- API keys or tokens
- User passwords
- Full user content (may contain PII)
- Internal system secrets

### 5. Use println! Only for User-Facing Output

In examples and CLIs:
```rust
// Examples: println! is fine for user-facing output
println!("Starting application...");

// Libraries: Use tracing for all logging
tracing::info!("Service initialized");
```

## Troubleshooting

### No Logs Appearing

Check `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run
```

### Too Many Logs

Filter to specific modules:
```bash
RUST_LOG=rak_agent=info,rak_runner=warn cargo run
```

### OpenTelemetry Spans Not Exporting

Ensure you've:
1. Registered span processors before calling `init_telemetry()`
2. Initialized telemetry at application startup
3. Configured the exporter properly

## Examples

See these examples for practical usage:

- `examples/quickstart.rs` - Basic tracing setup
- `examples/telemetry_usage.rs` - Full OpenTelemetry integration
- `examples/tool_usage.rs` - Tool execution tracing

## Resources

- [Tracing Documentation](https://docs.rs/tracing/)
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Go ZDK Telemetry](https://github.com/google/zdk-go/blob/main/internal/telemetry/telemetry.go)

## Summary

ZDK provides production-ready observability matching the Go ZDK implementation:

✅ OpenTelemetry tracing for LLM and tool calls  
✅ Structured logging with contextual fields  
✅ HTTP request/response logging middleware  
✅ Health check endpoints for monitoring  
✅ Environment-based log level control  
✅ Custom span processor support  

All logs and traces include proper context (invocation_id, session_id, user_id) for distributed system debugging.

