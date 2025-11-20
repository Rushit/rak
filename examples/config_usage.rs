//! Example demonstrating the new configuration system
//!
//! This example shows:
//! 1. Loading configuration from config.toml
//! 2. Priority: config.toml > environment variables > defaults
//! 3. Different config files for different environments
//!
//! Setup:
//! ```bash
//! # Copy the example config
//! cp config.toml.example config.toml
//!
//! # Edit with your API key
//! vim config.toml
//!
//! # Run the example
//! cargo run --example config_usage
//! ```

use anyhow::Result;
use rak_core::RakConfig;
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== RAK Configuration System Demo ===\n");

    // Example 1: Load default config (config.toml)
    println!("1️⃣  Loading default configuration...");
    match RakConfig::load() {
        Ok(config) => {
            println!("   ✅ Config loaded successfully!");
            println!("   📁 Provider: {}", config.model.provider);
            println!("   🤖 Model: {}", config.model.model_name);
            println!("   🔑 API Key: {}", 
                if config.model.api_key.is_some() { "✅ Set" } else { "❌ Not Set" });
            println!("   🌐 Server: {}:{}", config.server.host, config.server.port);
            println!("   💾 Session: {}", config.session.provider);
        }
        Err(e) => {
            println!("   ⚠️  Could not load config.toml: {}", e);
            println!("   💡 This is expected if you haven't created config.toml yet");
            println!("   💡 Run: cp config.toml.example config.toml");
        }
    }

    println!("\n2️⃣  Loading test configuration...");
    match RakConfig::load_test() {
        Ok(config) => {
            println!("   ✅ Test config loaded!");
            println!("   📁 Provider: {}", config.model.provider);
            println!("   🤖 Model: {}", config.model.model_name);
            println!("   🔑 API Key: {}", config.model.api_key.unwrap_or_default());
        }
        Err(e) => {
            println!("   ⚠️  Could not load test config: {}", e);
        }
    }

    // Example 3: Show priority demonstration
    println!("\n3️⃣  Configuration Priority Demo:");
    println!("   Priority: config.toml > environment variables > defaults\n");

    // Check environment variable
    if let Ok(env_key) = env::var("GEMINI_API_KEY") {
        println!("   🌍 GEMINI_API_KEY env var: Set (length: {})", env_key.len());
    } else {
        println!("   🌍 GEMINI_API_KEY env var: Not set");
    }

    // Try to load config
    if let Ok(config) = RakConfig::load() {
        match config.api_key() {
            Ok(key) => {
                println!("   📝 Resolved API key: Set (length: {})", key.len());
                println!("   ✅ Priority worked! Using value from config or env");
            }
            Err(_) => {
                println!("   ❌ No API key found in config or env");
            }
        }
    }

    // Example 4: Different config files
    println!("\n4️⃣  Environment-Specific Configs:");
    println!("   You can use different configs for different environments:");
    println!("   • config.toml        → Development (your local work)");
    println!("   • config.test.toml   → Tests (CI/CD)");
    println!("   • config.prod.toml   → Production (deployed service)");
    println!("   • config.staging.toml → Staging environment");

    // Try to load custom config if it exists
    let custom_configs = ["config.prod.toml", "config.dev.toml"];
    for config_file in &custom_configs {
        let path = Path::new(config_file);
        if path.exists() {
            println!("\n   📄 Found custom config: {}", config_file);
            if let Ok(config) = RakConfig::load_from(Some(path)) {
                println!("      ✅ Loaded successfully");
                println!("      🤖 Model: {}", config.model.model_name);
            }
        }
    }

    // Example 5: Configuration reference patterns
    println!("\n5️⃣  Configuration Patterns:");
    println!("   \n   Pattern 1: Direct value (simple, for local dev)");
    println!("   ```toml");
    println!("   [model]");
    println!("   api_key = \"your-actual-key-here\"");
    println!("   ```");

    println!("\n   Pattern 2: Environment variable reference (flexible)");
    println!("   ```toml");
    println!("   [model]");
    println!("   api_key = \"${{GEMINI_API_KEY}}\"  # Resolves from env");
    println!("   ```");

    println!("\n   Pattern 3: No config file (fallback to env vars)");
    println!("   ```bash");
    println!("   export GEMINI_API_KEY=\"your-key\"");
    println!("   cargo run  # Falls back to env vars");
    println!("   ```");

    // Example 6: Security best practices
    println!("\n6️⃣  Security Best Practices:");
    println!("   ✅ DO: Use config.toml for development (in .gitignore)");
    println!("   ✅ DO: Use environment variables in production");
    println!("   ✅ DO: Commit config.toml.example (no real keys)");
    println!("   ✅ DO: Use config.test.toml for tests (fake keys)");
    println!("   ❌ DON'T: Commit config.toml with real keys");
    println!("   ❌ DON'T: Hard-code API keys in source code");

    println!("\n=== Configuration Demo Complete ===");
    println!("\n💡 Next Steps:");
    println!("   1. Create config.toml: cp config.toml.example config.toml");
    println!("   2. Add your API key to config.toml");
    println!("   3. Run examples: cargo run --example quickstart");

    Ok(())
}

