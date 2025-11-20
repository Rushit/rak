//! # RAK Telemetry
//!
//! OpenTelemetry integration for distributed tracing and observability.
//!
//! This crate provides automatic tracing for LLM calls, tool executions, and agent runs
//! using OpenTelemetry standards. It includes structured span attributes compatible with
//! GCP Vertex AI Agent telemetry format for seamless cloud integration.

mod spans;
mod tracer;

pub use spans::{trace_llm_call, trace_tool_call, LLMSpanAttributes, ToolSpanAttributes};
pub use tracer::{init_telemetry, register_span_processor};

/// OpenTelemetry span attribute constants for AI agent observability.
///
/// These constants follow OpenTelemetry semantic conventions for generative AI
/// and GCP Vertex AI Agent attributes for compatibility with cloud services.
pub mod attributes {
    // Generic AI attributes
    pub const GEN_AI_OPERATION_NAME: &str = "gen_ai.operation.name";
    pub const GEN_AI_SYSTEM: &str = "gen_ai.system";
    pub const GEN_AI_REQUEST_MODEL: &str = "gen_ai.request.model";
    pub const GEN_AI_REQUEST_TOP_P: &str = "gen_ai.request.top_p";
    pub const GEN_AI_REQUEST_MAX_TOKENS: &str = "gen_ai.request.max_tokens";

    // Tool-specific attributes
    pub const GEN_AI_TOOL_NAME: &str = "gen_ai.tool.name";
    pub const GEN_AI_TOOL_DESCRIPTION: &str = "gen_ai.tool.description";
    pub const GEN_AI_TOOL_CALL_ID: &str = "gen_ai.tool.call.id";

    // GCP Vertex Agent attributes
    pub const GCP_VERTEX_AGENT_LLM_REQUEST: &str = "gcp.vertex.agent.llm_request";
    pub const GCP_VERTEX_AGENT_LLM_RESPONSE: &str = "gcp.vertex.agent.llm_response";
    pub const GCP_VERTEX_AGENT_TOOL_CALL_ARGS: &str = "gcp.vertex.agent.tool_call_args";
    pub const GCP_VERTEX_AGENT_TOOL_RESPONSE: &str = "gcp.vertex.agent.tool_response";
    pub const GCP_VERTEX_AGENT_EVENT_ID: &str = "gcp.vertex.agent.event_id";
    pub const GCP_VERTEX_AGENT_INVOCATION_ID: &str = "gcp.vertex.agent.invocation_id";
    pub const GCP_VERTEX_AGENT_SESSION_ID: &str = "gcp.vertex.agent.session_id";

    // System name constant
    pub const SYSTEM_NAME: &str = "gcp.vertex.agent";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attributes_constants() {
        // Verify attribute names follow OpenTelemetry semantic conventions
        assert_eq!(attributes::GEN_AI_OPERATION_NAME, "gen_ai.operation.name");
        assert_eq!(attributes::GEN_AI_SYSTEM, "gen_ai.system");
        assert_eq!(attributes::SYSTEM_NAME, "gcp.vertex.agent");
    }
}

