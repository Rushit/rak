//! Web Tools Usage Example
//!
//! This example demonstrates how to use ZDK's web tools:
//! - GeminiGoogleSearchTool - Search the web using Gemini's built-in capability
//! - GeminiUrlContextTool - Read web pages using Gemini's built-in capability
//! - WebScraperTool - Parse HTML content from any URL
//!
//! ## 🔑 Authentication
//!
//! **✅ ZERO additional API keys needed for web tools!**
//!
//! Configure authentication in config.toml:
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

use std::sync::Arc;
use zdk_agent::LLMAgent;
use zdk_core::{Content, ZConfig, ZConfigExt};
use zdk_runner::Runner;
use zdk_session::SessionService;
use zdk_session::inmemory::InMemorySessionService;
use zdk_web_tools::{GeminiGoogleSearchTool, GeminiUrlContextTool, WebScraperTool};

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

    println!("📦 Creating web tools...");

    // Create web tools - NO additional API keys needed!
    let google_search = Arc::new(GeminiGoogleSearchTool::new());
    let url_context = Arc::new(GeminiUrlContextTool::new());
    let web_scraper = Arc::new(WebScraperTool::new()?);

    println!("  ✓ GeminiGoogleSearchTool (uses Gemini's built-in search)");
    println!("  ✓ GeminiUrlContextTool (uses Gemini's built-in URL fetching)");
    println!("  ✓ WebScraperTool (direct HTTP + HTML parsing)\n");

    // Create agent with web tools
    let agent = LLMAgent::builder()
        .name("web_research_agent")
        .description("An AI agent that can search the web and read web pages")
        .model(provider)
        .tool(google_search)
        .tool(url_context)
        .tool(web_scraper)
        .build()?;

    println!("🤖 Agent created with 3 web tools\n");

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

    // Example 1: Web Search (Gemini built-in)
    println!("═══════════════════════════════════════");
    println!("Example 1: Web Search (Gemini built-in)");
    println!("═══════════════════════════════════════\n");

    let search_message = Content::new_user_text(
        "What are the latest features in Rust 1.80? Search the web for recent updates.",
    );

    println!("User: What are the latest features in Rust 1.80? Search the web for recent updates.");
    println!("\n🔍 Agent response:\n");

    let mut stream = runner
        .run(
            "demo-user".to_string(),
            session.id().to_string(),
            search_message,
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

    // Example 2: Read URL Content (Gemini built-in)
    println!("═══════════════════════════════════════");
    println!("Example 2: Read URL (Gemini built-in)");
    println!("═══════════════════════════════════════\n");

    let url_message = Content::new_user_text(
        "Read the content from https://www.rust-lang.org and summarize what Rust is.",
    );

    println!("User: Read the content from https://www.rust-lang.org and summarize what Rust is.");
    println!("\n📖 Agent response:\n");

    let mut stream = runner
        .run(
            "demo-user".to_string(),
            session.id().to_string(),
            url_message,
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

    // Example 3: Web Scraping (Universal)
    println!("═══════════════════════════════════════");
    println!("Example 3: Web Scraping (Universal)");
    println!("═══════════════════════════════════════\n");

    let scrape_message = Content::new_user_text(
        "Use the web_scraper tool to fetch https://httpbin.org/html and extract the main heading.",
    );

    println!(
        "User: Use the web_scraper tool to fetch https://httpbin.org/html and extract the main heading."
    );
    println!("\n🕷️  Agent response:\n");

    let mut stream = runner
        .run(
            "demo-user".to_string(),
            session.id().to_string(),
            scrape_message,
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
    println!("✅ Web Tools Example Complete!");
    println!("═══════════════════════════════════════\n");

    println!("📊 Summary:");
    println!("  • GeminiGoogleSearchTool: Searches the web via Gemini API");
    println!("  • GeminiUrlContextTool: Reads URLs via Gemini API");
    println!("  • WebScraperTool: Parses HTML directly (works with any model)");
    println!("\n🔑 NO additional API keys needed - uses your Gemini API key!");

    // Validate all responses were received
    println!("\nValidating responses...");
    
    if response1.trim().is_empty() {
        eprintln!("❌ VALIDATION FAILED: No response from web search example");
        std::process::exit(1);
    }
    
    if response2.trim().is_empty() {
        eprintln!("❌ VALIDATION FAILED: No response from URL reading example");
        std::process::exit(1);
    }
    
    if response3.trim().is_empty() {
        eprintln!("❌ VALIDATION FAILED: No response from web scraping example");
        std::process::exit(1);
    }

    println!("✅ VALIDATION PASSED: All web tools verified with responses");

    Ok(())
}
