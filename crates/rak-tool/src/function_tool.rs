use rak_core::{Result, Tool, ToolContext, ToolResponse};
use async_trait::async_trait;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for tool execution function
pub type ToolFn = Box<
    dyn Fn(
            Arc<dyn ToolContext>,
            Value,
        ) -> Pin<Box<dyn Future<Output = Result<ToolResponse>> + Send>>
        + Send
        + Sync,
>;

/// A function-based tool implementation
pub struct FunctionTool {
    name: String,
    description: String,
    schema: Value,
    is_long_running: bool,
    execute_fn: ToolFn,
}

impl FunctionTool {
    pub fn builder() -> FunctionToolBuilder {
        FunctionToolBuilder::new()
    }
}

impl std::fmt::Debug for FunctionTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("schema", &self.schema)
            .field("is_long_running", &self.is_long_running)
            .finish()
    }
}

#[async_trait]
impl Tool for FunctionTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn schema(&self) -> Value {
        self.schema.clone()
    }

    fn is_long_running(&self) -> bool {
        self.is_long_running
    }

    async fn execute(&self, ctx: Arc<dyn ToolContext>, params: Value) -> Result<ToolResponse> {
        (self.execute_fn)(ctx, params).await
    }
}

/// Builder for FunctionTool
pub struct FunctionToolBuilder {
    name: Option<String>,
    description: Option<String>,
    schema: Option<Value>,
    is_long_running: bool,
    execute_fn: Option<ToolFn>,
}

impl FunctionToolBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            schema: None,
            is_long_running: false,
            execute_fn: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn schema(mut self, schema: Value) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn long_running(mut self, is_long_running: bool) -> Self {
        self.is_long_running = is_long_running;
        self
    }

    pub fn execute<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(Arc<dyn ToolContext>, Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ToolResponse>> + Send + 'static,
    {
        self.execute_fn = Some(Box::new(move |ctx, params| Box::pin(f(ctx, params))));
        self
    }

    pub fn build(self) -> Result<FunctionTool> {
        Ok(FunctionTool {
            name: self
                .name
                .ok_or_else(|| rak_core::Error::Other(anyhow::anyhow!("Tool name is required")))?,
            description: self.description.ok_or_else(|| {
                rak_core::Error::Other(anyhow::anyhow!("Tool description is required"))
            })?,
            schema: self.schema.unwrap_or(Value::Null),
            is_long_running: self.is_long_running,
            execute_fn: self.execute_fn.ok_or_else(|| {
                rak_core::Error::Other(anyhow::anyhow!("Tool execute function is required"))
            })?,
        })
    }
}

impl Default for FunctionToolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::DefaultToolContext;
    use crate::schema::ToolSchema;

    #[tokio::test]
    async fn test_function_tool_creation() {
        let schema = ToolSchema::new()
            .property("x", "number", "First number")
            .property("y", "number", "Second number")
            .required("x")
            .required("y")
            .build();

        let tool = FunctionTool::builder()
            .name("add")
            .description("Adds two numbers")
            .schema(schema)
            .execute(|_ctx, params| async move {
                let x = params["x"].as_f64().unwrap_or(0.0);
                let y = params["y"].as_f64().unwrap_or(0.0);
                let result = x + y;

                Ok(ToolResponse {
                    result: serde_json::json!({"sum": result}),
                })
            })
            .build()
            .unwrap();

        assert_eq!(tool.name(), "add");
        assert_eq!(tool.description(), "Adds two numbers");
        assert!(!tool.is_long_running());

        // Test execution
        let ctx = Arc::new(DefaultToolContext::new(
            "call-1".to_string(),
            "inv-1".to_string(),
        ));
        let params = serde_json::json!({"x": 5.0, "y": 3.0});
        let response = tool.execute(ctx, params).await.unwrap();

        assert_eq!(response.result["sum"], 8.0);
    }
}
