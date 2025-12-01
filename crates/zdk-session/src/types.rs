use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRequest {
    pub app_name: String,
    pub user_id: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequest {
    pub app_name: String,
    pub user_id: String,
    pub session_id: Option<String>,
}
