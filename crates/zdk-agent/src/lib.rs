//! Agent implementations for RAK

pub mod builder;
pub mod llm_agent;
pub mod workflow;

pub use builder::LLMAgentBuilder;
pub use llm_agent::LLMAgent;
pub use workflow::{
    LoopAgent, LoopAgentBuilder, ParallelAgent, ParallelAgentBuilder, SequentialAgent,
    SequentialAgentBuilder,
};

#[cfg(test)]
mod tests {
    use super::*;
    use zdk_core::{Agent, Content, LLMRequest, LLMResponse, Part, Result, LLM};
    use async_stream::stream;
    use async_trait::async_trait;
    use futures::stream::Stream;
    use std::sync::Arc;

    struct MockLLM;

    #[async_trait]
    impl LLM for MockLLM {
        fn name(&self) -> &str {
            "mock"
        }

        async fn generate_content(
            &self,
            _request: LLMRequest,
            _stream: bool,
        ) -> Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin> {
            Box::new(Box::pin(stream! {
                yield Ok(LLMResponse {
                    content: Some(Content {
                        role: "model".to_string(),
                        parts: vec![Part::Text { text: "Test response".to_string() }],
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

    #[test]
    fn test_builder_creates_agent() {
        let model = Arc::new(MockLLM);

        let agent = LLMAgent::builder()
            .name("test-agent")
            .description("A test agent")
            .model(model)
            .build()
            .unwrap();

        assert_eq!(agent.name(), "test-agent");
        assert_eq!(agent.description(), "A test agent");
    }

    #[test]
    fn test_builder_requires_name() {
        let model = Arc::new(MockLLM);

        let result = LLMAgent::builder().model(model).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_requires_model() {
        let result = LLMAgent::builder().name("test-agent").build();

        assert!(result.is_err());
    }
}
