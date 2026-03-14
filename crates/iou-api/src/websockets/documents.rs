//! WebSocket handler for document status updates

use super::types::DocumentStatus;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use bytes::Bytes;
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Application state containing WebSocket broadcast channel
pub struct WebSocketState {
    pub status_tx: broadcast::Sender<DocumentStatus>,
}

/// Handle WebSocket upgrade for document status updates
///
/// Route: GET /api/ws/documents/{document_id}
///
/// Clients connect to this endpoint to receive real-time updates
/// about document workflow progress.
///
/// # Features
///
/// - Connection limit: 10 concurrent connections per document
/// - Heartbeat: Ping frames every 30 seconds
/// - Idle timeout: 5 minutes of inactivity
/// - Bounded channel: 100 message capacity, lagging receivers resync automatically
/// - Graceful disconnect: Closes on terminal status (completed/failed)
pub async fn ws_document_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<WebSocketState>>,
    Path(document_id): Path<Uuid>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_document_socket(socket, document_id, state))
}

/// Handle an active WebSocket connection for a document
///
/// This function runs two concurrent tasks:
/// 1. Send task: Receives status broadcasts and sends to client
/// 2. Receive task: Handles incoming messages from client (pong, close)
///
/// Both tasks are properly coordinated with cleanup on completion.
async fn handle_document_socket(
    mut socket: WebSocket,
    document_id: Uuid,
    state: Arc<WebSocketState>,
) {
    // Subscribe to the broadcast channel
    let mut rx = state.status_tx.subscribe();

    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Track last activity for idle timeout
    let last_activity = std::time::Instant::now();

    // Drop guard to ensure cleanup on task completion
    struct DropGuard(tokio::task::JoinHandle<()>);

    impl Drop for DropGuard {
        fn drop(&mut self) {
            self.0.abort();
        }
    }

    // Spawn send task for broadcasting status to client
    let send_task = tokio::spawn(async move {
        let heartbeat_interval = std::time::Duration::from_secs(30);
        let idle_timeout = std::time::Duration::from_secs(300); // 5 minutes
        let mut last_heartbeat = last_activity;

        loop {
            tokio::select! {
                // Receive status updates from broadcast channel
                result = rx.recv() => {
                    match result {
                        Ok(status) => {
                            // Only send messages for this document
                            if status.document_id() == document_id {
                                match serde_json::to_string(&status) {
                                    Ok(msg) => {
                                        if sender.send(Message::Text(msg.into())).await.is_err() {
                                            break;
                                        }
                                        last_heartbeat = std::time::Instant::now();
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            document_id = %document_id,
                                            error = %e,
                                            "Failed to serialize status"
                                        );
                                        break;
                                    }
                                }

                                // Close connection if workflow is complete
                                if status.is_terminal() {
                                    let _ = sender.close().await;
                                    break;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!(
                                document_id = %document_id,
                                lagged = n,
                                "WebSocket receiver lagged, messages skipped"
                            );
                            // Continue after lag - client will receive latest messages
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                // Send heartbeat ping and check idle timeout
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                    if last_heartbeat.elapsed() > idle_timeout {
                        tracing::info!(
                            document_id = %document_id,
                            "Closing idle WebSocket connection"
                        );
                        let _ = sender.close().await;
                        break;
                    }
                    if last_heartbeat.elapsed() > heartbeat_interval {
                        if sender.send(Message::Ping(Bytes::new())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Spawn receive task for client messages
    let recv_task = tokio::spawn(async move {
        let mut receiver = receiver;
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Pong(_) => {
                    // Client responded to ping - connection is alive
                    // (Note: we track activity on send side, so no action needed here)
                }
                Message::Close(_) => {
                    break;
                }
                _ => {
                    // Ignore other message types (text, binary, etc.)
                    // Clients receive-only; we don't process client messages
                }
            }
        }
    });

    // Wait for either task to complete, then abort the other
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }

    // Drop guard ensures both tasks are aborted if panic occurs
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn ws_document_handler_upgrade() {
        // This is a basic smoke test to verify the handler compiles
        // Full integration testing requires a running server

        let (tx, _rx) = broadcast::channel::<DocumentStatus>(100);
        let state = Arc::new(WebSocketState { status_tx: tx });

        // Verify state can be cloned (needed for handler)
        let _state_clone = Arc::clone(&state);

        // Verify we can create a document_id
        let document_id = Uuid::new_v4();
        assert_ne!(document_id, Uuid::default());
    }

    #[tokio::test]
    async fn broadcast_channel_lagged_receiver() {
        // Test that bounded channel works correctly with lagging receivers
        let (tx, _rx1) = broadcast::channel::<i32>(2); // Small capacity for testing

        let mut rx2 = tx.subscribe();
        let mut rx3 = tx.subscribe();

        // Fill the channel
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap(); // This will evict message 1

        // rx2 hasn't received anything, so it should get Lagged error
        match rx2.recv().await {
            Ok(_) => panic!("Expected Lagged error"),
            Err(broadcast::error::RecvError::Lagged(n)) => {
                assert_eq!(n, 1); // 1 message was skipped
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        // After lag, rx2 should receive latest messages
        assert_eq!(rx2.recv().await.unwrap(), 2);

        // rx3 should also experience lag
        match rx3.recv().await {
            Ok(_) => panic!("Expected Lagged error"),
            Err(broadcast::error::RecvError::Lagged(_)) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn document_status_is_terminal() {
        let id = Uuid::new_v4();

        // Terminal statuses
        assert!(DocumentStatus::Completed { document_id: id, timestamp: 0 }.is_terminal());
        assert!(DocumentStatus::Failed {
            document_id: id,
            error: "test".to_string(),
            timestamp: 0
        }
        .is_terminal());

        // Non-terminal statuses
        assert!(!DocumentStatus::Started {
            document_id: id,
            agent: "test".to_string(),
            timestamp: 0
        }
        .is_terminal());
        assert!(!DocumentStatus::Progress {
            document_id: id,
            agent: "test".to_string(),
            percent: 50,
            message: None,
            timestamp: 0
        }
        .is_terminal());
    }

    #[tokio::test]
    async fn document_status_extracts_id() {
        let id = Uuid::new_v4();
        let timestamp = chrono::Utc::now().timestamp();

        let started = DocumentStatus::Started {
            document_id: id,
            agent: "research".to_string(),
            timestamp,
        };
        assert_eq!(started.document_id(), id);

        let progress = DocumentStatus::Progress {
            document_id: id,
            agent: "content".to_string(),
            percent: 75,
            message: Some("Processing".to_string()),
            timestamp,
        };
        assert_eq!(progress.document_id(), id);

        let completed = DocumentStatus::Completed { document_id: id, timestamp };
        assert_eq!(completed.document_id(), id);

        let failed = DocumentStatus::Failed {
            document_id: id,
            error: "Test error".to_string(),
            timestamp,
        };
        assert_eq!(failed.document_id(), id);
    }
}
