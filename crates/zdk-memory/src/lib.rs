//! # ZDK Memory Service
//!
//! Long-term memory storage for AI agents, enabling knowledge retention
//! and retrieval across multiple sessions.
//!
//! ## Overview
//!
//! The memory service stores information from past sessions and enables
//! keyword-based search for relevant context. This provides agents with
//! long-term knowledge beyond the current conversation.
//!
//! ## Features
//!
//! - **Long-term storage**: Persist knowledge across sessions
//! - **User-scoped**: Memories are isolated per user and application
//! - **Keyword search**: Simple and efficient text-based search
//! - **Thread-safe**: Safe for concurrent access
//!
//! ## Usage
//!
//! ```rust,no_run
//! use zdk_memory::{InMemoryMemoryService, MemoryService, SearchRequest};
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create memory service
//! let memory_service = InMemoryMemoryService::new();
//!
//! // Add a session to memory (after conversation completes)
//! // memory_service.add_session(session).await?;
//!
//! // Search for relevant memories
//! let results = memory_service.search(SearchRequest {
//!     query: "weather forecast".into(),
//!     user_id: "user123".into(),
//!     app_name: "my_app".into(),
//! }).await?;
//!
//! // Use memories in agent context
//! for memory in results.memories {
//!     println!("Found memory from {}", memory.author);
//! }
//! # Ok(())
//! }
//! ```

mod inmemory;
mod service;

pub use inmemory::InMemoryMemoryService;
pub use service::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use zdk_core::{Content, Event};
    use zdk_session::Session;

    // Mock session for testing
    struct MockSession {
        id: String,
        app_name: String,
        user_id: String,
        events: Vec<Event>,
    }

    impl Session for MockSession {
        fn id(&self) -> &str {
            &self.id
        }

        fn app_name(&self) -> &str {
            &self.app_name
        }

        fn user_id(&self) -> &str {
            &self.user_id
        }

        fn events(&self) -> Vec<Event> {
            self.events.clone()
        }

        fn state(&self) -> HashMap<String, serde_json::Value> {
            HashMap::new()
        }
    }

    fn create_event_with_text(author: &str, text: &str, time: i64) -> Event {
        let mut event = Event::new("inv1".to_string(), author.to_string());
        event.content = Some(Content::new_user_text(text));
        event.time = time;
        event
    }

    #[tokio::test]
    async fn test_basic_search() {
        let service = InMemoryMemoryService::new();

        let session = Arc::new(MockSession {
            id: "sess1".to_string(),
            app_name: "app1".to_string(),
            user_id: "user1".to_string(),
            events: vec![
                create_event_with_text("user", "The quick brown fox", 1000),
                create_event_with_text("agent", "jumps over the lazy dog", 2000),
            ],
        });

        service.add_session(session).await.unwrap();

        let results = service
            .search(SearchRequest {
                query: "quick".to_string(),
                user_id: "user1".to_string(),
                app_name: "app1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(results.memories.len(), 1);
        assert_eq!(results.memories[0].author, "user");
    }

    #[tokio::test]
    async fn test_no_leakage_different_app() {
        let service = InMemoryMemoryService::new();

        let session = Arc::new(MockSession {
            id: "sess1".to_string(),
            app_name: "app1".to_string(),
            user_id: "user1".to_string(),
            events: vec![create_event_with_text("user", "test text", 1000)],
        });

        service.add_session(session).await.unwrap();

        let results = service
            .search(SearchRequest {
                query: "test".to_string(),
                user_id: "user1".to_string(),
                app_name: "other_app".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(results.memories.len(), 0);
    }

    #[tokio::test]
    async fn test_no_leakage_different_user() {
        let service = InMemoryMemoryService::new();

        let session = Arc::new(MockSession {
            id: "sess1".to_string(),
            app_name: "app1".to_string(),
            user_id: "user1".to_string(),
            events: vec![create_event_with_text("user", "test text", 1000)],
        });

        service.add_session(session).await.unwrap();

        let results = service
            .search(SearchRequest {
                query: "test".to_string(),
                user_id: "other_user".to_string(),
                app_name: "app1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(results.memories.len(), 0);
    }

    #[tokio::test]
    async fn test_case_insensitive_search() {
        let service = InMemoryMemoryService::new();

        let session = Arc::new(MockSession {
            id: "sess1".to_string(),
            app_name: "app1".to_string(),
            user_id: "user1".to_string(),
            events: vec![create_event_with_text("user", "The QUICK Brown FOX", 1000)],
        });

        service.add_session(session).await.unwrap();

        let results = service
            .search(SearchRequest {
                query: "quick brown".to_string(),
                user_id: "user1".to_string(),
                app_name: "app1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(results.memories.len(), 1);
    }

    #[tokio::test]
    async fn test_empty_store() {
        let service = InMemoryMemoryService::new();

        let results = service
            .search(SearchRequest {
                query: "test".to_string(),
                user_id: "user1".to_string(),
                app_name: "app1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(results.memories.len(), 0);
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        let service = InMemoryMemoryService::new();

        let session1 = Arc::new(MockSession {
            id: "sess1".to_string(),
            app_name: "app1".to_string(),
            user_id: "user1".to_string(),
            events: vec![create_event_with_text("user", "hello world", 1000)],
        });

        let session2 = Arc::new(MockSession {
            id: "sess2".to_string(),
            app_name: "app1".to_string(),
            user_id: "user1".to_string(),
            events: vec![create_event_with_text("agent", "hello there", 2000)],
        });

        service.add_session(session1).await.unwrap();
        service.add_session(session2).await.unwrap();

        let results = service
            .search(SearchRequest {
                query: "hello".to_string(),
                user_id: "user1".to_string(),
                app_name: "app1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(results.memories.len(), 2);
    }

    #[tokio::test]
    async fn test_no_matches() {
        let service = InMemoryMemoryService::new();

        let session = Arc::new(MockSession {
            id: "sess1".to_string(),
            app_name: "app1".to_string(),
            user_id: "user1".to_string(),
            events: vec![create_event_with_text("user", "test text", 1000)],
        });

        service.add_session(session).await.unwrap();

        let results = service
            .search(SearchRequest {
                query: "completely different query".to_string(),
                user_id: "user1".to_string(),
                app_name: "app1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(results.memories.len(), 0);
    }
}
