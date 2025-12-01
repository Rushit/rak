//! Example demonstrating config-driven gcloud authentication.
//!
//! This example shows how to use Gemini via Vertex AI with gcloud auth
//! configured through config.toml. This is the RECOMMENDED approach for
//! production deployments.
//!
//! ## Setup
//!
//! 1. Install and authenticate with gcloud:
//!    ```bash
//!    gcloud auth login
//!    ```
//!
//! 2. Set default project:
//!    ```bash
//!    gcloud config set project YOUR_PROJECT_ID
//!    ```
//!
//! 3. Enable Vertex AI API in your GCP project
//!
//! 4. Configure config.toml:
//!    ```toml
//!    [auth]
//!    provider = "gcloud"
//!    
//!    [auth.gcloud]
//!    # Optional: project_id = "my-project" (auto-detected if not set)
//!    # Optional: location = "us-central1" (default if not set)
//!    ```
//!
//! ## Run
//!
//! ```bash
//! cargo run --example gemini_gcloud_usage
//! ```
//!
//! ## Benefits of Config-Driven Auth
//!
//! - No hardcoded credentials in code
//! - Easy to switch between environments (dev/staging/prod)
//! - Automatic token refresh
//! - Consistent auth pattern across all examples
//! - Production-ready

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
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    common::print_header("Gemini with Config-Driven gcloud Auth");

    println!("This example demonstrates using gcloud authentication via config.toml.");
    println!("Make sure your config.toml has:\n");
    println!("  [auth]");
    println!("  provider = \"gcloud\"\n");

    // Load configuration (should have gcloud auth configured)
    println!("Loading configuration...");
    let config = common::load_config()?;

    // Show detailed auth info
    common::show_auth_info(&config)?;
    println!();

    // Create authenticated Gemini model
    // This uses gcloud CLI to fetch fresh tokens automatically!
    println!("Creating Gemini model with gcloud auth...");
    let model = common::create_gemini_model(&config)?;

    // Create agent
    println!("Creating LLM agent...");
    let agent = LLMAgent::builder()
        .name("gemini-assistant")
        .description("An AI assistant powered by Gemini via Vertex AI")
        .model(model)
        .system_instruction("You are a helpful AI assistant.")
        .build()?;

    // Create services
    println!("Initializing session service...");
    let session_service = Arc::new(InMemorySessionService::new());

    // Create runner
    println!("Creating runner...");
    let runner = Runner::builder()
        .app_name("gemini_gcloud_example")
        .agent(Arc::new(agent))
        .session_service(session_service.clone())
        .build()?;

    // Create session
    println!("Creating session...\n");
    let session = session_service
        .create(&zdk_session::CreateRequest {
            app_name: "gemini_gcloud_example".into(),
            user_id: "user123".into(),
            session_id: None,
        })
        .await?;

    println!("Session created: {}\n", session.id());

    // Example interaction
    println!("Example: Simple Question");
    println!("========================");
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
                        if let zdk_core::Part::Text { text } = part {
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

    println!("Done! Gemini with gcloud auth is working correctly.");
    println!("\nNote: Access tokens expire after 1 hour. For production, implement token refresh.");

    Ok(())
}
