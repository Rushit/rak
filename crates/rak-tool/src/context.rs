use rak_core::ToolContext;

/// Default implementation of ToolContext
#[derive(Debug, Clone)]
pub struct DefaultToolContext {
    function_call_id: String,
    invocation_id: String,
}

impl DefaultToolContext {
    pub fn new(function_call_id: String, invocation_id: String) -> Self {
        Self {
            function_call_id,
            invocation_id,
        }
    }
}

impl ToolContext for DefaultToolContext {
    fn function_call_id(&self) -> &str {
        &self.function_call_id
    }

    fn invocation_id(&self) -> &str {
        &self.invocation_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_context_creation() {
        let ctx = DefaultToolContext::new("call-123".to_string(), "inv-456".to_string());

        assert_eq!(ctx.function_call_id(), "call-123");
        assert_eq!(ctx.invocation_id(), "inv-456");
    }
}
