use super::types::*;
use zdk_core::{Content, Error, LLMRequest, LLMResponse, Part, Result, LLM};
use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use reqwest::Client;

pub struct OpenAIModel {
    client: Client,
    api_key: String,
    model_name: String,
    base_url: String,
}

impl OpenAIModel {
    pub fn new(api_key: String, model_name: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model_name,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    fn build_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    /// Convert RAK Content format to OpenAI messages format
    fn convert_contents_to_messages(&self, contents: Vec<Content>) -> Vec<OpenAIMessage> {
        contents
            .into_iter()
            .map(|content| {
                let role = match content.role.as_str() {
                    "user" => "user",
                    "model" => "assistant",
                    "system" => "system",
                    _ => "user",
                };

                // Extract text from parts
                let text = content
                    .parts
                    .iter()
                    .filter_map(|part| {
                        if let Part::Text { text } = part {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                OpenAIMessage {
                    role: role.to_string(),
                    content: text,
                }
            })
            .collect()
    }

    /// Convert OpenAI response to RAK Content format
    fn convert_message_to_content(message: &OpenAIMessage) -> Content {
        Content {
            role: match message.role.as_str() {
                "assistant" => "model".to_string(),
                "user" => "user".to_string(),
                "system" => "system".to_string(),
                _ => "model".to_string(),
            },
            parts: vec![Part::Text {
                text: message.content.clone(),
            }],
        }
    }
}

#[async_trait]
impl LLM for OpenAIModel {
    fn name(&self) -> &str {
        &self.model_name
    }

    async fn generate_content(
        &self,
        request: LLMRequest,
        do_stream: bool,
    ) -> Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin> {
        let url = self.build_url();
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let model_name = self.model_name.clone();

        // Convert LLMRequest to OpenAIRequest
        let messages = self.convert_contents_to_messages(request.contents);
        let openai_req = OpenAIRequest {
            model: model_name,
            messages,
            temperature: request.config.as_ref().and_then(|c| c.temperature),
            max_tokens: request.config.as_ref().and_then(|c| c.max_tokens),
            top_p: request.config.as_ref().and_then(|c| c.top_p),
            stream: Some(do_stream),
        };

        if do_stream {
            // Streaming response
            Box::new(Box::pin(stream! {
                let response = client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&openai_req)
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            let status = resp.status();
                            let error_text = resp.text().await.unwrap_or_default();
                            yield Err(Error::LLMError(format!("OpenAI API error {}: {}", status, error_text)));
                            return;
                        }

                        let mut stream = resp.bytes_stream();
                        let mut accumulated_text = String::new();

                        while let Some(chunk) = stream.next().await {
                            match chunk {
                                Ok(bytes) => {
                                    if let Ok(text) = std::str::from_utf8(&bytes) {
                                        // Parse SSE format: "data: {json}\n\n"
                                        for line in text.lines() {
                                            if line.starts_with("data: ") {
                                                let json_str = &line[6..];
                                                
                                                // Check for end of stream
                                                if json_str.trim() == "[DONE]" {
                                                    continue;
                                                }

                                                match serde_json::from_str::<OpenAIStreamResponse>(json_str) {
                                                    Ok(stream_resp) => {
                                                        if let Some(choice) = stream_resp.choices.first() {
                                                            if let Some(ref content) = choice.delta.content {
                                                                accumulated_text.push_str(content);
                                                                
                                                                let finish_reason = choice.finish_reason.clone();
                                                                let is_done = finish_reason.is_some();

                                                                yield Ok(LLMResponse {
                                                                    content: Some(Content {
                                                                        role: "model".to_string(),
                                                                        parts: vec![Part::Text { text: content.clone() }],
                                                                    }),
                                                                    partial: true,
                                                                    turn_complete: is_done,
                                                                    interrupted: false,
                                                                    finish_reason,
                                                                    error_code: None,
                                                                    error_message: None,
                                                                });
                                                            }
                                                        }
                                                    }
                                                    Err(_e) => {
                                                        // Skip invalid SSE chunks
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    yield Err(Error::LLMError(format!("Stream error: {}", e)));
                                    return;
                                }
                            }
                        }

                        // Final response
                        yield Ok(LLMResponse {
                            content: None,
                            partial: false,
                            turn_complete: true,
                            interrupted: false,
                            finish_reason: Some("stop".to_string()),
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
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&openai_req)
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            let status = resp.status();
                            let error_text = resp.text().await.unwrap_or_default();
                            yield Err(Error::LLMError(format!("OpenAI API error {}: {}", status, error_text)));
                            return;
                        }

                        match resp.json::<OpenAIResponse>().await {
                            Ok(openai_resp) => {
                                if let Some(choice) = openai_resp.choices.first() {
                                    let content = Self::convert_message_to_content(&choice.message);
                                    yield Ok(LLMResponse {
                                        content: Some(content),
                                        partial: false,
                                        turn_complete: true,
                                        interrupted: false,
                                        finish_reason: choice.finish_reason.clone(),
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

