use zdk_agent::LLMAgent;
use zdk_core::{Content, LLM, LLMRequest, LLMResponse, Part, Result};
use zdk_runner::Runner;
use zdk_session::inmemory::InMemorySessionService;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use async_stream::stream;

// Mock LLM for testing
struct MockLLM {
    name: String,
    response_text: String,
}

impl MockLLM {
    fn new(response: impl Into<String>) -> Self {
        Self {
            name: "mock-model".to_string(),
            response_text: response.into(),
        }
    }
}

#[async_trait]
impl LLM for MockLLM {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn generate_content(
        &self,
        _request: LLMRequest,
        stream_mode: bool,
    ) -> Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin> {
        let response_text = self.response_text.clone();
        
        Box::new(Box::pin(stream! {
            if stream_mode {
                // Simulate streaming response
                let words: Vec<&str> = response_text.split_whitespace().collect();
                for word in words {
                    let content = Content {
                        role: "model".to_string(),
                        parts: vec![Part::Text { text: format!("{} ", word) }],
                    };
                    
                    yield Ok(LLMResponse {
                        content: Some(content),
                        partial: true,
                        turn_complete: false,
                        interrupted: false,
                        finish_reason: None,
                        error_code: None,
                        error_message: None,
                    });
                }
            }
            
            // Final response
            let content = Content {
                role: "model".to_string(),
                parts: vec![Part::Text { text: response_text }],
            };
            
            yield Ok(LLMResponse {
                content: Some(content),
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
async fn test_basic_agent_flow() -> anyhow::Result<()> {
    // Create mock LLM
    let model = Arc::new(MockLLM::new("Hello! I am an AI assistant."));
    
    // Create agent
    let agent = LLMAgent::builder()
        .name("test-agent")
        .description("A test agent")
        .model(model)
        .build()?;
    
    // Create session service
    let session_service = Arc::new(InMemorySessionService::new());
    
    // Create runner
    let runner = Runner::builder()
        .app_name("test-app")
        .agent(Arc::new(agent))
        .session_service(session_service.clone())
        .build()?;
    
    // Create session
    let session = session_service.create(&zdk_session::CreateRequest {
        app_name: "test-app".into(),
        user_id: "test-user".into(),
        session_id: Some("test-session".into()),
    }).await?;
    
    // Run agent
    let message = Content::new_user_text("Hello!");
    let mut stream = runner.run(
        "test-user".into(),
        session.id().into(),
        message,
        Default::default(),
    ).await?;
    
    // Collect events
    let mut events = Vec::new();
    while let Some(event_result) = stream.next().await {
        let event = event_result?;
        events.push(event);
    }
    
    // Verify events
    assert!(!events.is_empty(), "Should have at least one event");
    
    // Check that we have a final response
    let has_final = events.iter().any(|e| e.is_final_response());
    assert!(has_final, "Should have a final response");
    
    // Check event structure
    for event in &events {
        assert_eq!(event.author, "test-agent");
        assert!(!event.invocation_id.is_empty());
        assert!(!event.id.is_empty());
    }
    
    // Verify session history
    let updated_session = session_service.get(&zdk_session::GetRequest {
        app_name: "test-app".into(),
        user_id: "test-user".into(),
        session_id: session.id().into(),
    }).await?;
    
    let session_events = updated_session.events();
    assert!(session_events.len() >= 2, "Should have user message and agent response");
    
    Ok(())
}

#[tokio::test]
async fn test_session_persistence() -> anyhow::Result<()> {
    let session_service = Arc::new(InMemorySessionService::new());
    
    // Create session
    let session = session_service.create(&zdk_session::CreateRequest {
        app_name: "test-app".into(),
        user_id: "user1".into(),
        session_id: Some("session1".into()),
    }).await?;
    
    assert_eq!(session.id(), "session1");
    assert_eq!(session.app_name(), "test-app");
    assert_eq!(session.user_id(), "user1");
    
    // Retrieve session
    let retrieved = session_service.get(&zdk_session::GetRequest {
        app_name: "test-app".into(),
        user_id: "user1".into(),
        session_id: "session1".into(),
    }).await?;
    
    assert_eq!(retrieved.id(), "session1");
    
    Ok(())
}

