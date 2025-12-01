//! OpenAPI toolset container.

use crate::auth::AuthConfig;
use crate::error::Result;
use crate::parser::OpenApiParser;
use crate::rest_api_tool::RestApiTool;
use crate::types::ParsedOperation;
use zdk_tool::Tool;
use std::sync::Arc;
use tracing::{debug, info};

/// A collection of tools generated from an OpenAPI specification.
///
/// The `OpenApiToolset` parses an OpenAPI spec and creates a `RestApiTool` for
/// each operation defined in the spec. All tools can then be used in ZDK agents.
///
/// # Example
///
/// ```no_run
/// use zdk_openapi::{OpenApiToolset, AuthConfig};
/// use std::env;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// // Load from file
/// let toolset = OpenApiToolset::from_file("./api/openapi.yaml")?
///     .with_auth(AuthConfig::api_key_header("X-API-Key", env::var("API_KEY")?));
///
/// // Get all tools
/// let tools = toolset.tools();
/// println!("Generated {} tools", tools.len());
/// # Ok(())
/// # }
/// ```
pub struct OpenApiToolset {
    /// All tools generated from the spec
    tools: Vec<Arc<dyn Tool>>,
    /// Original parsed operations (for recreating tools with auth)
    operations: Vec<ParsedOperation>,
}

impl OpenApiToolset {
    /// Load an OpenAPI spec from a file and generate tools.
    ///
    /// Supports both JSON and YAML formats.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zdk_openapi::OpenApiToolset;
    ///
    /// let toolset = OpenApiToolset::from_file("./api/openapi.yaml")?;
    /// # Ok::<(), zdk_openapi::OpenApiError>(())
    /// ```
    pub fn from_file(path: &str) -> Result<Self> {
        info!("Loading OpenAPI spec from file: {}", path);
        let parser = OpenApiParser::from_file(path)?;
        Self::from_parser(parser)
    }

    /// Load an OpenAPI spec from a URL and generate tools.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zdk_openapi::OpenApiToolset;
    ///
    /// # async fn example() -> Result<(), zdk_openapi::OpenApiError> {
    /// let toolset = OpenApiToolset::from_url("https://api.example.com/openapi.json").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_url(url: &str) -> Result<Self> {
        info!("Loading OpenAPI spec from URL: {}", url);
        let parser = OpenApiParser::from_url(url).await?;
        Self::from_parser(parser)
    }

    /// Parse an OpenAPI spec from a string and generate tools.
    ///
    /// Automatically detects JSON or YAML format.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zdk_openapi::OpenApiToolset;
    ///
    /// let spec = r#"
    /// openapi: 3.0.0
    /// info:
    ///   title: Example API
    ///   version: 1.0.0
    /// paths:
    ///   /users:
    ///     get:
    ///       operationId: listUsers
    ///       summary: List all users
    /// "#;
    ///
    /// let toolset = OpenApiToolset::from_str(spec)?;
    /// # Ok::<(), zdk_openapi::OpenApiError>(())
    /// ```
    pub fn from_str(content: &str) -> Result<Self> {
        debug!("Parsing OpenAPI spec from string");
        let parser = OpenApiParser::from_str(content)?;
        Self::from_parser(parser)
    }

    /// Create toolset from an OpenAPI parser.
    fn from_parser(parser: OpenApiParser) -> Result<Self> {
        let operations = parser.parse()?;
        info!("Parsed {} operations from OpenAPI spec", operations.len());

        let tools: Vec<Arc<dyn Tool>> = operations
            .iter()
            .map(|op| {
                let tool = RestApiTool::from_parsed_operation(op.clone());
                Arc::new(tool) as Arc<dyn Tool>
            })
            .collect();

        debug!("Generated {} tools", tools.len());

        Ok(Self { tools, operations })
    }

    /// Configure authentication for all tools in the toolset.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zdk_openapi::{OpenApiToolset, AuthConfig};
    ///
    /// let toolset = OpenApiToolset::from_file("./api/openapi.yaml")?
    ///     .with_auth(AuthConfig::bearer("my-token"));
    /// # Ok::<(), zdk_openapi::OpenApiError>(())
    /// ```
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        info!("Configuring authentication for all tools");

        // Recreate tools from operations with auth
        self.tools = self
            .operations
            .iter()
            .map(|op| {
                let tool = RestApiTool::from_parsed_operation(op.clone())
                    .with_auth(auth.clone());
                Arc::new(tool) as Arc<dyn Tool>
            })
            .collect();

        self
    }

    /// Get all tools generated from the OpenAPI spec.
    ///
    /// Returns a vector of tools that can be used in ZDK agents.
    pub fn tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }

    /// Get a specific tool by name.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zdk_openapi::OpenApiToolset;
    ///
    /// let toolset = OpenApiToolset::from_file("./api/openapi.yaml")?;
    /// let tool = toolset.get_tool("get_user");
    /// # Ok::<(), zdk_openapi::OpenApiError>(())
    /// ```
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == name).cloned()
    }

    /// Get the names of all tools in the toolset.
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.name().to_string()).collect()
    }

    /// Get the number of tools in the toolset.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the toolset is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SPEC: &str = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
servers:
  - url: https://api.example.com
paths:
  /users:
    get:
      operationId: listUsers
      summary: List all users
      responses:
        '200':
          description: Success
  /users/{id}:
    get:
      operationId: getUser
      summary: Get user by ID
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Success
"#;

    #[test]
    fn test_toolset_from_str() {
        let toolset = OpenApiToolset::from_str(TEST_SPEC).unwrap();
        assert_eq!(toolset.len(), 2);
        assert!(!toolset.is_empty());

        let names = toolset.tool_names();
        assert!(names.contains(&"list_users".to_string()));
        assert!(names.contains(&"get_user".to_string()));
    }

    #[test]
    fn test_get_tool_by_name() {
        let toolset = OpenApiToolset::from_str(TEST_SPEC).unwrap();

        let tool = toolset.get_tool("list_users");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "list_users");

        let missing = toolset.get_tool("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_with_auth() {
        let toolset = OpenApiToolset::from_str(TEST_SPEC)
            .unwrap()
            .with_auth(AuthConfig::bearer("test-token"));

        assert_eq!(toolset.len(), 2);
    }
}

