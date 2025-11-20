//! In-memory artifact service implementation

use crate::*;
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

/// Artifact key for unique identification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ArtifactKey {
    app_name: String,
    user_id: String,
    session_id: String,
    file_name: String,
    version: i64,
}

impl ArtifactKey {
    fn new(
        app_name: String,
        user_id: String,
        session_id: String,
        file_name: String,
        version: i64,
    ) -> Self {
        Self {
            app_name,
            user_id,
            session_id,
            file_name,
            version,
        }
    }

    /// Create a key with version set to max (for range queries)
    fn with_max_version(
        app_name: String,
        user_id: String,
        session_id: String,
        file_name: String,
    ) -> Self {
        Self::new(app_name, user_id, session_id, file_name, i64::MAX)
    }

    /// Create a key with version set to 0 (for range queries)
    fn with_min_version(
        app_name: String,
        user_id: String,
        session_id: String,
        file_name: String,
    ) -> Self {
        Self::new(app_name, user_id, session_id, file_name, 0)
    }
}

/// In-memory artifact service implementation.
///
/// This is primarily for testing and demonstration purposes.
/// Data is stored in memory and is not persisted across restarts.
#[derive(Clone)]
pub struct InMemoryArtifactService {
    artifacts: Arc<RwLock<BTreeMap<ArtifactKey, ArtifactPart>>>,
}

impl InMemoryArtifactService {
    /// Create a new in-memory artifact service
    pub fn new() -> Self {
        Self {
            artifacts: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Find the latest version of an artifact
    fn find_latest_version(
        &self,
        app_name: &str,
        user_id: &str,
        session_id: &str,
        file_name: &str,
    ) -> Option<(i64, ArtifactPart)> {
        let artifacts = self.artifacts.read().unwrap();

        let max_key = ArtifactKey::with_max_version(
            app_name.to_string(),
            user_id.to_string(),
            session_id.to_string(),
            file_name.to_string(),
        );

        let min_key = ArtifactKey::with_min_version(
            app_name.to_string(),
            user_id.to_string(),
            session_id.to_string(),
            file_name.to_string(),
        );

        // Find the highest version (closest to max_key, but less than it)
        artifacts
            .range(min_key..max_key)
            .next_back()
            .map(|(k, v)| (k.version, v.clone()))
    }
}

impl Default for InMemoryArtifactService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ArtifactService for InMemoryArtifactService {
    async fn save(&self, req: SaveRequest) -> Result<SaveResponse> {
        req.validate()?;

        let mut session_id = req.session_id.clone();

        // Handle user-namespaced artifacts
        if file_has_user_namespace(&req.file_name) {
            session_id = USER_SCOPED_ARTIFACT_KEY.to_string();
        }

        let next_version = if let Some(version) = req.version {
            version
        } else {
            // Find the current latest version and increment
            self.find_latest_version(&req.app_name, &req.user_id, &session_id, &req.file_name)
                .map(|(v, _)| v + 1)
                .unwrap_or(1)
        };

        let key = ArtifactKey::new(
            req.app_name,
            req.user_id,
            session_id,
            req.file_name,
            next_version,
        );

        let mut artifacts = self.artifacts.write().unwrap();
        artifacts.insert(key, req.part);

        Ok(SaveResponse {
            version: next_version,
        })
    }

    async fn load(&self, req: LoadRequest) -> Result<LoadResponse> {
        req.validate()?;

        let mut session_id = req.session_id.clone();

        // Handle user-namespaced artifacts
        if file_has_user_namespace(&req.file_name) {
            session_id = USER_SCOPED_ARTIFACT_KEY.to_string();
        }

        if let Some(version) = req.version {
            // Load specific version
            let app_name = req.app_name.clone();
            let user_id = req.user_id.clone();
            let orig_session_id = req.session_id.clone();
            let file_name = req.file_name.clone();

            let key = ArtifactKey::new(
                req.app_name,
                req.user_id,
                session_id,
                req.file_name,
                version,
            );

            let artifacts = self.artifacts.read().unwrap();
            artifacts
                .get(&key)
                .cloned()
                .map(|part| LoadResponse { part })
                .ok_or_else(|| {
                    ArtifactError::NotFound(format!(
                        "Artifact not found: {}/{}/{}/{} version {}",
                        app_name, user_id, orig_session_id, file_name, version
                    ))
                })
        } else {
            // Load latest version
            let app_name = req.app_name.clone();
            let user_id = req.user_id.clone();
            let orig_session_id = req.session_id.clone();
            let file_name = req.file_name.clone();

            self.find_latest_version(&req.app_name, &req.user_id, &session_id, &req.file_name)
                .map(|(_, part)| LoadResponse { part })
                .ok_or_else(|| {
                    ArtifactError::NotFound(format!(
                        "Artifact not found: {}/{}/{}/{}",
                        app_name, user_id, orig_session_id, file_name
                    ))
                })
        }
    }

    async fn delete(&self, req: DeleteRequest) -> Result<()> {
        req.validate()?;

        let mut session_id = req.session_id.clone();

        // Handle user-namespaced artifacts
        if file_has_user_namespace(&req.file_name) {
            session_id = USER_SCOPED_ARTIFACT_KEY.to_string();
        }

        let mut artifacts = self.artifacts.write().unwrap();

        if let Some(version) = req.version {
            // Delete specific version
            let key = ArtifactKey::new(
                req.app_name,
                req.user_id,
                session_id,
                req.file_name,
                version,
            );
            artifacts.remove(&key);
        } else {
            // Delete all versions
            let max_key = ArtifactKey::with_max_version(
                req.app_name.clone(),
                req.user_id.clone(),
                session_id.clone(),
                req.file_name.clone(),
            );

            let min_key =
                ArtifactKey::with_min_version(req.app_name, req.user_id, session_id, req.file_name);

            let keys_to_remove: Vec<_> = artifacts
                .range(min_key..max_key)
                .map(|(k, _)| k.clone())
                .collect();

            for key in keys_to_remove {
                artifacts.remove(&key);
            }
        }

        Ok(())
    }

    async fn list(&self, req: ListRequest) -> Result<ListResponse> {
        req.validate()?;

        let artifacts = self.artifacts.read().unwrap();

        let mut file_names = std::collections::HashSet::new();

        for key in artifacts.keys() {
            if key.app_name == req.app_name
                && key.user_id == req.user_id
                && (key.session_id == req.session_id || key.session_id == USER_SCOPED_ARTIFACT_KEY)
            {
                file_names.insert(key.file_name.clone());
            }
        }

        let mut file_names: Vec<_> = file_names.into_iter().collect();
        file_names.sort();

        Ok(ListResponse { file_names })
    }

    async fn versions(&self, req: VersionsRequest) -> Result<VersionsResponse> {
        req.validate()?;

        let mut session_id = req.session_id.clone();

        // Handle user-namespaced artifacts
        if file_has_user_namespace(&req.file_name) {
            session_id = USER_SCOPED_ARTIFACT_KEY.to_string();
        }

        let artifacts = self.artifacts.read().unwrap();

        let max_key = ArtifactKey::with_max_version(
            req.app_name.clone(),
            req.user_id.clone(),
            session_id.clone(),
            req.file_name.clone(),
        );

        let min_key =
            ArtifactKey::with_min_version(req.app_name, req.user_id, session_id, req.file_name);

        let mut versions: Vec<_> = artifacts
            .range(min_key..max_key)
            .map(|(k, _)| k.version)
            .collect();

        versions.sort();

        Ok(VersionsResponse { versions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_load() {
        let service = InMemoryArtifactService::new();

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
        let service = InMemoryArtifactService::new();

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

        // Load version 1
        let load_req = LoadRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "test.txt".to_string(),
            version: Some(1),
        };

        let load_resp = service.load(load_req).await.unwrap();
        match load_resp.part {
            ArtifactPart::Text(text) => assert_eq!(text, "Version 1"),
            _ => panic!("Expected text part"),
        }
    }

    #[tokio::test]
    async fn test_user_namespaced_artifacts() {
        let service = InMemoryArtifactService::new();

        // Save user-namespaced artifact in session1
        let save_req = SaveRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session1".to_string(),
            file_name: "user:profile.json".to_string(),
            part: ArtifactPart::text("User profile"),
            version: None,
        };
        service.save(save_req).await.unwrap();

        // Load from different session (should work because it's user-namespaced)
        let load_req = LoadRequest {
            app_name: "test_app".to_string(),
            user_id: "user1".to_string(),
            session_id: "session2".to_string(),
            file_name: "user:profile.json".to_string(),
            version: None,
        };

        let load_resp = service.load(load_req).await.unwrap();
        match load_resp.part {
            ArtifactPart::Text(text) => assert_eq!(text, "User profile"),
            _ => panic!("Expected text part"),
        }
    }

    #[tokio::test]
    async fn test_list_artifacts() {
        let service = InMemoryArtifactService::new();

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
        let service = InMemoryArtifactService::new();

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
