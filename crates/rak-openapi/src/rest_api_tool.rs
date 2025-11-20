//! REST API tool implementation.

use crate::auth::AuthConfig;
use crate::error::{OpenApiError, Result};
use crate::types::{ApiParameter, OperationEndpoint, ParameterLocation, ParsedOperation};
use async_trait::async_trait;
use rak_core::ToolResponse;
use rak_tool::{Tool, ToolContext};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, instrument};

/// A tool that executes a REST API operation.
///
/// Each `RestApiTool` represents a single API endpoint/operation from an OpenAPI spec.
/// It implements the `Tool` trait and can be used directly in RAK agents.
pub struct RestApiTool {
    /// Tool name (operation ID in snake_case)
    pub(crate) name: String,
    /// Tool description
    pub(crate) description: String,
    /// API endpoint information
    pub(crate) endpoint: OperationEndpoint,
    /// Operation parameters
    pub(crate) parameters: Vec<ApiParameter>,
    /// Response schema
    pub(crate) response_schema: Option<Value>,
    /// Authentication configuration
    pub(crate) auth: AuthConfig,
    /// HTTP client
    pub(crate) client: reqwest::Client,
}

impl RestApiTool {
    /// Create a new REST API tool from a parsed operation.
    pub fn from_parsed_operation(operation: ParsedOperation) -> Self {
        Self {
            name: operation.name,
            description: operation.description,
            endpoint: operation.endpoint,
            parameters: operation.parameters,
            response_schema: operation.response_schema,
            auth: AuthConfig::None,
            client: reqwest::Client::new(),
        }
    }

    /// Set authentication configuration for this tool.
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    /// Build an HTTP request from the tool parameters.
    #[instrument(skip(self, params))]
    fn build_request(&self, params: &Value) -> Result<reqwest::RequestBuilder> {
        let params_map = params.as_object().ok_or_else(|| {
            OpenApiError::InvalidParameter("root".to_string(), "Expected object".to_string())
        })?;

        // Extract parameters by location
        let mut path_params: HashMap<String, String> = HashMap::new();
        let mut query_params: Vec<(String, String)> = Vec::new();
        let mut header_params: Vec<(String, String)> = Vec::new();
        let mut body_value: Option<Value> = None;

        for param in &self.parameters {
            let value = params_map.get(&param.name);

            // Check required parameters
            if param.required && value.is_none() {
                return Err(OpenApiError::MissingParameter(param.name.clone()));
            }

            if let Some(val) = value {
                match param.location {
                    ParameterLocation::Path => {
                        path_params.insert(
                            param.original_name.clone(),
                            val.as_str()
                                .unwrap_or(&val.to_string())
                                .to_string(),
                        );
                    }
                    ParameterLocation::Query => {
                        if !val.is_null() {
                            query_params.push((
                                param.original_name.clone(),
                                val.as_str()
                                    .unwrap_or(&val.to_string())
                                    .to_string(),
                            ));
                        }
                    }
                    ParameterLocation::Header => {
                        header_params.push((
                            param.original_name.clone(),
                            val.as_str()
                                .unwrap_or(&val.to_string())
                                .to_string(),
                        ));
                    }
                    ParameterLocation::Body => {
                        body_value = Some(val.clone());
                    }
                    ParameterLocation::Cookie => {
                        // Cookie handling not yet implemented
                        debug!("Cookie parameters not yet supported: {}", param.name);
                    }
                }
            }
        }

        // Build URL with path parameters
        let mut url = format!("{}{}", self.endpoint.base_url, self.endpoint.path);
        for (key, value) in path_params {
            url = url.replace(&format!("{{{}}}", key), &value);
        }

        debug!("Request URL: {} {}", self.endpoint.method, url);

        // Create request builder
        let method = reqwest::Method::from_bytes(self.endpoint.method.as_bytes())
            .map_err(|e| OpenApiError::Other(format!("Invalid HTTP method: {}", e)))?;

        let mut builder = self.client.request(method, &url);

        // Add query parameters
        if !query_params.is_empty() {
            builder = builder.query(&query_params);
        }

        // Add headers
        for (name, value) in header_params {
            builder = builder.header(name, value);
        }

        // Add body
        if let Some(body) = body_value {
            builder = builder.json(&body);
        }

        // Apply authentication
        builder = self.auth.apply_to_request(builder);

        Ok(builder)
    }

    /// Execute the HTTP request and parse the response.
    #[instrument(skip(self, builder))]
    async fn execute_request(&self, builder: reqwest::RequestBuilder) -> Result<Value> {
        let response = builder.send().await?;
        let status = response.status();

        debug!("Response status: {}", status);

        if status.is_success() {
            // Get response bytes once
            let bytes = response.bytes().await?;
            
            // Try to parse as JSON first
            match serde_json::from_slice::<Value>(&bytes) {
                Ok(json) => Ok(json),
                Err(_) => {
                    // If not JSON, return as text
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    Ok(json!({ "text": text }))
                }
            }
        } else {
            // Return error as tool response (so LLM can see it and retry)
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!(
                "API request failed: {} {} - Status: {} - Error: {}",
                self.endpoint.method, self.endpoint.path, status, error_text
            );

            Ok(json!({
                "error": format!(
                    "Tool '{}' execution failed. Status Code: {}. Error: {}. Analyze your inputs and retry if applicable, but do not retry more than 3 times.",
                    self.name,
                    status.as_u16(),
                    error_text
                )
            }))
        }
    }
}

#[async_trait]
impl Tool for RestApiTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn schema(&self) -> Value {
        // Build JSON schema from parameters
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &self.parameters {
            properties.insert(param.name.clone(), param.schema.clone());

            if param.required {
                required.push(param.name.clone());
            }
        }

        let mut schema = json!({
            "type": "object",
            "description": self.description,
            "properties": properties,
        });

        if !required.is_empty() {
            schema["required"] = json!(required);
        }

        schema
    }

    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        params: Value,
    ) -> rak_core::Result<ToolResponse> {
        debug!("Executing REST API tool: {}", self.name);
        debug!("Parameters: {:?}", params);

        // Build request
        let builder = self.build_request(&params)
            .map_err(|e| rak_core::Error::ToolFailed {
                tool: self.name.clone(),
                source: e.into(),
            })?;

        // Execute request
        let result = self.execute_request(builder).await
            .map_err(|e| rak_core::Error::ToolFailed {
                tool: self.name.clone(),
                source: e.into(),
            })?;

        Ok(ToolResponse { result })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_operation() -> ParsedOperation {
        ParsedOperation {
            name: "get_user".to_string(),
            description: "Get user by ID".to_string(),
            endpoint: OperationEndpoint {
                base_url: "https://api.example.com".to_string(),
                path: "/users/{id}".to_string(),
                method: "GET".to_string(),
            },
            parameters: vec![ApiParameter {
                original_name: "id".to_string(),
                name: "id".to_string(),
                location: ParameterLocation::Path,
                required: true,
                schema: json!({"type": "string"}),
                description: Some("User ID".to_string()),
            }],
            response_schema: Some(json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "name": {"type": "string"}
                }
            })),
            security: vec![],
        }
    }

    #[test]
    fn test_rest_api_tool_creation() {
        let operation = create_test_operation();
        let tool = RestApiTool::from_parsed_operation(operation);

        assert_eq!(tool.name(), "get_user");
        assert_eq!(tool.description(), "Get user by ID");
    }

    #[test]
    fn test_tool_schema_generation() {
        let operation = create_test_operation();
        let tool = RestApiTool::from_parsed_operation(operation);
        let schema = tool.schema();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["required"].is_array());

        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }
}

