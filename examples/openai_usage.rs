use rak_agent::LLMAgent;
use rak_core::{Content, RakConfig};
use rak_model::OpenAIModel;
use rak_runner::Runner;
use rak_session::{inmemory::InMemorySessionService, SessionService};
use futures::StreamExt;
use std::sync::Arc;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    common::print_header("RAK OpenAI Example");

    // Load configuration
    println!("Loading configuration...");
    let config = RakConfig::load()?;
    
    // Get OpenAI API key from config
    // You can set it in config.toml:
    //   openai_api_key = "sk-..."
    // Or via environment variable:
    //   export OPENAI_API_KEY="sk-..."
    let openai_api_key = config
        .openai_api_key
        .ok_or_else(|| anyhow::anyhow!("OpenAI API key not found. Set openai_api_key in config.toml or OPENAI_API_KEY environment variable"))?;

    // Create OpenAI model
    println!("Creating OpenAI model...");
    let model = Arc::new(OpenAIModel::new(
        openai_api_key,
        "gpt-4o-mini".to_string(), // or gpt-4o, gpt-3.5-turbo, etc.
    ));

    // Create agent
    println!("Creating LLM agent...");
    let agent = LLMAgent::builder()
        .name("openai-assistant")
        .description("An AI assistant powered by OpenAI")
        .model(model)
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
        .create(&rak_session::CreateRequest {
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
    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let rak_core::Part::Text { text } = part {
                            print!("{}", text);
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                        }
                    }
                }
                if event.is_final_response() {
                    println!("\n");
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(e.into());
            }
        }
    }

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
    while let Some(event_result) = stream2.next().await {
        match event_result {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let rak_core::Part::Text { text } = part {
                            print!("{}", text);
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                        }
                    }
                }
                if event.is_final_response() {
                    println!("\n");
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(e.into());
            }
        }
    }

    println!("Done! OpenAI integration is working correctly.");

    Ok(())
}

