# Telemetry Ingestion Service - ClickHouse Backend

**Date**: 2025-11-23 13:00  
**Status**: 🎯 Design Phase  
**Type**: New Binary (`rak-ingest`)

---

## 🎯 Objectives

Build a **standalone telemetry ingestion service** that:
- Receives traces, metrics, and profiles from RAK agents
- Batches data for efficient storage
- Stores all telemetry in ClickHouse (columnar database)
- Provides query APIs for visualization
- Scales horizontally for high throughput

**Key Principle**: This is the **server-side** complement to the client-side `rak-telemetry` crate.

---

## 🏗️ Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    RAK Agents (Clients)                       │
│                                                               │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │ Agent 1 │  │ Agent 2 │  │ Agent 3 │  │ Agent N │        │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘        │
└───────┼────────────┼────────────┼────────────┼──────────────┘
        │            │            │            │
        │  OTLP gRPC / HTTP / pprof Protocol  │
        │            │            │            │
        └────────────┴────────────┴────────────┘
                     │
        ┌────────────▼────────────┐
        │   Load Balancer         │
        │   (HAProxy / Nginx)     │
        └────────────┬────────────┘
                     │
        ┌────────────▼────────────────────────────────┐
        │     rak-ingest Service Cluster              │
        │                                              │
        │  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
        │  │ Instance │  │ Instance │  │ Instance │ │
        │  │    1     │  │    2     │  │    N     │ │
        │  └────┬─────┘  └────┬─────┘  └────┬─────┘ │
        │       │             │             │        │
        │  ┌────▼─────────────▼─────────────▼─────┐ │
        │  │    In-Memory Batch Buffers          │ │
        │  │    (Traces / Metrics / Profiles)    │ │
        │  └────┬─────────────┬─────────────┬────┘ │
        └───────┼─────────────┼─────────────┼──────┘
                │             │             │
        ┌───────▼─────────────▼─────────────▼──────┐
        │         ClickHouse Cluster                │
        │                                            │
        │  ┌────────┐  ┌────────┐  ┌────────┐     │
        │  │ Shard1 │  │ Shard2 │  │ ShardN │     │
        │  │        │  │        │  │        │     │
        │  │ Tables:│  │ Tables:│  │ Tables:│     │
        │  │ - spans│  │ - spans│  │ - spans│     │
        │  │ -metrics│ │ -metrics│ │ -metrics│    │
        │  │ -profiles│ │ -profiles│ │ -profiles│  │
        │  └────────┘  └────────┘  └────────┘     │
        └────────────────────────────────────────────┘
                     │
        ┌────────────▼────────────┐
        │   Query API              │
        │   (HTTP REST / gRPC)     │
        └────────────┬────────────┘
                     │
        ┌────────────▼────────────┐
        │   Visualization Layer    │
        │   (Grafana / Custom UI)  │
        └──────────────────────────┘
```

---

## 📦 New Binary: `rak-ingest`

### Project Structure

```
rak/
├── crates/
│   ├── rak-ingest/              # NEW: Ingestion service
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs          # Binary entry point
│   │   │   ├── lib.rs           # Library code
│   │   │   ├── server.rs        # HTTP/gRPC server
│   │   │   ├── otlp/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── traces.rs   # OTLP trace ingestion
│   │   │   │   ├── metrics.rs  # OTLP metrics ingestion
│   │   │   │   └── logs.rs     # OTLP logs (future)
│   │   │   ├── pprof/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── ingest.rs   # pprof profile ingestion
│   │   │   │   └── decode.rs   # pprof protobuf decoding
│   │   │   ├── batch/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── buffer.rs   # In-memory batch buffer
│   │   │   │   ├── flusher.rs  # Periodic flush to ClickHouse
│   │   │   │   └── strategy.rs # Batching strategies
│   │   │   ├── storage/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── clickhouse.rs  # ClickHouse client
│   │   │   │   ├── schema.rs      # Table schemas
│   │   │   │   └── queries.rs     # Query builders
│   │   │   ├── query/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── traces.rs   # Trace query API
│   │   │   │   ├── metrics.rs  # Metrics query API
│   │   │   │   └── profiles.rs # Profile query API
│   │   │   └── config.rs        # Configuration
│   │   └── Dockerfile           # Container image
│   └── ... (existing crates)
```

---

## 🗄️ ClickHouse Schema Design

### 1. Spans Table (Distributed Tracing)

```sql
CREATE TABLE spans_local ON CLUSTER '{cluster}' (
    -- Identity
    trace_id FixedString(32),           -- Hex-encoded trace ID
    span_id FixedString(16),            -- Hex-encoded span ID
    parent_span_id FixedString(16),     -- Parent span ID
    
    -- Timing
    timestamp DateTime64(9, 'UTC'),     -- Span start time (nanosecond precision)
    duration_ns UInt64,                 -- Duration in nanoseconds
    
    -- Metadata
    service_name LowCardinality(String), -- Service name (e.g., "rak-rust-app")
    span_name LowCardinality(String),    -- Span name (e.g., "agent.run")
    span_kind Enum8(                     -- Span kind
        'INTERNAL' = 1,
        'SERVER' = 2,
        'CLIENT' = 3,
        'PRODUCER' = 4,
        'CONSUMER' = 5
    ),
    
    -- Status
    status_code Enum8(                   -- Span status
        'UNSET' = 0,
        'OK' = 1,
        'ERROR' = 2
    ),
    status_message String,               -- Error message if any
    
    -- Attributes (indexed for fast queries)
    invocation_id String,                -- RAK invocation ID
    session_id String,                   -- RAK session ID
    user_id String,                      -- User ID
    agent_name LowCardinality(String),   -- Agent name
    agent_type LowCardinality(String),   -- Agent type (llm, workflow, etc.)
    
    -- LLM-specific attributes
    llm_model LowCardinality(String),    -- Model name
    llm_system LowCardinality(String),   -- gen_ai.system
    
    -- Tool-specific attributes
    tool_name LowCardinality(String),    -- Tool name
    tool_call_id String,                 -- Tool call ID
    
    -- Resource attributes (CPU/Memory from profiling)
    resource_cpu_micros UInt64,          -- CPU time in microseconds
    resource_memory_bytes UInt64,        -- Memory allocated in bytes
    resource_memory_peak UInt64,         -- Peak memory in bytes
    
    -- All attributes as JSON (for flexibility)
    attributes String,                   -- JSON string of all attributes
    
    -- Events (nested structure)
    events Nested (
        timestamp DateTime64(9, 'UTC'),
        name String,
        attributes String
    ),
    
    -- Links to other spans
    links Nested (
        trace_id FixedString(32),
        span_id FixedString(16),
        attributes String
    ),
    
    -- Metadata
    ingested_at DateTime DEFAULT now(),  -- When ingested
    
    -- Primary key for efficient queries
    INDEX idx_invocation_id invocation_id TYPE bloom_filter GRANULARITY 1,
    INDEX idx_session_id session_id TYPE bloom_filter GRANULARITY 1,
    INDEX idx_agent_name agent_name TYPE set(100) GRANULARITY 1,
    INDEX idx_status_code status_code TYPE set(10) GRANULARITY 1
)
ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/spans', '{replica}')
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (service_name, timestamp, trace_id, span_id)
TTL timestamp + INTERVAL 30 DAY;  -- Retention: 30 days

-- Distributed table (query this for multi-shard)
CREATE TABLE spans ON CLUSTER '{cluster}' AS spans_local
ENGINE = Distributed('{cluster}', default, spans_local, rand());
```

---

### 2. Metrics Table

```sql
CREATE TABLE metrics_local ON CLUSTER '{cluster}' (
    -- Identity
    metric_name LowCardinality(String),  -- Metric name (e.g., "agent.duration")
    metric_type Enum8(                   -- Metric type
        'COUNTER' = 1,
        'GAUGE' = 2,
        'HISTOGRAM' = 3,
        'SUMMARY' = 4
    ),
    
    -- Timing
    timestamp DateTime64(9, 'UTC'),      -- Measurement time
    
    -- Value
    value Float64,                       -- Metric value
    count UInt64,                        -- Sample count (for histograms)
    sum Float64,                         -- Sum (for histograms)
    min Float64,                         -- Min value (for histograms)
    max Float64,                         -- Max value (for histograms)
    
    -- Histogram buckets
    buckets Nested (
        upper_bound Float64,
        count UInt64
    ),
    
    -- Labels (dimensions)
    service_name LowCardinality(String),
    agent_name LowCardinality(String),
    user_id String,
    session_id String,
    
    -- Additional labels as key-value
    labels Map(String, String),
    
    -- Metadata
    ingested_at DateTime DEFAULT now(),
    
    INDEX idx_metric_name metric_name TYPE set(1000) GRANULARITY 1
)
ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/metrics', '{replica}')
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (metric_name, timestamp)
TTL timestamp + INTERVAL 90 DAY;  -- Longer retention for metrics

CREATE TABLE metrics ON CLUSTER '{cluster}' AS metrics_local
ENGINE = Distributed('{cluster}', default, metrics_local, rand());
```

---

### 3. CPU Profiles Table

```sql
CREATE TABLE cpu_profiles_local ON CLUSTER '{cluster}' (
    -- Identity
    profile_id String,                   -- Unique profile ID
    timestamp DateTime64(9, 'UTC'),      -- Profile collection time
    duration_seconds UInt32,             -- Profile duration
    
    -- Context
    service_name LowCardinality(String),
    invocation_id String,                -- Link to trace
    trace_id FixedString(32),            -- Link to specific trace
    span_id FixedString(16),             -- Link to specific span
    agent_name LowCardinality(String),
    
    -- Profile data (compressed)
    profile_type Enum8(                  -- Profile type
        'CPU' = 1,
        'HEAP' = 2,
        'GOROUTINE' = 3,
        'BLOCK' = 4
    ),
    sample_type String,                  -- e.g., "cpu/nanoseconds"
    
    -- Samples (flattened for efficient querying)
    samples Nested (
        locations Array(String),         -- Stack trace (function names)
        location_ids Array(UInt64),      -- Stack trace (function IDs)
        values Array(Int64),             -- Sample values
        labels Map(String, String)       -- Sample labels
    ),
    
    -- Function metadata
    functions Nested (
        id UInt64,
        name String,
        filename String,
        start_line Int64
    ),
    
    -- Raw pprof data (for full reconstruction)
    raw_pprof String CODEC(ZSTD),       -- Compressed pprof protobuf
    
    -- Metadata
    ingested_at DateTime DEFAULT now(),
    
    INDEX idx_invocation_id invocation_id TYPE bloom_filter GRANULARITY 1,
    INDEX idx_trace_id trace_id TYPE bloom_filter GRANULARITY 1
)
ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/cpu_profiles', '{replica}')
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (service_name, timestamp, profile_id)
TTL timestamp + INTERVAL 7 DAY;  -- Shorter retention due to size

CREATE TABLE cpu_profiles ON CLUSTER '{cluster}' AS cpu_profiles_local
ENGINE = Distributed('{cluster}', default, cpu_profiles_local, rand());
```

---

### 4. Memory Profiles Table

```sql
CREATE TABLE memory_profiles_local ON CLUSTER '{cluster}' (
    -- Identity
    profile_id String,
    timestamp DateTime64(9, 'UTC'),
    
    -- Context
    service_name LowCardinality(String),
    invocation_id String,
    agent_name LowCardinality(String),
    
    -- Memory statistics
    total_allocated UInt64,              -- Total bytes allocated
    current_usage UInt64,                -- Current memory usage
    peak_usage UInt64,                   -- Peak memory usage
    allocation_count UInt64,             -- Number of allocations
    
    -- System memory
    rss_bytes UInt64,                    -- Resident Set Size
    virtual_bytes UInt64,                -- Virtual memory
    
    -- Allocation samples (top allocators)
    allocations Nested (
        location String,                 -- Allocation site (function)
        size UInt64,                     -- Bytes allocated
        count UInt64                     -- Number of allocations
    ),
    
    -- Metadata
    ingested_at DateTime DEFAULT now()
)
ENGINE = ReplicatedMergeTree('/clickhouse/tables/{shard}/memory_profiles', '{replica}')
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (service_name, timestamp, profile_id)
TTL timestamp + INTERVAL 7 DAY;

CREATE TABLE memory_profiles ON CLUSTER '{cluster}' AS memory_profiles_local
ENGINE = Distributed('{cluster}', default, memory_profiles_local, rand());
```

---

### 5. Materialized Views (Pre-aggregated Queries)

```sql
-- Agent execution summary (per hour)
CREATE MATERIALIZED VIEW agent_execution_summary_mv
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMMDD(hour)
ORDER BY (hour, service_name, agent_name)
AS SELECT
    toStartOfHour(timestamp) AS hour,
    service_name,
    agent_name,
    count() AS execution_count,
    avg(duration_ns) / 1e9 AS avg_duration_seconds,
    quantile(0.5)(duration_ns) / 1e9 AS p50_duration,
    quantile(0.95)(duration_ns) / 1e9 AS p95_duration,
    quantile(0.99)(duration_ns) / 1e9 AS p99_duration,
    countIf(status_code = 'ERROR') AS error_count
FROM spans_local
WHERE span_name = 'agent.run'
GROUP BY hour, service_name, agent_name;

-- LLM call statistics
CREATE MATERIALIZED VIEW llm_call_summary_mv
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMMDD(hour)
ORDER BY (hour, service_name, llm_model)
AS SELECT
    toStartOfHour(timestamp) AS hour,
    service_name,
    llm_model,
    count() AS call_count,
    avg(duration_ns) / 1e9 AS avg_duration_seconds,
    sum(resource_cpu_micros) / 1e6 AS total_cpu_seconds,
    sum(resource_memory_bytes) / 1024 / 1024 AS total_memory_mb
FROM spans_local
WHERE span_name = 'llm.generate'
GROUP BY hour, service_name, llm_model;

-- Tool execution statistics
CREATE MATERIALIZED VIEW tool_execution_summary_mv
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMMDD(hour)
ORDER BY (hour, service_name, tool_name)
AS SELECT
    toStartOfHour(timestamp) AS hour,
    service_name,
    tool_name,
    count() AS execution_count,
    avg(duration_ns) / 1e9 AS avg_duration_seconds,
    countIf(status_code = 'ERROR') AS error_count
FROM spans_local
WHERE span_name = 'execute_tool' AND tool_name != ''
GROUP BY hour, service_name, tool_name;
```

---

## 🔧 Implementation: `rak-ingest` Service

### 1. Main Entry Point

**File**: `crates/rak-ingest/src/main.rs`

```rust
use anyhow::Result;
use rak_ingest::{IngestConfig, IngestServer};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();
    
    // Load configuration
    let config = IngestConfig::from_file("config.toml")
        .or_else(|_| IngestConfig::from_env())?;
    
    info!("Starting rak-ingest service");
    info!("ClickHouse: {}", config.clickhouse.url);
    info!("OTLP gRPC: 0.0.0.0:{}", config.otlp_grpc_port);
    info!("OTLP HTTP: 0.0.0.0:{}", config.otlp_http_port);
    info!("pprof HTTP: 0.0.0.0:{}", config.pprof_port);
    
    // Create and start server
    let server = IngestServer::new(config).await?;
    
    // Graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Shutdown signal received");
    };
    
    // Run server
    tokio::select! {
        result = server.serve() => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = shutdown_signal => {
            info!("Shutting down gracefully");
        }
    }
    
    Ok(())
}
```

---

### 2. Server Implementation

**File**: `crates/rak-ingest/src/server.rs`

```rust
use crate::{
    IngestConfig,
    otlp::OtlpHandler,
    pprof::PprofHandler,
    batch::BatchBuffer,
    storage::ClickHouseStorage,
};
use anyhow::Result;
use axum::{Router, routing::post};
use std::sync::Arc;
use tokio::net::TcpListener;
use tonic::transport::Server as TonicServer;

pub struct IngestServer {
    config: IngestConfig,
    storage: Arc<ClickHouseStorage>,
    batch_buffer: Arc<BatchBuffer>,
}

impl IngestServer {
    pub async fn new(config: IngestConfig) -> Result<Self> {
        // Initialize ClickHouse storage
        let storage = Arc::new(ClickHouseStorage::new(&config.clickhouse).await?);
        
        // Initialize batch buffer
        let batch_buffer = Arc::new(BatchBuffer::new(
            config.batch_size,
            config.batch_timeout,
            storage.clone(),
        ));
        
        // Start background flush task
        batch_buffer.start_flusher().await;
        
        Ok(Self {
            config,
            storage,
            batch_buffer,
        })
    }
    
    pub async fn serve(self) -> Result<()> {
        let batch_buffer = self.batch_buffer.clone();
        
        // Start OTLP gRPC server
        let otlp_grpc = self.serve_otlp_grpc(batch_buffer.clone());
        
        // Start OTLP HTTP server
        let otlp_http = self.serve_otlp_http(batch_buffer.clone());
        
        // Start pprof HTTP server
        let pprof = self.serve_pprof(batch_buffer.clone());
        
        // Start query API
        let query_api = self.serve_query_api();
        
        // Run all servers concurrently
        tokio::try_join!(otlp_grpc, otlp_http, pprof, query_api)?;
        
        Ok(())
    }
    
    async fn serve_otlp_grpc(&self, batch_buffer: Arc<BatchBuffer>) -> Result<()> {
        use opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::TraceServiceServer;
        use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::MetricsServiceServer;
        
        let otlp_handler = OtlpHandler::new(batch_buffer);
        let addr = format!("0.0.0.0:{}", self.config.otlp_grpc_port).parse()?;
        
        tracing::info!("Starting OTLP gRPC server on {}", addr);
        
        TonicServer::builder()
            .add_service(TraceServiceServer::new(otlp_handler.clone()))
            .add_service(MetricsServiceServer::new(otlp_handler))
            .serve(addr)
            .await?;
        
        Ok(())
    }
    
    async fn serve_otlp_http(&self, batch_buffer: Arc<BatchBuffer>) -> Result<()> {
        let otlp_handler = OtlpHandler::new(batch_buffer);
        
        let app = Router::new()
            .route("/v1/traces", post({
                let handler = otlp_handler.clone();
                move |body| handler.handle_http_traces(body)
            }))
            .route("/v1/metrics", post({
                let handler = otlp_handler;
                move |body| handler.handle_http_metrics(body)
            }));
        
        let addr = format!("0.0.0.0:{}", self.config.otlp_http_port);
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("Starting OTLP HTTP server on {}", addr);
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    async fn serve_pprof(&self, batch_buffer: Arc<BatchBuffer>) -> Result<()> {
        let pprof_handler = PprofHandler::new(batch_buffer);
        
        let app = Router::new()
            .route("/pprof/profile", post({
                let handler = pprof_handler.clone();
                move |body| handler.handle_cpu_profile(body)
            }))
            .route("/pprof/heap", post({
                let handler = pprof_handler;
                move |body| handler.handle_memory_profile(body)
            }));
        
        let addr = format!("0.0.0.0:{}", self.config.pprof_port);
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("Starting pprof HTTP server on {}", addr);
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    async fn serve_query_api(&self) -> Result<()> {
        use crate::query::{TraceQueryHandler, MetricsQueryHandler};
        
        let trace_handler = TraceQueryHandler::new(self.storage.clone());
        let metrics_handler = MetricsQueryHandler::new(self.storage.clone());
        
        let app = Router::new()
            .route("/api/v1/traces", axum::routing::get({
                let handler = trace_handler.clone();
                move |query| handler.query_traces(query)
            }))
            .route("/api/v1/traces/:trace_id", axum::routing::get({
                let handler = trace_handler;
                move |path| handler.get_trace(path)
            }))
            .route("/api/v1/metrics", axum::routing::get({
                let handler = metrics_handler;
                move |query| handler.query_metrics(query)
            }));
        
        let addr = format!("0.0.0.0:{}", self.config.query_api_port);
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("Starting Query API on {}", addr);
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}
```

---

### 3. Batch Buffer Implementation

**File**: `crates/rak-ingest/src/batch/buffer.rs`

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use crate::storage::ClickHouseStorage;

pub struct BatchBuffer {
    traces: Arc<RwLock<Vec<Span>>>,
    metrics: Arc<RwLock<Vec<Metric>>>,
    profiles: Arc<RwLock<Vec<Profile>>>,
    
    max_batch_size: usize,
    flush_interval: Duration,
    storage: Arc<ClickHouseStorage>,
}

#[derive(Clone)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub timestamp: i64,
    pub duration_ns: u64,
    pub service_name: String,
    pub span_name: String,
    pub attributes: serde_json::Value,
    // ... other fields
}

#[derive(Clone)]
pub struct Metric {
    pub name: String,
    pub metric_type: String,
    pub timestamp: i64,
    pub value: f64,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Clone)]
pub struct Profile {
    pub profile_id: String,
    pub timestamp: i64,
    pub profile_type: String,
    pub raw_pprof: Vec<u8>,
    // ... other fields
}

impl BatchBuffer {
    pub fn new(
        max_batch_size: usize,
        flush_interval: Duration,
        storage: Arc<ClickHouseStorage>,
    ) -> Self {
        Self {
            traces: Arc::new(RwLock::new(Vec::with_capacity(max_batch_size))),
            metrics: Arc::new(RwLock::new(Vec::with_capacity(max_batch_size))),
            profiles: Arc::new(RwLock::new(Vec::with_capacity(max_batch_size))),
            max_batch_size,
            flush_interval,
            storage,
        }
    }
    
    pub async fn start_flusher(&self) {
        let traces = self.traces.clone();
        let metrics = self.metrics.clone();
        let profiles = self.profiles.clone();
        let storage = self.storage.clone();
        let flush_interval = self.flush_interval;
        let max_batch_size = self.max_batch_size;
        
        tokio::spawn(async move {
            let mut ticker = interval(flush_interval);
            
            loop {
                ticker.tick().await;
                
                // Flush traces
                let mut trace_buffer = traces.write().await;
                if !trace_buffer.is_empty() {
                    let batch = trace_buffer.drain(..).collect::<Vec<_>>();
                    drop(trace_buffer); // Release lock
                    
                    if let Err(e) = storage.insert_spans(&batch).await {
                        tracing::error!("Failed to insert spans: {}", e);
                    } else {
                        tracing::debug!("Flushed {} spans", batch.len());
                    }
                }
                
                // Flush metrics
                let mut metric_buffer = metrics.write().await;
                if !metric_buffer.is_empty() {
                    let batch = metric_buffer.drain(..).collect::<Vec<_>>();
                    drop(metric_buffer);
                    
                    if let Err(e) = storage.insert_metrics(&batch).await {
                        tracing::error!("Failed to insert metrics: {}", e);
                    } else {
                        tracing::debug!("Flushed {} metrics", batch.len());
                    }
                }
                
                // Flush profiles
                let mut profile_buffer = profiles.write().await;
                if !profile_buffer.is_empty() {
                    let batch = profile_buffer.drain(..).collect::<Vec<_>>();
                    drop(profile_buffer);
                    
                    if let Err(e) = storage.insert_profiles(&batch).await {
                        tracing::error!("Failed to insert profiles: {}", e);
                    } else {
                        tracing::debug!("Flushed {} profiles", batch.len());
                    }
                }
            }
        });
    }
    
    pub async fn add_span(&self, span: Span) {
        let mut buffer = self.traces.write().await;
        buffer.push(span);
        
        // Force flush if batch is full
        if buffer.len() >= self.max_batch_size {
            let batch = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer); // Release lock before insert
            
            if let Err(e) = self.storage.insert_spans(&batch).await {
                tracing::error!("Failed to insert spans: {}", e);
            }
        }
    }
    
    pub async fn add_metric(&self, metric: Metric) {
        let mut buffer = self.metrics.write().await;
        buffer.push(metric);
        
        if buffer.len() >= self.max_batch_size {
            let batch = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer);
            
            if let Err(e) = self.storage.insert_metrics(&batch).await {
                tracing::error!("Failed to insert metrics: {}", e);
            }
        }
    }
    
    pub async fn add_profile(&self, profile: Profile) {
        let mut buffer = self.profiles.write().await;
        buffer.push(profile);
        
        if buffer.len() >= self.max_batch_size {
            let batch = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer);
            
            if let Err(e) = self.storage.insert_profiles(&batch).await {
                tracing::error!("Failed to insert profiles: {}", e);
            }
        }
    }
}
```

---

### 4. ClickHouse Storage Implementation

**File**: `crates/rak-ingest/src/storage/clickhouse.rs`

```rust
use clickhouse::{Client, Row};
use anyhow::Result;
use crate::batch::{Span, Metric, Profile};

pub struct ClickHouseStorage {
    client: Client,
}

impl ClickHouseStorage {
    pub async fn new(config: &ClickHouseConfig) -> Result<Self> {
        let client = Client::default()
            .with_url(&config.url)
            .with_user(&config.user)
            .with_password(&config.password)
            .with_database(&config.database)
            .with_compression(clickhouse::Compression::Lz4);
        
        Ok(Self { client })
    }
    
    pub async fn insert_spans(&self, spans: &[Span]) -> Result<()> {
        if spans.is_empty() {
            return Ok(());
        }
        
        let mut insert = self.client.insert("spans")?;
        
        for span in spans {
            insert.write(&SpanRow::from(span)).await?;
        }
        
        insert.end().await?;
        
        tracing::info!("Inserted {} spans into ClickHouse", spans.len());
        Ok(())
    }
    
    pub async fn insert_metrics(&self, metrics: &[Metric]) -> Result<()> {
        if metrics.is_empty() {
            return Ok(());
        }
        
        let mut insert = self.client.insert("metrics")?;
        
        for metric in metrics {
            insert.write(&MetricRow::from(metric)).await?;
        }
        
        insert.end().await?;
        
        tracing::info!("Inserted {} metrics into ClickHouse", metrics.len());
        Ok(())
    }
    
    pub async fn insert_profiles(&self, profiles: &[Profile]) -> Result<()> {
        if profiles.is_empty() {
            return Ok(());
        }
        
        let mut insert = self.client.insert("cpu_profiles")?;
        
        for profile in profiles {
            insert.write(&ProfileRow::from(profile)).await?;
        }
        
        insert.end().await?;
        
        tracing::info!("Inserted {} profiles into ClickHouse", profiles.len());
        Ok(())
    }
    
    pub async fn query_traces(
        &self,
        start_time: i64,
        end_time: i64,
        service_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Span>> {
        let mut query = "SELECT * FROM spans WHERE timestamp BETWEEN ? AND ?".to_string();
        
        if let Some(svc) = service_name {
            query.push_str(" AND service_name = ?");
        }
        
        query.push_str(" ORDER BY timestamp DESC LIMIT ?");
        
        let rows = self.client
            .query(&query)
            .bind(start_time)
            .bind(end_time);
        
        let rows = if let Some(svc) = service_name {
            rows.bind(svc)
        } else {
            rows
        };
        
        let rows = rows.bind(limit as u64).fetch_all::<SpanRow>().await?;
        
        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(Row, Serialize, Deserialize)]
struct SpanRow {
    trace_id: String,
    span_id: String,
    parent_span_id: String,
    timestamp: i64,
    duration_ns: u64,
    service_name: String,
    span_name: String,
    // ... all other fields from schema
}

impl From<&Span> for SpanRow {
    fn from(span: &Span) -> Self {
        SpanRow {
            trace_id: span.trace_id.clone(),
            span_id: span.span_id.clone(),
            parent_span_id: span.parent_span_id.clone().unwrap_or_default(),
            timestamp: span.timestamp,
            duration_ns: span.duration_ns,
            service_name: span.service_name.clone(),
            span_name: span.span_name.clone(),
            // ...
        }
    }
}
```

---

## ⚙️ Configuration

**File**: `config.toml` for `rak-ingest`

```toml
[server]
# Server ports
otlp_grpc_port = 4317
otlp_http_port = 4318
pprof_port = 4319
query_api_port = 8080

# Batch configuration
batch_size = 1000                # Max items per batch
batch_timeout_seconds = 10       # Max time before flush

[clickhouse]
url = "http://localhost:8123"
user = "default"
password = ""
database = "telemetry"

# Connection pool
max_connections = 10
connection_timeout_seconds = 5

[storage]
# Retention policies (in days)
spans_retention_days = 30
metrics_retention_days = 90
profiles_retention_days = 7

[query]
# Query limits
max_query_time_seconds = 30
max_result_rows = 10000
```

---

## 📊 Example Queries

### 1. Find All Traces for a Session

```sql
SELECT
    trace_id,
    span_name,
    timestamp,
    duration_ns / 1e9 AS duration_seconds,
    status_code
FROM spans
WHERE session_id = 'sess-123'
ORDER BY timestamp ASC;
```

### 2. Agent Performance Summary

```sql
SELECT
    agent_name,
    count() AS execution_count,
    avg(duration_ns) / 1e9 AS avg_duration_seconds,
    quantile(0.95)(duration_ns) / 1e9 AS p95_duration,
    countIf(status_code = 'ERROR') / count() * 100 AS error_rate_percent
FROM spans
WHERE span_name = 'agent.run'
  AND timestamp >= now() - INTERVAL 1 DAY
GROUP BY agent_name
ORDER BY execution_count DESC;
```

### 3. Find Slow Traces

```sql
SELECT
    trace_id,
    service_name,
    agent_name,
    timestamp,
    duration_ns / 1e9 AS duration_seconds
FROM spans
WHERE span_name = 'agent.run'
  AND duration_ns > 5e9  -- > 5 seconds
  AND timestamp >= now() - INTERVAL 1 HOUR
ORDER BY duration_ns DESC
LIMIT 10;
```

### 4. LLM Token Usage (if tracked)

```sql
SELECT
    llm_model,
    count() AS call_count,
    sum(JSONExtractInt(attributes, 'token_count')) AS total_tokens,
    avg(duration_ns) / 1e9 AS avg_latency
FROM spans
WHERE span_name = 'llm.generate'
  AND timestamp >= now() - INTERVAL 1 DAY
GROUP BY llm_model;
```

### 5. CPU Hotspots

```sql
SELECT
    arrayJoin(samples.locations) AS function_name,
    sum(arraySum(samples.values)) AS total_samples,
    total_samples / (SELECT sum(arraySum(samples.values)) FROM cpu_profiles) * 100 AS percent
FROM cpu_profiles
WHERE timestamp >= now() - INTERVAL 1 HOUR
GROUP BY function_name
ORDER BY total_samples DESC
LIMIT 20;
```

---

## 🚀 Deployment

### Docker Compose (Development)

```yaml
version: '3.8'

services:
  clickhouse:
    image: clickhouse/clickhouse-server:latest
    ports:
      - "8123:8123"   # HTTP
      - "9000:9000"   # Native
    environment:
      CLICKHOUSE_DB: telemetry
      CLICKHOUSE_USER: default
      CLICKHOUSE_PASSWORD: ""
    volumes:
      - clickhouse_data:/var/lib/clickhouse
      - ./init-schema.sql:/docker-entrypoint-initdb.d/init.sql
  
  rak-ingest:
    build: .
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
      - "4319:4319"   # pprof
      - "8080:8080"   # Query API
    environment:
      CLICKHOUSE_URL: "http://clickhouse:8123"
      RUST_LOG: "info,rak_ingest=debug"
    depends_on:
      - clickhouse
    volumes:
      - ./config.toml:/app/config.toml

volumes:
  clickhouse_data:
```

### Dockerfile

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/rak-ingest/Cargo.toml ./crates/rak-ingest/

# Build dependencies (cached layer)
RUN mkdir crates/rak-ingest/src && \
    echo "fn main() {}" > crates/rak-ingest/src/main.rs && \
    cargo build --release --package rak-ingest

# Copy source
COPY crates/rak-ingest/src ./crates/rak-ingest/src

# Build application
RUN touch crates/rak-ingest/src/main.rs && \
    cargo build --release --package rak-ingest

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rak-ingest /usr/local/bin/

EXPOSE 4317 4318 4319 8080

CMD ["rak-ingest"]
```

---

## 📈 Performance Characteristics

### Ingestion Throughput

| Metric | Expected Performance |
|--------|---------------------|
| **Spans/sec** | ~50,000 (single instance) |
| **Metrics/sec** | ~100,000 (single instance) |
| **Profiles/sec** | ~100 (single instance) |
| **Batch latency** | < 10 seconds (configurable) |

### Storage Requirements

| Data Type | Size per Item | Daily Volume (1000 agents) |
|-----------|---------------|----------------------------|
| **Span** | ~500 bytes | ~43 GB/day (uncompressed) |
| **Metric** | ~100 bytes | ~8.6 GB/day (uncompressed) |
| **Profile** | ~50 KB | ~4.3 GB/day (uncompressed) |

With ClickHouse compression (~10x):
- **Total**: ~5.6 GB/day compressed

### Query Performance

| Query Type | Expected Latency |
|------------|------------------|
| **Single trace** | < 100ms |
| **Trace search (1 day)** | < 1s |
| **Aggregated metrics** | < 500ms (with materialized views) |
| **Profile lookup** | < 200ms |

---

## 🔐 Security Considerations

1. **Authentication**
   - API key authentication for ingestion endpoints
   - mTLS for gRPC connections
   - JWT tokens for query API

2. **Authorization**
   - Role-based access control (RBAC)
   - Per-service data isolation
   - Query rate limiting

3. **Data Privacy**
   - PII scrubbing in attributes
   - Configurable data retention
   - Audit logging

---

## 📊 Monitoring the Ingestion Service

### Key Metrics to Track

```rust
// Expose Prometheus metrics from rak-ingest
use prometheus::{Counter, Histogram, Gauge};

lazy_static! {
    static ref SPANS_RECEIVED: Counter = Counter::new(
        "ingest_spans_received_total",
        "Total spans received"
    ).unwrap();
    
    static ref BATCH_SIZE: Histogram = Histogram::new(
        "ingest_batch_size",
        "Batch size distribution"
    ).unwrap();
    
    static ref FLUSH_DURATION: Histogram = Histogram::new(
        "ingest_flush_duration_seconds",
        "Time to flush to ClickHouse"
    ).unwrap();
    
    static ref BUFFER_SIZE: Gauge = Gauge::new(
        "ingest_buffer_size",
        "Current buffer size"
    ).unwrap();
}
```

---

## ✅ Implementation Checklist

### Phase 1: Core Infrastructure (Week 1)
- [ ] Create `rak-ingest` binary crate
- [ ] Implement configuration loading
- [ ] Set up ClickHouse client
- [ ] Define table schemas
- [ ] Create migration scripts

### Phase 2: OTLP Ingestion (Week 2)
- [ ] Implement OTLP gRPC handler
- [ ] Implement OTLP HTTP handler
- [ ] Parse and validate OTLP data
- [ ] Map OTLP to ClickHouse schema

### Phase 3: Batching (Week 2)
- [ ] Implement BatchBuffer
- [ ] Add background flusher
- [ ] Handle flush failures and retries
- [ ] Add buffer metrics

### Phase 4: Profile Ingestion (Week 3)
- [ ] Implement pprof HTTP handler
- [ ] Decode pprof protobuf
- [ ] Extract and flatten samples
- [ ] Store in ClickHouse

### Phase 5: Query API (Week 3)
- [ ] Implement trace query API
- [ ] Implement metrics query API
- [ ] Implement profile query API
- [ ] Add pagination and filtering

### Phase 6: Testing & Optimization (Week 4)
- [ ] Load testing (10k+ spans/sec)
- [ ] Query performance testing
- [ ] Memory leak testing
- [ ] Docker compose setup

### Phase 7: Production Ready (Week 5)
- [ ] Add authentication
- [ ] Add monitoring/metrics
- [ ] Documentation
- [ ] Deployment guides

---

## 🎯 Estimated Timeline

**Total Implementation Time**: ~5 weeks (1 developer)

- **Core Infrastructure**: 1 week
- **OTLP Ingestion + Batching**: 1.5 weeks
- **Profile Ingestion**: 1 week
- **Query API**: 1 week
- **Testing & Polish**: 0.5 weeks

---

## 💰 Cost Estimates

### Infrastructure Costs (Monthly)

| Component | Spec | Cost (AWS) |
|-----------|------|------------|
| **rak-ingest** (2 instances) | t3.medium | ~$60/month |
| **ClickHouse** (3-node cluster) | r6i.xlarge | ~$450/month |
| **Load Balancer** | ALB | ~$20/month |
| **Storage** (compressed) | 170 GB/month | ~$17/month |
| **Backup** | S3 | ~$10/month |
| **Total** | | **~$557/month** |

For 1000 agents producing ~5.6 GB/day compressed.

---

## 🚀 Next Steps

1. **Review Design** - Approve architecture and schema
2. **Set up ClickHouse** - Local dev environment
3. **Implement Core** - Start with basic ingestion
4. **Test Locally** - Use docker-compose
5. **Deploy to Staging** - Test with real data
6. **Production Rollout** - Monitor and optimize

---

**Ready to build a production-grade telemetry backend?** 📊🚀

This will give you complete ownership of your observability data with ClickHouse's incredible query performance!

