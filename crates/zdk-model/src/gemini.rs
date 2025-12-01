use super::types::*;
use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use zdk_core::{Error, LLM, LLMRequest, LLMResponse, Result};

/// Authentication method for Gemini API
#[derive(Clone, Debug)]
pub enum GeminiAuth {
    /// API Key authentication (for generativelanguage.googleapis.com)
    ApiKey(String),
    /// Bearer token authentication (for Vertex AI via gcloud)
    BearerToken(String),
}

pub struct GeminiModel {
    client: Client,
    auth: GeminiAuth,
    model_name: String,
    base_url: String,
}

impl GeminiModel {
    /// Create a new Gemini model with API key authentication.
    ///
    /// This uses the public Gemini API endpoint.
    pub fn new(api_key: String, model_name: String) -> Self {
        Self {
            client: Client::new(),
            auth: GeminiAuth::ApiKey(api_key),
            model_name,
            base_url: "https://generativelanguage.googleapis.com/v1/models".to_string(),
        }
    }

    /// Create a new Gemini model with Bearer token authentication (e.g., from gcloud).
    ///
    /// This is typically used with Vertex AI endpoints.
    ///
    /// # Arguments
    ///
    /// * `access_token` - Bearer access token (e.g., from `gcloud auth print-access-token`)
    /// * `model_name` - Model name (e.g., "gemini-1.5-flash")
    /// * `project_id` - GCP project ID
    /// * `location` - GCP location (e.g., "us-central1")
    pub fn with_bearer_token(
        access_token: String,
        model_name: String,
        project_id: String,
        location: String,
    ) -> Self {
        Self {
            client: Client::new(),
            auth: GeminiAuth::BearerToken(access_token),
            model_name: model_name.clone(),
            base_url: format!(
                "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models",
                location, project_id, location
            ),
        }
    }

    fn build_url(&self, stream: bool) -> String {
        let method = if stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };

        match &self.auth {
            GeminiAuth::ApiKey(key) => {
                format!(
                    "{}/{}:{}?key={}",
                    self.base_url, self.model_name, method, key
                )
            }
            GeminiAuth::BearerToken(_) => {
                format!("{}/{}:{}", self.base_url, self.model_name, method)
            }
        }
    }
}

#[async_trait]
impl LLM for GeminiModel {
    fn name(&self) -> &str {
        &self.model_name
    }

    async fn generate_content(
        &self,
        request: LLMRequest,
        do_stream: bool,
    ) -> Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin> {
        let url = self.build_url(do_stream);
        let client = self.client.clone();
        let auth = self.auth.clone();

        // Convert tools to Gemini format
        let tools = if request.tools.is_empty() {
            vec![]
        } else {
            vec![GeminiTool {
                function_declarations: request
                    .tools
                    .iter()
                    .map(|tool| GeminiFunctionDeclaration {
                        name: tool.name().to_string(),
                        description: tool.description().to_string(),
                        parameters: tool.schema(),
                    })
                    .collect(),
            }]
        };

        // Convert LLMRequest to GeminiRequest
        let gemini_req = GeminiRequest {
            contents: request.contents,
            generation_config: request.config.map(|c| GenerationConfig {
                temperature: c.temperature,
                max_output_tokens: c.max_tokens,
                top_p: c.top_p,
                top_k: c.top_k,
            }),
            system_instruction: None,
            tools,
        };

        // Debug logging removed - uncomment if needed
        // eprintln!(
        //     "DEBUG: Gemini API request: {}",
        //     serde_json::to_string_pretty(&gemini_req).unwrap_or_else(|_| "failed to serialize".to_string())
        // );

        if do_stream {
            // Streaming response
            Box::new(Box::pin(stream! {
                let mut req_builder = client.post(&url).json(&gemini_req);

                // Add auth header if using bearer token
                if let GeminiAuth::BearerToken(token) = &auth {
                    req_builder = req_builder.bearer_auth(token);
                }

                let response = req_builder.send().await;

                match response {
                    Ok(resp) => {
                        let mut stream = resp.bytes_stream();
                        let mut buffer = String::new();

                        while let Some(chunk) = stream.next().await {
                            match chunk {
                                Ok(bytes) => {
                                    if let Ok(text) = std::str::from_utf8(&bytes) {
                                        buffer.push_str(text);

                                        // Parse JSON objects from buffer
                                        if let Some(json_str) = extract_json(&mut buffer) {
                                            // eprintln!("DEBUG: Streaming response chunk: {}", json_str);
                                            match serde_json::from_str::<GeminiResponse>(&json_str) {
                                                Ok(gemini_resp) => {
                                                    if let Some(candidate) = gemini_resp.candidates.first() {
                                                        yield Ok(LLMResponse {
                                                            content: Some(candidate.content.clone()),
                                                            partial: true,
                                                            turn_complete: false,
                                                            interrupted: false,
                                                            finish_reason: candidate.finish_reason.clone(),
                                                            error_code: None,
                                                            error_message: None,
                                                        });
                                                    }
                                                }
                                                Err(e) => {
                                                    yield Err(Error::LLMError(format!("Failed to parse response: {}", e)));
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    yield Err(Error::LLMError(format!("Stream error: {}", e)));
                                }
                            }
                        }

                        // Final response
                        yield Ok(LLMResponse {
                            content: None,
                            partial: false,
                            turn_complete: true,
                            interrupted: false,
                            finish_reason: Some("STOP".to_string()),
                            error_code: None,
                            error_message: None,
                        });
                    }
                    Err(e) => {
                        yield Err(Error::LLMError(format!("Request failed: {}", e)));
                    }
                }
            }))
        } else {
            // Non-streaming response
            Box::new(Box::pin(stream! {
                let mut req_builder = client.post(&url).json(&gemini_req);

                // Add auth header if using bearer token
                if let GeminiAuth::BearerToken(token) = &auth {
                    req_builder = req_builder.bearer_auth(token);
                }

                let response = req_builder.send().await;

                match response {
                    Ok(resp) => {
                        // Get response text for debugging
                        let response_text = resp.text().await.unwrap_or_else(|_| "failed to read response".to_string());
                        // eprintln!("DEBUG: Gemini API response: {}", response_text);

                        match serde_json::from_str::<GeminiResponse>(&response_text) {
                            Ok(gemini_resp) => {
                                if let Some(candidate) = gemini_resp.candidates.first() {
                                    yield Ok(LLMResponse {
                                        content: Some(candidate.content.clone()),
                                        partial: false,
                                        turn_complete: true,
                                        interrupted: false,
                                        finish_reason: candidate.finish_reason.clone(),
                                        error_code: None,
                                        error_message: None,
                                    });
                                }
                            }
                            Err(e) => {
                                yield Err(Error::LLMError(format!("Failed to parse response: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(Error::LLMError(format!("Request failed: {}", e)));
                    }
                }
            }))
        }
    }
}

// Helper function to extract JSON from SSE format
fn extract_json(buffer: &mut String) -> Option<String> {
    // Find the start of a JSON object
    let start = buffer.find('{')?;

    // Track brace depth to find the matching closing brace
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in buffer[start..].char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    // Found complete JSON object
                    let end = start + i + 1;
                    let json_str = buffer[start..end].to_string();
                    buffer.drain(..end);
                    return Some(json_str);
                }
            }
            _ => {}
        }
    }

    None
}
