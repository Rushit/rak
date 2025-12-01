use zdk_core::{Agent, Error, Event, InvocationContext, Result};
use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

/// ParallelAgent runs its sub-agents in parallel in an isolated manner.
///
/// This approach is beneficial for scenarios requiring multiple perspectives or
/// attempts on a single task, such as:
/// - Running different algorithms simultaneously
/// - Generating multiple responses for review by a subsequent evaluation agent
pub struct ParallelAgent {
    pub(crate) name: String,
    pub(crate) description: String,
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
    name: Option<String>,
    description: Option<String>,
    sub_agents: Vec<Arc<dyn Agent>>,
}

impl ParallelAgentBuilder {
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

    pub fn build(self) -> Result<ParallelAgent> {
        let name = self
            .name
            .ok_or_else(|| Error::Other(anyhow::anyhow!("ParallelAgent name is required")))?;
        let description = self
            .description
            .unwrap_or_else(|| "A parallel agent that runs sub-agents concurrently".to_string());

        if self.sub_agents.is_empty() {
            return Err(Error::Other(anyhow::anyhow!(
                "ParallelAgent requires at least one sub-agent"
            )));
        }

        Ok(ParallelAgent {
            name,
            description,
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
    use zdk_core::{Content, Part};
    use futures::StreamExt;

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
    fn test_parallel_agent_builder() {
        let agent1 = Arc::new(MockAgent {
            name: "agent1".to_string(),
            response: "Response 1".to_string(),
        });

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
        let agent1 = Arc::new(MockAgent {
            name: "agent1".to_string(),
            response: "Response 1".to_string(),
        });

        let result = ParallelAgent::builder().sub_agent(agent1).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_parallel_agent_requires_sub_agents() {
        let result = ParallelAgent::builder().name("test_parallel").build();

        assert!(result.is_err());
    }
}
