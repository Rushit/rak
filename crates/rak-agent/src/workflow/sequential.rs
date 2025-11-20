use super::loop_agent::LoopAgent;
use rak_core::{Agent, Error, Event, InvocationContext, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use std::sync::Arc;

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
    name: Option<String>,
    description: Option<String>,
    sub_agents: Vec<Arc<dyn Agent>>,
}

impl SequentialAgentBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            sub_agents: Vec::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
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
        let name = self
            .name
            .ok_or_else(|| Error::Other(anyhow::anyhow!("SequentialAgent name is required")))?;
        let description = self
            .description
            .unwrap_or_else(|| "A sequential agent that runs sub-agents in order".to_string());

        if self.sub_agents.is_empty() {
            return Err(Error::Other(anyhow::anyhow!(
                "SequentialAgent requires at least one sub-agent"
            )));
        }

        // Create LoopAgent with max_iterations=1
        let inner = LoopAgent {
            name,
            description,
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
    use rak_core::{Content, Part};
    use async_stream::stream;

    struct MockAgent {
        name: String,
        response: String,
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Mock agent"
        }

        async fn run(
            &self,
            ctx: Arc<dyn InvocationContext>,
        ) -> Box<dyn Stream<Item = Result<Event>> + Send + Unpin> {
            let response = self.response.clone();
            let invocation_id = ctx.invocation_id().to_string();
            let name = self.name.clone();

            Box::new(Box::pin(stream! {
                let mut event = Event::new(invocation_id, name);
                event.content = Some(Content {
                    role: "model".to_string(),
                    parts: vec![Part::Text { text: response }],
                });
                event.turn_complete = true;

                yield Ok(event);
            }))
        }

        fn sub_agents(&self) -> &[Arc<dyn Agent>] {
            &[]
        }
    }

    #[test]
    fn test_sequential_agent_builder() {
        let agent1 = Arc::new(MockAgent {
            name: "agent1".to_string(),
            response: "Response 1".to_string(),
        });

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
        let agent1 = Arc::new(MockAgent {
            name: "agent1".to_string(),
            response: "Response 1".to_string(),
        });

        let result = SequentialAgent::builder().sub_agent(agent1).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_sequential_agent_requires_sub_agents() {
        let result = SequentialAgent::builder().name("test_sequential").build();

        assert!(result.is_err());
    }
}
