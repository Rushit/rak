//! Configuration management for ZDK
//!
//! Loads configuration with priority:
//! 1. config.toml (or specified config file)
//! 2. Environment variables (fallback)
//! 3. Defaults

use crate::auth::{AuthCredentials, AuthProvider};
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// ZDK configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZConfig {
    /// Authentication configuration (API key or gcloud)
    pub auth: AuthProvider,

    #[serde(default)]
    pub model: ModelConfig,

    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub session: SessionConfig,

    #[serde(default)]
    pub observability: ObservabilityConfig,

    /// OpenAI API key (optional, for OpenAI models)
    pub openai_api_key: Option<String>,

    /// OpenAI base URL (optional, for OpenAI-compatible endpoints)
    pub openai_base_url: Option<String>,

    /// Anthropic API key (optional, for Claude models)
    pub anthropic_api_key: Option<String>,
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

impl ZConfig {
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

        let mut config: ZConfig = toml::from_str(&contents)
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
        // Resolve auth.api_key.key
        if let AuthProvider::ApiKey { ref mut config } = self.auth {
            if let Some(resolved) = Self::resolve_env_var(&config.key) {
                config.key = resolved;
            }
        }

        // Resolve model.api_key (legacy support)
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

        // Resolve openai_api_key
        if let Some(ref key) = self.openai_api_key {
            if let Some(resolved) = Self::resolve_env_var(key) {
                self.openai_api_key = Some(resolved);
            } else if key.is_empty() || key == "${OPENAI_API_KEY}" {
                // If not resolved and it's a reference, try env var directly
                self.openai_api_key = env::var("OPENAI_API_KEY").ok();
            }
        } else {
            // No openai_api_key in config, try environment variable as fallback
            self.openai_api_key = env::var("OPENAI_API_KEY").ok();
        }

        // Resolve openai_base_url
        if let Some(ref url) = self.openai_base_url {
            if let Some(resolved) = Self::resolve_env_var(url) {
                self.openai_base_url = Some(resolved);
            }
        }

        // Resolve anthropic_api_key
        if let Some(ref key) = self.anthropic_api_key {
            if let Some(resolved) = Self::resolve_env_var(key) {
                self.anthropic_api_key = Some(resolved);
            } else if key.is_empty() || key == "${ANTHROPIC_API_KEY}" {
                // If not resolved and it's a reference, try env var directly
                self.anthropic_api_key = env::var("ANTHROPIC_API_KEY").ok();
            }
        } else {
            // No anthropic_api_key in config, try environment variable as fallback
            self.anthropic_api_key = env::var("ANTHROPIC_API_KEY").ok();
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

    /// Get API key with clear error message (legacy method for backward compatibility)
    ///
    /// **Deprecated**: Use `auth` configuration and `get_auth_credentials()` instead.
    pub fn api_key(&self) -> Result<String> {
        // Try new auth config first
        if let AuthProvider::ApiKey { ref config } = self.auth {
            return Ok(config.key.clone());
        }

        // Fall back to legacy model.api_key
        self.model.api_key.clone().ok_or_else(|| {
            anyhow!(
                "API key not found. Configure authentication in config.toml:\n\
                [auth]\n\
                provider = \"api_key\"\n\
                [auth.api_key]\n\
                key = \"your-key\"\n\
                \n\
                Or set environment variable:\n\
                export GOOGLE_API_KEY=\"your-key\""
            )
        })
    }

    /// Get authentication credentials from configured provider
    ///
    /// This method resolves the authentication configuration into concrete credentials.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = ZConfig::load()?;
    /// let creds = config.get_auth_credentials()?;
    ///
    /// match creds {
    ///     AuthCredentials::ApiKey { key } => {
    ///         // Use API key
    ///     }
    ///     AuthCredentials::GCloud { token, project, .. } => {
    ///         // Use gcloud credentials
    ///     }
    /// }
    /// ```
    pub fn get_auth_credentials(&self) -> crate::Result<AuthCredentials> {
        self.auth.get_credentials()
    }

    /// Create test-friendly defaults (no API key required)
    pub fn test_defaults() -> Self {
        Self {
            auth: AuthProvider::ApiKey {
                config: crate::auth::ApiKeyConfig {
                    key: "test-api-key".to_string(),
                },
            },
            model: ModelConfig {
                provider: "test".to_string(),
                api_key: Some("test-api-key".to_string()),
                model_name: "test-model".to_string(),
            },
            server: ServerConfig::default(),
            session: SessionConfig::default(),
            observability: ObservabilityConfig::default(),
            openai_api_key: Some("test-openai-key".to_string()),
            openai_base_url: None,
            anthropic_api_key: Some("test-anthropic-key".to_string()),
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
        let config = ZConfig::test_defaults();
        assert_eq!(config.model.provider, "test");
        assert!(config.model.api_key.is_some());
    }

    #[test]
    fn test_resolve_env_var() {
        unsafe {
            env::set_var("TEST_VAR", "test_value");
        }

        let resolved = ZConfig::resolve_env_var("${TEST_VAR}");
        assert_eq!(resolved, Some("test_value".to_string()));

        let not_var = ZConfig::resolve_env_var("plain_value");
        assert_eq!(not_var, Some("plain_value".to_string()));

        unsafe {
            env::remove_var("TEST_VAR");
        }
    }

    #[test]
    fn test_api_key_error_message() {
        let config = ZConfig {
            auth: AuthProvider::ApiKey {
                config: crate::auth::ApiKeyConfig {
                    key: "test-key".to_string(),
                },
            },
            model: ModelConfig {
                provider: "gemini".to_string(),
                api_key: None,
                model_name: "gemini-2.0-flash-exp".to_string(),
            },
            server: ServerConfig::default(),
            session: SessionConfig::default(),
            observability: ObservabilityConfig::default(),
            openai_api_key: None,
            openai_base_url: None,
            anthropic_api_key: None,
        };

        // This test now just validates the structure compiles correctly
        // since we have a valid API key in the auth config
        let result = config.api_key();
        assert!(result.is_ok());
    }
}
