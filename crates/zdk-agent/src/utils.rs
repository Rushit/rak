//! Utility functions for agent implementations

use std::collections::HashMap;
use std::sync::Arc;
use zdk_core::{InvocationContext, Tool, Toolset};

/// Load tools from multiple toolsets in parallel
///
/// This function loads tools from all provided toolsets concurrently, improving
/// performance when multiple toolsets (e.g., MCP servers) are used.
///
/// # Arguments
/// * `toolsets` - The toolsets to load tools from
/// * `ctx` - The invocation context
/// * `invocation_id` - The current invocation ID for logging
///
/// # Returns
/// A HashMap of tool names to Tool instances
///
/// # Example
/// ```no_run
/// use zdk_agent::utils::load_toolsets;
/// use std::sync::Arc;
///
/// # async fn example(toolsets: Vec<Arc<dyn zdk_core::Toolset>>, ctx: Arc<dyn zdk_core::InvocationContext>) {
/// let tools = load_toolsets(&toolsets, &ctx, "inv-123").await;
/// # }
/// ```
pub async fn load_toolsets(
    toolsets: &[Arc<dyn Toolset>],
    ctx: &Arc<dyn InvocationContext>,
    invocation_id: &str,
) -> HashMap<String, Arc<dyn Tool>> {
    let mut tools = HashMap::new();

    // Load all toolsets in parallel for better performance
    let toolset_futures: Vec<_> = toolsets
        .iter()
        .map(|toolset| {
            let ctx = ctx.clone();
            let toolset_name = toolset.name().to_string();
            let toolset = toolset.clone();
            let invocation_id = invocation_id.to_string();
            async move {
                match toolset.get_tools(&*ctx).await {
                    Ok(tools) => {
                        tracing::info!(
                            invocation_id = %invocation_id,
                            toolset = %toolset_name,
                            count = tools.len(),
                            "Loaded tools from toolset"
                        );
                        Some(tools)
                    }
                    Err(e) => {
                        tracing::error!(
                            invocation_id = %invocation_id,
                            toolset = %toolset_name,
                            error = %e,
                            "Failed to load toolset"
                        );
                        None
                    }
                }
            }
        })
        .collect();

    let toolset_results = futures::future::join_all(toolset_futures).await;

    // Merge loaded tools into the tools HashMap
    for tools_opt in toolset_results.into_iter().flatten() {
        for tool in tools_opt {
            tools.insert(tool.name().to_string(), tool);
        }
    }

    tools
}
