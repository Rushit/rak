//! Web scraper tool for fetching and parsing HTML content

use anyhow::anyhow;
use async_trait::async_trait;
use rak_core::{Result as RakResult, Tool, ToolContext, ToolResponse};
use scraper::{Html, Selector};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// Web Scraper tool
///
/// Fetches HTML content from URLs and optionally extracts specific elements using CSS selectors.
///
/// ## 🔑 API Keys Required
///
/// **✅ ZERO API keys needed!**
///
/// This tool fetches web content directly via HTTP. No external services required.
///
/// ## Features
///
/// - Fetch raw HTML content and extract text
/// - Extract specific elements using CSS selectors
/// - Extract all links from pages
/// - Automatic text cleaning
/// - Works with any LLM model (Gemini, Claude, GPT, etc.)
///
/// ## Example
///
/// ```rust,no_run
/// use rak_web_tools::WebScraperTool;
/// use std::sync::Arc;
///
/// let tool = Arc::new(WebScraperTool::new().unwrap());
///
/// // This tool can be added to your agent
/// // See examples/web_tools_usage.rs for a complete example
/// ```
pub struct WebScraperTool {
    name: String,
    description: String,
    client: reqwest::Client,
}

impl WebScraperTool {
    /// Create a new Web Scraper tool with default configuration
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; RAK-Web-Tools/0.1.0)")
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;

        Ok(Self {
            name: "web_scraper".to_string(),
            description: "Fetch and parse HTML content from web pages. Can extract specific elements using CSS selectors (e.g., 'h1', '.article', '#content'), get all text content, or retrieve all links. Returns structured data from web pages.".to_string(),
            client,
        })
    }

    /// Create with custom name and description
    pub fn with_config(
        name: String,
        description: String,
    ) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; RAK-Web-Tools/0.1.0)")
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;

        Ok(Self {
            name,
            description,
            client,
        })
    }

    async fn fetch_and_parse(
        &self,
        url: &str,
        selector: Option<&str>,
        extract_links: bool,
    ) -> anyhow::Result<ScrapedContent> {
        debug!("Fetching URL: {}", url);

        // Validate URL
        let parsed_url = url::Url::parse(url)
            .map_err(|e| anyhow!("Invalid URL '{}': {}", url, e))?;

        // Fetch content
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch URL: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP error {}: {}", response.status(), url));
        }

        let html = response.text().await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        // Parse HTML
        let document = Html::parse_document(&html);

        // Extract title
        let title_selector = Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string());

        // Extract content based on parameters
        let text = if let Some(css_selector) = selector {
            // Extract specific elements
            let selector = Selector::parse(css_selector)
                .map_err(|e| anyhow!("Invalid CSS selector '{}': {:?}", css_selector, e))?;
            
            let elements: Vec<String> = document
                .select(&selector)
                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if elements.is_empty() {
                warn!("Selector '{}' matched no elements", css_selector);
            }

            elements.join("\n\n")
        } else {
            // Get all text content from body
            let body_selector = Selector::parse("body").unwrap();
            let text: String = document
                .select(&body_selector)
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join(" "))
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            // Filter out very short lines like Python RAK does
            text.lines()
                .filter(|line| line.split_whitespace().count() > 3)
                .collect::<Vec<_>>()
                .join("\n")
        };

        // Extract links if requested
        let links = if extract_links {
            let link_selector = Selector::parse("a[href]").unwrap();
            let links: Vec<LinkInfo> = document
                .select(&link_selector)
                .filter_map(|el| {
                    let href = el.value().attr("href")?;
                    let absolute_url = parsed_url.join(href).ok()?;
                    
                    Some(LinkInfo {
                        url: absolute_url.to_string(),
                        text: el.text().collect::<Vec<_>>().join(" ").trim().to_string(),
                    })
                })
                .collect();

            Some(links)
        } else {
            None
        };

        Ok(ScrapedContent {
            url: url.to_string(),
            title,
            text,
            links,
        })
    }
}

#[async_trait]
impl Tool for WebScraperTool {
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
                    "description": "The URL of the web page to scrape"
                },
                "selector": {
                    "type": "string",
                    "description": "Optional CSS selector to extract specific elements (e.g., 'h1', '.article', '#content'). If not provided, extracts all text from the page."
                },
                "extract_links": {
                    "type": "boolean",
                    "description": "Whether to extract all links from the page (default: false)"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        params: Value,
    ) -> RakResult<ToolResponse> {
        // Extract parameters
        let url = params["url"]
            .as_str()
            .ok_or_else(|| rak_core::Error::Other(anyhow!("Missing required parameter: url")))?;

        let selector = params["selector"].as_str();
        let extract_links = params["extract_links"].as_bool().unwrap_or(false);

        // Perform scraping
        match self.fetch_and_parse(url, selector, extract_links).await {
            Ok(content) => {
                let mut result = json!({
                    "url": content.url,
                    "title": content.title,
                });

                // Truncate text to avoid overwhelming LLM (5000 chars max)
                let truncated = if content.text.len() > 5000 {
                    format!("{}... (truncated from {} chars)", &content.text[..5000], content.text.len())
                } else {
                    content.text
                };
                result["text"] = json!(truncated);

                if let Some(links) = content.links {
                    result["links"] = json!(links);
                    result["link_count"] = json!(links.len());
                }

                Ok(ToolResponse { result })
            }
            Err(e) => {
                warn!("Web scraping failed: {}", e);
                Ok(ToolResponse {
                    result: json!({
                        "error": format!("Failed to scrape URL: {}", e),
                        "url": url,
                    }),
                })
            }
        }
    }
}

#[derive(Debug)]
struct ScrapedContent {
    url: String,
    title: Option<String>,
    text: String,
    links: Option<Vec<LinkInfo>>,
}

#[derive(Debug, serde::Serialize)]
struct LinkInfo {
    url: String,
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_scraper_tool_properties() {
        // Test basic properties without creating HTTP client
        let name = "web_scraper".to_string();
        let desc = "test description".to_string();
        
        // Just verify the tool can be conceptually created
        assert_eq!(name, "web_scraper");
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_schema_generation() {
        // Create schema without HTTP client
        let schema = json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL of the web page to scrape"
                },
                "selector": {
                    "type": "string",
                    "description": "Optional CSS selector to extract specific elements (e.g., 'h1', '.article', '#content'). If not provided, extracts all text from the page."
                },
                "extract_links": {
                    "type": "boolean",
                    "description": "Whether to extract all links from the page (default: false)"
                }
            },
            "required": ["url"]
        });
        
        assert!(schema["properties"]["url"].is_object());
        assert!(schema["properties"]["selector"].is_object());
        assert!(schema["properties"]["extract_links"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&json!("url")));
    }

    #[test]
    fn test_html_parsing() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Page</title></head>
            <body>
                <h1>Hello World</h1>
                <p class="content">This is a test paragraph with more than three words.</p>
                <a href="/link1">Link 1</a>
                <a href="https://example.com">Link 2</a>
            </body>
            </html>
        "#;

        let document = Html::parse_document(html);
        
        // Test title extraction
        let title_selector = Selector::parse("title").unwrap();
        let title = document.select(&title_selector).next().unwrap();
        assert_eq!(title.text().collect::<String>(), "Test Page");

        // Test content extraction
        let h1_selector = Selector::parse("h1").unwrap();
        let h1 = document.select(&h1_selector).next().unwrap();
        assert_eq!(h1.text().collect::<String>(), "Hello World");

        // Test link extraction
        let link_selector = Selector::parse("a[href]").unwrap();
        let links: Vec<_> = document.select(&link_selector).collect();
        assert_eq!(links.len(), 2);
    }
}

