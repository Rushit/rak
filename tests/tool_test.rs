use zdk_agent::LLMAgent;
use zdk_core::{
    Agent, Content, FunctionCall, InvocationContext, LLMRequest, LLMResponse, Part, Tool, LLM,
};
use zdk_tool::builtin::{create_calculator_tool, create_echo_tool};
use zdk_tool::DefaultToolContext;
use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;

// Mock LLM for testing
struct MockLLM {
    call_count: std::sync::Mutex<usize>,
}

impl MockLLM {
    fn new() -> Self {
        Self {
            call_count: std::sync::Mutex::new(0),
        }
    }

    fn get_responses(&self, call_num: usize) -> Vec<LLMResponse> {
        match call_num {
            0 => {
                // First call: model calls calculator
                vec![LLMResponse {
                    content: Some(Content {
                        role: "model".to_string(),
                        parts: vec![Part::FunctionCall {
                            function_call: FunctionCall {
                                name: "calculator".to_string(),
                                args: serde_json::json!({"expression": "2 + 2"}),
                                id: Some("call-1".to_string()),
                            },
                        }],
                    }),
                    partial: false,
                    turn_complete: false,
                    interrupted: false,
                    finish_reason: None,
                    error_code: None,
                    error_message: None,
                }]
            }
            _ => {
                // Subsequent calls: model processes result
                vec![LLMResponse {
                    content: Some(Content {
                        role: "model".to_string(),
                        parts: vec![Part::Text {
                            text: "The result is 4".to_string(),
                        }],
                    }),
                    partial: false,
                    turn_complete: true,
                    interrupted: false,
                    finish_reason: Some("STOP".to_string()),
                    error_code: None,
                    error_message: None,
                }]
            }
        }
    }
}

#[async_trait]
impl LLM for MockLLM {
    fn name(&self) -> &str {
        "mock-llm"
    }

    async fn generate_content(
        &self,
        _request: LLMRequest,
        _stream: bool,
    ) -> Box<dyn Stream<Item = zdk_core::Result<LLMResponse>> + Send + Unpin> {
        let mut count = self.call_count.lock().unwrap();
        let call_num = *count;
        *count += 1;
        drop(count);

        let responses = self.get_responses(call_num);
        Box::new(Box::pin(stream! {
            for response in responses {
                yield Ok(response);
            }
        }))
    }
}

// Mock context
struct MockContext {
    invocation_id: String,
    user_content: Option<Content>,
}

impl MockContext {
    fn new() -> Self {
        Self {
            invocation_id: "test-inv-1".to_string(),
            user_content: Some(Content::new_user_text("Calculate 2 + 2")),
        }
    }
}

#[async_trait]
impl InvocationContext for MockContext {
    fn invocation_id(&self) -> &str {
        &self.invocation_id
    }

    fn user_content(&self) -> Option<&Content> {
        self.user_content.as_ref()
    }
}

impl zdk_core::ReadonlyContext for MockContext {
    fn app_name(&self) -> &str {
        "test-app"
    }

    fn user_id(&self) -> &str {
        "test-user"
    }

    fn session_id(&self) -> &str {
        "test-session"
    }
}

#[tokio::test]
async fn test_tool_execution() {
    // Create mock LLM
    let model = Arc::new(MockLLM::new());

    // Create tools
    let calculator = Arc::new(create_calculator_tool().unwrap());

    // Create agent with tool
    let agent = LLMAgent::builder()
        .name("test-agent")
        .description("Test agent with calculator")
        .model(model)
        .tool(calculator)
        .build()
        .unwrap();

    // Run agent
    let ctx = Arc::new(MockContext::new());
    let mut stream = agent.run(ctx).await;

    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        let event = result.unwrap();
        events.push(event);
    }

    // Verify we got events
    assert!(!events.is_empty(), "Should have received events");

    // Find function call event
    let has_function_call = events.iter().any(|e| {
        e.content.as_ref().map_or(false, |c| {
            c.parts
                .iter()
                .any(|p| matches!(p, Part::FunctionCall { .. }))
        })
    });
    assert!(has_function_call, "Should have function call event");

    // Find function response event
    let has_function_response = events.iter().any(|e| {
        e.content.as_ref().map_or(false, |c| {
            c.parts
                .iter()
                .any(|p| matches!(p, Part::FunctionResponse { .. }))
        })
    });
    assert!(has_function_response, "Should have function response event");

    // Find final text response
    let has_text_response = events.iter().any(|e| {
        e.content.as_ref().map_or(false, |c| {
            c.parts
                .iter()
                .any(|p| matches!(p, Part::Text { text } if text.contains("result")))
        })
    });
    assert!(has_text_response, "Should have text response with result");
}

#[tokio::test]
async fn test_calculator_tool() {
    let tool = create_calculator_tool().unwrap();

    assert_eq!(tool.name(), "calculator");

    let ctx = Arc::new(DefaultToolContext::new(
        "call-1".to_string(),
        "inv-1".to_string(),
    ));
    let params = serde_json::json!({"expression": "10 + 5 * 2"});
    let response = tool.execute(ctx, params).await.unwrap();

    assert_eq!(response.result["result"], 20.0);
}

#[tokio::test]
async fn test_echo_tool() {
    let tool = create_echo_tool().unwrap();

    assert_eq!(tool.name(), "echo");

    let ctx = Arc::new(DefaultToolContext::new(
        "call-2".to_string(),
        "inv-2".to_string(),
    ));
    let params = serde_json::json!({"message": "Hello, tools!"});
    let response = tool.execute(ctx, params).await.unwrap();

    assert_eq!(response.result["message"], "Hello, tools!");
    assert_eq!(response.result["invocation_id"], "inv-2");
}
