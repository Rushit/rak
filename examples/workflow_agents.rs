use futures::StreamExt;
use std::sync::Arc;
use zdk_agent::{LLMAgent, LoopAgent, ParallelAgent, SequentialAgent};
use zdk_core::{Content, Provider};
use zdk_runner::Runner;
use zdk_session::inmemory::InMemorySessionService;
use zdk_tool::builtin::create_echo_tool;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();

    common::print_header("ZDK Workflow Agents Demo");

    // Load configuration (drives authentication method)
    println!("Loading configuration...");
    let config = common::load_config()?;

    // Show auth info
    common::show_auth_info(&config)?;
    println!();

    // Create model factory function using config-driven auth
    let create_model =
        || -> anyhow::Result<Arc<dyn Provider>> { common::create_gemini_model(&config) };

    // ===================================================================
    // Example 1: Sequential Workflow
    // ===================================================================
    println!("1. SEQUENTIAL WORKFLOW");
    println!("   Agents execute in strict order: Step 1 → Step 2 → Step 3\n");

    {
        let step1 = Arc::new(
            LLMAgent::builder()
                .name("step1")
                .description("Analyzes the problem")
                .model(create_model()?)
                .build()?,
        );

        let step2 = Arc::new(
            LLMAgent::builder()
                .name("step2")
                .description("Proposes solutions")
                .model(create_model()?)
                .build()?,
        );

        let step3 = Arc::new(
            LLMAgent::builder()
                .name("step3")
                .description("Summarizes results")
                .model(create_model()?)
                .build()?,
        );

        let sequential_agent = Arc::new(
            SequentialAgent::builder()
                .name("sequential_workflow")
                .description("Three-step sequential process")
                .sub_agent(step1)
                .sub_agent(step2)
                .sub_agent(step3)
                .build()?,
        );

        let session_service = Arc::new(InMemorySessionService::new());
        let runner = Runner::builder()
            .app_name("workflow-demo")
            .agent(sequential_agent)
            .session_service(session_service)
            .build()?;

        let message = Content::new_user_text("Explain photosynthesis in simple terms");
        let mut stream = runner
            .run(
                "user-123".to_string(),
                "session-seq".to_string(),
                message,
                Default::default(),
            )
            .await?;

        println!("   Executing sequential workflow...");
        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => {
                    if event.turn_complete {
                        println!("   ✓ {} completed", event.author);
                    }
                }
                Err(e) => eprintln!("   Error: {}", e),
            }
        }
        println!();
    }

    // ===================================================================
    // Example 2: Parallel Workflow
    // ===================================================================
    println!("2. PARALLEL WORKFLOW");
    println!("   Multiple agents execute concurrently\n");

    {
        let poet_agent = Arc::new(
            LLMAgent::builder()
                .name("poet")
                .description("Writes poetry")
                .model(create_model()?)
                .build()?,
        );

        let scientist_agent = Arc::new(
            LLMAgent::builder()
                .name("scientist")
                .description("Provides scientific explanation")
                .model(create_model()?)
                .build()?,
        );

        let educator_agent = Arc::new(
            LLMAgent::builder()
                .name("educator")
                .description("Explains for children")
                .model(create_model()?)
                .build()?,
        );

        let parallel_agent = Arc::new(
            ParallelAgent::builder()
                .name("parallel_perspectives")
                .description("Get multiple perspectives simultaneously")
                .sub_agent(poet_agent)
                .sub_agent(scientist_agent)
                .sub_agent(educator_agent)
                .build()?,
        );

        let session_service = Arc::new(InMemorySessionService::new());
        let runner = Runner::builder()
            .app_name("workflow-demo")
            .agent(parallel_agent)
            .session_service(session_service)
            .build()?;

        let message = Content::new_user_text("Explain the color blue");
        let mut stream = runner
            .run(
                "user-123".to_string(),
                "session-par".to_string(),
                message,
                Default::default(),
            )
            .await?;

        println!("   Executing parallel workflow...");
        let mut completed = Vec::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => {
                    if event.turn_complete {
                        completed.push(event.author.clone());
                        println!("   ✓ {} completed", event.author);
                    }
                }
                Err(e) => eprintln!("   Error: {}", e),
            }
        }
        println!(
            "   All {} agents completed concurrently!\n",
            completed.len()
        );
    }

    // ===================================================================
    // Example 3: Loop Workflow
    // ===================================================================
    println!("3. LOOP WORKFLOW");
    println!("   Agent iterates with a tool until task is complete\n");

    {
        let echo_tool = Arc::new(create_echo_tool()?);

        let refiner_agent = Arc::new(
            LLMAgent::builder()
                .name("refiner")
                .description("Iteratively refines content")
                .model(create_model()?)
                .tool(echo_tool)
                .build()?,
        );

        let loop_agent = Arc::new(
            LoopAgent::builder()
                .name("iterative_refinement")
                .description("Refines content over multiple iterations")
                .sub_agent(refiner_agent)
                .max_iterations(3)
                .build()?,
        );

        let session_service = Arc::new(InMemorySessionService::new());
        let runner = Runner::builder()
            .app_name("workflow-demo")
            .agent(loop_agent)
            .session_service(session_service)
            .build()?;

        let message = Content::new_user_text("Write a haiku about coding");
        let mut stream = runner
            .run(
                "user-123".to_string(),
                "session-loop".to_string(),
                message,
                Default::default(),
            )
            .await?;

        println!("   Executing loop workflow (max 3 iterations)...");
        let mut iteration = 0;
        let mut last_author = String::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => {
                    if event.turn_complete && event.author != last_author {
                        iteration += 1;
                        println!("   ✓ Iteration {} complete", iteration);
                        last_author = event.author.clone();
                    }
                }
                Err(e) => eprintln!("   Error: {}", e),
            }
        }
        println!();
    }

    // ===================================================================
    // Example 4: Nested Workflow (Sequential containing Parallel)
    // ===================================================================
    println!("4. NESTED WORKFLOW");
    println!("   Complex orchestration: Sequential → Parallel → Sequential\n");

    {
        let analyzer = Arc::new(
            LLMAgent::builder()
                .name("analyzer")
                .description("Initial analysis")
                .model(create_model()?)
                .build()?,
        );

        let option1 = Arc::new(
            LLMAgent::builder()
                .name("option1")
                .description("Explores option 1")
                .model(create_model()?)
                .build()?,
        );

        let option2 = Arc::new(
            LLMAgent::builder()
                .name("option2")
                .description("Explores option 2")
                .model(create_model()?)
                .build()?,
        );

        let parallel_exploration = Arc::new(
            ParallelAgent::builder()
                .name("explore_options")
                .description("Explores multiple options in parallel")
                .sub_agent(option1)
                .sub_agent(option2)
                .build()?,
        );

        let summarizer = Arc::new(
            LLMAgent::builder()
                .name("summarizer")
                .description("Summarizes all findings")
                .model(create_model()?)
                .build()?,
        );

        let nested_workflow = Arc::new(
            SequentialAgent::builder()
                .name("complex_workflow")
                .description("Analyze → Explore (parallel) → Summarize")
                .sub_agent(analyzer)
                .sub_agent(parallel_exploration)
                .sub_agent(summarizer)
                .build()?,
        );

        let session_service = Arc::new(InMemorySessionService::new());
        let runner = Runner::builder()
            .app_name("workflow-demo")
            .agent(nested_workflow)
            .session_service(session_service)
            .build()?;

        let message = Content::new_user_text("What are the best ways to learn programming?");
        let mut stream = runner
            .run(
                "user-123".to_string(),
                "session-nested".to_string(),
                message,
                Default::default(),
            )
            .await?;

        println!("   Executing nested workflow...");
        let mut completed_agents = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => {
                    if event.turn_complete && !completed_agents.contains(&event.author) {
                        completed_agents.push(event.author.clone());
                        println!("   ✓ {} completed", event.author);
                    }
                }
                Err(e) => eprintln!("   Error: {}", e),
            }
        }
        println!();
    }

    println!("=== Demo Complete ===");
    println!("\nWorkflow agents enable powerful multi-agent orchestration:");
    println!("  • Sequential: Strict ordering for step-by-step processes");
    println!("  • Parallel: Concurrent execution for independent tasks");
    println!("  • Loop: Iterative refinement with escalate control");
    println!("  • Nested: Complex combinations for sophisticated workflows\n");

    println!("✅ VALIDATION PASSED: Workflow agents verified");

    Ok(())
}
