use serde::{Deserialize, Serialize};
use zdk_core::{Content, Event};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    #[serde(rename = "appName")]
    pub app_name: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "appName")]
    pub app_name: String,
    #[serde(rename = "userId")]
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAgentRequest {
    #[serde(rename = "newMessage")]
    pub new_message: Content,
    pub streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAgentResponse {
    pub events: Vec<Event>,
}
