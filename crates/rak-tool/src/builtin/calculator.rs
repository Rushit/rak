use crate::{FunctionTool, ToolSchema};
use rak_core::{Error, Result, ToolResponse};

/// Creates a calculator tool that evaluates mathematical expressions
pub fn create_calculator_tool() -> Result<FunctionTool> {
    let schema = ToolSchema::new()
        .property(
            "expression",
            "string",
            "Mathematical expression to evaluate (e.g., '2 + 2', '10 * 5')",
        )
        .required("expression")
        .build();

    FunctionTool::builder()
        .name("calculator")
        .description(
            "Evaluates mathematical expressions. Supports +, -, *, /, parentheses, and numbers.",
        )
        .schema(schema)
        .execute(|ctx, params| async move {
            let expression = params["expression"]
                .as_str()
                .ok_or_else(|| Error::Other(anyhow::anyhow!("Missing 'expression' parameter")))?;

            tracing::debug!(
                invocation_id = %ctx.invocation_id(),
                tool_call_id = %ctx.function_call_id(),
                expression = %expression,
                "Calculating expression"
            );

            // Simple expression evaluator
            let result = evaluate_expression(expression)?;

            tracing::debug!(
                invocation_id = %ctx.invocation_id(),
                tool_call_id = %ctx.function_call_id(),
                result = %result,
                "Calculation completed"
            );

            Ok(ToolResponse {
                result: serde_json::json!({
                    "result": result,
                    "expression": expression
                }),
            })
        })
        .build()
}

/// Simple expression evaluator
/// Supports: +, -, *, /, parentheses, and numbers
fn evaluate_expression(expr: &str) -> Result<f64> {
    let expr = expr.trim().replace(" ", "");

    // Use meval for expression evaluation
    match meval::eval_str(&expr) {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::Other(anyhow::anyhow!(
            "Failed to evaluate expression: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::DefaultToolContext;
    use rak_core::Tool;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_calculator_tool() {
        let tool = create_calculator_tool().unwrap();

        assert_eq!(tool.name(), "calculator");

        // Test simple addition
        let ctx = Arc::new(DefaultToolContext::new(
            "call-1".to_string(),
            "inv-1".to_string(),
        ));
        let params = serde_json::json!({"expression": "2 + 2"});
        let response = tool.execute(ctx.clone(), params).await.unwrap();
        assert_eq!(response.result["result"], 4.0);

        // Test multiplication
        let params = serde_json::json!({"expression": "10 * 5"});
        let response = tool.execute(ctx.clone(), params).await.unwrap();
        assert_eq!(response.result["result"], 50.0);

        // Test complex expression
        let params = serde_json::json!({"expression": "(10 + 5) * 2"});
        let response = tool.execute(ctx, params).await.unwrap();
        assert_eq!(response.result["result"], 30.0);
    }
}
