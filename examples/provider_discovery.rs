//! Example demonstrating provider discovery and capability queries
//!
//! This example shows how to:
//! - Discover all available providers
//! - Query provider capabilities
//! - Find providers with specific capabilities
//! - Access provider metadata
//!
//! Run with:
//! ```bash
//! cargo run --example provider_discovery
//! ```

use zdk_core::{Capability, ZConfig, ZConfigExt};

fn main() -> anyhow::Result<()> {
    println!("🔍 ZDK Provider Discovery Example");
    println!("==================================\n");

    // Load configuration
    let config = ZConfig::load()?;

    // Discover all available providers
    println!("📦 Discovering providers...\n");
    let providers = config.discover_providers();

    println!("Found {} provider(s):\n", providers.len());

    for provider_meta in &providers {
        println!("╭─ {} ({})", provider_meta.display_name, provider_meta.name);
        println!("│");
        
        // Show capabilities
        println!("│  Capabilities:");
        for capability in &provider_meta.capabilities {
            let icon = match capability {
                Capability::TextGeneration => "💬",
                Capability::Embedding => "🔢",
                Capability::Transcription => "🎤",
                Capability::ImageGeneration => "🖼️ ",
                Capability::AudioGeneration => "🔊",
            };
            println!("│    {} {:?}", icon, capability);
        }
        
        // Show models
        if !provider_meta.models.is_empty() {
            println!("│");
            println!("│  Models ({}):", provider_meta.models.len());
            for model in &provider_meta.models {
                println!("│    • {}", model.id);
                if let Some(context) = model.context_window {
                    println!("│      Context: {} tokens", format_number(context));
                }
                if let Some(dims) = model.embedding_dimensions {
                    println!("│      Embedding dimensions: {}", dims);
                }
            }
        }
        
        println!("╰─");
        println!();
    }

    // Query by capability
    println!("\n🔎 Querying by capability...\n");

    let capabilities = vec![
        Capability::TextGeneration,
        Capability::Embedding,
        Capability::Transcription,
        Capability::ImageGeneration,
        Capability::AudioGeneration,
    ];

    for capability in capabilities {
        let providers_with_cap = config.find_providers_with(capability);
        
        if !providers_with_cap.is_empty() {
            println!("✓ {:?}:", capability);
            for name in providers_with_cap {
                println!("  - {}", name);
            }
        } else {
            println!("✗ {:?}: No providers found", capability);
        }
    }

    // Create a provider
    println!("\n\n🚀 Creating provider from config...\n");
    let provider = config.create_provider()?;
    let metadata = provider.metadata();
    
    println!("Created: {} ({})", metadata.display_name, metadata.name);
    println!("Supports {} capabilities", metadata.capabilities.len());

    println!("\n✅ Discovery complete!");

    // Validation
    if providers.is_empty() {
        eprintln!("❌ VALIDATION FAILED: No providers discovered");
        std::process::exit(1);
    }
    
    if metadata.capabilities.is_empty() {
        eprintln!("❌ VALIDATION FAILED: Created provider has no capabilities");
        std::process::exit(1);
    }

    println!("✅ VALIDATION PASSED: Provider discovery verified");

    Ok(())
}

/// Format large numbers with commas for readability
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;
    
    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
        count += 1;
    }
    
    result
}

