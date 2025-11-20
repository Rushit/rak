use rak_core::{Agent, Content, InvocationContext, ReadonlyContext};
use async_trait::async_trait;
use std::sync::Arc;

pub struct DefaultInvocationContext {
    invocation_id: String,
    app_name: String,
    user_id: String,
    session_id: String,
    user_content: Option<Content>,
    agent: Arc<dyn Agent>,
}

impl DefaultInvocationContext {
    pub fn new(
        invocation_id: String,
        app_name: String,
        user_id: String,
        session_id: String,
        user_content: Option<Content>,
        agent: Arc<dyn Agent>,
    ) -> Self {
        Self {
            invocation_id,
            app_name,
            user_id,
            session_id,
            user_content,
            agent,
        }
    }
}

#[async_trait]
impl InvocationContext for DefaultInvocationContext {
    fn invocation_id(&self) -> &str {
        &self.invocation_id
    }

    fn user_content(&self) -> Option<&Content> {
        self.user_content.as_ref()
    }
}

impl ReadonlyContext for DefaultInvocationContext {
    fn app_name(&self) -> &str {
        &self.app_name
    }

    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn session_id(&self) -> &str {
        &self.session_id
    }
}
