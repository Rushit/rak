//! Tool system for ZDK
//!
//! This crate provides the tool execution framework, including:
//! - Tool trait and utilities
//! - Function tools with automatic schema generation
//! - Built-in tools (calculator, search, etc.)
//! - Tool context management

pub mod builtin;
pub mod context;
pub mod function_tool;
pub mod schema;

// Re-exports
pub use context::DefaultToolContext;
pub use function_tool::FunctionTool;
pub use schema::{ToolSchema, generate_schema};

// Re-export core types
pub use zdk_core::{Result, Tool, ToolContext, ToolResponse};
