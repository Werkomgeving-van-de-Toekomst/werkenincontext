Now I have all the context I need. Let me generate the section content for `section-06-integration-testing`. This section covers the final integration and testing phase that depends on all previous sections.

---

# Section 6: Integration & Testing

## Overview

This section covers the final integration of all components implemented in previous sections (Foundation, Orchestrator, Authentication, S3 Storage, and WebSocket Support) and the comprehensive testing suite to validate the complete system.

**Dependencies:** This section requires completion of:
- Section 1: Foundation & Configuration
- Section 2: Orchestrator Integration
- Section 3: Authentication Integration
- Section 4: S3 Storage Integration
- Section 5: WebSocket Support

## Tasks

1. Wire all components together in `main.rs`
2. Create mock infrastructure for testing
3. Implement end-to-end tests for complete document flow
4. Add unit tests for each component
5. Implement error handling tests
6. Add performance tests for concurrent operations

## Files to Modify

### Main Integration

**File: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs`**

Wire together all the components configured in previous sections:

1. Initialize AppState with all required fields:
   - Database connection
   - S3 client (with startup validation)
   - WebSocket broadcast channel
   - Connection limiter
   - Orchestrator workflow manager

2. Register all routes:
   - Document CRUD routes (with auth middleware)
   - WebSocket upgrade endpoint
   - Health check endpoint

3. Startup validation sequence:
   ```rust
   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Initialize logging
       tracing_subscriber::init();
       
       // Load configuration
       let config = Config::from_env()?;
       
       // Initialize database
       let db = Arc::new(Database::connect(&config.database_url).await?);
       
       // Initialize and validate S3 client
       let s3_client = Arc::new(S3Client::new_from_env()?);
       s3_client.validate().await?;
       
       // Create broadcast channel for WebSocket updates
       let (status_tx, _) = broadcast::channel(100);
       
       // Create connection limiter
       let connection_limiter = Arc::new(ConnectionLimiter::new());
       
       // Build app state
       let state = Arc::new(AppState {
           db,
           s3_client,
           status_tx: Arc::new(status_tx),
           connection_limiter,
           // ... orchestrator and other fields
       });
       
       // Build router
       let app = Router::new()
           .route("/api/documents", post(create_document).get(list_documents))
           .route("/api/documents/:id", get(get_document))
           .route("/api/documents/:id/download", get(download_document))
           .route("/api/documents/:id/approve", post(approve_document))
           .route("/ws/documents/:id", get(ws_document_handler))
           .layer(Extension(config.clone()))
           .with_state(state.clone());
       
       // Start server
       let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
       axum::serve(listener, app).await?;
       
       Ok(())
   }
   ```

## Test Infrastructure

Create the following test support files to enable comprehensive testing without external dependencies.

### Mock S3 Client

**File: `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/mocks/s3.rs`**

Mock S3 client for testing upload/download flows:

```rust
use iou_core::storage::{S3Client, S3Error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MockS3Client {
    storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    fail_uploads: Arc<RwLock<bool>>,
    fail_downloads: Arc<RwLock<bool>>,
}

impl MockS3Client {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            fail_uploads: Arc::new(RwLock::new(false)),
            fail_downloads: Arc::new(RwLock::new(false)),
        }
    }
    
    pub async fn upload(&self, key: &str, data: Vec<u8>) -> Result<(), S3Error> {
        if *self.fail_uploads.read().await {
            return Err(S3Error::HttpFailWithBody(503, "Simulated failure".to_string()));
        }
        self.storage.write().await.insert(key.to_string(), data);
        Ok(())
    }
    
    pub async fn download(&self, key: &str) -> Result<Vec<u8>, S3Error> {
        if *self.fail_downloads.read().await {
            return Err(S3Error::HttpFailWithBody(503, "Simulated failure".to_string()));
        }
        self.storage.read().await
            .get(key)
            .cloned()
            .ok_or_else(|| S3Error::HttpFailWithBody(404, "Not found".to_string()))
    }
    
    pub async fn set_fail_uploads(&self, fail: bool) {
        *self.fail_uploads.write().await = fail;
    }
    
    pub async fn set_fail_downloads(&self, fail: bool) {
        *self.fail_downloads.write().await = fail;
    }
}
```

### Mock Orchestrator

**File: `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/mocks/orchestrator.rs`**

Mock workflow state machine for controlled testing:

```rust
use iou_orchestrator::{WorkflowStateMachine, WorkflowState, WorkflowError};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

pub struct MockWorkflowStateMachine {
    state: Arc<Mutex<WorkflowState>>,
    should_fail: Arc<Mutex<bool>>,
    delay_ms: Arc<Mutex<u64>>,
}

impl MockWorkflowStateMachine {
    pub fn new(initial_state: WorkflowState) -> Self {
        Self {
            state: Arc::new(Mutex::new(initial_state)),
            should_fail: Arc::new(Mutex::new(false)),
            delay_ms: Arc::new(Mutex::new(0)),
        }
    }
    
    pub async fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().await = fail;
    }
    
    pub async fn set_delay(&self, millis: u64) {
        *self.delay_ms.lock().await = millis;
    }
    
    pub async fn get_state(&self) -> WorkflowState {
        *self.state.lock().await
    }
}

#[async_trait::async_trait]
impl WorkflowStateMachine for MockWorkflowStateMachine {
    async fn start(&self) -> Result<(), WorkflowError> {
        let delay = *self.delay_ms.lock().await;
        if delay > 0 {
            sleep(Duration::from_millis(delay)).await;
        }
        if *self.should_fail.lock().await {
            return Err(WorkflowError::AgentError("Simulated failure".to_string()));
        }
        *self.state.lock().await = WorkflowState::Running;
        Ok(())
    }
    
    // Implement other required methods...
}
```

### WebSocket Test Helper

**File: `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/helpers/websocket.rs`**

WebSocket client helper for testing real-time updates:

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::Value;

pub struct WebSocketTestClient {
    url: String,
}

impl WebSocketTestClient {
    pub fn new(document_id: Uuid) -> Self {
        Self {
            url: format!("ws://localhost:3000/ws/documents/{}", document_id),
        }
    }
    
    pub async fn connect(&self) -> Result<WebSocketConnection, Error> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (write, read) = ws_stream.split();
        Ok(WebSocketConnection { write, read })
    }
}

pub struct WebSocketConnection {
    // Wrapper around split stream/sink
}

impl WebSocketConnection {
    pub async fn recv_status(&mut self) -> Result<DocumentStatus, Error> {
        // Receive and parse next message
    }
    
    pub async fn close(mut self) -> Result<(), Error> {
        // Close connection gracefully
    }
}
```

## End-to-End Tests

### Test: Document Creation Flow

**Location:** `crates/iou-api/tests/integration/document_flow.rs`

```rust
#[tokio::test]
async fn document_creation_to_completion_flow() {
    // Given: Authenticated user with ObjectCreator role
    let auth_context = AuthContext {
        user_id: Uuid::new_v4(),
        organization_id: Uuid::new_v4(),
        roles: vec![Role::ObjectCreator],
    };
    
    // When: POST /api/documents/create is called
    let response = client
        .post("/api/documents/create")
        .header("Authorization", format!("Bearer {}", token))
        .json(&CreateDocumentRequest {
            id: Some(Uuid::new_v4()),
            template_id: "standard_template".to_string(),
            organization_id: auth_context.organization_id,
            // ... other fields
        })
        .send()
        .await;
    
    // Then: Returns 201 Created
    assert_eq!(response.status(), 201);
    
    // And: Workflow starts asynchronously
    let document: CreateDocumentResponse = response.json().await;
    assert!(matches!(document.state, DocumentState::Processing));
    
    // And: WebSocket clients receive status updates
    let mut ws = WebSocketTestClient::new(document.id).connect().await.unwrap();
    
    let status = ws.recv_status().await.unwrap();
    assert!(matches!(status, DocumentStatus::Started { .. }));
    
    let status = ws.recv_status().await.unwrap();
    assert!(matches!(status, DocumentStatus::Progress { .. }));
    
    let status = ws.recv_status().await.unwrap();
    assert!(matches!(status, DocumentStatus::Completed { .. }));
    
    // And: Document is stored in S3 on completion
    let stored_doc = s3_client.download(&document.s3_key()).await.unwrap();
    assert!(!stored_doc.is_empty());
}
```

### Test: Unauthorized Access

```rust
#[tokio::test]
async fn unauthorized_user_cannot_create_document() {
    // Given: User without required role
    let auth_context = AuthContext {
        user_id: Uuid::new_v4(),
        organization_id: Uuid::new_v4(),
        roles: vec![Role::DomainViewer], // Insufficient role
    };
    
    // When: POST /api/documents/create is called
    let response = client
        .post("/api/documents/create")
        .header("Authorization", format!("Bearer {}", viewer_token))
        .json(&valid_request)
        .send()
        .await;
    
    // Then: Returns 403 Forbidden
    assert_eq!(response.status(), 403);
    
    // And: No workflow is started
    // Verify no entries in orchestrator
}
```

### Test: Organization Isolation

```rust
#[tokio::test]
async fn user_cannot_access_other_organization_documents() {
    // Given: User from organization A
    let org_a = Uuid::new_v4();
    let org_b = Uuid::new_v4();
    
    // And: Document from organization B
    let doc_id = create_document_for_org(org_b).await;
    
    // When: GET /api/documents/{id} is called by user from org A
    let response = client
        .get(&format!("/api/documents/{}", doc_id))
        .header("Authorization", format!("Bearer {}", org_a_token))
        .send()
        .await;
    
    // Then: Returns 403 Forbidden or 404 Not Found
    assert!(matches!(response.status(), 403 | 404));
}
```

### Test: Size Validation

```rust
#[tokio::test]
async fn oversized_document_is_rejected() {
    // Given: Document payload exceeding 10MB
    let large_payload = vec![0u8; 11 * 1024 * 1024]; // 11MB
    
    // When: POST /api/documents/create is called
    let response = client
        .post("/api/documents/create")
        .json(&CreateDocumentRequest {
            content: large_payload,
            // ... other fields
        })
        .send()
        .await;
    
    // Then: Returns 413 Payload Too Large
    assert_eq!(response.status(), 413);
    
    // And: No upload to S3 is attempted
    assert!(!s3_client.contains_key(&expected_key).await);
}
```

### Test: Workflow Timeout

```rust
#[tokio::test]
async fn workflow_timeout_propagates_correctly() {
    // Given: Mock orchestrator configured to exceed timeout
    mock_orchestrator.set_delay(9 * 60 * 1000).await; // 9 minutes
    
    // When: Workflow starts
    let document = create_document().await;
    
    // Then: Workflow is cancelled after 8 minutes
    sleep(Duration::from_secs(8 * 60 + 5)).await;
    
    let updated = get_document(document.id).await;
    assert!(matches!(updated.state, DocumentState::Failed { .. }));
    
    // And: WebSocket clients receive Failed status
    let mut ws = WebSocketTestClient::new(document.id).connect().await.unwrap();
    let status = ws.recv_status().await.unwrap();
    assert!(matches!(status, DocumentStatus::Failed { .. }));
}
```

## Concurrency Tests

### Test: Concurrent Document Creation

```rust
#[tokio::test]
async fn multiple_concurrent_document_creations() {
    // Given: 10 simultaneous create requests
    let requests: Vec<_> = (0..10)
        .map(|_| create_document_async())
        .collect();
    
    // When: All requests are processed concurrently
    let results = futures::future::join_all(requests).await;
    
    // Then: All documents are created
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
    }
    
    // And: All workflows execute independently
    // Verify each document has unique ID and workflow
}
```

### Test: Concurrent WebSocket Connections

```rust
#[tokio::test]
async fn concurrent_websocket_connections_receive_correct_updates() {
    // Given: 5 WebSocket connections for document A
    // And: 3 connections for document B
    let mut ws_clients_a = connect_websockets(document_a, 5).await;
    let mut ws_clients_b = connect_websockets(document_b, 3).await;
    
    // When: Both documents have status updates
    broadcast_status(document_a, Status::Progress).await;
    broadcast_status(document_b, Status::Progress).await;
    
    // Then: Only connections for respective documents receive updates
    for ws in &mut ws_clients_a {
        let status = ws.recv_status().await.unwrap();
        assert_eq!(status.document_id(), document_a);
    }
    
    for ws in &mut ws_clients_b {
        let status = ws.recv_status().await.unwrap();
        assert_eq!(status.document_id(), document_b);
    }
}
```

## Performance Tests

### Test: Streaming Memory Usage

```rust
#[tokio::test]
async fn s3_streaming_download_does_not_buffer_large_file() {
    // Given: 10MB document in S3
    let large_doc = vec![0u8; 10 * 1024 * 1024];
    s3_client.upload("test_key", large_doc.clone()).await.unwrap();
    
    // When: Download endpoint streams response
    let response = client
        .get("/api/documents/test_id/download")
        .send()
        .await;
    
    // Then: Returns 200 OK with streaming response
    assert_eq!(response.status(), 200);
    
    // And: Memory usage during download is < 1MB
    // This requires benchmarking tools to verify
    let bytes = response.bytes().await.unwrap();
    assert_eq!(bytes.len(), large_doc.len());
}
```

### Test: Broadcast Channel Performance

```rust
#[tokio::test]
async fn broadcast_channel_performance_under_load() {
    // Given: 100 status updates per second
    // And: 10 WebSocket subscribers
    let mut subscribers = create_websocket_subscribers(10).await;
    let mut tx = broadcast::channel(100).0;
    
    // When: Load test runs for 1 minute
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(60) {
        tx.send(DocumentStatus::Progress { ... }).unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Then: No message loss for non-lagging receivers
    for mut sub in &mut subscribers {
        while let Ok(status) = sub.try_recv() {
            // Verify message received
        }
    }
    
    // And: Lagging receivers receive Lagged errors
    // (if configured with bounded channel)
}
```

## Error Handling Tests

### Test: S3 Failure Recovery

```rust
#[tokio::test]
async fn s3_upload_retry_on_transient_error() {
    // Given: Mock S3 that fails once then succeeds
    mock_s3.set_fail_uploads(true).await;
    let attempts = Arc::new(Mutex::new(0));
    
    // When: upload_with_retry is called
    let result = upload_with_retry(&mock_s3, "key", data.clone(), 3).await;
    
    // Then: Eventually succeeds after retry
    assert!(result.is_ok());
    assert_eq!(*attempts.lock().await, 2); // Failed once, succeeded on retry
}
```

### Test: Connection Limit Enforcement

```rust
#[tokio::test]
async fn eleventh_websocket_connection_is_rejected() {
    // Given: 10 existing connections for document
    let mut existing = connect_websockets(document_id, 10).await;
    
    // When: 11th connection attempt is made
    let result = WebSocketTestClient::new(document_id).connect().await;
    
    // Then: Connection is rejected
    assert!(result.is_err());
    
    // And: Error indicates limit reached
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Max WebSocket connections"));
}
```

## Running Tests

All tests can be run with:

```bash
# Run all tests
cargo test --package iou-api

# Run integration tests only
cargo test --package iou-api --test integration

# Run with logging
RUST_LOG=debug cargo test --package iou-api

# Run specific test
cargo test --package iou-api document_creation_to_completion_flow
```

## Test Organization

```
crates/iou-api/tests/
├── mocks/
│   ├── s3.rs              # Mock S3 client
│   └── orchestrator.rs    # Mock workflow state machine
├── helpers/
│   └── websocket.rs       # WebSocket test utilities
├── integration/
│   ├── document_flow.rs   # End-to-end document lifecycle
│   ├── auth_tests.rs      # Authentication/authorization tests
│   └── storage_tests.rs   # S3 upload/download tests
├── concurrency/
│   └── concurrent_operations.rs  # Concurrent request handling
└── performance/
    └── streaming_memory.rs       # Memory usage verification
```