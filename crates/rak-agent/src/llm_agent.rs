use crate::builder::LLMAgentBuilder;
use rak_core::{
    Agent, Content, Event, FunctionCall, InvocationContext, LLMRequest, Part, Result, Tool, Toolset, LLM,
};
use rak_telemetry::{trace_llm_call, LLMSpanAttributes};
use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;

pub struct LLMAgent {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) model: Arc<dyn LLM>,
    pub(crate) system_instruction: Option<String>,
    pub(crate) sub_agents: Vec<Arc<dyn Agent>>,
    pub(crate) tools: HashMap<String, Arc<dyn Tool>>,
    pub(crate) toolsets: Vec<Arc<dyn Toolset>>,
}

impl LLMAgent {
    pub fn builder() -> LLMAgentBuilder {
        LLMAgentBuilder::new()
    }

    pub fn new(
        name: String,
        description: String,
        model: Arc<dyn LLM>,
        system_instruction: Option<String>,
    ) -> Self {
        Self {
            name,
            description,
            model,
            system_instruction,
            sub_agents: Vec::new(),
            tools: HashMap::new(),
            toolsets: Vec::new(),
        }
    }
}

#[async_trait]
impl Agent for LLMAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn run(
        &self,
        ctx: Arc<dyn InvocationContext>,
    ) -> Box<dyn Stream<Item = Result<Event>> + Send + Unpin> {
        let model = self.model.clone();
        let agent_name = self.name.clone();
        let invocation_id = ctx.invocation_id().to_string();
        let mut tools = self.tools.clone();
        let toolsets = self.toolsets.clone();
        let ctx_clone = ctx.clone();

        Box::new(Box::pin(stream! {
            // Load tools from toolsets
            for toolset in &toolsets {
                match toolset.get_tools(&*ctx_clone).await {
                    Ok(toolset_tools) => {
                        tracing::info!(
                            invocation_id = %invocation_id,
                            toolset = %toolset.name(),
                            count = toolset_tools.len(),
                            "Loaded tools from toolset"
                        );
                        for tool in toolset_tools {
                            tools.insert(tool.name().to_string(), tool);
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            invocation_id = %invocation_id,
                            toolset = %toolset.name(),
                            error = %e,
                            "Failed to load toolset"
                        );
                    }
                }
            }

            // Build LLM request from context
            let mut conversation = Vec::new();

            // Add user content if available
            if let Some(user_content) = ctx.user_content() {
                conversation.push(user_content.clone());
            }

            let session_id = ctx.session_id().to_string();

            tracing::info!(
                invocation_id = %invocation_id,
                session_id = %session_id,
                agent = %agent_name,
                "Starting LLM agent execution"
            );

            // Tool execution loop
            let max_iterations = 10; // Prevent infinite loops
            for iteration in 0..max_iterations {
                // Convert tools HashMap to Vec for LLMRequest
                let tool_list: Vec<Arc<dyn Tool>> = tools.values().cloned().collect();
                
                let request = LLMRequest {
                    model: model.name().to_string(),
                    contents: conversation.clone(),
                    config: None,
                    tools: tool_list,
                };

                tracing::debug!(
                    invocation_id = %invocation_id,
                    session_id = %session_id,
                    model = %request.model,
                    iteration = iteration,
                    "Calling LLM"
                );

                let mut llm_stream = model.generate_content(request.clone(), true).await;
                let mut accumulated_content: Option<Content> = None;
                let mut function_calls: Vec<FunctionCall> = Vec::new();
                let mut turn_is_complete = false;
                let mut last_event_id: Option<String> = None;

                // Stream LLM responses
                while let Some(llm_result) = llm_stream.next().await {
                    match llm_result {
                        Ok(llm_response) => {
                            let mut event = Event::new(
                                invocation_id.clone(),
                                agent_name.clone(),
                            );

                            last_event_id = Some(event.id.clone());

                            event.content = llm_response.content.clone();
                            event.partial = llm_response.partial;
                            event.turn_complete = llm_response.turn_complete;
                            event.interrupted = llm_response.interrupted;

                            if let Some(code) = llm_response.error_code {
                                event.error_code = code;
                            }
                            if let Some(msg) = llm_response.error_message {
                                event.error_message = msg;
                            }

                            // Accumulate content for tool extraction
                            if let Some(ref content) = llm_response.content {
                                accumulated_content = Some(content.clone());

                                // Extract function calls
                                for part in &content.parts {
                                    if let Part::FunctionCall { function_call } = part {
                                        function_calls.push(function_call.clone());
                                    }
                                }
                            }

                            turn_is_complete = llm_response.turn_complete;

                            yield Ok(event);
                        }
                        Err(e) => {
                            tracing::error!(
                                error = %e,
                                invocation_id = %invocation_id,
                                session_id = %session_id,
                                "LLM call failed"
                            );
                            yield Err(e);
                            return;
                        }
                    }
                }

                // Trace the LLM call after stream completes
                if let Some(event_id) = last_event_id {
                    // Serialize request contents manually (simplified for tracing)
                    let request_json = format!(
                        r#"{{"model":"{}","content_count":{}}}"#,
                        request.model,
                        request.contents.len()
                    );
                    
                    let response_json = if let Some(ref content) = accumulated_content {
                        serde_json::to_string(&content).unwrap_or_else(|_| "{}".to_string())
                    } else {
                        "{}".to_string()
                    };

                    trace_llm_call(LLMSpanAttributes {
                        model: request.model.clone(),
                        invocation_id: invocation_id.clone(),
                        session_id: session_id.clone(),
                        event_id,
                        request_json,
                        response_json,
                        top_p: request.config.as_ref().and_then(|c| c.top_p.map(|p| p as f64)),
                        max_tokens: request.config.as_ref().and_then(|c| c.max_tokens.map(|t| t as i64)),
                    });

                    tracing::debug!(
                        invocation_id = %invocation_id,
                        session_id = %session_id,
                        has_function_calls = !function_calls.is_empty(),
                        turn_complete = turn_is_complete,
                        "LLM call completed"
                    );
                }

                // Add model response to conversation
                if let Some(content) = accumulated_content {
                    conversation.push(content);
                }

                // If no function calls, we're done
                if function_calls.is_empty() || turn_is_complete {
                    tracing::info!(
                        invocation_id = %invocation_id,
                        session_id = %session_id,
                        "Agent execution completed"
                    );
                    break;
                }

                tracing::debug!(
                    invocation_id = %invocation_id,
                    session_id = %session_id,
                    num_function_calls = function_calls.len(),
                    "Executing function calls"
                );

                // Execute function calls
                let mut function_responses = Vec::new();
                for fc in function_calls {
                    if let Some(tool) = tools.get(&fc.name) {
                        // Generate ID if not provided by API
                        let call_id = fc.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                        
                        tracing::debug!(
                            invocation_id = %invocation_id,
                            session_id = %session_id,
                            tool_name = %fc.name,
                            tool_id = %call_id,
                            "Executing tool"
                        );

                        // Create tool context
                        let tool_ctx = Arc::new(rak_tool::DefaultToolContext::new(
                            call_id.clone(),
                            invocation_id.clone(),
                        ));

                        // Execute tool
                        match tool.execute(tool_ctx, fc.args.clone()).await {
                            Ok(response) => {
                                function_responses.push(Part::FunctionResponse {
                                    function_response: rak_core::FunctionResponse {
                                        name: fc.name.clone(),
                                        response: response.result,
                                        id: Some(call_id.clone()),
                                    },
                                });

                                // Emit tool execution event
                                let mut tool_event = Event::new(
                                    invocation_id.clone(),
                                    agent_name.clone(),
                                );
                                tool_event.content = Some(Content {
                                    role: "function".to_string(),
                                    parts: vec![function_responses.last().unwrap().clone()],
                                });
                                yield Ok(tool_event);
                            }
                            Err(e) => {
                                // Emit error event
                                let mut error_event = Event::new(
                                    invocation_id.clone(),
                                    agent_name.clone(),
                                );
                                error_event.error_code = "TOOL_ERROR".to_string();
                                error_event.error_message = format!("Tool {} failed: {}", fc.name, e);
                                yield Ok(error_event);
                            }
                        }
                    } else {
                        // Tool not found
                        let mut error_event = Event::new(
                            invocation_id.clone(),
                            agent_name.clone(),
                        );
                        error_event.error_code = "TOOL_NOT_FOUND".to_string();
                        error_event.error_message = format!("Tool {} not found", fc.name);
                        yield Ok(error_event);
                    }
                }

                // Add function responses to conversation
                if !function_responses.is_empty() {
                    conversation.push(Content {
                        role: "function".to_string(),
                        parts: function_responses,
                    });
                }

                // Continue to next iteration for LLM to process tool results
            }
        }))
    }

    fn sub_agents(&self) -> &[Arc<dyn Agent>] {
        &self.sub_agents
    }
}
