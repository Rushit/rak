use futures::StreamExt;
use std::sync::Arc;
use zdk_agent::LLMAgent;
use zdk_core::Content;
use zdk_runner::Runner;
use zdk_session::{SessionService, inmemory::InMemorySessionService};

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for structured logging
    // Set RUST_LOG env var to control log level, e.g.:
    // RUST_LOG=debug cargo run --example quickstart
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    common::print_header("ZDK Quickstart Example");

    // Load configuration (drives authentication method)
    println!("Loading configuration...");
    let config = common::load_config()?;

    // Create authenticated Gemini model (auth method from config!)
    println!("Creating Gemini model...");
    let model = common::create_gemini_model(&config)?;

    // Create agent
    println!("Creating LLM agent...");
    let agent = LLMAgent::builder()
        .name("assistant")
        .description("A helpful AI assistant")
        .model(model)
        .system_instruction("You are a helpful AI assistant.")
        .build()?;

    // Create services
    println!("Initializing session service...");
    let session_service = Arc::new(InMemorySessionService::new());

    // Create runner
    println!("Creating runner...");
    let runner = Runner::builder()
        .app_name("quickstart")
        .agent(Arc::new(agent))
        .session_service(session_service.clone())
        .build()?;

    // Create session
    println!("Creating session...\n");
    let session = session_service
        .create(&zdk_session::CreateRequest {
            app_name: "quickstart".into(),
            user_id: "user123".into(),
            session_id: None,
        })
        .await?;

    println!("Session created: {}\n", session.id());

    // Run agent
    let message = Content::new_user_text("Hello! Can you explain what ZDK is in one sentence?");
    println!("User: Hello! Can you explain what ZDK is in one sentence?\n");

    let mut stream = runner
        .run(
            "user123".into(),
            session.id().into(),
            message,
            Default::default(),
        )
        .await?;

    // Print responses and validate
    print!("Agent: ");
    let response = common::collect_and_print_response(&mut stream, "agent query").await?;

    // Validate response quality
    common::validate_response_not_empty(&response, "agent response");
    common::validate_response_min_length(&response, 20, "explanation");

    common::validation_passed("Agent responded successfully");
    println!("Done!");

    Ok(())
}
