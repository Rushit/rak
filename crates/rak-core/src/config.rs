//! Configuration management for RAK
//!
//! Loads configuration with priority:
//! 1. config.toml (or specified config file)
//! 2. Environment variables (fallback)
//! 3. Defaults

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// RAK configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RakConfig {
    #[serde(default)]
    pub model: ModelConfig,
    
    #[serde(default)]
    pub server: ServerConfig,
    
    #[serde(default)]
    pub session: SessionConfig,
    
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

/// Model/LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model provider (gemini, anthropic, openai, etc.)
    #[serde(default = "default_provider")]
    pub provider: String,
    
    /// API key (can reference env var with ${VAR_NAME})
    pub api_key: Option<String>,
    
    /// Model name
    #[serde(default = "default_model_name")]
    pub model_name: String,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    
    #[serde(default = "default_port")]
    pub port: u16,
}

/// Session storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_session_provider")]
    pub provider: String,
    
    pub connection_string: Option<String>,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub otel_endpoint: Option<String>,
    pub service_name: Option<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            api_key: None,
            model_name: default_model_name(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            provider: default_session_provider(),
            connection_string: None,
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            otel_endpoint: None,
            service_name: None,
        }
    }
}

impl RakConfig {
    /// Load configuration with the following priority:
    /// 1. Specified config file (if provided)
    /// 2. config.toml in current directory
    /// 3. Environment variables (fallback)
    /// 4. Defaults
    pub fn load() -> Result<Self> {
        Self::load_from(None)
    }

    /// Load configuration from a specific file
    pub fn load_from(path: Option<&Path>) -> Result<Self> {
        let config_path = if let Some(p) = path {
            p.to_path_buf()
        } else {
            // Try to find config.toml in current directory or parent directories
            Self::find_config_file()?
        };

        tracing::debug!("Loading configuration from: {:?}", config_path);

        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let mut config: RakConfig = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

        // Resolve environment variable references
        config.resolve_env_vars()?;

        Ok(config)
    }

    /// Load test configuration (config.test.toml)
    pub fn load_test() -> Result<Self> {
        let test_config = PathBuf::from("config.test.toml");
        if test_config.exists() {
            Self::load_from(Some(&test_config))
        } else {
            // Fallback to mock configuration for tests
            Ok(Self::test_defaults())
        }
    }

    /// Find config.toml by searching current directory and parents
    fn find_config_file() -> Result<PathBuf> {
        let mut current = env::current_dir()?;
        
        loop {
            let config_path = current.join("config.toml");
            if config_path.exists() {
                return Ok(config_path);
            }

            if !current.pop() {
                break;
            }
        }

        Err(anyhow!(
            "config.toml not found. Create one with: cp config.toml.example config.toml"
        ))
    }

    /// Resolve ${VAR_NAME} references to environment variables
    fn resolve_env_vars(&mut self) -> Result<()> {
        // Resolve model.api_key
        if let Some(ref key) = self.model.api_key {
            if let Some(resolved) = Self::resolve_env_var(key) {
                self.model.api_key = Some(resolved);
            } else if key.is_empty() || key == "${GEMINI_API_KEY}" {
                // If not resolved and it's a reference, try env var directly
                self.model.api_key = env::var("GEMINI_API_KEY").ok();
            }
        } else {
            // No api_key in config, try environment variable as fallback
            self.model.api_key = env::var("GEMINI_API_KEY").ok();
        }

        // Resolve session.connection_string
        if let Some(ref conn) = self.session.connection_string {
            if let Some(resolved) = Self::resolve_env_var(conn) {
                self.session.connection_string = Some(resolved);
            }
        }

        Ok(())
    }

    /// Resolve a single ${VAR_NAME} reference
    fn resolve_env_var(value: &str) -> Option<String> {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            env::var(var_name).ok()
        } else {
            Some(value.to_string())
        }
    }

    /// Get API key with clear error message
    pub fn api_key(&self) -> Result<String> {
        self.model.api_key.clone().ok_or_else(|| {
            anyhow!(
                "API key not found. Set it in config.toml:\n\
                [model]\n\
                api_key = \"your-key\"\n\
                \n\
                Or set environment variable:\n\
                export GEMINI_API_KEY=\"your-key\""
            )
        })
    }

    /// Create test-friendly defaults (no API key required)
    pub fn test_defaults() -> Self {
        Self {
            model: ModelConfig {
                provider: "test".to_string(),
                api_key: Some("test-api-key".to_string()),
                model_name: "test-model".to_string(),
            },
            server: ServerConfig::default(),
            session: SessionConfig::default(),
            observability: ObservabilityConfig::default(),
        }
    }
}

fn default_provider() -> String {
    "gemini".to_string()
}

fn default_model_name() -> String {
    "gemini-2.0-flash-exp".to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_session_provider() -> String {
    "in-memory".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RakConfig::test_defaults();
        assert_eq!(config.model.provider, "test");
        assert!(config.model.api_key.is_some());
    }

    #[test]
    fn test_resolve_env_var() {
        unsafe {
            env::set_var("TEST_VAR", "test_value");
        }
        
        let resolved = RakConfig::resolve_env_var("${TEST_VAR}");
        assert_eq!(resolved, Some("test_value".to_string()));
        
        let not_var = RakConfig::resolve_env_var("plain_value");
        assert_eq!(not_var, Some("plain_value".to_string()));
        
        unsafe {
            env::remove_var("TEST_VAR");
        }
    }

    #[test]
    fn test_api_key_error_message() {
        let config = RakConfig {
            model: ModelConfig {
                provider: "gemini".to_string(),
                api_key: None,
                model_name: "gemini-2.0-flash-exp".to_string(),
            },
            server: ServerConfig::default(),
            session: SessionConfig::default(),
            observability: ObservabilityConfig::default(),
        };

        let result = config.api_key();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("config.toml"));
        assert!(error_msg.contains("GEMINI_API_KEY"));
    }
}

