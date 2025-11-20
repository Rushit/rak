//! Artifact service trait definition

use crate::*;
use async_trait::async_trait;

/// The artifact storage service trait.
///
/// An artifact is a file identified by an application name, a user ID, a session ID,
/// and a filename. The service provides basic storage operations for artifacts,
/// such as Save, Load, Delete, and List. It also supports versioning of artifacts.
#[async_trait]
pub trait ArtifactService: Send + Sync {
    /// Save an artifact to storage.
    ///
    /// After saving the artifact, a version ID is returned to identify the artifact version.
    async fn save(&self, req: SaveRequest) -> Result<SaveResponse>;

    /// Load an artifact from storage.
    async fn load(&self, req: LoadRequest) -> Result<LoadResponse>;

    /// Delete an artifact. Deleting a non-existing entry is not an error.
    async fn delete(&self, req: DeleteRequest) -> Result<()>;

    /// List all artifact filenames within a session.
    async fn list(&self, req: ListRequest) -> Result<ListResponse>;

    /// List all versions of an artifact.
    async fn versions(&self, req: VersionsRequest) -> Result<VersionsResponse>;
}

/// Check if a filename has a user namespace prefix
pub fn file_has_user_namespace(file_name: &str) -> bool {
    file_name.starts_with("user:")
}

/// Constant for user-scoped artifact key
pub const USER_SCOPED_ARTIFACT_KEY: &str = "user";
