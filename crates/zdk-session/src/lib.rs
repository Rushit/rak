//! Session management for ZDK

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use zdk_core::{Event, Result};

pub mod inmemory;
pub mod types;

#[cfg(feature = "sqlx")]
pub mod database;

pub use types::{CreateRequest, GetRequest};

#[cfg(feature = "postgres")]
pub use database::PostgresSessionService;

#[cfg(feature = "sqlite")]
pub use database::SqliteSessionService;

/// Session service trait
#[async_trait]
pub trait SessionService: Send + Sync {
    async fn get(&self, req: &GetRequest) -> Result<Arc<dyn Session>>;
    async fn create(&self, req: &CreateRequest) -> Result<Arc<dyn Session>>;
    async fn append_event(&self, session_id: &str, event: Event) -> Result<()>;
}

/// Session trait
pub trait Session: Send + Sync {
    fn id(&self) -> &str;
    fn app_name(&self) -> &str;
    fn user_id(&self) -> &str;
    fn events(&self) -> Vec<Event>;
    fn state(&self) -> HashMap<String, serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inmemory::InMemorySessionService;

    #[tokio::test]
    async fn test_create_session() {
        let service = InMemorySessionService::new();

        let session = service
            .create(&CreateRequest {
                app_name: "test-app".to_string(),
                user_id: "user1".to_string(),
                session_id: Some("session1".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(session.id(), "session1");
        assert_eq!(session.app_name(), "test-app");
        assert_eq!(session.user_id(), "user1");
    }

    #[tokio::test]
    async fn test_get_session() {
        let service = InMemorySessionService::new();

        // Create session
        service
            .create(&CreateRequest {
                app_name: "test-app".to_string(),
                user_id: "user1".to_string(),
                session_id: Some("session1".to_string()),
            })
            .await
            .unwrap();

        // Get session
        let session = service
            .get(&GetRequest {
                app_name: "test-app".to_string(),
                user_id: "user1".to_string(),
                session_id: "session1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(session.id(), "session1");
    }

    #[tokio::test]
    async fn test_append_events() {
        let service = InMemorySessionService::new();

        let _session = service
            .create(&CreateRequest {
                app_name: "test-app".to_string(),
                user_id: "user1".to_string(),
                session_id: Some("session1".to_string()),
            })
            .await
            .unwrap();

        // Add events
        let event1 = zdk_core::Event::new("inv1".to_string(), "user".to_string());
        let event2 = zdk_core::Event::new("inv1".to_string(), "agent".to_string());

        service.append_event("session1", event1).await.unwrap();
        service.append_event("session1", event2).await.unwrap();

        // Verify events
        let updated_session = service
            .get(&GetRequest {
                app_name: "test-app".to_string(),
                user_id: "user1".to_string(),
                session_id: "session1".to_string(),
            })
            .await
            .unwrap();

        let events = updated_session.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].author, "user");
        assert_eq!(events[1].author, "agent");
    }
}
