//! Example demonstrating the memory service
//!
//! This example shows how to:
//! - Add sessions to memory
//! - Search for relevant memories using keywords
//! - Use memories across sessions
//!
//! Run with: cargo run --example memory_usage

use std::collections::HashMap;
use std::sync::Arc;
use zdk_core::{Content, Event};
use zdk_memory::{InMemoryMemoryService, MemoryService, SearchRequest};
use zdk_session::Session;

/// Simple mock session for demonstration
struct DemoSession {
    id: String,
    app_name: String,
    user_id: String,
    events: Vec<Event>,
}

impl Session for DemoSession {
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

fn create_text_event(author: &str, text: &str, timestamp: i64) -> Event {
    let mut event = Event::new("inv1".to_string(), author.to_string());
    event.content = Some(Content::new_user_text(text));
    event.time = timestamp;
    event
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== ZDK Memory Service Example ===\n");

    // Create memory service
    let memory_service = InMemoryMemoryService::new();
    println!("✓ Created in-memory memory service\n");

    // Session 1: Weather conversation
    println!("Session 1: Adding weather conversation to memory");
    let session1 = Arc::new(DemoSession {
        id: "sess-001".to_string(),
        app_name: "weather-app".to_string(),
        user_id: "alice".to_string(),
        events: vec![
            create_text_event("user", "What's the weather forecast for tomorrow?", 1000),
            create_text_event(
                "agent",
                "Tomorrow will be sunny with temperatures around 75°F",
                2000,
            ),
            create_text_event("user", "Should I bring an umbrella?", 3000),
            create_text_event(
                "agent",
                "No need for an umbrella tomorrow, it will be clear skies",
                4000,
            ),
        ],
    });

    memory_service.add_session(session1).await?;
    println!("  ✓ Added 4 events from weather conversation\n");

    // Session 2: Recipe conversation
    println!("Session 2: Adding recipe conversation to memory");
    let session2 = Arc::new(DemoSession {
        id: "sess-002".to_string(),
        app_name: "weather-app".to_string(),
        user_id: "alice".to_string(),
        events: vec![
            create_text_event("user", "How do I make chocolate chip cookies?", 5000),
            create_text_event(
                "agent",
                "Mix flour, sugar, butter, eggs, and chocolate chips, then bake at 350°F",
                6000,
            ),
        ],
    });

    memory_service.add_session(session2).await?;
    println!("  ✓ Added 2 events from recipe conversation\n");

    // Search 1: Weather-related query
    println!("Search 1: Looking for 'weather forecast'");
    let results = memory_service
        .search(SearchRequest {
            query: "weather forecast".to_string(),
            user_id: "alice".to_string(),
            app_name: "weather-app".to_string(),
        })
        .await?;

    println!("  Found {} matching memories:", results.memories.len());
    for (i, memory) in results.memories.iter().enumerate() {
        if let Some(content) = &memory.content {
            for part in &content.parts {
                if let zdk_core::Part::Text { text } = part {
                    println!("    {}. [{}] {}", i + 1, memory.author, text);
                }
            }
        }
    }
    println!();

    // Search 2: Umbrella-related query
    println!("Search 2: Looking for 'umbrella'");
    let results = memory_service
        .search(SearchRequest {
            query: "umbrella".to_string(),
            user_id: "alice".to_string(),
            app_name: "weather-app".to_string(),
        })
        .await?;

    println!("  Found {} matching memories:", results.memories.len());
    for (i, memory) in results.memories.iter().enumerate() {
        if let Some(content) = &memory.content {
            for part in &content.parts {
                if let zdk_core::Part::Text { text } = part {
                    println!("    {}. [{}] {}", i + 1, memory.author, text);
                }
            }
        }
    }
    println!();

    // Search 3: Recipe-related query
    println!("Search 3: Looking for 'chocolate cookies'");
    let results = memory_service
        .search(SearchRequest {
            query: "chocolate cookies".to_string(),
            user_id: "alice".to_string(),
            app_name: "weather-app".to_string(),
        })
        .await?;

    println!("  Found {} matching memories:", results.memories.len());
    for (i, memory) in results.memories.iter().enumerate() {
        if let Some(content) = &memory.content {
            for part in &content.parts {
                if let zdk_core::Part::Text { text } = part {
                    println!("    {}. [{}] {}", i + 1, memory.author, text);
                }
            }
        }
    }
    println!();

    // Search 4: No match
    println!("Search 4: Looking for 'astronomy' (no match expected)");
    let results = memory_service
        .search(SearchRequest {
            query: "astronomy".to_string(),
            user_id: "alice".to_string(),
            app_name: "weather-app".to_string(),
        })
        .await?;

    println!("  Found {} matching memories", results.memories.len());
    println!();

    // Demonstrate user isolation
    println!("Demonstrating user isolation:");
    let session3 = Arc::new(DemoSession {
        id: "sess-003".to_string(),
        app_name: "weather-app".to_string(),
        user_id: "bob".to_string(),
        events: vec![create_text_event("user", "I like sunny weather", 7000)],
    });

    memory_service.add_session(session3).await?;
    println!("  ✓ Added session for user 'bob'\n");

    println!("  Searching 'sunny' for alice:");
    let results = memory_service
        .search(SearchRequest {
            query: "sunny".to_string(),
            user_id: "alice".to_string(),
            app_name: "weather-app".to_string(),
        })
        .await?;
    println!("    Found {} memories", results.memories.len());

    println!("  Searching 'sunny' for bob:");
    let results = memory_service
        .search(SearchRequest {
            query: "sunny".to_string(),
            user_id: "bob".to_string(),
            app_name: "weather-app".to_string(),
        })
        .await?;
    println!("    Found {} memories", results.memories.len());
    println!();

    println!("=== Example Complete ===");
    println!("\nKey takeaways:");
    println!("  • Memory service stores events from completed sessions");
    println!("  • Keyword-based search finds relevant past conversations");
    println!("  • Memories are isolated per user and application");
    println!("  • Case-insensitive matching for better recall");

    Ok(())
}
