//! # ZDK OpenAPI Tool Generator
//!
//! Automatically generates ZDK tools from OpenAPI specifications.
//!
//! ## Features
//!
//! - Parse OpenAPI v3.0+ specifications (JSON and YAML)
//! - Generate tools for each API operation
//! - Support for common authentication methods (API Key, Bearer Token, Basic Auth)
//! - HTTP request building and execution
//! - Error handling with LLM-friendly error messages
//!
//! ## Example
//!
//! ```no_run
//! use zdk_openapi::{OpenApiToolset, AuthConfig};
//! use std::env;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! // Load OpenAPI spec from file
//! let toolset = OpenApiToolset::from_file("./api/openapi.yaml")?
//!     .with_auth(AuthConfig::api_key_header("X-API-Key", env::var("API_KEY")?));
//!
//! // Get all generated tools
//! let tools = toolset.tools();
//! println!("Generated {} tools", tools.len());
//! # Ok(())
//! # }
//! ```

mod auth;
mod error;
mod parser;
mod rest_api_tool;
mod toolset;
mod types;

pub use auth::{AuthConfig, AuthLocation};
pub use error::{OpenApiError, Result};
pub use parser::OpenApiParser;
pub use rest_api_tool::RestApiTool;
pub use toolset::OpenApiToolset;
pub use types::{ApiParameter, OperationEndpoint, ParsedOperation};
