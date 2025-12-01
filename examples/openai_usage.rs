use futures::StreamExt;
use std::sync::Arc;
use zdk_agent::LLMAgent;
use zdk_core::{Content, ZConfig, ZConfigExt};
use zdk_runner::Runner;
use zdk_session::{SessionService, inmemory::InMemorySessionService};

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    common::print_header("ZDK OpenAI Example");

    // Load configuration
    println!("Loading configuration...");
    let config = ZConfig::load()?;

    // Create OpenAI provider using the unified provider system
    // The provider will automatically use the credentials from config
    println!("Creating OpenAI provider...");
    let provider = config.create_provider()?;

    // Create agent
    println!("Creating LLM agent...");
    let agent = LLMAgent::builder()
        .name("openai-assistant")
        .description("An AI assistant powered by OpenAI")
        .model(provider)
        .system_instruction("You are a helpful AI assistant powered by OpenAI.")
        .build()?;

    // Create services
    println!("Initializing session service...");
    let session_service = Arc::new(InMemorySessionService::new());

    // Create runner
    println!("Creating runner...");
    let runner = Runner::builder()
        .app_name("openai_example")
        .agent(Arc::new(agent))
        .session_service(session_service.clone())
        .build()?;

    // Create session
    println!("Creating session...\n");
    let session = session_service
        .create(&zdk_session::CreateRequest {
            app_name: "openai_example".into(),
            user_id: "user123".into(),
            session_id: None,
        })
        .await?;

    println!("Session created: {}\n", session.id());

    // Example 1: Simple question
    println!("Example 1: Simple Question");
    println!("==========================");
    let message = Content::new_user_text("What is Rust programming language in one sentence?");
    println!("User: What is Rust programming language in one sentence?\n");

    let mut stream = runner
        .run(
            "user123".into(),
            session.id().into(),
            message,
            Default::default(),
        )
        .await?;

    print!("Assistant: ");
    let response1 = common::collect_and_print_response(&mut stream, "first conversation").await?;

    // Example 2: Follow-up question (tests conversation history)
    println!("Example 2: Follow-up Question");
    println!("==============================");
    let message2 = Content::new_user_text("What are its main benefits?");
    println!("User: What are its main benefits?\n");

    let mut stream2 = runner
        .run(
            "user123".into(),
            session.id().into(),
            message2,
            Default::default(),
        )
        .await?;

    print!("Assistant: ");
    let response2 = common::collect_and_print_response(&mut stream2, "second conversation").await?;

    // Validate both responses
    common::validate_response_not_empty(&response1, "first response");
    common::validate_response_not_empty(&response2, "follow-up response");

    common::validation_passed("OpenAI integration verified");
    println!("Done! OpenAI integration is working correctly.");

    Ok(())
}


