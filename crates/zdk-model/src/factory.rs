//! Model factory for creating LLM instances from configuration

use std::sync::Arc;
use zdk_core::{Error, LLM, Result, ZConfig};

/// Supported LLM providers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    /// Google Gemini models
    Gemini,
    /// OpenAI models  
    OpenAI,
    /// Anthropic Claude models (future)
    Anthropic,
    /// Local models via Ollama or llama.cpp (future)
    Local,
}

impl Provider {
    /// Parse provider from string
    ///
    /// # Example
    /// ```
    /// use zdk_model::Provider;
    ///
    /// let provider = Provider::from_str("gemini").unwrap();
    /// assert_eq!(provider, Provider::Gemini);
    /// ```
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "gemini" => Ok(Provider::Gemini),
            "openai" => Ok(Provider::OpenAI),
            "anthropic" | "claude" => Ok(Provider::Anthropic),
            "local" | "ollama" => Ok(Provider::Local),
            _ => Err(Error::config_error(format!("Unknown provider: {}", s))),
        }
    }

    /// Get the provider name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::Gemini => "gemini",
            Provider::OpenAI => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Local => "local",
        }
    }
}

/// Model factory for creating LLM instances from configuration
///
/// The factory pattern simplifies model creation by automatically handling
/// provider selection and authentication based on configuration.
///
/// # Example
/// ```no_run
/// use zdk_core::ZConfig;
/// use zdk_model::ModelFactory;
///
/// # async fn example() -> zdk_core::Result<()> {
/// let config = ZConfig::load()?;
/// let model = ModelFactory::from_config(&config)?;
/// # Ok(())
/// # }
/// ```
pub struct ModelFactory;

impl ModelFactory {
    /// Create a model from ZConfig
    ///
    /// Automatically selects provider and auth method based on config.
    ///
    /// # Example
    /// ```no_run
    /// use zdk_core::ZConfig;
    /// use zdk_model::ModelFactory;
    ///
    /// # async fn example() -> zdk_core::Result<()> {
    /// let config = ZConfig::load()?;
    /// let model = ModelFactory::from_config(&config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_config(config: &ZConfig) -> Result<Arc<dyn LLM>> {
        let provider = Provider::from_str(&config.model.provider)?;
        Self::create(provider, config)
    }

    /// Create model with explicit provider
    ///
    /// Bypasses config.model.provider and uses the specified provider.
    ///
    /// # Example
    /// ```no_run
    /// use zdk_core::ZConfig;
    /// use zdk_model::{ModelFactory, Provider};
    ///
    /// # async fn example() -> zdk_core::Result<()> {
    /// let config = ZConfig::load()?;
    /// let model = ModelFactory::create(Provider::OpenAI, &config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(provider: Provider, config: &ZConfig) -> Result<Arc<dyn LLM>> {
        match provider {
            Provider::Gemini => Self::create_gemini(config),
            Provider::OpenAI => Self::create_openai(config),
            Provider::Anthropic => Self::create_anthropic(config),
            Provider::Local => Self::create_local(config),
        }
    }

    #[cfg(feature = "gemini")]
    fn create_gemini(config: &ZConfig) -> Result<Arc<dyn LLM>> {
        use crate::gemini::GeminiModel;
        use zdk_core::AuthCredentials;

        let creds = config.get_auth_credentials()?;

        let model: Arc<dyn LLM> = match creds {
            AuthCredentials::ApiKey { key } => {
                Arc::new(GeminiModel::new(key, config.model.model_name.clone()))
            }
            AuthCredentials::GCloud {
                token,
                project,
                location,
                ..
            } => Arc::new(GeminiModel::with_bearer_token(
                token,
                config.model.model_name.clone(),
                project,
                location,
            )),
        };

        Ok(model)
    }

    #[cfg(not(feature = "gemini"))]
    fn create_gemini(_config: &ZConfig) -> Result<Arc<dyn LLM>> {
        Err(Error::config_error(
            "Gemini provider not enabled. Add 'gemini' feature to zdk-model",
        ))
    }

    #[cfg(feature = "openai")]
    fn create_openai(config: &ZConfig) -> Result<Arc<dyn LLM>> {
        use crate::openai::OpenAIModel;

        let api_key = config
            .openai_api_key
            .clone()
            .ok_or_else(|| Error::config_error(
                "OpenAI API key not found. Set openai_api_key in config.toml or OPENAI_API_KEY env var"
            ))?;

        let mut model = OpenAIModel::new(api_key, config.model.model_name.clone());

        // Allow custom base URL for OpenAI-compatible endpoints
        if let Some(ref base_url) = config.openai_base_url {
            model = model.with_base_url(base_url.clone());
        }

        Ok(Arc::new(model))
    }

    #[cfg(not(feature = "openai"))]
    fn create_openai(_config: &ZConfig) -> Result<Arc<dyn LLM>> {
        Err(Error::config_error(
            "OpenAI provider not enabled. Add 'openai' feature to zdk-model",
        ))
    }

    fn create_anthropic(_config: &ZConfig) -> Result<Arc<dyn LLM>> {
        Err(Error::config_error(
            "Anthropic provider not yet implemented",
        ))
    }

    fn create_local(_config: &ZConfig) -> Result<Arc<dyn LLM>> {
        Err(Error::config_error("Local provider not yet implemented"))
    }
}
