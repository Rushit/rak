//! Example demonstrating OpenTelemetry integration with ZDK
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

use futures::StreamExt;
use std::sync::Arc;
use zdk_agent::LLMAgent;
use zdk_core::{Content, Part, ZConfig, ZConfigExt};
use zdk_runner::Runner;
use zdk_session::inmemory::InMemorySessionService;
use zdk_telemetry::init_telemetry;
use zdk_tool::builtin::{create_calculator_tool, create_echo_tool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry with OpenTelemetry support
    // This sets up:
    // - OpenTelemetry tracer
    // - Structured logging with tracing
    // - Automatic span creation for LLM calls and tool executions
    init_telemetry();

    println!("ZDK Telemetry Example");
    println!("==========================\n");
    println!("Watch the logs for structured tracing output!\n");

    // Load configuration
    let config = ZConfig::load()?;

    // Create provider using the new unified provider system
    let provider = config.create_provider()?;
    
    println!("✓ Provider created: {}", config.model.provider);
    println!("  Model: {}\n", config.model.model_name);

    // Create agent with tools
    let calculator = create_calculator_tool()?;
    let echo = create_echo_tool()?;

    let agent = LLMAgent::builder()
        .name("telemetry_demo")
        .description("Agent demonstrating telemetry integration")
        .model(provider)
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

    // Collect response and validate
    let mut full_response = String::new();
    let mut tool_executed = false;
    
    while let Some(event_result) = stream.next().await {
        let event = event_result?;

        if let Some(content) = &event.content {
            for part in &content.parts {
                match part {
                    Part::Text { text } => {
                        if !text.is_empty() {
                            full_response.push_str(text);
                        }
                    }
                    Part::FunctionCall { function_call } => {
                        if function_call.name == "calculator" || function_call.name == "echo" {
                            tool_executed = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    println!("Assistant: {}", full_response);
    
    // Validate results
    println!("\nValidating example results...");
    
    if !tool_executed {
        eprintln!("❌ VALIDATION FAILED: No tools were executed");
        std::process::exit(1);
    }
    
    if full_response.is_empty() {
        eprintln!("❌ VALIDATION FAILED: No response text received from agent");
        std::process::exit(1);
    }
    
    if !full_response.contains("105") {
        eprintln!("❌ VALIDATION FAILED: Response doesn't contain expected calculation result (105)");
        eprintln!("   Got: '{}'", full_response.trim());
        std::process::exit(1);
    }
    
    println!("✅ VALIDATION PASSED: All checks successful");
    println!("\n✅ Example complete!");
    println!("\nKey telemetry features demonstrated:");
    println!("  • Automatic LLM call tracing");
    println!("  • Tool execution tracing");
    println!("  • Structured log fields (invocation_id, session_id, etc.)");
    println!("  • OpenTelemetry integration");
    println!("\nCheck the logs above to see the structured tracing output!");

    Ok(())
}
