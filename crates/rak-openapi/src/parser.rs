//! OpenAPI specification parser.
//!
//! This module handles parsing OpenAPI v3.0+ specifications and extracting
//! operations that can be converted into tools.

use crate::error::{OpenApiError, Result};
use crate::types::{
    ApiParameter, OperationEndpoint, ParameterLocation, ParsedOperation, SecurityRequirement,
};
use openapiv3::{OpenAPI, Operation, Parameter, ParameterSchemaOrContent, ReferenceOr};
use serde_json::Value;
use tracing::{debug, warn};

/// Parser for OpenAPI specifications.
pub struct OpenApiParser {
    spec: OpenAPI,
}

impl OpenApiParser {
    /// Load and parse an OpenAPI spec from a file.
    ///
    /// Supports both JSON and YAML formats.
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        
        // Try JSON first, then YAML
        let spec = if path.ends_with(".json") {
            serde_json::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };
        
        Ok(Self { spec })
    }

    /// Load and parse an OpenAPI spec from a URL.
    pub async fn from_url(url: &str) -> Result<Self> {
        let response = reqwest::get(url).await?;
        let content = response.text().await?;
        
        // Try JSON first, then YAML
        let spec = serde_json::from_str(&content)
            .or_else(|_| serde_yaml::from_str(&content))?;
        
        Ok(Self { spec })
    }

    /// Parse an OpenAPI spec from a string.
    ///
    /// Automatically detects JSON or YAML format.
    pub fn from_str(content: &str) -> Result<Self> {
        // Try JSON first
        let spec = serde_json::from_str(content)
            .or_else(|_| serde_yaml::from_str(content))
            .map_err(|e| OpenApiError::ParseError(e.to_string()))?;
        
        Ok(Self { spec })
    }

    /// Parse the OpenAPI spec and extract all operations.
    pub fn parse(&self) -> Result<Vec<ParsedOperation>> {
        let mut operations = Vec::new();
        
        // Get base URL from servers
        let base_url = self
            .spec
            .servers
            .first()
            .and_then(|s| Some(s.url.clone()))
            .unwrap_or_default();
        
        debug!("Base URL: {}", base_url);
        
        // Iterate through all paths
        for (path, path_item_ref) in &self.spec.paths.paths {
            let path_item = match path_item_ref {
                ReferenceOr::Item(item) => item,
                ReferenceOr::Reference { .. } => {
                    warn!("Path references not yet supported: {}", path);
                    continue;
                }
            };
            
            // Check all HTTP methods
            let methods = [
                ("get", &path_item.get),
                ("post", &path_item.post),
                ("put", &path_item.put),
                ("delete", &path_item.delete),
                ("patch", &path_item.patch),
                ("head", &path_item.head),
                ("options", &path_item.options),
                ("trace", &path_item.trace),
            ];
            
            for (method_name, operation_opt) in methods {
                if let Some(operation) = operation_opt {
                    match self.parse_operation(
                        operation,
                        &base_url,
                        path,
                        method_name,
                        &path_item.parameters,
                    ) {
                        Ok(parsed_op) => operations.push(parsed_op),
                        Err(e) => {
                            warn!(
                                "Failed to parse operation {} {}: {}",
                                method_name.to_uppercase(),
                                path,
                                e
                            );
                        }
                    }
                }
            }
        }
        
        debug!("Parsed {} operations", operations.len());
        Ok(operations)
    }

    fn parse_operation(
        &self,
        operation: &Operation,
        base_url: &str,
        path: &str,
        method: &str,
        path_params: &[ReferenceOr<Parameter>],
    ) -> Result<ParsedOperation> {
        // Get operation ID or generate one
        let operation_id = operation
            .operation_id
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.generate_operation_id(path, method));
        
        let name = to_snake_case(&operation_id);
        
        // Get description
        let description = operation
            .description
            .as_ref()
            .or(operation.summary.as_ref())
            .map(|s| s.clone())
            .unwrap_or_else(|| format!("{} {}", method.to_uppercase(), path));
        
        // Parse parameters
        let mut parameters = Vec::new();
        
        // Add path-level parameters
        for param_ref in path_params {
            if let ReferenceOr::Item(param) = param_ref {
                if let Some(api_param) = self.parse_parameter(param)? {
                    parameters.push(api_param);
                }
            }
        }
        
        // Add operation-level parameters
        for param_ref in &operation.parameters {
            let param = match param_ref {
                ReferenceOr::Item(p) => p,
                ReferenceOr::Reference { .. } => {
                    warn!("Parameter references not yet supported");
                    continue;
                }
            };
            
            if let Some(api_param) = self.parse_parameter(param)? {
                parameters.push(api_param);
            }
        }
        
        // Parse request body if present
        if let Some(request_body_ref) = &operation.request_body {
            match request_body_ref {
                ReferenceOr::Item(request_body) => {
                    // Get JSON content type if available
                    if let Some(media_type) = request_body
                        .content
                        .get("application/json")
                        .or_else(|| request_body.content.values().next())
                    {
                        if let Some(schema_ref) = &media_type.schema {
                            match schema_ref {
                                ReferenceOr::Item(schema) => {
                                    let schema_json = serde_json::to_value(schema)
                                        .unwrap_or(Value::Object(Default::default()));
                                    
                                    parameters.push(ApiParameter {
                                        original_name: "body".to_string(),
                                        name: "body".to_string(),
                                        location: ParameterLocation::Body,
                                        required: request_body.required,
                                        schema: schema_json,
                                        description: request_body.description.clone(),
                                    });
                                }
                                ReferenceOr::Reference { .. } => {
                                    warn!("Schema references not yet supported");
                                }
                            }
                        }
                    }
                }
                ReferenceOr::Reference { .. } => {
                    warn!("RequestBody references not yet supported");
                }
            }
        }
        
        // Parse security requirements
        let security = operation
            .security
            .as_ref()
            .or(self.spec.security.as_ref())
            .map(|sec_reqs| {
                sec_reqs
                    .iter()
                    .flat_map(|req| {
                        req.iter().map(|(name, scopes)| SecurityRequirement {
                            scheme_name: name.clone(),
                            scopes: scopes.clone(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        // Get response schema (use 200 response if available)
        let response_schema = operation
            .responses
            .responses
            .get(&openapiv3::StatusCode::Code(200))
            .or_else(|| operation.responses.default.as_ref())
            .and_then(|resp_ref| match resp_ref {
                ReferenceOr::Item(response) => response
                    .content
                    .get("application/json")
                    .or_else(|| response.content.values().next())
                    .and_then(|media_type| media_type.schema.as_ref())
                    .and_then(|schema_ref| match schema_ref {
                        ReferenceOr::Item(schema) => {
                            serde_json::to_value(schema).ok()
                        }
                        ReferenceOr::Reference { .. } => None,
                    }),
                ReferenceOr::Reference { .. } => None,
            });
        
        Ok(ParsedOperation {
            name,
            description,
            endpoint: OperationEndpoint {
                base_url: base_url.to_string(),
                path: path.to_string(),
                method: method.to_uppercase(),
            },
            parameters,
            response_schema,
            security,
        })
    }

    fn parse_parameter(&self, param: &Parameter) -> Result<Option<ApiParameter>> {
        let param_data = match param {
            Parameter::Query { parameter_data, .. } => (parameter_data, ParameterLocation::Query),
            Parameter::Header { parameter_data, .. } => (parameter_data, ParameterLocation::Header),
            Parameter::Path { parameter_data, .. } => (parameter_data, ParameterLocation::Path),
            Parameter::Cookie { parameter_data, .. } => (parameter_data, ParameterLocation::Cookie),
        };
        
        let (data, location) = param_data;
        
        // Get schema
        let schema = match &data.format {
            ParameterSchemaOrContent::Schema(schema_ref) => match schema_ref {
                ReferenceOr::Item(schema) => {
                    serde_json::to_value(schema).unwrap_or(Value::Object(Default::default()))
                }
                ReferenceOr::Reference { .. } => {
                    warn!("Schema references not yet supported");
                    return Ok(None);
                }
            },
            ParameterSchemaOrContent::Content(_) => {
                warn!("Parameter content not yet supported");
                return Ok(None);
            }
        };
        
        Ok(Some(ApiParameter {
            original_name: data.name.clone(),
            name: to_snake_case(&data.name),
            location,
            required: data.required,
            schema,
            description: data.description.clone(),
        }))
    }

    fn generate_operation_id(&self, path: &str, method: &str) -> String {
        // Generate operation ID like: get_users_id
        let path_parts: Vec<&str> = path
            .split('/')
            .filter(|s| !s.is_empty() && !s.starts_with('{'))
            .collect();
        
        let path_str = if path_parts.is_empty() {
            "root".to_string()
        } else {
            path_parts.join("_")
        };
        
        format!("{}_{}", method, path_str)
    }
}

/// Convert a string to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_upper = false;
    
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_upper {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_upper = true;
        } else {
            if c == '-' || c == ' ' {
                result.push('_');
            } else {
                result.push(c);
            }
            prev_upper = false;
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("getUserById"), "get_user_by_id");
        assert_eq!(to_snake_case("HTTPResponse"), "httpresponse"); // Consecutive capitals stay together
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("kebab-case"), "kebab_case");
        assert_eq!(to_snake_case("listUsers"), "list_users");
    }
}

