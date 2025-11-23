# Phase 9: Profiling Integration - CPU & Memory Analysis

**Date**: 2025-11-23 12:10  
**Status**: 🎯 Design Phase  
**Extends**: Phase 9 Advanced Observability

---

## 🎯 Objectives

Add **continuous profiling** capabilities to track CPU and memory usage at runtime, enabling:
- Identify memory leaks and excessive allocations
- Find CPU-intensive code paths (hot spots)
- Understand resource consumption per agent/session
- Correlate performance issues with specific traces
- Optimize agent execution efficiency

**Key Principle**: Profile data complements traces - traces show **what happened**, profiles show **resource costs**.

---

## 📊 Profiling vs Tracing vs Metrics

| Aspect | Traces | Metrics | Profiles |
|--------|--------|---------|----------|
| **What** | Execution flow | Aggregated stats | Resource usage |
| **When** | Per request | Continuous | Continuous/Sampled |
| **Granularity** | Spans/operations | Counters/histograms | Function-level |
| **Use Case** | Debugging flow | Monitoring health | Finding bottlenecks |
| **Example** | "Agent took 5s" | "95p latency: 2s" | "malloc used 500MB" |

**Integration**: Link profiles to traces via `trace_id` for complete picture.

---

## 🏗️ Architecture Changes

### Updated Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     RAK Application                          │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │          Telemetry Collector (Enhanced)              │  │
│  │                                                       │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌────────┐ │  │
│  │  │ Traces  │  │ Metrics │  │   CPU   │  │ Memory │ │  │
│  │  │         │  │         │  │ Profile │  │ Profile│ │  │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └───┬────┘ │  │
│  │       │            │            │            │       │  │
│  └───────┼────────────┼────────────┼────────────┼───────┘  │
│          │            │            │            │           │
└──────────┼────────────┼────────────┼────────────┼───────────┘
           │            │            │            │
           │            │            │            │
      ┌────▼────────────▼────────────▼────────────▼────┐
      │      OTLP / Profiling Protocol Export         │
      └────┬────────────┬────────────┬────────────┬────┘
           │            │            │            │
    ┌──────▼──┐   ┌─────▼────┐  ┌───▼────┐  ┌───▼────┐
    │ Jaeger/ │   │Prometheus│  │Pyro-   │  │  GCP   │
    │ Tempo   │   │          │  │scope   │  │Profiler│
    └─────────┘   └──────────┘  └────────┘  └────────┘
```

---

## 📦 New Components

### 1. Profiling Module

**File**: `crates/rak-telemetry/src/profiling/mod.rs` (NEW)

```rust
pub mod cpu;
pub mod memory;
pub mod config;
pub mod exporters;

pub use cpu::CpuProfiler;
pub use memory::MemoryProfiler;
pub use config::ProfilingConfig;
```

---

### 2. CPU Profiling

**File**: `crates/rak-telemetry/src/profiling/cpu.rs` (NEW)

```rust
use pprof::ProfilerGuard;
use std::time::Duration;

/// CPU profiler using pprof format
pub struct CpuProfiler {
    /// Sampling frequency (Hz)
    frequency: u32,
    
    /// Whether profiling is active
    active: bool,
    
    /// Current profiler guard (if running)
    guard: Option<ProfilerGuard<'static>>,
}

impl CpuProfiler {
    pub fn new(config: CpuProfilingConfig) -> Result<Self> {
        Ok(Self {
            frequency: config.frequency,
            active: false,
            guard: None,
        })
    }
    
    /// Start CPU profiling
    pub fn start(&mut self) -> Result<()> {
        if self.active {
            return Ok(());
        }
        
        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(self.frequency as i32)
            .blocklist(&["libc", "libpthread"])
            .build()?;
        
        self.guard = Some(guard);
        self.active = true;
        
        tracing::info!(frequency = %self.frequency, "CPU profiling started");
        Ok(())
    }
    
    /// Stop and collect CPU profile
    pub fn stop_and_collect(&mut self) -> Result<pprof::Report> {
        if !self.active {
            return Err(Error::Other(anyhow::anyhow!("Profiler not active")));
        }
        
        let guard = self.guard.take()
            .ok_or_else(|| Error::Other(anyhow::anyhow!("No profiler guard")))?;
        
        let report = guard.report().build()?;
        self.active = false;
        
        tracing::info!("CPU profile collected");
        Ok(report)
    }
    
    /// Get flamegraph as SVG
    pub fn flamegraph(&mut self) -> Result<Vec<u8>> {
        let report = self.stop_and_collect()?;
        let mut buf = Vec::new();
        report.flamegraph(&mut buf)?;
        Ok(buf)
    }
    
    /// Get profile in pprof protobuf format
    pub fn pprof(&mut self) -> Result<Vec<u8>> {
        let report = self.stop_and_collect()?;
        let mut buf = Vec::new();
        report.pprof()?.encode(&mut buf)?;
        Ok(buf)
    }
}

#[derive(Debug, Clone)]
pub struct CpuProfilingConfig {
    /// Sampling frequency in Hz (default: 100)
    pub frequency: u32,
    
    /// Profile duration (for periodic profiling)
    pub duration: Duration,
    
    /// Whether to enable continuous profiling
    pub continuous: bool,
}

impl Default for CpuProfilingConfig {
    fn default() -> Self {
        Self {
            frequency: 100, // 100 Hz = 10ms intervals
            duration: Duration::from_secs(30),
            continuous: false,
        }
    }
}
```

---

### 3. Memory Profiling

**File**: `crates/rak-telemetry/src/profiling/memory.rs` (NEW)

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Memory profiler tracking allocations and usage
pub struct MemoryProfiler {
    /// Total bytes allocated (across all time)
    total_allocated: Arc<AtomicU64>,
    
    /// Current bytes in use
    current_usage: Arc<AtomicU64>,
    
    /// Peak memory usage
    peak_usage: Arc<AtomicU64>,
    
    /// Allocation count
    allocation_count: Arc<AtomicU64>,
    
    /// Configuration
    config: MemoryProfilingConfig,
}

impl MemoryProfiler {
    pub fn new(config: MemoryProfilingConfig) -> Self {
        Self {
            total_allocated: Arc::new(AtomicU64::new(0)),
            current_usage: Arc::new(AtomicU64::new(0)),
            peak_usage: Arc::new(AtomicU64::new(0)),
            allocation_count: Arc::new(AtomicU64::new(0)),
            config,
        }
    }
    
    /// Record an allocation
    pub fn record_allocation(&self, size: u64) {
        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        
        let new_usage = self.current_usage.fetch_add(size, Ordering::Relaxed) + size;
        
        // Update peak if necessary
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while new_usage > peak {
            match self.peak_usage.compare_exchange_weak(
                peak,
                new_usage,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
    }
    
    /// Record a deallocation
    pub fn record_deallocation(&self, size: u64) {
        self.current_usage.fetch_sub(size, Ordering::Relaxed);
    }
    
    /// Get current memory statistics
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            current_usage: self.current_usage.load(Ordering::Relaxed),
            peak_usage: self.peak_usage.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
        }
    }
    
    /// Get system memory info (via procfs or system calls)
    pub fn system_memory(&self) -> Result<SystemMemoryInfo> {
        #[cfg(target_os = "linux")]
        {
            self.linux_memory_info()
        }
        
        #[cfg(target_os = "macos")]
        {
            self.macos_memory_info()
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(Error::Other(anyhow::anyhow!("Platform not supported")))
        }
    }
    
    #[cfg(target_os = "linux")]
    fn linux_memory_info(&self) -> Result<SystemMemoryInfo> {
        use std::fs;
        
        let status = fs::read_to_string("/proc/self/status")?;
        let mut rss = 0;
        let mut vm_size = 0;
        
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                rss = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("VmSize:") {
                vm_size = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        
        Ok(SystemMemoryInfo {
            rss_bytes: rss * 1024, // Convert KB to bytes
            virtual_bytes: vm_size * 1024,
        })
    }
    
    #[cfg(target_os = "macos")]
    fn macos_memory_info(&self) -> Result<SystemMemoryInfo> {
        // Use mach task_info
        use libc::{c_int, size_t};
        
        unsafe {
            let mut info: libc::task_basic_info = std::mem::zeroed();
            let mut count = (std::mem::size_of::<libc::task_basic_info>() / 
                            std::mem::size_of::<libc::natural_t>()) as u32;
            
            let kr = libc::task_info(
                libc::mach_task_self(),
                libc::TASK_BASIC_INFO,
                &mut info as *mut _ as *mut i32,
                &mut count,
            );
            
            if kr == libc::KERN_SUCCESS {
                Ok(SystemMemoryInfo {
                    rss_bytes: info.resident_size,
                    virtual_bytes: info.virtual_size,
                })
            } else {
                Err(Error::Other(anyhow::anyhow!("task_info failed")))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    /// Total bytes allocated (lifetime)
    pub total_allocated: u64,
    
    /// Current bytes in use
    pub current_usage: u64,
    
    /// Peak memory usage
    pub peak_usage: u64,
    
    /// Total allocation count
    pub allocation_count: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct SystemMemoryInfo {
    /// Resident set size (physical memory)
    pub rss_bytes: u64,
    
    /// Virtual memory size
    pub virtual_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct MemoryProfilingConfig {
    /// Track individual allocations (expensive)
    pub track_allocations: bool,
    
    /// Sampling rate (1 = track all, 100 = track 1/100)
    pub sampling_rate: u64,
    
    /// Enable heap profiling (very expensive)
    pub heap_profiling: bool,
}

impl Default for MemoryProfilingConfig {
    fn default() -> Self {
        Self {
            track_allocations: false, // Too expensive for production
            sampling_rate: 100, // Sample 1% of allocations
            heap_profiling: false,
        }
    }
}
```

---

### 4. Resource Attribution

**File**: `crates/rak-telemetry/src/profiling/attribution.rs` (NEW)

```rust
use std::sync::Arc;
use dashmap::DashMap;

/// Track resource usage per agent/session/invocation
pub struct ResourceAttribution {
    /// CPU time per invocation (microseconds)
    cpu_time: Arc<DashMap<String, u64>>,
    
    /// Memory allocated per invocation (bytes)
    memory_allocated: Arc<DashMap<String, u64>>,
    
    /// Active invocations
    active: Arc<DashMap<String, ResourceContext>>,
}

#[derive(Debug, Clone)]
pub struct ResourceContext {
    pub invocation_id: String,
    pub session_id: String,
    pub agent_name: String,
    pub start_time: std::time::Instant,
    pub start_memory: u64,
}

impl ResourceAttribution {
    pub fn new() -> Self {
        Self {
            cpu_time: Arc::new(DashMap::new()),
            memory_allocated: Arc::new(DashMap::new()),
            active: Arc::new(DashMap::new()),
        }
    }
    
    /// Start tracking resources for an invocation
    pub fn start_invocation(&self, ctx: ResourceContext) {
        self.active.insert(ctx.invocation_id.clone(), ctx);
    }
    
    /// Stop tracking and record final resource usage
    pub fn end_invocation(&self, invocation_id: &str, end_memory: u64) {
        if let Some((_, ctx)) = self.active.remove(invocation_id) {
            let duration = ctx.start_time.elapsed();
            let cpu_micros = duration.as_micros() as u64;
            let memory_delta = end_memory.saturating_sub(ctx.start_memory);
            
            self.cpu_time.insert(invocation_id.to_string(), cpu_micros);
            self.memory_allocated.insert(invocation_id.to_string(), memory_delta);
            
            // Also record in span attributes
            tracing::info!(
                invocation_id = %invocation_id,
                agent = %ctx.agent_name,
                cpu_micros = cpu_micros,
                memory_bytes = memory_delta,
                "Invocation resource usage"
            );
        }
    }
    
    /// Get resource usage for an invocation
    pub fn get_usage(&self, invocation_id: &str) -> Option<ResourceUsage> {
        let cpu = self.cpu_time.get(invocation_id).map(|v| *v);
        let memory = self.memory_allocated.get(invocation_id).map(|v| *v);
        
        match (cpu, memory) {
            (Some(c), Some(m)) => Some(ResourceUsage {
                cpu_micros: c,
                memory_bytes: m,
            }),
            _ => None,
        }
    }
    
    /// Get aggregated stats per agent
    pub fn agent_stats(&self) -> Vec<(String, AgentResourceStats)> {
        let mut stats: std::collections::HashMap<String, AgentResourceStats> = 
            std::collections::HashMap::new();
        
        for entry in self.active.iter() {
            let ctx = entry.value();
            let stat = stats.entry(ctx.agent_name.clone())
                .or_insert_with(AgentResourceStats::default);
            stat.active_invocations += 1;
        }
        
        stats.into_iter().collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceUsage {
    pub cpu_micros: u64,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AgentResourceStats {
    pub active_invocations: usize,
    pub total_cpu_micros: u64,
    pub total_memory_bytes: u64,
    pub avg_cpu_micros: u64,
    pub avg_memory_bytes: u64,
}
```

---

### 5. Profiling Exporters

**File**: `crates/rak-telemetry/src/profiling/exporters.rs` (NEW)

```rust
use pprof::protos::Message;
use reqwest::Client;
use std::time::Duration;

/// Export profiles to various backends
pub trait ProfileExporter: Send + Sync {
    fn export_cpu_profile(&self, profile: &[u8]) -> Result<()>;
    fn export_memory_profile(&self, profile: &[u8]) -> Result<()>;
}

/// Pyroscope exporter (continuous profiling platform)
pub struct PyroscopeExporter {
    client: Client,
    endpoint: String,
    application_name: String,
}

impl PyroscopeExporter {
    pub fn new(config: PyroscopeConfig) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()?,
            endpoint: config.endpoint,
            application_name: config.application_name,
        })
    }
}

#[async_trait::async_trait]
impl ProfileExporter for PyroscopeExporter {
    fn export_cpu_profile(&self, profile: &[u8]) -> Result<()> {
        // Upload to Pyroscope
        let url = format!("{}/ingest", self.endpoint);
        
        tokio::spawn({
            let client = self.client.clone();
            let url = url.clone();
            let app_name = self.application_name.clone();
            let profile = profile.to_vec();
            
            async move {
                let _ = client
                    .post(&url)
                    .header("Content-Type", "application/octet-stream")
                    .query(&[
                        ("name", app_name.as_str()),
                        ("sampleType", "cpu"),
                    ])
                    .body(profile)
                    .send()
                    .await;
            }
        });
        
        Ok(())
    }
    
    fn export_memory_profile(&self, profile: &[u8]) -> Result<()> {
        // Similar to CPU but with sampleType=alloc_space
        Ok(())
    }
}

/// GCP Cloud Profiler exporter
pub struct GcpProfilerExporter {
    // Use GCP Cloud Profiler API
    project_id: String,
    service_name: String,
    // ... GCP client
}

/// File exporter (for local development)
pub struct FileExporter {
    output_dir: std::path::PathBuf,
}

impl FileExporter {
    pub fn new(output_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
        }
    }
}

impl ProfileExporter for FileExporter {
    fn export_cpu_profile(&self, profile: &[u8]) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let path = self.output_dir.join(format!("cpu_profile_{}.pb.gz", timestamp));
        
        std::fs::create_dir_all(&self.output_dir)?;
        std::fs::write(path, profile)?;
        
        Ok(())
    }
    
    fn export_memory_profile(&self, profile: &[u8]) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let path = self.output_dir.join(format!("mem_profile_{}.pb.gz", timestamp));
        
        std::fs::create_dir_all(&self.output_dir)?;
        std::fs::write(path, profile)?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PyroscopeConfig {
    pub endpoint: String,
    pub application_name: String,
}
```

---

### 6. Configuration Updates

**Update**: `crates/rak-core/src/config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub logging: bool,
    pub tracing: TracingConfig,
    pub metrics: MetricsConfig,
    
    /// NEW: Profiling configuration
    #[serde(default)]
    pub profiling: ProfilingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfilingConfig {
    /// Enable profiling
    #[serde(default)]
    pub enabled: bool,
    
    /// CPU profiling
    #[serde(default)]
    pub cpu: CpuProfilingConfig,
    
    /// Memory profiling
    #[serde(default)]
    pub memory: MemoryProfilingConfig,
    
    /// Exporter configuration
    #[serde(default)]
    pub exporter: ProfileExporterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileExporterConfig {
    /// Type: "pyroscope", "gcp", "file"
    #[serde(default = "default_file")]
    pub type_: String,
    
    /// Pyroscope endpoint
    pub pyroscope_endpoint: Option<String>,
    
    /// GCP project ID
    pub gcp_project_id: Option<String>,
    
    /// File output directory
    pub file_output_dir: Option<String>,
}

impl Default for ProfileExporterConfig {
    fn default() -> Self {
        Self {
            type_: "file".to_string(),
            pyroscope_endpoint: None,
            gcp_project_id: None,
            file_output_dir: Some("./profiles".to_string()),
        }
    }
}
```

**Example `config.toml`**:

```toml
[telemetry]
enabled = true

[telemetry.profiling]
enabled = true

[telemetry.profiling.cpu]
frequency = 100  # Hz
continuous = true
duration_seconds = 30

[telemetry.profiling.memory]
track_allocations = false
sampling_rate = 100  # Sample 1%
heap_profiling = false

[telemetry.profiling.exporter]
type = "pyroscope"
pyroscope_endpoint = "http://localhost:4040"

# For GCP:
# type = "gcp"
# gcp_project_id = "my-project"

# For local files:
# type = "file"
# file_output_dir = "./profiles"
```

---

## 🔧 Integration Points

### 1. Runner Integration

**Update**: `crates/rak-runner/src/runner.rs`

```rust
use rak_telemetry::profiling::{CpuProfiler, MemoryProfiler, ResourceAttribution};

impl Runner {
    pub async fn run(
        &self,
        user_id: String,
        session_id: String,
        message: Content,
        config: RunConfig,
    ) -> Result<Box<dyn Stream<Item = Result<Event>> + Send + Unpin>> {
        let invocation_id = Uuid::new_v4().to_string();
        
        // Start resource tracking
        if let Some(profiler) = self.profiler.as_ref() {
            profiler.attribution.start_invocation(ResourceContext {
                invocation_id: invocation_id.clone(),
                session_id: session_id.clone(),
                agent_name: self.agent.name().to_string(),
                start_time: Instant::now(),
                start_memory: profiler.memory.stats().current_usage,
            });
        }
        
        // ... run agent
        
        // On completion
        if let Some(profiler) = self.profiler.as_ref() {
            let end_memory = profiler.memory.stats().current_usage;
            profiler.attribution.end_invocation(&invocation_id, end_memory);
        }
        
        Ok(stream)
    }
}
```

### 2. Periodic Profile Collection

**File**: `crates/rak-telemetry/src/profiling/collector.rs` (NEW)

```rust
use tokio::time::{interval, Duration};

/// Background profiler that periodically collects and exports profiles
pub struct ProfileCollector {
    cpu_profiler: Arc<Mutex<CpuProfiler>>,
    memory_profiler: Arc<MemoryProfiler>,
    exporter: Arc<dyn ProfileExporter>,
    interval: Duration,
}

impl ProfileCollector {
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = interval(self.interval);
            
            loop {
                ticker.tick().await;
                
                // Collect CPU profile
                if let Ok(profile) = self.cpu_profiler.lock().unwrap().pprof() {
                    let _ = self.exporter.export_cpu_profile(&profile);
                }
                
                // Collect memory stats
                let mem_stats = self.memory_profiler.stats();
                tracing::info!(
                    current_bytes = mem_stats.current_usage,
                    peak_bytes = mem_stats.peak_usage,
                    allocations = mem_stats.allocation_count,
                    "Memory statistics"
                );
            }
        })
    }
}
```

---

## 📊 Span Attributes for Profiling

Add resource usage to existing spans:

```rust
// In trace_agent_start
pub struct AgentSpanAttributes {
    // ... existing fields
    
    /// NEW: CPU time in microseconds
    pub cpu_micros: Option<u64>,
    
    /// NEW: Memory allocated in bytes
    pub memory_bytes: Option<u64>,
    
    /// NEW: Peak memory in bytes
    pub peak_memory_bytes: Option<u64>,
}

// When completing a span:
span.record("resource.cpu.micros", cpu_micros);
span.record("resource.memory.bytes", memory_bytes);
span.record("resource.memory.peak", peak_memory_bytes);
```

---

## 🔗 Correlating Traces with Profiles

**Key Technique**: Include `trace_id` and `span_id` in profile samples.

```rust
// In profile collection:
let trace_id = opentelemetry::trace::TraceId::from_hex(&invocation_id)?;
let span_id = opentelemetry::trace::SpanId::from_hex(&event_id)?;

// Add to pprof profile labels
profile.add_label("trace_id", trace_id.to_string());
profile.add_label("span_id", span_id.to_string());
profile.add_label("agent_name", agent_name);
```

**Visualization**: In observability UI, click on a slow span → view profile for that time range.

---

## 📈 Dependencies

```toml
[dependencies]
# Existing...

# NEW for profiling
pprof = { version = "0.13", features = ["flamegraph", "protobuf-codec"] }
dashmap = "5.5"
tokio = { version = "1.35", features = ["time"] }

# Platform-specific
[target.'cfg(target_os = "linux")'.dependencies]
procfs = "0.16"

[target.'cfg(target_os = "macos")'.dependencies]
libc = "0.2"
```

---

## 🎯 Key Metrics to Track

### CPU Metrics
- `cpu.profile.samples` - Total CPU samples collected
- `cpu.time.micros` - CPU time per invocation
- `cpu.hotspots` - Top functions by CPU time

### Memory Metrics
- `memory.allocated.bytes` - Total allocations
- `memory.current.bytes` - Current usage
- `memory.peak.bytes` - Peak usage
- `memory.allocations.count` - Allocation count
- `memory.rss.bytes` - Resident set size

### Resource Attribution
- `resource.cpu_per_agent` - CPU by agent type
- `resource.memory_per_session` - Memory by session
- `resource.invocation_overhead` - Runner overhead

---

## 🚀 Usage Example

```rust
use rak_telemetry::profiling::{CpuProfiler, MemoryProfiler, ProfileCollector};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize telemetry with profiling
    let config = RakConfig::from_file("config.toml")?;
    
    let cpu_profiler = CpuProfiler::new(config.telemetry.profiling.cpu)?;
    let memory_profiler = MemoryProfiler::new(config.telemetry.profiling.memory);
    
    // Start continuous profiling
    let collector = ProfileCollector {
        cpu_profiler: Arc::new(Mutex::new(cpu_profiler)),
        memory_profiler: Arc::new(memory_profiler.clone()),
        exporter: Arc::new(PyroscopeExporter::new(/* ... */)?),
        interval: Duration::from_secs(30),
    };
    
    let _collector_handle = collector.start();
    
    // Run your agent application
    let runner = Runner::builder()
        .app_name("my-app")
        .agent(agent)
        .session_service(session_service)
        .profiler(Some(ProfilerConfig {
            cpu: cpu_profiler,
            memory: memory_profiler,
        }))
        .build()?;
    
    // ... run agents
    
    Ok(())
}
```

---

## 📊 Visualization Options

### 1. **Flamegraphs** (CPU)
- Shows call stacks and time spent
- Identify hot functions
- Export from pprof directly

### 2. **Timeline View** (Memory)
- Memory usage over time
- Allocation/deallocation patterns
- Leak detection

### 3. **Differential Profiles**
- Compare before/after optimization
- Compare different agents
- Identify regressions

### 4. **Integrated Trace + Profile View**
- Click on slow span in Jaeger/Tempo
- Jump to profile for that time period
- See exact code path with resource costs

---

## ⚠️ Performance Considerations

### CPU Profiling
- **Overhead**: ~1-3% at 100 Hz sampling
- **Recommendation**: 
  - Development: 100 Hz (10ms intervals)
  - Production: 50 Hz (20ms intervals)

### Memory Profiling
- **Overhead**: Depends on tracking level
  - Basic stats: < 1%
  - Sampled allocations: 2-5%
  - Full heap profiling: 10-20% ⚠️
- **Recommendation**: 
  - Production: Basic stats + 1% sampling
  - Debug: Full tracking for specific sessions

### Best Practices
1. Use sampling to reduce overhead
2. Profile specific invocations (not all traffic)
3. Export profiles asynchronously
4. Set maximum profile size limits
5. Implement back-pressure if export lags

---

## 🎯 Updated Implementation Plan

### Phase 9.7: CPU Profiling (NEW)
- Integrate `pprof` crate
- Implement CpuProfiler with start/stop
- Add flamegraph generation
- Export to Pyroscope/GCP
- **Duration**: 6 hours

### Phase 9.8: Memory Profiling (NEW)
- Implement MemoryProfiler with stats tracking
- Add platform-specific memory info (Linux/macOS)
- Track allocations with sampling
- Add heap profiling (optional)
- **Duration**: 8 hours

### Phase 9.9: Resource Attribution (NEW)
- Track CPU/memory per invocation
- Aggregate by agent/session
- Add resource metrics to spans
- Correlate with traces
- **Duration**: 6 hours

### Phase 9.10: Profile Exporters (NEW)
- Implement Pyroscope exporter
- Implement GCP Cloud Profiler exporter
- Implement file exporter
- Add periodic profile collection
- **Duration**: 8 hours

### Phase 9.11: Integration & Testing (NEW)
- Integrate with Runner
- Add configuration support
- Test with real workloads
- Validate overhead < 5%
- **Duration**: 6 hours

**Total Additional Time**: ~34 hours

---

## 📝 Summary of Changes

### New Files
- `crates/rak-telemetry/src/profiling/mod.rs`
- `crates/rak-telemetry/src/profiling/cpu.rs`
- `crates/rak-telemetry/src/profiling/memory.rs`
- `crates/rak-telemetry/src/profiling/attribution.rs`
- `crates/rak-telemetry/src/profiling/exporters.rs`
- `crates/rak-telemetry/src/profiling/collector.rs`

### Modified Files
- `crates/rak-core/src/config.rs` - Add profiling config
- `crates/rak-runner/src/runner.rs` - Add profiler integration
- `crates/rak-telemetry/src/spans.rs` - Add resource attributes
- `crates/rak-telemetry/Cargo.toml` - Add dependencies

### New Dependencies
- `pprof` - CPU profiling
- `dashmap` - Concurrent maps for attribution
- `procfs` - Linux memory info
- `libc` - macOS memory info

---

## ✅ Success Criteria

Phase 9 with profiling is complete when:

1. ✅ CPU profiles collected and exported
2. ✅ Memory usage tracked and reported
3. ✅ Resource usage attributed to invocations
4. ✅ Profiles correlated with traces via trace_id
5. ✅ < 5% performance overhead with default config
6. ✅ Flamegraphs generated and viewable
7. ✅ Integration with at least one backend (Pyroscope or GCP)
8. ✅ Configuration flexible and documented

---

**With profiling, you'll have complete visibility**: traces show execution flow, metrics show aggregates, and **profiles show where resources go**! 🚀

