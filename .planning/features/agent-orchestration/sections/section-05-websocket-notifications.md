# Section 05: WebSocket Server & Notifications

## Overview

This section implements the WebSocket server for real-time workflow updates and the multi-channel notification system. The WebSocket handler authenticates connections using JWT before upgrade, delivers real-time events to subscribed clients, and handles approval messages with user attribution from authentication context.

**Dependencies:**
- `section-03-checkpoint-recovery` - PostgreSQL for persistent data
- `section-02-parallel-executor` - Event bus for subscription

**Files to Create:**
- `/server/crates/iou-orchestrator/src/websocket/mod.rs`
- `/server/crates/iou-orchestrator/src/websocket/handler.rs`
- `/server/crates/iou-orchestrator/src/websocket/messages.rs`
- `/server/crates/iou-orchestrator/src/notification/mod.rs`
- `/server/crates/iou-orchestrator/src/notification/channels.rs`
- `/server/crates/iou-orchestrator/src/escalation/mod.rs`

---

## Tests (TDD)

### 3.2 WebSocket Server

**Test: WebSocket handler rejects unauthenticated connections**
- Attempt WebSocket upgrade without auth header
- Assert connection closed with 401
- Assert no socket created

**Test: WebSocket handler accepts valid JWT**
- Send WebSocket upgrade with valid Bearer token
- Assert connection upgraded
- Assert socket receives initial message

**Test: WebSocket handler validates JWT before upgrade**
- Send WebSocket upgrade with expired JWT
- Assert connection rejected
- Assert error message about expired credentials

**Test: WebSocket subscription receives workflow events**
- Connect authenticated WebSocket
- Subscribe to workflow ID
- Trigger agent completion event
- Assert WebSocket receives `AgentCompleted` message

**Test: WebSocket approve message uses authenticated user ID**
- Connect authenticated WebSocket as user "alice"
- Send `Approve` message with `request_id` (no approver_id)
- Assert approval recorded with `approver="alice"`
- Assert other users cannot approve as "alice"

**Test: WebSocket rejects approve for unauthorized workflow**
- Connect authenticated WebSocket
- Send `Approve` message for workflow user cannot access
- Assert receives `Error` message
- Assert approval not applied

**Test: WebSocket handles subscribe/unsubscribe messages**
- Send `Subscribe` message for workflow ID
- Send `Unsubscribe` message
- Trigger workflow event
- Assert no message received (unsubscribed)

**Test: Multiple WebSocket subscribers receive same event**
- Connect 3 WebSocket clients to same workflow
- Trigger workflow event
- Assert all 3 clients receive message

### 3.3 Notification System

**Test: Email notification sent on approval required**
- Create approval request
- Call `NotificationDispatcher.dispatch_approval_required`
- Assert email channel notified
- Assert email contains approval URL and timeout

**Test: Multiple notification channels all receive notification**
- Configure email, Slack, and FCM channels
- Dispatch approval required event
- Assert all 3 channels notified
- Assert no channel errors block others

**Test: Notification channel failure logged but doesn't block dispatch**
- Mock one channel to return error
- Dispatch approval required
- Assert other channels still notified
- Assert error logged

**Test: Escalation notification includes escalation level**
- Escalate approval to level 2
- Dispatch escalation notification
- Assert notification includes "Escalated to: Manager"
- Assert notification marked as urgent

### 3.4 Escalation Configuration

**Test: Escalation timeout triggers after configured minutes**
- Create approval request with timeout=60 minutes
- Set current time to +61 minutes
- Run escalation checker task
- Assert approval escalated to next level

**Test: Escalation chain progresses through levels**
- Configure 3-level escalation chain
- Let first level timeout
- Let second level timeout
- Assert escalated to level 3 (final)
- Assert third level timeout marks workflow as failed

**Test: Escalation notification sends to correct role**
- Escalate to supervisor level
- Assert notification sent to all users with "supervisor" role
- Assert users without "supervisor" role don't receive notification

**Test: Max escalations reached fails workflow**
- Create approval request with `max_escalations=2`
- Escalate through all levels
- Let final level timeout
- Assert workflow state = `Failed`
- Assert administrators notified of failure

**Test: Approval at escalated level clears escalation**
- Create escalated approval request
- Approve at level 2 (manager)
- Assert workflow resumes normal execution
- Assert no further escalation notifications sent

---

## Implementation

### 1. WebSocket Message Types

**File:** `server/crates/iou-orchestrator/src/websocket/messages.rs`

```rust
use serde::{Deserialize, Serialize};

/// Client-to-server messages (approverId from auth context, not input)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    Approve { request_id: String, comment: Option<String> },
    Modify { request_id: String, modifications: Vec<Modification>, comment: Option<String> },
    Reject { request_id: String, reason: String },
    Subscribe { workflow_id: String },
    Unsubscribe { workflow_id: String },
}

/// Server-to-client messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsServerMessage {
    ApprovalRequired { request: ApprovalRequest },
    WorkflowUpdated { workflow: Workflow },
    AgentStarted { workflow_id: String, agent: AgentType },
    AgentCompleted { workflow_id: String, agent: AgentType, result: AgentResult },
    AgentFailed { workflow_id: String, agent: AgentType, error: String },
    WorkflowCompleted { workflow_id: String },
    Error { message: String },
}
```

### 2. WebSocket Handler with Authentication

**File:** `server/crates/iou-orchestrator/src/websocket/handler.rs`

```rust
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    headers::Authorization,
    Response,
};
use uuid::Uuid;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
) -> Response {
    // Validate JWT before upgrade
    let credentials = auth_header.token();
    match validate_jwt_credentials(credentials, &state.jwt_secret).await {
        Ok(user_id) => {
            ws.on_upgrade(move |socket| handle_socket(socket, state, user_id))
        }
        Err(e) => {
            tracing::warn!("WebSocket auth failed: {}", e);
            return axum::response::Response::builder()
                .status(401)
                .body("Invalid authentication".into())
                .unwrap();
        }
    }
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<AppState>,
    user_id: Uuid,  // Authenticated user
) {
    // Subscribe to event bus
    let mut event_rx = state.event_bus.subscribe();

    // Handle incoming messages and outgoing events
    loop {
        tokio::select! {
            Some(msg_result) = socket.next() => {
                match msg_result {
                    Ok(msg) => handle_client_message(msg, &state, user_id).await,
                    Err(e) => break,
                }
            }
            Ok(event) = event_rx.recv() => {
                if let Some(ws_msg) = event_to_ws_message(event) {
                    let _ = socket.send(axum::extract::ws::Message::Text(
                        json!(ws_msg).to_string()
                    )).await;
                }
            }
        }
    }
}
```

### 3. Notification Channels

**File:** `server/crates/iou-orchestrator/src/notification/channels.rs`

```rust
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    async fn notify_approval_required(&self, request: &ApprovalRequest) -> Result<(), NotificationError>;
    async fn notify_approval_decision(&self, request_id: Uuid, decision: &ApprovalDecision) -> Result<(), NotificationError>;
    async fn notify_escalation(&self, request: &ApprovalRequest) -> Result<(), NotificationError>;
}

pub struct NotificationDispatcher {
    channels: Vec<Box<dyn NotificationChannel>>,
}
```

### 4. Escalation Configuration

**File:** `server/crates/iou-orchestrator/src/escalation/mod.rs`

```rust
pub struct EscalationConfig {
    pub timeout_minutes: u64,
    pub escalation_chain: Vec<EscalationLevel>,
    pub max_escalations: usize,
}

pub struct EscalationLevel {
    pub level: u32,
    pub approver_role: String,
    pub notification_channels: Vec<String>,
    pub timeout_minutes: Option<u64>,
}

// Background task checks for expired approvals and escalates
pub async fn run_escalation_checker(
    state: Arc<AppState>,
    check_interval_secs: u64,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(check_interval_secs));

    loop {
        interval.tick().await;

        // Find expired approvals
        let expired = state.approval_store.find_expired().await;

        for request in expired {
            if request.escalation_level < request.config.max_escalations {
                escalate_approval(&state, &request).await;
            } else {
                fail_workflow(&state, &request).await;
            }
        }
    }
}
```
