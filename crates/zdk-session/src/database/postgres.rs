//! PostgreSQL-backed session service

use super::models::{AppStateRow, EventRow, SessionRow, UserStateRow};
use crate::{CreateRequest, GetRequest, Session, SessionService};
use zdk_core::{Error as ZdkError, Event, Result as ZdkResult};
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// PostgreSQL-backed session service
pub struct PostgresSessionService {
    pool: Pool<Postgres>,
}

impl PostgresSessionService {
    /// Create a new PostgreSQL session service
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        // Run migrations
        super::migrations::run_postgres_migrations(&pool).await?;
        super::migrations::create_events_index_postgres(&pool).await?;

        Ok(Self { pool })
    }

    /// Create from an existing pool
    pub fn from_pool(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Get or create app state
    async fn get_or_create_app_state(
        &self,
        app_name: &str,
    ) -> Result<HashMap<String, serde_json::Value>, sqlx::Error> {
        let row: Option<AppStateRow> =
            sqlx::query_as("SELECT * FROM app_states WHERE app_name = $1")
                .bind(app_name)
                .fetch_optional(&self.pool)
                .await?;

        if let Some(row) = row {
            serde_json::from_str(&row.state).map_err(|e| sqlx::Error::Decode(Box::new(e)))
        } else {
            // Create new app state
            sqlx::query(
                "INSERT INTO app_states (app_name, state) VALUES ($1, $2) ON CONFLICT (app_name) DO NOTHING"
            )
            .bind(app_name)
            .bind("{}")
            .execute(&self.pool)
            .await?;

            Ok(HashMap::new())
        }
    }

    /// Get or create user state
    async fn get_or_create_user_state(
        &self,
        app_name: &str,
        user_id: &str,
    ) -> Result<HashMap<String, serde_json::Value>, sqlx::Error> {
        let row: Option<UserStateRow> =
            sqlx::query_as("SELECT * FROM user_states WHERE app_name = $1 AND user_id = $2")
                .bind(app_name)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

        if let Some(row) = row {
            serde_json::from_str(&row.state).map_err(|e| sqlx::Error::Decode(Box::new(e)))
        } else {
            // Create new user state
            sqlx::query(
                "INSERT INTO user_states (app_name, user_id, state) VALUES ($1, $2, $3) ON CONFLICT (app_name, user_id) DO NOTHING"
            )
            .bind(app_name)
            .bind(user_id)
            .bind("{}")
            .execute(&self.pool)
            .await?;

            Ok(HashMap::new())
        }
    }

    /// Merge app, user, and session state
    fn merge_states(
        app_state: HashMap<String, serde_json::Value>,
        user_state: HashMap<String, serde_json::Value>,
        session_state: HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        let mut result = app_state;
        result.extend(user_state);
        result.extend(session_state);
        result
    }
}

/// Database-backed session implementation
struct DatabaseSession {
    id: String,
    app_name: String,
    user_id: String,
    events: Vec<Event>,
    state: HashMap<String, serde_json::Value>,
}

impl Session for DatabaseSession {
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
        self.events.clone()
    }

    fn state(&self) -> HashMap<String, serde_json::Value> {
        self.state.clone()
    }
}

#[async_trait]
impl SessionService for PostgresSessionService {
    async fn get(&self, req: &GetRequest) -> ZdkResult<Arc<dyn Session>> {
        // Fetch session
        let session_row: SessionRow = sqlx::query_as(
            "SELECT * FROM sessions WHERE app_name = $1 AND user_id = $2 AND id = $3",
        )
        .bind(&req.app_name)
        .bind(&req.user_id)
        .bind(&req.session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ZdkError::Other(anyhow!("Failed to fetch session: {}", e)))?;

        // Parse session state
        let session_state: HashMap<String, serde_json::Value> =
            serde_json::from_str(&session_row.state)
                .map_err(|e| ZdkError::Other(anyhow!("Failed to parse session state: {}", e)))?;

        // Fetch app and user states
        let app_state = self
            .get_or_create_app_state(&req.app_name)
            .await
            .map_err(|e| ZdkError::Other(anyhow!("Failed to fetch app state: {}", e)))?;

        let user_state = self
            .get_or_create_user_state(&req.app_name, &req.user_id)
            .await
            .map_err(|e| ZdkError::Other(anyhow!("Failed to fetch user state: {}", e)))?;

        // Merge states
        let merged_state = Self::merge_states(app_state, user_state, session_state);

        // Fetch events
        let event_rows: Vec<EventRow> = sqlx::query_as(
            "SELECT * FROM events WHERE app_name = $1 AND user_id = $2 AND session_id = $3 ORDER BY timestamp ASC"
        )
        .bind(&req.app_name)
        .bind(&req.user_id)
        .bind(&req.session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ZdkError::Other(anyhow!("Failed to fetch events: {}", e)))?;

        let events: Result<Vec<Event>, _> = event_rows.iter().map(|row| row.to_event()).collect();

        let events =
            events.map_err(|e| ZdkError::Other(anyhow!("Failed to parse events: {}", e)))?;

        Ok(Arc::new(DatabaseSession {
            id: session_row.id,
            app_name: session_row.app_name,
            user_id: session_row.user_id,
            events,
            state: merged_state,
        }))
    }

    async fn create(&self, req: &CreateRequest) -> ZdkResult<Arc<dyn Session>> {
        let session_id = req
            .session_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Create session
        let state_json = "{}";
        sqlx::query(
            "INSERT INTO sessions (app_name, user_id, id, state) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING"
        )
        .bind(&req.app_name)
        .bind(&req.user_id)
        .bind(&session_id)
        .bind(state_json)
        .execute(&self.pool)
        .await
        .map_err(|e| ZdkError::Other(anyhow!("Failed to create session: {}", e)))?;

        // Ensure app and user states exist
        let app_state = self
            .get_or_create_app_state(&req.app_name)
            .await
            .map_err(|e| ZdkError::Other(anyhow!("Failed to create app state: {}", e)))?;

        let user_state = self
            .get_or_create_user_state(&req.app_name, &req.user_id)
            .await
            .map_err(|e| ZdkError::Other(anyhow!("Failed to create user state: {}", e)))?;

        let merged_state = Self::merge_states(app_state, user_state, HashMap::new());

        Ok(Arc::new(DatabaseSession {
            id: session_id,
            app_name: req.app_name.clone(),
            user_id: req.user_id.clone(),
            events: Vec::new(),
            state: merged_state,
        }))
    }

    async fn append_event(&self, session_id: &str, event: Event) -> ZdkResult<()> {
        // First, find the session to get app_name and user_id
        let session_row: (String, String) =
            sqlx::query_as("SELECT app_name, user_id FROM sessions WHERE id = $1 LIMIT 1")
                .bind(session_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ZdkError::Other(anyhow!("Failed to find session: {}", e)))?;

        let (app_name, user_id) = session_row;

        // Convert event to EventRow
        let event_row = EventRow::from_event(&event, &app_name, &user_id, session_id)
            .map_err(|e| ZdkError::Other(anyhow!("Failed to serialize event: {}", e)))?;

        // Insert event
        sqlx::query(
            r#"
            INSERT INTO events (
                id, app_name, user_id, session_id, invocation_id, author, actions,
                long_running_tool_ids, branch, timestamp, content, grounding_metadata,
                custom_metadata, usage_metadata, citation_metadata, partial, turn_complete,
                error_code, error_message, interrupted
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            "#
        )
        .bind(&event_row.id)
        .bind(&event_row.app_name)
        .bind(&event_row.user_id)
        .bind(&event_row.session_id)
        .bind(&event_row.invocation_id)
        .bind(&event_row.author)
        .bind(&event_row.actions)
        .bind(&event_row.long_running_tool_ids)
        .bind(&event_row.branch)
        .bind(&event_row.timestamp)
        .bind(&event_row.content)
        .bind(&event_row.grounding_metadata)
        .bind(&event_row.custom_metadata)
        .bind(&event_row.usage_metadata)
        .bind(&event_row.citation_metadata)
        .bind(&event_row.partial)
        .bind(&event_row.turn_complete)
        .bind(&event_row.error_code)
        .bind(&event_row.error_message)
        .bind(&event_row.interrupted)
        .execute(&self.pool)
        .await
        .map_err(|e| ZdkError::Other(anyhow!("Failed to insert event: {}", e)))?;

        Ok(())
    }
}
