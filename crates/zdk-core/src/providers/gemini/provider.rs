//! Gemini provider implementation

use super::{GeminiConfig, auth::GeminiAuth, types::*};
use crate::{
    EmbeddingVector, LLMRequest, LLMResponse, Result,
    providers::provider::{Capability, ModelInfo, Provider, ProviderMetadata},
};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;

/// Gemini provider with multi-capability support
pub struct GeminiProvider {
    client: Client,
    auth: GeminiAuth,
    config: GeminiConfig,
}

impl GeminiProvider {
    /// Create a new Gemini provider
    pub fn new(auth: GeminiAuth, config: GeminiConfig) -> Self {
        Self {
            client: Client::new(),
            auth,
            config,
        }
    }

    /// Get static metadata (for factory)
    pub fn static_metadata() -> ProviderMetadata {
        ProviderMetadata {
            name: "gemini".to_string(),
            display_name: "Google Gemini".to_string(),
            capabilities: vec![
                Capability::TextGeneration,
                Capability::Embedding,
                // Capability::Transcription,  // Future
                // Capability::ImageGeneration, // Future
            ],
            models: vec![
                ModelInfo {
                    id: "gemini-2.0-flash-exp".to_string(),
                    display_name: "Gemini 2.0 Flash Experimental".to_string(),
                    capabilities: vec![Capability::TextGeneration],
                    context_window: Some(1_048_576),
                    embedding_dimensions: None,
                },
                ModelInfo {
                    id: "gemini-1.5-flash".to_string(),
                    display_name: "Gemini 1.5 Flash".to_string(),
                    capabilities: vec![Capability::TextGeneration],
                    context_window: Some(1_048_576),
                    embedding_dimensions: None,
                },
                ModelInfo {
                    id: "text-embedding-004".to_string(),
                    display_name: "Text Embedding 004".to_string(),
                    capabilities: vec![Capability::Embedding],
                    context_window: None,
                    embedding_dimensions: Some(768),
                },
            ],
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
                    self.config.base_url, self.config.model, method, key
                )
            }
            GeminiAuth::BearerToken(_) => {
                format!("{}/{}:{}", self.config.base_url, self.config.model, method)
            }
        }
    }
}

#[async_trait]
impl crate::LLM for GeminiProvider {
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
impl Provider for GeminiProvider {
    fn metadata(&self) -> ProviderMetadata {
        Self::static_metadata()
    }

    async fn generate_content(
        &self,
        request: LLMRequest,
        do_stream: bool,
    ) -> Result<Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin>> {
        use async_stream::stream;
        use futures::stream::StreamExt;

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

        if do_stream {
            // Streaming response
            Ok(Box::new(Box::pin(stream! {
                let mut req_builder = client.post(&url).json(&gemini_req);

                // Apply authentication
                req_builder = auth.apply(req_builder);

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
                                        while let Some(json_str) = extract_json(&mut buffer) {
                                            match serde_json::from_str::<GeminiResponse>(&json_str) {
                                                Ok(gemini_resp) => {
                                                    // Check for API error
                                                    if let Some(error) = gemini_resp.error {
                                                        yield Err(crate::Error::LLMError(format!(
                                                            "Gemini API error: {} (code: {})",
                                                            error.message,
                                                            error.code.unwrap_or(0)
                                                        )));
                                                        return;
                                                    }

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
                                                    yield Err(crate::Error::LLMError(format!("Failed to parse response: {}", e)));
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    yield Err(crate::Error::LLMError(format!("Stream error: {}", e)));
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
                        yield Err(crate::Error::LLMError(format!("Request failed: {}", e)));
                    }
                }
            })))
        } else {
            // Non-streaming response
            Ok(Box::new(Box::pin(stream! {
                let mut req_builder = client.post(&url).json(&gemini_req);

                // Apply authentication
                req_builder = auth.apply(req_builder);

                let response = req_builder.send().await;

                match response {
                    Ok(resp) => {
                        let response_text = resp.text().await.unwrap_or_else(|_| "failed to read response".to_string());

                        match serde_json::from_str::<GeminiResponse>(&response_text) {
                            Ok(gemini_resp) => {
                                // Check for API error
                                if let Some(error) = gemini_resp.error {
                                    yield Err(crate::Error::LLMError(format!(
                                        "Gemini API error: {} (code: {})",
                                        error.message,
                                        error.code.unwrap_or(0)
                                    )));
                                    return;
                                }

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
            .unwrap_or_else(|| "text-embedding-004".to_string());

        // Build batch embedding request
        let requests: Vec<_> = texts
            .iter()
            .map(|text| {
                json!({
                    "model": format!("models/{}", embedding_model),
                    "content": {
                        "parts": [{ "text": text }]
                    }
                })
            })
            .collect();

        let request_body = json!({ "requests": requests });

        // Build URL for embedding API
        let url = format!(
            "{}/{}:batchEmbedContents",
            self.config.base_url, embedding_model
        );

        let mut req_builder = self.client.post(&url).json(&request_body);

        // Apply authentication
        req_builder = self.auth.apply(req_builder);

        let response = req_builder
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
        let embeddings = json["embeddings"]
            .as_array()
            .ok_or_else(|| crate::Error::LLMError("Missing embeddings array".into()))?;

        let results = embeddings
            .iter()
            .map(|emb| {
                let values = emb["values"]
                    .as_array()
                    .ok_or_else(|| crate::Error::LLMError("Missing embedding values".into()))?;

                let vector: Vec<f32> = values
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();

                Ok(EmbeddingVector::new(vector))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }

    fn embedding_dimensions(&self) -> Option<usize> {
        Some(768)
    }

    fn max_embedding_batch_size(&self) -> Option<usize> {
        Some(100)
    }
}

/// Helper function to extract JSON from SSE format
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
