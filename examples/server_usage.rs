//! Example demonstrating RAK server with WebSocket support
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
use zdk_agent::LLMAgent;
use zdk_core::{AuthCredentials, RakConfig, LLM};
use zdk_model::GeminiModel;
use zdk_runner::Runner;
use zdk_server::rest::create_router;
use zdk_session::inmemory::InMemorySessionService;
use zdk_tool::Tool;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║         RAK Server Example - WebSocket Enabled           ║");
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
            .app_name("rak-server")
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

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║                    Server Running                         ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    println!("🌐 Server listening on: http://{}", addr);
    println!("\n📡 Available endpoints:");
    println!("   GET  /health                                    - Health check");
    println!("   GET  /readiness                                 - Readiness check");
    println!("   POST /api/v1/sessions                           - Create session");
    println!("   POST /api/v1/sessions/:id/run                   - Run agent (batch)");
    println!("   POST /api/v1/sessions/:id/run/sse               - Run agent (SSE stream)");
    println!("   GET  /api/v1/sessions/:id/run/ws                - WebSocket connection");
    println!("\n🧪 Test with:");
    println!("   curl http://{}/health", addr);
    println!("   cargo run --example websocket_usage");
    println!("\n⏹️  Press Ctrl+C to stop\n");

    axum::serve(listener, app)
        .await
        .context("Server failed to start")?;

    Ok(())
}

/// Load configuration from file
///
/// Priority:
/// 1. CONFIG_FILE environment variable
/// 2. config.test.toml (for examples and tests)
/// 3. config.toml (for development)
fn load_config() -> Result<RakConfig> {
    // Check for explicit config file
    if let Ok(config_path) = env::var("CONFIG_FILE") {
        println!("📄 Loading config from: {}", config_path);
        return RakConfig::load_from(Some(&PathBuf::from(config_path)))
            .context("Failed to load specified config file");
    }

    // Try config.test.toml first (for examples)
    if PathBuf::from("config.test.toml").exists() {
        println!("📄 Loading config from: config.test.toml");
        return RakConfig::load_from(Some(&PathBuf::from("config.test.toml")))
            .context("Failed to load config.test.toml");
    }

    // Fall back to config.toml
    println!("📄 Loading config from: config.toml");
    RakConfig::load().context("Failed to load config.toml")
}

/// Describe authentication method for logging
fn describe_auth(creds: &AuthCredentials) -> String {
    match creds {
        AuthCredentials::ApiKey { .. } => "API Key (Public Gemini API)".to_string(),
        AuthCredentials::GCloud {
            project, location, ..
        } => format!("Google Cloud (project: {}, location: {})", project, location),
    }
}

/// Create model from authentication credentials
///
/// This demonstrates how to use the new auth abstraction to create models
/// based on the configured authentication method.
fn create_model_from_auth(creds: AuthCredentials, config: &RakConfig) -> Result<Arc<dyn LLM>> {
    let model: Arc<dyn LLM> = match creds {
        AuthCredentials::ApiKey { key } => {
            // Public Gemini API with API key
            Arc::new(GeminiModel::new(
                key,
                config.model.model_name.clone(),
            ))
        }
        AuthCredentials::GCloud {
            token,
            project,
            location,
            ..
        } => {
            // Vertex AI with gcloud bearer token
            Arc::new(GeminiModel::with_bearer_token(
                token,
                config.model.model_name.clone(),
                project,
                location,
            ))
        }
    };
    Ok(model)
}

/// Create an agent with tools
fn create_agent(model: Arc<dyn LLM>) -> Result<Arc<dyn zdk_core::Agent>> {
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
