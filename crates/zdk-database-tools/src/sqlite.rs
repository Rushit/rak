//! SQLite database tools

use crate::config::{DatabaseToolConfig, SqlOperation};
use crate::types::{ColumnInfo, ConstraintInfo, IndexInfo, TableInfo, TableSchema};
use zdk_core::{Error as ZError, Result as ZResult, Tool, ToolContext, ToolResponse};
use zdk_tool::{FunctionTool, ToolSchema};
use async_trait::async_trait;
use sqlx::{sqlite::SqlitePoolOptions, Column, Pool, Row, Sqlite};
use std::sync::Arc;
use std::time::Duration;

/// Create SQLite tools with default configuration (read-only)
pub async fn create_sqlite_tools(connection_string: &str) -> ZResult<Vec<Arc<dyn Tool>>> {
    create_sqlite_tools_with_config(connection_string, DatabaseToolConfig::default()).await
}

/// Create SQLite tools with custom configuration
pub async fn create_sqlite_tools_with_config(
    connection_string: &str,
    config: DatabaseToolConfig,
) -> ZResult<Vec<Arc<dyn Tool>>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(config.timeout_secs))
        .connect(connection_string)
        .await
        .map_err(|e| ZError::Other(anyhow::anyhow!("Failed to connect to SQLite: {}", e)))?;

    let mut tools: Vec<Arc<dyn Tool>> = Vec::new();

    // List tables tool
    tools.push(Arc::new(create_list_tables_tool(pool.clone())?));

    // Describe table tool
    tools.push(Arc::new(create_describe_table_tool(pool.clone())?));

    // Query tool (always available)
    tools.push(Arc::new(create_query_tool(pool.clone(), config.clone())?));

    // Execute tool (only if not read-only)
    if !config.read_only {
        tools.push(Arc::new(create_execute_tool(pool.clone(), config.clone())?));
    }

    Ok(tools)
}

/// Create a tool to list all tables
fn create_list_tables_tool(pool: Pool<Sqlite>) -> ZResult<FunctionTool> {
    let schema = ToolSchema::new().build();

    FunctionTool::builder()
        .name("sqlite_list_tables")
        .description("List all tables in the SQLite database")
        .schema(schema)
        .execute(move |ctx, _params| {
            let pool = pool.clone();
            async move {
                tracing::debug!(
                    invocation_id = %ctx.invocation_id(),
                    "Listing SQLite tables"
                );

                let query = r#"
                    SELECT name
                    FROM sqlite_master
                    WHERE type='table'
                    AND name NOT LIKE 'sqlite_%'
                    ORDER BY name
                "#;

                let rows = sqlx::query(query)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| ZError::Other(anyhow::anyhow!("Failed to list tables: {}", e)))?;

                let tables: Vec<TableInfo> = rows
                    .iter()
                    .map(|row| TableInfo {
                        name: row.get("name"),
                        row_count: 0, // Would require separate COUNT queries
                        size_bytes: None,
                    })
                    .collect();

                Ok(ToolResponse {
                    result: serde_json::to_value(&tables)
                        .map_err(|e| ZError::Other(anyhow::anyhow!("Serialization error: {}", e)))?,
                })
            }
        })
        .build()
}

/// Create a tool to describe a table's schema
fn create_describe_table_tool(pool: Pool<Sqlite>) -> ZResult<FunctionTool> {
    let schema = ToolSchema::new()
        .property("table_name", "string", "Name of the table to describe")
        .required("table_name")
        .build();

    FunctionTool::builder()
        .name("sqlite_describe_table")
        .description("Get the schema information for a SQLite table")
        .schema(schema)
        .execute(move |ctx, params| {
            let pool = pool.clone();
            async move {
                let table_name = params["table_name"]
                    .as_str()
                    .ok_or_else(|| ZError::Other(anyhow::anyhow!("Missing table_name parameter")))?;

                tracing::debug!(
                    invocation_id = %ctx.invocation_id(),
                    table = %table_name,
                    "Describing SQLite table"
                );

                // Get column information using PRAGMA
                let query = format!("PRAGMA table_info({})", table_name);

                let rows = sqlx::query(&query)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| ZError::Other(anyhow::anyhow!("Failed to describe table: {}", e)))?;

                let columns: Vec<ColumnInfo> = rows
                    .iter()
                    .map(|row| ColumnInfo {
                        name: row.get("name"),
                        data_type: row.get("type"),
                        nullable: row.get::<i32, _>("notnull") == 0,
                        default_value: row.get("dflt_value"),
                    })
                    .collect();

                let table_schema = TableSchema {
                    table_name: table_name.to_string(),
                    columns,
                    indexes: Vec::new(), // Would need PRAGMA index_list
                    constraints: Vec::new(), // Would need additional queries
                };

                Ok(ToolResponse {
                    result: serde_json::to_value(&table_schema)
                        .map_err(|e| ZError::Other(anyhow::anyhow!("Serialization error: {}", e)))?,
                })
            }
        })
        .build()
}

/// Create a tool to execute SELECT queries
fn create_query_tool(pool: Pool<Sqlite>, config: DatabaseToolConfig) -> ZResult<FunctionTool> {
    let schema = ToolSchema::new()
        .property("sql", "string", "SQL SELECT query to execute")
        .required("sql")
        .build();

    FunctionTool::builder()
        .name("sqlite_query")
        .description("Execute a SELECT query on the SQLite database")
        .schema(schema)
        .execute(move |ctx, params| {
            let pool = pool.clone();
            let max_rows = config.max_rows;
            async move {
                let sql = params["sql"]
                    .as_str()
                    .ok_or_else(|| ZError::Other(anyhow::anyhow!("Missing sql parameter")))?;

                // Basic SQL validation - ensure it's a SELECT query
                let sql_upper = sql.trim().to_uppercase();
                if !sql_upper.starts_with("SELECT") {
                    return Err(ZError::Other(anyhow::anyhow!(
                        "Only SELECT queries are allowed in read-only mode"
                    )));
                }

                tracing::debug!(
                    invocation_id = %ctx.invocation_id(),
                    sql = %sql,
                    "Executing SQLite query"
                );

                // Add LIMIT if not present
                let final_sql = if !sql_upper.contains("LIMIT") {
                    format!("{} LIMIT {}", sql, max_rows)
                } else {
                    sql.to_string()
                };

                let rows = sqlx::query(&final_sql)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| ZError::Other(anyhow::anyhow!("Query failed: {}", e)))?;

                // Convert rows to JSON
                let result: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|row| {
                        let mut map = serde_json::Map::new();
                        for (i, column) in row.columns().iter().enumerate() {
                            let value: Option<String> = row.try_get(i).ok();
                            map.insert(
                                column.name().to_string(),
                                value.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
                            );
                        }
                        serde_json::Value::Object(map)
                    })
                    .collect();

                Ok(ToolResponse {
                    result: serde_json::json!({
                        "rows": result,
                        "row_count": result.len(),
                    }),
                })
            }
        })
        .build()
}

/// Create a tool to execute INSERT/UPDATE/DELETE queries
fn create_execute_tool(pool: Pool<Sqlite>, config: DatabaseToolConfig) -> ZResult<FunctionTool> {
    let schema = ToolSchema::new()
        .property("sql", "string", "SQL query to execute (INSERT, UPDATE, DELETE)")
        .required("sql")
        .build();

    FunctionTool::builder()
        .name("sqlite_execute")
        .description("Execute an INSERT, UPDATE, or DELETE query on the SQLite database")
        .schema(schema)
        .execute(move |ctx, params| {
            let pool = pool.clone();
            async move {
                let sql = params["sql"]
                    .as_str()
                    .ok_or_else(|| ZError::Other(anyhow::anyhow!("Missing sql parameter")))?;

                // Basic SQL validation
                let sql_upper = sql.trim().to_uppercase();
                let is_allowed = sql_upper.starts_with("INSERT")
                    || sql_upper.starts_with("UPDATE")
                    || sql_upper.starts_with("DELETE");

                if !is_allowed {
                    return Err(ZError::Other(anyhow::anyhow!(
                        "Only INSERT, UPDATE, and DELETE queries are allowed"
                    )));
                }

                tracing::warn!(
                    invocation_id = %ctx.invocation_id(),
                    sql = %sql,
                    "Executing SQLite write operation"
                );

                let result = sqlx::query(sql)
                    .execute(&pool)
                    .await
                    .map_err(|e| ZError::Other(anyhow::anyhow!("Execute failed: {}", e)))?;

                Ok(ToolResponse {
                    result: serde_json::json!({
                        "rows_affected": result.rows_affected(),
                    }),
                })
            }
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_in_memory() {
        let tools = create_sqlite_tools("sqlite::memory:")
            .await
            .expect("Failed to create SQLite tools");

        assert!(!tools.is_empty());
        assert_eq!(tools[0].name(), "sqlite_list_tables");
    }
}

