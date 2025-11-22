//! Core traits and types for RAK
//!
//! This crate provides the foundational abstractions for building AI agents.

pub mod auth;
pub mod config;
pub mod content;
pub mod context;
pub mod error;
pub mod event;
pub mod traits;

// Re-exports
pub use auth::{AuthCredentials, AuthProvider, ApiKeyConfig, GCloudConfig};
pub use config::RakConfig;
pub use content::{Content, FunctionCall, FunctionResponse, InlineData, Part};
pub use context::{InvocationContext, ReadonlyContext, ToolContext};
pub use error::{Error, Result};
pub use event::{Event, EventActions};
pub use traits::{Agent, GenerateConfig, LLMRequest, LLMResponse, Tool, ToolResponse, LLM};
