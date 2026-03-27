# Implementation Plan: Agent Orchestration API Extensions

## Introduction

Dit plan beschrijft de implementatie van vier uitbreidingen op de bestaande document API:
1. Pipeline executor integration
2. Authentication checks
3. S3 storage integration
4. WebSocket support

De implementatie volgt een dependency-gestuurde volgorde waarbij sommige components parallel kunnen worden ontwikkeld.

## Section 1: Foundation & Configuration

**Dependencies:** None

**Description:** Setup van de basis infrastructuur voor alle uitbreidingen.

**Tasks:**
1. Voeg nieuwe dependencies toe aan `Cargo.toml`
2. Maak S3 client abstraction in `iou-core`
3. Maak WebSocket handler structure in `iou-api`
4. Voeg environment variables toe voor configuratie
5. Voeg `iou-orchestrator` dependency toe aan `iou-api`
6. Implementeer S3 connectivity check bij startup

**Files to create:**
- `crates/iou-core/src/storage/mod.rs` - Storage module definition
- `crates/iou-core/src/storage/s3.rs` - S3 client implementation
- `crates/iou-api/src/websockets/mod.rs` - WebSocket module definition
- `crates/iou-api/src/orchestrator/mod.rs` - Orchestrator wrapper module

**Files to modify:**
- `crates/iou-api/Cargo.toml` - Add tokio-tungstenite, futures-util, rust-s3, iou-orchestrator
- `crates/iou-core/Cargo.toml` - Add rust-s3
- `crates/iou-api/src/config.rs` - Add S3 config loading with validation

## Section 2: Orchestrator Integration

**Dependencies:** Section 1

**Description:** Integreert de bestaande `iou-orchestrator` crate voor workflow execution.

**Tasks:**
1. Maak `orchestrator/wrapper.rs` met `WorkflowStateMachine` integratie
2. Implementeer broadcast channel voor status updates
3. Voeg overall timeout handling toe (8 min totaal voor alle agents)
4. Implementeer idempotency check voor duplicate document creation
5. Update database met progress na elke agent
6. Koppel templates uit `iou-ai` aan de workflow

**Files to create:**
- `crates/iou-api/src/orchestrator/wrapper.rs` - Orchestrator wrapper
- `crates/iou-api/src/orchestrator/types.rs` - Type mappings

**Files to modify:**
- `crates/iou-api/src/orchestrator/mod.rs` - Export modules
- `crates/iou-api/src/routes/documents.rs` - Integrate orchestrator in create_document()

**Key implementation details:**
- Gebruik bestaande `WorkflowStateMachine` van `iou-orchestrator`
- Use `tokio::spawn` voor async execution
- Use `tokio::sync::broadcast::channel(100)` for status updates (bounded)
- Idempotency: check bestaande document_id voor create
- Overall timeout: 8 minuten cumulatief (niet per agent)
- Map `WorkflowState` → `DocumentState` consistent

## Section 3: Authentication Integration

**Dependencies:** None (kan parallel met Section 1)

**Description:** Voeg RBAC checks toe aan document endpoints.

**Tasks:**
1. Voeg AuthContext extractor toe aan endpoints
2. Check `ObjectCreator`/`DomainEditor` role voor create
3. Check `ObjectApprover` role voor approve
4. Sla user_id op in audit trail
5. Voeg organization-based access control toe
6. Validate JWT secret via environment variable

**Files to modify:**
- `crates/iou-api/src/routes/documents.rs`
  - `create_document()`: Add `Extension<AuthContext>` parameter
  - `approve_document()`: Add role check
  - Add organization scoping to queries
  - Update audit logging with user_id
- `crates/iou-api/src/middleware/auth.rs`
  - Load JWT_SECRET from environment

**Key implementation details:**
```rust
use crate::middleware::auth::AuthContext;

pub async fn create_document(
    Extension(db): Extension<Arc<Database>>,
    Extension(auth): Extension<AuthContext>,  // NEW
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, ApiError> {
    // Check roles (FIXED: correct pattern matching)
    let has_permission = auth.roles.iter()
        .any(|r| *r == Role::ObjectCreator || *r == Role::DomainEditor);
    if !has_permission {
        return Err(ApiError::Forbidden(
            "Requires ObjectCreator or DomainEditor role".to_string()
        ));
    }

    // Check organization access (NEW)
    if auth.organization_id != req.organization_id {
        return Err(ApiError::Forbidden(
            "Cannot access documents outside your organization".to_string()
        ));
    }
    // ... rest of implementation
}
```

## Section 4: S3 Storage Integration

**Dependencies:** Section 1

**Description:** Implementeer document opslag in S3/MinIO.

**Tasks:**
1. Implementeer S3 client met path-style URLs
2. Upload logica na pipeline completion
3. Download proxy met streaming (geen presigned URLs)
4. Size validation (max 10MB)
5. Error handling voor S3 failures met proper error conversion
6. Add S3 connectivity validation bij startup

**Files to create:**
- `crates/iou-core/src/storage/s3.rs` - S3 operations met streaming

**Files to modify:**
- `crates/iou-api/src/routes/documents.rs`
  - `download_document()`: Stream from S3
- `crates/iou-api/src/orchestrator/wrapper.rs`
  - Upload after completion
- `crates/iou-api/src/main.rs`
  - Validate S3 connectivity bij startup

**Key implementation details:**
```rust
// S3 Client setup
pub struct S3Client {
    bucket: Bucket,
}

impl S3Client {
    pub fn new_from_env() -> Result<Self, S3Error> {
        let credentials = Credentials::new(
            Some(std::env::var("S3_ACCESS_KEY")?),
            Some(std::env::var("S3_SECRET_KEY")?),
            None, None, None,
        )?;
        let bucket = Bucket::new(
            &std::env::var("S3_BUCKET")?,
            "us-east-1".parse()?,
            credentials,
        )?.with_path_style();
        Ok(Self { bucket })
    }

    // NEW: Validate connectivity at startup
    pub async fn validate(&self) -> Result<(), S3Error> {
        // Test connection with HEAD request
        self.bucket.head_object("").await?;
        Ok(())
    }

    pub async fn upload(&self, key: &str, data: Vec<u8>) -> Result<(), S3Error> {
        // Validate size
        if data.len() > 10 * 1024 * 1024 {
            return Err(S3Error::HttpFailWithBody(
                413,
                "Document exceeds 10MB limit".to_string(),
            ));
        }
        let reader = Cursor::new(data);
        self.bucket.put_object(key, &mut reader.into_bytes()).await?;
        Ok(())
    }

    // UPDATED: Streaming download
    pub async fn download_stream(&self, key: &str) -> Result<impl AsyncRead + Send, S3Error> {
        let response = self.bucket.get_object(key).await?;
        Ok(response.bytes().to_reader()) // Returns reader for streaming
    }
}

// Error conversion layer (NEW)
impl From<S3Error> for ApiError {
    fn from(err: S3Error) -> Self {
        match err {
            S3Error::HttpFailWithBody(code, msg) => match code {
                404 => ApiError::NotFound("Document not found in storage".to_string()),
                413 => ApiError::PayloadTooLarge(msg),
                _ => ApiError::Internal(format!("Storage error: {}", msg)),
            },
            _ => ApiError::Internal("Storage connection error".to_string()),
        }
    }
}
```

## Section 5: WebSocket Support

**Dependencies:** Section 1, Section 2

**Description:** Real-time status updates via WebSocket.

**Tasks:**
1. WebSocket upgrade handler
2. Per-document broadcast subscription met bounded channel (cap 100)
3. Connection limit per document (max 10)
4. Status message serialization
5. Idle timeout (5 min) met heartbeat
6. Graceful disconnect bij completion
7. Panic recovery in send/receive tasks
8. Proper cleanup bij abnormal closure

**Files to create:**
- `crates/iou-api/src/websockets/documents.rs` - Document WebSocket handler
- `crates/iou-api/src/websockets/types.rs` - Message types
- `crates/iou-api/src/websockets/limiter.rs` - Connection limiter

**Files to modify:**
- `crates/iou-api/src/main.rs` - Add WebSocket route
- `crates/iou-api/src/orchestrator/wrapper.rs` - Broadcast to WebSocket

**Key implementation details:**
```rust
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Semaphore;

// NEW: Connection limiter
pub struct ConnectionLimiter {
    limits: dashmap::DashMap<Uuid, Arc<Semaphore>>,
}

impl ConnectionLimiter {
    pub fn new() -> Self {
        Self { limits: Default::default() }
    }

    pub async fn acquire(&self, document_id: Uuid) -> Result<OwnedSemaphorePermit, ApiError> {
        let semaphore = self.limits
            .entry(document_id)
            .or_insert_with(|| Arc::new(Semaphore::new(10))); // Max 10 connections
        semaphore.acquire().await
            .map_err(|_| ApiError::TooManyRequests("Max WebSocket connections reached".to_string()))
    }
}

// Status message type
#[derive(Clone, Serialize)]
#[serde(tag = "type")]
enum DocumentStatus {
    #[serde(rename = "started")]
    Started { document_id: Uuid, agent: String },
    #[serde(rename = "progress")]
    Progress { document_id: Uuid, agent: String, percent: u8 },
    #[serde(rename = "completed")]
    Completed { document_id: Uuid },
    #[serde(rename = "failed")]
    Failed { document_id: Uuid, error: String },
}

// UPDATED: Bounded broadcast channel
// In AppState: status_tx: broadcast::Sender<DocumentStatus>(100)

// WebSocket handler
pub async fn ws_document_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_document_socket(socket, document_id, state))
}

async fn handle_document_socket(
    mut socket: WebSocket,
    document_id: Uuid,
    state: Arc<AppState>,
) {
    // NEW: Acquire connection permit
    let _permit = match state.connection_limiter.acquire(document_id).await {
        Ok(p) => p,
        Err(e) => {
            let _ = socket.close().await;
            return;
        }
    };

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.status_tx.subscribe();

    // NEW: Drop guard for cleanup
    struct DropGuard(tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>);
    impl Drop for DropGuard {
        fn drop(&mut self) {
            self.0.abort();
            self.1.abort();
        }
    }

    // Send task met panic recovery
    let send_task = tokio::spawn(async move {
        let mut last_heartbeat = Instant::now();
        let heartbeat_interval = Duration::from_secs(30);
        let idle_timeout = Duration::from_secs(300); // 5 min

        loop {
            tokio::select! {
                // Receive from broadcast channel
                result = rx.recv() => {
                    match result {
                        Ok(status) => {
                            if status.document_id() == document_id {
                                let msg = serde_json::to_string(&status).unwrap();
                                if sender.send(Message::Text(msg)).await.is_err() {
                                    break;
                                }
                                last_heartbeat = Instant::now();
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            // NEW: Handle lag - reconnect with latest
                            tracing::warn!("WebSocket lagged, skipped {} messages", n);
                        }
                        Err(_) => break,
                    }
                }
                // Heartbeat / idle timeout check
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    if last_heartbeat.elapsed() > idle_timeout {
                        // Send close frame
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

    // Receive task met panic recovery
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Pong(_) => {}, // Reset heartbeat if needed
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // NEW: Proper cleanup
    let _guard = DropGuard(send_task, recv_task);
    tokio::select! {
        _ = send_task => recv_task.abort(),
        _ = recv_task => send_task.abort(),
    }
    // Guard drops here, ensuring cleanup
}
```

## Section 6: Integration & Testing

**Dependencies:** Section 2, Section 3, Section 4, Section 5

**Description:** Volledige integratie en testing.

**Tasks:**
1. Integreer alle components in main.rs
2. End-to-end tests voor volledige flow
3. Unit tests voor elke component
4. Error handling tests
5. Performance tests (concurrent requests)
6. Mock infrastructure voor S3 en pipeline testing

**Files to modify:**
- `crates/iou-api/src/main.rs` - Wire everything together
- Add comprehensive tests

**Test infrastructure:**
- `crates/iou-api/tests/mocks/s3.rs` - Mock S3 client
- `crates/iou-api/tests/mocks/orchestrator.rs` - Mock orchestrator
- `crates/iou-api/tests/helpers/websocket.rs` - WebSocket test helper

**Test scenarios:**
- Document creatie → async pipeline → completion
- Idempotency: duplicate document creation → returns existing
- Invalid role attempt → 403 Forbidden
- Organization bypass attempt → 403 Forbidden
- S3 upload/download roundtrip
- WebSocket subscribe → receive updates → auto disconnect
- Connection limit: 11th connection → rejected
- Overall timeout scenario (> 8 min totaal)
- Oversized document (>10MB) → 413 error
- S3 unavailable → proper error propagation

## Execution Order

**Batch 1 (Parallel):**
- Section 1: Foundation & Configuration
- Section 3: Authentication Integration

**Batch 2 (After 1):**
- Section 2: Pipeline Executor

**Batch 3 (After 1):**
- Section 4: S3 Storage Integration

**Batch 4 (After 2):**
- Section 5: WebSocket Support

**Batch 5 (After 2, 3, 4, 5):**
- Section 6: Integration & Testing

## Dependencies Graph

```
Section 1 (Foundation)
    ├── Section 2 (Orchestrator Integration)
    │       └── Section 5 (WebSocket)
    └── Section 4 (S3 Storage)

Section 3 (Auth) - independent
    └── Section 6 (Integration)

Section 6 (Integration) - depends on 2, 3, 4, 5
```

**Key Changes from Review:**
- Section 2 now integrates existing `iou-orchestrator` instead of creating new executor
- Added organization-based access control in Section 3
- Added S3 startup validation in Section 1/4
- WebSocket uses bounded channels + connection limits in Section 5

## Risk Mitigation

| Risk | Mitigatie |
|------|-----------|
| Pipeline deadlock | Overall timeout (8 min) + per-agent tracking |
| S3 connection issues | Startup validation + retry logic met exponential backoff |
| WebSocket memory leak | Bounded channel (100) + idle timeout + connection limits (10) |
| Auth bypass | Comprehensive role checks + organization scoping + tests |
| Duplicate document creation | Idempotency check met document_id |
| Connection exhaustion | Per-document connection limit |
| S3 credential issues | Fail-fast validation bij startup |
| Download OOM | Streaming response, no buffering |
| WebSocket panic | Recovery guards in send/receive tasks |
