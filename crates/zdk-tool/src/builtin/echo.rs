use crate::{FunctionTool, ToolSchema};
use zdk_core::{Error, Result, ToolResponse};

/// Creates an echo tool for testing purposes
pub fn create_echo_tool() -> Result<FunctionTool> {
    let schema = ToolSchema::new()
        .property("message", "string", "Message to echo back")
        .required("message")
        .build();

    FunctionTool::builder()
        .name("echo")
        .description("Echoes back the provided message. Useful for testing tool execution.")
        .schema(schema)
        .execute(|ctx, params| async move {
            let message = params["message"]
                .as_str()
                .ok_or_else(|| Error::Other(anyhow::anyhow!("Missing 'message' parameter")))?;

            tracing::debug!(
                invocation_id = %ctx.invocation_id(),
                tool_call_id = %ctx.function_call_id(),
                message = %message,
                "Echo tool called"
            );

            Ok(ToolResponse {
                result: serde_json::json!({
                    "message": message,
                    "invocation_id": ctx.invocation_id(),
                    "function_call_id": ctx.function_call_id()
                }),
            })
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::DefaultToolContext;
    use std::sync::Arc;
    use zdk_core::Tool;

    #[tokio::test]
    async fn test_echo_tool() {
        let tool = create_echo_tool().unwrap();

        assert_eq!(tool.name(), "echo");

        let ctx = Arc::new(DefaultToolContext::new(
            "call-123".to_string(),
            "inv-456".to_string(),
        ));
        let params = serde_json::json!({"message": "Hello, World!"});
        let response = tool.execute(ctx, params).await.unwrap();

        assert_eq!(response.result["message"], "Hello, World!");
        assert_eq!(response.result["invocation_id"], "inv-456");
        assert_eq!(response.result["function_call_id"], "call-123");
    }
}
