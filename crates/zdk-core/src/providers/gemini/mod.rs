//! Google Gemini provider
//!
//! Supports multiple capabilities:
//! - Text generation: gemini-2.0-flash-exp, gemini-1.5-flash, etc.
//! - Embeddings: text-embedding-004 (768 dimensions)
//! - Transcription: gemini-pro-audio
//! - Image generation: Imagen

pub mod auth;
pub mod provider;
pub mod types;

pub use auth::GeminiAuth;
pub use provider::GeminiProvider;

/// Gemini configuration
#[derive(Clone, Debug)]
pub struct GeminiConfig {
    /// Model name for text generation
    pub model: String,
    /// Base URL for API requests
    pub base_url: String,
    /// Embedding model name
    pub embedding_model: Option<String>,
    /// Audio/transcription model name
    pub audio_model: Option<String>,
}

impl GeminiConfig {
    /// Create default configuration for API key auth
    pub fn default_api_key(model: String) -> Self {
        Self {
            model,
            base_url: "https://generativelanguage.googleapis.com/v1/models".to_string(),
            embedding_model: Some("text-embedding-004".to_string()),
            audio_model: Some("gemini-pro-audio".to_string()),
        }
    }
    
    /// Create default configuration for Vertex AI (bearer token)
    pub fn default_vertex_ai(model: String, project_id: String, location: String) -> Self {
        Self {
            model,
            base_url: format!(
                "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models",
                location, project_id, location
            ),
            embedding_model: Some("text-embedding-004".to_string()),
            audio_model: Some("gemini-pro-audio".to_string()),
        }
    }
}

/// Builder for GeminiProvider
pub struct GeminiBuilder {
    auth: Option<GeminiAuth>,
    config: Option<GeminiConfig>,
}

impl GeminiBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            auth: None,
            config: None,
        }
    }
    
    /// Set API key authentication
    pub fn with_api_key(mut self, api_key: String, model: String) -> Self {
        self.auth = Some(GeminiAuth::ApiKey(api_key));
        self.config = Some(GeminiConfig::default_api_key(model));
        self
    }
    
    /// Set bearer token authentication (Vertex AI)
    pub fn with_bearer_token(
        mut self,
        token: String,
        model: String,
        project_id: String,
        location: String,
    ) -> Self {
        self.auth = Some(GeminiAuth::BearerToken(token));
        self.config = Some(GeminiConfig::default_vertex_ai(model, project_id, location));
        self
    }
    
    /// Set custom configuration
    pub fn with_config(mut self, config: GeminiConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Build the provider
    pub fn build(self) -> crate::Result<GeminiProvider> {
        let auth = self
            .auth
            .ok_or_else(|| crate::Error::config_error("Authentication is required"))?;
        let config = self
            .config
            .ok_or_else(|| crate::Error::config_error("Configuration is required"))?;
        
        Ok(GeminiProvider::new(auth, config))
    }
}

impl Default for GeminiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

