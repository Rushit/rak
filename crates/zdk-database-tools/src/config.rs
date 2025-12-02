//! Configuration types for database tools

use std::collections::HashSet;

/// SQL operations that can be performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SqlOperation {
    Select,
    Insert,
    Update,
    Delete,
    CreateTable,
    DropTable,
    AlterTable,
    CreateIndex,
    DropIndex,
}

/// Configuration for database tools
#[derive(Debug, Clone)]
pub struct DatabaseToolConfig {
    /// Whether the tool is read-only (default: true)
    pub read_only: bool,
    /// Maximum number of rows to return (default: 1000)
    pub max_rows: usize,
    /// Query timeout in seconds (default: 30)
    pub timeout_secs: u64,
    /// Allowed SQL operations
    pub allowed_operations: HashSet<SqlOperation>,
}

impl Default for DatabaseToolConfig {
    fn default() -> Self {
        let mut allowed_operations = HashSet::new();
        allowed_operations.insert(SqlOperation::Select);

        Self {
            read_only: true,
            max_rows: 1000,
            timeout_secs: 30,
            allowed_operations,
        }
    }
}

impl DatabaseToolConfig {
    /// Create a new config with write permissions enabled
    pub fn with_write_enabled() -> Self {
        let mut allowed_operations = std::collections::HashSet::new();
        allowed_operations.insert(SqlOperation::Insert);
        allowed_operations.insert(SqlOperation::Update);
        allowed_operations.insert(SqlOperation::Delete);
        Self {
            read_only: false,
            allowed_operations,
            ..Default::default()
        }
    }

    /// Create a new config with DDL permissions enabled
    pub fn with_ddl_enabled() -> Self {
        let mut config = Self::with_write_enabled();
        config.allowed_operations.insert(SqlOperation::CreateTable);
        config.allowed_operations.insert(SqlOperation::DropTable);
        config.allowed_operations.insert(SqlOperation::AlterTable);
        config.allowed_operations.insert(SqlOperation::CreateIndex);
        config.allowed_operations.insert(SqlOperation::DropIndex);
        config
    }
}
