//! PostgreSQL database tools

use crate::config::DatabaseToolConfig;
use crate::types::{ColumnInfo, TableInfo, TableSchema};
use sqlx::{postgres::PgPoolOptions, Column, Pool, Postgres, Row};
use std::sync::Arc;
use std::time::Duration;
use zdk_core::{Error as ZError, Result as ZResult, Tool, ToolResponse};
use zdk_tool::{FunctionTool, ToolSchema};

/// Create PostgreSQL tools with default configuration (read-only)
pub async fn create_postgres_tools(connection_string: &str) -> ZResult<Vec<Arc<dyn Tool>>> {
    create_postgres_tools_with_config(connection_string, DatabaseToolConfig::default()).await
}

/// Create PostgreSQL tools with custom configuration
pub async fn create_postgres_tools_with_config(
    connection_string: &str,
    config: DatabaseToolConfig,
) -> ZResult<Vec<Arc<dyn Tool>>> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(config.timeout_secs))
        .connect(connection_string)
        .await
        .map_err(|e| ZError::Other(anyhow::anyhow!("Failed to connect to PostgreSQL: {}", e)))?;

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
fn create_list_tables_tool(pool: Pool<Postgres>) -> ZResult<FunctionTool> {
    let schema = ToolSchema::new()
        .property(
            "schema",
            "string",
            "Database schema to list tables from (default: 'public')",
        )
        .build();

    FunctionTool::builder()
        .name("postgres_list_tables")
        .description("List all tables in the PostgreSQL database")
        .schema(schema)
        .execute(move |ctx, params| {
            let pool = pool.clone();
            async move {
                let schema_name = params["schema"].as_str().unwrap_or("public");

                tracing::debug!(
                    invocation_id = %ctx.invocation_id(),
                    schema = %schema_name,
                    "Listing PostgreSQL tables"
                );

                let query = r#"
                    SELECT
                        table_name,
                        (SELECT COUNT(*) FROM information_schema.tables t2
                         WHERE t2.table_schema = t.table_schema
                         AND t2.table_name = t.table_name) as row_count
                    FROM information_schema.tables t
                    WHERE table_schema = $1
                    AND table_type = 'BASE TABLE'
                    ORDER BY table_name
                "#;

                let rows = sqlx::query(query)
                    .bind(schema_name)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| ZError::Other(anyhow::anyhow!("Failed to list tables: {}", e)))?;

                let tables: Vec<TableInfo> = rows
                    .iter()
                    .map(|row| TableInfo {
                        name: row.get("table_name"),
                        row_count: 0, // Actual count would require separate queries
                        size_bytes: None,
                    })
                    .collect();

                Ok(ToolResponse {
                    result: serde_json::to_value(&tables).map_err(|e| {
                        ZError::Other(anyhow::anyhow!("Serialization error: {}", e))
                    })?,
                })
            }
        })
        .build()
}

/// Create a tool to describe a table's schema
fn create_describe_table_tool(pool: Pool<Postgres>) -> ZResult<FunctionTool> {
    let schema = ToolSchema::new()
        .property("table_name", "string", "Name of the table to describe")
        .property("schema", "string", "Database schema (default: 'public')")
        .required("table_name")
        .build();

    FunctionTool::builder()
        .name("postgres_describe_table")
        .description("Get the schema information for a PostgreSQL table")
        .schema(schema)
        .execute(move |ctx, params| {
            let pool = pool.clone();
            async move {
                let table_name = params["table_name"].as_str().ok_or_else(|| {
                    ZError::Other(anyhow::anyhow!("Missing table_name parameter"))
                })?;
                let schema_name = params["schema"].as_str().unwrap_or("public");

                tracing::debug!(
                    invocation_id = %ctx.invocation_id(),
                    table = %table_name,
                    schema = %schema_name,
                    "Describing PostgreSQL table"
                );

                // Get column information
                let column_query = r#"
                    SELECT
                        column_name,
                        data_type,
                        is_nullable,
                        column_default
                    FROM information_schema.columns
                    WHERE table_schema = $1 AND table_name = $2
                    ORDER BY ordinal_position
                "#;

                let rows = sqlx::query(column_query)
                    .bind(schema_name)
                    .bind(table_name)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| {
                        ZError::Other(anyhow::anyhow!("Failed to describe table: {}", e))
                    })?;

                let columns: Vec<ColumnInfo> = rows
                    .iter()
                    .map(|row| ColumnInfo {
                        name: row.get("column_name"),
                        data_type: row.get("data_type"),
                        nullable: row.get::<String, _>("is_nullable") == "YES",
                        default_value: row.get("column_default"),
                    })
                    .collect();

                let table_schema = TableSchema {
                    table_name: table_name.to_string(),
                    columns,
                    indexes: Vec::new(),     // Would need additional queries
                    constraints: Vec::new(), // Would need additional queries
                };

                Ok(ToolResponse {
                    result: serde_json::to_value(&table_schema).map_err(|e| {
                        ZError::Other(anyhow::anyhow!("Serialization error: {}", e))
                    })?,
                })
            }
        })
        .build()
}

/// Create a tool to execute SELECT queries
fn create_query_tool(pool: Pool<Postgres>, config: DatabaseToolConfig) -> ZResult<FunctionTool> {
    let schema = ToolSchema::new()
        .property("sql", "string", "SQL SELECT query to execute")
        .required("sql")
        .build();

    FunctionTool::builder()
        .name("postgres_query")
        .description("Execute a SELECT query on the PostgreSQL database")
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
                    "Executing PostgreSQL query"
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
                                value
                                    .map(serde_json::Value::String)
                                    .unwrap_or(serde_json::Value::Null),
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
fn create_execute_tool(pool: Pool<Postgres>, _config: DatabaseToolConfig) -> ZResult<FunctionTool> {
    let schema = ToolSchema::new()
        .property(
            "sql",
            "string",
            "SQL query to execute (INSERT, UPDATE, DELETE)",
        )
        .required("sql")
        .build();

    FunctionTool::builder()
        .name("postgres_execute")
        .description("Execute an INSERT, UPDATE, or DELETE query on the PostgreSQL database")
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
                    "Executing PostgreSQL write operation"
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

    #[test]
    fn test_config_defaults() {
        let config = DatabaseToolConfig::default();
        assert!(config.read_only);
        assert_eq!(config.max_rows, 1000);
        assert_eq!(config.timeout_secs, 30);
    }
}
