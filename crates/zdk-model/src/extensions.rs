//! Extension traits for ZDK core types

use crate::factory::{ModelFactory, Provider};
use std::sync::Arc;
use zdk_core::{LLM, Result, ZConfig};

/// Extension trait for ZConfig to simplify model creation
///
/// This trait provides convenience methods on ZConfig for creating
/// LLM instances without manually calling ModelFactory.
///
/// # Example
/// ```no_run
/// use zdk_core::ZConfig;
/// use zdk_model::ZConfigExt;
///
/// # async fn example() -> zdk_core::Result<()> {
/// let config = ZConfig::load()?;
///
/// // Create model using configured provider
/// let model = config.create_model()?;
///
/// // Or override provider
/// let openai_model = config.create_model_with_provider(zdk_model::Provider::OpenAI)?;
/// # Ok(())
/// # }
/// ```
pub trait ZConfigExt {
    /// Create a model from this config using the configured provider
    ///
    /// The provider is determined by the `model.provider` field in the config.
    ///
    /// # Example
    /// ```no_run
    /// use zdk_core::ZConfig;
    /// use zdk_model::ZConfigExt;
    ///
    /// # fn example() -> zdk_core::Result<()> {
    /// let config = ZConfig::load()?;
    /// let model = config.create_model()?;
    /// # Ok(())
    /// # }
    /// ```
    fn create_model(&self) -> Result<Arc<dyn LLM>>;

    /// Create a model with explicit provider, overriding config
    ///
    /// This allows you to create a model for a different provider than
    /// what's configured in `model.provider`.
    ///
    /// # Example
    /// ```no_run
    /// use zdk_core::ZConfig;
    /// use zdk_model::{ZConfigExt, Provider};
    ///
    /// # fn example() -> zdk_core::Result<()> {
    /// let config = ZConfig::load()?;
    /// let openai_model = config.create_model_with_provider(Provider::OpenAI)?;
    /// # Ok(())
    /// # }
    /// ```
    fn create_model_with_provider(&self, provider: Provider) -> Result<Arc<dyn LLM>>;
}

impl ZConfigExt for ZConfig {
    fn create_model(&self) -> Result<Arc<dyn LLM>> {
        ModelFactory::from_config(self)
    }

    fn create_model_with_provider(&self, provider: Provider) -> Result<Arc<dyn LLM>> {
        ModelFactory::create(provider, self)
    }
}
