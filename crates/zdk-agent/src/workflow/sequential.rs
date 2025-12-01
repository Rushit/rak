use super::loop_agent::LoopAgent;
use crate::builder_common::AgentBuilderCore;
use async_trait::async_trait;
use futures::stream::Stream;
use std::sync::Arc;
use zdk_core::{Agent, Error, Event, InvocationContext, Result};

/// SequentialAgent executes its sub-agents once, in the order they are listed.
///
/// Use the SequentialAgent when you want execution to occur in a fixed,
/// strict order. This is internally implemented as a LoopAgent with max_iterations=1.
pub struct SequentialAgent {
    inner: LoopAgent,
}

impl SequentialAgent {
    pub fn builder() -> SequentialAgentBuilder {
        SequentialAgentBuilder::new()
    }
}

#[async_trait]
impl Agent for SequentialAgent {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    async fn run(
        &self,
        ctx: Arc<dyn InvocationContext>,
    ) -> Box<dyn Stream<Item = Result<Event>> + Send + Unpin> {
        self.inner.run(ctx).await
    }

    fn sub_agents(&self) -> &[Arc<dyn Agent>] {
        self.inner.sub_agents()
    }
}

/// Builder for SequentialAgent
pub struct SequentialAgentBuilder {
    core: AgentBuilderCore,
    sub_agents: Vec<Arc<dyn Agent>>,
}

impl SequentialAgentBuilder {
    pub fn new() -> Self {
        Self {
            core: AgentBuilderCore::new(),
            sub_agents: Vec::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.core.with_name(name);
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.core.with_description(description);
        self
    }

    pub fn sub_agent(mut self, agent: Arc<dyn Agent>) -> Self {
        self.sub_agents.push(agent);
        self
    }

    pub fn sub_agents(mut self, agents: Vec<Arc<dyn Agent>>) -> Self {
        self.sub_agents = agents;
        self
    }

    pub fn build(self) -> Result<SequentialAgent> {
        let (name, description) = self.core.validate(
            "SequentialAgent",
            "A sequential agent that runs sub-agents in order",
        )?;

        if self.sub_agents.is_empty() {
            return Err(Error::Config(
                "SequentialAgent requires at least one sub-agent".to_string(),
            ));
        }

        // Create LoopAgent with max_iterations=1
        let inner = LoopAgent {
            name: Arc::from(name),
            description: Arc::from(description),
            sub_agents: self.sub_agents,
            max_iterations: 1,
        };

        Ok(SequentialAgent { inner })
    }
}

impl Default for SequentialAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockAgent;

    #[test]
    fn test_sequential_agent_builder() {
        let agent1 = Arc::new(MockAgent::new("agent1").with_response("Response 1"));

        let sequential_agent = SequentialAgent::builder()
            .name("test_sequential")
            .description("Test sequential agent")
            .sub_agent(agent1)
            .build()
            .unwrap();

        assert_eq!(sequential_agent.name(), "test_sequential");
        assert_eq!(sequential_agent.sub_agents().len(), 1);
    }

    #[test]
    fn test_sequential_agent_requires_name() {
        let agent1 = Arc::new(MockAgent::new("agent1").with_response("Response 1"));

        let result = SequentialAgent::builder().sub_agent(agent1).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_sequential_agent_requires_sub_agents() {
        let result = SequentialAgent::builder().name("test_sequential").build();

        assert!(result.is_err());
    }
}
