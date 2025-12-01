//! Data structures for OpenAPI parsing.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents an API endpoint with base URL, path, and HTTP method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationEndpoint {
    /// Base URL of the API (e.g., "https://api.example.com")
    pub base_url: String,
    /// Path template (e.g., "/users/{id}")
    pub path: String,
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,
}

/// Represents a parameter in an API operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiParameter {
    /// Original name from OpenAPI spec
    pub original_name: String,
    /// Python/Rust-friendly name (snake_case)
    pub name: String,
    /// Location of the parameter
    pub location: ParameterLocation,
    /// Whether the parameter is required
    pub required: bool,
    /// JSON schema for the parameter
    pub schema: Value,
    /// Description of the parameter
    pub description: Option<String>,
}

/// Location where a parameter appears in the request.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    /// Path parameter (e.g., /users/{id})
    Path,
    /// Query parameter (e.g., ?search=value)
    Query,
    /// Header parameter (e.g., X-Custom-Header)
    Header,
    /// Cookie parameter
    Cookie,
    /// Request body parameter
    Body,
}

impl std::fmt::Display for ParameterLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterLocation::Path => write!(f, "path"),
            ParameterLocation::Query => write!(f, "query"),
            ParameterLocation::Header => write!(f, "header"),
            ParameterLocation::Cookie => write!(f, "cookie"),
            ParameterLocation::Body => write!(f, "body"),
        }
    }
}

/// A parsed OpenAPI operation ready to be converted into a tool.
#[derive(Debug, Clone)]
pub struct ParsedOperation {
    /// Tool/operation name (snake_case)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Endpoint information
    pub endpoint: OperationEndpoint,
    /// Operation parameters
    pub parameters: Vec<ApiParameter>,
    /// Response schema (if available)
    pub response_schema: Option<Value>,
    /// Security requirements for this operation
    pub security: Vec<SecurityRequirement>,
}

/// Security requirement for an operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    /// Security scheme name (from components.securitySchemes)
    pub scheme_name: String,
    /// Required scopes (for OAuth2)
    pub scopes: Vec<String>,
}
