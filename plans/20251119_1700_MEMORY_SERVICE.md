# ZDK Memory Service

**Date**: November 19, 2025, 17:00  
**Type**: Implementation Documentation  
**Status**: Complete  

## Overview

The ZDK Memory Service provides long-term memory storage for AI agents, enabling knowledge retention and retrieval across multiple sessions. This allows agents to remember past conversations and use that context in future interactions.

## Key Concepts

### Short-Term vs. Long-Term Memory

- **Short-Term Memory**: Managed by `SessionService`, this contains the current conversation history and session state. It's temporary and scoped to a single session.

- **Long-Term Memory**: Managed by `MemoryService`, this persists knowledge across sessions. It's permanent (until explicitly removed) and enables agents to recall information from past interactions.

### Architecture

```
┌─────────────────┐     ┌──────────────────┐
│  SessionService │────>│  MemoryService   │
│  (Short-term)   │     │  (Long-term)     │
└─────────────────┘     └──────────────────┘
        │                        │
        │                        │
   Current Session          Past Sessions
   (Conversation)          (Knowledge Base)
```

### User & Application Scoping

Memories are isolated by:
- **User ID**: Each user has their own memories
- **Application Name**: Memories are separated by application

This ensures privacy and prevents cross-contamination of knowledge.

## API Reference

### MemoryService Trait

```rust
#[async_trait]
pub trait MemoryService: Send + Sync {
    /// Add a session to memory storage
    async fn add_session(&self, session: Arc<dyn Session>) -> Result<()>;
    
    /// Search for relevant memories
    async fn search(&self, req: SearchRequest) -> Result<SearchResponse>;
}
```

### SearchRequest

```rust
pub struct SearchRequest {
    /// The search query (keywords)
    pub query: String,
    /// User ID to scope the search
    pub user_id: String,
    /// Application name to scope the search
    pub app_name: String,
}
```

### SearchResponse

```rust
pub struct SearchResponse {
    /// List of matching memory entries
    pub memories: Vec<MemoryEntry>,
}
```

### MemoryEntry

```rust
pub struct MemoryEntry {
    /// Content of the memory
    pub content: Option<Content>,
    /// Author of the memory (user, agent, etc.)
    pub author: String,
    /// Timestamp when the original content happened
    pub timestamp: DateTime<Utc>,
}
```

## Implementations

### InMemoryMemoryService

The in-memory implementation is suitable for development and testing. It stores all memories in RAM using a thread-safe data structure.

**Features**:
- Thread-safe with `RwLock`
- Fast keyword-based search
- No persistence (data lost on restart)
- Simple setup, no external dependencies

**Usage**:

```rust
use rak_memory::{InMemoryMemoryService, MemoryService, SearchRequest};
use std::sync::Arc;

// Create service
let memory_service = InMemoryMemoryService::new();

// Add a session to memory
memory_service.add_session(session).await?;

// Search for memories
let results = memory_service.search(SearchRequest {
    query: "weather forecast".to_string(),
    user_id: "alice".to_string(),
    app_name: "weather-app".to_string(),
}).await?;

// Use memories
for memory in results.memories {
    println!("Found: {:?}", memory.content);
}
```

## Search Implementation

### Keyword Matching

The current implementation uses simple keyword-based matching:

1. **Word Extraction**: Text is split by whitespace and converted to lowercase
2. **Pre-computation**: Word sets are computed when adding sessions
3. **Intersection Check**: Search queries are matched against stored word sets
4. **Case-Insensitive**: All matching is case-insensitive

### Algorithm

```rust
fn extract_words(text: &str) -> HashSet<String> {
    text.split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

fn check_word_intersection(words1: &HashSet<String>, words2: &HashSet<String>) -> bool {
    // Iterate over smaller set for efficiency
    let (smaller, larger) = if words1.len() < words2.len() {
        (words1, words2)
    } else {
        (words2, words1)
    };
    
    smaller.iter().any(|word| larger.contains(word))
}
```

### Performance Characteristics

- **Time Complexity**: O(n × m) where n = number of stored memories, m = average words per memory
- **Space Complexity**: O(n × w) where w = average unique words per memory
- **Optimization**: Word intersection uses the smaller set for efficiency

The keyword-based approach is simple and efficient for moderate memory sizes (up to thousands of entries). For larger scales or semantic search, consider embedding-based approaches in future enhancements.

## Examples

### Basic Usage

See `examples/memory_usage.rs` for a complete example showing:

- Adding multiple sessions to memory
- Searching with different keywords
- User isolation (memories are scoped to users)
- Application isolation (memories are scoped to apps)

### Workflow Integration

```rust
// After a conversation completes, add it to memory
let session = runner.create_session(app_name, user_id).await?;
// ... run agent interaction ...
memory_service.add_session(session.clone()).await?;

// In a new conversation, retrieve relevant context
let memories = memory_service.search(SearchRequest {
    query: user_input.clone(),
    user_id: user_id.clone(),
    app_name: app_name.clone(),
}).await?;

// Inject memories into agent context or system prompt
if !memories.memories.is_empty() {
    let context = format!("Relevant past conversations:\n{}", 
        format_memories(&memories.memories));
    // Add context to agent...
}
```

## Testing

The implementation includes comprehensive tests covering:

- ✅ Basic add and search operations
- ✅ Multi-session storage
- ✅ Keyword matching (case-insensitive)
- ✅ User isolation (no cross-user leakage)
- ✅ Application isolation (no cross-app leakage)
- ✅ Empty store behavior
- ✅ No matches scenario
- ✅ Thread safety (via RwLock)

All tests match the behavior of the Go ZDK implementation.

## Comparison with Go ZDK

The Rust implementation closely follows the Go ZDK's memory service:

### Similarities

- Same API structure (`AddSession`, `Search`)
- Same keyword-based matching algorithm
- Same isolation guarantees (user + app scoping)
- Same test cases for compatibility

### Differences

- **Type Safety**: Rust's type system provides stronger guarantees at compile time
- **Thread Safety**: Explicit with `RwLock` instead of Go's goroutine-safe maps
- **Error Handling**: Uses `Result` types with `anyhow` for error propagation
- **Async**: Uses Rust's `async/await` with `tokio` runtime

## Future Enhancements

The current implementation provides a solid foundation. Future phases could add:

### Phase 6+: Advanced Memory Features

1. **Embeddings-Based Search**
   - Vector embeddings for semantic search
   - Integration with embedding models
   - Similarity scoring and ranking

2. **Database Persistence**
   - PostgreSQL backend for long-term storage
   - SQLite for single-user applications
   - Vector database integration (pgvector, Qdrant, Pinecone)

3. **Memory Management**
   - Time-based decay (older memories become less relevant)
   - Memory summarization (compress old conversations)
   - Memory limits and pruning strategies
   - Importance scoring

4. **Advanced Search**
   - Relevance scoring
   - Time-range filtering
   - Metadata-based filtering
   - Hybrid search (keyword + semantic)

5. **Cross-Session Context**
   - Automatic context injection into agents
   - Memory-augmented generation (MAG)
   - Conversation threading
   - Topic clustering

## Integration with Other Components

### Session Service

The session service manages short-term memory (current conversation):

```rust
// Session Service (Short-term)
let session = session_service.get(session_id).await?;

// Memory Service (Long-term)
let memories = memory_service.search(SearchRequest { ... }).await?;
```

### Artifact Service

Artifacts can be referenced in memories for file-based knowledge:

```rust
// Store artifact reference in conversation
agent.process("Here's the report I generated").await?;
// ... save artifact ...

// Later, memory search can find references to that artifact
let memories = memory_service.search("report").await?;
```

### Future: Agent Integration

In future phases, agents could automatically:
- Query memories before responding
- Add important events to memory
- Use memories to maintain consistency
- Build user profiles over time

## Limitations

### Current Phase

- **No Persistence**: Memories are lost on restart (use a database backend in future)
- **Keyword-Only**: No semantic understanding (embeddings in future)
- **Simple Matching**: No stopwords, stemming, or advanced NLP
- **No Ranking**: Results are unordered (relevance scoring in future)
- **Linear Search**: O(n) time complexity (indexing in future)

These limitations are intentional to keep the initial implementation simple and aligned with the Go ZDK. Future enhancements will address them.

## Conclusion

The Memory Service provides essential long-term memory capabilities for ZDK agents. The keyword-based implementation is simple, efficient, and matches the Go ZDK behavior. It provides a solid foundation for future enhancements while maintaining compatibility and simplicity.

**Key Takeaways**:
- ✅ Long-term memory across sessions
- ✅ User and application isolation
- ✅ Keyword-based search
- ✅ Thread-safe and async
- ✅ Compatible with Go ZDK
- ✅ Foundation for future enhancements

