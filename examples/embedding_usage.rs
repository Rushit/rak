//! Example demonstrating embedding capabilities
//!
//! This example shows how to:
//! - Generate embeddings for text
//! - Calculate semantic similarity
//! - Find similar documents
//! - Batch process embeddings
//!
//! Setup:
//! Configure authentication in config.toml (see config.toml.example)
//!
//! Run with:
//! ```bash
//! cargo run --example embedding_usage
//! ```

use zdk_core::{Capability, ZConfig, ZConfigExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔢 ZDK Embedding Usage Example");
    println!("===============================\n");

    // Load configuration
    let config = ZConfig::load()?;

    // Create provider
    let provider = config.create_provider()?;

    // Check if provider supports embeddings
    if !provider.supports(Capability::Embedding) {
        eprintln!("❌ Provider does not support embeddings");
        eprintln!("   Please use a provider with embedding support (e.g., Gemini, OpenAI)");
        std::process::exit(1);
    }

    println!("✓ Provider: {}", provider.metadata().display_name);

    if let Some(dims) = provider.embedding_dimensions() {
        println!("✓ Embedding dimensions: {}", dims);
    }

    if let Some(batch_size) = provider.max_embedding_batch_size() {
        println!("✓ Max batch size: {}", batch_size);
    }

    println!();

    // ============================================================================
    // Example 1: Basic embedding generation
    // ============================================================================
    println!("📝 Example 1: Basic Embedding Generation");
    println!("─────────────────────────────────────────");

    let text = "Rust is a systems programming language focused on safety and performance.";
    println!("\nText: \"{}\"", text);

    let embeddings = provider.embed_texts(vec![text.to_string()]).await?;
    let embedding = &embeddings[0];

    println!("✓ Generated embedding:");
    println!("  Dimensions: {}", embedding.dimensions);
    println!("  First 10 values: {:?}", &embedding.vector[..10]);
    println!();

    // ============================================================================
    // Example 2: Semantic similarity
    // ============================================================================
    println!("\n🔍 Example 2: Semantic Similarity");
    println!("──────────────────────────────────");

    let query = "programming languages for systems development";
    let documents = vec![
        "Rust is a systems programming language.",
        "Python is a high-level interpreted language.",
        "C++ is used for systems and application development.",
        "JavaScript runs in web browsers.",
        "Go is designed for concurrent programming.",
    ];

    println!("\nQuery: \"{}\"", query);
    println!("Documents:");
    for (i, doc) in documents.iter().enumerate() {
        println!("  {}. {}", i + 1, doc);
    }

    // Embed query and documents
    let mut texts = vec![query.to_string()];
    texts.extend(documents.iter().map(|s| s.to_string()));

    println!("\n⏳ Generating embeddings...");
    let embeddings = provider.embed_texts(texts).await?;

    let query_embedding = &embeddings[0];
    let doc_embeddings = &embeddings[1..];

    // Calculate similarities
    println!("✓ Calculating similarities...\n");
    let mut similarities: Vec<(usize, f32)> = doc_embeddings
        .iter()
        .enumerate()
        .map(|(i, emb)| {
            let sim = cosine_similarity(&query_embedding.vector, &emb.vector);
            (i, sim)
        })
        .collect();

    // Sort by similarity (descending)
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Results (ranked by similarity):");
    for (rank, (doc_idx, similarity)) in similarities.iter().enumerate() {
        println!(
            "  {}. [Score: {:.4}] {}",
            rank + 1,
            similarity,
            documents[*doc_idx]
        );
    }

    // ============================================================================
    // Example 3: Batch processing
    // ============================================================================
    println!("\n\n📦 Example 3: Batch Processing");
    println!("───────────────────────────────");

    let large_corpus = vec![
        "Machine learning models learn from data.",
        "Deep learning uses neural networks.",
        "Natural language processing handles text.",
        "Computer vision processes images.",
        "Reinforcement learning learns through trial and error.",
        "Supervised learning uses labeled data.",
        "Unsupervised learning finds patterns in data.",
        "Transfer learning reuses pre-trained models.",
    ];

    println!("\nProcessing {} documents...", large_corpus.len());

    let texts: Vec<String> = large_corpus.iter().map(|s| s.to_string()).collect();
    let batch_embeddings = provider.embed_texts(texts).await?;

    println!("✓ Generated {} embeddings", batch_embeddings.len());

    // Show statistics
    let avg_magnitude: f32 = batch_embeddings
        .iter()
        .map(|emb| emb.vector.iter().map(|x| x * x).sum::<f32>().sqrt())
        .sum::<f32>()
        / batch_embeddings.len() as f32;

    println!("  Average vector magnitude: {:.4}", avg_magnitude);

    // ============================================================================
    // Summary
    // ============================================================================
    println!("\n\n✅ Summary");
    println!("───────────");
    println!(
        "✓ Generated embeddings for {} texts",
        batch_embeddings.len() + 2
    );
    println!("✓ Performed semantic similarity search");
    println!("✓ Demonstrated batch processing");

    // Validation
    if batch_embeddings.is_empty() {
        eprintln!("❌ VALIDATION FAILED: No embeddings generated");
        std::process::exit(1);
    }

    for emb in &batch_embeddings {
        if emb.vector.is_empty() {
            eprintln!("❌ VALIDATION FAILED: Empty embedding vector");
            std::process::exit(1);
        }
    }

    println!("\n✅ VALIDATION PASSED: Embedding generation verified");

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
