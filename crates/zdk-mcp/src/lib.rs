//! MCP (Model Context Protocol) integration for RAK
//!
//! This crate provides MCP support using the official rmcp Rust SDK.
//! It allows agents to dynamically load tools from external MCP servers.

pub mod client;
pub mod connection;
pub mod toolset;
pub mod tool_wrapper;
pub mod types;

// Re-exports
pub use client::McpClient;
pub use connection::StdioConnectionParams;
pub use toolset::McpToolset;
pub use tool_wrapper::McpToolWrapper;
pub use types::McpToolInfo;

