//! Gemini API types

use serde::{Deserialize, Serialize};
use crate::Content;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<SystemInstruction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<GeminiTool>,
}

/// Tool definition for Gemini API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiTool {
    pub function_declarations: Vec<GeminiFunctionDeclaration>,
}

/// Function declaration for Gemini API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInstruction {
    pub parts: Vec<SystemPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPart {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeminiResponse {
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
    pub error: Option<GeminiError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeminiError {
    pub code: Option<i32>,
    pub message: String,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Candidate {
    pub content: Content,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    pub prompt_token_count: Option<u32>,
    pub candidates_token_count: Option<u32>,
    pub total_token_count: Option<u32>,
}

