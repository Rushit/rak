//! Gemini authentication strategies

use reqwest::RequestBuilder;

/// Authentication method for Gemini API
#[derive(Clone, Debug)]
pub enum GeminiAuth {
    /// API Key authentication (for generativelanguage.googleapis.com)
    ApiKey(String),
    /// Bearer token authentication (for Vertex AI via gcloud)
    BearerToken(String),
}

impl GeminiAuth {
    /// Apply authentication to a request builder
    ///
    /// # Arguments
    /// * `builder` - Request builder to add authentication to
    ///
    /// # Returns
    /// Modified request builder with authentication applied
    pub fn apply(&self, builder: RequestBuilder) -> RequestBuilder {
        match self {
            GeminiAuth::ApiKey(key) => builder.query(&[("key", key.as_str())]),
            GeminiAuth::BearerToken(token) => builder.bearer_auth(token),
        }
    }
}
