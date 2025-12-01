//! Gemini URL Context built-in tool
//!
//! This tool enables Gemini's built-in URL fetching capability, where URL content is fetched
//! **inside the Gemini API**, not locally.

use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;
use zdk_core::{Result as ZResult, Tool, ToolContext, ToolResponse};

/// Gemini URL Context Tool
///
/// Enables Gemini's built-in URL fetching capability. The URL content is fetched
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
/// ⚠️ **Note**: This tool currently requires future enhancement to `zdk-model` to support
/// adding tools to the Gemini API request config. See documentation for details.
///
/// ## How It Works
///
/// Unlike traditional tools that execute locally, Gemini built-in tools:
/// 1. Are added to the model's configuration
/// 2. Execute inside Gemini's API
/// 3. Fetch and process URLs automatically
/// 4. Return extracted content to the model
/// 5. The model incorporates results into its response
///
/// This means zero local execution, automatic handling of authentication/redirects,
/// and smart content extraction!
///
/// ## Example
///
/// ```rust
/// use zdk_web_tools::GeminiUrlContextTool;
/// use std::sync::Arc;
///
/// // Create URL Context tool - no additional keys needed!
/// let url_context = Arc::new(GeminiUrlContextTool::new());
///
/// // This tool can be added to your Gemini 2.0+ agent
/// // See examples/web_tools_usage.rs for a complete example
/// ```
///
/// ## Python ZDK Equivalent
///
/// This matches Python ZDK's `UrlContextTool`:
///
/// ```python
/// from google.adk.tools import url_context
///
/// agent = LlmAgent(
///     model="gemini-2.0-flash-exp",
///     tools=[url_context],  # No API key needed!
/// )
/// ```
///
/// ## Use Cases
///
/// - Summarize articles from URLs
/// - Answer questions about web page content
/// - Extract specific information from websites
/// - Compare content across multiple URLs
/// - Fact-check using source URLs
///
/// ## vs WebScraperTool
///
/// | Feature | GeminiUrlContextTool | WebScraperTool |
/// |---------|---------------------|----------------|
/// | Execution | Inside Gemini API | Local |
/// | Model Support | Gemini 2.0+ only | All models |
/// | Setup | Zero | Zero |
/// | Content Extraction | Automatic, smart | Manual parsing |
/// | Authentication | Handled by Google | Basic |
/// | Best For | Simple URL reading | Advanced scraping |
///
/// ## Future Enhancement Needed
///
/// To fully enable this tool, `zdk-model`'s `GeminiModel` needs to:
///
/// 1. Accept tools in `LLMRequest`
/// 2. Add them to the Gemini API request config:
///    ```json
///    {
///      "contents": [...],
///      "tools": [{
///        "urlContext": {}
///      }]
///    }
///    ```
///
/// See: https://ai.google.dev/api/generate-content#tools
pub struct GeminiUrlContextTool {
    name: String,
    description: String,
}

impl GeminiUrlContextTool {
    /// Create a new Gemini URL Context tool
    pub fn new() -> Self {
        Self {
            name: "url_context".to_string(),
            description: "Fetch and read content from web URLs. Returns the text content of web pages. Useful for reading articles, documentation, and any web content. This tool uses Gemini's built-in URL fetching capability (no additional API keys needed).".to_string(),
        }
    }

    /// Create with custom name and description
    pub fn with_config(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl Default for GeminiUrlContextTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GeminiUrlContextTool {
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
                "url": {
                    "type": "string",
                    "description": "The URL to fetch and read"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _params: Value) -> ZResult<ToolResponse> {
        // This tool doesn't execute locally - it's handled by Gemini API
        // When model-level tool support is added, this will never be called
        //
        // For now, return a helpful message
        Ok(ToolResponse {
            result: json!({
                "info": "This is a Gemini built-in tool. It requires model-level configuration support in zdk-model.",
                "status": "Model integration pending",
                "documentation": "See gemini_url_context.rs for implementation details"
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_url_context_tool_creation() {
        let tool = GeminiUrlContextTool::new();
        assert_eq!(tool.name(), "url_context");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_schema_generation() {
        let tool = GeminiUrlContextTool::new();
        let schema = tool.schema();
        assert!(schema["properties"]["url"].is_object());
        assert!(
            schema["required"]
                .as_array()
                .unwrap()
                .contains(&json!("url"))
        );
    }

    #[test]
    fn test_default() {
        let tool = GeminiUrlContextTool::default();
        assert_eq!(tool.name(), "url_context");
    }

    #[test]
    fn test_custom_config() {
        let tool = GeminiUrlContextTool::with_config(
            "custom_url_reader".to_string(),
            "Custom description".to_string(),
        );
        assert_eq!(tool.name(), "custom_url_reader");
        assert_eq!(tool.description(), "Custom description");
    }
}
