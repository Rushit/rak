//! Tests for the provider system

#[cfg(test)]
mod tests {
    use crate::{
        providers::{Capability, GeminiProvider, OpenAIProvider, ProviderRegistry},
        ZConfig,
    };

    #[test]
    fn test_gemini_metadata() {
        let metadata = GeminiProvider::static_metadata();
        
        assert_eq!(metadata.name, "gemini");
        assert_eq!(metadata.display_name, "Google Gemini");
        assert!(metadata.capabilities.contains(&Capability::TextGeneration));
        assert!(metadata.capabilities.contains(&Capability::Embedding));
        assert!(!metadata.models.is_empty());
    }

    #[test]
    fn test_openai_metadata() {
        let metadata = OpenAIProvider::static_metadata();
        
        assert_eq!(metadata.name, "openai");
        assert_eq!(metadata.display_name, "OpenAI");
        assert!(metadata.capabilities.contains(&Capability::TextGeneration));
        assert!(metadata.capabilities.contains(&Capability::Embedding));
        assert!(metadata.capabilities.contains(&Capability::Transcription));
        assert!(!metadata.models.is_empty());
    }

    #[test]
    fn test_registry_discovery() {
        let registry = ProviderRegistry::global();
        let providers = registry.list_providers();
        
        // Should have at least Gemini and OpenAI
        assert!(providers.len() >= 2);
        
        let names: Vec<String> = providers.iter().map(|p| p.name.clone()).collect();
        assert!(names.contains(&"gemini".to_string()));
        assert!(names.contains(&"openai".to_string()));
    }

    #[test]
    fn test_capability_search() {
        let registry = ProviderRegistry::global();
        
        let text_gen_providers = registry.find_by_capability(Capability::TextGeneration);
        assert!(text_gen_providers.contains(&"gemini".to_string()));
        assert!(text_gen_providers.contains(&"openai".to_string()));
        
        let embedding_providers = registry.find_by_capability(Capability::Embedding);
        assert!(embedding_providers.contains(&"gemini".to_string()));
        assert!(embedding_providers.contains(&"openai".to_string()));
        
        let transcription_providers = registry.find_by_capability(Capability::Transcription);
        assert!(transcription_providers.contains(&"openai".to_string()));
    }

    #[test]
    fn test_provider_not_found() {
        let registry = ProviderRegistry::global();
        
        // Create a minimal config for testing
        let config_toml = r#"
            [auth]
            provider = "api_key"
            key = "test-key"
            
            [model]
            provider = "nonexistent"
            model_name = "test-model"
        "#;
        
        let config: ZConfig = toml::from_str(config_toml).unwrap();
        let result = registry.create("nonexistent", &config);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_capability_enum() {
        // Test that all capabilities are distinct
        let capabilities = vec![
            Capability::TextGeneration,
            Capability::Embedding,
            Capability::Transcription,
            Capability::ImageGeneration,
            Capability::AudioGeneration,
        ];
        
        // Create a set to ensure all are unique
        use std::collections::HashSet;
        let set: HashSet<_> = capabilities.iter().collect();
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn test_embedding_vector() {
        use crate::EmbeddingVector;
        
        let vector = vec![0.1, 0.2, 0.3];
        let embedding = EmbeddingVector::new(vector.clone());
        
        assert_eq!(embedding.dimensions, 3);
        assert_eq!(embedding.vector, vector);
    }

    #[test]
    fn test_transcription_result() {
        use crate::TranscriptionResult;
        
        let result = TranscriptionResult {
            text: "Hello world".to_string(),
            language: Some("en".to_string()),
            duration: Some(2.5),
            segments: None,
        };
        
        assert_eq!(result.text, "Hello world");
        assert_eq!(result.language, Some("en".to_string()));
        assert_eq!(result.duration, Some(2.5));
    }
}

