//! Database Tools Example
//!
//! Demonstrates how to use native PostgreSQL and SQLite tools with RAK agents.
//! Shows both read-only (default) and write-enabled modes.

#[path = "common.rs"]
mod common;

use anyhow::Result;
use zdk_agent::LLMAgent;
use zdk_core::{Agent, Content, Part};
use zdk_database_tools::{create_postgres_tools, create_sqlite_tools, DatabaseToolConfig};
use zdk_runner::{RunConfig, Runner};
use zdk_session::inmemory::InMemorySessionService;
use futures::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("=== Database Tools Example ===");

    // Load configuration
    let config = common::load_config()?;

    // Create Gemini model
    let model = common::create_gemini_model(&config)?;

    // Example 1: SQLite with read-only mode (default)
    tracing::info!("Example 1: SQLite Read-Only Mode");
    run_sqlite_readonly_example(model.clone()).await?;

    // Example 2: SQLite with write mode
    tracing::info!("Example 2: SQLite Write-Enabled Mode");
    run_sqlite_write_example(model.clone()).await?;

    // Example 3: PostgreSQL (if connection string is provided)
    if let Ok(postgres_url) = std::env::var("DATABASE_URL") {
        tracing::info!("Example 3: PostgreSQL");
        run_postgres_example(model.clone(), &postgres_url).await?;
    } else {
        tracing::info!("Skipping PostgreSQL example (DATABASE_URL not set)");
        tracing::info!("To run PostgreSQL example, set: export DATABASE_URL=postgresql://user:pass@localhost/db");
    }

    Ok(())
}

/// Example 1: SQLite with read-only tools (default)
async fn run_sqlite_readonly_example(model: Arc<dyn zdk_core::LLM>) -> Result<()> {
    // Create an in-memory SQLite database
    let readonly_tools = create_sqlite_tools("sqlite::memory:").await?;

    tracing::info!(
        tool_count = readonly_tools.len(),
        "Created SQLite tools (read-only mode)"
    );

    // Create an agent with read-only database tools
    let agent = LLMAgent::builder()
        .name("data_analyst")
        .description("Analyzes data in SQLite databases")
        .model(model)
        .tools(readonly_tools)
        .build()?;

    tracing::info!(agent = %agent.name(), "Created agent");

    // Create session service and runner
    let session_service = Arc::new(InMemorySessionService::new());
    let runner = Runner::builder()
        .app_name("database-tools-demo")
        .agent(Arc::new(agent))
        .session_service(session_service)
        .build()?;

    // Run agent with query
    let content = Content::new_user_text("List all available database tools");
    let mut stream = runner
        .run("user-1".to_string(), "session-readonly".to_string(), content, RunConfig::default())
        .await?;

    // Process events
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let Part::Text { text } = part {
                            tracing::info!("Agent response: {}", text);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Example 2: SQLite with write-enabled tools
async fn run_sqlite_write_example(model: Arc<dyn zdk_core::LLM>) -> Result<()> {
    // Create configuration with write permissions
    let config = DatabaseToolConfig::with_write_enabled();

    let write_tools = zdk_database_tools::create_sqlite_tools_with_config(
        "sqlite::memory:",
        config,
    )
    .await?;

    tracing::info!(
        tool_count = write_tools.len(),
        "Created SQLite tools (write-enabled mode)"
    );

    // Create an agent with write-enabled database tools
    let agent = LLMAgent::builder()
        .name("data_admin")
        .description("Manages data in SQLite databases with write permissions")
        .model(model)
        .tools(write_tools)
        .build()?;

    tracing::info!(agent = %agent.name(), "Created agent");

    // Create session service and runner
    let session_service = Arc::new(InMemorySessionService::new());
    let runner = Runner::builder()
        .app_name("database-tools-demo")
        .agent(Arc::new(agent))
        .session_service(session_service)
        .build()?;

    // Run agent with query
    let content = Content::new_user_text(
        "Create a table named 'users' with columns: id (integer), name (text), email (text)"
    );
    let mut stream = runner
        .run("user-1".to_string(), "session-write".to_string(), content, RunConfig::default())
        .await?;

    // Process events
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let Part::Text { text } = part {
                            tracing::info!("Agent response: {}", text);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Example 3: PostgreSQL with connection string
async fn run_postgres_example(model: Arc<dyn zdk_core::LLM>, connection_url: &str) -> Result<()> {
    // Create PostgreSQL tools (read-only by default)
    let postgres_tools = create_postgres_tools(connection_url).await?;

    tracing::info!(
        tool_count = postgres_tools.len(),
        "Created PostgreSQL tools (read-only mode)"
    );

    // Create an agent with PostgreSQL tools
    let agent = LLMAgent::builder()
        .name("postgres_analyst")
        .description("Analyzes data in PostgreSQL databases")
        .model(model)
        .tools(postgres_tools)
        .build()?;

    tracing::info!(agent = %agent.name(), "Created agent");

    // Create session service and runner
    let session_service = Arc::new(InMemorySessionService::new());
    let runner = Runner::builder()
        .app_name("database-tools-demo")
        .agent(Arc::new(agent))
        .session_service(session_service)
        .build()?;

    // Run agent with query
    let content = Content::new_user_text("List all tables in the database and describe the first one");
    let mut stream = runner
        .run("user-1".to_string(), "session-postgres".to_string(), content, RunConfig::default())
        .await?;

    // Process events
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let Part::Text { text } = part {
                            tracing::info!("Agent response: {}", text);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error: {}", e);
            }
        }
    }

    Ok(())
}

