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
use zdk_core::{Content, ZConfig};  // NEW: ZConfig
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup (NEW: Load from config.toml)
    let config = ZConfig::load()?;
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

### Quick Reference

```bash
# Fast workflow (before committing)
make fmt                # Format code
make clippy             # Lint with warnings
make test               # Run all tests
make test-examples      # Test all examples

# Development cycle
make check              # Fast check without building
make build              # Build all crates
make doc                # Generate and open docs

# Cleanup
make clean              # Remove build artifacts
```

### Building and Testing

```bash
# Build Commands
make build              # Build all workspace crates
make check              # Fast check without full build
make release            # Build optimized release version
make clean              # Clean build artifacts and lock files

# Testing Commands
make test               # Run all tests (80+ tests)
make test-verbose       # Run tests with debug logging (RUST_LOG=debug)
make test-examples      # Test all examples with scripts/test_examples.sh

# Code Quality
make fmt                # Format all code with rustfmt
make clippy             # Lint with clippy (warnings allowed)
make clippy-strict      # Lint with -D warnings (recommended for high quality)

# Documentation
make doc                # Generate and open documentation in browser
```

### Detailed Cargo Commands

If you prefer direct `cargo` commands or need more control:

#### Build Commands
```bash
# Standard build
cargo build

# Release build (optimized)
cargo build --release

# Build specific crate
cargo build -p zdk-core

# Check without building (fast)
cargo check --workspace

# Build with all features
cargo build --all-features
```

#### Test Commands
```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific crate
cargo test -p zdk-agent

# Run tests with debug logging
RUST_LOG=debug cargo test --workspace

# Run integration tests only
cargo test --test '*'

# Run doc tests
cargo test --doc
```

#### Code Quality Commands
```bash
# Format code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace

# Clippy with all warnings
cargo clippy --workspace -- -D warnings

# Clippy for specific crate
cargo clippy -p zdk-core

# Fix automatically fixable issues
cargo clippy --fix --workspace --allow-dirty
```

#### Documentation Commands
```bash
# Generate docs
cargo doc --workspace --no-deps

# Generate and open docs
cargo doc --workspace --no-deps --open

# Generate docs with private items
cargo doc --workspace --no-deps --document-private-items

# Check doc links
cargo doc --workspace --no-deps --all-features
```

#### Example Commands
```bash
# Run specific example
cargo run --example quickstart

# Run with logging
RUST_LOG=debug cargo run --example tool_usage

# List all examples
cargo run --example 2>&1 | grep "    "

# Run example with release optimizations
cargo run --release --example workflow_agents
```

#### Dependency Management
```bash
# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated  # requires cargo-outdated

# Show dependency tree
cargo tree

# Show dependencies for specific crate
cargo tree -p zdk-core

# Audit dependencies for security issues
cargo audit  # requires cargo-audit
```

#### Cleaning Commands
```bash
# Clean build artifacts
cargo clean

# Clean specific target
cargo clean --release

# Clean and rebuild
cargo clean && cargo build

# Remove Cargo.lock and clean
rm Cargo.lock && cargo clean
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

**Option 3: Use gcloud authentication** (No API key needed!)
```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
cargo run --example quickstart
```

### Available Examples

#### Core Examples
| Example | Command | Description |
|---------|---------|-------------|
| **Quickstart** | `make example-quickstart` | Basic agent with LLM interaction |
| **Config Usage** | `make example-config_usage` | Configuration system demo |
| **Tool Usage** | `make example-tool_usage` | Function calling with tools |
| **Workflow Agents** | `make example-workflow_agents` | Sequential/Parallel/Loop orchestration |

#### Agent Features
| Example | Command | Description |
|---------|---------|-------------|
| **Memory Usage** | `make example-memory_usage` | Long-term memory service |
| **Artifact Usage** | `make example-artifact_usage` | File and document storage |
| **Database Session** | `make example-database_session` | PostgreSQL/SQLite session storage |

#### Tool Integration
| Example | Command | Description |
|---------|---------|-------------|
| **OpenAPI Usage** | `make example-openapi_usage` | Generate tools from OpenAPI specs |
| **Web Tools** | `make example-web_tools_usage` | Search & scrape the web (zero API keys!) |
| **Database Tools** | `cargo run --example database_tools_usage` | Query PostgreSQL/SQLite with agents |
| **MCP Toolset** | `cargo run --example mcp_toolset_usage` | Dynamic tool loading via MCP protocol |

#### Server & Streaming
| Example | Command | Description |
|---------|---------|-------------|
| **Server Usage** | `make example-server_usage` | REST API with SSE streaming |
| **WebSocket Usage** | `make example-websocket_usage` | Bidirectional WebSocket streaming |
| **Telemetry Usage** | `make example-telemetry_usage` | OpenTelemetry tracing & logging |

#### Advanced
| Example | Command | Description |
|---------|---------|-------------|
| **Gemini Gcloud** | `cargo run --example gemini_gcloud_usage` | Vertex AI authentication |

#### Testing Examples
```bash
# Test all examples at once
make test-examples

# Or use the script directly
./scripts/test_examples.sh

# Test specific example with logging
RUST_LOG=debug cargo run --example quickstart
```

### Advanced Commands

```bash
# Build & Quality
make release            # Build optimized release version
make clippy-strict      # Strict linting (recommended)
make watch              # Watch for changes and rebuild (requires cargo-watch)

# Testing
make test-verbose       # Tests with debug logging
make test-examples      # Test all examples with validation
cargo test -p zdk-core  # Test specific crate

# Documentation
make doc                # Generate and open docs
cargo doc --document-private-items --open  # Include private items

# Help
make help               # View all available commands
```

### Common Development Workflows

#### Before Committing
```bash
# Full pre-commit check (recommended)
make fmt && make clippy-strict && make test && make test-examples
```

#### Adding a New Feature
```bash
# 1. Create feature branch
git checkout -b feature/my-feature

# 2. Develop with fast feedback
make check              # Quick syntax check

# 3. Add tests and validate
make test

# 4. Format and lint
make fmt && make clippy

# 5. Test examples still work
make test-examples

# 6. Commit
git add .
git commit -m "feat: Add my feature"
```

#### Debugging Failed Tests
```bash
# Run specific test with logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Run test in specific crate
cargo test -p zdk-agent test_name -- --nocapture

# Run integration test
cargo test --test integration_test -- --nocapture
```

#### Debugging Examples
```bash
# Run with full debug logging
RUST_LOG=trace cargo run --example quickstart

# Run with specific module logging
RUST_LOG=zdk_agent=debug,zdk_runner=info cargo run --example tool_usage

# Run with backtrace on panic
RUST_BACKTRACE=1 cargo run --example workflow_agents
```

#### Performance Profiling
```bash
# Build with release optimizations
cargo build --release

# Run with profiling
cargo run --release --example quickstart

# Benchmark (if benchmarks exist)
cargo bench

# Check binary size
cargo build --release && ls -lh target/release/
```

#### Dependency Management
```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Audit for security vulnerabilities
cargo audit

# Show what depends on a crate
cargo tree -i zdk-core
```

### Makefile Targets Reference

The ZDK Makefile provides convenient shortcuts for common tasks. Here's a complete reference:

#### Build Targets
```bash
make build              # Build all workspace crates
make check              # Fast check without building binaries
make release            # Build with --release optimizations
make clean              # Remove target/ directory and Cargo.lock
```

#### Testing Targets
```bash
make test               # Run all tests (default target)
make test-verbose       # Run tests with RUST_LOG=debug
make test-examples      # Run ./scripts/test_examples.sh
```

#### Code Quality Targets
```bash
make fmt                # Format code with rustfmt
make clippy             # Lint with clippy (warnings allowed)
make clippy-strict      # Lint with -D warnings (fail on warnings - recommended)
```

#### Documentation Targets
```bash
make doc                # Generate and open documentation
```

#### Example Targets
```bash
make example-quickstart              # Run quickstart example
make example-config_usage            # Run config usage example
make example-openai_usage            # Run OpenAI example
make example-tool_usage              # Run tool usage example
make example-workflow_agents         # Run workflow agents example
make example-artifact_usage          # Run artifact usage example
make example-database_session        # Run database session example
make example-memory_usage            # Run memory usage example
make example-websocket_usage         # Run WebSocket example
make example-telemetry_usage         # Run telemetry example (with logging)
make example-openapi_usage           # Run OpenAPI tool generator
make example-web_tools_usage         # Run web tools example
make example-database_tools_usage    # Run database tools example
make example-mcp_toolset_usage       # Run MCP toolset example
make example-gemini_gcloud_usage     # Run Gemini gcloud example
```

#### Utility Targets
```bash
make help               # Show all available targets with descriptions
```

#### Example: Full Pre-Commit Workflow
```bash
# One-liner to prepare for commit (recommended)
make fmt && make clippy-strict && make test && make test-examples
```

---

## Testing

ZDK has a comprehensive testing strategy with multiple test types for different scenarios.

### Test Types

#### 1. Unit Tests (Default)
Fast, isolated tests using mocks. Run by default with `cargo test`.

```bash
# Run all unit tests
make test

# Run with debug logging
make test-verbose

# Run tests for specific crate
cargo test -p zdk-core
```

#### 2. Integration Tests (Default)
End-to-end tests with mock services. Run automatically.

```bash
# Run specific integration test
cargo test --test integration_test
cargo test --test tool_test
cargo test --test workflow_agents_test
```

#### 3. Optional Tests (Ignored)
Tests requiring external setup (gcloud auth, API keys, etc.). Run explicitly when needed.

```bash
# Run all ignored tests
cargo test -- --ignored

# Run specific ignored test with output
cargo test openapi_usage_test -- --ignored --nocapture
```

### Authentication Options

ZDK supports three authentication methods for testing and examples:

#### Option 1: Config File (Recommended)
```bash
# Copy and edit config
cp config.toml.example config.toml
# Add your API keys to config.toml

# Run tests/examples
make test
cargo run --example quickstart
```

#### Option 2: GCloud Authentication (No API Keys!)
```bash
# One-time setup
gcloud auth login
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID

# Run tests/examples - uses your gcloud credentials
cargo run --example gemini_gcloud_usage
cargo test -- --ignored  # Tests using gcloud auth
```

#### Option 3: Environment Variables (Fallback)
```bash
export GOOGLE_API_KEY="your-api-key"
export OPENAI_API_KEY="sk-..."
cargo run --example quickstart
```

### Quick Testing Commands

```bash
# Standard workflow
make test               # Run all unit + integration tests (80+ tests)
make test-verbose       # Run with debug logging (RUST_LOG=debug)
make test-examples      # Test all examples with validation script

# Specific tests
cargo test test_name                    # Run specific test
cargo test -p zdk-agent                 # Test specific crate
cargo test -- --nocapture              # Show test output

# Optional tests (require auth setup)
cargo test -- --ignored                 # Run all optional tests
cargo test openapi_test -- --ignored   # Run specific optional test
```

### Test Organization

```
zdk/
├── tests/
│   ├── common.rs              # Shared utilities (gcloud auth helpers)
│   ├── integration_test.rs    # E2E integration tests
│   ├── tool_test.rs           # Tool execution tests
│   ├── workflow_agents_test.rs # Workflow orchestration tests
│   └── openapi_usage_test.rs  # OpenAPI toolset test (#[ignore])
└── crates/*/src/
    └── lib.rs                 # Unit tests in #[cfg(test)] modules
```

### Writing Tests

#### Unit Test Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_something() {
        let mock = MockLLM::new();
        // Test with mock...
    }
}
```

#### Optional Integration Test Pattern
```rust
mod common;

#[tokio::test]
#[ignore]  // Only run when explicitly requested
async fn test_real_api() {
    // Get gcloud credentials
    let token = common::get_gcloud_access_token()
        .expect("Run: gcloud auth application-default login");
    let project = common::get_gcloud_project()
        .expect("Run: gcloud config set project PROJECT_ID");
    
    // Use real API
    let model = GeminiModel::with_bearer_token(
        token,
        "gemini-1.5-flash".to_string(),
        project,
        "us-central1".to_string(),
    );
    // Test...
}
```

### Continuous Integration

CI/CD pipelines run:
- ✅ All unit tests (fast, no external dependencies)
- ✅ All integration tests with mocks
- ❌ **Not** ignored tests (require manual setup)

To run the same tests as CI locally:
```bash
make test  # Same as CI
```

### Troubleshooting

#### "gcloud command not found"
```bash
# Install gcloud CLI
brew install google-cloud-sdk  # macOS
# OR
curl https://sdk.cloud.google.com | bash  # Linux
```

#### "Failed to get gcloud token"
```bash
gcloud auth login
gcloud auth application-default login
```

#### "No default project set"
```bash
gcloud config set project YOUR_PROJECT_ID
```

#### "API key not found"
Either:
1. Set in `config.toml`: `api_key = "your-key"`
2. Set environment: `export GOOGLE_API_KEY="your-key"`
3. Use gcloud auth (recommended)

### Best Practices

1. **Use mocks for unit tests** - Fast, reliable, no setup required
2. **Mark API tests as `#[ignore]`** - Only run when needed
3. **Use gcloud auth** - Easier than managing API keys
4. **Document prerequisites** - Clear instructions in test comments
5. **Provide helpful errors** - Guide users to fix auth issues

### See Also

- **Detailed Testing Guide**: [README_TESTING.md](README_TESTING.md)
- **Contributing Guidelines**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Documentation Index**: [../zdk-idocs/INDEX.md](../zdk-idocs/INDEX.md)

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


