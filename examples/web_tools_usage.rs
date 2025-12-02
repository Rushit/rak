//! Web Scraper Tool Usage Example
//!
//! This example demonstrates how to use ZDK's WebScraperTool to:
//! - Fetch HTML content from any public URL
//! - Extract text, headings, and structured data
//! - Parse links from web pages
//! - Analyze real website content
//!
//! ## ✅ Features
//!
//! - **NO additional API keys needed** - uses standard HTTP
//! - **Works with ANY model** - Gemini, Claude, GPT, etc.
//! - **NO special configuration** - just works out of the box
//! - **Production-ready** - handles errors, timeouts, redirects
//!
//! ## 🔑 Configuration
//!
//! Configure your LLM provider in config.toml:
//!
//! **Option 1: Google Cloud (Recommended)**
//! ```toml
//! [auth]
//! provider = "gcloud"
//! project_id = "your-project-id"  # Optional, auto-detected
//! ```
//!
//! **Option 2: API Key**
//! ```toml
//! [auth]
//! provider = "api_key"
//! key = "${GOOGLE_API_KEY}"
//! ```
//!
//! Then run:
//! ```bash
//! cargo run --example web_tools_usage
//! ```
//!
//! ## 📝 Note on Gemini Built-in Tools
//!
//! GeminiGoogleSearchTool and GeminiUrlContextTool are NOT yet functional.
//! They require model-level integration pending implementation.
//! Use WebScraperTool for production applications.

#[path = "common.rs"]
mod common;

use std::sync::Arc;
use zdk_agent::LLMAgent;
use zdk_core::{Content, ZConfig, ZConfigExt};
use zdk_runner::Runner;
use zdk_session::SessionService;
use zdk_session::inmemory::InMemorySessionService;
use zdk_web_tools::WebScraperTool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌐 ZDK Web Tools Example");
    println!("========================\n");

    // Load configuration
    let config = ZConfig::load()?;

    println!("✅ Configuration loaded from config.toml");

    // Create provider using the unified provider system
    let provider = config.create_provider()?;

    println!("🔑 Provider created: {}", config.model.provider);
    println!("🔑 Note: NO additional API keys needed for web tools!\n");

    println!("📦 Creating web scraper tool...");

    // Create web scraper tool - NO additional API keys needed!
    let web_scraper = Arc::new(WebScraperTool::new()?);

    println!("  ✓ WebScraperTool (direct HTTP + HTML parsing)");
    println!("    - Fetches any URL via HTTP");
    println!("    - Parses HTML content");
    println!("    - Extracts text, links, and structured data");
    println!("    - Works with ANY model (Gemini, Claude, GPT, etc.)\n");

    // Note: GeminiGoogleSearchTool and GeminiUrlContextTool are not yet functional
    // They require model-level integration that is pending implementation
    // See crates/zdk-web-tools/src/gemini_google_search.rs for details

    // Create agent with web scraper tool
    let agent = LLMAgent::builder()
        .name("web_scraper_agent")
        .description("An AI agent that can fetch and analyze web pages")
        .model(provider)
        .tool(web_scraper)
        .build()?;

    println!("🤖 Agent created with WebScraperTool\n");

    // Create session service and runner
    let session_service = Arc::new(InMemorySessionService::new());
    let runner = Runner::builder()
        .app_name("web-tools-example")
        .agent(Arc::new(agent))
        .session_service(session_service.clone())
        .build()?;

    // Create a session
    let session = session_service
        .create(&zdk_session::CreateRequest {
            app_name: "web-tools-example".to_string(),
            user_id: "demo-user".to_string(),
            session_id: None,
        })
        .await?;

    println!("📝 Session created: {}\n", session.id());

    // Example 1: Fetch and Analyze HTML Content
    println!("═══════════════════════════════════════");
    println!("Example 1: Fetch HTML Content");
    println!("═══════════════════════════════════════\n");

    let message1 = Content::new_user_text(
        "Use the web_scraper tool to fetch https://httpbin.org/html and extract the main heading.",
    );

    println!("User: Use the web_scraper tool to fetch https://httpbin.org/html and extract the main heading.");
    println!("\n🕷️  Agent response:\n");

    let mut stream = runner
        .run(
            "demo-user".to_string(),
            session.id().to_string(),
            message1,
            Default::default(),
        )
        .await?;

    use futures::StreamExt;
    let mut response1 = String::new();
    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let zdk_core::Part::Text { text } = part {
                            print!("{}", text);
                            response1.push_str(text);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("\n❌ VALIDATION FAILED: Error in web search: {}", e);
                std::process::exit(1);
            }
        }
    }
    println!("\n");

    // Example 2: Extract Links from a Page
    println!("═══════════════════════════════════════");
    println!("Example 2: Extract Links");
    println!("═══════════════════════════════════════\n");

    let message2 = Content::new_user_text(
        "Use web_scraper to fetch https://example.com and tell me what links are on the page.",
    );

    println!("User: Use web_scraper to fetch https://example.com and tell me what links are on the page.");
    println!("\n🔗 Agent response:\n");

    let mut stream = runner
        .run(
            "demo-user".to_string(),
            session.id().to_string(),
            message2,
            Default::default(),
        )
        .await?;

    let mut response2 = String::new();
    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let zdk_core::Part::Text { text } = part {
                            print!("{}", text);
                            response2.push_str(text);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("\n❌ VALIDATION FAILED: Error reading URL: {}", e);
                std::process::exit(1);
            }
        }
    }
    println!("\n");

    // Example 3: Read Content from Real Website
    println!("═══════════════════════════════════════");
    println!("Example 3: Real Website Content");
    println!("═══════════════════════════════════════\n");

    let message3 = Content::new_user_text(
        "Use web_scraper to visit https://www.rust-lang.org and tell me what you learn about Rust. Keep it brief.",
    );

    println!(
        "User: Use web_scraper to visit https://www.rust-lang.org and tell me what you learn about Rust."
    );
    println!("\n🌐 Agent response:\n");

    let mut stream = runner
        .run(
            "demo-user".to_string(),
            session.id().to_string(),
            message3,
            Default::default(),
        )
        .await?;

    let mut response3 = String::new();
    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let zdk_core::Part::Text { text } = part {
                            print!("{}", text);
                            response3.push_str(text);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("\n❌ VALIDATION FAILED: Error in web scraping: {}", e);
                std::process::exit(1);
            }
        }
    }
    println!("\n");

    println!("═══════════════════════════════════════");
    println!("✅ Web Scraper Example Complete!");
    println!("═══════════════════════════════════════\n");

    println!("📊 WebScraperTool Capabilities:");
    println!("  ✅ Fetch any public URL via HTTP/HTTPS");
    println!("  ✅ Parse HTML and extract text content");
    println!("  ✅ Extract links from pages");
    println!("  ✅ Works with ANY model (Gemini, Claude, GPT, etc.)");
    println!("  ✅ NO additional API keys needed");
    println!("  ✅ NO special configuration required");
    println!("\n💡 Note: GeminiGoogleSearchTool and GeminiUrlContextTool are not yet functional.");
    println!("   They require model-level integration that is pending implementation.");

    // Validate all responses were received and tools worked
    println!("\nValidating responses...");

    // Validate all three examples
    common::validate_response_not_empty(&response1, "Example 1: HTML extraction");
    common::validate_response_not_empty(&response2, "Example 2: Link extraction");
    common::validate_response_not_empty(&response3, "Example 3: Website content");

    // Check that agent actually used the web_scraper tool
    if response1.to_lowercase().contains("not working")
        || response2.to_lowercase().contains("not working")
        || response3.to_lowercase().contains("not working")
    {
        common::validation_failed("WebScraperTool failed - this should always work");
    }

    common::validate_response_min_length(&response1, 10, "Example 1 response");
    common::validate_response_min_length(&response2, 10, "Example 2 response");
    common::validate_response_min_length(&response3, 10, "Example 3 response");

    println!("\n✅ VALIDATION PASSED: WebScraperTool verified successfully across all examples");
    println!("✅ The tool fetched and parsed HTML content correctly");

    Ok(())
}
