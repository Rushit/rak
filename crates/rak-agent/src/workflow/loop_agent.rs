use rak_core::{Agent, Error, Event, InvocationContext, Result};
use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;

/// LoopAgent repeatedly runs its sub-agents in sequence for a specified number
/// of iterations or until a termination condition is met.
///
/// Use the LoopAgent when your workflow involves repetition or iterative
/// refinement, such as revising code or iteratively improving responses.
pub struct LoopAgent {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) sub_agents: Vec<Arc<dyn Agent>>,
    pub(crate) max_iterations: u32,
}

impl LoopAgent {
    pub fn builder() -> LoopAgentBuilder {
        LoopAgentBuilder::new()
    }
}

#[async_trait]
impl Agent for LoopAgent {
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
        let max_iterations = self.max_iterations;

        Box::new(Box::pin(stream! {
            let mut count = max_iterations;

            // Loop indefinitely if max_iterations is 0, otherwise loop count times
            loop {
                let mut should_exit = false;

                // Run each sub-agent in sequence
                for sub_agent in &sub_agents {
                    let mut sub_stream = sub_agent.run(ctx.clone()).await;

                    while let Some(result) = sub_stream.next().await {
                        match result {
                            Ok(event) => {
                                // Check for escalate flag
                                if event.actions.escalate {
                                    should_exit = true;
                                }

                                yield Ok(event);
                            }
                            Err(e) => {
                                yield Err(e);
                                return;
                            }
                        }
                    }

                    if should_exit {
                        return;
                    }
                }

                // Decrement count and check if we should exit
                if max_iterations > 0 {
                    count -= 1;
                    if count == 0 {
                        return;
                    }
                }
            }
        }))
    }

    fn sub_agents(&self) -> &[Arc<dyn Agent>] {
        &self.sub_agents
    }
}

/// Builder for LoopAgent
pub struct LoopAgentBuilder {
    name: Option<String>,
    description: Option<String>,
    sub_agents: Vec<Arc<dyn Agent>>,
    max_iterations: u32,
}

impl LoopAgentBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            sub_agents: Vec::new(),
            max_iterations: 0, // 0 = infinite
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

    pub fn max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    pub fn build(self) -> Result<LoopAgent> {
        let name = self
            .name
            .ok_or_else(|| Error::Other(anyhow::anyhow!("LoopAgent name is required")))?;
        let description = self
            .description
            .unwrap_or_else(|| "A loop agent that iterates over sub-agents".to_string());

        if self.sub_agents.is_empty() {
            return Err(Error::Other(anyhow::anyhow!(
                "LoopAgent requires at least one sub-agent"
            )));
        }

        Ok(LoopAgent {
            name,
            description,
            sub_agents: self.sub_agents,
            max_iterations: self.max_iterations,
        })
    }
}

impl Default for LoopAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rak_core::{Content, Part};

    // Mock agent for testing
    struct MockAgent {
        name: String,
        response: String,
        escalate: bool,
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
            let escalate = self.escalate;
            let invocation_id = ctx.invocation_id().to_string();
            let name = self.name.clone();

            Box::new(Box::pin(stream! {
                let mut event = Event::new(invocation_id, name);
                event.content = Some(Content {
                    role: "model".to_string(),
                    parts: vec![Part::Text { text: response }],
                });
                event.turn_complete = true;
                event.actions.escalate = escalate;

                yield Ok(event);
            }))
        }

        fn sub_agents(&self) -> &[Arc<dyn Agent>] {
            &[]
        }
    }

    #[test]
    fn test_loop_agent_builder() {
        let agent1 = Arc::new(MockAgent {
            name: "agent1".to_string(),
            response: "Response 1".to_string(),
            escalate: false,
        });

        let loop_agent = LoopAgent::builder()
            .name("test_loop")
            .description("Test loop agent")
            .sub_agent(agent1)
            .max_iterations(2)
            .build()
            .unwrap();

        assert_eq!(loop_agent.name(), "test_loop");
        assert_eq!(loop_agent.sub_agents().len(), 1);
    }

    #[test]
    fn test_loop_agent_requires_name() {
        let agent1 = Arc::new(MockAgent {
            name: "agent1".to_string(),
            response: "Response 1".to_string(),
            escalate: false,
        });

        let result = LoopAgent::builder().sub_agent(agent1).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_loop_agent_requires_sub_agents() {
        let result = LoopAgent::builder().name("test_loop").build();

        assert!(result.is_err());
    }
}
