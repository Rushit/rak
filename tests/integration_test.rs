// Integration tests for ZDK
// These tests verify the full stack E2E functionality

use async_stream::stream;
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use tower::ServiceExt;
use zdk_agent::LLMAgent;
use zdk_core::{Content, Event, LLM, LLMRequest, LLMResponse, Part, Result};
use zdk_runner::{RunConfig, Runner};
use zdk_session::{SessionService, inmemory::InMemorySessionService};

// Mock LLM for deterministic testing
struct TestLLM {
    responses: Vec<String>,
}

impl TestLLM {
    fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: responses.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[async_trait]
impl LLM for TestLLM {
    fn name(&self) -> &str {
        "test-llm"
    }

    async fn generate_content(
        &self,
        _request: LLMRequest,
        stream_mode: bool,
    ) -> Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin> {
        let responses = self.responses.clone();

        Box::new(Box::pin(stream! {
            if stream_mode {
                // Simulate streaming by sending words one at a time
                for response in &responses {
                    for word in response.split_whitespace() {
                        yield Ok(LLMResponse {
                            content: Some(Content {
                                role: "model".to_string(),
                                parts: vec![Part::Text { text: format!("{} ", word) }],
                            }),
                            partial: true,
                            turn_complete: false,
                            interrupted: false,
                            finish_reason: None,
                            error_code: None,
                            error_message: None,
                        });
                    }
                }
            }

            // Final complete response
            let full_response = responses.join(" ");
            yield Ok(LLMResponse {
                content: Some(Content {
                    role: "model".to_string(),
                    parts: vec![Part::Text { text: full_response }],
                }),
                partial: false,
                turn_complete: true,
                interrupted: false,
                finish_reason: Some("STOP".to_string()),
                error_code: None,
                error_message: None,
            });
        }))
    }
}

#[tokio::test]
async fn test_e2e_session_and_agent_execution() {
    // Setup
    let llm = Arc::new(TestLLM::new(vec!["Hello!", "I am a test agent."]));

    let agent = LLMAgent::builder()
        .name("test-assistant")
        .description("A test assistant")
        .model(llm)
        .build()
        .unwrap();

    let session_service = Arc::new(InMemorySessionService::new());

    let runner = Runner::builder()
        .app_name("test-app")
        .agent(Arc::new(agent))
        .session_service(session_service.clone())
        .build()
        .unwrap();

    // Create session
    let session = session_service
        .create(&zdk_session::CreateRequest {
            app_name: "test-app".to_string(),
            user_id: "test-user".to_string(),
            session_id: Some("test-session".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(session.id(), "test-session");

    // Run agent
    let message = Content::new_user_text("Hello, agent!");
    let mut event_stream = runner
        .run(
            "test-user".to_string(),
            session.id().to_string(),
            message,
            RunConfig { streaming: false },
        )
        .await
        .unwrap();

    // Collect events
    let mut events = Vec::new();
    while let Some(result) = event_stream.next().await {
        let event = result.unwrap();
        events.push(event);
    }

    // Verify events
    assert!(!events.is_empty(), "Should have at least one event");

    let final_event = events.iter().find(|e| e.is_final_response());
    assert!(final_event.is_some(), "Should have a final response");

    // Verify event structure
    for event in &events {
        assert_eq!(event.author, "test-assistant");
        assert!(!event.invocation_id.is_empty());
        assert!(!event.id.is_empty());
        assert!(event.time > 0);
    }

    // Verify session persistence
    let updated_session = session_service
        .get(&zdk_session::GetRequest {
            app_name: "test-app".to_string(),
            user_id: "test-user".to_string(),
            session_id: "test-session".to_string(),
        })
        .await
        .unwrap();

    let persisted_events = updated_session.events();
    assert!(
        persisted_events.len() >= 2,
        "Should have user message and agent response"
    );
}

#[tokio::test]
async fn test_e2e_streaming_events() {
    let llm = Arc::new(TestLLM::new(vec!["Streaming", "response", "test"]));

    let agent = LLMAgent::builder()
        .name("streaming-agent")
        .description("A streaming agent")
        .model(llm)
        .build()
        .unwrap();

    let session_service = Arc::new(InMemorySessionService::new());

    let runner = Runner::builder()
        .app_name("streaming-app")
        .agent(Arc::new(agent))
        .session_service(session_service)
        .build()
        .unwrap();

    let message = Content::new_user_text("Test streaming");
    let mut stream = runner
        .run(
            "user1".to_string(),
            "session1".to_string(),
            message,
            RunConfig { streaming: true },
        )
        .await
        .unwrap();

    let mut partial_count = 0;
    let mut final_count = 0;

    while let Some(result) = stream.next().await {
        let event = result.unwrap();
        if event.partial {
            partial_count += 1;
        }
        if event.is_final_response() {
            final_count += 1;
        }
    }

    assert!(
        partial_count > 0,
        "Should have partial events in streaming mode"
    );
    assert_eq!(final_count, 1, "Should have exactly one final event");
}

#[tokio::test]
async fn test_e2e_multiple_turns_in_session() {
    let llm = Arc::new(TestLLM::new(vec!["Response 1"]));

    let agent = LLMAgent::builder()
        .name("multi-turn-agent")
        .description("Multi-turn agent")
        .model(llm)
        .build()
        .unwrap();

    let session_service = Arc::new(InMemorySessionService::new());

    let runner = Runner::builder()
        .app_name("multi-turn-app")
        .agent(Arc::new(agent))
        .session_service(session_service.clone())
        .build()
        .unwrap();

    let session_id = "multi-turn-session";

    // Turn 1
    let mut stream = runner
        .run(
            "user1".to_string(),
            session_id.to_string(),
            Content::new_user_text("First message"),
            RunConfig::default(),
        )
        .await
        .unwrap();

    while let Some(_) = stream.next().await {}

    // Turn 2
    let mut stream = runner
        .run(
            "user1".to_string(),
            session_id.to_string(),
            Content::new_user_text("Second message"),
            RunConfig::default(),
        )
        .await
        .unwrap();

    while let Some(_) = stream.next().await {}

    // Verify session has all messages
    let session = session_service
        .get(&zdk_session::GetRequest {
            app_name: "multi-turn-app".to_string(),
            user_id: "user1".to_string(),
            session_id: session_id.to_string(),
        })
        .await
        .unwrap();

    let events = session.events();
    assert!(
        events.len() >= 4,
        "Should have 2 user messages and 2 agent responses"
    );
}

#[tokio::test]
async fn test_event_format_matches_spec() {
    let event = Event::new("test-inv".to_string(), "test-agent".to_string());

    // Serialize to JSON
    let json = serde_json::to_value(&event).unwrap();

    // Verify required fields exist with correct casing
    assert!(json.get("id").is_some());
    assert!(json.get("time").is_some());
    assert!(json.get("invocationId").is_some());
    assert!(json.get("author").is_some());
    assert!(json.get("partial").is_some());
    assert!(json.get("turnComplete").is_some());
    assert!(json.get("actions").is_some());

    // Verify camelCase format
    assert!(json.get("invocationId").is_some(), "Should use camelCase");
    assert!(
        json.get("invocation_id").is_none(),
        "Should not use snake_case"
    );
}

#[tokio::test]
async fn test_content_serialization() {
    let content = Content {
        role: "user".to_string(),
        parts: vec![Part::Text {
            text: "Hello".to_string(),
        }],
    };

    let json = serde_json::to_string(&content).unwrap();
    let parsed: Content = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.role, "user");
    assert_eq!(parsed.parts.len(), 1);
}
