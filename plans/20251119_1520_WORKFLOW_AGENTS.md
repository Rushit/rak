# ZDK Workflow Agents Documentation

**Created:** 2025-11-19 15:20  
**Last Updated:** 2025-11-19 15:20  
**Status:** Phase 3 Complete

## Purpose

This document provides comprehensive documentation for ZDK workflow agents: LoopAgent, SequentialAgent, and ParallelAgent. These agents enable sophisticated multi-agent orchestration patterns.

---

## Overview

Workflow agents allow you to compose multiple agents into complex execution patterns. Unlike LLMAgent which interacts directly with an LLM, workflow agents orchestrate other agents (including other workflow agents) to create sophisticated multi-step processes.

###  Workflow Agent Types

1. **SequentialAgent**: Executes sub-agents in strict order
2. **ParallelAgent**: Executes sub-agents concurrently  
3. **LoopAgent**: Iterates over sub-agents with termination control

---

## SequentialAgent

### Description

SequentialAgent executes its sub-agents once, in the order they are listed. This is useful when you need a fixed, step-by-step process where each step depends on the previous one.

### When to Use

- Multi-step workflows with dependencies
- Pipeline processing
- Staged analysis (analyze → process → summarize)
- Any process requiring strict ordering

### Example

```rust
use rak_agent::{LLMAgent, SequentialAgent};
use std::sync::Arc;

// Create individual step agents
let step1 = Arc::new(
    LLMAgent::builder()
        .name("analyzer")
        .description("Analyzes the input")
        .model(model.clone())
        .build()?
);

let step2 = Arc::new(
    LLMAgent::builder()
        .name("processor")
        .description("Processes the analysis")
        .model(model.clone())
        .build()?
);

let step3 = Arc::new(
    LLMAgent::builder()
        .name("summarizer")
        .description("Summarizes the results")
        .model(model.clone())
        .build()?
);

// Create sequential workflow
let sequential_agent = SequentialAgent::builder()
    .name("three_step_process")
    .description("Analyze → Process → Summarize")
    .sub_agent(step1)
    .sub_agent(step2)
    .sub_agent(step3)
    .build()?;
```

### API Reference

```rust
pub struct SequentialAgent { /* ... */ }

impl SequentialAgent {
    pub fn builder() -> SequentialAgentBuilder;
}

pub struct SequentialAgentBuilder {
    pub fn name(self, name: impl Into<String>) -> Self;
    pub fn description(self, description: impl Into<String>) -> Self;
    pub fn sub_agent(self, agent: Arc<dyn Agent>) -> Self;
    pub fn sub_agents(self, agents: Vec<Arc<dyn Agent>>) -> Self;
    pub fn build(self) -> Result<SequentialAgent>;
}
```

### Execution Flow

```
User Input
   ↓
Agent 1 → Event 1.1, Event 1.2, ...
   ↓
Agent 2 → Event 2.1, Event 2.2, ...
   ↓
Agent 3 → Event 3.1, Event 3.2, ...
   ↓
Complete
```

---

## ParallelAgent

### Description

ParallelAgent executes its sub-agents concurrently using Tokio. All agents start at the same time and their events are merged into a single stream as they arrive.

### When to Use

- Independent tasks that can run simultaneously
- Multiple perspectives on the same input
- A/B testing different approaches
- Generating multiple options for comparison
- Maximizing throughput

### Example

```rust
use rak_agent::{LLMAgent, ParallelAgent};
use std::sync::Arc;

// Create agents with different perspectives
let poet = Arc::new(
    LLMAgent::builder()
        .name("poet")
        .description("Writes poetry")
        .model(model.clone())
        .build()?
);

let scientist = Arc::new(
    LLMAgent::builder()
        .name("scientist")
        .description("Provides scientific explanation")
        .model(model.clone())
        .build()?
);

let educator = Arc::new(
    LLMAgent::builder()
        .name("educator")
        .description("Explains for children")
        .model(model.clone())
        .build()?
);

// Create parallel workflow
let parallel_agent = ParallelAgent::builder()
    .name("multi_perspective")
    .description("Get multiple viewpoints simultaneously")
    .sub_agent(poet)
    .sub_agent(scientist)
    .sub_agent(educator)
    .build()?;
```

### API Reference

```rust
pub struct ParallelAgent { /* ... */ }

impl ParallelAgent {
    pub fn builder() -> ParallelAgentBuilder;
}

pub struct ParallelAgentBuilder {
    pub fn name(self, name: impl Into<String>) -> Self;
    pub fn description(self, description: impl Into<String>) -> Self;
    pub fn sub_agent(self, agent: Arc<dyn Agent>) -> Self;
    pub fn sub_agents(self, agents: Vec<Arc<dyn Agent>>) -> Self;
    pub fn build(self) -> Result<ParallelAgent>;
}
```

### Execution Flow

```
         User Input
            ↓
    ┌───────┼───────┐
    ↓       ↓       ↓
 Agent 1  Agent 2  Agent 3
    ↓       ↓       ↓
    └───────┼───────┘
            ↓
    Merged Event Stream
    (order may vary)
```

### Important Notes

- Events from different agents may be interleaved
- No guaranteed ordering between agents
- All agents share the same context
- Errors from any agent are propagated immediately

---

## LoopAgent

### Description

LoopAgent repeatedly executes its sub-agents in sequence for a specified number of iterations or until a termination condition is met via the escalate flag.

### When to Use

- Iterative refinement
- Retry logic
- Continuous processing until condition met
- Code review and improvement cycles
- Progressive enhancement

### Example

```rust
use rak_agent::{LLMAgent, LoopAgent};
use std::sync::Arc;

let refiner = Arc::new(
    LLMAgent::builder()
        .name("refiner")
        .description("Iteratively improves content")
        .model(model)
        .build()?
);

// Loop 5 times or until escalate
let loop_agent = LoopAgent::builder()
    .name("iterative_refinement")
    .description("Refines output over multiple passes")
    .sub_agent(refiner)
    .max_iterations(5)
    .build()?;

// Infinite loop (use escalate to exit)
let infinite_loop = LoopAgent::builder()
    .name("continuous_process")
    .sub_agent(worker)
    .max_iterations(0)  // 0 = infinite
    .build()?;
```

### API Reference

```rust
pub struct LoopAgent { /* ... */ }

impl LoopAgent {
    pub fn builder() -> LoopAgentBuilder;
}

pub struct LoopAgentBuilder {
    pub fn name(self, name: impl Into<String>) -> Self;
    pub fn description(self, description: impl Into<String>) -> Self;
    pub fn sub_agent(self, agent: Arc<dyn Agent>) -> Self;
    pub fn sub_agents(self, agents: Vec<Arc<dyn Agent>>) -> Self;
    pub fn max_iterations(self, max: u32) -> Self;  // 0 = infinite
    pub fn build(self) -> Result<LoopAgent>;
}
```

### Escalate Pattern

The escalate flag provides early exit from loops:

```rust
// In a tool or agent, set escalate to exit the loop
event.actions.escalate = true;
```

When any event has `actions.escalate = true`, the loop terminates immediately.

### Execution Flow

```
Iteration 1:
  Agent 1 → Events
  Agent 2 → Events (check escalate)
  ...

Iteration 2:
  Agent 1 → Events
  Agent 2 → Events (check escalate)
  ...

(continues until max_iterations or escalate = true)
```

---

## Nested Workflows

Workflow agents can contain other workflow agents, enabling complex orchestration patterns.

### Example 1: Sequential containing Parallel

```rust
// Create parallel exploration phase
let option1 = Arc::new(create_agent("option1", model.clone())?);
let option2 = Arc::new(create_agent("option2", model.clone())?);

let parallel_phase = Arc::new(
    ParallelAgent::builder()
        .name("explore_options")
        .sub_agent(option1)
        .sub_agent(option2)
        .build()?
);

// Wrap in sequential workflow
let workflow = SequentialAgent::builder()
    .name("analyze_explore_summarize")
    .sub_agent(analyzer)
    .sub_agent(parallel_phase)  // Nested parallel
    .sub_agent(summarizer)
    .build()?;
```

Flow:
```
Analyzer (sequential)
   ↓
┌─────────────┐
│ Option 1    │ (parallel)
│ Option 2    │
└─────────────┘
   ↓
Summarizer (sequential)
```

### Example 2: Loop containing Sequential

```rust
let sequential_step = Arc::new(
    SequentialAgent::builder()
        .name("review_cycle")
        .sub_agent(reviewer)
        .sub_agent(improver)
        .build()?
);

let loop_workflow = LoopAgent::builder()
    .name("iterative_improvement")
    .sub_agent(sequential_step)
    .max_iterations(3)
    .build()?;
```

Flow:
```
Iteration 1: Reviewer → Improver
Iteration 2: Reviewer → Improver
Iteration 3: Reviewer → Improver
```

### Example 3: Parallel containing Loops

```rust
let loop1 = Arc::new(
    LoopAgent::builder()
        .name("approach_a")
        .sub_agent(agent_a)
        .max_iterations(3)
        .build()?
);

let loop2 = Arc::new(
    LoopAgent::builder()
        .name("approach_b")
        .sub_agent(agent_b)
        .max_iterations(3)
        .build()?
);

let parallel_loops = ParallelAgent::builder()
    .name("concurrent_iterations")
    .sub_agent(loop1)
    .sub_agent(loop2)
    .build()?;
```

---

## Best Practices

### 1. Agent Naming

Use descriptive, hierarchical names:
```rust
// Good
"data_pipeline.extract"
"data_pipeline.transform"
"data_pipeline.load"

// Avoid
"agent1", "agent2", "agent3"
```

### 2. Iteration Limits

Always set reasonable max_iterations for LoopAgent:
```rust
// Good
.max_iterations(10)  // Clear limit

// Risky
.max_iterations(0)   // Infinite - ensure escalate is used
```

### 3. Error Handling

Workflow agents propagate errors immediately:
```rust
while let Some(result) = stream.next().await {
    match result {
        Ok(event) => {
            // Process event
        }
        Err(e) => {
            // Error from any sub-agent stops the workflow
            eprintln!("Workflow error: {}", e);
            break;
        }
    }
}
```

### 4. Escalate Usage

Use escalate for logical exits:
```rust
// Create a tool that sets escalate when done
let exit_tool = FunctionTool::builder()
    .name("mark_complete")
    .execute(|ctx, params| async move {
        let mut response = ToolResponse {
            result: json!({"status": "complete"}),
        };
        
        // Event will have escalate set via tool execution
        Ok(response)
    })
    .build()?;
```

### 5. Event Ordering

Remember event ordering guarantees:
- **Sequential**: Strict order (A then B then C)
- **Parallel**: No ordering guarantee (events interleaved)
- **Loop**: Order preserved within each iteration

---

## Common Patterns

### Pattern 1: Research Workflow

```rust
// Gather → Analyze → Synthesize
SequentialAgent::builder()
    .sub_agent(research_agent)
    .sub_agent(analysis_agent)
    .sub_agent(synthesis_agent)
    .build()?
```

### Pattern 2: Consensus Building

```rust
// Multiple agents vote in parallel, then decide
let voting = ParallelAgent::builder()
    .sub_agent(judge1)
    .sub_agent(judge2)
    .sub_agent(judge3)
    .build()?;

SequentialAgent::builder()
    .sub_agent(voting)
    .sub_agent(decision_maker)
    .build()?
```

### Pattern 3: Iterative Refinement

```rust
// Draft → Review → Improve (loop)
let review_cycle = SequentialAgent::builder()
    .sub_agent(reviewer)
    .sub_agent(improver)
    .build()?;

LoopAgent::builder()
    .sub_agent(review_cycle)
    .max_iterations(5)
    .build()?
```

### Pattern 4: A/B Testing

```rust
// Try two approaches in parallel
ParallelAgent::builder()
    .sub_agent(approach_a)
    .sub_agent(approach_b)
    .build()?
```

---

## Testing Workflow Agents

### Unit Testing

```rust
#[tokio::test]
async fn test_sequential_order() {
    let agent1 = Arc::new(MockAgent::new("agent1", "Response 1"));
    let agent2 = Arc::new(MockAgent::new("agent2", "Response 2"));
    
    let sequential = SequentialAgent::builder()
        .name("test")
        .sub_agent(agent1)
        .sub_agent(agent2)
        .build()
        .unwrap();
    
    let ctx = Arc::new(MockContext::new());
    let mut stream = sequential.run(ctx).await;
    
    let events: Vec<_> = stream.collect().await;
    
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].author, "agent1");
    assert_eq!(events[1].author, "agent2");
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_nested_workflow() {
    let inner = Arc::new(
        ParallelAgent::builder()
            .name("inner")
            .sub_agent(agent1)
            .sub_agent(agent2)
            .build()?
    );
    
    let outer = SequentialAgent::builder()
        .name("outer")
        .sub_agent(analyzer)
        .sub_agent(inner)
        .sub_agent(summarizer)
        .build()?;
    
    // Test execution and verify event order
}
```

---

## Performance Considerations

### Sequential Performance

- **Latency**: Sum of all sub-agent latencies
- **Throughput**: Limited by slowest agent
- **Memory**: One agent active at a time

### Parallel Performance

- **Latency**: Maximum of all sub-agent latencies
- **Throughput**: N agents execute simultaneously
- **Memory**: All agents active concurrently

### Loop Performance

- **Latency**: Iterations × sequential latency
- **Throughput**: Depends on max_iterations
- **Memory**: Same as sequential (one iteration at a time)

---

## Troubleshooting

### Issue: Parallel events out of order

**Solution**: This is expected. If order matters, use Sequential instead or process events after collection.

### Issue: Loop doesn't terminate

**Solution**: Check max_iterations is set (not 0) or ensure escalate flag is being set.

### Issue: Workflow stops unexpectedly

**Solution**: Check for errors in sub-agents. Any error propagates and stops the workflow.

### Issue: Nested workflow too complex

**Solution**: Break into smaller, testable components. Use clear naming conventions.

---

## See Also

- [Tool System Documentation](./20251119_1500_TOOL_SYSTEM.md)
- [Implementation Summary](./20251119_1400_IMPLEMENTATION_SUMMARY.md)
- [Project Scope](./20251119_1410_PROJECT_SCOPE.md)
- [Testing Guide](./20251119_1425_TESTING_GUIDE.md)

---

**Document Version**: 1.0  
**Status**: Phase 3 Complete  
**Last Updated**: November 19, 2024

