//! WebSocket protocol types for bidirectional communication

use zdk_core::{Content, Event};
use serde::{Deserialize, Serialize};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    /// Run an agent with a new message
    Run {
        #[serde(rename = "sessionId")]
        session_id: String,
        #[serde(rename = "newMessage")]
        new_message: Content,
    },
    /// Cancel a running invocation
    Cancel {
        #[serde(rename = "invocationId")]
        invocation_id: String,
    },
    /// Query the status of an invocation
    Status {
        #[serde(rename = "invocationId")]
        invocation_id: String,
    },
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerMessage {
    /// An event from the agent execution
    Event {
        #[serde(rename = "invocationId")]
        invocation_id: String,
        data: Event,
    },
    /// Status response for a status query
    Status {
        #[serde(rename = "invocationId")]
        invocation_id: String,
        status: InvocationStatus,
    },
    /// Error message
    Error { message: String },
    /// Confirmation that an invocation was cancelled
    Cancelled {
        #[serde(rename = "invocationId")]
        invocation_id: String,
    },
    /// Invocation started successfully
    Started {
        #[serde(rename = "invocationId")]
        invocation_id: String,
    },
    /// Invocation completed
    Completed {
        #[serde(rename = "invocationId")]
        invocation_id: String,
    },
}

/// Status of an invocation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvocationStatus {
    /// Invocation is currently running
    Active,
    /// Invocation completed successfully
    Completed,
    /// Invocation was cancelled
    Cancelled,
    /// Invocation not found (may have expired)
    NotFound,
}

