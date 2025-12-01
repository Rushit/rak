//! Web and Search Tools for ZDK
//!
//! This crate provides tools for web search and scraping, optimized for Gemini models.
//!
//! ## 🔑 API Keys Required
//!
//! ### Gemini Built-in Tools (Recommended)
//!
//! **✅ ZERO API keys needed!**
//!
//! - **GeminiGoogleSearchTool** - No keys required, uses Gemini's built-in search
//! - **GeminiUrlContextTool** - No keys required, uses Gemini's built-in URL fetching
//!
//! **Requirements**:
//! - Gemini 2.0+ model (e.g., `gemini-2.0-flash-exp`)
//! - Gemini API key (same one you use for the model)
//!
//! ### Web Scraper Tool
//!
//! **✅ ZERO API keys needed!**
//!
//! - **WebScraperTool** - No keys required, fetches and parses HTML directly
//!
//! **Requirements**:
//! - Internet connection
//! - Works with any model (Gemini, Claude, GPT, etc.)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use zdk_web_tools::{GeminiGoogleSearchTool, GeminiUrlContextTool, WebScraperTool};
//! use std::sync::Arc;
//!
//! // Create tools - NO additional API keys needed!
//! let google_search = Arc::new(GeminiGoogleSearchTool::new());
//! let url_context = Arc::new(GeminiUrlContextTool::new());
//! let web_scraper = Arc::new(WebScraperTool::new().unwrap());
//!
//! // These tools can be added to your agent
//! // See examples/web_tools_usage.rs for a complete example
//! ```
//!
//! ## Tool Details
//!
//! ### GeminiGoogleSearchTool
//!
//! Enables Gemini's built-in Google Search capability. The search is performed
//! **inside the Gemini API**, not locally.
//!
//! - ✅ No setup required
//! - ✅ No rate limits (managed by Google)
//! - ✅ Returns search results automatically
//! - ⚠️ Only works with Gemini 2.0+ models
//!
//! ### GeminiUrlContextTool
//!
//! Enables Gemini's built-in URL fetching capability. The URL content is fetched
//! **inside the Gemini API**, not locally.
//!
//! - ✅ No setup required
//! - ✅ Handles authentication, redirects, etc.
//! - ✅ Automatically extracts relevant content
//! - ⚠️ Only works with Gemini 2.0+ models
//!
//! ### WebScraperTool
//!
//! Fetches and parses HTML content from any URL. Works with any model.
//!
//! - ✅ CSS selector support for targeted extraction
//! - ✅ Link extraction
//! - ✅ Automatic text cleaning
//! - ✅ Works with all models
//!
//! ## Future Extensions
//!
//! This crate currently focuses on Gemini's built-in capabilities. Future versions may add:
//!
//! - Google Custom Search API support (for non-Gemini models)
//! - Advanced web scraping features
//! - Rate limiting and caching
//! - Support for other search providers

mod gemini_google_search;
mod gemini_url_context;
mod web_scraper;

pub use gemini_google_search::GeminiGoogleSearchTool;
pub use gemini_url_context::GeminiUrlContextTool;
pub use web_scraper::WebScraperTool;

/// Result type for web tools
pub type Result<T> = std::result::Result<T, anyhow::Error>;
