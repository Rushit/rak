//! Integration tests for the provider system
//!
//! These tests verify that the provider system works correctly end-to-end.

use zdk_core::{Capability, ZConfig, ZConfigExt};

#[test]
fn test_provider_discovery() {
    // Create a minimal test config
    let config_toml = r#"
        [auth]
        provider = "api_key"
        key = "test-key"
        
        [model]
        provider = "gemini"
        model_name = "gemini-1.5-flash"
    "#;

    let config: ZConfig = toml::from_str(config_toml).expect("Failed to parse config");

    // Discover all providers
    let providers = config.discover_providers();
    assert!(providers.len() >= 2, "Should have at least 2 providers");

    // Verify provider names
    let names: Vec<String> = providers.iter().map(|p| p.name.clone()).collect();
    assert!(names.contains(&"gemini".to_string()));
    assert!(names.contains(&"openai".to_string()));

    println!("✓ Discovered {} providers", providers.len());
    for provider in &providers {
        println!("  - {} ({})", provider.display_name, provider.name);
        println!("    Capabilities: {:?}", provider.capabilities);
    }
}

#[test]
fn test_capability_filtering() {
    let config_toml = r#"
        [auth]
        provider = "api_key"
        key = "test-key"
        
        [model]
        provider = "gemini"
        model_name = "gemini-1.5-flash"
    "#;

    let config: ZConfig = toml::from_str(config_toml).expect("Failed to parse config");

    // Find providers with text generation
    let text_gen_providers = config.find_providers_with(Capability::TextGeneration);
    assert!(
        !text_gen_providers.is_empty(),
        "Should find text generation providers"
    );
    assert!(text_gen_providers.contains(&"gemini".to_string()));
    println!("✓ Text generation providers: {:?}", text_gen_providers);

    // Find providers with embedding
    let embedding_providers = config.find_providers_with(Capability::Embedding);
    assert!(
        !embedding_providers.is_empty(),
        "Should find embedding providers"
    );
    println!("✓ Embedding providers: {:?}", embedding_providers);

    // Find providers with transcription
    let transcription_providers = config.find_providers_with(Capability::Transcription);
    println!("✓ Transcription providers: {:?}", transcription_providers);
}

#[test]
fn test_gemini_provider_creation() {
    let config_toml = r#"
        [auth]
        provider = "api_key"
        key = "test-api-key-12345"
        
        [model]
        provider = "gemini"
        model_name = "gemini-1.5-flash"
    "#;

    let config: ZConfig = toml::from_str(config_toml).expect("Failed to parse config");

    // Create provider
    let provider = config
        .create_provider()
        .expect("Should create Gemini provider");

    // Check metadata
    let metadata = provider.metadata();
    assert_eq!(metadata.name, "gemini");
    assert!(metadata.capabilities.contains(&Capability::TextGeneration));
    assert!(metadata.capabilities.contains(&Capability::Embedding));

    println!("✓ Created Gemini provider");
    println!("  Name: {}", metadata.display_name);
    println!("  Capabilities: {:?}", metadata.capabilities);
}

#[test]
fn test_openai_provider_creation() {
    let config_toml = r#"
        openai_api_key = "sk-test-key-12345"
        
        [auth]
        provider = "api_key"
        key = "test-key"
        
        [model]
        provider = "openai"
        model_name = "gpt-4o"
    "#;

    let config: ZConfig = toml::from_str(config_toml).expect("Failed to parse config");

    // Create provider
    let provider = config
        .create_provider()
        .expect("Should create OpenAI provider");

    // Check metadata
    let metadata = provider.metadata();
    assert_eq!(metadata.name, "openai");
    assert!(metadata.capabilities.contains(&Capability::TextGeneration));
    assert!(metadata.capabilities.contains(&Capability::Embedding));
    assert!(metadata.capabilities.contains(&Capability::Transcription));

    println!("✓ Created OpenAI provider");
    println!("  Name: {}", metadata.display_name);
    println!("  Capabilities: {:?}", metadata.capabilities);
}

#[test]
fn test_provider_capability_support() {
    let config_toml = r#"
        [auth]
        provider = "api_key"
        key = "test-key"
        
        [model]
        provider = "gemini"
        model_name = "gemini-1.5-flash"
    "#;

    let config: ZConfig = toml::from_str(config_toml).expect("Failed to parse config");
    let provider = config.create_provider().expect("Should create provider");

    // Test capability support
    assert!(provider.supports(Capability::TextGeneration));
    assert!(provider.supports(Capability::Embedding));

    println!("✓ Provider capability checks working");
}

#[test]
fn test_openai_missing_api_key() {
    let config_toml = r#"
        [auth]
        provider = "api_key"
        key = "test-key"
        
        [model]
        provider = "openai"
        model_name = "gpt-4o"
    "#;

    let config: ZConfig = toml::from_str(config_toml).expect("Failed to parse config");

    // Should fail because openai_api_key is missing
    let result = config.create_provider();
    assert!(result.is_err(), "Should fail without OpenAI API key");

    println!("✓ Correctly validates OpenAI API key requirement");
}

#[test]
fn test_provider_by_name() {
    let config_toml = r#"
        openai_api_key = "sk-test-key"
        
        [auth]
        provider = "api_key"
        key = "test-key"
        
        [model]
        provider = "gemini"
        model_name = "gemini-1.5-flash"
    "#;

    let config: ZConfig = toml::from_str(config_toml).expect("Failed to parse config");

    // Create Gemini provider
    let gemini = config
        .create_provider_by_name("gemini")
        .expect("Should create Gemini");
    assert_eq!(gemini.metadata().name, "gemini");

    // Create OpenAI provider
    let openai = config
        .create_provider_by_name("openai")
        .expect("Should create OpenAI");
    assert_eq!(openai.metadata().name, "openai");

    println!("✓ Provider creation by name working");
}
