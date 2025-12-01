//! Agent implementations for ZDK

pub mod builder;
pub mod builder_common;
pub mod llm_agent;
#[cfg(test)]
pub mod testing;
pub mod utils;
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
    use crate::testing::MockLLM;
    use std::sync::Arc;
    use zdk_core::Agent;

    #[test]
    fn test_builder_creates_agent() {
        let model = Arc::new(MockLLM::new());

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
        let model = Arc::new(MockLLM::new());

        let result = LLMAgent::builder().model(model).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_requires_model() {
        let result = LLMAgent::builder().name("test-agent").build();

        assert!(result.is_err());
    }
}
