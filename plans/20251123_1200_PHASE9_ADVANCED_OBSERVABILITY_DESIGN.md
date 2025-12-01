# Phase 9: Advanced Observability & Distributed Tracing

**Date**: 2025-11-23 12:00  
**Status**: 🎯 Design Phase  
**Priority**: High

---

## 🎯 Objectives

Build a comprehensive observability system that tracks **every step** of agent execution using OpenTelemetry, enabling:
- Complete distributed tracing of agent workflows
- Performance analysis and bottleneck identification  
- Debugging production issues with full context
- Integration with external observability platforms (Jaeger, Tempo, GCP Cloud Trace, etc.)

**Key Principle**: This repo is the **client** - it generates and exports telemetry data to external systems for storage and visualization.

---

## 📊 Current State Analysis

### What Exists (Phase 7 - Basic Observability)

**Crate**: `zdk-telemetry`  
**Capabilities**:
- ✅ LLM call tracing (`trace_llm_call`)
- ✅ Tool execution tracing (`trace_tool_call`)
- ✅ Basic span attributes (invocation_id, session_id, event_id)
- ✅ OpenTelemetry integration with custom span processors
- ✅ Structured logging with `tracing` crate

**Limitations**:
- ❌ No agent lifecycle tracing (start/end/error)
- ❌ No runner execution tracing
- ❌ No session operation tracing
- ❌ No parent-child span relationships
- ❌ No async span context propagation
- ❌ No OTLP exporter configuration
- ❌ No metrics collection
- ❌ No trace sampling configuration
- ❌ Limited error tracking

---

## 🎨 Design Principles

1. **Zero Performance Impact**: Observability should not degrade agent performance
2. **Opt-in by Configuration**: Users control what to trace and where to export
3. **Standards-Based**: Use OpenTelemetry standards for portability
4. **Async-First**: Properly handle Rust async context propagation
5. **Production-Ready**: Include sampling, batching, and error handling

---

## 🏗️ Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     ZDK Application                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  Agent   │  │  Runner  │  │ Session  │  │   LLM    │   │
│  │ Tracing  │  │ Tracing  │  │ Tracing  │  │ Tracing  │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │             │              │             │          │
│       └─────────────┴──────────────┴─────────────┘          │
│                          │                                   │
│              ┌───────────▼───────────┐                      │
│              │  Telemetry Collector  │                      │
│              │  (zdk-telemetry)      │                      │
│              └───────────┬───────────┘                      │
│                          │                                   │
└──────────────────────────┼───────────────────────────────────┘
                           │
              ┌────────────▼────────────┐
              │   OTLP Exporter         │
              │   (Configurable)        │
              └────────────┬────────────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
     ┌────▼─────┐    ┌────▼─────┐    ┌────▼─────┐
     │  Jaeger  │    │  Tempo   │    │   GCP    │
     │  (OSS)   │    │  (OSS)   │    │  Cloud   │
     └──────────┘    └──────────┘    └──────────┘
```

### Trace Hierarchy

```
Span Tree for a Single Request:

runner.run                          [ROOT SPAN]
├─ session.get_or_create           [Child]
├─ agent.run                       [Child]
│  ├─ agent.prepare_request        [Child]
│  ├─ llm.generate                 [Child]
│  │  └─ http.request              [Child]
│  ├─ agent.process_response       [Child]
│  ├─ tool.execute                 [Child] (if tool call)
│  │  └─ http.request              [Child] (for REST tools)
│  └─ llm.generate                 [Child] (next iteration)
├─ session.append_event            [Child] (per event)
└─ runner.complete                 [Child]
```

---

## 📦 Components to Build

### 1. Enhanced Span Management

**File**: `crates/zdk-telemetry/src/spans.rs`

#### New Span Types

```rust
/// Agent execution span
pub struct AgentSpan {
    span: tracing::Span,
    agent_name: String,
    invocation_id: String,
}

/// Runner execution span
pub struct RunnerSpan {
    span: tracing::Span,
    app_name: String,
    user_id: String,
    session_id: String,
}

/// Session operation span
pub struct SessionSpan {
    span: tracing::Span,
    operation: String,
    session_id: String,
}

/// Workflow agent span (for sequential/parallel/loop agents)
pub struct WorkflowSpan {
    span: tracing::Span,
    workflow_type: String,
    sub_agent_count: usize,
}
```

#### Span Creation Functions

```rust
pub fn trace_runner_start(attrs: RunnerSpanAttributes) -> RunnerSpan;
pub fn trace_agent_start(attrs: AgentSpanAttributes) -> AgentSpan;
pub fn trace_session_operation(attrs: SessionSpanAttributes) -> SessionSpan;
pub fn trace_workflow_start(attrs: WorkflowSpanAttributes) -> WorkflowSpan;

// Record completion
impl AgentSpan {
    pub fn complete(self, result: Result<()>);
    pub fn record_event(&self, name: &str, attrs: Vec<(&str, String)>);
}
```

---

### 2. OTLP Exporter Configuration

**File**: `crates/zdk-telemetry/src/exporters.rs` (NEW)

```rust
pub struct OtlpExporterConfig {
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub endpoint: String,
    
    /// Protocol: grpc or http/protobuf
    pub protocol: OtlpProtocol,
    
    /// Optional authentication headers
    pub headers: HashMap<String, String>,
    
    /// Batch configuration
    pub batch_config: BatchConfig,
    
    /// Timeout for exports
    pub timeout: Duration,
}

pub enum OtlpProtocol {
    Grpc,
    HttpProtobuf,
}

pub struct BatchConfig {
    /// Max time to wait before sending batch
    pub scheduled_delay: Duration,
    
    /// Max spans in a batch
    pub max_export_batch_size: usize,
    
    /// Max queue size
    pub max_queue_size: usize,
}

impl Default for OtlpExporterConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            protocol: OtlpProtocol::Grpc,
            headers: HashMap::new(),
            batch_config: BatchConfig::default(),
            timeout: Duration::from_secs(10),
        }
    }
}

pub fn create_otlp_exporter(
    config: OtlpExporterConfig
) -> Result<opentelemetry_otlp::SpanExporter>;
```

---

### 3. Trace Sampling Configuration

**File**: `crates/zdk-telemetry/src/sampling.rs` (NEW)

```rust
pub enum SamplingStrategy {
    /// Always sample (development)
    AlwaysOn,
    
    /// Never sample (production with selective tracing)
    AlwaysOff,
    
    /// Sample a percentage of traces
    Probability(f64),
    
    /// Sample based on parent span decision
    ParentBased(Box<SamplingStrategy>),
    
    /// Custom sampling logic
    Custom(Box<dyn Fn(&TraceContext) -> bool + Send + Sync>),
}

pub struct SamplingConfig {
    pub strategy: SamplingStrategy,
    
    /// Force sampling for error traces
    pub always_sample_errors: bool,
    
    /// Force sampling for slow traces (>threshold)
    pub always_sample_slow: Option<Duration>,
}
```

---

### 4. Configuration Integration

**Update**: `crates/zdk-core/src/config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable telemetry
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Enable structured logging
    #[serde(default = "default_true")]
    pub logging: bool,
    
    /// Enable distributed tracing
    #[serde(default)]
    pub tracing: TracingConfig,
    
    /// Enable metrics (future)
    #[serde(default)]
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// OTLP exporter configuration
    #[serde(default)]
    pub otlp: Option<OtlpConfig>,
    
    /// Sampling configuration
    #[serde(default)]
    pub sampling: SamplingConfig,
    
    /// Additional exporters (stdout, jaeger, etc.)
    #[serde(default)]
    pub exporters: Vec<ExporterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpConfig {
    pub endpoint: String,
    
    #[serde(default = "default_grpc")]
    pub protocol: String, // "grpc" or "http/protobuf"
    
    #[serde(default)]
    pub headers: HashMap<String, String>,
    
    #[serde(default)]
    pub timeout_seconds: u64,
}
```

**Example `config.toml`**:

```toml
[telemetry]
enabled = true
logging = true

[telemetry.tracing]
enabled = true

[telemetry.tracing.otlp]
endpoint = "http://localhost:4317"
protocol = "grpc"
timeout_seconds = 10

[telemetry.tracing.sampling]
strategy = "probability"
probability = 0.1  # Sample 10% of traces
always_sample_errors = true
always_sample_slow_seconds = 5.0
```

---

### 5. Instrumentation Points

#### Runner Instrumentation

**Update**: `crates/zdk-runner/src/runner.rs`

```rust
use rak_telemetry::{trace_runner_start, RunnerSpanAttributes};

impl Runner {
    pub async fn run(
        &self,
        user_id: String,
        session_id: String,
        message: Content,
        config: RunConfig,
    ) -> Result<Box<dyn Stream<Item = Result<Event>> + Send + Unpin>> {
        // Create root span for this run
        let runner_span = trace_runner_start(RunnerSpanAttributes {
            app_name: self.app_name.clone(),
            user_id: user_id.clone(),
            session_id: session_id.clone(),
            agent_name: self.agent.name().to_string(),
        });
        
        let _guard = runner_span.enter();
        
        // ... rest of implementation
        
        // Record events throughout execution
        runner_span.record_event("session_created", vec![]);
        runner_span.record_event("agent_started", vec![]);
        
        // Complete span on finish
        runner_span.complete(Ok(()));
    }
}
```

#### Agent Instrumentation

**Update**: `crates/zdk-agent/src/llm_agent.rs`

```rust
use rak_telemetry::{trace_agent_start, AgentSpanAttributes};

impl Agent for LLMAgent {
    #[tracing::instrument(
        name = "agent.run",
        skip(self, ctx),
        fields(
            agent_name = %self.name,
            invocation_id = %ctx.invocation_id(),
            session_id = %ctx.session_id(),
        )
    )]
    async fn run(
        &self,
        ctx: Arc<dyn InvocationContext>,
    ) -> Box<dyn Stream<Item = Result<Event>> + Send + Unpin> {
        // Create agent span
        let agent_span = trace_agent_start(AgentSpanAttributes {
            agent_name: self.name.clone(),
            invocation_id: ctx.invocation_id().to_string(),
            session_id: ctx.session_id().to_string(),
            system_instruction: self.system_instruction.clone(),
        });
        
        // ... implementation
    }
}
```

#### Workflow Agent Instrumentation

**Update**: `crates/zdk-agent/src/workflow.rs`

```rust
#[tracing::instrument(
    name = "workflow.sequential",
    skip(self, ctx),
    fields(
        workflow_type = "sequential",
        sub_agents = self.agents.len(),
    )
)]
async fn run(&self, ctx: Arc<dyn InvocationContext>) -> ... {
    for (idx, agent) in self.agents.iter().enumerate() {
        // Each sub-agent run creates a child span
        tracing::debug!(
            sub_agent_index = idx,
            sub_agent_name = agent.name(),
            "Starting sub-agent"
        );
        
        let mut stream = agent.run(ctx.clone()).await;
        // ...
    }
}
```

---

### 6. Context Propagation

**File**: `crates/zdk-telemetry/src/context.rs` (NEW)

```rust
use opentelemetry::Context as OtelContext;

/// Helper to propagate trace context across async boundaries
pub struct TraceContext {
    context: OtelContext,
}

impl TraceContext {
    /// Capture current trace context
    pub fn current() -> Self {
        Self {
            context: OtelContext::current(),
        }
    }
    
    /// Attach this context to current async task
    pub fn attach(&self) -> OtelContextGuard {
        self.context.attach()
    }
    
    /// Run a future with this context
    pub async fn scope<F, T>(self, f: F) -> T
    where
        F: Future<Output = T>,
    {
        let _guard = self.attach();
        f.await
    }
}
```

---

### 7. Metrics Collection (Optional but Recommended)

**File**: `crates/zdk-telemetry/src/metrics.rs` (NEW)

```rust
use opentelemetry::metrics::{Counter, Histogram};

pub struct TelemetryMetrics {
    /// Total agent invocations
    pub agent_invocations: Counter<u64>,
    
    /// Agent execution duration
    pub agent_duration: Histogram<f64>,
    
    /// LLM call count
    pub llm_calls: Counter<u64>,
    
    /// LLM call duration
    pub llm_duration: Histogram<f64>,
    
    /// Tool execution count
    pub tool_executions: Counter<u64>,
    
    /// Tool execution duration
    pub tool_duration: Histogram<f64>,
    
    /// Error count by type
    pub errors: Counter<u64>,
}

impl TelemetryMetrics {
    pub fn new(meter: Meter) -> Result<Self>;
    
    pub fn record_agent_start(&self, agent_name: &str);
    pub fn record_agent_complete(&self, agent_name: &str, duration: Duration);
    pub fn record_error(&self, error_type: &str);
}
```

---

## 🔧 Implementation Plan

### Phase 9.1: Enhanced Span Management ✅

**Tasks**:
1. Add new span types (AgentSpan, RunnerSpan, SessionSpan, WorkflowSpan)
2. Implement span lifecycle management (start, record_event, complete)
3. Add error tracking to spans
4. Add comprehensive span attributes
5. Write unit tests for span creation and management

**Files**:
- `crates/zdk-telemetry/src/spans.rs` (enhance existing)
- `crates/zdk-telemetry/src/lib.rs` (update exports)

**Tests**:
- Span creation with all attributes
- Span completion with success/error
- Span event recording
- Span attribute serialization

---

### Phase 9.2: OTLP Exporter Configuration ✅

**Tasks**:
1. Create OTLP exporter configuration types
2. Implement OTLP exporter builder (gRPC and HTTP)
3. Add batch configuration support
4. Add authentication header support
5. Integrate with telemetry initialization

**Files**:
- `crates/zdk-telemetry/src/exporters.rs` (new)
- `crates/zdk-telemetry/src/tracer.rs` (update)

**Dependencies**:
```toml
opentelemetry-otlp = { version = "0.15", features = ["grpc-tonic"] }
opentelemetry-semantic-conventions = "0.15"
tonic = "0.11"
```

**Tests**:
- OTLP exporter creation with default config
- OTLP exporter with custom endpoint
- Batch configuration validation

---

### Phase 9.3: Sampling Configuration ✅

**Tasks**:
1. Implement sampling strategies
2. Add parent-based sampling
3. Add probability-based sampling
4. Add custom sampling logic support
5. Integrate with tracer provider

**Files**:
- `crates/zdk-telemetry/src/sampling.rs` (new)
- `crates/zdk-telemetry/src/tracer.rs` (update)

**Tests**:
- Always-on sampling
- Probability sampling at different rates
- Parent-based sampling inheritance
- Error-based sampling

---

### Phase 9.4: Configuration Integration ✅

**Tasks**:
1. Add telemetry config to RakConfig
2. Support OTLP endpoint configuration
3. Support sampling configuration
4. Add config examples
5. Update config documentation

**Files**:
- `crates/zdk-core/src/config.rs` (update)
- `config.toml.example` (update)

**Config Example**:
```toml
[telemetry]
enabled = true

[telemetry.tracing]
enabled = true

[telemetry.tracing.otlp]
endpoint = "http://localhost:4317"
protocol = "grpc"

[telemetry.tracing.sampling]
strategy = "probability"
probability = 0.1
always_sample_errors = true
```

---

### Phase 9.5: Instrumentation ✅

**Tasks**:
1. Instrument Runner with spans
2. Instrument LLMAgent with spans
3. Instrument workflow agents (Sequential, Parallel, Loop)
4. Instrument session operations
5. Add context propagation

**Files**:
- `crates/zdk-runner/src/runner.rs` (update)
- `crates/zdk-agent/src/llm_agent.rs` (update)
- `crates/zdk-agent/src/workflow.rs` (update)
- `crates/zdk-session/src/inmemory.rs` (update)
- `crates/zdk-session/src/database.rs` (update)

**Instrumentation Points**:
- Runner start/complete
- Agent preparation
- LLM request/response
- Tool execution (already exists, enhance)
- Session operations (create, get, append)
- Workflow orchestration

---

### Phase 9.6: Context Propagation ✅

**Tasks**:
1. Create TraceContext helper
2. Implement context attachment for async tasks
3. Update stream processing to preserve context
4. Add context propagation to sub-agent calls
5. Test context propagation across async boundaries

**Files**:
- `crates/zdk-telemetry/src/context.rs` (new)
- `crates/zdk-runner/src/runner.rs` (update stream handling)
- `crates/zdk-agent/src/workflow.rs` (update sub-agent calls)

---

### Phase 9.7: Metrics (Optional) 🎯

**Tasks**:
1. Define metrics schema
2. Implement metrics collection
3. Add metrics exporters (OTLP, Prometheus)
4. Instrument key operations
5. Add metrics configuration

**Files**:
- `crates/zdk-telemetry/src/metrics.rs` (new)
- `crates/zdk-telemetry/src/exporters.rs` (update)

**Metrics**:
- `agent.invocations` (counter)
- `agent.duration` (histogram)
- `llm.calls` (counter)
- `llm.duration` (histogram)
- `tool.executions` (counter)
- `errors.total` (counter)

---

### Phase 9.8: Examples & Documentation ✅

**Tasks**:
1. Create comprehensive telemetry example
2. Update observability guide
3. Add Jaeger setup documentation
4. Add Tempo setup documentation
5. Add GCP Cloud Trace documentation
6. Create troubleshooting guide

**Files**:
- `examples/advanced_telemetry.rs` (new)
- `plans/20251123_1200_OBSERVABILITY_GUIDE.md` (new)
- `README.md` (update)

---

## 📊 Testing Strategy

### Unit Tests
- Span creation and lifecycle
- Configuration parsing
- Exporter creation
- Sampling logic

### Integration Tests
- End-to-end trace collection
- OTLP export to local collector
- Multiple exporters simultaneously
- Context propagation across agents

### Manual Testing
1. Run with Jaeger locally
2. Run with Tempo locally
3. Run with GCP Cloud Trace
4. Verify trace hierarchy in UI
5. Test sampling at different rates

---

## 🚀 Deployment Scenarios

### Scenario 1: Local Development with Jaeger

```bash
# Start Jaeger all-in-one
docker run -d -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:latest

# Configure ZDK
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# Run application
cargo run --example advanced_telemetry
```

### Scenario 2: Production with GCP Cloud Trace

```toml
[telemetry.tracing.otlp]
endpoint = "https://cloudtrace.googleapis.com/v2"
protocol = "http/protobuf"

[telemetry.tracing.otlp.headers]
"x-goog-api-key" = "${GCP_API_KEY}"

[telemetry.tracing.sampling]
strategy = "probability"
probability = 0.05  # 5% sampling in production
always_sample_errors = true
```

### Scenario 3: Kubernetes with Tempo

```yaml
# Tempo as sidecar or cluster service
apiVersion: v1
kind: ConfigMap
metadata:
  name: zdk-config
data:
  config.toml: |
    [telemetry.tracing.otlp]
    endpoint = "http://tempo.observability.svc.cluster.local:4317"
    protocol = "grpc"
    
    [telemetry.tracing.sampling]
    strategy = "parent_based"
```

---

## 📈 Observability Queries

### Example Queries in Jaeger/Tempo

1. **Find slow agent executions**:
   ```
   duration > 5s AND service.name="zdk-rust-app"
   ```

2. **Find all traces with errors**:
   ```
   error=true AND service.name="zdk-rust-app"
   ```

3. **Find traces for specific session**:
   ```
   gcp.vertex.agent.session_id="sess-123"
   ```

4. **Find tool execution traces**:
   ```
   span.name="execute_tool" AND gen_ai.tool.name="web_scraper"
   ```

---

## 🎯 Success Metrics

1. **Complete Trace Coverage**: Every agent execution produces a complete trace
2. **Performance Impact**: < 5% overhead with default sampling
3. **Context Preservation**: Parent-child relationships correctly maintained
4. **External Integration**: Successfully export to Jaeger, Tempo, and GCP
5. **Developer Experience**: Easy to configure and understand traces

---

## 🔄 Comparison with Python ZDK

| Feature | Python ZDK | Rust ZDK (Phase 9) |
|---------|-----------|-------------------|
| OpenTelemetry Tracing | ✅ | ✅ |
| LLM Call Tracing | ✅ | ✅ (exists) |
| Tool Call Tracing | ✅ | ✅ (exists) |
| Agent Lifecycle Tracing | ✅ | 🎯 New |
| Workflow Tracing | ✅ | 🎯 New |
| OTLP Exporter | ✅ | 🎯 New |
| Sampling Configuration | ✅ | 🎯 New |
| Metrics Collection | ✅ | 🎯 Optional |
| Context Propagation | ✅ | 🎯 New |

---

## 📝 Dependencies

### New Crate Dependencies

```toml
[dependencies]
# Existing
opentelemetry = "0.21"
opentelemetry-sdk = "0.21"
tracing = "0.1"
tracing-opentelemetry = "0.22"
tracing-subscriber = "0.3"

# New for Phase 9
opentelemetry-otlp = { version = "0.14", features = ["grpc-tonic", "http-proto"] }
opentelemetry-semantic-conventions = "0.13"
tonic = "0.10"
prost = "0.12"

# Optional for metrics
opentelemetry-prometheus = { version = "0.14", optional = true }
prometheus = { version = "0.13", optional = true }
```

---

## 🎓 Learning Resources

### OpenTelemetry
- [OpenTelemetry Rust Docs](https://docs.rs/opentelemetry/)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)
- [Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)

### Backends
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Grafana Tempo](https://grafana.com/docs/tempo/latest/)
- [GCP Cloud Trace](https://cloud.google.com/trace/docs)

---

## 🚦 Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Performance overhead | High | Implement sampling, lazy span creation |
| Context propagation complexity | Medium | Use tracing-opentelemetry's built-in context |
| Exporter failures | Medium | Add retry logic, buffering, and fallbacks |
| Large trace payloads | Medium | Implement span compression, attribute limits |
| Async span management | High | Use tracing's async instrumentation macros |

---

## ✅ Definition of Done

Phase 9 is complete when:

1. ✅ All span types implemented and tested
2. ✅ OTLP exporter configured and working
3. ✅ Sampling strategies implemented
4. ✅ Configuration integrated into RakConfig
5. ✅ All components instrumented (Runner, Agent, Workflow, Session)
6. ✅ Context propagation working across async boundaries
7. ✅ Complete example demonstrating full trace collection
8. ✅ Documentation updated with observability guide
9. ✅ Successfully tested with Jaeger locally
10. ✅ Successfully tested with at least one cloud provider

---

## 📅 Estimated Timeline

- **Phase 9.1**: Enhanced Span Management - 4 hours
- **Phase 9.2**: OTLP Exporter - 6 hours
- **Phase 9.3**: Sampling Configuration - 4 hours
- **Phase 9.4**: Configuration Integration - 3 hours
- **Phase 9.5**: Instrumentation - 8 hours
- **Phase 9.6**: Context Propagation - 6 hours
- **Phase 9.7**: Metrics (Optional) - 8 hours
- **Phase 9.8**: Examples & Docs - 6 hours

**Total**: ~45 hours (excluding metrics: ~37 hours)

---

## 🎯 Next Steps

1. Review and approve this design
2. Start with Phase 9.1: Enhanced Span Management
3. Iterate through each phase
4. Test with real observability backends
5. Document patterns and best practices

---

**Ready to implement?** Let's build world-class observability for ZDK! 🚀

