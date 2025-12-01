//! Span creation helpers for LLM calls and tool executions

use crate::attributes::*;

/// Attributes for tracing an LLM call
#[derive(Debug, Clone)]
pub struct LLMSpanAttributes {
    pub model: String,
    pub invocation_id: String,
    pub session_id: String,
    pub event_id: String,
    pub request_json: String,
    pub response_json: String,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
}

/// Attributes for tracing a tool call
#[derive(Debug, Clone)]
pub struct ToolSpanAttributes {
    pub tool_name: String,
    pub tool_description: String,
    pub tool_call_id: String,
    pub invocation_id: String,
    pub session_id: String,
    pub event_id: String,
    pub args_json: String,
    pub response_json: String,
}

/// Create and record an OpenTelemetry span for an LLM generation call.
///
/// Records comprehensive telemetry including the model name, request/response payloads,
/// invocation context, and optional parameters like top_p and max_tokens. The span
/// follows OpenTelemetry semantic conventions for generative AI operations.
pub fn trace_llm_call(attrs: LLMSpanAttributes) {
    let span = tracing::info_span!(
        "call_llm",
        { GEN_AI_SYSTEM } = SYSTEM_NAME,
        { GEN_AI_REQUEST_MODEL } = %attrs.model,
        { GCP_VERTEX_AGENT_INVOCATION_ID } = %attrs.invocation_id,
        { GCP_VERTEX_AGENT_SESSION_ID } = %attrs.session_id,
        { GCP_VERTEX_AGENT_EVENT_ID } = %attrs.event_id,
        { GCP_VERTEX_AGENT_LLM_REQUEST } = %attrs.request_json,
        { GCP_VERTEX_AGENT_LLM_RESPONSE } = %attrs.response_json,
    );

    // Add optional attributes if present
    if let Some(top_p) = attrs.top_p {
        span.record(GEN_AI_REQUEST_TOP_P, top_p);
    }
    if let Some(max_tokens) = attrs.max_tokens {
        span.record(GEN_AI_REQUEST_MAX_TOKENS, max_tokens);
    }

    // Enter and immediately exit the span (it's recorded)
    let _guard = span.enter();
}

/// Create and record an OpenTelemetry span for a tool execution.
///
/// Records tool invocation details including tool name, description, call ID,
/// arguments, and response. This enables distributed tracing of tool calls
/// throughout the agent execution flow.
pub fn trace_tool_call(attrs: ToolSpanAttributes) {
    let span = tracing::info_span!(
        "execute_tool",
        { GEN_AI_OPERATION_NAME } = "execute_tool",
        { GEN_AI_TOOL_NAME } = %attrs.tool_name,
        { GEN_AI_TOOL_DESCRIPTION } = %attrs.tool_description,
        { GEN_AI_TOOL_CALL_ID } = %attrs.tool_call_id,
        { GCP_VERTEX_AGENT_INVOCATION_ID } = %attrs.invocation_id,
        { GCP_VERTEX_AGENT_SESSION_ID } = %attrs.session_id,
        { GCP_VERTEX_AGENT_EVENT_ID } = %attrs.event_id,
        { GCP_VERTEX_AGENT_TOOL_CALL_ARGS } = %attrs.args_json,
        { GCP_VERTEX_AGENT_TOOL_RESPONSE } = %attrs.response_json,
        // Set empty LLM request/response for compatibility with UI
        { GCP_VERTEX_AGENT_LLM_REQUEST } = "{}",
        { GCP_VERTEX_AGENT_LLM_RESPONSE } = "{}",
    );

    // Enter and immediately exit the span (it's recorded)
    let _guard = span.enter();
}

/// Helper to safely serialize to JSON string
pub fn safe_serialize<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<not serializable>".to_string())
}

/// Helper to safely serialize to pretty JSON string
pub fn safe_serialize_pretty<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "<not serializable>".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_serialize() {
        let value = serde_json::json!({"test": "value"});
        let result = safe_serialize(&value);
        assert!(result.contains("test"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_llm_span_attributes() {
        let attrs = LLMSpanAttributes {
            model: "gemini-2.0".to_string(),
            invocation_id: "inv-123".to_string(),
            session_id: "sess-456".to_string(),
            event_id: "event-789".to_string(),
            request_json: "{}".to_string(),
            response_json: "{}".to_string(),
            top_p: Some(0.95),
            max_tokens: Some(1024),
        };

        // Just verify we can create and use the attributes
        trace_llm_call(attrs);
    }

    #[test]
    fn test_tool_span_attributes() {
        let attrs = ToolSpanAttributes {
            tool_name: "calculator".to_string(),
            tool_description: "Calculate math".to_string(),
            tool_call_id: "call-123".to_string(),
            invocation_id: "inv-123".to_string(),
            session_id: "sess-456".to_string(),
            event_id: "event-789".to_string(),
            args_json: r#"{"expr": "2+2"}"#.to_string(),
            response_json: r#"{"result": 4}"#.to_string(),
        };

        // Just verify we can create and use the attributes
        trace_tool_call(attrs);
    }
}

