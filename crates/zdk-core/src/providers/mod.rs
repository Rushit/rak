//! LLM Provider implementations with multi-capability support
//!
//! This module provides a unified provider system where each provider can support
//! multiple capabilities (text generation, embeddings, transcription, etc.).
//!
//! # Architecture
//!
//! Providers implement the `Provider` trait which includes optional methods for
//! each capability. Providers opt-in to capabilities by implementing the relevant
//! methods.
//!
//! # Available Providers
//!
//! - **Gemini**: Google's Gemini models
//! - **OpenAI**: OpenAI's GPT and other models
//!
//! # Example
//!
//! ```ignore
//! use zdk_core::{Provider, Capability, ZConfig, ZConfigExt};
//!
//! // Create provider from config
//! let config = ZConfig::load()?;
//! let provider = config.create_provider()?;
//!
//! // Check capabilities
//! if provider.supports(Capability::TextGeneration) {
//!     let response = provider.generate_content(request, true).await?;
//! }
//!
//! if provider.supports(Capability::Embedding) {
//!     let vectors = provider.embed_texts(vec!["Hello".into()]).await?;
//! }
//! ```

pub mod factory;
pub mod provider;

// Core utilities (will be added in next milestone)
// pub mod core;

// Provider implementations
pub mod gemini;
pub mod openai;

// Tests
#[cfg(test)]
mod provider_tests;

// Re-exports
pub use factory::{ProviderFactory, ProviderRegistry};
pub use provider::{Capability, ModelInfo, Provider, ProviderMetadata};

// Provider re-exports
pub use gemini::{GeminiAuth, GeminiProvider};
pub use openai::OpenAIProvider;
