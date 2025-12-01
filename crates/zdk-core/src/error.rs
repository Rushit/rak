use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Tool '{tool}' execution failed: {source}")]
    ToolFailed {
        tool: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("LLM request failed: {0}")]
    LLMError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Artifact error: {0}")]
    ArtifactError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// Helper for creating configuration errors
    ///
    /// # Example
    /// ```
    /// use zdk_core::Error;
    /// let err = Error::config_error("Invalid model configuration");
    /// ```
    pub fn config_error(msg: impl Into<String>) -> Self {
        Error::Config(msg.into())
    }

    /// Helper for creating general errors with a message
    ///
    /// # Example
    /// ```
    /// use zdk_core::Error;
    /// let err = Error::message("Something went wrong");
    /// ```
    pub fn message(msg: impl Into<String>) -> Self {
        Error::Other(anyhow::anyhow!("{}", msg.into()))
    }

    /// Helper for creating authentication errors
    ///
    /// # Example
    /// ```
    /// use zdk_core::Error;
    /// let err = Error::auth_error("Invalid API key");
    /// ```
    pub fn auth_error(msg: impl Into<String>) -> Self {
        Error::Auth(msg.into())
    }
}
