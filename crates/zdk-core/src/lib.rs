//! Core traits and types for ZDK
//!
//! This crate provides the foundational abstractions for building AI agents.

pub mod auth;
pub mod capabilities;
pub mod config;
pub mod content;
pub mod context;
pub mod error;
pub mod event;
pub mod extensions;
pub mod providers;
pub mod traits;

// Re-exports
pub use auth::{ApiKeyConfig, AuthCredentials, AuthProvider, GCloudConfig};
pub use capabilities::{
    AudioInput, AudioRequest, AudioResult, EmbeddingVector, GeneratedImage, ImageRequest,
    ImageResult, TranscriptionResult, TranscriptionSegment,
};
pub use config::ZConfig;
pub use content::{Content, FunctionCall, FunctionResponse, InlineData, Part};
pub use context::{InvocationContext, ReadonlyContext, ToolContext};
pub use error::{Error, Result};
pub use event::{Event, EventActions};
pub use extensions::ZConfigExt;
pub use providers::{
    Capability, GeminiAuth, GeminiProvider, ModelInfo, OpenAIProvider, Provider, ProviderFactory,
    ProviderMetadata, ProviderRegistry,
};
pub use traits::{
    Agent, GenerateConfig, LLM, LLMRequest, LLMResponse, Tool, ToolResponse, Toolset,
};
