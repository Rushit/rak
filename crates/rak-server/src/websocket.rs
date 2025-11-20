//! WebSocket handler for bidirectional communication

use crate::rest::AppState;
use crate::ws_types::{WsClientMessage, WsServerMessage};
use rak_core::Content;
use rak_runner::RunConfig;
use axum::{
    extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tracing::{error, info, warn};

/// WebSocket handler for agent interaction
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket connection established");

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WsClientMessage>(&text) {
                    Ok(WsClientMessage::Run {
                        session_id,
                        new_message,
                    }) => {
                        handle_run(&session_id, new_message, &state, &mut sender).await;
                    }
                    Ok(WsClientMessage::Cancel { invocation_id }) => {
                        handle_cancel(&invocation_id, &state, &mut sender).await;
                    }
                    Ok(WsClientMessage::Status { invocation_id }) => {
                        handle_status(&invocation_id, &state, &mut sender).await;
                    }
                    Err(e) => {
                        send_error(&mut sender, format!("Invalid message format: {}", e)).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed by client");
                break;
            }
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                // Handle ping/pong for keep-alive
            }
            Ok(Message::Binary(_)) => {
                send_error(&mut sender, "Binary messages not supported".to_string()).await;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket connection closed");
}

/// Handle a run command
async fn handle_run(
    session_id: &str,
    new_message: Content,
    state: &AppState,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) {
    // Register invocation
    let (invocation_id, cancel_token) = state.invocation_tracker.register();

    // Send started message
    let started_msg = WsServerMessage::Started {
        invocation_id: invocation_id.clone(),
    };
    if let Err(e) = send_message(sender, &started_msg).await {
        error!("Failed to send started message: {}", e);
        state.invocation_tracker.unregister(&invocation_id);
        return;
    }

    // Get user_id from session (simplified - in production, should be properly authenticated)
    let user_id = "user".to_string(); // TODO: Get from session or auth

    // Run agent with cancellation support
    let config = RunConfig { streaming: true };
    let event_stream = match state
        .runner
        .run_with_cancellation(
            user_id,
            session_id.to_string(),
            new_message,
            config,
            Some(cancel_token),
        )
        .await
    {
        Ok(stream) => stream,
        Err(e) => {
            send_error(sender, format!("Failed to run agent: {}", e)).await;
            state.invocation_tracker.unregister(&invocation_id);
            return;
        }
    };

    // Stream events
    let mut pinned_stream = Box::pin(event_stream);
    while let Some(event_result) = pinned_stream.next().await {
        match event_result {
            Ok(event) => {
                let event_msg = WsServerMessage::Event {
                    invocation_id: invocation_id.clone(),
                    data: event,
                };
                if let Err(e) = send_message(sender, &event_msg).await {
                    error!("Failed to send event: {}", e);
                    break;
                }
            }
            Err(e) => {
                send_error(sender, format!("Agent error: {}", e)).await;
                break;
            }
        }
    }

    // Send completed message
    let completed_msg = WsServerMessage::Completed {
        invocation_id: invocation_id.clone(),
    };
    if let Err(e) = send_message(sender, &completed_msg).await {
        error!("Failed to send completed message: {}", e);
    }

    // Mark as complete and unregister
    state.invocation_tracker.complete(&invocation_id);
}

/// Handle a cancel command
async fn handle_cancel(
    invocation_id: &str,
    state: &AppState,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) {
    if state.invocation_tracker.cancel(invocation_id) {
        info!("Invocation {} cancelled", invocation_id);
        let msg = WsServerMessage::Cancelled {
            invocation_id: invocation_id.to_string(),
        };
        if let Err(e) = send_message(sender, &msg).await {
            error!("Failed to send cancelled message: {}", e);
        }
    } else {
        warn!("Attempted to cancel unknown invocation: {}", invocation_id);
        send_error(
            sender,
            format!("Invocation {} not found or already completed", invocation_id),
        )
        .await;
    }
}

/// Handle a status query
async fn handle_status(
    invocation_id: &str,
    state: &AppState,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) {
    let status = state.invocation_tracker.status(invocation_id);
    let msg = WsServerMessage::Status {
        invocation_id: invocation_id.to_string(),
        status,
    };
    if let Err(e) = send_message(sender, &msg).await {
        error!("Failed to send status message: {}", e);
    }
}

/// Send an error message
async fn send_error(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    message: String,
) {
    let error_msg = WsServerMessage::Error { message };
    if let Err(e) = send_message(sender, &error_msg).await {
        error!("Failed to send error message: {}", e);
    }
}

/// Send a WebSocket message
async fn send_message(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: &WsServerMessage,
) -> Result<(), axum::Error> {
    let json = serde_json::to_string(msg).map_err(|e| {
        error!("Failed to serialize message: {}", e);
        axum::Error::new(e)
    })?;
    sender.send(Message::Text(json)).await
}

