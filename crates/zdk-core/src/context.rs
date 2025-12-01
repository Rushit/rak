use super::Content;
use async_trait::async_trait;

/// Invocation context provided to agents during execution
#[async_trait]
pub trait InvocationContext: ReadonlyContext + Send + Sync {
    /// Returns a unique ID for this invocation
    fn invocation_id(&self) -> &str;

    /// Returns the user content that triggered this invocation
    fn user_content(&self) -> Option<&Content>;
}

/// Read-only context for callbacks and tools
pub trait ReadonlyContext: Send + Sync {
    /// Returns the app name
    fn app_name(&self) -> &str;

    /// Returns the user ID
    fn user_id(&self) -> &str;

    /// Returns the session ID
    fn session_id(&self) -> &str;
}

/// Tool context provided during tool execution
pub trait ToolContext: Send + Sync {
    fn function_call_id(&self) -> &str;
    fn invocation_id(&self) -> &str;
}
