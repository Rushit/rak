use super::*;
use std::collections::HashMap as StdHashMap;
use std::sync::RwLock;
use uuid::Uuid;
use zdk_core::Error;

pub struct InMemorySessionService {
    sessions: Arc<RwLock<StdHashMap<String, Arc<InMemorySession>>>>,
}

impl InMemorySessionService {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(StdHashMap::new())),
        }
    }
}

impl Default for InMemorySessionService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionService for InMemorySessionService {
    async fn get(&self, req: &GetRequest) -> Result<Arc<dyn Session>> {
        let sessions = self.sessions.read().unwrap();
        sessions
            .get(&req.session_id)
            .cloned()
            .map(|s| s as Arc<dyn Session>)
            .ok_or_else(|| Error::SessionError(format!("Session {} not found", req.session_id)))
    }

    async fn create(&self, req: &CreateRequest) -> Result<Arc<dyn Session>> {
        let session_id = req
            .session_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let session = Arc::new(InMemorySession {
            id: session_id.clone(),
            app_name: req.app_name.clone(),
            user_id: req.user_id.clone(),
            events: RwLock::new(Vec::new()),
            state: RwLock::new(HashMap::new()),
        });

        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id, session.clone());

        Ok(session as Arc<dyn Session>)
    }

    async fn append_event(&self, session_id: &str, event: Event) -> Result<()> {
        let sessions = self.sessions.read().unwrap();
        if let Some(session) = sessions.get(session_id) {
            let mut events = session.events.write().unwrap();
            events.push(event);
            Ok(())
        } else {
            Err(Error::SessionError(format!(
                "Session {} not found",
                session_id
            )))
        }
    }
}

pub struct InMemorySession {
    id: String,
    app_name: String,
    user_id: String,
    events: RwLock<Vec<Event>>,
    state: RwLock<HashMap<String, serde_json::Value>>,
}

impl Session for InMemorySession {
    fn id(&self) -> &str {
        &self.id
    }

    fn app_name(&self) -> &str {
        &self.app_name
    }

    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn events(&self) -> Vec<Event> {
        self.events.read().unwrap().clone()
    }

    fn state(&self) -> HashMap<String, serde_json::Value> {
        self.state.read().unwrap().clone()
    }
}
