//! Example demonstrating ZDK server with WebSocket support
//!
//! This example shows how to:
//! - Start an HTTP/WebSocket server with the new auth abstraction
//! - Use either API key or gcloud authentication (configured in config.toml)
//! - Create a Runner with a model that uses the configured auth
//! - Accept WebSocket connections for real-time agent interaction
//! - Handle sessions and invocations
//!
//! ## Setup
//!
//! ### Option 1: Google Cloud Authentication (Recommended)
//! ```bash
//! # Authenticate with gcloud
//! gcloud auth login
//!
//! # Set your project
//! gcloud config set project YOUR_PROJECT_ID
//!
//! # Update config.test.toml (or config.toml) to use gcloud
//! [auth]
//! provider = "gcloud"
//! ```
//!
//! ### Option 2: API Key Authentication
//! ```bash
//! # Set environment variable
//! export GOOGLE_API_KEY="your-api-key"
//!
//! # Update config.test.toml (or config.toml) to use API key
//! [auth]
//! provider = "api_key"
//!
//! [auth.api_key]
//! key = "${GOOGLE_API_KEY}"
//! ```
//!
//! ## Running
//!
//! Run the server (uses config.test.toml by default):
//! ```bash
//! cargo run --example server_usage
//! ```
//!
//! Or with a specific config file:
//! ```bash
//! CONFIG_FILE=config.toml cargo run --example server_usage
//! ```
//!
//! Then in another terminal, test the WebSocket client:
//! ```bash
//! cargo run --example websocket_usage
//! ```
//!
//! Or test with curl:
//! ```bash
//! # Health check
//! curl http://localhost:18080/health
//!
//! # Create session
//! curl -X POST http://localhost:18080/api/v1/sessions \
//!   -H "Content-Type: application/json" \
//!   -d '{"appName":"test","userId":"user1"}'
//!
//! # Run agent (SSE streaming)
//! curl -X POST http://localhost:18080/api/v1/sessions/SESSION_ID/run/sse \
//!   -H "Content-Type: application/json" \
//!   -d '{"newMessage":{"parts":[{"text":"Hello!"}]},"streaming":true}'
//! ```

use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{Duration, sleep, timeout};
use zdk_agent::LLMAgent;
use zdk_core::{AuthCredentials, Provider, ZConfig};
use zdk_runner::Runner;
use zdk_server::rest::create_router;
use zdk_session::inmemory::InMemorySessionService;
use zdk_tool::Tool;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║         ZDK Server Example - WebSocket Enabled           ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Load configuration
    let config = load_config()?;
    println!("✓ Configuration loaded");

    // Get authentication credentials
    let auth_creds = config
        .get_auth_credentials()
        .context("Failed to get authentication credentials")?;

    println!("✓ Authentication: {}", describe_auth(&auth_creds));

    // Create model with authentication
    let model = create_model_from_auth(auth_creds, &config)?;
    println!("✓ Model created: {}", config.model.model_name);

    // Create agent
    let agent = create_agent(model)?;
    println!("✓ Agent created: assistant");

    // Create session service (shared by runner and server)
    let session_service = Arc::new(InMemorySessionService::new());
    println!("✓ Session service created (in-memory)");

    // Create runner with agent and session service
    let runner = Arc::new(
        Runner::builder()
            .app_name("zdk-server")
            .agent(agent)
            .session_service(session_service.clone())
            .build()
            .context("Failed to create runner")?,
    );
    println!("✓ Runner created");

    // Build router
    let app = create_router(runner, session_service);
    println!("✓ Routes configured");

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let server_url = format!("http://{}", addr);

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║                    Server Running                         ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    println!("🌐 Server listening on: {}", server_url);
    println!("\n📡 Available endpoints:");
    println!("   GET  /health                                    - Health check");
    println!("   GET  /readiness                                 - Readiness check");
    println!("   POST /api/v1/sessions                           - Create session");
    println!("   POST /api/v1/sessions/:id/run                   - Run agent (batch)");
    println!("   POST /api/v1/sessions/:id/run/sse               - Run agent (SSE stream)");
    println!("   GET  /api/v1/sessions/:id/run/ws                - WebSocket connection");

    println!("\n🧪 Running example workflow...");
    run_example_workflow(server_url, app, listener).await?;

    Ok(())
}

/// Run example workflow: start server, create session, interact with agent, shutdown
async fn run_example_workflow(
    server_url: String,
    app: axum::Router,
    listener: tokio::net::TcpListener,
) -> Result<()> {
    use serde_json::json;

    // Start server in background
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .context("Server failed to start")
    });

    // Wait for server to be ready
    println!("⏳ Waiting for server to start...");
    sleep(Duration::from_millis(500)).await;

    let client = reqwest::Client::new();

    // Step 1: Health check
    println!("\n📋 Step 1: Health Check");
    let health_url = format!("{}/health", server_url);
    match timeout(Duration::from_secs(5), client.get(&health_url).send()).await {
        Ok(Ok(response)) => {
            if response.status().is_success() {
                println!("   ✅ Server is healthy ({})", response.status());
            } else {
                anyhow::bail!("Health check failed with status: {}", response.status());
            }
        }
        Ok(Err(e)) => {
            anyhow::bail!("Failed to connect to server: {}", e);
        }
        Err(_) => {
            anyhow::bail!("Health check timed out");
        }
    }

    // Step 2: Create session
    println!("\n📋 Step 2: Create Session");
    let session_url = format!("{}/api/v1/sessions", server_url);
    let session_payload = json!({
        "appName": "example-app",
        "userId": "demo-user"
    });

    let session_response = timeout(
        Duration::from_secs(5),
        client.post(&session_url).json(&session_payload).send(),
    )
    .await
    .context("Session creation timed out")?
    .context("Failed to create session")?;

    if !session_response.status().is_success() {
        anyhow::bail!("Session creation failed: {}", session_response.status());
    }

    let session_data: serde_json::Value = session_response.json().await?;
    let session_id = session_data["sessionId"]
        .as_str()
        .context("Session ID not found in response")?;

    println!("   ✅ Session created: {}", session_id);

    // Step 3: Send message to agent (batch mode for simplicity)
    println!("\n📋 Step 3: Interact with Agent");
    let run_url = format!("{}/api/v1/sessions/{}/run", server_url, session_id);
    let message_payload = json!({
        "newMessage": {
            "role": "user",
            "parts": [{"text": "Hello! Can you tell me what 2+2 equals?"}]
        },
        "streaming": false
    });

    println!("   💬 Sending: 'Hello! Can you tell me what 2+2 equals?'");

    let agent_response = timeout(
        Duration::from_secs(10),
        client.post(&run_url).json(&message_payload).send(),
    )
    .await
    .context("Agent interaction timed out")?
    .context("Failed to run agent")?;

    if !agent_response.status().is_success() {
        anyhow::bail!("Agent interaction failed: {}", agent_response.status());
    }

    let response_data: serde_json::Value = agent_response.json().await?;
    println!("   ✅ Agent responded successfully");

    // Extract and display response text from events
    if let Some(events) = response_data.get("events").and_then(|e| e.as_array()) {
        // Look for the final response event
        for event in events.iter().rev() {
            if let Some(content) = event.get("content") {
                if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                    if let Some(text) = parts
                        .first()
                        .and_then(|p| p.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        // Truncate long responses for display
                        let display_text = if text.len() > 100 {
                            format!("{}...", &text[..100])
                        } else {
                            text.to_string()
                        };
                        println!("   💭 Response: {}", display_text);
                        break;
                    }
                }
            }
        }
    }

    // Step 4: Shutdown
    println!("\n📋 Step 4: Cleanup");
    server_handle.abort();
    println!("   ✅ Server shutdown gracefully");

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║              Example Completed Successfully!             ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    Ok(())
}

/// Load configuration from file
///
/// Priority:
/// 1. CONFIG_FILE environment variable
/// 2. config.test.toml (for examples and tests)
/// 3. config.toml (for development)
fn load_config() -> Result<ZConfig> {
    // Check for explicit config file
    if let Ok(config_path) = env::var("CONFIG_FILE") {
        println!("📄 Loading config from: {}", config_path);
        return ZConfig::load_from(Some(&PathBuf::from(config_path)))
            .context("Failed to load specified config file");
    }

    // Try config.test.toml first (for examples)
    if PathBuf::from("config.test.toml").exists() {
        println!("📄 Loading config from: config.test.toml");
        return ZConfig::load_from(Some(&PathBuf::from("config.test.toml")))
            .context("Failed to load config.test.toml");
    }

    // Fall back to config.toml
    println!("📄 Loading config from: config.toml");
    ZConfig::load().context("Failed to load config.toml")
}

/// Describe authentication method for logging
fn describe_auth(creds: &AuthCredentials) -> String {
    match creds {
        AuthCredentials::ApiKey { .. } => "API Key (Public Gemini API)".to_string(),
        AuthCredentials::GCloud {
            project, location, ..
        } => format!(
            "Google Cloud (project: {}, location: {})",
            project, location
        ),
    }
}

/// Create model from configuration using the factory
///
/// This uses the unified provider system which automatically
/// selects the appropriate provider and authentication method.
fn create_model_from_auth(_creds: AuthCredentials, config: &ZConfig) -> Result<Arc<dyn Provider>> {
    use zdk_core::ZConfigExt;
    config
        .create_provider()
        .context("Failed to create provider from configuration")
}

/// Create an agent with tools
fn create_agent(model: Arc<dyn Provider>) -> Result<Arc<dyn zdk_core::Agent>> {
    // For this example, we'll create a simple agent without tools
    // In a real application, you would register tools here
    let tools: Vec<Arc<dyn Tool>> = vec![];

    let agent = LLMAgent::builder()
        .name("assistant")
        .model(model)
        .system_instruction(
            "You are a helpful AI assistant. Be concise and clear in your responses.",
        )
        .tools(tools)
        .build()?;

    Ok(Arc::new(agent))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_auth_api_key() {
        let creds = AuthCredentials::ApiKey {
            key: "test-key".to_string(),
        };
        let desc = describe_auth(&creds);
        assert!(desc.contains("API Key"));
    }

    #[test]
    fn test_describe_auth_gcloud() {
        let creds = AuthCredentials::GCloud {
            token: "test-token".to_string(),
            project: "test-project".to_string(),
            location: "us-central1".to_string(),
            endpoint: None,
        };
        let desc = describe_auth(&creds);
        assert!(desc.contains("Google Cloud"));
        assert!(desc.contains("test-project"));
        assert!(desc.contains("us-central1"));
    }
}
