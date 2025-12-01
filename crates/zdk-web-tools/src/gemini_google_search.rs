//! Gemini Google Search built-in tool
//!
//! This tool enables Gemini's built-in Google Search capability, where search is performed
//! **inside the Gemini API**, not locally.

use async_trait::async_trait;
use zdk_core::{Result as RakResult, Tool, ToolContext, ToolResponse};
use serde_json::{json, Value};
use std::sync::Arc;

/// Gemini Google Search Tool
///
/// Enables Gemini's built-in Google Search capability. The search is performed
/// **inside the Gemini API**, not locally.
///
/// ## 🔑 API Keys Required
///
/// **✅ ZERO additional API keys needed!**
///
/// Uses the same Gemini API key you're already using for the model.
///
/// ## Requirements
///
/// - **Gemini 2.0+** model (e.g., `gemini-2.0-flash-exp`, `gemini-2.0-pro-exp`)
/// - Gemini API key (same one used for model)
///
/// ⚠️ **Note**: This tool currently requires future enhancement to `rak-model` to support
/// adding tools to the Gemini API request config. See documentation for details.
///
/// ## How It Works
///
/// Unlike traditional tools that execute locally, Gemini built-in tools:
/// 1. Are added to the model's configuration
/// 2. Execute inside Gemini's API
/// 3. Return results automatically to the model
/// 4. The model incorporates results into its response
///
/// This means zero local execution, zero API key management, and zero rate limiting concerns!
///
/// ## Example
///
/// ```rust
/// use zdk_web_tools::GeminiGoogleSearchTool;
/// use std::sync::Arc;
///
/// // Create Google Search tool - no additional keys needed!
/// let google_search = Arc::new(GeminiGoogleSearchTool::new());
///
/// // This tool can be added to your Gemini 2.0+ agent
/// // See examples/web_tools_usage.rs for a complete example
/// ```
///
/// ## Python RAK Equivalent
///
/// This matches Python RAK's `GoogleSearchTool`:
///
/// ```python
/// from google.adk.tools import google_search
///
/// agent = LlmAgent(
///     model="gemini-2.0-flash-exp",
///     tools=[google_search],  # No API key needed!
/// )
/// ```
///
/// ## Future Enhancement Needed
///
/// To fully enable this tool, `rak-model`'s `GeminiModel` needs to:
///
/// 1. Accept tools in `LLMRequest`
/// 2. Add them to the Gemini API request config:
///    ```json
///    {
///      "contents": [...],
///      "tools": [{
///        "googleSearch": {}
///      }]
///    }
///    ```
///
/// See: https://ai.google.dev/api/generate-content#tools
pub struct GeminiGoogleSearchTool {
    name: String,
    description: String,
}

impl GeminiGoogleSearchTool {
    /// Create a new Gemini Google Search tool
    pub fn new() -> Self {
        Self {
            name: "google_search".to_string(),
            description: "Search the web using Google. Returns top search results with titles, snippets, and URLs. Useful for finding current information, news, websites, and general knowledge. This tool uses Gemini's built-in search capability (no additional API keys needed).".to_string(),
        }
    }

    /// Create with custom name and description
    pub fn with_config(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl Default for GeminiGoogleSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GeminiGoogleSearchTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        _params: Value,
    ) -> RakResult<ToolResponse> {
        // This tool doesn't execute locally - it's handled by Gemini API
        // When model-level tool support is added, this will never be called
        //
        // For now, return a helpful message
        Ok(ToolResponse {
            result: json!({
                "info": "This is a Gemini built-in tool. It requires model-level configuration support in rak-model.",
                "status": "Model integration pending",
                "documentation": "See gemini_google_search.rs for implementation details"
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_google_search_tool_creation() {
        let tool = GeminiGoogleSearchTool::new();
        assert_eq!(tool.name(), "google_search");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_schema_generation() {
        let tool = GeminiGoogleSearchTool::new();
        let schema = tool.schema();
        assert!(schema["properties"]["query"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&json!("query")));
    }

    #[test]
    fn test_default() {
        let tool = GeminiGoogleSearchTool::default();
        assert_eq!(tool.name(), "google_search");
    }

    #[test]
    fn test_custom_config() {
        let tool = GeminiGoogleSearchTool::with_config(
            "custom_search".to_string(),
            "Custom description".to_string(),
        );
        assert_eq!(tool.name(), "custom_search");
        assert_eq!(tool.description(), "Custom description");
    }
}

