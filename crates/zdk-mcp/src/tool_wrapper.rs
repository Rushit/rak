//! MCP tool wrapper - bridges MCP tools to ZDK Tool trait

use crate::client::McpClient;
use crate::types::McpToolInfo;
use async_trait::async_trait;
use zdk_core::{Error, Result, Tool, ToolContext, ToolResponse};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Wrapper that adapts an MCP tool to the ZDK Tool trait
pub struct McpToolWrapper {
    mcp_tool: McpToolInfo,
    client: Arc<Mutex<Option<McpClient>>>,
}

impl McpToolWrapper {
    /// Create a new MCP tool wrapper
    pub fn new(mcp_tool: McpToolInfo, client: Arc<Mutex<Option<McpClient>>>) -> Self {
        Self { mcp_tool, client }
    }
}

#[async_trait]
impl Tool for McpToolWrapper {
    fn name(&self) -> &str {
        &self.mcp_tool.name
    }

    fn description(&self) -> &str {
        &self.mcp_tool.description
    }

    fn schema(&self) -> Value {
        self.mcp_tool.input_schema.clone()
    }

    async fn execute(&self, ctx: Arc<dyn ToolContext>, params: Value) -> Result<ToolResponse> {
        tracing::debug!(
            invocation_id = %ctx.invocation_id(),
            tool = %self.mcp_tool.name,
            "Executing MCP tool"
        );

        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| Error::Other(anyhow::anyhow!("MCP client not initialized")))?;

        let result = client
            .call_tool(&self.mcp_tool.name, params)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("MCP tool execution failed: {}", e)))?;

        tracing::debug!(
            invocation_id = %ctx.invocation_id(),
            tool = %self.mcp_tool.name,
            "MCP tool execution completed"
        );

        // Convert MCP result to ToolResponse
        Ok(ToolResponse {
            result: serde_json::to_value(&result)
                .map_err(|e| Error::Other(anyhow::anyhow!("Failed to serialize result: {}", e)))?,
        })
    }
}

