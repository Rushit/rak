use super::types::*;
use rak_core::{Error, LLMRequest, LLMResponse, Result, LLM};
use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::Client;

pub struct GeminiModel {
    client: Client,
    api_key: String,
    model_name: String,
    base_url: String,
}

impl GeminiModel {
    pub fn new(api_key: String, model_name: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model_name,
            base_url: "https://generativelanguage.googleapis.com/v1/models".to_string(),
        }
    }

    fn build_url(&self, stream: bool) -> String {
        let method = if stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };
        format!(
            "{}/{}:{}?key={}",
            self.base_url, self.model_name, method, self.api_key
        )
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
        };

        if do_stream {
            // Streaming response
            Box::new(Box::pin(stream! {
                let response = client
                    .post(&url)
                    .json(&gemini_req)
                    .send()
                    .await;

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
                let response = client
                    .post(&url)
                    .json(&gemini_req)
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        match resp.json::<GeminiResponse>().await {
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
    // Simple JSON extraction - look for complete objects
    if let Some(start) = buffer.find('{') {
        if let Some(end) = buffer[start..].find('}') {
            let json_str = buffer[start..start + end + 1].to_string();
            buffer.drain(..start + end + 1);
            return Some(json_str);
        }
    }
    None
}
