//! Runner for executing agents

pub mod context;
pub mod runner;

pub use context::DefaultInvocationContext;
pub use runner::{RunConfig, Runner, RunnerBuilder};

#[cfg(test)]
mod tests {
    use super::*;
    use zdk_core::{Agent, Content, LLMRequest, LLMResponse, Part, Result, LLM};
    use zdk_session::{inmemory::InMemorySessionService, SessionService};
    use async_stream::stream;
    use async_trait::async_trait;
    use futures::stream::{Stream, StreamExt};
    use std::sync::Arc;

    // Mock LLM for testing
    struct MockLLM {
        response: String,
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
        ) -> Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin> {
            let response = self.response.clone();
            Box::new(Box::pin(stream! {
                yield Ok(LLMResponse {
                    content: Some(Content {
                        role: "model".to_string(),
                        parts: vec![Part::Text { text: response }],
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

    // Mock Agent
    struct MockAgent {
        name: String,
        llm: Arc<dyn LLM>,
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Test agent"
        }

        async fn run(
            &self,
            ctx: Arc<dyn zdk_core::InvocationContext>,
        ) -> Box<dyn Stream<Item = Result<zdk_core::Event>> + Send + Unpin> {
            let name = self.name.clone();
            let invocation_id = ctx.invocation_id().to_string();
            let llm = self.llm.clone();

            Box::new(Box::pin(stream! {
                let mut llm_stream = llm.generate_content(
                    LLMRequest {
                        model: "mock".to_string(),
                        contents: vec![],
                        config: None,
                        tools: vec![],
                    },
                    false,
                ).await;

                while let Some(response) = llm_stream.next().await {
                    let response = response?;
                    let mut event = zdk_core::Event::new(invocation_id.clone(), name.clone());
                    event.content = response.content;
                    event.turn_complete = response.turn_complete;
                    yield Ok(event);
                }
            }))
        }

        fn sub_agents(&self) -> &[Arc<dyn Agent>] {
            &[]
        }
    }

    #[tokio::test]
    async fn test_runner_executes_agent() {
        let llm = Arc::new(MockLLM {
            response: "Hello from mock LLM!".to_string(),
        });

        let agent = Arc::new(MockAgent {
            name: "test-agent".to_string(),
            llm,
        });

        let session_service = Arc::new(InMemorySessionService::new());

        let runner = Runner::builder()
            .app_name("test-app")
            .agent(agent)
            .session_service(session_service)
            .build()
            .unwrap();

        let message = Content::new_user_text("Hello!");
        let mut stream = runner
            .run(
                "user1".to_string(),
                "session1".to_string(),
                message,
                RunConfig::default(),
            )
            .await
            .unwrap();

        let mut events = Vec::new();
        while let Some(result) = stream.next().await {
            events.push(result.unwrap());
        }

        assert!(!events.is_empty());
        assert_eq!(events[0].author, "test-agent");
    }

    #[tokio::test]
    async fn test_runner_persists_events() {
        let llm = Arc::new(MockLLM {
            response: "Response".to_string(),
        });

        let agent = Arc::new(MockAgent {
            name: "test-agent".to_string(),
            llm,
        });

        let session_service = Arc::new(InMemorySessionService::new());

        let runner = Runner::builder()
            .app_name("test-app")
            .agent(agent)
            .session_service(session_service.clone())
            .build()
            .unwrap();

        let message = Content::new_user_text("Test");
        let mut stream = runner
            .run(
                "user1".to_string(),
                "session1".to_string(),
                message,
                RunConfig::default(),
            )
            .await
            .unwrap();

        // Consume stream
        while let Some(_) = stream.next().await {}

        // Verify events persisted
        let session = session_service
            .get(&zdk_session::GetRequest {
                app_name: "test-app".to_string(),
                user_id: "user1".to_string(),
                session_id: "session1".to_string(),
            })
            .await
            .unwrap();

        let events = session.events();
        assert!(events.len() >= 2); // User message + agent response
    }
}
