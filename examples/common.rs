//! Common utilities for ZDK examples
//!
//! This module provides shared helper functions that all examples can use,
//! making them simpler and more consistent. The key design principle is
//! **config-driven authentication** - users control auth method via config.toml.
//!
//! Note: This file is meant to be included in examples via `#[path = "common.rs"] mod common;`
//! It's not a standalone example, so Cargo warnings about unused functions are expected.

#![allow(dead_code)] // Functions are used by examples that include this module

use anyhow::{Context, Result};
use std::sync::Arc;
use zdk_core::{AuthCredentials, Provider, ZConfig, ZConfigExt};

/// Create an authenticated Gemini model from configuration
///
/// This function reads authentication credentials from config.toml and creates
/// the appropriate model based on the configured auth provider:
/// - `api_key` → Public Gemini API with API key
/// - `gcloud` → Vertex AI with gcloud CLI authentication
///
/// # Example
///
/// ```rust
/// let config = common::load_config()?;
/// let model = common::create_gemini_model(&config)?;
/// ```
///
/// # Configuration
///
/// User controls authentication via config.toml:
///
/// ```toml
/// [auth]
/// provider = "gcloud"  # or "api_key"
///
/// [auth.gcloud]
/// # project_id auto-detected from gcloud
/// # location defaults to us-central1
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - gcloud CLI is not installed or authenticated (when using gcloud provider)
/// - API key is missing or invalid (when using api_key provider)
/// - GCP project cannot be determined
pub fn create_gemini_model(config: &ZConfig) -> Result<Arc<dyn Provider>> {
    // Use the new provider system
    let provider = config
        .create_provider()
        .context("Failed to create provider from config")?;

    // Print what we're using
    println!(
        "✓ Provider created: {} (provider: {})",
        config.model.model_name, config.model.provider
    );

    Ok(provider)
}

/// Load ZDK configuration with helpful error messages
///
/// Attempts to load configuration from:
/// 1. config.toml (current directory)
/// 2. Parent directories (searches upward)
///
/// # Example
///
/// ```rust
/// let config = common::load_config()?;
/// ```
///
/// # Errors
///
/// Returns a helpful error message if:
/// - config.toml doesn't exist
/// - config.toml has invalid syntax
/// - Required fields are missing
///
/// The error message includes setup instructions to help users get started.
pub fn load_config() -> Result<ZConfig> {
    ZConfig::load().map_err(|e| {
        anyhow::anyhow!(
            "Failed to load config: {}\n\
             \n\
             📋 Setup Steps:\n\
             \n\
             1. Copy example config:\n\
                cp config.toml.example config.toml\n\
             \n\
             2. Choose authentication method:\n\
             \n\
             Option A - Google Cloud (Recommended):\n\
             [auth]\n\
             provider = \"gcloud\"\n\
             \n\
             Then run: gcloud auth login\n\
             \n\
             Option B - API Key:\n\
             [auth]\n\
             provider = \"api_key\"\n\
             \n\
             [auth.api_key]\n\
             key = \"${{GOOGLE_API_KEY}}\"\n\
             \n\
             Then set: export GOOGLE_API_KEY=\"your-key\"",
            e
        )
    })
}

/// Display authentication status for debugging
///
/// Prints information about the current authentication configuration.
/// Useful for troubleshooting auth issues.
///
/// # Example
///
/// ```rust
/// let config = common::load_config()?;
/// common::show_auth_info(&config)?;
/// ```
pub fn show_auth_info(config: &ZConfig) -> Result<()> {
    match config.get_auth_credentials()? {
        AuthCredentials::ApiKey { .. } => {
            println!("🔑 Authentication: API Key");
            println!("   Provider: Public Gemini API");
            println!("   Endpoint: https://generativelanguage.googleapis.com");
        }
        AuthCredentials::GCloud {
            project, location, ..
        } => {
            println!("🔑 Authentication: Google Cloud");
            println!("   Provider: Vertex AI");
            println!("   Project: {}", project);
            println!("   Location: {}", location);
            println!(
                "   Endpoint: https://{}-aiplatform.googleapis.com",
                location
            );
        }
    }
    Ok(())
}

/// Print example header banner
///
/// Displays a nice formatted header for examples.
///
/// # Example
///
/// ```rust
/// common::print_header("Quickstart Example");
/// ```
pub fn print_header(title: &str) {
    let width = 60;
    let padding = (width - title.len() - 2) / 2;

    println!("\n╔{}╗", "═".repeat(width));
    println!(
        "║{}{title}{}║",
        " ".repeat(padding),
        " ".repeat(width - padding - title.len())
    );
    println!("╚{}╝\n", "═".repeat(width));
}

/// Validation result for examples
pub enum ValidationResult {
    Pass(String),
    Fail(String),
}

/// Validate that a response is not empty
///
/// # Example
///
/// ```rust
/// let response = "Hello!";
/// common::validate_response_not_empty(response, "agent response");
/// ```
pub fn validate_response_not_empty(response: &str, context: &str) {
    if response.trim().is_empty() {
        validation_failed(&format!("No {} received", context));
    }
}

/// Validate that a response contains expected text
///
/// # Example
///
/// ```rust
/// let response = "The answer is 445";
/// common::validate_response_contains(response, "445", "calculation result");
/// ```
pub fn validate_response_contains(response: &str, expected: &str, context: &str) {
    if !response.contains(expected) {
        validation_failed(&format!(
            "{} doesn't contain expected '{}'\n   Got: '{}'",
            context,
            expected,
            response.trim()
        ));
    }
}

/// Validate response has minimum length
///
/// # Example
///
/// ```rust
/// let response = "Short explanation about ZDK";
/// common::validate_response_min_length(response, 20, "explanation");
/// ```
pub fn validate_response_min_length(response: &str, min_len: usize, context: &str) {
    if response.len() < min_len {
        validation_failed(&format!(
            "{} too short (expected at least {} chars)\n   Got: '{}'",
            context,
            min_len,
            response.trim()
        ));
    }
}

/// Print validation success message
///
/// # Example
///
/// ```rust
/// common::validation_passed("All checks successful");
/// ```
pub fn validation_passed(message: &str) {
    println!("\n✅ VALIDATION PASSED: {}", message);
}

/// Print validation failure message and exit with code 1
///
/// # Example
///
/// ```rust
/// if some_check_failed {
///     common::validation_failed("Tool was not called");
/// }
/// ```
pub fn validation_failed(message: &str) -> ! {
    eprintln!("❌ VALIDATION FAILED: {}", message);
    std::process::exit(1);
}

/// Collect text response from event stream with error handling
///
/// Returns the collected text response, handling errors appropriately.
///
/// # Example
///
/// ```rust
/// use futures::StreamExt;
/// 
/// let mut stream = runner.run(...).await?;
/// let response = common::collect_text_response(&mut stream, "example execution").await?;
/// common::validate_response(&response, "agent response");
/// ```
pub async fn collect_text_response<S>(
    stream: &mut S,
    context: &str,
) -> Result<String>
where
    S: futures::Stream<Item = Result<zdk_core::Event, zdk_core::Error>> + Unpin,
{
    use futures::StreamExt;
    use zdk_core::Part;
    
    let mut response = String::new();
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let Part::Text { text } = part {
                            response.push_str(text);
                        }
                    }
                }
            }
            Err(e) => {
                validation_failed(&format!("Error during {}: {}", context, e));
            }
        }
    }
    
    Ok(response)
}

/// Collect and print text response from event stream
///
/// Similar to `collect_text_response` but also prints the text as it arrives.
///
/// # Example
///
/// ```rust
/// let mut stream = runner.run(...).await?;
/// let response = common::collect_and_print_response(&mut stream, "example").await?;
/// ```
pub async fn collect_and_print_response<S>(
    stream: &mut S,
    context: &str,
) -> Result<String>
where
    S: futures::Stream<Item = Result<zdk_core::Event, zdk_core::Error>> + Unpin,
{
    use futures::StreamExt;
    use zdk_core::Part;
    
    let mut response = String::new();
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                if let Some(content) = &event.content {
                    for part in &content.parts {
                        if let Part::Text { text } = part {
                            print!("{}", text);
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                            response.push_str(text);
                        }
                    }
                }
                if event.is_final_response() {
                    println!();
                }
            }
            Err(e) => {
                validation_failed(&format!("Error during {}: {}", context, e));
            }
        }
    }
    
    Ok(response)
}

/// Check if a tool with given name was called in the events
///
/// Returns true if the tool was executed.
///
/// # Example
///
/// ```rust
/// if !tool_was_called(&events, "calculator") {
///     common::validation_failed("Calculator tool was not called");
/// }
/// ```
pub fn tool_was_called(events: &[zdk_core::Event], tool_name: &str) -> bool {
    use zdk_core::Part;
    
    events.iter().any(|event| {
        if let Some(content) = &event.content {
            content.parts.iter().any(|part| {
                matches!(part, Part::FunctionCall { function_call } if function_call.name == tool_name)
            })
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_functions_exist() {
        // Just verify the functions compile and have correct signatures
        // Actual functionality tested in integration tests
    }
}
