use crate::builder_common::AgentBuilderCore;
use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use zdk_core::{Agent, Error, Event, InvocationContext, Result};

/// ParallelAgent runs its sub-agents in parallel in an isolated manner.
///
/// This approach is beneficial for scenarios requiring multiple perspectives or
/// attempts on a single task, such as:
/// - Running different algorithms simultaneously
/// - Generating multiple responses for review by a subsequent evaluation agent
pub struct ParallelAgent {
    pub(crate) name: Arc<str>,
    pub(crate) description: Arc<str>,
    pub(crate) sub_agents: Vec<Arc<dyn Agent>>,
}

impl ParallelAgent {
    pub fn builder() -> ParallelAgentBuilder {
        ParallelAgentBuilder::new()
    }
}

#[async_trait]
impl Agent for ParallelAgent {
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
        let sub_agents = self.sub_agents.clone();
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Spawn a task for each sub-agent
        for sub_agent in sub_agents {
            let ctx = ctx.clone();
            let tx = tx.clone();

            tokio::spawn(async move {
                let mut stream = sub_agent.run(ctx).await;

                while let Some(result) = stream.next().await {
                    if tx.send(result).is_err() {
                        // Receiver dropped, stop processing
                        break;
                    }
                }
            });
        }

        // Drop the original sender so the receiver knows when all senders are done
        drop(tx);

        Box::new(Box::pin(stream! {
            while let Some(result) = rx.recv().await {
                yield result;
            }
        }))
    }

    fn sub_agents(&self) -> &[Arc<dyn Agent>] {
        &self.sub_agents
    }
}

/// Builder for ParallelAgent
pub struct ParallelAgentBuilder {
    core: AgentBuilderCore,
    sub_agents: Vec<Arc<dyn Agent>>,
}

impl ParallelAgentBuilder {
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

    pub fn build(self) -> Result<ParallelAgent> {
        let (name, description) = self.core.validate(
            "ParallelAgent",
            "A parallel agent that runs sub-agents concurrently",
        )?;

        if self.sub_agents.is_empty() {
            return Err(Error::Config(
                "ParallelAgent requires at least one sub-agent".to_string(),
            ));
        }

        Ok(ParallelAgent {
            name: Arc::from(name),
            description: Arc::from(description),
            sub_agents: self.sub_agents,
        })
    }
}

impl Default for ParallelAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockAgent;

    #[test]
    fn test_parallel_agent_builder() {
        let agent1 = Arc::new(MockAgent::new("agent1").with_response("Response 1"));

        let parallel_agent = ParallelAgent::builder()
            .name("test_parallel")
            .description("Test parallel agent")
            .sub_agent(agent1)
            .build()
            .unwrap();

        assert_eq!(parallel_agent.name(), "test_parallel");
        assert_eq!(parallel_agent.sub_agents().len(), 1);
    }

    #[test]
    fn test_parallel_agent_requires_name() {
        let agent1 = Arc::new(MockAgent::new("agent1").with_response("Response 1"));

        let result = ParallelAgent::builder().sub_agent(agent1).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_parallel_agent_requires_sub_agents() {
        let result = ParallelAgent::builder().name("test_parallel").build();

        assert!(result.is_err());
    }
}
