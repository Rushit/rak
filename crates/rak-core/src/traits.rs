use super::{Content, Event, InvocationContext, Result, ToolContext};
use async_trait::async_trait;
use futures::stream::Stream;
use std::sync::Arc;

/// Agent trait - the core abstraction for all agents
#[async_trait]
pub trait Agent: Send + Sync {
    /// Returns the unique name of the agent
    fn name(&self) -> &str;

    /// Returns a description of the agent's capabilities
    fn description(&self) -> &str;

    /// Runs the agent with the given invocation context
    async fn run(
        &self,
        ctx: Arc<dyn InvocationContext>,
    ) -> Box<dyn Stream<Item = Result<Event>> + Send + Unpin>;

    /// Returns the sub-agents of this agent
    fn sub_agents(&self) -> &[Arc<dyn Agent>];
}

/// LLM trait - abstraction for language models
#[async_trait]
pub trait LLM: Send + Sync {
    /// Returns the name of the model
    fn name(&self) -> &str;

    /// Generates content based on the request
    async fn generate_content(
        &self,
        request: LLMRequest,
        stream: bool,
    ) -> Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin>;
}

/// Tool trait - abstraction for callable tools
#[async_trait]
pub trait Tool: Send + Sync {
    /// Returns the name of the tool
    fn name(&self) -> &str;

    /// Returns a description of what the tool does
    fn description(&self) -> &str;

    /// Returns the JSON schema for the tool's parameters
    fn schema(&self) -> serde_json::Value;

    /// Indicates whether this is a long-running tool
    fn is_long_running(&self) -> bool {
        false
    }

    /// Executes the tool with given parameters
    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        params: serde_json::Value,
    ) -> Result<ToolResponse>;
}

/// Request to an LLM
#[derive(Debug, Clone)]
pub struct LLMRequest {
    pub model: String,
    pub contents: Vec<Content>,
    pub config: Option<GenerateConfig>,
}

/// Response from an LLM
#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub content: Option<Content>,
    pub partial: bool,
    pub turn_complete: bool,
    pub interrupted: bool,
    pub finish_reason: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

/// Tool execution response
#[derive(Debug, Clone)]
pub struct ToolResponse {
    pub result: serde_json::Value,
}

/// Generation configuration
#[derive(Debug, Clone, Default)]
pub struct GenerateConfig {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
}
