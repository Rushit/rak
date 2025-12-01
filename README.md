# ZDK (zdk-rs)

ZDK - ZAgent Development Kit for Rust - A code-first framework for building AI agents with Rust.


## Features

- **Database Tools**: Native PostgreSQL and SQLite tools with security-first design 🗄️ (NEW)
- **MCP Support**: Model Context Protocol integration for dynamic tool loading 🔌 (NEW)
- **Web Tools**: Search the web and scrape content - ZERO additional API keys needed! 🌐
- **OpenAPI Tools**: Automatically generate tools from OpenAPI specs - instant API integration! 🚀
- **Async/Await**: Built on Tokio for high-performance async operations
- **Streaming Support**: Real-time SSE and WebSocket streaming for agent responses
- **WebSocket Support**: Bidirectional communication with cancellation and status queries
- **Type-Safe**: Leverages Rust's type system for safe agent development
- **Model Agnostic**: Pluggable LLM providers (Gemini, OpenAI, etc.)
- **Tool System**: Function calling with built-in tools and custom tool support
- **Workflow Agents**: Sequential, Parallel, and Loop orchestration patterns
- **Storage Providers**: Artifact storage (memory, filesystem) and database sessions (PostgreSQL, SQLite)
- **Memory Service**: Long-term memory with keyword-based search across sessions
- **Observability**: OpenTelemetry tracing, structured logging, and health checks
- **Extensible**: Easy-to-extend tool system and agent workflows

## Quick Start

### Prerequisites

- Rust 1.90.0+ (edition 2024)
- Gemini API key (or other LLM provider)

### Configuration Setup

ZDK now uses a **config-first** approach with the following priority:

```
Priority: config.toml > Environment Variables > Defaults
```

**Quick Setup**:
```bash
# 1. Copy the example config
cp config.toml.example config.toml

# 2. Edit with your API key
vim config.toml

# 3. Run examples
cargo run --example quickstart
```

**config.toml**:
```toml
[model]
api_key = "your-api-key-here"  # Or: "${GEMINI_API_KEY}"
model_name = "gemini-2.0-flash-exp"
```

✅ Benefits:
- Separate configs for dev/test/prod environments
- `config.toml` is in `.gitignore` (safe)
- Can still use environment variables as fallback

See the [Configuration Guide](docs/20251119_2210_CONFIG_MIGRATION.md) for more details.

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
zdk-core = { path = "path/to/zdk-rs/crates/zdk-core" }
zdk-agent = { path = "path/to/zdk-rs/crates/zdk-agent" }
zdk-model = { path = "path/to/zdk-rs/crates/zdk-model" }
zdk-runner = { path = "path/to/zdk-rs/crates/zdk-runner" }
zdk-session = { path = "path/to/zdk-rs/crates/zdk-session", features = ["sqlite"] }  # Optional: postgres
zdk-tool = { path = "path/to/zdk-rs/crates/zdk-tool" }
zdk-artifact = { path = "path/to/zdk-rs/crates/zdk-artifact" }
zdk-memory = { path = "path/to/zdk-rs/crates/zdk-memory" }
zdk-telemetry = { path = "path/to/zdk-rs/crates/zdk-telemetry" }  # Optional: for OpenTelemetry
zdk-openapi = { path = "path/to/zdk-rs/crates/zdk-openapi" }  # Optional: for OpenAPI tool generation
zdk-web-tools = { path = "path/to/zdk-rs/crates/zdk-web-tools" }  # Optional: for web search & scraping
zdk-database-tools = { path = "path/to/zdk-rs/crates/zdk-database-tools" }  # Optional: for database access
zdk-mcp = { path = "path/to/zdk-rs/crates/zdk-mcp" }  # Optional: for MCP protocol support
```

### Example

```rust
use zdk_agent::LLMAgent;
use zdk_model::GeminiModel;
use zdk_runner::Runner;
use zdk_session::inmemory::InMemorySessionService;
use zdk_core::{Content, ZdkConfig};  // NEW: ZdkConfig
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup (NEW: Load from config.toml)
    let config = ZdkConfig::load()?;
    let api_key = config.api_key()?;
    let model = Arc::new(GeminiModel::new(api_key, config.model.model_name));
    
    let agent = LLMAgent::builder()
        .name("assistant")
        .description("A helpful AI assistant")
        .model(model)
        .build()?;
    
    let session_service = Arc::new(InMemorySessionService::new());
    let runner = Runner::builder()
        .app_name("my-app")
        .agent(Arc::new(agent))
        .session_service(session_service.clone())
        .build()?;
    
    // Create session
    let session = session_service.create(&rak_session::CreateRequest {
        app_name: "my-app".into(),
        user_id: "user1".into(),
        session_id: None,
    }).await?;
    
    // Run agent
    let message = Content::new_user_text("Hello, how are you?");
    let mut stream = runner.run(
        "user1".into(),
        session.id().into(),
        message,
        Default::default(),
    ).await?;
    
    // Process responses
    while let Some(event) = stream.next().await {
        let event = event?;
        println!("{:?}", event);
    }
    
    Ok(())
}
```

## Using Tools

ZDK supports function calling with built-in and custom tools:

```rust
use zdk_tool::builtin::create_calculator_tool;

// Create an agent with tools
let calculator = Arc::new(create_calculator_tool()?);

let agent = LLMAgent::builder()
    .name("math_assistant")
    .model(model)
    .tool(calculator)  // Add tool
    .build()?;
```

See [examples/tool_usage.rs](examples/tool_usage.rs) for a complete example.

## Using Workflow Agents

ZDK supports multi-agent orchestration with workflow patterns:

```rust
use zdk_agent::{SequentialAgent, ParallelAgent, LoopAgent};

// Sequential: Execute agents in order
let sequential = SequentialAgent::builder()
    .name("pipeline")
    .sub_agent(step1)
    .sub_agent(step2)
    .sub_agent(step3)
    .build()?;

// Parallel: Execute agents concurrently
let parallel = ParallelAgent::builder()
    .name("multi_perspective")
    .sub_agent(agent1)
    .sub_agent(agent2)
    .build()?;

// Loop: Iterate with termination control
let loop_agent = LoopAgent::builder()
    .name("refiner")
    .sub_agent(worker)
    .max_iterations(5)
    .build()?;
```

See [examples/workflow_agents.rs](examples/workflow_agents.rs) for a complete example.

## Using Memory Service

ZDK provides long-term memory for agents to remember past conversations:

```rust
use zdk_memory::{InMemoryMemoryService, MemoryService, SearchRequest};

// Create memory service
let memory_service = InMemoryMemoryService::new();

// Add completed sessions to memory
memory_service.add_session(session).await?;

// Search for relevant memories
let results = memory_service.search(SearchRequest {
    query: "weather forecast".into(),
    user_id: "user123".into(),
    app_name: "my_app".into(),
}).await?;

// Use memories in agent context
for memory in results.memories {
    println!("Found memory: {:?}", memory.content);
}
```

See [examples/memory_usage.rs](examples/memory_usage.rs) for a complete example.

## Using Web Tools

ZDK provides web tools for searching and scraping content - **ZERO additional API keys needed!**

```rust
use zdk_web_tools::{GeminiGoogleSearchTool, GeminiUrlContextTool, WebScraperTool};

// Create web tools - NO additional API keys needed!
let google_search = Arc::new(GeminiGoogleSearchTool::new());
let url_context = Arc::new(GeminiUrlContextTool::new());
let web_scraper = Arc::new(WebScraperTool::new()?);

// Add to Gemini 2.0+ agent
let agent = LLMAgent::builder()
    .name("research_agent")
    .model(gemini_2_0_model)
    .tool(google_search)     // Search the web (Gemini built-in)
    .tool(url_context)       // Read URLs (Gemini built-in)
    .tool(web_scraper)       // Parse HTML (works with any model)
    .build()?;
```

**🔑 API Keys Required**: ZERO! Uses your existing Gemini API key.

**Tools included**:
- **GeminiGoogleSearchTool** - Search the web using Gemini's built-in capability
- **GeminiUrlContextTool** - Fetch URL content using Gemini's built-in capability
- **WebScraperTool** - Parse HTML with CSS selectors (works with any model)

See [examples/web_tools_usage.rs](examples/web_tools_usage.rs) for a complete example.

## Using WebSocket Support

ZDK provides WebSocket support for bidirectional communication:

```rust
// Connect to WebSocket endpoint
let url = "ws://localhost:8080/api/v1/sessions/my-session/run/ws";
let (ws_stream, _) = connect_async(url).await?;
let (mut write, mut read) = ws_stream.split();

// Send run command
let run_msg = WsClientMessage::Run {
    session_id: "my-session".to_string(),
    new_message: Content::new_user_text("Hello!"),
};
write.send(Message::Text(serde_json::to_string(&run_msg)?)).await?;

// Receive events
while let Some(msg) = read.next().await {
    // Handle messages...
}

// Cancel invocation
let cancel_msg = WsClientMessage::Cancel {
    invocation_id: "inv-123".to_string(),
};
write.send(Message::Text(serde_json::to_string(&cancel_msg)?)).await?;
```

**Features**:
- Bidirectional communication
- Cancel running invocations
- Query invocation status
- Real-time event streaming

See [examples/websocket_usage.rs](examples/websocket_usage.rs) for a complete example.

## Using Database Tools

ZDK provides native database tools with **security-first design** - read-only by default!

```rust
use zdk_database_tools::{create_postgres_tools, create_sqlite_tools, DatabaseToolConfig};

// Read-only mode (default) - safe for data analysts
let readonly_tools = create_postgres_tools("postgresql://localhost/mydb").await?;

let analyst_agent = LLMAgent::builder()
    .name("data_analyst")
    .model(model)
    .tools(readonly_tools)  // Can only SELECT
    .build()?;

// Write-enabled mode (opt-in) - for administrators
let config = DatabaseToolConfig::with_write_enabled();
let write_tools = create_postgres_tools_with_config(
    "postgresql://localhost/mydb",
    config
).await?;

let admin_agent = LLMAgent::builder()
    .name("data_admin")
    .model(model)
    .tools(write_tools)  // Can INSERT/UPDATE/DELETE
    .build()?;
```

**🔒 Security Features**:
- **Read-only by default** - Prevents accidental data modification
- **Opt-in writes** - Explicit configuration required for INSERT/UPDATE/DELETE
- **Parameter binding** - Prevents SQL injection attacks
- **Query limits** - Automatic row limits (default: 1000 rows)
- **Timeouts** - Per-query timeouts (default: 30 seconds)

**Supported Databases**:
- **PostgreSQL** - Full-featured PostgreSQL integration
- **SQLite** - Complete SQLite support (including in-memory databases)

**Tools included**:
- `list_tables` - List all tables in the database
- `describe_table` - Get detailed schema information
- `query` - Execute SELECT queries (read-only default)
- `execute` - Execute INSERT/UPDATE/DELETE (opt-in only)

See [examples/database_tools_usage.rs](examples/database_tools_usage.rs) for a complete example.

## Using MCP (Model Context Protocol)

ZDK supports MCP for **dynamic tool loading** from external servers!

```rust
use zdk_mcp::{McpToolset, StdioConnectionParams};

// Connect to PostgreSQL MCP server
let postgres_mcp = Arc::new(
    McpToolset::builder()
        .name("postgres_mcp")
        .connection(
            StdioConnectionParams::new("uvx")
                .arg("postgres-mcp")
                .arg("--access-mode=unrestricted")
                .env("DATABASE_URI", "postgresql://localhost/mydb")
        )
        .tool_filter(vec![
            "list_tables".to_string(),
            "query".to_string(),
            "describe_table".to_string()
        ])
        .build()?
);

// Agent with MCP toolset - tools loaded dynamically!
let agent = LLMAgent::builder()
    .name("database_agent")
    .model(model)
    .toolset(postgres_mcp)  // Dynamic tool loading
    .build()?;

// When agent runs, it will:
// 1. Spawn the MCP server as a subprocess
// 2. Discover available tools
// 3. Load filtered tools
// 4. Execute tools as needed
```

**✨ Benefits**:
- **Dynamic Loading** - Tools loaded at runtime, not compile-time
- **External Servers** - Connect to any MCP-compatible server
- **Language Agnostic** - Use tools written in Python, Node.js, etc.
- **Zero Code Changes** - Add new tools without recompiling

**How it works**:
1. Agent starts and calls `toolset.get_tools()`
2. MCP client spawns server subprocess
3. Client discovers available tools via MCP protocol
4. Tools are wrapped as native ZDK tools
5. Agent can call MCP tools like any other tool

**Prerequisites**:
```bash
# Install uv (for Python MCP servers)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Set database connection
export DATABASE_URI=postgresql://localhost/mydb

# MCP server will be spawned automatically
```

See [examples/mcp_toolset_usage.rs](examples/mcp_toolset_usage.rs) for a complete example.

## Observability & Monitoring

ZDK provides comprehensive observability through OpenTelemetry and structured logging:

### Telemetry Setup

```rust
use zdk_telemetry::init_telemetry;

#[tokio::main]
async fn main() {
    // Initialize telemetry with OpenTelemetry support
    init_telemetry();
    
    // Your application code...
}
```

### Structured Logging

Control logging with the `RUST_LOG` environment variable:

```bash
# Debug level for all components
RUST_LOG=debug cargo run --example quickstart

# Module-specific logging
RUST_LOG=zdk_agent=debug,rak_runner=info cargo run

# Production logging (info and above)
RUST_LOG=info cargo run
```

### Features

- **OpenTelemetry Tracing**: Automatic tracing of LLM calls and tool executions
- **Structured Logs**: All logs include context (invocation_id, session_id, user_id)
- **HTTP Middleware**: Request/response logging with latency tracking
- **Health Checks**: `/health` and `/readiness` endpoints for monitoring
- **Custom Span Processors**: Register custom exporters for traces

### Health Endpoints

```bash
# Liveness check
curl http://localhost:8080/health
# Response: OK

# Readiness check  
curl http://localhost:8080/readiness
# Response: READY
```


## Development Commands

ZDK uses `make` for all common development tasks. Run `make help` to see all available commands.

### Building and Testing

```bash
# Run all tests (default command)
make test

# Build all workspace crates
make build

# Check code without building
make check

# Run clippy linter
make clippy

# Format code
make fmt

# Generate and open documentation
make doc

# Clean build artifacts
make clean
```

### Running Examples

**Option 1: Use config.toml** (Recommended)
```bash
# Copy and edit config
cp config.toml.example config.toml
# Add your API key to config.toml

# Run examples
cargo run --example quickstart
```

**Option 2: Use environment variables** (Fallback)
```bash
export GEMINI_API_KEY="your-api-key-here"
cargo run --example quickstart
```

**Available Examples**:
```bash
# Quickstart example
make example-quickstart

# Tool usage example
make example-tool_usage

# Database tools example (NEW)
cargo run --example database_tools_usage

# MCP toolset example (NEW)
export DATABASE_URI=postgresql://localhost/mydb
cargo run --example mcp_toolset_usage

# Workflow agents example
make example-workflow_agents

# Artifact usage example
make example-artifact_usage

# Database session example
make example-database_session

# Memory service example
make example-memory_usage

# WebSocket client example
make example-websocket_usage

# Telemetry example (with debug logging)
make example-telemetry_usage

# OpenAPI tool generator example
make example-openapi_usage

# Web tools example (search and scrape the web)
make example-web_tools_usage

# Configuration system example
make example-config_usage
```

### Advanced Commands

```bash
# Run tests with verbose output
make test-verbose

# Test all examples
make test-examples

# Build release version
make release

# View all available commands
make help
```

## Testing

Run the test suite with `make`:

```bash
# Run all tests (recommended)
make test

# Run tests with debug logging
make test-verbose
```

**Note**: This runs all 80+ tests across all workspace crates.

For advanced testing options and cargo commands, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Architecture

ZDK follows a modular architecture:

- **zdk-core**: Core traits and types (Agent, LLM, Tool)
- **zdk-model**: LLM provider implementations (Gemini, etc.)
- **zdk-session**: Session management (in-memory, database)
- **zdk-agent**: Agent implementations (LLMAgent, workflow agents)
- **zdk-runner**: Execution orchestration
- **zdk-server**: REST API with SSE and WebSocket streaming
- **zdk-tool**: Tool system with function calling support
- **zdk-macros**: Procedural macros for ergonomic tool creation
- **zdk-artifact**: Artifact storage (files, documents)
- **zdk-memory**: Long-term memory with keyword search
- **zdk-openapi**: OpenAPI tool generator (automatic API integration)
- **zdk-telemetry**: OpenTelemetry tracing and observability

## REST API

Start a server with the `zdk-server` crate:

```rust
use zdk_server::create_router;

let router = create_router(runner, session_service);
let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
axum::serve(listener, router).await?;
```

### Endpoints

- `POST /api/v1/sessions` - Create a session
- `POST /api/v1/sessions/:id/run` - Run agent (batch mode)
- `POST /api/v1/sessions/:id/run/sse` - Run agent (SSE streaming)
- `GET /api/v1/sessions/:id/run/ws` - Run agent (WebSocket with cancellation)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup and workflow
- Code style guidelines
- Testing requirements
- Pull request process
- Detailed cargo commands

## License

Apache 2.0


