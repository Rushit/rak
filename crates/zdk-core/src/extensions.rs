//! Extension traits for ZDK core types

use crate::{
    Result, ZConfig,
    providers::{Capability, Provider, ProviderMetadata, ProviderRegistry},
};
use std::sync::Arc;

/// Extension trait for ZConfig to simplify provider creation
///
/// This trait provides convenience methods on ZConfig for creating
/// provider instances without manually calling ProviderRegistry.
///
/// # Example
/// ```no_run
/// use zdk_core::{ZConfig, ZConfigExt};
///
/// # fn example() -> zdk_core::Result<()> {
/// let config = ZConfig::load()?;
///
/// // Create provider using configured provider
/// let provider = config.create_provider()?;
///
/// // Discover all available providers
/// let available = config.discover_providers();
/// # Ok(())
/// # }
/// ```
pub trait ZConfigExt {
    /// Create a provider from this config using the configured provider
    ///
    /// The provider is determined by the `model.provider` field in the config.
    ///
    /// # Example
    /// ```no_run
    /// use zdk_core::{ZConfig, ZConfigExt};
    ///
    /// # fn example() -> zdk_core::Result<()> {
    /// let config = ZConfig::load()?;
    /// let provider = config.create_provider()?;
    /// # Ok(())
    /// # }
    /// ```
    fn create_provider(&self) -> Result<Arc<dyn Provider>>;

    /// Create a provider with explicit provider name, overriding config
    ///
    /// This allows you to create a provider for a different implementation than
    /// what's configured in `model.provider`.
    ///
    /// # Example
    /// ```no_run
    /// use zdk_core::{ZConfig, ZConfigExt};
    ///
    /// # fn example() -> zdk_core::Result<()> {
    /// let config = ZConfig::load()?;
    /// let openai = config.create_provider_by_name("openai")?;
    /// # Ok(())
    /// # }
    /// ```
    fn create_provider_by_name(&self, provider: &str) -> Result<Arc<dyn Provider>>;

    /// Discover all available providers
    ///
    /// Returns metadata for all registered providers in the system.
    ///
    /// # Example
    /// ```no_run
    /// use zdk_core::{ZConfig, ZConfigExt};
    ///
    /// # fn example() -> zdk_core::Result<()> {
    /// let config = ZConfig::load()?;
    /// let providers = config.discover_providers();
    /// for provider in providers {
    ///     println!("Provider: {} ({})", provider.display_name, provider.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn discover_providers(&self) -> Vec<ProviderMetadata>;

    /// Find providers that support a specific capability
    ///
    /// # Example
    /// ```no_run
    /// use zdk_core::{Capability, ZConfig, ZConfigExt};
    ///
    /// # fn example() -> zdk_core::Result<()> {
    /// let config = ZConfig::load()?;
    /// let embedding_providers = config.find_providers_with(Capability::Embedding);
    /// println!("Embedding providers: {:?}", embedding_providers);
    /// # Ok(())
    /// # }
    /// ```
    fn find_providers_with(&self, capability: Capability) -> Vec<String>;
}

impl ZConfigExt for ZConfig {
    fn create_provider(&self) -> Result<Arc<dyn Provider>> {
        let registry = ProviderRegistry::global();
        registry.create(&self.model.provider, self)
    }

    fn create_provider_by_name(&self, provider: &str) -> Result<Arc<dyn Provider>> {
        let registry = ProviderRegistry::global();
        registry.create(provider, self)
    }

    fn discover_providers(&self) -> Vec<ProviderMetadata> {
        let registry = ProviderRegistry::global();
        registry.list_providers()
    }

    fn find_providers_with(&self, capability: Capability) -> Vec<String> {
        let registry = ProviderRegistry::global();
        registry.find_by_capability(capability)
    }
}
