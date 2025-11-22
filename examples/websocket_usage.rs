//! Example demonstrating WebSocket client for agent interaction
//!
//! This example shows how to:
//! - Automatically start a RAK server with WebSocket support
//! - Connect to the WebSocket endpoint
//! - Send run commands
//! - Receive streamed events
//! - Cancel running invocations (demonstrated in code)
//! - Gracefully shut down
//!
//! ## Running This Example
//!
//! Just run it - the server starts automatically:
//! ```bash
//! cargo run --example websocket_usage
//! ```
//!
//! The example will:
//! 1. Start a temporary server on port 18080
//! 2. Connect to it via WebSocket
//! 3. Send a message and receive responses
//! 4. Clean up and stop the server

use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use rak_agent::LLMAgent;
use rak_core::Content;
use rak_runner::Runner;
use rak_server::{rest::create_router, WsClientMessage, WsServerMessage};
use rak_session::inmemory::InMemorySessionService;
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║       RAK WebSocket Example (Self-Contained)             ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Start server in background
    println!("1. Starting temporary server...");
    let server_handle = tokio::spawn(start_server());
    
    // Wait for server to be ready
    println!("2. Waiting for server to be ready...");
    sleep(Duration::from_secs(2)).await;
    
    // Check if server is healthy
    if !check_server_health().await {
        anyhow::bail!("Server failed to start properly");
    }
    println!("   ✓ Server is ready\n");

    // Run the WebSocket client
    println!("3. Running WebSocket client...\n");
    let client_result = run_websocket_client().await;

    // Give server time to finish processing
    sleep(Duration::from_millis(500)).await;
    
    // Stop server
    println!("\n4. Stopping server...");
    server_handle.abort();
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║                    Example Complete                       ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    println!("Key takeaways:");
    println!("  • WebSocket enables bidirectional communication");
    println!("  • Server started automatically for this demo");
    println!("  • Real apps would run server separately");
    println!("  • Run command starts agent execution");
    println!("  • Events are streamed in real-time\n");

    client_result
}

/// Start the RAK server with WebSocket support
async fn start_server() -> Result<()> {
    // Load configuration
    let config = common::load_config()?;

    // Create model
    let model = common::create_gemini_model(&config)?;

    // Create agent
    let agent = Arc::new(
        LLMAgent::builder()
            .name("assistant")
            .model(model)
            .system_instruction("You are a helpful AI assistant. Be concise.")
            .build()?,
    );

    // Create session service
    let session_service = Arc::new(InMemorySessionService::new());

    // Create runner
    let runner = Arc::new(
        Runner::builder()
            .app_name("websocket-demo")
            .agent(agent)
            .session_service(session_service.clone())
            .build()?,
    );

    // Create router
    let app = create_router(runner, session_service);

    // Start server
    let addr = "127.0.0.1:18080";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app).await?;

    Ok(())
}

/// Check if server is healthy by attempting to connect to the port
async fn check_server_health() -> bool {
    for _ in 0..10 {
        if tokio::net::TcpStream::connect("127.0.0.1:18080").await.is_ok() {
            return true;
        }
        sleep(Duration::from_millis(500)).await;
    }
    false
}

/// Run the WebSocket client
async fn run_websocket_client() -> Result<()> {
    // Connect to WebSocket endpoint
    let url = "ws://127.0.0.1:18080/api/v1/sessions/demo-session/run/ws";
    println!("   → Connecting to WebSocket: {}", url);

    let (ws_stream, _) = connect_async(url)
        .await
        .context("Failed to connect to WebSocket")?;
    
    println!("   ✓ Connected to WebSocket\n");

    let (mut write, mut read) = ws_stream.split();

    // Spawn task to receive messages
    let read_handle = tokio::spawn(async move {
        println!("   → Listening for messages...\n");
        
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsServerMessage>(&text) {
                        Ok(server_msg) => match server_msg {
                            WsServerMessage::Started { invocation_id } => {
                                println!("   ✓ Invocation started: {}", invocation_id);
                            }
                            WsServerMessage::Event {
                                invocation_id: _,
                                data,
                            } => {
                                if let Some(content) = &data.content {
                                    for part in &content.parts {
                                        if let rak_core::Part::Text { text } = part {
                                            print!("   📨 {}: {}", data.author, text);
                                            std::io::Write::flush(&mut std::io::stdout()).ok();
                                        }
                                    }
                                }
                                if data.turn_complete {
                                    println!();
                                }
                            }
                            WsServerMessage::Completed { invocation_id } => {
                                println!("\n   ✓ Invocation completed: {}", invocation_id);
                            }
                            WsServerMessage::Cancelled { invocation_id } => {
                                println!("   ⚠️  Invocation cancelled: {}", invocation_id);
                            }
                            WsServerMessage::Status {
                                invocation_id,
                                status,
                            } => {
                                println!("   ℹ️  Status for {}: {:?}", invocation_id, status);
                            }
                            WsServerMessage::Error { message } => {
                                println!("   ❌ Error: {}", message);
                            }
                        },
                        Err(e) => {
                            eprintln!("   ⚠️  Failed to parse message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("   → WebSocket closed");
                    break;
                }
                Err(e) => {
                    eprintln!("   ❌ WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Give read task time to start
    sleep(Duration::from_millis(100)).await;

    // Send a run command
    println!("   → Sending message: 'Explain what WebSockets are in one sentence'");
    let run_msg = WsClientMessage::Run {
        session_id: "demo-session".to_string(),
        new_message: Content::new_user_text("Explain what WebSockets are in one sentence"),
    };

    let json = serde_json::to_string(&run_msg)?;
    write.send(Message::Text(json.into())).await?;

    // Wait for responses
    sleep(Duration::from_secs(5)).await;

    // Close connection
    println!("\n   → Closing WebSocket connection");
    write.send(Message::Close(None)).await?;

    // Wait for read task to finish
    let _ = tokio::time::timeout(Duration::from_secs(2), read_handle).await;

    Ok(())
}
