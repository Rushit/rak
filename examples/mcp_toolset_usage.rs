//! MCP Toolset Example
//!
//! Demonstrates how to use MCP (Model Context Protocol) to dynamically load tools
//! from external servers. This example shows connecting to a PostgreSQL MCP server.
//!
//! Prerequisites:
//! - Install uv: `curl -LsSf https://astral.sh/uv/install.sh | sh`
//! - Set DATABASE_URI: `export DATABASE_URI=postgresql://localhost/mydb`
//!
//! The MCP server will be automatically spawned as a subprocess.

#[path = "common.rs"]
mod common;

use anyhow::Result;
use rak_agent::LLMAgent;
use rak_core::{Agent, Content, Part};
use rak_mcp::{McpToolset, StdioConnectionParams};
use rak_runner::{RunConfig, Runner};
use rak_session::inmemory::InMemorySessionService;
use futures::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("=== MCP Toolset Example ===");

    // Load configuration
    let config = common::load_config()?;

    // Create Gemini model
    let model = common::create_gemini_model(&config)?;

    // Check if DATABASE_URI is set
    let database_uri = std::env::var("DATABASE_URI")
        .unwrap_or_else(|_| "postgresql://localhost/postgres".to_string());

    tracing::info!("Using database URI: {}", database_uri);

    // Create MCP toolset for PostgreSQL
    let postgres_mcp = Arc::new(
        McpToolset::builder()
            .name("postgres_mcp")
            .connection(
                StdioConnectionParams::new("uvx")
                    .arg("postgres-mcp")
                    .arg("--access-mode=unrestricted")
                    .env("DATABASE_URI", database_uri.clone()),
            )
            .tool_filter(vec![
                "list_tables".to_string(),
                "query".to_string(),
                "describe_table".to_string(),
            ])
            .build()?,
    );

    tracing::info!("Created MCP toolset");

    // Create an agent with the MCP toolset
    let agent = LLMAgent::builder()
        .name("database_agent")
        .description("Interacts with PostgreSQL databases via MCP")
        .model(model)
        .toolset(postgres_mcp)
        .build()?;

    tracing::info!(agent = %agent.name(), "Created agent with MCP toolset");

    // Create session service and runner
    let session_service = Arc::new(InMemorySessionService::new());
    let runner = Runner::builder()
        .app_name("mcp-demo")
        .agent(Arc::new(agent))
        .session_service(session_service)
        .build()?;

    // Run the agent - tools will be dynamically loaded from MCP server
    let content = Content::new_user_text("List all tables in the database and show their structure");
    let mut stream = runner
        .run("user-1".to_string(), "session-mcp".to_string(), content, RunConfig::default())
        .await?;

    tracing::info!("Starting agent execution with MCP tools...");

    // Process events
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        match part {
                            Part::Text { text } => {
                                tracing::info!("Agent: {}", text);
                            }
                            Part::FunctionCall { function_call } => {
                                tracing::info!("Calling MCP tool: {}", function_call.name);
                            }
                            Part::FunctionResponse { function_response } => {
                                tracing::info!("MCP tool response: {:?}", function_response.response);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error: {}", e);
            }
        }
    }

    tracing::info!("=== MCP Example Complete ===");

    Ok(())
}

