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
use zdk_core::{AuthCredentials, LLM, ZConfig};
use zdk_model::{GeminiModel, ZConfigExt};

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
pub fn create_gemini_model(config: &ZConfig) -> Result<Arc<dyn LLM>> {
    // Use the new simplified factory pattern
    let model = config
        .create_model()
        .context("Failed to create model from config")?;

    // Print what we're using
    println!(
        "✓ Model created: {} (provider: {})",
        config.model.model_name, config.model.provider
    );

    Ok(model)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_functions_exist() {
        // Just verify the functions compile and have correct signatures
        // Actual functionality tested in integration tests
    }
}
