use crate::invocation_tracker::InvocationTracker;
use crate::types::*;
use crate::websocket::ws_handler;
use rak_runner::{RunConfig, Runner};
use rak_session::{CreateRequest, SessionService};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{
        sse::{Event as SseEvent, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Router,
};
use futures::stream::{Stream, StreamExt};
use std::convert::Infallible;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[derive(Clone)]
pub struct AppState {
    pub runner: Arc<Runner>,
    pub session_service: Arc<dyn SessionService>,
    pub invocation_tracker: Arc<InvocationTracker>,
}

pub fn create_router(runner: Arc<Runner>, session_service: Arc<dyn SessionService>) -> Router {
    let state = AppState {
        runner,
        session_service,
        invocation_tracker: Arc::new(InvocationTracker::new()),
    };

    Router::new()
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/readiness", get(readiness_check))
        // API endpoints
        .route("/api/v1/sessions", post(create_session))
        .route("/api/v1/sessions/:id/run", post(run_agent_batch))
        .route("/api/v1/sessions/:id/run/sse", post(run_agent_sse))
        .route("/api/v1/sessions/:id/run/ws", get(ws_handler))
        // Middleware layers (applied in reverse order)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Health check endpoint - returns OK if the service is running
async fn health_check() -> impl IntoResponse {
    tracing::debug!("Health check requested");
    (StatusCode::OK, "OK")
}

/// Readiness check endpoint - verifies service dependencies
async fn readiness_check(State(_state): State<AppState>) -> impl IntoResponse {
    // Check if services are available
    // For now, just return OK as services are injected at startup
    tracing::debug!("Readiness check requested");
    
    // Could add checks like:
    // - Database connectivity (if using database session service)
    // - Runner availability
    // - etc.
    
    (StatusCode::OK, "READY")
}

async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, AppError> {
    let session = state
        .session_service
        .create(&CreateRequest {
            app_name: req.app_name.clone(),
            user_id: req.user_id.clone(),
            session_id: req.session_id,
        })
        .await?;

    Ok(Json(CreateSessionResponse {
        session_id: session.id().to_string(),
        app_name: session.app_name().to_string(),
        user_id: session.user_id().to_string(),
    }))
}

async fn run_agent_batch(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<RunAgentRequest>,
) -> Result<Json<RunAgentResponse>, AppError> {
    // Extract user_id from session (simplified - in production, parse from session)
    let user_id = "user".to_string(); // TODO: Get from session

    let config = RunConfig { streaming: false };

    let mut event_stream = state
        .runner
        .run(user_id, session_id, req.new_message, config)
        .await?;

    let mut events = Vec::new();
    while let Some(event_result) = event_stream.next().await {
        match event_result {
            Ok(event) => events.push(event),
            Err(e) => return Err(AppError::from(e)),
        }
    }

    Ok(Json(RunAgentResponse { events }))
}

async fn run_agent_sse(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<RunAgentRequest>,
) -> Result<Sse<impl Stream<Item = Result<SseEvent, Infallible>>>, AppError> {
    // Extract user_id from session (simplified)
    let user_id = "user".to_string(); // TODO: Get from session

    let config = RunConfig { streaming: true };

    let event_stream = state
        .runner
        .run(user_id, session_id, req.new_message, config)
        .await?;

    let sse_stream = event_stream.map(|event_result| match event_result {
        Ok(event) => {
            let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
            Ok(SseEvent::default().data(json))
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": e.to_string()
            });
            Ok(SseEvent::default().data(error_json.to_string()))
        }
    });

    Ok(Sse::new(sse_stream))
}

// Error handling
pub struct AppError(anyhow::Error);

impl From<rak_core::Error> for AppError {
    fn from(err: rak_core::Error) -> Self {
        AppError(err.into())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_message = self.0.to_string();
        let json = serde_json::json!({
            "error": error_message
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json)).into_response()
    }
}
