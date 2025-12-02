//! Shared test utilities for agent testing
//!
//! This module provides common mock implementations that can be reused
//! across all test modules, reducing duplication and ensuring consistency.

use async_stream::stream;
use async_trait::async_trait;
use futures::stream::Stream;
use std::sync::Arc;
use zdk_core::{
    Agent, Content, Event, InvocationContext, LLM, LLMRequest, LLMResponse, Part, Result,
};

/// Mock LLM for testing
///
/// Returns a simple test response for any request.
pub struct MockLLM {
    response_text: String,
}

impl MockLLM {
    /// Create a new MockLLM with default response
    pub fn new() -> Self {
        Self {
            response_text: "Test response".to_string(),
        }
    }

    /// Create a MockLLM with custom response text
    pub fn with_response(response: impl Into<String>) -> Self {
        Self {
            response_text: response.into(),
        }
    }
}

impl Default for MockLLM {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLM for MockLLM {
    fn name(&self) -> &str {
        "mock"
    }

    async fn generate_content(
        &self,
        _request: LLMRequest,
        _stream: bool,
    ) -> Box<dyn Stream<Item = Result<LLMResponse>> + Send + Unpin> {
        let response_text = self.response_text.clone();
        Box::new(Box::pin(stream! {
            yield Ok(LLMResponse {
                content: Some(Content {
                    role: "model".to_string(),
                    parts: vec![Part::Text { text: response_text }],
                }),
                partial: false,
                turn_complete: true,
                interrupted: false,
                finish_reason: Some("STOP".to_string()),
                error_code: None,
                error_message: None,
            });
        }))
    }
}

/// Mock Agent for testing workflows
///
/// Returns a configurable response and supports escalation flag.
pub struct MockAgent {
    name: String,
    response: String,
    escalate: bool,
}

impl MockAgent {
    /// Create a new MockAgent with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            response: "Mock agent response".to_string(),
            escalate: false,
        }
    }

    /// Set the response text
    pub fn with_response(mut self, response: impl Into<String>) -> Self {
        self.response = response.into();
        self
    }

    /// Set whether to escalate
    pub fn with_escalate(mut self, escalate: bool) -> Self {
        self.escalate = escalate;
        self
    }
}

#[async_trait]
impl Agent for MockAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Mock agent for testing"
    }

    async fn run(
        &self,
        ctx: Arc<dyn InvocationContext>,
    ) -> Box<dyn Stream<Item = Result<Event>> + Send + Unpin> {
        let response = self.response.clone();
        let escalate = self.escalate;
        let invocation_id = ctx.invocation_id().to_string();
        let name = self.name.clone();

        Box::new(Box::pin(stream! {
            let mut event = Event::new(invocation_id, name);
            event.content = Some(Content {
                role: "model".to_string(),
                parts: vec![Part::Text { text: response }],
            });
            event.turn_complete = true;
            event.actions.escalate = escalate;

            yield Ok(event);
        }))
    }
}
