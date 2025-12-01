//! Database tools for ZDK agents
//!
//! This crate provides database interaction tools for PostgreSQL and SQLite,
//! with security features like read-only mode, parameter binding, and query limits.

pub mod config;
pub mod postgres;
pub mod sqlite;
pub mod types;

// Re-exports
pub use config::{DatabaseToolConfig, SqlOperation};
pub use postgres::{create_postgres_tools, create_postgres_tools_with_config};
pub use sqlite::{create_sqlite_tools, create_sqlite_tools_with_config};
pub use types::{ColumnInfo, ConstraintInfo, IndexInfo, TableInfo, TableSchema};
