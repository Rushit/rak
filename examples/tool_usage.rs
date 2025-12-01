use zdk_agent::LLMAgent;
use zdk_core::Content;
use zdk_runner::Runner;
use zdk_session::inmemory::InMemorySessionService;
use zdk_tool::builtin::{create_calculator_tool, create_echo_tool};
use futures::StreamExt;
use std::sync::Arc;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();

    common::print_header("RAK Tool Usage Example");

    // Load configuration (drives authentication method)
    println!("Loading configuration...");
    let config = common::load_config()?;

    // Create authenticated Gemini model (auth method from config!)
    println!("Creating Gemini model...");
    let model = common::create_gemini_model(&config)?;

    // Create tools
    let calculator = Arc::new(create_calculator_tool()?);
    let echo = Arc::new(create_echo_tool()?);

    // Create agent with tools
    let agent = Arc::new(
        LLMAgent::builder()
            .name("math_assistant")
            .description("An AI assistant that can perform calculations")
            .model(model)
            .system_instruction("You are a helpful math assistant. When asked to calculate something, use the calculator tool.")
            .tool(calculator)
            .tool(echo)
            .build()?
    );

    // Create session service
    let session_service = Arc::new(InMemorySessionService::new());

    // Create runner
    let runner = Runner::builder()
        .app_name("tool-demo")
        .agent(agent)
        .session_service(session_service)
        .build()?;

    // Run agent
    let message = Content::new_user_text("Calculate 15 * 23 + 100");
    let mut stream = runner
        .run(
            "user-123".to_string(),
            "session-456".to_string(),
            message,
            Default::default(),
        )
        .await?;

    println!("Running agent with tool support...\n");

    // Process stream
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                if let Some(content) = &event.content {
                    println!("Event from {}: role={}", event.author, content.role);
                    for part in &content.parts {
                        match part {
                            zdk_core::Part::Text { text } => {
                                println!("  Text: {}", text);
                            }
                            zdk_core::Part::FunctionCall { function_call } => {
                                println!(
                                    "  Function Call: {} ({})",
                                    function_call.name, 
                                    function_call.id.as_deref().unwrap_or("no-id")
                                );
                                println!("    Args: {}", function_call.args);
                            }
                            zdk_core::Part::FunctionResponse { function_response } => {
                                println!(
                                    "  Function Response: {} ({})",
                                    function_response.name, 
                                    function_response.id.as_deref().unwrap_or("no-id")
                                );
                                println!("    Result: {}", function_response.response);
                            }
                            _ => {
                                println!("  Other part type");
                            }
                        }
                    }
                }

                if !event.error_code.is_empty() {
                    println!("  Error: {} - {}", event.error_code, event.error_message);
                }

                if event.turn_complete {
                    println!("\n✓ Turn complete\n");
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
