//! Database models for session storage

use chrono::{DateTime, Utc};

/// Session model for database storage
#[derive(Debug, Clone, sqlx::FromRow)]
pub(super) struct SessionRow {
    pub app_name: String,
    pub user_id: String,
    pub id: String,
    pub state: String, // JSON string
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

/// Event model for database storage
#[derive(Debug, Clone, sqlx::FromRow)]
pub(super) struct EventRow {
    pub id: String,
    pub app_name: String,
    pub user_id: String,
    pub session_id: String,
    pub invocation_id: String,
    pub author: String,
    pub actions: String,                       // JSON string
    pub long_running_tool_ids: Option<String>, // JSON string
    pub branch: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub content: Option<String>,            // JSON string
    pub grounding_metadata: Option<String>, // JSON string
    pub custom_metadata: Option<String>,    // JSON string (kept for compatibility)
    pub usage_metadata: Option<String>,     // JSON string (kept for compatibility)
    pub citation_metadata: Option<String>,  // JSON string (kept for compatibility)
    pub partial: Option<i32>,               // Boolean as integer
    pub turn_complete: Option<i32>,         // Boolean as integer
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub interrupted: Option<i32>, // Boolean as integer
}

/// App state model for database storage
#[derive(Debug, Clone, sqlx::FromRow)]
pub(super) struct AppStateRow {
    pub app_name: String,
    pub state: String, // JSON string
    pub update_time: DateTime<Utc>,
}

/// User state model for database storage
#[derive(Debug, Clone, sqlx::FromRow)]
pub(super) struct UserStateRow {
    pub app_name: String,
    pub user_id: String,
    pub state: String, // JSON string
    pub update_time: DateTime<Utc>,
}

impl EventRow {
    /// Convert from rak_core::Event to EventRow
    pub fn from_event(
        event: &rak_core::Event,
        app_name: &str,
        user_id: &str,
        session_id: &str,
    ) -> Result<Self, serde_json::Error> {
        // Convert Unix timestamp (i64) to DateTime
        let timestamp = DateTime::from_timestamp(event.time, 0).unwrap_or_else(Utc::now);

        Ok(Self {
            id: event.id.clone(),
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            invocation_id: event.invocation_id.clone(),
            author: event.author.clone(),
            actions: serde_json::to_string(&event.actions)?,
            long_running_tool_ids: if event.long_running_tool_ids.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&event.long_running_tool_ids)?)
            },
            branch: if event.branch.is_empty() {
                None
            } else {
                Some(event.branch.clone())
            },
            timestamp,
            content: event
                .content
                .as_ref()
                .map(|c| serde_json::to_string(c))
                .transpose()?,
            grounding_metadata: event
                .grounding_metadata
                .as_ref()
                .map(|g| serde_json::to_string(g))
                .transpose()?,
            custom_metadata: None,   // Not present in current Event structure
            usage_metadata: None,    // Not present in current Event structure
            citation_metadata: None, // Not present in current Event structure
            partial: Some(if event.partial { 1 } else { 0 }),
            turn_complete: Some(if event.turn_complete { 1 } else { 0 }),
            error_code: if event.error_code.is_empty() {
                None
            } else {
                Some(event.error_code.clone())
            },
            error_message: if event.error_message.is_empty() {
                None
            } else {
                Some(event.error_message.clone())
            },
            interrupted: Some(if event.interrupted { 1 } else { 0 }),
        })
    }

    /// Convert from EventRow to rak_core::Event
    pub fn to_event(&self) -> Result<rak_core::Event, serde_json::Error> {
        let actions: rak_core::EventActions = serde_json::from_str(&self.actions)?;

        let long_running_tool_ids = self
            .long_running_tool_ids
            .as_ref()
            .map(|s| serde_json::from_str(s))
            .transpose()?
            .unwrap_or_default();

        let content = self
            .content
            .as_ref()
            .map(|s| serde_json::from_str(s))
            .transpose()?;

        let grounding_metadata = self
            .grounding_metadata
            .as_ref()
            .map(|s| serde_json::from_str(s))
            .transpose()?;

        Ok(rak_core::Event {
            id: self.id.clone(),
            time: self.timestamp.timestamp(),
            invocation_id: self.invocation_id.clone(),
            branch: self.branch.clone().unwrap_or_default(),
            author: self.author.clone(),
            partial: self.partial.unwrap_or(0) != 0,
            turn_complete: self.turn_complete.unwrap_or(0) != 0,
            interrupted: self.interrupted.unwrap_or(0) != 0,
            content,
            grounding_metadata,
            error_code: self.error_code.clone().unwrap_or_default(),
            error_message: self.error_message.clone().unwrap_or_default(),
            long_running_tool_ids,
            actions,
        })
    }
}
