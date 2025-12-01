//! Error types for OpenAPI tool generation.

use thiserror::Error;

/// Result type for OpenAPI operations.
pub type Result<T> = std::result::Result<T, OpenApiError>;

/// Errors that can occur during OpenAPI tool generation and execution.
#[derive(Error, Debug)]
pub enum OpenApiError {
    /// OpenAPI spec parsing error
    #[error("Failed to parse OpenAPI spec: {0}")]
    ParseError(String),

    /// Invalid OpenAPI specification
    #[error("Invalid OpenAPI spec: {0}")]
    InvalidSpec(String),

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// URL parsing error
    #[error("Invalid URL: {0}")]
    UrlError(#[from] url::ParseError),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// YAML parsing error
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Missing required parameter
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    /// Invalid parameter value
    #[error("Invalid parameter value for '{0}': {1}")]
    InvalidParameter(String, String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Operation not found
    #[error("Operation '{0}' not found in OpenAPI spec")]
    OperationNotFound(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}
