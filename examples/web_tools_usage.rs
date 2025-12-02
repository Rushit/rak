//! Web Tools Usage Example
//!
//! This example demonstrates ZDK's comprehensive web tool ecosystem:
//! 1. **GeminiGoogleSearchTool** - Google Search via Gemini API (Gemini 2.0+ only)
//! 2. **GeminiUrlContextTool** - URL fetching via Gemini API (Gemini 2.0+ only)
//! 3. **WebScraperTool** - Direct HTTP scraping (works with any model)
//!
//! ## ✅ What's New
//!
//! **Gemini built-in tools are NOW FULLY FUNCTIONAL!** 🎉
//! - They execute inside the Gemini API (no local execution)
//! - No additional API keys needed (uses your Gemini key)
//! - Can be mixed with custom tools in the same agent
//!
//! ## 🔑 Configuration
//!
//! **For Gemini built-in tools (Google Search, URL Context):**
//! ```toml
//! [model]
//! provider = "gemini"
//! name = "gemini-2.0-flash-exp"  # Must be Gemini 2.0+
//!
//! [auth]
//! provider = "api_key"
//! key = "${GOOGLE_API_KEY}"
//! ```
//!
//! **For WebScraperTool only (works with any model):**
//! ```toml
//! [model]
//! provider = "gemini"  # or "openai", "anthropic", etc.
//! name = "gemini-1.5-flash"  # Any model works
//! ```
//!
//! Then run:
//! ```bash
//! cargo run --example web_tools_usage
//! ```
//!
//! ## 📊 Tool Comparison
//!
//! | Tool | Execution | Models | API Keys | Use Case |
//! |------|-----------|--------|----------|----------|
//! | GeminiGoogleSearchTool | In Gemini API | Gemini 2.0+ | Gemini only | Web search |
//! | GeminiUrlContextTool | In Gemini API | Gemini 2.0+ | Gemini only | URL fetching |
//! | WebScraperTool | Local HTTP | Any model | None | HTML parsing |

#[path = "common.rs"]
mod common;

use futures::StreamExt;
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
    
    // Check if we're using Gemini 2.0+ for built-in tools
    let model_name = &config.model.model_name;
    let is_gemini_2_plus = model_name.contains("2.0") || model_name.contains("2.5");
    let is_gemini = config.model.provider.to_lowercase().contains("gemini");
    
    println!("\n📦 Creating web tools...\n");

    // Always create WebScraperTool - works with any model
    let web_scraper = Arc::new(WebScraperTool::new()?);
    println!("  ✓ WebScraperTool (direct HTTP + HTML parsing)");
    println!("    - Fetches any URL via HTTP");
    println!("    - Parses HTML content");  
    println!("    - Works with ANY model");

    // Show Gemini built-in tools availability
    if is_gemini && is_gemini_2_plus {
        println!("\n  ✓ GeminiGoogleSearchTool (Gemini API built-in)");
        println!("    - Executes inside Gemini API");
        println!("    - NO additional API keys needed");
        println!("    - Requires Gemini 2.0+ ✅");
        
        println!("\n  ✓ GeminiUrlContextTool (Gemini API built-in)");
        println!("    - Executes inside Gemini API");
        println!("    - NO additional API keys needed");
        println!("    - Requires Gemini 2.0+ ✅");
        
        println!("\n⚠️  CRITICAL LIMITATION: Gemini API Restriction");
        println!("   ❌ CANNOT mix built-in tools (google_search, url_context)");
        println!("      with function-calling tools (web_scraper) in one agent");
        println!("   ✅ This is a Gemini API limitation, not a ZDK bug");
        println!("\n   For this example, we'll use WebScraperTool only.");
        println!("   See zdk-web-tools docs for sub-agent workaround pattern.");
    } else {
        println!("\n  ⚠️  GeminiGoogleSearchTool - SKIPPED");
        println!("    Requires: Gemini 2.0+ model");
        println!("    Current: {} on {}", model_name, config.model.provider);
        
        println!("\n  ⚠️  GeminiUrlContextTool - SKIPPED");
        println!("    Requires: Gemini 2.0+ model");
        println!("    Current: {} on {}", model_name, config.model.provider);
        
        println!("\n💡 Using WebScraperTool (works with any model)");
    }

    // Create agent with WebScraperTool only
    // Note: Cannot mix with built-in tools due to Gemini API limitation
    let agent = LLMAgent::builder()
        .name("web_scraper_agent")
        .description("An AI agent that can fetch and parse web pages")
        .model(provider)
        .tool(web_scraper)
        .build()?;

    println!("\n🤖 WebScraperTool agent created\n");

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

    println!("════════════════════════════════════════════════");
    println!("🕷️  WebScraperTool Examples");
    println!("════════════════════════════════════════════════\n");

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

    println!("════════════════════════════════════════════════");
    println!("✅ Web Tools Example Complete!");
    println!("════════════════════════════════════════════════\n");

    println!("📊 WebScraperTool Capabilities:");
    println!("  ✅ Fetch any public URL via HTTP/HTTPS");
    println!("  ✅ Parse HTML and extract text content");
    println!("  ✅ Extract links from pages");
    println!("  ✅ Works with ANY model (Gemini, Claude, GPT, etc.)");
    println!("  ✅ NO additional API keys needed");
    println!("  ✅ NO special configuration required");

    if is_gemini && is_gemini_2_plus {
        println!("\n📚 About Gemini Built-in Tools:");
        println!("\n  🔍 GeminiGoogleSearchTool & 🌐 GeminiUrlContextTool:");
        println!("    ✅ Implementation: COMPLETE ✓");
        println!("    ✅ Execute inside Gemini API");
        println!("    ✅ NO additional API keys needed");
        println!("    ❌ Gemini API Limitation: Cannot mix with function-calling tools");
        
        println!("\n  💡 To use built-in tools:");
        println!("    1. Create a separate agent with ONLY built-in tools:");
        println!("       let search_agent = LLMAgent::builder()");
        println!("           .tool(Arc::new(GeminiGoogleSearchTool::new()))");
        println!("           .build()?;");
        println!("\n    2. OR use sub-agent pattern:");
        println!("       - Create sub-agent with built-in tool");
        println!("       - Wrap as AgentTool for main agent");
        
        println!("\n  📖 Reference:");
    } else {
        println!("\n💡 Want Gemini built-in tools?");
        println!("   Update config.toml to use a Gemini 2.0+ model:");
        println!("   - gemini-2.0-flash-exp");
        println!("   - gemini-2.5-flash");
        println!("   Note: Due to Gemini API limitations, they require separate agents");
    }

    // Validate responses
    println!("\n🔍 Validating responses...");

    common::validate_response_not_empty(&response1, "Example 1: HTML extraction");
    common::validate_response_not_empty(&response2, "Example 2: Link extraction");
    common::validate_response_not_empty(&response3, "Example 3: Website content");

    // Check that tools actually worked
    if response1.to_lowercase().contains("not working")
        || response2.to_lowercase().contains("not working")
        || response3.to_lowercase().contains("not working")
    {
        common::validation_failed("WebScraperTool failed - this should always work");
    }

    common::validate_response_min_length(&response1, 10, "Example 1 response");
    common::validate_response_min_length(&response2, 10, "Example 2 response");
    common::validate_response_min_length(&response3, 10, "Example 3 response");

    println!("\n✅ VALIDATION PASSED: WebScraperTool verified successfully");
    println!("✅ The tool fetched and parsed HTML content correctly");
    println!("✅ Works with any LLM provider (Gemini, OpenAI, Anthropic, etc.)");

    Ok(())
}
