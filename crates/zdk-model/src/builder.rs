//! Builder pattern for programmatic model creation

use crate::factory::{ModelFactory, Provider};
use std::sync::Arc;
use zdk_core::{Error, LLM, Result, ZConfig};

/// Builder for creating LLM models programmatically
///
/// The ModelBuilder provides a fluent API for creating models without
/// relying on configuration files. This is useful for:
/// - Testing
/// - Dynamic model selection
/// - Applications that manage configuration differently
///
/// # Example
/// ```no_run
/// use zdk_model::{ModelBuilder, Provider};
///
/// # async fn example() -> zdk_core::Result<()> {
/// let model = ModelBuilder::new()
///     .provider(Provider::OpenAI)
///     .model_name("gpt-4o")
///     .api_key(std::env::var("OPENAI_API_KEY")?)
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ModelBuilder {
    provider: Option<Provider>,
    model_name: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    // GCloud-specific fields
    project_id: Option<String>,
    location: Option<String>,
    token: Option<String>,
}

impl ModelBuilder {
    /// Create a new ModelBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the provider
    ///
    /// # Arguments
    /// * `provider` - The LLM provider to use
    pub fn provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the model name
    ///
    /// # Arguments
    /// * `name` - The model identifier (e.g., "gemini-2.0-flash-exp", "gpt-4o")
    pub fn model_name(mut self, name: impl Into<String>) -> Self {
        self.model_name = Some(name.into());
        self
    }

    /// Set the API key
    ///
    /// # Arguments
    /// * `key` - The API key for authentication
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set a custom base URL
    ///
    /// Useful for OpenAI-compatible endpoints (e.g., local Ollama server)
    ///
    /// # Arguments
    /// * `url` - The base URL for the API
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set Google Cloud project ID (for Gemini/Vertex AI)
    ///
    /// # Arguments
    /// * `project` - The GCP project ID
    pub fn project_id(mut self, project: impl Into<String>) -> Self {
        self.project_id = Some(project.into());
        self
    }

    /// Set Google Cloud location (for Gemini/Vertex AI)
    ///
    /// # Arguments
    /// * `location` - The GCP location (e.g., "us-central1")
    pub fn location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set access token (for Gemini/Vertex AI with bearer auth)
    ///
    /// # Arguments
    /// * `token` - The bearer access token
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Build the model with explicit configuration
    ///
    /// # Returns
    /// An Arc-wrapped LLM instance
    ///
    /// # Errors
    /// Returns an error if:
    /// - Required fields are missing
    /// - Provider is not enabled via features
    /// - Model initialization fails
    pub fn build(self) -> Result<Arc<dyn LLM>> {
        let provider = self
            .provider
            .clone()
            .ok_or_else(|| Error::config_error("Provider is required"))?;
        let model_name = self
            .model_name
            .clone()
            .ok_or_else(|| Error::config_error("Model name is required"))?;

        match provider {
            Provider::Gemini => self.build_gemini(model_name),
            Provider::OpenAI => self.build_openai(model_name),
            Provider::Anthropic => Err(Error::config_error(
                "Anthropic provider not yet implemented",
            )),
            Provider::Local => Err(Error::config_error("Local provider not yet implemented")),
        }
    }

    /// Build from config (convenience method)
    ///
    /// This is equivalent to `ModelFactory::from_config(config)` but allows
    /// using the builder pattern for consistency.
    ///
    /// # Arguments
    /// * `config` - The ZDK configuration
    ///
    /// # Returns
    /// An Arc-wrapped LLM instance
    pub fn from_config(config: &ZConfig) -> Result<Arc<dyn LLM>> {
        ModelFactory::from_config(config)
    }

    #[cfg(feature = "gemini")]
    fn build_gemini(self, model_name: String) -> Result<Arc<dyn LLM>> {
        use crate::gemini::GeminiModel;

        // Check if we have bearer token (for Vertex AI)
        if let Some(token) = self.token {
            let project = self
                .project_id
                .ok_or_else(|| Error::config_error("Project ID required for Vertex AI"))?;
            let location = self.location.unwrap_or_else(|| "us-central1".to_string());

            Ok(Arc::new(GeminiModel::with_bearer_token(
                token, model_name, project, location,
            )))
        } else {
            // Use API key
            let api_key = self
                .api_key
                .ok_or_else(|| Error::config_error("API key required for Gemini"))?;

            Ok(Arc::new(GeminiModel::new(api_key, model_name)))
        }
    }

    #[cfg(not(feature = "gemini"))]
    fn build_gemini(self, _model_name: String) -> Result<Arc<dyn LLM>> {
        Err(Error::config_error(
            "Gemini provider not enabled. Add 'gemini' feature to zdk-model",
        ))
    }

    #[cfg(feature = "openai")]
    fn build_openai(self, model_name: String) -> Result<Arc<dyn LLM>> {
        use crate::openai::OpenAIModel;

        let api_key = self
            .api_key
            .ok_or_else(|| Error::config_error("API key required for OpenAI"))?;

        let mut model = OpenAIModel::new(api_key, model_name);

        if let Some(base_url) = self.base_url {
            model = model.with_base_url(base_url);
        }

        Ok(Arc::new(model))
    }

    #[cfg(not(feature = "openai"))]
    fn build_openai(self, _model_name: String) -> Result<Arc<dyn LLM>> {
        Err(Error::config_error(
            "OpenAI provider not enabled. Add 'openai' feature to zdk-model",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_requires_provider() {
        let result = ModelBuilder::new()
            .model_name("test-model")
            .api_key("test-key")
            .build();

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Provider"));
        }
    }

    #[test]
    fn test_builder_requires_model_name() {
        let result = ModelBuilder::new()
            .provider(Provider::Gemini)
            .api_key("test-key")
            .build();

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Model name"));
        }
    }
}
