//! File system artifact service implementation

use crate::*;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// File system artifact service implementation.
///
/// Stores artifacts as files on the local file system.
/// Directory structure: `base_path/app_name/user_id/session_id/file_name/version`
pub struct FileSystemArtifactService {
    base_path: PathBuf,
}

impl FileSystemArtifactService {
    /// Create a new file system artifact service
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Get the directory path for an artifact
    fn get_artifact_dir(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
        file_name: &str,
    ) -> PathBuf {
        let mut session_id = session_id.to_string();

        // Handle user-namespaced artifacts
        if file_has_user_namespace(file_name) {
            session_id = USER_SCOPED_ARTIFACT_KEY.to_string();
        }

        // Sanitize file name for file system
        let safe_file_name = file_name.replace(['/', '\\', ':'], "_");

        self.base_path
            .join(app_name)
            .join(user_id)
            .join(session_id)
            .join(safe_file_name)
    }

    /// Get the file path for a specific version of an artifact
    fn get_artifact_file(&self, base_dir: &Path, version: i64) -> PathBuf {
        base_dir.join(format!("{}.json", version))
    }

    /// Get all versions for an artifact
    async fn list_versions(&self, artifact_dir: &Path) -> Result<Vec<i64>> {
        if !artifact_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        let mut entries = fs::read_dir(artifact_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && let Ok(version) = stem.parse::<i64>()
            {
                versions.push(version);
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Find the latest version
    async fn find_latest_version(&self, artifact_dir: &Path) -> Result<Option<i64>> {
        let versions = self.list_versions(artifact_dir).await?;
        Ok(versions.into_iter().max())
    }
}

#[async_trait]
impl ArtifactService for FileSystemArtifactService {
    async fn save(&self, req: SaveRequest) -> Result<SaveResponse> {
        req.validate()?;

        let artifact_dir =
            self.get_artifact_dir(&req.app_name, &req.user_id, &req.session_id, &req.file_name);

        // Ensure directory exists
        fs::create_dir_all(&artifact_dir).await?;

        let next_version = if let Some(version) = req.version {
            version
        } else {
            // Find the current latest version and increment
            self.find_latest_version(&artifact_dir)
                .await?
                .map(|v| v + 1)
                .unwrap_or(1)
        };

        let file_path = self.get_artifact_file(&artifact_dir, next_version);

        // Serialize and write the artifact
        let json = serde_json::to_string(&req.part)?;
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(json.as_bytes()).await?;

        Ok(SaveResponse {
            version: next_version,
        })
    }

    async fn load(&self, req: LoadRequest) -> Result<LoadResponse> {
        req.validate()?;

        let artifact_dir =
            self.get_artifact_dir(&req.app_name, &req.user_id, &req.session_id, &req.file_name);

        let version = if let Some(v) = req.version {
            v
        } else {
            // Load latest version
            self.find_latest_version(&artifact_dir)
                .await?
                .ok_or_else(|| {
                    ArtifactError::NotFound(format!(
                        "Artifact not found: {}/{}/{}/{}",
                        req.app_name, req.user_id, req.session_id, req.file_name
                    ))
                })?
        };

        let file_path = self.get_artifact_file(&artifact_dir, version);

        if !file_path.exists() {
            return Err(ArtifactError::NotFound(format!(
                "Artifact not found: {}/{}/{}/{} version {}",
                req.app_name, req.user_id, req.session_id, req.file_name, version
            )));
        }

        let mut file = fs::File::open(&file_path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let part: ArtifactPart = serde_json::from_str(&contents)?;

        Ok(LoadResponse { part })
    }

    async fn delete(&self, req: DeleteRequest) -> Result<()> {
        req.validate()?;

        let artifact_dir =
            self.get_artifact_dir(&req.app_name, &req.user_id, &req.session_id, &req.file_name);

        if !artifact_dir.exists() {
            // Deleting non-existent artifact is not an error
            return Ok(());
        }

        if let Some(version) = req.version {
            // Delete specific version
            let file_path = self.get_artifact_file(&artifact_dir, version);
            if file_path.exists() {
                fs::remove_file(&file_path).await?;
            }
        } else {
            // Delete all versions (entire directory)
            fs::remove_dir_all(&artifact_dir).await?;
        }

        Ok(())
    }

    async fn list(&self, req: ListRequest) -> Result<ListResponse> {
        req.validate()?;

        let session_dir = self
            .base_path
            .join(&req.app_name)
            .join(&req.user_id)
            .join(&req.session_id);

        let user_dir = self
            .base_path
            .join(&req.app_name)
            .join(&req.user_id)
            .join(USER_SCOPED_ARTIFACT_KEY);

        let mut file_names = std::collections::HashSet::new();

        // List session-specific artifacts
        if session_dir.exists() {
            let mut entries = fs::read_dir(&session_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir()
                    && let Some(file_name) = entry.file_name().to_str()
                {
                    // Reverse sanitization
                    file_names.insert(file_name.to_string());
                }
            }
        }

        // List user-scoped artifacts
        if user_dir.exists() {
            let mut entries = fs::read_dir(&user_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir()
                    && let Some(file_name) = entry.file_name().to_str()
                {
                    file_names.insert(file_name.to_string());
                }
            }
        }

        let mut file_names: Vec<_> = file_names.into_iter().collect();
        file_names.sort();

        Ok(ListResponse { file_names })
    }

    async fn versions(&self, req: VersionsRequest) -> Result<VersionsResponse> {
        req.validate()?;

        let artifact_dir =
            self.get_artifact_dir(&req.app_name, &req.user_id, &req.session_id, &req.file_name);

        let versions = self.list_versions(&artifact_dir).await?;

        Ok(VersionsResponse { versions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let service = FileSystemArtifactService::new(temp_dir.path());

        let save_req = SaveRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
            part: ArtifactPart::text("Hello, world!"),
            version: None,
        };

        let save_resp = service.save(save_req).await.unwrap();
        assert_eq!(save_resp.version, 1);

        let load_req = LoadRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
            version: None,
        };

        let load_resp = service.load(load_req).await.unwrap();
        match load_resp.part {
            ArtifactPart::Text(text) => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected text part"),
        }
    }

    #[tokio::test]
    async fn test_versioning() {
        let temp_dir = TempDir::new().unwrap();
        let service = FileSystemArtifactService::new(temp_dir.path());

        // Save version 1
        let save_req = SaveRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
            part: ArtifactPart::text("Version 1"),
            version: None,
        };
        service.save(save_req).await.unwrap();

        // Save version 2
        let save_req = SaveRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
            part: ArtifactPart::text("Version 2"),
            version: None,
        };
        service.save(save_req).await.unwrap();

        // List versions
        let versions_req = VersionsRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
        };

        let versions_resp = service.versions(versions_req).await.unwrap();
        assert_eq!(versions_resp.versions, vec![1, 2]);

        // Load latest (should be version 2)
        let load_req = LoadRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
            version: None,
        };

        let load_resp = service.load(load_req).await.unwrap();
        match load_resp.part {
            ArtifactPart::Text(text) => assert_eq!(text, "Version 2"),
            _ => panic!("Expected text part"),
        }
    }

    #[tokio::test]
    async fn test_binary_artifact() {
        let temp_dir = TempDir::new().unwrap();
        let service = FileSystemArtifactService::new(temp_dir.path());

        let binary_data = vec![1, 2, 3, 4, 5];
        let save_req = SaveRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "image.png".to_string(),
            part: ArtifactPart::binary("image/png", binary_data.clone()),
            version: None,
        };

        service.save(save_req).await.unwrap();

        let load_req = LoadRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "image.png".to_string(),
            version: None,
        };

        let load_resp = service.load(load_req).await.unwrap();
        match load_resp.part {
            ArtifactPart::Binary { mime_type, data } => {
                assert_eq!(mime_type, "image/png");
                assert_eq!(data, binary_data);
            }
            _ => panic!("Expected binary part"),
        }
    }

    #[tokio::test]
    async fn test_list_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let service = FileSystemArtifactService::new(temp_dir.path());

        // Save multiple artifacts
        for i in 1..=3 {
            let save_req = SaveRequest {
                app_name: "test_app".to_string(),
                user_id: "user1".to_string(),
                session_id: "session1".to_string(),
                file_name: format!("file{}.txt", i),
                part: ArtifactPart::text(format!("Content {}", i)),
                version: None,
            };
            service.save(save_req).await.unwrap();
        }

        // List artifacts
        let list_req = ListRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
        };

        let list_resp = service.list(list_req).await.unwrap();
        assert_eq!(
            list_resp.file_names,
            vec!["file1.txt", "file2.txt", "file3.txt"]
        );
    }

    #[tokio::test]
    async fn test_delete_artifact() {
        let temp_dir = TempDir::new().unwrap();
        let service = FileSystemArtifactService::new(temp_dir.path());

        // Save artifact
        let save_req = SaveRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
            part: ArtifactPart::text("To be deleted"),
            version: None,
        };
        service.save(save_req).await.unwrap();

        // Delete artifact
        let delete_req = DeleteRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
            version: None,
        };
        service.delete(delete_req).await.unwrap();

        // Try to load (should fail)
        let load_req = LoadRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
            version: None,
        };

        assert!(service.load(load_req).await.is_err());
    }
}
