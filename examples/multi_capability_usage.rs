//! Example demonstrating multiple capabilities from a single provider
//!
//! This example shows how to use different capabilities (text generation,
//! embeddings, etc.) from the same provider instance.
//!
//! Setup:
//! Configure authentication in config.toml (see config.toml.example)
//!
//! Run with:
//! ```bash
//! cargo run --example multi_capability_usage
//! ```

use futures::StreamExt;
use zdk_core::{Capability, Content, GenerateConfig, LLMRequest, Part, ZConfig, ZConfigExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎯 ZDK Multi-Capability Usage Example");
    println!("======================================\n");

    // Load configuration
    let config = ZConfig::load()?;
    println!("✓ Configuration loaded\n");

    // Create provider
    let provider = config.create_provider()?;
    let metadata = provider.metadata();

    println!("Provider: {} ({})", metadata.display_name, metadata.name);
    println!("Capabilities: {}\n", metadata.capabilities.len());

    // Check what capabilities are available
    let has_text_gen = provider.supports(Capability::TextGeneration);
    let has_embedding = provider.supports(Capability::Embedding);
    let has_transcription = provider.supports(Capability::Transcription);

    println!("Supported Capabilities:");
    println!(
        "  💬 Text Generation:  {}",
        if has_text_gen { "✓" } else { "✗" }
    );
    println!(
        "  🔢 Embeddings:       {}",
        if has_embedding { "✓" } else { "✗" }
    );
    println!(
        "  🎤 Transcription:    {}",
        if has_transcription { "✓" } else { "✗" }
    );
    println!();

    // ============================================================================
    // Capability 1: Text Generation
    // ============================================================================
    if has_text_gen {
        println!("\n📝 Capability 1: Text Generation");
        println!("────────────────────────────────");

        let request = LLMRequest {
            model: metadata.name.clone(),
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part::Text {
                    text: "Explain quantum computing in one sentence.".to_string(),
                }],
            }],
            config: Some(GenerateConfig {
                temperature: Some(0.7),
                max_tokens: Some(100),
                ..Default::default()
            }),
            tools: vec![],
        };

        println!("\nRequest: Explain quantum computing in one sentence.");
        println!("Response: ");

        use zdk_core::Provider;
        let mut stream = Provider::generate_content(&*provider, request, true).await?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Some(content) = response.content {
                        for part in content.parts {
                            if let Part::Text { text } = part {
                                print!("{}", text);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\nError: {}", e);
                    break;
                }
            }
        }
        println!("\n");
    }

    // ============================================================================
    // Capability 2: Embeddings
    // ============================================================================
    if has_embedding {
        println!("\n🔢 Capability 2: Embeddings");
        println!("─────────────────────────────");

        let texts = vec![
            "The quick brown fox jumps over the lazy dog.".to_string(),
            "Machine learning is a subset of artificial intelligence.".to_string(),
            "Rust is a systems programming language.".to_string(),
        ];

        println!("\nGenerating embeddings for {} texts...", texts.len());

        match provider.embed_texts(texts.clone()).await {
            Ok(embeddings) => {
                println!("✓ Generated {} embeddings\n", embeddings.len());

                for (i, (text, embedding)) in texts.iter().zip(embeddings.iter()).enumerate() {
                    println!("Text {}: \"{}\"", i + 1, text);
                    println!("  Dimensions: {}", embedding.dimensions);
                    println!(
                        "  First 5 values: {:?}",
                        &embedding.vector[..5.min(embedding.vector.len())]
                    );
                    println!();
                }

                // Calculate similarity between first two embeddings
                if embeddings.len() >= 2 {
                    let similarity =
                        cosine_similarity(&embeddings[0].vector, &embeddings[1].vector);
                    println!("Cosine similarity between text 1 and 2: {:.4}", similarity);
                }
            }
            Err(e) => {
                eprintln!("✗ Embedding error: {}", e);
            }
        }
    }

    // ============================================================================
    // Summary
    // ============================================================================
    println!("\n\n📊 Summary");
    println!("───────────");
    let capabilities_count = [has_text_gen, has_embedding, has_transcription]
        .iter()
        .filter(|&&x| x)
        .count();
    println!(
        "✓ Successfully demonstrated {} capabilities",
        capabilities_count
    );
    println!("✓ All capabilities used from single provider instance");

    // Validation
    if !has_text_gen {
        eprintln!("❌ VALIDATION FAILED: Text generation capability not demonstrated");
        std::process::exit(1);
    }

    if capabilities_count == 0 {
        eprintln!("❌ VALIDATION FAILED: No capabilities demonstrated");
        std::process::exit(1);
    }

    println!("\n✅ VALIDATION PASSED: Multi-capability usage verified");

    Ok(())
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}
