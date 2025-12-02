use crate::context::DefaultInvocationContext;
use async_stream::stream;
use futures::stream::{Stream, StreamExt};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use zdk_core::{Agent, Content, Error, Event, Result};
use zdk_session::{CreateRequest, SessionService};

pub struct Runner {
    app_name: String,
    agent: Arc<dyn Agent>,
    session_service: Arc<dyn SessionService>,
}

impl Runner {
    pub fn builder() -> RunnerBuilder {
        RunnerBuilder::new()
    }

    pub async fn run(
        &self,
        user_id: String,
        session_id: String,
        message: Content,
        config: RunConfig,
    ) -> Result<Box<dyn Stream<Item = Result<Event>> + Send + Unpin>> {
        self.run_with_cancellation(user_id, session_id, message, config, None)
            .await
    }

    pub async fn run_with_cancellation(
        &self,
        user_id: String,
        session_id: String,
        message: Content,
        _config: RunConfig,
        cancel_token: Option<CancellationToken>,
    ) -> Result<Box<dyn Stream<Item = Result<Event>> + Send + Unpin>> {
        // Get or create session
        let _session = match self
            .session_service
            .get(&zdk_session::GetRequest {
                app_name: self.app_name.clone(),
                user_id: user_id.clone(),
                session_id: session_id.clone(),
            })
            .await
        {
            Ok(s) => s,
            Err(_) => {
                // Session doesn't exist, create it
                self.session_service
                    .create(&CreateRequest {
                        app_name: self.app_name.clone(),
                        user_id: user_id.clone(),
                        session_id: Some(session_id.clone()),
                    })
                    .await?
            }
        };

        // Create invocation context
        let invocation_id = Uuid::new_v4().to_string();
        let ctx = Arc::new(DefaultInvocationContext::new(
            invocation_id.clone(),
            self.app_name.clone(),
            user_id,
            session_id.clone(),
            Some(message.clone()),
            self.agent.clone(),
        ));

        // Add user message to session as an event
        let mut user_event = Event::new(invocation_id.clone(), "user".to_string());
        user_event.content = Some(message);
        user_event.turn_complete = true;

        self.session_service
            .append_event(&session_id, user_event)
            .await?;

        // Run agent and stream events
        let agent = self.agent.clone();
        let session_service = self.session_service.clone();
        let session_id_clone = session_id.clone();

        Ok(Box::new(Box::pin(stream! {
            let mut event_stream = agent.run(ctx).await;

            loop {
                // Check cancellation
                if let Some(ref token) = cancel_token
                    && token.is_cancelled()
                {
                    // Create cancellation event
                    let mut cancel_event = Event::new(invocation_id.clone(), "system".to_string());
                    cancel_event.error_message = "Invocation cancelled".to_string();
                    cancel_event.turn_complete = true;
                    yield Ok(cancel_event);
                    return;
                }

                // Get next event
                match event_stream.next().await {
                    Some(event_result) => {
                        match event_result {
                            Ok(event) => {
                                // Append non-partial events to session
                                if !event.partial
                                    && let Err(e) = session_service.append_event(&session_id_clone, event.clone()).await
                                {
                                    yield Err(e);
                                    return;
                                }

                                yield Ok(event);
                            }
                            Err(e) => {
                                yield Err(e);
                                return;
                            }
                        }
                    }
                    None => {
                        // Stream ended normally
                        return;
                    }
                }
            }
        })))
    }
}

pub struct RunnerBuilder {
    app_name: Option<String>,
    agent: Option<Arc<dyn Agent>>,
    session_service: Option<Arc<dyn SessionService>>,
}

impl RunnerBuilder {
    pub fn new() -> Self {
        Self {
            app_name: None,
            agent: None,
            session_service: None,
        }
    }

    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    pub fn agent(mut self, agent: Arc<dyn Agent>) -> Self {
        self.agent = Some(agent);
        self
    }

    pub fn session_service(mut self, service: Arc<dyn SessionService>) -> Self {
        self.session_service = Some(service);
        self
    }

    pub fn build(self) -> Result<Runner> {
        let app_name = self
            .app_name
            .ok_or_else(|| Error::Other(anyhow::anyhow!("App name is required")))?;
        let agent = self
            .agent
            .ok_or_else(|| Error::Other(anyhow::anyhow!("Agent is required")))?;
        let session_service = self
            .session_service
            .ok_or_else(|| Error::Other(anyhow::anyhow!("Session service is required")))?;

        Ok(Runner {
            app_name,
            agent,
            session_service,
        })
    }
}

impl Default for RunnerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RunConfig {
    pub streaming: bool,
}
