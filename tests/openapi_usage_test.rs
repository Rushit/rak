//! Integration test for OpenAPI tool generation from a spec.
//!
//! This test demonstrates how to:
//! 1. Load an OpenAPI specification
//! 2. Generate tools automatically
//! 3. Configure authentication
//! 4. Use the tools in an agent
//!
//! Run with:
//!   cargo test openapi_usage_test -- --ignored --nocapture

use rak_openapi::{AuthConfig, OpenApiToolset};
use tracing::{error, info};
use tracing_subscriber;

#[tokio::test]
#[ignore] // Optional test - run with: cargo test -- --ignored
async fn test_openapi_toolset_generation() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("RAK OpenAPI Tool Generator Test");

    // Example 1: Load from YAML string
    let yaml_spec = r#"
openapi: 3.0.0
info:
  title: Example API
  version: 1.0.0
servers:
  - url: https://api.example.com
paths:
  /users:
    get:
      operationId: listUsers
      summary: List all users
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
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
    delete:
      operationId: deleteUser
      summary: Delete user by ID
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
      responses:
        '204':
          description: Deleted
"#;

    match OpenApiToolset::from_str(yaml_spec) {
        Ok(toolset) => {
            info!("✓ Successfully parsed OpenAPI spec");
            assert_eq!(toolset.len(), 3, "Should generate 3 tools");

            info!("\nAvailable tools:");
            for name in toolset.tool_names() {
                if let Some(tool) = toolset.get_tool(&name) {
                    info!("  - {}: {}", tool.name(), tool.description());
                }
            }

            // Example 2: Add authentication
            let toolset_with_auth = toolset
                .with_auth(AuthConfig::api_key_header("X-API-Key", "demo-key-12345"));
            info!("\n✓ Configured API Key authentication for all tools");

            // Example 3: Use specific tools
            if let Some(tool) = toolset_with_auth.get_tool("get_user") {
                info!("\nTool: {}", tool.name());
                info!("Description: {}", tool.description());
                info!("Schema: {}", serde_json::to_string_pretty(&tool.schema()).unwrap());
            }

            info!("\n✓ Tools are ready to be used in agents!");
        }
        Err(e) => {
            error!("Failed to parse OpenAPI spec: {}", e);
            panic!("OpenAPI spec parsing failed: {}", e);
        }
    }

    // Example 4: Bearer token auth
    info!("\n---");
    info!("Example: Bearer Token Authentication");

    let toolset_bearer = OpenApiToolset::from_str(yaml_spec)
        .unwrap()
        .with_auth(AuthConfig::bearer("my-bearer-token"));
    info!("✓ Configured Bearer token authentication");
    assert_eq!(toolset_bearer.len(), 3);

    // Example 5: Basic auth
    info!("\n---");
    info!("Example: Basic Authentication");

    let toolset_basic = OpenApiToolset::from_str(yaml_spec)
        .unwrap()
        .with_auth(AuthConfig::basic("username", "password"));
    info!("✓ Configured Basic authentication");
    assert_eq!(toolset_basic.len(), 3);

    info!("\n---");
    info!("✓ OpenAPI tool generation test complete!");
}

