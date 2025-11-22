//! MCP Toolset implementation

use crate::client::McpClient;
use crate::connection::StdioConnectionParams;
use crate::tool_wrapper::McpToolWrapper;
use async_trait::async_trait;
use rak_core::{InvocationContext, Result, Tool, Toolset};
use std::sync::Arc;
use tokio::sync::Mutex;

/// A toolset that dynamically loads tools from an MCP server
pub struct McpToolset {
    name: String,
    connection_params: StdioConnectionParams,
    client: Arc<Mutex<Option<McpClient>>>,
    tool_filter: Option<Vec<String>>,
}

impl McpToolset {
    /// Create a new McpToolset builder
    pub fn builder() -> McpToolsetBuilder {
        McpToolsetBuilder::new()
    }
}

#[async_trait]
impl Toolset for McpToolset {
    fn name(&self) -> &str {
        &self.name
    }

    async fn get_tools(&self, _ctx: &dyn InvocationContext) -> Result<Vec<Arc<dyn Tool>>> {
        tracing::info!(toolset = %self.name, "Loading tools from MCP server");

        // Create or reuse MCP client
        let mut client_guard = self.client.lock().await;
        if client_guard.is_none() {
            let client = McpClient::new(self.connection_params.clone())
                .await
                .map_err(|e| rak_core::Error::Other(anyhow::anyhow!("Failed to create MCP client: {}", e)))?;
            *client_guard = Some(client);
        }

        let client = client_guard
            .as_ref()
            .ok_or_else(|| rak_core::Error::Other(anyhow::anyhow!("MCP client not initialized")))?;

        // List tools from MCP server
        let mcp_tools = client
            .list_tools()
            .await
            .map_err(|e| rak_core::Error::Other(anyhow::anyhow!("Failed to list tools: {}", e)))?;

        // Filter tools if specified
        let filtered = if let Some(filter) = &self.tool_filter {
            mcp_tools
                .into_iter()
                .filter(|t| filter.contains(&t.name))
                .collect()
        } else {
            mcp_tools
        };

        tracing::info!(
            toolset = %self.name,
            count = filtered.len(),
            "Loaded tools from MCP server"
        );

        // Wrap each MCP tool as RAK Tool
        let rak_tools: Vec<Arc<dyn Tool>> = filtered
            .into_iter()
            .map(|mcp_tool| {
                Arc::new(McpToolWrapper::new(mcp_tool, self.client.clone())) as Arc<dyn Tool>
            })
            .collect();

        Ok(rak_tools)
    }
}

/// Builder for McpToolset
pub struct McpToolsetBuilder {
    name: Option<String>,
    connection_params: Option<StdioConnectionParams>,
    tool_filter: Option<Vec<String>>,
}

impl McpToolsetBuilder {
    fn new() -> Self {
        Self {
            name: None,
            connection_params: None,
            tool_filter: None,
        }
    }

    /// Set the name of the toolset
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the connection parameters
    pub fn connection(mut self, params: StdioConnectionParams) -> Self {
        self.connection_params = Some(params);
        self
    }

    /// Set a filter for which tools to include
    pub fn tool_filter(mut self, filter: Vec<String>) -> Self {
        self.tool_filter = Some(filter);
        self
    }

    /// Build the McpToolset
    pub fn build(self) -> Result<McpToolset> {
        let name = self
            .name
            .ok_or_else(|| rak_core::Error::Other(anyhow::anyhow!("Toolset name is required")))?;
        let connection_params = self.connection_params.ok_or_else(|| {
            rak_core::Error::Other(anyhow::anyhow!("Connection parameters are required"))
        })?;

        Ok(McpToolset {
            name,
            connection_params,
            client: Arc::new(Mutex::new(None)),
            tool_filter: self.tool_filter,
        })
    }
}

