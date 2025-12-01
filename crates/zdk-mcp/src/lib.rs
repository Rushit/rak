//! MCP (Model Context Protocol) integration for ZDK
//!
//! This crate provides MCP support using the official rmcp Rust SDK.
//! It allows agents to dynamically load tools from external MCP servers.

pub mod client;
pub mod connection;
pub mod tool_wrapper;
pub mod toolset;
pub mod types;

// Re-exports
pub use client::McpClient;
pub use connection::StdioConnectionParams;
pub use tool_wrapper::McpToolWrapper;
pub use toolset::McpToolset;
pub use types::McpToolInfo;
