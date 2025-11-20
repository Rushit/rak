use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

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
