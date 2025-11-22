//! Types for MCP integration

use serde::{Deserialize, Serialize};

/// Information about an MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool execution result content
pub type ToolContent = serde_json::Value;

