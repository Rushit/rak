//! Example demonstrating OpenTelemetry integration with RAK
//!
//! This example shows how to:
//! - Initialize telemetry with OpenTelemetry
//! - Register custom span processors
//! - Trace LLM calls and tool executions
//!
//! ## Authentication
//!
//! Configure in config.toml (supports both gcloud and API key):
//! ```toml
//! [auth]
//! provider = "gcloud"  # or "api_key"
//! ```
//!
//! Run with:
//! ```bash
//! RUST_LOG=debug cargo run --example telemetry_usage
//! ```

use zdk_agent::LLMAgent;
use zdk_core::{AuthCredentials, Content, Part, RakConfig};
use zdk_model::GeminiModel;
use zdk_runner::Runner;
use zdk_session::inmemory::InMemorySessionService;
use zdk_telemetry::init_telemetry;
use zdk_tool::builtin::{create_calculator_tool, create_echo_tool};
use futures::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry with OpenTelemetry support
    // This sets up:
    // - OpenTelemetry tracer
    // - Structured logging with tracing
    // - Automatic span creation for LLM calls and tool executions
    init_telemetry();

    println!("RAK Telemetry Example");
    println!("==========================\n");
    println!("Watch the logs for structured tracing output!\n");

    // Load configuration
    let config = RakConfig::load()?;
    
    // Get authentication credentials from config
    let creds = config.get_auth_credentials()?;
    
    // Create LLM model based on auth type
    let model: Arc<GeminiModel> = match creds {
        AuthCredentials::ApiKey { key } => {
            println!("✓ Using API Key authentication\n");
            Arc::new(GeminiModel::new(key, config.model.model_name.clone()))
        }
        AuthCredentials::GCloud { token, project, location, .. } => {
            println!("✓ Using Google Cloud authentication");
            println!("  Project: {}", project);
            println!("  Location: {}\n", location);
            Arc::new(GeminiModel::with_bearer_token(
                token,
                config.model.model_name.clone(),
                project,
                location,
            ))
        }
    };

    // Create agent with tools
    let calculator = create_calculator_tool()?;
    let echo = create_echo_tool()?;

    let agent = LLMAgent::builder()
        .name("telemetry_demo")
        .description("Agent demonstrating telemetry integration")
        .model(model)
        .system_instruction("You are a helpful assistant with access to calculator and echo tools.")
        .tool(Arc::new(calculator))
        .tool(Arc::new(echo))
        .build()?;

    // Create session service and runner
    let session_service = Arc::new(InMemorySessionService::new());
    let runner = Runner::builder()
        .app_name("telemetry-example")
        .agent(Arc::new(agent))
        .session_service(session_service)
        .build()?;

    // Run a query that will trigger tool usage
    let user_id = "demo-user";
    let session_id = "demo-session";
    let message = Content::new_user_text("Calculate 15 * 7 and then echo the result back to me");

    if let Some(Part::Text { text }) = message.parts.first() {
        println!("User: {}", text);
    }
    println!("\nProcessing (check logs for detailed tracing)...\n");

    let mut stream = runner
        .run(
            user_id.to_string(),
            session_id.to_string(),
            message,
            Default::default(),
        )
        .await?;

    // Collect response
    let mut full_response = String::new();
    while let Some(event_result) = stream.next().await {
        let event = event_result?;

        if let Some(content) = &event.content {
            for part in &content.parts {
                if let Part::Text { text } = part {
                    if !text.is_empty() && !event.partial {
                        full_response.push_str(text);
                    }
                }
            }
        }
    }

    println!("Assistant: {}", full_response);
    println!("\n✅ Example complete!");
    println!("\nKey telemetry features demonstrated:");
    println!("  • Automatic LLM call tracing");
    println!("  • Tool execution tracing");
    println!("  • Structured log fields (invocation_id, session_id, etc.)");
    println!("  • OpenTelemetry integration");
    println!("\nCheck the logs above to see the structured tracing output!");

    Ok(())
}

