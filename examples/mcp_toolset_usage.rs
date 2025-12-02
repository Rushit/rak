//! MCP Toolset Example - SQLite Edition
//!
//! This is a complete end-to-end example demonstrating MCP (Model Context Protocol) integration
//! with SQLite. The example:
//! 1. Creates a temporary SQLite database
//! 2. Populates it with sample e-commerce data (users, products, orders)
//! 3. Connects to the database via MCP server
//! 4. Uses an LLM agent to query the database using natural language
//! 5. Gracefully shuts down the MCP server
//!
//! Prerequisites:
//! - Install uv: `curl -LsSf https://astral.sh/uv/install.sh | sh`
//! - sqlite3 command-line tool (usually pre-installed on macOS/Linux)
//!
//! The MCP server will be automatically spawned and managed as a subprocess.

#[path = "common.rs"]
mod common;

use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use zdk_agent::LLMAgent;
use zdk_core::{Agent, Content};
use zdk_mcp::{McpToolset, StdioConnectionParams};
use zdk_runner::{RunConfig, Runner};
use zdk_session::inmemory::InMemorySessionService;

/// Initialize SQLite database with sample e-commerce data using sqlite3 CLI
fn initialize_database(db_path: &PathBuf) -> Result<()> {
    tracing::info!("Creating SQLite database at: {}", db_path.display());

    // SQL script to initialize the database
    let init_sql = r#"
-- Create users table
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at TEXT NOT NULL
);

-- Create products table
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    price REAL NOT NULL,
    stock INTEGER NOT NULL
);

-- Create orders table
CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    total_price REAL NOT NULL,
    order_date TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (product_id) REFERENCES products (id)
);

-- Insert sample users
INSERT INTO users (name, email, created_at) VALUES 
    ('Alice Johnson', 'alice@example.com', '2024-01-15'),
    ('Bob Smith', 'bob@example.com', '2024-02-20'),
    ('Carol White', 'carol@example.com', '2024-03-10'),
    ('David Brown', 'david@example.com', '2024-04-05');

-- Insert sample products
INSERT INTO products (name, category, price, stock) VALUES 
    ('Laptop Pro', 'Electronics', 1299.99, 15),
    ('Wireless Mouse', 'Electronics', 29.99, 50),
    ('Office Chair', 'Furniture', 249.99, 20),
    ('Desk Lamp', 'Furniture', 49.99, 30),
    ('Coffee Maker', 'Appliances', 89.99, 25),
    ('Notebook Set', 'Stationery', 19.99, 100);

-- Insert sample orders
INSERT INTO orders (user_id, product_id, quantity, total_price, order_date) VALUES 
    (1, 1, 1, 1299.99, '2024-05-01'),  -- Alice bought Laptop
    (1, 2, 2, 59.98, '2024-05-01'),    -- Alice bought 2 Mice
    (2, 3, 1, 249.99, '2024-05-10'),   -- Bob bought Chair
    (2, 4, 1, 49.99, '2024-05-10'),    -- Bob bought Lamp
    (3, 5, 1, 89.99, '2024-05-15'),    -- Carol bought Coffee Maker
    (4, 6, 3, 59.97, '2024-05-20');    -- David bought 3 Notebooks
"#;

    // Execute SQL using sqlite3 command
    let mut child = Command::new("sqlite3")
        .arg(db_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn sqlite3. Make sure sqlite3 is installed.")?;

    // Write SQL to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(init_sql.as_bytes())
            .context("Failed to write SQL to sqlite3")?;
    }

    // Wait for completion
    let output = child
        .wait_with_output()
        .context("Failed to wait for sqlite3 process")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("sqlite3 failed: {}", stderr);
    }

    tracing::info!("Sample data inserted successfully");
    tracing::info!("Database contains: 4 users, 6 products, 6 orders");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Use common utilities - don't repeat yourself!
    common::print_header("MCP Toolset Example - SQLite Edition");

    // Load config with helpful error messages
    let config = common::load_config()?;
    common::show_auth_info(&config)?;

    // Create provider from config
    let model = common::create_gemini_model(&config)?;

    // Create temporary SQLite database
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join("zdk_mcp_example.db");

    // Remove existing database if it exists
    if db_path.exists() {
        std::fs::remove_file(&db_path)?;
        tracing::info!("Removed existing database");
    }

    // Initialize database with sample data
    initialize_database(&db_path)?;

    // Create MCP toolset for SQLite
    tracing::info!("Starting SQLite MCP server...");
    let sqlite_mcp = Arc::new(
        McpToolset::builder()
            .name("sqlite_mcp")
            .connection(
                StdioConnectionParams::new("uvx")
                    .arg("mcp-server-sqlite")
                    .arg("--db-path")
                    .arg(db_path.to_str().unwrap()),
            )
            .tool_filter(vec![
                "read-query".to_string(),
                "write-query".to_string(),
                "create-table".to_string(),
                "list-tables".to_string(),
                "describe-table".to_string(),
                "append-insight".to_string(),
            ])
            .build()?,
    );

    tracing::info!("✅ SQLite MCP server started successfully");

    // Wrap the main logic in a block to ensure proper cleanup
    let result = async {
        // Create an agent with the MCP toolset
        let agent = LLMAgent::builder()
            .name("database_agent")
            .description("Analyzes e-commerce data in SQLite database via MCP")
            .model(model)
            .toolset(sqlite_mcp.clone())
            .system_instruction(
                "You are a data analyst assistant with access to an e-commerce SQLite database. \
                 The database contains three tables: users, products, and orders. \
                 Use the available tools to query the database and provide insights. \
                 Always explain your findings clearly.",
            )
            .build()?;

        tracing::info!(agent = %agent.name(), "Created agent with MCP toolset");

        // Create session service and runner
        let session_service = Arc::new(InMemorySessionService::new());
        let runner = Runner::builder()
            .app_name("mcp-sqlite-demo")
            .agent(Arc::new(agent))
            .session_service(session_service)
            .build()?;

        // Run the agent with a complex query
        let content = Content::new_user_text(
            "Please analyze the database: \
             1) List all tables and their structures \
             2) Show the top 3 customers by total spending \
             3) Which product category has the highest sales? \
             Provide a summary of your findings.",
        );

        let mut stream = runner
            .run(
                "user-1".to_string(),
                "session-mcp-sqlite".to_string(),
                content,
                RunConfig::default(),
            )
            .await?;

        tracing::info!("Starting agent execution with SQLite MCP tools...");

        // Collect and print response using common helper
        let response =
            common::collect_and_print_response(&mut stream, "MCP agent execution").await?;

        // Validate response
        common::validate_response_not_empty(&response, "agent response");
        common::validate_response_min_length(&response, 50, "agent response");

        // Additional validation: Check that agent actually used the database
        if !response.to_lowercase().contains("table") && !response.to_lowercase().contains("users")
        {
            common::validation_failed(
                "Agent response doesn't mention database tables - MCP tools may not be working",
            );
        }

        Ok::<_, anyhow::Error>(())
    }
    .await;

    // Gracefully shutdown the MCP server
    tracing::info!("Shutting down SQLite MCP server...");
    drop(sqlite_mcp);
    tracing::info!("✅ SQLite MCP server stopped gracefully");

    // Check if there were any errors during execution
    result?;

    // Success! Use common validation helper
    println!("\n📊 Database location: {}", db_path.display());
    println!(
        "💡 Tip: You can inspect the database using: sqlite3 {}",
        db_path.display()
    );

    common::validation_passed(
        "SQLite MCP toolset integration verified - server started, agent executed queries, server stopped gracefully",
    );
    std::process::exit(0);
}
