//! Provider factory and registry
//!
//! Provides a global registry for provider discovery and instantiation.

use super::provider::{Provider, ProviderMetadata};
use crate::{Error, Result, ZConfig};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;

/// Global provider registry
static REGISTRY: Lazy<ProviderRegistry> = Lazy::new(|| {
    let registry = ProviderRegistry::new();

    // Register all providers (all included by default, no feature flags)
    registry.register("gemini", Box::new(GeminiFactory));
    registry.register("openai", Box::new(OpenAIFactory));

    registry
});

/// Provider registry for discovery and instantiation
pub struct ProviderRegistry {
    providers: DashMap<String, Box<dyn ProviderFactory>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
        }
    }

    /// Get the global registry instance
    pub fn global() -> &'static Self {
        &REGISTRY
    }

    /// Register a provider factory
    ///
    /// # Arguments
    /// * `name` - Provider name (e.g., "gemini", "openai")
    /// * `factory` - Factory implementation for creating the provider
    pub fn register(&self, name: &str, factory: Box<dyn ProviderFactory>) {
        self.providers.insert(name.to_string(), factory);
    }

    /// Create a provider by name
    ///
    /// # Arguments
    /// * `name` - Provider name
    /// * `config` - ZDK configuration
    ///
    /// # Returns
    /// Provider instance wrapped in Arc
    ///
    /// # Errors
    /// Returns error if provider not found or creation fails
    pub fn create(&self, name: &str, config: &ZConfig) -> Result<Arc<dyn Provider>> {
        let factory = self
            .providers
            .get(name)
            .ok_or_else(|| Error::config_error(format!("Provider '{}' not found", name)))?;

        factory.create(config)
    }

    /// List all available providers
    ///
    /// # Returns
    /// Vector of provider metadata
    pub fn list_providers(&self) -> Vec<ProviderMetadata> {
        self.providers
            .iter()
            .filter_map(|entry| entry.value().metadata().ok())
            .collect()
    }

    /// Find providers by capability
    ///
    /// # Arguments
    /// * `capability` - Capability to search for
    ///
    /// # Returns
    /// Vector of provider names that support the capability
    pub fn find_by_capability(&self, capability: super::provider::Capability) -> Vec<String> {
        self.providers
            .iter()
            .filter_map(|entry| {
                if let Ok(meta) = entry.value().metadata()
                    && meta.capabilities.contains(&capability)
                {
                    return Some(entry.key().clone());
                }
                None
            })
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory trait for creating providers
///
/// Each provider implements this trait to enable creation from config
pub trait ProviderFactory: Send + Sync {
    /// Create a provider instance from config
    ///
    /// # Arguments
    /// * `config` - ZDK configuration
    ///
    /// # Returns
    /// Provider instance wrapped in Arc
    fn create(&self, config: &ZConfig) -> Result<Arc<dyn Provider>>;

    /// Get provider metadata
    ///
    /// # Returns
    /// Provider metadata including capabilities
    fn metadata(&self) -> Result<ProviderMetadata>;
}

// ============================================================================
// Provider Factory Implementations
// ============================================================================

/// Gemini provider factory
struct GeminiFactory;

impl ProviderFactory for GeminiFactory {
    fn create(&self, config: &ZConfig) -> Result<Arc<dyn Provider>> {
        use crate::AuthCredentials;
        use crate::providers::gemini::{GeminiAuth, GeminiConfig, GeminiProvider};

        let creds = config.get_auth_credentials()?;

        let (auth, gemini_config) = match creds {
            AuthCredentials::ApiKey { key } => {
                let auth = GeminiAuth::ApiKey(key);
                let config = GeminiConfig::default_api_key(config.model.model_name.clone());
                (auth, config)
            }
            AuthCredentials::GCloud {
                token,
                project,
                location,
                ..
            } => {
                let auth = GeminiAuth::BearerToken(token);
                let config = GeminiConfig::default_vertex_ai(
                    config.model.model_name.clone(),
                    project,
                    location,
                );
                (auth, config)
            }
        };

        Ok(Arc::new(GeminiProvider::new(auth, gemini_config)))
    }

    fn metadata(&self) -> Result<ProviderMetadata> {
        use crate::providers::gemini::GeminiProvider;
        Ok(GeminiProvider::static_metadata())
    }
}

/// OpenAI provider factory
struct OpenAIFactory;

impl ProviderFactory for OpenAIFactory {
    fn create(&self, config: &ZConfig) -> Result<Arc<dyn Provider>> {
        use crate::providers::openai::{OpenAIConfig, OpenAIProvider};

        let api_key = config
            .openai_api_key
            .clone()
            .ok_or_else(|| {
                Error::config_error(
                    "OpenAI API key not found. Set openai_api_key in config.toml or OPENAI_API_KEY env var"
                )
            })?;

        let openai_config = if let Some(ref base_url) = config.openai_base_url {
            OpenAIConfig::with_base_url(config.model.model_name.clone(), base_url.clone())
        } else {
            OpenAIConfig::default(config.model.model_name.clone())
        };

        Ok(Arc::new(OpenAIProvider::new(api_key, openai_config)))
    }

    fn metadata(&self) -> Result<ProviderMetadata> {
        use crate::providers::openai::OpenAIProvider;
        Ok(OpenAIProvider::static_metadata())
    }
}
