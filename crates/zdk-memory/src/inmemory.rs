//! In-memory memory service implementation

use crate::{MemoryEntry, MemoryService, SearchRequest, SearchResponse};
use zdk_core::{Content, Result};
use zdk_session::Session;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Key for memory storage, scoped to app and user
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MemoryKey {
    app_name: String,
    user_id: String,
}

/// Value stored in memory with pre-computed word set
#[derive(Debug, Clone)]
struct MemoryValue {
    content: Option<Content>,
    author: String,
    timestamp: DateTime<Utc>,
    /// Pre-computed set of words for efficient keyword matching
    words: HashSet<String>,
}

/// Memory store type: (app_name, user_id) -> session_id -> [memories]
type MemoryStore = HashMap<MemoryKey, HashMap<String, Vec<MemoryValue>>>;

/// In-memory implementation of the memory service.
///
/// This is suitable for testing and development. For production use,
/// consider using a persistent backend.
///
/// Thread-safe.
#[derive(Clone)]
pub struct InMemoryMemoryService {
    /// Storage: (app_name, user_id) -> session_id -> [memories]
    store: Arc<RwLock<MemoryStore>>,
}

impl InMemoryMemoryService {
    /// Create a new in-memory memory service
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryMemoryService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryService for InMemoryMemoryService {
    async fn add_session(&self, session: Arc<dyn Session>) -> Result<()> {
        let mut values = Vec::new();

        // Extract memories from session events
        for event in session.events() {
            // Skip events without content
            let content = match &event.content {
                Some(c) => c,
                None => continue,
            };

            // Extract words from content
            let mut words = HashSet::new();
            for part in &content.parts {
                if let zdk_core::Part::Text { text } = part {
                    words.extend(extract_words(text));
                }
            }

            // Skip if no words found
            if words.is_empty() {
                continue;
            }

            // Create memory value
            values.push(MemoryValue {
                content: Some(content.clone()),
                author: event.author.clone(),
                timestamp: DateTime::from_timestamp(event.time, 0).unwrap_or_else(Utc::now),
                words,
            });
        }

        // Store in memory
        let key = MemoryKey {
            app_name: session.app_name().to_string(),
            user_id: session.user_id().to_string(),
        };

        let mut store = self.store.write().unwrap();
        let session_map = store.entry(key).or_default();
        session_map.insert(session.id().to_string(), values);

        Ok(())
    }

    async fn search(&self, req: SearchRequest) -> Result<SearchResponse> {
        let query_words = extract_words(&req.query);

        let key = MemoryKey {
            app_name: req.app_name,
            user_id: req.user_id,
        };

        let store = self.store.read().unwrap();

        // Get memories for this user/app
        let session_map = match store.get(&key) {
            Some(map) => map,
            None => return Ok(SearchResponse { memories: vec![] }),
        };

        let mut memories = Vec::new();

        // Search through all sessions for matches
        for values in session_map.values() {
            for value in values {
                if check_word_intersection(&value.words, &query_words) {
                    memories.push(MemoryEntry {
                        content: value.content.clone(),
                        author: value.author.clone(),
                        timestamp: value.timestamp,
                    });
                }
            }
        }

        Ok(SearchResponse { memories })
    }
}

/// Extract words from text for keyword matching.
///
/// Words are:
/// - Split by whitespace
/// - Converted to lowercase
/// - Non-empty
fn extract_words(text: &str) -> HashSet<String> {
    text.split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// Check if two word sets have any intersection.
///
/// Optimized to iterate over the smaller set for efficiency.
fn check_word_intersection(words1: &HashSet<String>, words2: &HashSet<String>) -> bool {
    if words1.is_empty() || words2.is_empty() {
        return false;
    }

    // Iterate over the smaller set for efficiency
    let (smaller, larger) = if words1.len() < words2.len() {
        (words1, words2)
    } else {
        (words2, words1)
    };

    smaller.iter().any(|word| larger.contains(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_words() {
        let words = extract_words("The Quick Brown Fox");
        assert_eq!(words.len(), 4);
        assert!(words.contains("the"));
        assert!(words.contains("quick"));
        assert!(words.contains("brown"));
        assert!(words.contains("fox"));
    }

    #[test]
    fn test_extract_words_duplicates() {
        let words = extract_words("hello hello world");
        assert_eq!(words.len(), 2);
        assert!(words.contains("hello"));
        assert!(words.contains("world"));
    }

    #[test]
    fn test_word_intersection() {
        let words1: HashSet<_> = vec!["hello".to_string(), "world".to_string()]
            .into_iter()
            .collect();
        let words2: HashSet<_> = vec!["world".to_string(), "test".to_string()]
            .into_iter()
            .collect();

        assert!(check_word_intersection(&words1, &words2));
    }

    #[test]
    fn test_no_word_intersection() {
        let words1: HashSet<_> = vec!["hello".to_string()].into_iter().collect();
        let words2: HashSet<_> = vec!["world".to_string()].into_iter().collect();

        assert!(!check_word_intersection(&words1, &words2));
    }

    #[test]
    fn test_empty_word_intersection() {
        let words1: HashSet<_> = HashSet::new();
        let words2: HashSet<_> = vec!["world".to_string()].into_iter().collect();

        assert!(!check_word_intersection(&words1, &words2));
    }
}
