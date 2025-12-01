//! Authentication abstraction for RAK
//!
//! Provides a unified interface for different authentication methods,
//! allowing users to configure their preferred auth provider in config.toml.

use crate::Error;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Authentication provider configuration
///
/// This enum represents the different authentication methods supported by RAK.
/// Users configure their preferred provider in config.toml.
///
/// # Examples
///
/// ## API Key Authentication
/// ```toml
/// [auth]
/// provider = "api_key"
///
/// [auth.api_key]
/// key = "${GOOGLE_API_KEY}"
/// ```
///
/// ## Google Cloud Authentication
/// ```toml
/// [auth]
/// provider = "gcloud"
///
/// [auth.gcloud]
/// project_id = "my-project"  # optional
/// location = "us-central1"   # optional
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum AuthProvider {
    /// API Key authentication (for public Gemini API)
    #[serde(rename = "api_key")]
    ApiKey {
        #[serde(flatten)]
        config: ApiKeyConfig,
    },

    /// Google Cloud authentication via gcloud CLI
    #[serde(rename = "gcloud")]
    GCloud {
        #[serde(flatten)]
        config: GCloudConfig,
    },
}

/// API Key authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// API key for authentication
    pub key: String,
}

/// Google Cloud authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCloudConfig {
    /// GCP project ID (optional - auto-detected from gcloud if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,

    /// GCP location/region (optional - defaults to us-central1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Custom Vertex AI endpoint (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

impl AuthProvider {
    /// Get authentication credentials from this provider
    ///
    /// This method resolves the authentication configuration into concrete credentials
    /// that can be used to authenticate with the LLM provider.
    ///
    /// # Returns
    ///
    /// Returns `AuthCredentials` containing the resolved authentication information.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - gcloud CLI is not installed or not authenticated
    /// - gcloud project cannot be determined
    /// - API key is invalid or empty
    pub fn get_credentials(&self) -> crate::Result<AuthCredentials> {
        match self {
            AuthProvider::ApiKey { config } => {
                if config.key.is_empty() {
                    return Err(Error::Config("API key is empty".into()));
                }
                Ok(AuthCredentials::ApiKey {
                    key: config.key.clone(),
                })
            }
            AuthProvider::GCloud { config } => {
                // Get access token from gcloud
                let token = get_gcloud_access_token()?;

                // Get project ID (from config or auto-detect)
                let project = if let Some(ref project) = config.project_id {
                    project.clone()
                } else {
                    get_gcloud_project()?
                };

                // Get location (from config or default)
                let location = config
                    .location
                    .clone()
                    .unwrap_or_else(|| "us-central1".to_string());

                Ok(AuthCredentials::GCloud {
                    token,
                    project,
                    location,
                    endpoint: config.endpoint.clone(),
                })
            }
        }
    }

    /// Returns the provider type as a string for display purposes
    pub fn provider_name(&self) -> &str {
        match self {
            AuthProvider::ApiKey { .. } => "API Key",
            AuthProvider::GCloud { .. } => "Google Cloud (gcloud)",
        }
    }
}

/// Resolved authentication credentials
///
/// This enum represents the actual credentials obtained from an `AuthProvider`.
/// Unlike `AuthProvider` which represents configuration, this contains the
/// actual tokens and keys needed for authentication.
#[derive(Debug, Clone)]
pub enum AuthCredentials {
    /// API Key credentials
    ApiKey {
        /// The API key
        key: String,
    },

    /// Google Cloud credentials
    GCloud {
        /// OAuth access token from gcloud
        token: String,
        /// GCP project ID
        project: String,
        /// GCP location/region
        location: String,
        /// Optional custom endpoint
        endpoint: Option<String>,
    },
}

impl AuthCredentials {
    /// Returns true if these credentials are for Google Cloud
    pub fn is_gcloud(&self) -> bool {
        matches!(self, AuthCredentials::GCloud { .. })
    }

    /// Returns true if these credentials are for API Key
    pub fn is_api_key(&self) -> bool {
        matches!(self, AuthCredentials::ApiKey { .. })
    }
}

/// Get access token from gcloud CLI
///
/// Executes `gcloud auth print-access-token` to retrieve the current user's
/// OAuth access token. Requires gcloud CLI to be installed and authenticated.
///
/// # Errors
///
/// Returns an error if:
/// - gcloud CLI is not installed
/// - User is not authenticated (needs `gcloud auth login`)
/// - Token is empty or invalid
fn get_gcloud_access_token() -> crate::Result<String> {
    let output = Command::new("gcloud")
        .args(["auth", "print-access-token"])
        .output()
        .map_err(|e| {
            Error::Auth(format!(
                "Failed to run gcloud command. Is gcloud CLI installed? Error: {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Auth(format!(
            "gcloud auth failed: {}. Run 'gcloud auth login' to authenticate",
            stderr
        )));
    }

    let token = String::from_utf8(output.stdout)
        .map_err(|e| Error::Auth(format!("Invalid gcloud output: {}", e)))?
        .trim()
        .to_string();

    if token.is_empty() {
        return Err(Error::Auth(
            "Empty token from gcloud. Run 'gcloud auth login' to authenticate".into(),
        ));
    }

    Ok(token)
}

/// Get default project ID from gcloud config
///
/// Executes `gcloud config get-value project` to retrieve the user's
/// configured default project.
///
/// # Errors
///
/// Returns an error if:
/// - gcloud CLI is not installed
/// - No default project is configured
/// - Project value is "(unset)"
fn get_gcloud_project() -> crate::Result<String> {
    let output = Command::new("gcloud")
        .args(["config", "get-value", "project"])
        .output()
        .map_err(|e| Error::Auth(format!("Failed to get gcloud project: {}", e)))?;

    if !output.status.success() {
        return Err(Error::Auth(
            "Failed to get gcloud project. Run 'gcloud config set project PROJECT_ID'".into(),
        ));
    }

    let project = String::from_utf8(output.stdout)
        .map_err(|e| Error::Auth(format!("Invalid project output: {}", e)))?
        .trim()
        .to_string();

    if project.is_empty() || project == "(unset)" {
        return Err(Error::Auth(
            "No gcloud project set. Run 'gcloud config set project PROJECT_ID'".into(),
        ));
    }

    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_auth_creation() {
        let auth = AuthProvider::ApiKey {
            config: ApiKeyConfig {
                key: "test-key".to_string(),
            },
        };

        assert_eq!(auth.provider_name(), "API Key");
    }

    #[test]
    fn test_gcloud_auth_creation() {
        let auth = AuthProvider::GCloud {
            config: GCloudConfig {
                project_id: Some("test-project".to_string()),
                location: Some("us-west1".to_string()),
                endpoint: None,
            },
        };

        assert_eq!(auth.provider_name(), "Google Cloud (gcloud)");
    }

    #[test]
    fn test_api_key_credentials() {
        let auth = AuthProvider::ApiKey {
            config: ApiKeyConfig {
                key: "test-key".to_string(),
            },
        };

        let creds = auth.get_credentials().unwrap();
        assert!(creds.is_api_key());
        assert!(!creds.is_gcloud());
    }

    #[test]
    fn test_empty_api_key_fails() {
        let auth = AuthProvider::ApiKey {
            config: ApiKeyConfig {
                key: String::new(),
            },
        };

        assert!(auth.get_credentials().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let auth = AuthProvider::ApiKey {
            config: ApiKeyConfig {
                key: "test-key".to_string(),
            },
        };

        let json = serde_json::to_string(&auth).unwrap();
        let deserialized: AuthProvider = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.provider_name(), "API Key");
    }

    #[test]
    #[ignore] // Only run when gcloud is available
    fn test_gcloud_auth_e2e() {
        let auth = AuthProvider::GCloud {
            config: GCloudConfig {
                project_id: None,
                location: None,
                endpoint: None,
            },
        };

        // This will fail if gcloud is not installed/configured
        // That's expected - it's an integration test
        let result = auth.get_credentials();

        // Just verify it returns something (success or expected error)
        match result {
            Ok(creds) => assert!(creds.is_gcloud()),
            Err(e) => {
                // Should be a clear auth error message
                assert!(e.to_string().contains("gcloud"));
            }
        }
    }
}

