//! Memory service trait and types

use rak_core::{Content, Result};
use rak_session::Session;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Memory service trait for long-term knowledge storage.
///
/// The service ingests sessions into memory so that they can be used for
/// user queries across user-scoped sessions.
#[async_trait]
pub trait MemoryService: Send + Sync {
    /// Add a session to memory storage.
    ///
    /// A session can be added multiple times during its lifetime to update
    /// the stored knowledge.
    async fn add_session(&self, session: Arc<dyn Session>) -> Result<()>;

    /// Search for relevant memories.
    ///
    /// Returns memory entries that match the query keywords.
    /// Empty slice is returned if there are no matches.
    async fn search(&self, req: SearchRequest) -> Result<SearchResponse>;
}

/// Request for memory search
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// The search query
    pub query: String,
    /// User ID to scope the search
    pub user_id: String,
    /// Application name to scope the search
    pub app_name: String,
}

/// Response from memory search
#[derive(Debug, Clone)]
pub struct SearchResponse {
    /// List of matching memory entries
    pub memories: Vec<MemoryEntry>,
}

/// A single memory entry
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    /// Content of the memory
    pub content: Option<Content>,
    /// Author of the memory
    pub author: String,
    /// Timestamp when the original content happened
    pub timestamp: DateTime<Utc>,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(content: Option<Content>, author: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            content,
            author,
            timestamp,
        }
    }
}
