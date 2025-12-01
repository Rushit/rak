//! MCP client wrapper using rmcp SDK

use crate::connection::StdioConnectionParams;
use crate::types::{McpToolInfo, ToolContent};
use anyhow::Result;
use rmcp::model::CallToolRequestParam;
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::TokioChildProcess;
use rmcp::ServiceExt;
use serde_json::Value;
use tokio::process::Command;

/// MCP client that wraps the rmcp SDK
///
/// This wraps a RunningService from rmcp to provide a simpler API
pub struct McpClient {
    service: RunningService<RoleClient, ()>,
}

impl McpClient {
    /// Create a new MCP client by spawning a subprocess
    pub async fn new(params: StdioConnectionParams) -> Result<Self> {
        tracing::debug!(
            command = %params.command,
            args = ?params.args,
            "Initializing MCP client with rmcp SDK"
        );

        // Build the tokio Command
        let mut command = Command::new(&params.command);
        for arg in &params.args {
            command.arg(arg);
        }
        for (key, value) in &params.env {
            command.env(key, value);
        }

        // Create transport using rmcp's TokioChildProcess
        let transport = TokioChildProcess::new(command)?;

        // Initialize the MCP server connection
        let service = ().serve(transport).await?;

        tracing::info!(
            server_info = ?service.peer_info(),
            "MCP client initialized successfully"
        );

        Ok(Self { service })
    }

    /// List all available tools from the MCP server
    pub async fn list_tools(&self) -> Result<Vec<McpToolInfo>> {
        tracing::debug!("Listing tools from MCP server");

        let response = self.service.list_tools(Default::default()).await?;

        let tools: Vec<McpToolInfo> = response
            .tools
            .into_iter()
            .map(|tool| McpToolInfo {
                name: tool.name.into_owned(),
                description: tool.description.map(|d| d.into_owned()).unwrap_or_default(),
                input_schema: serde_json::Value::Object((*tool.input_schema).clone()),
            })
            .collect();

        tracing::debug!(count = tools.len(), "Retrieved tools from MCP server");

        Ok(tools)
    }

    /// Call a tool on the MCP server
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Vec<ToolContent>> {
        tracing::debug!(tool = %name, "Calling MCP tool");

        let params = CallToolRequestParam {
            name: name.to_string().into(),
            arguments: arguments.as_object().cloned(),
        };

        let response = self.service.call_tool(params).await?;

        // Convert content to JSON values
        let content = response
            .content
            .into_iter()
            .map(|c| serde_json::to_value(c).unwrap_or(Value::Null))
            .collect();

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_params() {
        let params = StdioConnectionParams::new("test-command")
            .arg("--flag")
            .env("KEY", "value");

        assert_eq!(params.command, "test-command");
        assert_eq!(params.args.len(), 1);
        assert_eq!(params.env.len(), 1);
    }
}
