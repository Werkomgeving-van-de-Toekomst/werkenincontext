Now I have all the necessary context. Let me generate the section content for `section-05-websocket`.

# Section 5: WebSocket Support for Real-Time Document Status Updates

## Overview

This section implements real-time status updates for document generation workflows using WebSocket connections. Clients can subscribe to status updates for specific documents and receive instant notifications as the multi-agent pipeline progresses through various stages.

**Dependencies:**
- Section 1 (Foundation) - WebSocket module structure, AppState
- Section 2 (Orchestrator Integration) - Broadcast channel integration with workflow

**Files to Create:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/websockets/mod.rs` - WebSocket module definition
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/websockets/types.rs` - Message types for WebSocket communication
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/websockets/limiter.rs` - Per-document connection limiting
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/websockets/documents.rs` - Document-specific WebSocket handler

**Files to Modify:**
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs` - Add WebSocket route and update AppState
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/orchestrator/wrapper.rs` - Broadcast status updates to WebSocket subscribers
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/workflows/mod.rs` - Update AppState to include WebSocket components

## Architecture

```
Client WebSocket Connection
           |
           v
ws_document_handler (document_id)
           |
           +-- ConnectionLimiter (max 10 per document)
           |
           +-- broadcast::channel<DocumentStatus>(100)
                    |
                    +-- Orchestrator sends status updates
                    |
                    +-- WebSocket subscribers receive updates
```

### Key Design Decisions

1. **Bounded Broadcast Channel (capacity 100)**: Prevents unbounded memory growth if slow consumers cannot keep up with status updates. Lagging receivers receive a `Lagged` error and can resynchronize.

2. **Connection Limit (10 per document)**: Prevents resource exhaustion from too many simultaneous connections to a single document's status feed.

3. **Idle Timeout (5 minutes)**: Automatically closes connections that have been inactive for 5 minutes to prevent zombie connections.

4. **Heartbeat (30 seconds)**: Sends Ping frames every 30 seconds to detect dead connections early.

5. **Panic Recovery**: Send and receive tasks are wrapped in proper cleanup guards to ensure resources are released even if one task panics.

6. **Graceful Disconnect**: Connections close cleanly when the document workflow completes (success or failure).

## Tests

Write the following tests BEFORE implementing the WebSocket functionality.

### Test File Location
Create tests in `/Users/marc/Projecten/iou-modern/crates/iou-api/src/websockets/documents.rs` within a `#[cfg(test)]` module.

### WebSocket Handler Tests

```rust
#[tokio::test]
async fn ws_upgrade_succeeds() {
    // Given: A valid WebSocket upgrade request for a document
    // When: ws_document_handler is called
    // Then: Connection is upgraded (101 Switching Protocols)
    // And: Connection is registered in the limiter
}

#[tokio::test]
async fn ws_receives_status_broadcasts() {
    // Given: A connected WebSocket for document_id
    // When: A status broadcast is sent for that document_id
    // Then: WebSocket receives a JSON message
    // And: Message contains correct document_id
    // And: Message type matches the status (started/progress/completed/failed)
}

#[tokio::test]
async fn ws_ignores_other_document_broadcasts() {
    // Given: A connected WebSocket for document_id = A
    // When: Status broadcast is sent for document_id = B
    // Then: WebSocket does NOT receive the message
}
```

### Connection Limit Tests

```rust
#[tokio::test]
async fn tenth_connection_is_accepted() {
    // Given: ConnectionLimiter with 9 existing connections for document_id
    // When: 10th connection attempt is made
    // Then: Connection succeeds
    // And: Permit is acquired
}

#[tokio::test]
async fn eleventh_connection_is_rejected() {
    // Given: ConnectionLimiter with 10 existing connections for document_id
    // When: 11th connection attempt is made
    // Then: Connection fails
    // And: Returns ApiError::TooManyRequests
    // Or: Connection is closed immediately
}
```

### Bounded Channel Tests

```rust
#[tokio::test]
async fn lagging_receiver_receives_lagged_error() {
    // Given: Bounded channel with capacity 100
    // And: Receiver that has not consumed messages
    // When: 101st message is sent to the channel
    // Then: Receiver gets RecvError::Lagged(n) where n > 0
    // And: Receiver can still receive new messages after error
}

#[tokio::test]
async fn non_lagging_receives_all_messages() {
    // Given: Bounded channel with capacity 100
    // And: Active receiver consuming messages
    // When: 50 messages are sent
    // Then: Receiver receives all 50 messages
    // And: No Lagged error occurs
}
```

### Idle Timeout Tests

```rust
#[tokio::test]
async fn websocket_closes_after_five_minutes_inactive() {
    // Given: A connected WebSocket
    // And: No messages received for 5 minutes
    // When: Idle timeout check fires
    // Then: WebSocket is closed
    // And: Close frame is sent to client
}

#[tokio::test]
async fn heartbeat_keep_connection_alive() {
    // Given: A connected WebSocket
    // And: Ping/Pong exchanged every 30 seconds
    // When: 5 minutes pass with heartbeat activity
    // Then: Connection remains open
    // And: No idle timeout occurs
}
```

### Panic Recovery Tests

```rust
#[tokio::test]
async fn send_task_panic_does_not_crash_handler() {
    // Given: An active WebSocket connection
    // When: Send task panics (simulated)
    // Then: Receive task is aborted
    // And: Resources are cleaned up
    // And: No memory leak occurs
}

#[tokio::test]
async fn receive_task_panic_does_not_crash_handler() {
    // Given: An active WebSocket connection
    // When: Receive task panics (simulated)
    // Then: Send task is aborted
    // And: Resources are cleaned up
}
```

### Graceful Disconnect Tests

```rust
#[tokio::test]
async fn websocket_closes_on_completion() {
    // Given: A connected WebSocket for a document
    // When: Document workflow completes successfully
    // Then: Completion status is sent to client
    // And: WebSocket closes gracefully
}

#[tokio::test]
async fn websocket_closes_on_client_close_frame() {
    // Given: A connected WebSocket
    // When: Client sends Close frame
    // Then: Server acknowledges with Close frame
    // And: Connection terminates cleanly
}
```

## Implementation

### Step 1: Create WebSocket Types

File: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/websockets/types.rs`

```rust
//! WebSocket message types for document status updates

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status broadcast message sent to WebSocket subscribers
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DocumentStatus {
    /// Workflow has started
    Started {
        document_id: Uuid,
        agent: String,
        timestamp: i64,
    },
    /// Workflow progress update
    Progress {
        document_id: Uuid,
        agent: String,
        percent: u8,
        message: Option<String>,
        timestamp: i64,
    },
    /// Workflow completed successfully
    Completed {
        document_id: Uuid,
        timestamp: i64,
    },
    /// Workflow failed
    Failed {
        document_id: Uuid,
        error: String,
        timestamp: i64,
    },
}

impl DocumentStatus {
    /// Extract the document_id from any status variant
    pub fn document_id(&self) -> Uuid {
        match self {
            DocumentStatus::Started { document_id, .. } => *document_id,
            DocumentStatus::Progress { document_id, .. } => *document_id,
            DocumentStatus::Completed { document_id, .. } => *document_id,
            DocumentStatus::Failed { document_id, .. } => *document_id,
        }
    }

    /// Check if this status indicates workflow completion (success or failure)
    pub fn is_terminal(&self) -> bool {
        matches!(self, DocumentStatus::Completed { .. } | DocumentStatus::Failed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_status_serialization() {
        let status = DocumentStatus::Started {
            document_id: Uuid::new_v4(),
            agent: "research".to_string(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"type\":\"started\""));
        assert!(json.contains("research"));
    }

    #[test]
    fn test_document_status_extracts_id() {
        let id = Uuid::new_v4();
        let status = DocumentStatus::Progress {
            document_id: id,
            agent: "content".to_string(),
            percent: 50,
            message: None,
            timestamp: 0,
        };

        assert_eq!(status.document_id(), id);
    }

    #[test]
    fn test_terminal_status_detection() {
        let id = Uuid::new_v4();
        
        assert!(DocumentStatus::Completed { document_id: id, timestamp: 0 }.is_terminal());
        assert!(DocumentStatus::Failed { document_id: id, error: "err".to_string(), timestamp: 0 }.is_terminal());
        assert!(!DocumentStatus::Started { document_id: id, agent: "test".to_string(), timestamp: 0 }.is_terminal());
    }
}
```

### Step 2: Create Connection Limiter

File: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/websockets/limiter.rs`

```rust
//! Per-document WebSocket connection limiting

use crate::error::ApiError;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Limits concurrent WebSocket connections per document
pub struct ConnectionLimiter {
    /// Map of document_id -> semaphore (max 10 connections per document)
    limits: DashMap<Uuid, Arc<Semaphore>>,
}

impl ConnectionLimiter {
    /// Create a new connection limiter
    pub fn new() -> Self {
        Self {
            limits: DashMap::new(),
        }
    }

    /// Maximum concurrent connections allowed per document
    const MAX_CONNECTIONS: usize = 10;

    /// Acquire a permit for a WebSocket connection to the given document
    /// 
    /// Returns an error if the connection limit has been reached.
    /// The permit is automatically released when dropped.
    pub async fn acquire(
        &self,
        document_id: Uuid,
    ) -> Result<tokio::sync::OwnedSemaphorePermit, ApiError> {
        let semaphore = self
            .limits
            .entry(document_id)
            .or_insert_with(|| Arc::new(Semaphore::new(Self::MAX_CONNECTIONS)));

        semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| ApiError::TooManyRequests("Maximum WebSocket connections reached for this document".to_string()))
    }

    /// Get the current number of active connections for a document
    pub fn connection_count(&self, document_id: Uuid) -> usize {
        self.limits
            .get(&document_id)
            .map(|s| Self::MAX_CONNECTIONS - s.available_permits())
            .unwrap_or(0)
    }
}

impl Default for ConnectionLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tenth_connection_is_accepted() {
        let limiter = ConnectionLimiter::new();
        let document_id = Uuid::new_v4();

        // Acquire 10 permits
        let mut permits = Vec::new();
        for _ in 0..10 {
            let permit = limiter.acquire(document_id).await.unwrap();
            permits.push(permit);
        }

        assert_eq!(limiter.connection_count(document_id), 10);
    }

    #[tokio::test]
    async fn eleventh_connection_is_rejected() {
        let limiter = ConnectionLimiter::new();
        let document_id = Uuid::new_v4();

        // Acquire 10 permits
        let mut permits = Vec::new();
        for _ in 0..10 {
            permits.push(limiter.acquire(document_id).await.unwrap());
        }

        // 11th should fail
        let result = limiter.acquire(document_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::TooManyRequests(_) => {}
            _ => panic!("Expected TooManyRequests error"),
        }
    }

    #[tokio::test]
    async fn permit_release_allows_new_connection() {
        let limiter = ConnectionLimiter::new();
        let document_id = Uuid::new_v4();

        // Acquire 10 permits
        let mut permits = Vec::new();
        for _ in 0..10 {
            permits.push(limiter.acquire(document_id).await.unwrap());
        }

        // Drop one permit
        permits.pop();

        // Now we can acquire again
        assert!(limiter.acquire(document_id).await.is_ok());
    }
}
```

### Step 3: Create Document WebSocket Handler

File: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/websockets/documents.rs`

```rust
//! WebSocket handler for document status updates

use super::types::DocumentStatus;
use crate::error::ApiError;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Application state containing WebSocket broadcast channel
pub struct WebSocketState {
    pub status_tx: broadcast::Sender<DocumentStatus>,
}

/// Handle WebSocket upgrade for document status updates
pub async fn ws_document_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WebSocketState>>,
    Path(document_id): Path<Uuid>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_document_socket(socket, document_id, state))
}

/// Handle an active WebSocket connection for a document
async fn handle_document_socket(
    mut socket: WebSocket,
    document_id: Uuid,
    state: Arc<WebSocketState>,
) {
    // Subscribe to the broadcast channel
    let mut rx = state.status_tx.subscribe();

    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Track the last activity for idle timeout
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
                                        if sender.send(Message::Text(msg)).await.is_err() {
                                            break;
                                        }
                                        last_heartbeat = std::time::Instant::now();
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to serialize status: {}", e);
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
                // Send heartbeat ping
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
                        if sender.send(Message::Ping(vec![])).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Spawn receive task for client messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Pong(_) => {
                    // Client responded to ping - update activity tracking if needed
                }
                Message::Close(_) => {
                    break;
                }
                _ => {
                    // Ignore other message types (text, binary, etc.)
                }
            }
        }
    });

    // Wait for either task to complete, then abort the other
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }

    // Drop guard ensures both tasks are aborted
}
```

### Step 4: Update AppState and Main

Modify `/Users/marc/Projecten/iou-modern/crates/iou-api/src/workflows/mod.rs` to include WebSocket components:

```rust
use crate::websockets::limiter::ConnectionLimiter;
use crate::websockets::types::DocumentStatus;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub status_tx: broadcast::Sender<DocumentStatus>,
    pub connection_limiter: Arc<ConnectionLimiter>,
}
```

Update `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs`:

1. Add WebSocket module import:
```rust
mod websockets;
```

2. Update the Router to include WebSocket route:
```rust
// After existing routes...
.route("/ws/documents/{id}", get(websockets::documents::ws_document_handler))
```

3. Initialize broadcast channel and connection limiter:
```rust
// Create bounded broadcast channel for status updates
let (status_tx, _) = broadcast::channel(100);
let connection_limiter = Arc::new(websockets::limiter::ConnectionLimiter::new());

// Update AppState to include WebSocket components
let app_state = Arc::new(AppState {
    db: db_arc.clone(),
    status_tx,
    connection_limiter,
});
```

### Step 5: Integrate with Orchestrator

Update `/Users/marc/Projecten/iou-modern/crates/iou-api/src/orchestrator/wrapper.rs` to broadcast status updates:

```rust
use crate::websockets::types::DocumentStatus;

impl WorkflowOrchestrator {
    /// Send a status update to all WebSocket subscribers
    fn broadcast_status(&self, status: DocumentStatus) {
        // Ignore send errors - no subscribers is acceptable
        let _ = self.status_tx.send(status);
    }

    /// Called when an agent starts execution
    fn on_agent_started(&self, document_id: Uuid, agent_name: &str) {
        let status = DocumentStatus::Started {
            document_id,
            agent: agent_name.to_string(),
            timestamp: Utc::now().timestamp(),
        };
        self.broadcast_status(status);
    }

    /// Called when an agent makes progress
    fn on_agent_progress(&self, document_id: Uuid, agent_name: &str, percent: u8) {
        let status = DocumentStatus::Progress {
            document_id,
            agent: agent_name.to_string(),
            percent,
            message: None,
            timestamp: Utc::now().timestamp(),
        };
        self.broadcast_status(status);
    }

    /// Called when workflow completes successfully
    fn on_workflow_completed(&self, document_id: Uuid) {
        let status = DocumentStatus::Completed {
            document_id,
            timestamp: Utc::now().timestamp(),
        };
        self.broadcast_status(status);
    }

    /// Called when workflow fails
    fn on_workflow_failed(&self, document_id: Uuid, error: &str) {
        let status = DocumentStatus::Failed {
            document_id,
            error: error.to_string(),
            timestamp: Utc::now().timestamp(),
        };
        self.broadcast_status(status);
    }
}
```

### Step 6: Create WebSocket Module Export

File: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/websockets/mod.rs`

```rust
//! WebSocket support for real-time document status updates

pub mod documents;
pub mod limiter;
pub mod types;

pub use documents::ws_document_handler;
pub use limiter::ConnectionLimiter;
pub use types::DocumentStatus;
```

## Dependencies to Add

Update `/Users/marc/Projecten/iou-modern/crates/iou-api/Cargo.toml`:

```toml
# Add to existing dependencies
tokio-tungstenite = "0.24"
futures-util = "0.3"
dashmap = "6.1"

# Already present (ensure versions):
axum = { version = "0.7", features = ["ws"] }
```

## WebSocket URL and Usage

Once implemented, clients can connect to:

```
ws://localhost:3000/api/ws/documents/{document_id}
```

Where `{document_id}` is the UUID returned from `POST /api/documents/create`.

### Message Format

All messages are JSON with a `type` discriminator:

**Started:**
```json
{
  "type": "started",
  "document_id": "uuid-here",
  "agent": "research",
  "timestamp": 1234567890
}
```

**Progress:**
```json
{
  "type": "progress",
  "document_id": "uuid-here",
  "agent": "content",
  "percent": 50,
  "message": "Generating section 2 of 4",
  "timestamp": 1234567890
}
```

**Completed:**
```json
{
  "type": "completed",
  "document_id": "uuid-here",
  "timestamp": 1234567890
}
```

**Failed:**
```json
{
  "type": "failed",
  "document_id": "uuid-here",
  "error": "Agent timeout after 8 minutes",
  "timestamp": 1234567890
}
```

## Error Handling

The WebSocket handler should handle these error conditions:

1. **Connection limit exceeded**: Close connection immediately without sending error message (connection is rejected at TCP level).

2. **Serialization failure**: Log error and close connection.

3. **Lagged receiver**: Log warning, skip missed messages, continue with latest.

4. **Idle timeout**: Send close frame and terminate connection.

5. **Send failure**: Close connection and cleanup resources.

6. **Client disconnect**: Gracefully cleanup subscription and release connection permit.

## Testing Checklist

After implementation, verify:

- [ ] WebSocket connection succeeds for valid document_id
- [ ] Invalid document_id still allows connection (no document check at upgrade time)
- [ ] Status broadcasts are received only by subscribers of that document
- [ ] 10 concurrent connections allowed per document
- [ ] 11th connection is rejected
- [ ] Connection closes after 5 minutes of inactivity
- [ ] Ping frames are sent every 30 seconds
- [ ] Pong responses are handled correctly
- [ ] Lagging receiver receives warning and resyncs
- [ ] Connection closes gracefully on terminal status
- [ ] Client-initiated close is acknowledged
- [ ] Resources are cleaned up on abnormal termination