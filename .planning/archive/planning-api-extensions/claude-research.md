# API Extensions Research

**Date:** 2025-03-10

## Codebase Analysis

### 1. Agent Pipeline (`crates/iou-ai/src/agents/pipeline.rs`)

**Architecture:**
- **Sequential execution** with maker-checker iteration loop
- **Four agents**: Research, Content, Compliance, Review
- **Retry logic** with exponential backoff for transient errors
- **Checkpoint/restart capability** enabled by default

**Key Types:**
```rust
// Input
DocumentRequest + Template (with research context)

// Output: PipelineResult
- Final status (WorkflowStatus)
- Agent execution results
- Compliance score
- Confidence score
- Requires human approval flag
```

**Error Handling:**
- Transient errors: Retry with exponential backoff
- Permanent errors: Fail-fast with `PipelineError::AgentFailed`
- Async error propagation through the pipeline

**Async Support:**
- Fully async using `tokio`
- Async wrapper methods for each agent
- Spawn-blocking for database operations

### 2. Auth Middleware (`crates/iou-api/src/middleware/auth.rs`)

**AuthContext Structure:**
```rust
pub struct AuthContext {
    pub user_id: Uuid,
    pub email: String,
    pub organization_id: Uuid,
    pub roles: Vec<Role>,
}
```

**Role Hierarchy:**
- Admin > Auditor > DomainManager > DomainEditor > DomainViewer
- Domain-specific: ObjectCreator, ObjectEditor, **ObjectApprover**
- Compliance: ComplianceOfficer, ComplianceReviewer
- Woo-specific: WooOfficer, WooPublisher

**Permission Checking:**
- `Role::has_permission()` method
- JWT-based with 24-hour expiration
- Bearer token extraction from Authorization header

### 3. Document Routes (`crates/iou-api/src/routes/documents.rs`)

**Current Endpoints:**
- `POST /api/documents/create` - Create document (TODO: invoke pipeline)
- `GET /api/documents/{id}/status` - Get status
- `POST /api/documents/{id}/approve` - Approve/reject (TODO: check role)
- `GET /api/documents/{id}/audit` - Audit trail
- `GET /api/documents/{id}/download` - Download (TODO: S3)

**TODO Locations:**
1. Line 140: Invoke pipeline executor asynchronously
2. Line 160: Load domain config and check trust level
3. Line 183: Check for object_approver role
4. Line 216: Trigger final processing (storage, publication)
5. Line 276: Retrieve from S3 storage

**Database Pattern:**
- DuckDB with async wrappers using `tokio::task::spawn_blocking`
- Repository pattern: `get_document_async()`, `create_document_async()`

### 4. Test Patterns

- Uses `#[tokio::test]` for async tests
- Minimal mocking - real implementations with test fixtures
- In-memory DuckDB (`:memory:`) for testing

## Web Research

### 1. rust-s3 Crate (v0.37+)

**Configuration:**
```rust
use s3::Bucket;
use s3::creds::Credentials;

let credentials = Credentials::new(
    Some("ACCESS_KEY"),
    Some("SECRET_KEY"),
    None, None, None,
)?;

let bucket = Bucket::new("bucket", "us-east-1".parse()?, credentials)?
    .with_path_style(); // Required for MinIO
```

**Upload Pattern:**
```rust
// Simple upload
let response = bucket
    .put_object(key, &mut data.as_slice())
    .await?;
```

**Presigned URLs:**
```rust
// GET (download) - valid for 300 seconds
let url = bucket.presign_get(key, 300, None).await?;

// PUT (upload)
let url = bucket.presign_put(key, Duration::from_secs(300), None).await?;
```

**Features:**
```toml
rust-s3 = { version = "0.37", features = ["tokio", "fail-on-err"] }
```

### 2. Axum WebSocket Integration

**Basic Pattern:**
```rust
use axum::extract::ws::{WebSocketUpgrade, Message};
use futures_util::{sink::SinkExt, stream::StreamExt};

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // Send
    sender.send(Message::Text("Hello".into())).await;

    // Receive
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => { /* handle */ }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
```

**Broadcast Channel for Multiple Clients:**
```rust
use tokio::sync::broadcast;

struct AppState {
    tx: broadcast::Sender<String>,
}

async fn handle_websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let mut rx = state.tx.subscribe();

    // Task 1: Broadcast to WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Task 2: Receive from WebSocket
    let tx = state.tx.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = tx.send(text);
        }
    });

    tokio::select! {
        _ = send_task => recv_task.abort(),
        _ = recv_task => send_task.abort(),
    };
}
```

### 3. Async Pipeline with Tokio

**Broadcast Channel for Status Updates:**
```rust
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
enum PipelineStatus {
    Started(String),
    Progress { stage: String, percent: u8 },
    Completed(String),
    Failed { stage: String, error: String },
}

let (status_tx, _) = broadcast::channel(100);

// Multiple subscribers
let mut ui_rx = status_tx.subscribe();
let mut log_rx = status_tx.subscribe();

// Send updates
status_tx.send(PipelineStatus::Progress {
    stage: "Processing".to_string(),
    percent: 50,
})?;
```

**Timeout Handling:**
```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(30),
    long_running_operation(),
).await?;

match result {
    Ok(value) => Ok(value),
    Err(_) => Err("Operation timed out".into()),
}
```

### 4. S3 Presigned URLs vs API Proxy

**Recommendation: Use Presigned URLs**

| Aspect | Presigned URLs | API Proxy |
|--------|---------------|-----------|
| Security | Time-limited, IAM-based | Server control |
| Performance | Direct from S3 (faster) | Through server (slower) |
| Scaling | Automatic | Limited by infrastructure |
| Cost | Lower | Higher (double transfer) |

**When to use Presigned URLs:**
- Large file downloads
- High traffic scenarios
- Temporary access acceptable

**When to use API Proxy:**
- Real-time access control needed
- Document transformation required
- Detailed audit logging needed

## Implementation Recommendations

1. **Pipeline Executor Integration:**
   - Use `tokio::spawn` to run pipeline asynchronously
   - Use `broadcast::channel` for status updates
   - Store intermediate results in DuckDB

2. **Authentication:**
   - Check `ObjectApprover` role in approve endpoint
   - Extract `user_id` from AuthContext for audit trail

3. **S3 Storage:**
   - Use rust-s3 v0.37 with tokio feature
   - Implement presigned URLs for downloads
   - Use `with_path_style()` for MinIO compatibility

4. **WebSocket:**
   - Use tokio-tungstenite with Axum
   - Per-document broadcast channels
   - Automatic disconnect on completion
