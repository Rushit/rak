//! OpenAI provider
//!
//! Supports multiple capabilities:
//! - Text generation: gpt-4o, gpt-4-turbo, gpt-3.5-turbo, etc.
//! - Embeddings: text-embedding-3-small, text-embedding-3-large, text-embedding-ada-002
//! - Transcription: Whisper
//! - Image generation: DALL-E
//! - Audio generation: TTS

pub mod provider;
pub mod types;

pub use provider::OpenAIProvider;

/// OpenAI configuration
#[derive(Clone, Debug)]
pub struct OpenAIConfig {
    /// Model name for text generation
    pub model: String,
    /// Base URL for API requests
    pub base_url: String,
    /// Embedding model name
    pub embedding_model: Option<String>,
}

impl OpenAIConfig {
    /// Create default configuration
    pub fn default(model: String) -> Self {
        Self {
            model,
            base_url: "https://api.openai.com/v1".to_string(),
            embedding_model: Some("text-embedding-3-small".to_string()),
        }
    }
    
    /// Create configuration with custom base URL (e.g., for Ollama)
    pub fn with_base_url(model: String, base_url: String) -> Self {
        Self {
            model,
            base_url,
            embedding_model: Some("text-embedding-3-small".to_string()),
        }
    }
}

/// Builder for OpenAIProvider
pub struct OpenAIBuilder {
    api_key: Option<String>,
    config: Option<OpenAIConfig>,
}

impl OpenAIBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            api_key: None,
            config: None,
        }
    }
    
    /// Set API key
    pub fn with_api_key(mut self, api_key: String, model: String) -> Self {
        self.api_key = Some(api_key);
        self.config = Some(OpenAIConfig::default(model));
        self
    }
    
    /// Set API key with custom base URL
    pub fn with_api_key_and_base_url(
        mut self,
        api_key: String,
        model: String,
        base_url: String,
    ) -> Self {
        self.api_key = Some(api_key);
        self.config = Some(OpenAIConfig::with_base_url(model, base_url));
        self
    }
    
    /// Set custom configuration
    pub fn with_config(mut self, config: OpenAIConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Build the provider
    pub fn build(self) -> crate::Result<OpenAIProvider> {
        let api_key = self
            .api_key
            .ok_or_else(|| crate::Error::config_error("API key is required"))?;
        let config = self
            .config
            .ok_or_else(|| crate::Error::config_error("Configuration is required"))?;
        
        Ok(OpenAIProvider::new(api_key, config))
    }
}

impl Default for OpenAIBuilder {
    fn default() -> Self {
        Self::new()
    }
}

