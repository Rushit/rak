//! OpenAI provider implementation

use super::{OpenAIConfig, types::*};
use crate::{
    AudioInput, EmbeddingVector, LLMRequest, LLMResponse, Result, TranscriptionResult,
    providers::provider::{Capability, ModelInfo, Provider, ProviderMetadata},
};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;

/// OpenAI provider with multi-capability support
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    config: OpenAIConfig,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: String, config: OpenAIConfig) -> Self {
        Self {
            client: Client::new(),
            api_key,
            config,
        }
    }

    /// Convert ZDK Content format to OpenAI messages format
    fn convert_contents_to_messages(&self, contents: Vec<crate::Content>) -> Vec<OpenAIMessage> {
        use crate::Part;

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

    /// Convert OpenAI response to ZDK Content format
    fn convert_message_to_content(message: &OpenAIMessage) -> crate::Content {
        use crate::Part;

        crate::Content {
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

    /// Get static metadata (for factory)
    pub fn static_metadata() -> ProviderMetadata {
        ProviderMetadata {
            name: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            capabilities: vec![
                Capability::TextGeneration,
                Capability::Embedding,
                Capability::Transcription,
                // Capability::ImageGeneration, // Future
                // Capability::AudioGeneration,  // Future
            ],
            models: vec![
                ModelInfo {
                    id: "gpt-4o".to_string(),
                    display_name: "GPT-4o".to_string(),
                    capabilities: vec![Capability::TextGeneration],
                    context_window: Some(128_000),
                    embedding_dimensions: None,
                },
                ModelInfo {
                    id: "gpt-4-turbo".to_string(),
                    display_name: "GPT-4 Turbo".to_string(),
                    capabilities: vec![Capability::TextGeneration],
                    context_window: Some(128_000),
                    embedding_dimensions: None,
                },
                ModelInfo {
                    id: "text-embedding-3-small".to_string(),
                    display_name: "Text Embedding 3 Small".to_string(),
                    capabilities: vec![Capability::Embedding],
                    context_window: None,
                    embedding_dimensions: Some(1536),
                },
                ModelInfo {
                    id: "whisper-1".to_string(),
                    display_name: "Whisper".to_string(),
                    capabilities: vec![Capability::Transcription],
                    context_window: None,
                    embedding_dimensions: None,
                },
            ],
        }
    }
}

#[async_trait]
impl crate::LLM for OpenAIProvider {
    fn name(&self) -> &str {
        &self.config.model
    }

    async fn generate_content(
        &self,
        request: crate::LLMRequest,
        do_stream: bool,
    ) -> Box<dyn futures::stream::Stream<Item = crate::Result<crate::LLMResponse>> + Send + Unpin>
    {
        <Self as Provider>::generate_content(self, request, do_stream)
            .await
            .unwrap() // Safe because our implementation never returns Err at this level
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn metadata(&self) -> ProviderMetadata {
        Self::static_metadata()
    }

    async fn generate_content(
        &self,
        request: LLMRequest,
        do_stream: bool,
    ) -> Result<Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin>> {
        use crate::{Content, Part};
        use async_stream::stream;
        use futures::stream::StreamExt;

        let url = format!("{}/chat/completions", self.config.base_url);
        let client = self.client.clone();
        let api_key = self.api_key.clone();

        // Convert LLMRequest to OpenAIRequest
        let messages = self.convert_contents_to_messages(request.contents);
        let openai_req = OpenAIRequest {
            model: self.config.model.clone(),
            messages,
            temperature: request.config.as_ref().and_then(|c| c.temperature),
            max_tokens: request.config.as_ref().and_then(|c| c.max_tokens),
            top_p: request.config.as_ref().and_then(|c| c.top_p),
            stream: Some(do_stream),
        };

        if do_stream {
            // Streaming response
            Ok(Box::new(Box::pin(stream! {
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
                            yield Err(crate::Error::LLMError(format!("OpenAI API error {}: {}", status, error_text)));
                            return;
                        }

                        let mut stream = resp.bytes_stream();

                        while let Some(chunk) = stream.next().await {
                            match chunk {
                                Ok(bytes) => {
                                    if let Ok(text) = std::str::from_utf8(&bytes) {
                                        // Parse SSE format: "data: {json}\n\n"
                                        for line in text.lines() {
                                            if let Some(json_str) = line.strip_prefix("data: ") {
                                                // Check for end of stream
                                                if json_str.trim() == "[DONE]" {
                                                    continue;
                                                }

                                                match serde_json::from_str::<OpenAIStreamResponse>(json_str) {
                                                    Ok(stream_resp) => {
                                                        if let Some(choice) = stream_resp.choices.first()
                                                            && let Some(ref content) = choice.delta.content
                                                        {
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
                                                    Err(_e) => {
                                                        // Skip invalid SSE chunks
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    yield Err(crate::Error::LLMError(format!("Stream error: {}", e)));
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
                        yield Err(crate::Error::LLMError(format!("Request failed: {}", e)));
                    }
                }
            })))
        } else {
            // Non-streaming response
            Ok(Box::new(Box::pin(stream! {
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
                            yield Err(crate::Error::LLMError(format!("OpenAI API error {}: {}", status, error_text)));
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
                                yield Err(crate::Error::LLMError(format!("Failed to parse response: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(crate::Error::LLMError(format!("Request failed: {}", e)));
                    }
                }
            })))
        }
    }

    async fn embed_texts(&self, texts: Vec<String>) -> Result<Vec<EmbeddingVector>> {
        use serde_json::json;

        let embedding_model = self
            .config
            .embedding_model
            .clone()
            .unwrap_or_else(|| "text-embedding-3-small".to_string());

        let url = format!("{}/embeddings", self.config.base_url);

        let request_body = json!({
            "input": texts,
            "model": embedding_model,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::Error::LLMError(format!("Embedding request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::Error::LLMError(format!(
                "Embedding API error {}: {}",
                status, error_text
            )));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            crate::Error::LLMError(format!("Failed to parse embedding response: {}", e))
        })?;

        // Parse embeddings from response
        let data = json["data"]
            .as_array()
            .ok_or_else(|| crate::Error::LLMError("Missing data array".into()))?;

        let results = data
            .iter()
            .map(|item| {
                let embedding = item["embedding"]
                    .as_array()
                    .ok_or_else(|| crate::Error::LLMError("Missing embedding array".into()))?;

                let vector: Vec<f32> = embedding
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();

                Ok(EmbeddingVector::new(vector))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }

    fn embedding_dimensions(&self) -> Option<usize> {
        Some(1536)
    }

    fn max_embedding_batch_size(&self) -> Option<usize> {
        Some(2048)
    }

    async fn transcribe_audio(&self, audio: AudioInput) -> Result<TranscriptionResult> {
        use reqwest::multipart;

        let url = format!("{}/audio/transcriptions", self.config.base_url);

        // Create multipart form
        let file_part = multipart::Part::bytes(audio.data)
            .file_name(format!("audio.{}", audio.format))
            .mime_str(&format!("audio/{}", audio.format))
            .map_err(|e| crate::Error::LLMError(format!("Invalid mime type: {}", e)))?;

        let mut form = multipart::Form::new()
            .part("file", file_part)
            .text("model", "whisper-1");

        // Add language if specified
        if let Some(language) = audio.language {
            form = form.text("language", language);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| crate::Error::LLMError(format!("Transcription request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::Error::LLMError(format!(
                "Transcription API error {}: {}",
                status, error_text
            )));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            crate::Error::LLMError(format!("Failed to parse transcription response: {}", e))
        })?;

        let text = json["text"]
            .as_str()
            .ok_or_else(|| crate::Error::LLMError("Missing text field in response".into()))?
            .to_string();

        let language = json["language"].as_str().map(|s| s.to_string());
        let duration = json["duration"].as_f64().map(|d| d as f32);

        Ok(TranscriptionResult {
            text,
            language,
            duration,
            segments: None, // OpenAI Whisper API doesn't return segments by default
        })
    }

    fn supported_audio_formats(&self) -> Option<&[&str]> {
        Some(&["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm"])
    }
}
