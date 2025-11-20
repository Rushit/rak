//! Example demonstrating WebSocket usage for agent interaction
//!
//! This example shows how to:
//! - Connect to a WebSocket endpoint
//! - Send run commands
//! - Receive streamed events
//! - Cancel running invocations
//! - Query invocation status
//!
//! Run the server first:
//! ```bash
//! cargo run --example quickstart
//! ```
//!
//! Then run this example:
//! ```bash
//! cargo run --example websocket_usage
//! ```

use rak_core::Content;
use rak_server::{WsClientMessage, WsServerMessage};
use futures::{SinkExt, StreamExt};
use serde_json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== RAK WebSocket Client Example ===\n");

    // Connect to WebSocket endpoint
    let url = "ws://127.0.0.1:8080/api/v1/sessions/test-session/run/ws";
    println!("Connecting to: {}", url);

    let (ws_stream, _) = connect_async(url).await?;
    println!("✓ Connected to WebSocket\n");

    let (mut write, mut read) = ws_stream.split();

    // Spawn task to receive messages
    let read_handle = tokio::spawn(async move {
        println!("--- Listening for messages ---");
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WsServerMessage>(&text) {
                        Ok(server_msg) => match server_msg {
                            WsServerMessage::Started { invocation_id } => {
                                println!("✓ Invocation started: {}", invocation_id);
                            }
                            WsServerMessage::Event {
                                invocation_id,
                                data,
                            } => {
                                println!("  Event from {}: {:?}", invocation_id, data.author);
                                if let Some(content) = &data.content {
                                    for part in &content.parts {
                                        if let rak_core::Part::Text { text } = part {
                                            println!("    Text: {}", text);
                                        }
                                    }
                                }
                            }
                            WsServerMessage::Completed { invocation_id } => {
                                println!("✓ Invocation completed: {}", invocation_id);
                            }
                            WsServerMessage::Cancelled { invocation_id } => {
                                println!("✓ Invocation cancelled: {}", invocation_id);
                            }
                            WsServerMessage::Status {
                                invocation_id,
                                status,
                            } => {
                                println!("  Status for {}: {:?}", invocation_id, status);
                            }
                            WsServerMessage::Error { message } => {
                                println!("✗ Error: {}", message);
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to parse server message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("WebSocket closed");
                    break;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Give read task time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Send a run command
    println!("\n--- Sending run command ---");
    let run_msg = WsClientMessage::Run {
        session_id: "test-session".to_string(),
        new_message: Content::new_user_text("Hello, how are you?"),
    };

    let json = serde_json::to_string(&run_msg)?;
    write.send(Message::Text(json.into())).await?;
    println!("✓ Sent run command\n");

    // Wait for responses
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Example: Send a cancel command (uncomment to test)
    /*
    println!("\n--- Sending cancel command ---");
    let cancel_msg = WsClientMessage::Cancel {
        invocation_id: "some-invocation-id".to_string(),
    };
    let json = serde_json::to_string(&cancel_msg)?;
    write.send(Message::Text(json)).await?;
    println!("✓ Sent cancel command\n");
    */

    // Example: Send a status query (uncomment to test)
    /*
    println!("\n--- Sending status query ---");
    let status_msg = WsClientMessage::Status {
        invocation_id: "some-invocation-id".to_string(),
    };
    let json = serde_json::to_string(&status_msg)?;
    write.send(Message::Text(json)).await?;
    println!("✓ Sent status query\n");
    */

    // Wait a bit more
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Close connection
    println!("\n--- Closing connection ---");
    write.send(Message::Close(None)).await?;

    // Wait for read task to finish
    let _ = read_handle.await;

    println!("\n=== Example Complete ===");
    println!("\nKey takeaways:");
    println!("  • WebSocket enables bidirectional communication");
    println!("  • Run command starts agent execution and streams events");
    println!("  • Cancel command stops running invocations");
    println!("  • Status command queries invocation state");
    println!("  • Multiple invocations can run concurrently");

    Ok(())
}

