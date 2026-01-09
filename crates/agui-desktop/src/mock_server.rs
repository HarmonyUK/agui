//! Mock WebSocket server for development and testing
//!
//! Simulates the orchestrator server by sending AG-UI protocol events
//! to connected clients.

use crate::protocol::*;
use anyhow::Result;
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

/// Mock server state
#[derive(Clone)]
pub struct MockServer {
    /// Broadcast channel for sending events to all connected clients
    broadcast_tx: broadcast::Sender<EventEnvelope>,
    /// Server configuration
    config: Arc<MockServerConfig>,
}

/// Mock server configuration
#[derive(Debug, Clone)]
pub struct MockServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Enable demo event streaming
    pub auto_stream: bool,
    /// Event stream interval in milliseconds
    pub stream_interval_ms: u64,
}

impl Default for MockServerConfig {
    fn default() -> Self {
        Self {
            port: 3001,
            auto_stream: true,
            stream_interval_ms: 2000,
        }
    }
}

impl MockServer {
    /// Create a new mock server
    pub fn new(config: MockServerConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            broadcast_tx,
            config: Arc::new(config),
        }
    }

    /// Start the mock server (blocking)
    pub async fn run(&self) -> Result<()> {
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(self.clone());

        let addr = format!("127.0.0.1:{}", self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!("Mock server listening on ws://{}/ws", addr);

        let server = self.clone();
        if self.config.auto_stream {
            tokio::spawn(async move {
                server.stream_demo_events().await;
            });
        }

        axum::serve(listener, app).await?;
        Ok(())
    }

    /// Stream demo events to connected clients
    async fn stream_demo_events(&self) {
        let interval_ms = self.config.stream_interval_ms;
        let mut counter = 0u32;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;

            let event = match counter % 5 {
                0 => {
                    Event::TextMessage(TextMessage {
                        sender: "agent_demo".to_string(),
                        content: format!("Demo message #{}", counter),
                        metadata: None,
                    })
                }
                1 => {
                    Event::StateDelta(StateDelta {
                        path: format!("state.counter.{}", counter),
                        old_value: Some(serde_json::json!(counter - 1)),
                        new_value: serde_json::json!(counter),
                    })
                }
                2 => {
                    Event::ToolCallRequest(ToolCallRequest {
                        id: format!("tool_call_{}", counter),
                        tool_name: "demo_tool".to_string(),
                        parameters: serde_json::json!({"param": counter}),
                        agent_id: "agent_demo".to_string(),
                    })
                }
                3 => {
                    Event::PlanCard(PlanCard {
                        id: format!("plan_{}", counter),
                        title: "Demo Plan".to_string(),
                        content: format!("This is demo plan iteration #{}", counter),
                        status: CardStatus::Active,
                    })
                }
                _ => {
                    Event::ConnectionStatus(ConnectionStatus {
                        status: ConnectionState::Connected,
                        message: Some(format!("Heartbeat {}", counter)),
                    })
                }
            };

            let envelope = EventEnvelope::new(event);
            let _ = self.broadcast_tx.send(envelope);

            counter += 1;
        }
    }

    /// Get the broadcast sender for sending events
    pub fn sender(&self) -> broadcast::Sender<EventEnvelope> {
        self.broadcast_tx.clone()
    }
}

/// WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    info!("New WebSocket connection");

    // For now, just echo received messages
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                axum::extract::ws::Message::Text(text) => {
                    // Try to parse as JSON protocol event
                    match serde_json::from_str::<EventEnvelope>(&text) {
                        Ok(envelope) => {
                            info!("Received event: {:?}", envelope.event);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse event: {}", e);
                            let error = ErrorEvent {
                                code: "PARSE_ERROR".to_string(),
                                message: e.to_string(),
                                details: None,
                            };
                            let error_envelope = EventEnvelope::new(Event::Error(error));
                            if let Ok(json) = serde_json::to_string(&error_envelope) {
                                let _ = sender.send(axum::extract::ws::Message::Text(json)).await;
                            }
                        }
                    }
                }
                axum::extract::ws::Message::Binary(_) => {
                    tracing::warn!("Received binary message, not supported");
                }
                axum::extract::ws::Message::Ping(_) => {
                    let _ = sender.send(axum::extract::ws::Message::Pong(vec![])).await;
                }
                axum::extract::ws::Message::Pong(_) => {}
                axum::extract::ws::Message::Close(_) => {
                    info!("WebSocket connection closed");
                    break;
                }
            }
        } else {
            error!("WebSocket error");
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_creation() {
        let config = MockServerConfig::default();
        let server = MockServer::new(config);
        assert_eq!(server.config.port, 3001);
        assert!(server.config.auto_stream);
    }

    #[test]
    fn test_custom_config() {
        let config = MockServerConfig {
            port: 8080,
            auto_stream: false,
            stream_interval_ms: 1000,
        };
        let server = MockServer::new(config);
        assert_eq!(server.config.port, 8080);
        assert!(!server.config.auto_stream);
    }
}
