use super::Content;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Event represents a single interaction in a conversation.
///
/// Each event captures a moment in the agent's execution flow, including user messages,
/// agent responses, tool calls, and metadata. Events are JSON-serializable for streaming
/// and storage, with camelCase field names for API compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub time: i64,
    pub invocation_id: String,
    pub branch: String,
    pub author: String,
    pub partial: bool,
    pub turn_complete: bool,
    pub interrupted: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_metadata: Option<GroundingMetadata>,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub error_code: String,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub error_message: String,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub long_running_tool_ids: Vec<String>,

    pub actions: EventActions,
}

impl Event {
    pub fn new(invocation_id: String, author: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            time: Utc::now().timestamp(),
            invocation_id,
            branch: String::new(),
            author,
            partial: false,
            turn_complete: false,
            interrupted: false,
            content: None,
            grounding_metadata: None,
            error_code: String::new(),
            error_message: String::new(),
            long_running_tool_ids: Vec::new(),
            actions: EventActions::default(),
        }
    }

    pub fn is_final_response(&self) -> bool {
        !self.partial && self.turn_complete
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventActions {
    pub state_delta: HashMap<String, serde_json::Value>,
    pub artifact_delta: HashMap<String, i64>,
    #[serde(default)]
    pub escalate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingMetadata {
    // Placeholder for grounding metadata structure
    pub search_entry_point: Option<serde_json::Value>,
}
