# Complete Observability System Architecture

**Date**: 2025-11-23 12:20  
**Status**: 🎯 Comprehensive Design  
**Integrates**: Phase 9 + Profiling

---

## 🎯 Executive Summary

This document describes a **complete observability system** for AI agent applications, providing:

1. **Distributed Tracing** - What happened, when, and in what order
2. **Metrics** - Aggregated statistics and health indicators
3. **CPU Profiling** - Where CPU time is spent
4. **Memory Profiling** - Memory allocation patterns and leaks
5. **Resource Attribution** - Cost tracking per agent/session

**Key Principle**: This is a **client-side** implementation that exports data to external backends for storage, analysis, and visualization.

---

## 🏗️ Complete Architecture

```
┌───────────────────────────────────────────────────────────────────┐
│                      ZDK Application Layer                         │
│                                                                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │  Runner  │  │  Agents  │  │ Session  │  │   LLM    │         │
│  │          │  │          │  │          │  │          │         │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘         │
│       │             │              │             │                │
│       │    Instrumentation Points  │             │                │
│       │             │              │             │                │
└───────┼─────────────┼──────────────┼─────────────┼────────────────┘
        │             │              │             │
        └─────────────┴──────────────┴─────────────┘
                      │
        ┌─────────────▼──────────────┐
        │   zdk-telemetry (Enhanced) │
        │                            │
        │  ┌─────────────────────┐  │
        │  │  Trace Collection   │  │
        │  │  - Spans            │  │
        │  │  - Context Props    │  │
        │  │  - Sampling         │  │
        │  └─────────────────────┘  │
        │                            │
        │  ┌─────────────────────┐  │
        │  │  Metrics Collection │  │
        │  │  - Counters         │  │
        │  │  - Histograms       │  │
        │  │  - Gauges           │  │
        │  └─────────────────────┘  │
        │                            │
        │  ┌─────────────────────┐  │
        │  │  CPU Profiling      │  │
        │  │  - pprof            │  │
        │  │  - Flamegraphs      │  │
        │  │  - Sampling         │  │
        │  └─────────────────────┘  │
        │                            │
        │  ┌─────────────────────┐  │
        │  │  Memory Profiling   │  │
        │  │  - Allocations      │  │
        │  │  - RSS/VM Size      │  │
        │  │  - Leak Detection   │  │
        │  └─────────────────────┘  │
        │                            │
        │  ┌─────────────────────┐  │
        │  │  Resource           │  │
        │  │  Attribution        │  │
        │  │  - Per Agent        │  │
        │  │  - Per Session      │  │
        │  └─────────────────────┘  │
        └──────────┬─────────────────┘
                   │
        ┌──────────▼──────────────────┐
        │   Export Layer              │
        │                             │
        │  ┌────────┐  ┌────────┐    │
        │  │  OTLP  │  │ pprof  │    │
        │  │ (gRPC/ │  │Protocol│    │
        │  │  HTTP) │  │        │    │
        │  └────────┘  └────────┘    │
        └──────────┬──────────────────┘
                   │
     ┌─────────────┼─────────────┬────────────┐
     │             │             │            │
┌────▼────┐  ┌────▼────┐  ┌────▼────┐  ┌────▼────┐
│ Jaeger/ │  │Pyroscope│  │Prometheus│  │   GCP   │
│  Tempo  │  │         │  │         │  │  Cloud  │
│         │  │         │  │         │  │Platform │
└─────────┘  └─────────┘  └─────────┘  └─────────┘
   Traces     Profiles      Metrics       All
```

---

## 📦 Component Breakdown

### 1. Telemetry Collection (`zdk-telemetry`)

#### Module Structure
```
crates/zdk-telemetry/
├── src/
│   ├── lib.rs                    # Public API
│   ├── tracer.rs                 # OpenTelemetry setup
│   ├── spans.rs                  # Span creation helpers
│   ├── attributes.rs             # Span attribute constants
│   ├── context.rs                # Context propagation (NEW)
│   ├── sampling.rs               # Sampling strategies (NEW)
│   ├── exporters.rs              # OTLP exporters (NEW)
│   ├── metrics.rs                # Metrics collection (NEW)
│   └── profiling/
│       ├── mod.rs
│       ├── cpu.rs                # CPU profiling
│       ├── memory.rs             # Memory profiling
│       ├── attribution.rs        # Resource attribution
│       ├── exporters.rs          # Profile exporters
│       └── collector.rs          # Periodic collection
```

---

### 2. Data Types Collected

#### A. Traces (OpenTelemetry Spans)

**Hierarchy**:
```
runner.run (ROOT)
├─ session.operation
├─ agent.run
│  ├─ agent.prepare_request
│  ├─ llm.generate
│  │  └─ http.request
│  ├─ tool.execute
│  │  └─ http.request (for REST tools)
│  └─ llm.generate (next iteration)
└─ session.append_event
```

**Span Attributes**:
```rust
// Standard OpenTelemetry
- span.name: "agent.run"
- span.kind: INTERNAL/CLIENT/SERVER
- trace.id: "abc123..."
- span.id: "def456..."
- parent.span.id: "ghi789..."

// Application-specific
- invocation_id: "inv-xxx"
- session_id: "sess-yyy"
- user_id: "user-zzz"
- agent.name: "research_agent"
- agent.type: "llm" | "sequential" | "parallel"

// LLM-specific
- gen_ai.system: "gcp.vertex.agent"
- gen_ai.request.model: "gemini-2.0-flash"
- gen_ai.request.top_p: 0.95
- gen_ai.request.max_tokens: 1024

// Tool-specific
- gen_ai.tool.name: "web_scraper"
- gen_ai.tool.call.id: "call-123"

// Resource usage (NEW)
- resource.cpu.micros: 450000
- resource.memory.bytes: 2048000
- resource.memory.peak: 3072000
```

#### B. Metrics (OpenTelemetry Metrics)

**Counter Metrics**:
```rust
- agent.invocations.total      // Total agent runs
- llm.calls.total               // Total LLM calls
- tool.executions.total         // Total tool executions
- errors.total                  // Total errors
  - labels: error_type, agent_name
```

**Histogram Metrics**:
```rust
- agent.duration.seconds        // Agent execution time
- llm.duration.seconds          // LLM call duration
- tool.duration.seconds         // Tool execution duration
- session.events.count          // Events per session
```

**Gauge Metrics**:
```rust
- agent.active                  // Currently running agents
- memory.current.bytes          // Current memory usage
- memory.peak.bytes             // Peak memory usage
- cpu.utilization.percent       // CPU utilization
```

#### C. CPU Profiles (pprof format)

**Profile Structure**:
```
Sample {
  location_id: [stack trace]
  value: [cpu_nanoseconds]
  label: {
    "trace_id": "abc123",
    "span_id": "def456",
    "agent_name": "research_agent",
    "invocation_id": "inv-xxx"
  }
}
```

**Output Formats**:
- **pprof protobuf**: For Pyroscope, GCP Cloud Profiler
- **Flamegraph SVG**: For visualization
- **Text report**: For debugging

#### D. Memory Profiles

**Tracked Data**:
```rust
struct MemoryStats {
  total_allocated: u64,      // Lifetime allocations
  current_usage: u64,        // Current heap size
  peak_usage: u64,           // Maximum seen
  allocation_count: u64,     // Number of allocations
}

struct SystemMemoryInfo {
  rss_bytes: u64,            // Resident Set Size (physical)
  virtual_bytes: u64,        // Virtual memory
}
```

---

### 3. Configuration Schema

**Complete `config.toml`**:

```toml
[telemetry]
# Master switch
enabled = true

# Structured logging
logging = true

# ─── Distributed Tracing ───────────────────────────────────

[telemetry.tracing]
enabled = true

# OTLP Exporter (Jaeger, Tempo, GCP Cloud Trace)
[telemetry.tracing.otlp]
endpoint = "http://localhost:4317"
protocol = "grpc"  # or "http/protobuf"
timeout_seconds = 10

# Optional authentication
[telemetry.tracing.otlp.headers]
"x-api-key" = "${API_KEY}"
"x-goog-api-key" = "${GCP_API_KEY}"

# Sampling (reduce data volume)
[telemetry.tracing.sampling]
strategy = "probability"         # "always_on", "always_off", "probability", "parent_based"
probability = 0.1                # Sample 10% of traces
always_sample_errors = true      # Always sample traces with errors
always_sample_slow_seconds = 5.0 # Always sample slow traces (>5s)

# Additional exporters
[[telemetry.tracing.exporters]]
type = "stdout"  # For debugging

[[telemetry.tracing.exporters]]
type = "jaeger"
endpoint = "http://localhost:14250"

# ─── Metrics ────────────────────────────────────────────────

[telemetry.metrics]
enabled = true

# Prometheus exporter
[telemetry.metrics.prometheus]
enabled = true
port = 9090
path = "/metrics"

# OTLP metrics exporter
[telemetry.metrics.otlp]
endpoint = "http://localhost:4317"
protocol = "grpc"
interval_seconds = 60  # Push interval

# ─── CPU Profiling ──────────────────────────────────────────

[telemetry.profiling]
enabled = true

[telemetry.profiling.cpu]
enabled = true
frequency = 100                   # Sampling frequency (Hz)
continuous = true                 # Continuous vs on-demand
duration_seconds = 30             # Profile duration for periodic

# ─── Memory Profiling ───────────────────────────────────────

[telemetry.profiling.memory]
enabled = true
track_allocations = false         # Expensive, use for debugging
sampling_rate = 100               # Sample 1/100 allocations
heap_profiling = false            # Very expensive

# ─── Profile Exporters ──────────────────────────────────────

[telemetry.profiling.exporter]
type = "pyroscope"                # "pyroscope", "gcp", "file"

# Pyroscope
pyroscope_endpoint = "http://localhost:4040"
application_name = "zdk-rust-app"

# GCP Cloud Profiler
# gcp_project_id = "my-project"
# gcp_service_name = "zdk-service"

# File output (development)
# file_output_dir = "./profiles"
```

---

## 🔧 Implementation Checklist

### Core Tracing (Exists in Phase 7)
- [x] LLM call tracing
- [x] Tool execution tracing
- [x] Basic span attributes
- [x] OpenTelemetry integration

### Phase 9.1: Enhanced Span Management
- [ ] AgentSpan with lifecycle
- [ ] RunnerSpan with context
- [ ] SessionSpan for operations
- [ ] WorkflowSpan for orchestration
- [ ] Error tracking in spans
- [ ] Event recording in spans

### Phase 9.2: OTLP Exporter
- [ ] OTLP gRPC exporter
- [ ] OTLP HTTP/protobuf exporter
- [ ] Batch configuration
- [ ] Authentication headers
- [ ] Retry logic
- [ ] Export error handling

### Phase 9.3: Sampling
- [ ] Always-on/off strategies
- [ ] Probability-based sampling
- [ ] Parent-based sampling
- [ ] Error-based sampling
- [ ] Slow-trace sampling
- [ ] Custom sampling logic

### Phase 9.4: Configuration
- [ ] Telemetry config in RakConfig
- [ ] OTLP endpoint configuration
- [ ] Sampling configuration
- [ ] Exporter selection
- [ ] Environment variable overrides

### Phase 9.5: Instrumentation
- [ ] Runner start/complete spans
- [ ] Agent lifecycle spans
- [ ] LLM request/response spans
- [ ] Tool execution spans (enhance)
- [ ] Session operation spans
- [ ] Workflow orchestration spans

### Phase 9.6: Context Propagation
- [ ] TraceContext helper
- [ ] Async context attachment
- [ ] Stream context preservation
- [ ] Sub-agent context propagation
- [ ] Cross-service propagation (future)

### Phase 9.7: Metrics
- [ ] Counter metrics
- [ ] Histogram metrics
- [ ] Gauge metrics
- [ ] Metrics exporter (OTLP)
- [ ] Prometheus exporter
- [ ] Metrics configuration

### Phase 9.8: CPU Profiling
- [ ] pprof integration
- [ ] CPU profiler with sampling
- [ ] Flamegraph generation
- [ ] pprof protobuf export
- [ ] Pyroscope exporter
- [ ] GCP Cloud Profiler exporter

### Phase 9.9: Memory Profiling
- [ ] Memory statistics tracking
- [ ] Platform-specific memory info
- [ ] Allocation tracking (sampled)
- [ ] Heap profiling (optional)
- [ ] Memory leak detection

### Phase 9.10: Resource Attribution
- [ ] Per-invocation CPU tracking
- [ ] Per-invocation memory tracking
- [ ] Agent-level aggregation
- [ ] Session-level aggregation
- [ ] Span attribute integration

### Phase 9.11: Integration
- [ ] Runner profiler integration
- [ ] Periodic profile collection
- [ ] Profile-trace correlation
- [ ] Configuration validation
- [ ] Performance benchmarking

### Phase 9.12: Examples & Docs
- [ ] Complete telemetry example
- [ ] Observability guide
- [ ] Jaeger setup guide
- [ ] Pyroscope setup guide
- [ ] GCP integration guide
- [ ] Troubleshooting guide

---

## 📈 Usage Scenarios

### Scenario 1: Local Development with Full Observability

**Setup**:
```bash
# Start observability stack
docker-compose up -d

# docker-compose.yml
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "4317:4317"   # OTLP gRPC
      - "16686:16686" # UI
  
  pyroscope:
    image: pyroscope/pyroscope:latest
    ports:
      - "4040:4040"
  
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
```

**Config**:
```toml
[telemetry.tracing]
enabled = true

[telemetry.tracing.otlp]
endpoint = "http://localhost:4317"

[telemetry.profiling]
enabled = true

[telemetry.profiling.exporter]
type = "pyroscope"
pyroscope_endpoint = "http://localhost:4040"
```

**Access**:
- Traces: http://localhost:16686 (Jaeger UI)
- Profiles: http://localhost:4040 (Pyroscope UI)
- Metrics: http://localhost:9090 (Prometheus)

---

### Scenario 2: Production with GCP Cloud Platform

**Config**:
```toml
[telemetry.tracing]
enabled = true

[telemetry.tracing.otlp]
endpoint = "https://cloudtrace.googleapis.com/v2/projects/${GCP_PROJECT}/traces:batchWrite"
protocol = "http/protobuf"

[telemetry.tracing.otlp.headers]
"x-goog-api-key" = "${GCP_API_KEY}"

[telemetry.tracing.sampling]
strategy = "probability"
probability = 0.05  # 5% sampling
always_sample_errors = true

[telemetry.profiling]
enabled = true

[telemetry.profiling.exporter]
type = "gcp"
gcp_project_id = "${GCP_PROJECT}"
gcp_service_name = "zdk-production"
```

**Access**:
- Cloud Trace: https://console.cloud.google.com/traces
- Cloud Profiler: https://console.cloud.google.com/profiler

---

### Scenario 3: Kubernetes with Tempo & Prometheus

**K8s ConfigMap**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: zdk-config
data:
  config.toml: |
    [telemetry.tracing.otlp]
    endpoint = "http://tempo.observability.svc:4317"
    
    [telemetry.metrics.prometheus]
    enabled = true
    port = 9090
```

**ServiceMonitor** (Prometheus Operator):
```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: zdk-metrics
spec:
  selector:
    matchLabels:
      app: rak
  endpoints:
  - port: metrics
    interval: 30s
```

---

## 🔍 Debugging Workflow

### 1. Find Slow Requests

**In Jaeger**:
1. Search by service: `zdk-rust-app`
2. Filter: `min duration: 5s`
3. Select trace → see full hierarchy

**What you see**:
```
runner.run (8.2s)
├─ session.get (0.1s)
├─ agent.run (7.9s)
│  ├─ llm.generate (2.1s) ← LLM call
│  ├─ tool.execute (5.5s) ← SLOW TOOL!
│  │  └─ http.request (5.4s)
│  └─ llm.generate (0.3s)
└─ session.append_event (0.2s)
```

**Insight**: The `web_scraper` tool is taking 5.5s!

---

### 2. Check CPU Profile for That Time

**In Pyroscope**:
1. Select time range: 12:30:00 - 12:30:10
2. Filter by label: `agent_name=research_agent`
3. View flamegraph

**What you see**:
```
├─ scraper::parse_html (60% of CPU)
│  ├─ regex::match (40%)
│  └─ html5ever::parse (20%)
├─ http::client::send (30%)
└─ other (10%)
```

**Insight**: Regex matching in HTML parser is the bottleneck!

---

### 3. Check Memory Usage

**In span attributes**:
```
resource.memory.bytes: 15MB (allocated during execution)
resource.memory.peak: 45MB (peak usage)
```

**In Pyroscope memory profile**:
```
├─ scraper::download (25MB)
├─ html5ever::parse (15MB)
└─ regex::compile (5MB)
```

**Insight**: Large HTML document being downloaded and parsed.

---

### 4. Correlate with Metrics

**Prometheus query**:
```promql
# 95th percentile latency for web_scraper tool
histogram_quantile(0.95, 
  rate(tool_duration_seconds_bucket{tool_name="web_scraper"}[5m])
)
```

**Result**: P95 = 5.2s (consistent with our trace)

---

### 5. Root Cause & Fix

**Problem**: `web_scraper` downloads entire page into memory, then applies regex.

**Fix**: 
1. Stream HTML parsing (reduce memory)
2. Optimize regex patterns
3. Add timeout for large pages
4. Cache parsed results

**Validation**:
- Re-run with profiling enabled
- Verify P95 latency drops to < 1s
- Check memory usage drops to < 5MB

---

## 📊 Data Volume Estimates

### Traces

**Without Sampling**:
- 1000 agent runs/hour
- ~10 spans per run
- ~1KB per span
- **Total**: ~10 MB/hour

**With 10% Sampling**:
- **Total**: ~1 MB/hour

### Profiles

**CPU Profiles** (30s intervals):
- 120 profiles/hour
- ~50KB per profile (compressed)
- **Total**: ~6 MB/hour

**Memory Stats** (1min intervals):
- 60 snapshots/hour
- ~1KB per snapshot
- **Total**: ~60 KB/hour

### Metrics

- ~50 metrics
- 1-minute granularity
- **Total**: ~500 KB/hour

**Grand Total**: ~8 MB/hour (with sampling)

---

## ⚡ Performance Impact

| Component | Overhead | Mitigation |
|-----------|----------|------------|
| Tracing | 1-2% | Use sampling (10-20%) |
| Metrics | < 1% | Aggregate locally, push periodically |
| CPU Profiling | 1-3% | Use 50-100 Hz sampling |
| Memory Profiling | 2-5% | Sample 1% of allocations |
| **Total** | **~5-10%** | **Acceptable for production** |

---

## ✅ Success Metrics

### Technical Metrics
1. **Trace Coverage**: 100% of agent runs have traces
2. **Context Preservation**: Parent-child relationships correct
3. **Performance**: < 5% overhead with default config
4. **Reliability**: < 0.1% export failures
5. **Latency**: < 100ms from span creation to export

### Business Metrics
1. **MTTR**: Mean Time To Resolution reduced by 50%
2. **Debugging**: 90% of issues debuggable from traces alone
3. **Optimization**: Identify bottlenecks in < 5 minutes
4. **Cost**: Attribute resource costs to specific agents

---

## 🎓 Best Practices

### 1. Sampling Strategy
```toml
# Development
[telemetry.tracing.sampling]
strategy = "always_on"

# Staging
strategy = "probability"
probability = 0.5  # 50%

# Production
strategy = "probability"
probability = 0.1  # 10%
always_sample_errors = true
always_sample_slow_seconds = 5.0
```

### 2. Attribute Management
```rust
// DO: Add meaningful attributes
span.set_attribute("agent.name", agent_name);
span.set_attribute("model.name", "gemini-2.0");
span.set_attribute("user.tier", "premium");

// DON'T: Add PII or sensitive data
span.set_attribute("user.email", email); // ❌
span.set_attribute("api.key", key);      // ❌
```

### 3. Profiling in Production
```toml
# Use lower sampling for CPU
[telemetry.profiling.cpu]
frequency = 50  # 50 Hz instead of 100 Hz

# Use sampling for memory
[telemetry.profiling.memory]
sampling_rate = 1000  # 0.1% sampling
```

### 4. Export Batching
```toml
[telemetry.tracing.otlp]
# Batch exports to reduce network calls
[telemetry.tracing.otlp.batch]
scheduled_delay_seconds = 5
max_export_batch_size = 512
max_queue_size = 2048
```

---

## 🚀 Rollout Plan

### Phase 1: Development (Week 1-2)
- Implement core tracing
- OTLP exporter
- Local Jaeger testing
- Documentation

### Phase 2: Staging (Week 3)
- Add CPU profiling
- Add memory profiling
- Pyroscope integration
- Load testing

### Phase 3: Production Pilot (Week 4)
- Deploy to 10% of traffic
- Monitor overhead
- Validate data quality
- Fix any issues

### Phase 4: Full Rollout (Week 5-6)
- Deploy to 100% of traffic
- Enable sampling (10%)
- Set up alerting
- Train team on usage

---

## 📚 Resources

### OpenTelemetry
- [Specification](https://opentelemetry.io/docs/specs/)
- [Rust SDK](https://docs.rs/opentelemetry/)
- [Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)

### Profiling
- [pprof](https://github.com/google/pprof)
- [Pyroscope](https://pyroscope.io/docs/)
- [GCP Cloud Profiler](https://cloud.google.com/profiler/docs)

### Backends
- [Jaeger](https://www.jaegertracing.io/)
- [Grafana Tempo](https://grafana.com/docs/tempo/)
- [Prometheus](https://prometheus.io/docs/)

---

## 🎯 Conclusion

This complete observability system provides:

✅ **Full Visibility**: Traces, metrics, and profiles  
✅ **Low Overhead**: < 5% with sampling  
✅ **Production Ready**: Proven technologies  
✅ **Flexible**: Multiple backend options  
✅ **Actionable**: Fast debugging and optimization  

**Total Implementation Time**: ~70 hours (~2 weeks for 1 developer)

**ROI**: 
- 50% reduction in debugging time
- 90% faster root cause analysis
- Proactive performance optimization
- Cost attribution and optimization

---

**Ready to build world-class observability?** 🚀

Let's start with Phase 9.1 and iterate through each component systematically!

