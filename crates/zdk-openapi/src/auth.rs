//! Simple authentication support for OpenAPI tools.
//!
//! This module provides basic authentication methods:
//! - API Key (in header or query parameter)
//! - Bearer Token (Authorization: Bearer <token>)
//! - Basic Auth (Authorization: Basic <base64>)
//!
//! OAuth2 and advanced authentication features are planned for Phase 8.7.

use serde::{Deserialize, Serialize};

/// Authentication configuration for API requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    /// No authentication
    None,

    /// API Key authentication
    ApiKey {
        /// Location of the API key
        location: AuthLocation,
        /// Name of the header or query parameter
        name: String,
        /// The API key value
        key: String,
    },

    /// Bearer token authentication (Authorization: Bearer <token>)
    Bearer {
        /// The bearer token
        token: String,
    },

    /// HTTP Basic authentication (Authorization: Basic <base64>)
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
}

/// Location where authentication credentials are provided.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthLocation {
    /// In HTTP header
    Header,
    /// In URL query parameter
    Query,
}

impl AuthConfig {
    /// Create API key authentication in header.
    ///
    /// # Example
    ///
    /// ```
    /// use zdk_openapi::AuthConfig;
    ///
    /// let auth = AuthConfig::api_key_header("X-API-Key", "my-secret-key");
    /// ```
    pub fn api_key_header(header_name: impl Into<String>, key: impl Into<String>) -> Self {
        Self::ApiKey {
            location: AuthLocation::Header,
            name: header_name.into(),
            key: key.into(),
        }
    }

    /// Create API key authentication in query parameter.
    ///
    /// # Example
    ///
    /// ```
    /// use zdk_openapi::AuthConfig;
    ///
    /// let auth = AuthConfig::api_key_query("api_key", "my-secret-key");
    /// ```
    pub fn api_key_query(param_name: impl Into<String>, key: impl Into<String>) -> Self {
        Self::ApiKey {
            location: AuthLocation::Query,
            name: param_name.into(),
            key: key.into(),
        }
    }

    /// Create bearer token authentication.
    ///
    /// # Example
    ///
    /// ```
    /// use zdk_openapi::AuthConfig;
    ///
    /// let auth = AuthConfig::bearer("my-bearer-token");
    /// ```
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer {
            token: token.into(),
        }
    }

    /// Create basic authentication.
    ///
    /// # Example
    ///
    /// ```
    /// use zdk_openapi::AuthConfig;
    ///
    /// let auth = AuthConfig::basic("username", "password");
    /// ```
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Apply authentication to a reqwest RequestBuilder.
    pub(crate) fn apply_to_request(
        &self,
        mut builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        match self {
            AuthConfig::None => builder,
            AuthConfig::ApiKey {
                location,
                name,
                key,
            } => match location {
                AuthLocation::Header => builder.header(name, key),
                AuthLocation::Query => builder.query(&[(name, key)]),
            },
            AuthConfig::Bearer { token } => builder.bearer_auth(token),
            AuthConfig::Basic { username, password } => {
                builder.basic_auth(username, Some(password))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_constructors() {
        // API Key header
        let auth = AuthConfig::api_key_header("X-API-Key", "secret");
        assert!(matches!(
            auth,
            AuthConfig::ApiKey {
                location: AuthLocation::Header,
                ..
            }
        ));

        // API Key query
        let auth = AuthConfig::api_key_query("api_key", "secret");
        assert!(matches!(
            auth,
            AuthConfig::ApiKey {
                location: AuthLocation::Query,
                ..
            }
        ));

        // Bearer
        let auth = AuthConfig::bearer("token");
        assert!(matches!(auth, AuthConfig::Bearer { .. }));

        // Basic
        let auth = AuthConfig::basic("user", "pass");
        assert!(matches!(auth, AuthConfig::Basic { .. }));
    }
}
