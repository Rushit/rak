//! Provider trait and metadata definitions
//!
//! Defines the unified Provider trait that supports multiple capabilities.

use crate::{Error, LLMRequest, LLMResponse, Result, capabilities::*};
use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};

/// Provider capability enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Text generation (LLM completion)
    TextGeneration,
    /// Text to vector embeddings
    Embedding,
    /// Audio to text transcription
    Transcription,
    /// Text to image generation
    ImageGeneration,
    /// Text to speech audio generation
    AudioGeneration,
}

/// Provider metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Provider identifier (e.g., "gemini", "openai")
    pub name: String,
    /// Display name (e.g., "Google Gemini", "OpenAI")
    pub display_name: String,
    /// Supported capabilities
    pub capabilities: Vec<Capability>,
    /// Available models
    pub models: Vec<ModelInfo>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier
    pub id: String,
    /// Display name
    pub display_name: String,
    /// Capabilities this model supports
    pub capabilities: Vec<Capability>,
    /// Context window size (tokens)
    pub context_window: Option<usize>,
    /// Embedding dimensions (if applicable)
    pub embedding_dimensions: Option<usize>,
}

/// Unified provider trait supporting multiple capabilities
///
/// Providers implement this trait to expose their capabilities.
/// Methods have default implementations that return UnsupportedCapability errors,
/// allowing providers to opt-in to only the capabilities they support.
///
/// Provider extends LLM, so all providers are also valid LLM implementations.
#[async_trait]
pub trait Provider: crate::LLM {
    /// Get provider metadata
    fn metadata(&self) -> ProviderMetadata;

    /// Check if provider supports a capability
    fn supports(&self, capability: Capability) -> bool {
        self.metadata().capabilities.contains(&capability)
    }

    // ===== Text Generation Capability =====

    /// Generate text content (LLM completion)
    ///
    /// # Arguments
    /// * `request` - LLM request with contents and configuration
    /// * `stream` - Whether to stream the response
    ///
    /// # Returns
    /// A stream of LLM responses (one item for batch, multiple for streaming)
    async fn generate_content(
        &self,
        _request: LLMRequest,
        _stream: bool,
    ) -> Result<Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin>> {
        Err(Error::Other(anyhow::anyhow!(
            "Provider does not support text generation capability"
        )))
    }

    // ===== Embedding Capability =====

    /// Embed texts into vector representations
    ///
    /// # Arguments
    /// * `texts` - List of texts to embed
    ///
    /// # Returns
    /// Vector of embedding vectors
    async fn embed_texts(&self, _texts: Vec<String>) -> Result<Vec<EmbeddingVector>> {
        Err(Error::Other(anyhow::anyhow!(
            "Provider does not support embedding capability"
        )))
    }

    /// Get embedding dimensions for this provider
    ///
    /// Returns None if provider doesn't support embeddings
    fn embedding_dimensions(&self) -> Option<usize> {
        None
    }

    /// Get maximum batch size for embeddings
    ///
    /// Returns None if provider doesn't support embeddings
    fn max_embedding_batch_size(&self) -> Option<usize> {
        None
    }

    // ===== Transcription Capability =====

    /// Transcribe audio to text
    ///
    /// # Arguments
    /// * `audio` - Audio input with data and format
    ///
    /// # Returns
    /// Transcription result with text and metadata
    async fn transcribe_audio(&self, _audio: AudioInput) -> Result<TranscriptionResult> {
        Err(Error::Other(anyhow::anyhow!(
            "Provider does not support transcription capability"
        )))
    }

    /// Get supported audio formats for transcription
    ///
    /// Returns None if provider doesn't support transcription
    fn supported_audio_formats(&self) -> Option<&[&str]> {
        None
    }

    // ===== Image Generation Capability =====

    /// Generate image from text prompt
    ///
    /// # Arguments
    /// * `request` - Image generation request with prompt and options
    ///
    /// # Returns
    /// Image generation result with generated images
    async fn generate_image(&self, _request: ImageRequest) -> Result<ImageResult> {
        Err(Error::Other(anyhow::anyhow!(
            "Provider does not support image generation capability"
        )))
    }

    // ===== Audio Generation Capability =====

    /// Generate audio from text (text-to-speech)
    ///
    /// # Arguments
    /// * `request` - Audio generation request with text and options
    ///
    /// # Returns
    /// Audio generation result with audio data
    async fn generate_audio(&self, _request: AudioRequest) -> Result<AudioResult> {
        Err(Error::Other(anyhow::anyhow!(
            "Provider does not support audio generation capability"
        )))
    }
}
