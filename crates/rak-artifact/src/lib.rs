//! # RAK Artifact Service
//!
//! This crate provides artifact storage and retrieval services for RAK.
//! Artifacts are files identified by application name, user ID, session ID, and filename,
//! with support for versioning.
//!
//! ## Features
//!
//! - **Multiple Storage Backends**: In-memory, file system, and cloud storage
//! - **Versioning**: Automatic version tracking for all artifacts
//! - **User Namespacing**: Special "user:" prefix for user-scoped artifacts
//! - **Async/Await**: Fully asynchronous API using tokio

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod filesystem;
mod memory;
mod service;

pub use filesystem::FileSystemArtifactService;
pub use memory::InMemoryArtifactService;
pub use service::*;

/// Errors that can occur during artifact operations
#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid artifact part: {0}")]
    InvalidPart(String),

    #[error("Artifact not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for artifact operations
pub type Result<T> = std::result::Result<T, ArtifactError>;

/// Represents an artifact part (either text or binary data)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArtifactPart {
    /// Text content
    Text(String),
    /// Binary data with MIME type
    Binary {
        mime_type: String,
        #[serde(with = "base64_serde")]
        data: Vec<u8>,
    },
}

mod base64_serde {
    use base64::{engine::general_purpose, Engine as _};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&general_purpose::STANDARD.encode(data))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        general_purpose::STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)
    }
}

impl ArtifactPart {
    /// Create a text artifact part
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Create a binary artifact part
    pub fn binary(mime_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self::Binary {
            mime_type: mime_type.into(),
            data,
        }
    }

    /// Check if this part is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Text(s) => s.is_empty(),
            Self::Binary { data, .. } => data.is_empty(),
        }
    }
}

/// Request to save an artifact
#[derive(Debug, Clone)]
pub struct SaveRequest {
    pub app_name: String,
    pub user_id: String,
    pub session_id: String,
    pub file_name: String,
    pub part: ArtifactPart,
    /// Optional: specific version to save (if unset, creates new version)
    pub version: Option<i64>,
}

impl SaveRequest {
    /// Validate the save request
    pub fn validate(&self) -> Result<()> {
        let mut missing = Vec::new();

        if self.app_name.is_empty() {
            missing.push("app_name");
        }
        if self.user_id.is_empty() {
            missing.push("user_id");
        }
        if self.session_id.is_empty() {
            missing.push("session_id");
        }
        if self.file_name.is_empty() {
            missing.push("file_name");
        }

        if !missing.is_empty() {
            return Err(ArtifactError::MissingField(missing.join(", ")));
        }

        if self.part.is_empty() {
            return Err(ArtifactError::InvalidPart(
                "Part must contain either text or binary data".into(),
            ));
        }

        Ok(())
    }
}

/// Response from saving an artifact
#[derive(Debug, Clone)]
pub struct SaveResponse {
    pub version: i64,
}

/// Request to load an artifact
#[derive(Debug, Clone)]
pub struct LoadRequest {
    pub app_name: String,
    pub user_id: String,
    pub session_id: String,
    pub file_name: String,
    /// Optional: specific version to load (if unset, loads latest)
    pub version: Option<i64>,
}

impl LoadRequest {
    /// Validate the load request
    pub fn validate(&self) -> Result<()> {
        let mut missing = Vec::new();

        if self.app_name.is_empty() {
            missing.push("app_name");
        }
        if self.user_id.is_empty() {
            missing.push("user_id");
        }
        if self.session_id.is_empty() {
            missing.push("session_id");
        }
        if self.file_name.is_empty() {
            missing.push("file_name");
        }

        if !missing.is_empty() {
            return Err(ArtifactError::MissingField(missing.join(", ")));
        }

        Ok(())
    }
}

/// Response from loading an artifact
#[derive(Debug, Clone)]
pub struct LoadResponse {
    pub part: ArtifactPart,
}

/// Request to delete an artifact
#[derive(Debug, Clone)]
pub struct DeleteRequest {
    pub app_name: String,
    pub user_id: String,
    pub session_id: String,
    pub file_name: String,
    /// Optional: specific version to delete (if unset, deletes all versions)
    pub version: Option<i64>,
}

impl DeleteRequest {
    /// Validate the delete request
    pub fn validate(&self) -> Result<()> {
        let mut missing = Vec::new();

        if self.app_name.is_empty() {
            missing.push("app_name");
        }
        if self.user_id.is_empty() {
            missing.push("user_id");
        }
        if self.session_id.is_empty() {
            missing.push("session_id");
        }
        if self.file_name.is_empty() {
            missing.push("file_name");
        }

        if !missing.is_empty() {
            return Err(ArtifactError::MissingField(missing.join(", ")));
        }

        Ok(())
    }
}

/// Request to list artifacts in a session
#[derive(Debug, Clone)]
pub struct ListRequest {
    pub app_name: String,
    pub user_id: String,
    pub session_id: String,
}

impl ListRequest {
    /// Validate the list request
    pub fn validate(&self) -> Result<()> {
        let mut missing = Vec::new();

        if self.app_name.is_empty() {
            missing.push("app_name");
        }
        if self.user_id.is_empty() {
            missing.push("user_id");
        }
        if self.session_id.is_empty() {
            missing.push("session_id");
        }

        if !missing.is_empty() {
            return Err(ArtifactError::MissingField(missing.join(", ")));
        }

        Ok(())
    }
}

/// Response from listing artifacts
#[derive(Debug, Clone)]
pub struct ListResponse {
    pub file_names: Vec<String>,
}

/// Request to list versions of an artifact
#[derive(Debug, Clone)]
pub struct VersionsRequest {
    pub app_name: String,
    pub user_id: String,
    pub session_id: String,
    pub file_name: String,
}

impl VersionsRequest {
    /// Validate the versions request
    pub fn validate(&self) -> Result<()> {
        let mut missing = Vec::new();

        if self.app_name.is_empty() {
            missing.push("app_name");
        }
        if self.user_id.is_empty() {
            missing.push("user_id");
        }
        if self.session_id.is_empty() {
            missing.push("session_id");
        }
        if self.file_name.is_empty() {
            missing.push("file_name");
        }

        if !missing.is_empty() {
            return Err(ArtifactError::MissingField(missing.join(", ")));
        }

        Ok(())
    }
}

/// Response from listing versions
#[derive(Debug, Clone)]
pub struct VersionsResponse {
    pub versions: Vec<i64>,
}
