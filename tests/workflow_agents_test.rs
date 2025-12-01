use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use zdk_agent::{LoopAgent, ParallelAgent, SequentialAgent};
use zdk_core::{Agent, Content, Event, InvocationContext, Part, ReadonlyContext, Result};

// Mock context for testing
struct MockContext {
    invocation_id: String,
    user_content: Option<Content>,
}

impl MockContext {
    fn new() -> Self {
        Self {
            invocation_id: "test-inv-1".to_string(),
            user_content: Some(Content::new_user_text("Test message")),
        }
    }
}

#[async_trait]
impl InvocationContext for MockContext {
    fn invocation_id(&self) -> &str {
        &self.invocation_id
    }

    fn user_content(&self) -> Option<&Content> {
        self.user_content.as_ref()
    }
}

impl ReadonlyContext for MockContext {
    fn app_name(&self) -> &str {
        "test-app"
    }

    fn user_id(&self) -> &str {
        "test-user"
    }

    fn session_id(&self) -> &str {
        "test-session"
    }
}

// Mock agent that emits a single event
struct MockAgent {
    name: String,
    response: String,
    escalate: bool,
}

impl MockAgent {
    fn new(name: impl Into<String>, response: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            response: response.into(),
            escalate: false,
        }
    }

    fn with_escalate(mut self) -> Self {
        self.escalate = true;
        self
    }
}

#[async_trait]
impl Agent for MockAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Mock agent for testing"
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

#[tokio::test]
async fn test_loop_agent_with_max_iterations() {
    let agent1 = Arc::new(MockAgent::new("agent1", "Response 1"));
    let agent2 = Arc::new(MockAgent::new("agent2", "Response 2"));

    let loop_agent = LoopAgent::builder()
        .name("test_loop")
        .sub_agent(agent1)
        .sub_agent(agent2)
        .max_iterations(2)
        .build()
        .unwrap();

    let ctx = Arc::new(MockContext::new());
    let mut stream = loop_agent.run(ctx).await;

    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        events.push(result.unwrap());
    }

    // Should have 4 events: 2 iterations * 2 agents
    assert_eq!(events.len(), 4);
    assert_eq!(events[0].author, "agent1");
    assert_eq!(events[1].author, "agent2");
    assert_eq!(events[2].author, "agent1");
    assert_eq!(events[3].author, "agent2");
}

#[tokio::test]
async fn test_loop_agent_with_escalate() {
    let agent1 = Arc::new(MockAgent::new("agent1", "Response 1"));
    let agent2 = Arc::new(MockAgent::new("agent2", "Response 2").with_escalate());

    let loop_agent = LoopAgent::builder()
        .name("test_loop")
        .sub_agent(agent1)
        .sub_agent(agent2)
        .max_iterations(10) // Won't reach this
        .build()
        .unwrap();

    let ctx = Arc::new(MockContext::new());
    let mut stream = loop_agent.run(ctx).await;

    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        events.push(result.unwrap());
    }

    // Should have only 2 events: escalate stops after first iteration
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].author, "agent1");
    assert_eq!(events[1].author, "agent2");
    assert!(events[1].actions.escalate);
}

#[tokio::test]
async fn test_loop_agent_infinite_with_manual_break() {
    let agent1 = Arc::new(MockAgent::new("agent1", "Response 1"));

    let loop_agent = LoopAgent::builder()
        .name("test_loop")
        .sub_agent(agent1)
        .max_iterations(0) // Infinite
        .build()
        .unwrap();

    let ctx = Arc::new(MockContext::new());
    let mut stream = loop_agent.run(ctx).await;

    let mut events = Vec::new();
    let max_events = 5;
    while let Some(result) = stream.next().await {
        events.push(result.unwrap());
        if events.len() >= max_events {
            break;
        }
    }

    // Should collect exactly 5 events before manual break
    assert_eq!(events.len(), 5);
}

#[tokio::test]
async fn test_sequential_agent_order() {
    let agent1 = Arc::new(MockAgent::new("agent1", "First"));
    let agent2 = Arc::new(MockAgent::new("agent2", "Second"));
    let agent3 = Arc::new(MockAgent::new("agent3", "Third"));

    let sequential_agent = SequentialAgent::builder()
        .name("test_sequential")
        .sub_agent(agent1)
        .sub_agent(agent2)
        .sub_agent(agent3)
        .build()
        .unwrap();

    let ctx = Arc::new(MockContext::new());
    let mut stream = sequential_agent.run(ctx).await;

    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        events.push(result.unwrap());
    }

    // Should have exactly 3 events in order (one iteration only)
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].author, "agent1");
    assert_eq!(events[1].author, "agent2");
    assert_eq!(events[2].author, "agent3");

    // Verify content
    if let Some(content) = &events[0].content {
        if let Part::Text { text } = &content.parts[0] {
            assert_eq!(text, "First");
        } else {
            panic!("Expected text part");
        }
    }
}

#[tokio::test]
async fn test_parallel_agent_concurrent_execution() {
    let agent1 = Arc::new(MockAgent::new("agent1", "Response 1"));
    let agent2 = Arc::new(MockAgent::new("agent2", "Response 2"));
    let agent3 = Arc::new(MockAgent::new("agent3", "Response 3"));

    let parallel_agent = ParallelAgent::builder()
        .name("test_parallel")
        .sub_agent(agent1)
        .sub_agent(agent2)
        .sub_agent(agent3)
        .build()
        .unwrap();

    let ctx = Arc::new(MockContext::new());
    let mut stream = parallel_agent.run(ctx).await;

    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        events.push(result.unwrap());
    }

    // Should have exactly 3 events (order may vary due to concurrency)
    assert_eq!(events.len(), 3);

    // Collect authors (order may vary)
    let mut authors: Vec<String> = events.iter().map(|e| e.author.clone()).collect();
    authors.sort();

    assert_eq!(authors, vec!["agent1", "agent2", "agent3"]);
}

#[tokio::test]
async fn test_nested_sequential_agents() {
    let inner_agent1 = Arc::new(MockAgent::new("inner1", "Inner 1"));
    let inner_agent2 = Arc::new(MockAgent::new("inner2", "Inner 2"));

    let inner_sequential = Arc::new(
        SequentialAgent::builder()
            .name("inner_sequential")
            .sub_agent(inner_agent1)
            .sub_agent(inner_agent2)
            .build()
            .unwrap(),
    );

    let outer_agent1 = Arc::new(MockAgent::new("outer1", "Outer 1"));

    let outer_sequential = SequentialAgent::builder()
        .name("outer_sequential")
        .sub_agent(outer_agent1)
        .sub_agent(inner_sequential)
        .build()
        .unwrap();

    let ctx = Arc::new(MockContext::new());
    let mut stream = outer_sequential.run(ctx).await;

    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        events.push(result.unwrap());
    }

    // Should have 3 events: outer1, inner1, inner2
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].author, "outer1");
    assert_eq!(events[1].author, "inner1");
    assert_eq!(events[2].author, "inner2");
}

#[tokio::test]
async fn test_nested_loop_in_sequential() {
    let loop_agent1 = Arc::new(MockAgent::new("loop1", "Loop 1"));

    let loop_agent = Arc::new(
        LoopAgent::builder()
            .name("inner_loop")
            .sub_agent(loop_agent1)
            .max_iterations(2)
            .build()
            .unwrap(),
    );

    let seq_agent1 = Arc::new(MockAgent::new("seq1", "Seq 1"));

    let sequential_agent = SequentialAgent::builder()
        .name("outer_sequential")
        .sub_agent(seq_agent1)
        .sub_agent(loop_agent)
        .build()
        .unwrap();

    let ctx = Arc::new(MockContext::new());
    let mut stream = sequential_agent.run(ctx).await;

    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        events.push(result.unwrap());
    }

    // Should have 3 events: seq1, loop1 (iteration 1), loop1 (iteration 2)
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].author, "seq1");
    assert_eq!(events[1].author, "loop1");
    assert_eq!(events[2].author, "loop1");
}

#[tokio::test]
async fn test_parallel_with_sequential_nested() {
    let seq1 = Arc::new(MockAgent::new("seq1", "Seq 1"));
    let seq2 = Arc::new(MockAgent::new("seq2", "Seq 2"));

    let sequential1 = Arc::new(
        SequentialAgent::builder()
            .name("sequential1")
            .sub_agent(seq1)
            .sub_agent(seq2)
            .build()
            .unwrap(),
    );

    let parallel_agent1 = Arc::new(MockAgent::new("parallel1", "Parallel 1"));

    let parallel_agent = ParallelAgent::builder()
        .name("test_parallel")
        .sub_agent(sequential1)
        .sub_agent(parallel_agent1)
        .build()
        .unwrap();

    let ctx = Arc::new(MockContext::new());
    let mut stream = parallel_agent.run(ctx).await;

    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        events.push(result.unwrap());
    }

    // Should have 3 events: seq1, seq2, parallel1 (order may vary)
    assert_eq!(events.len(), 3);

    let mut authors: Vec<String> = events.iter().map(|e| e.author.clone()).collect();
    authors.sort();

    assert_eq!(authors, vec!["parallel1", "seq1", "seq2"]);
}
