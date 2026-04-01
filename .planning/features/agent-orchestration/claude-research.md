# Agent Orchestration - Research Findings

## Executive Summary

Research covered codebase exploration and web research for implementing agent orchestration in IOU-Modern. Key findings:

1. **Existing State Machine** - Well-designed hand-rolled implementation exists (smlang not needed)
2. **Human-in-the-Loop UX** - Microsoft patterns provide proven approval workflow approaches
3. **Testing Infrastructure** - loom and tokio-test already configured
4. **Integration Points** - Clear paths to existing orchestrator and AI pipeline code

---

## 1. Codebase Analysis

### Async/Runtime Patterns

**Tokio** is the primary async runtime throughout:
- **Server workspace**: `tokio = { version = "1.43", features = ["full"] }`
- **Core workspace**: Optional Tokio with rt-multi-thread, macros, time features
- Patterns include: async agent execution, exponential backoff retry, `Arc<T>` for shared state, WebSocket real-time updates

### Existing Test Setup

**Comprehensive testing infrastructure:**
- Integration tests in `tests/` directories with `#[tokio::test]`
- Mock implementations in `/server/crates/iou-api/tests/mocks/`
- **loom** crate for deterministic concurrency testing
- DuckDB for embedded database testing
- End-to-end workflow testing patterns

### State Machine Patterns (Already Implemented!)

**Two complementary state machine systems exist:**

1. **Document Workflow System** (`/crates/iou-core/src/workflows/mod.rs`):
   - WorkflowStatus: Draft, Submitted, InReview, ChangesRequested, Approved, Published, Rejected, Archived
   - `can_transition_to()` validation

2. **Agent Orchestrator State Machine** (`/server/crates/iou-orchestrator/src/state_machine/base.rs`):
   - **States**: Created, Running, AwaitingApproval, AwaitingEscalation, Completed, Failed, Cancelled, Retrying, Archived
   - **Events**: Start, AgentComplete, AgentFailed, RetryAttempt, Approved, Modified, Rejected, Cancel, Archive, TimeoutEscalated
   - **Guards**: `can_proceed()`, `can_retry_any()`
   - **Audit logging** on every transition
   - **Serialization support** for checkpoint recovery

**Important Note**: The codebase explicitly documents why smlang was NOT chosen:

> *"Using hand-rolled state machine for better control and maintainability. The smlang DSL was evaluated but a custom implementation provides clearer error handling and better integration with our async workflow."*

### Multi-Agent Pipeline Pattern

**File**: `/server/crates/iou-ai/src/agents/pipeline.rs`

Sequential execution pattern:
- Research → Content → Compliance → Review
- Maker-checker iteration loop
- Exponential backoff retry for transient errors
- Checkpoint/restart capability
- Audit trail logging

### API Structure & Integration Points

**REST API with Axum**:
- Routes in `/server/crates/iou-api/src/routes/`
- Document creation, workflow status, approval endpoints
- WebSocket support for real-time updates
- S3 integration for document storage

**Database Layer**:
- **DuckDB** - Analytical workloads (embedded)
- **PostgreSQL** - Relational data (SQLx)
- **ArangoDB** - Graph operations (GraphRAG)

### Error Handling Patterns

**Three-tier error hierarchy:**

1. **API Errors** (`/server/crates/iou-api/src/error.rs`):
   - HTTP status mapping, user-friendly messages

2. **Pipeline Errors** (`/server/crates/iou-ai/src/agents/error.rs`):
   - **ErrorSeverity**: Transient (retry) vs Permanent (fail)

3. **Orchestrator Errors** (`/server/crates/iou-orchestrator/src/error.rs`):
   - State machine validation, checkpoint errors

---

## 2. Web Research Findings

### smlang & State Machines

**Recommendation: Do NOT use smlang**

The existing hand-rolled state machine is superior for this use case:

| Aspect | Hand-rolled (existing) | smlang |
|--------|------------------------|--------|
| Compile-time validation | Manual match exhaustiveness | Macro-enforced |
| Async support | Native tokio | Requires `async` guards syntax |
| Error messages | Custom, contextual | Macro-generated |
| Debugging | Direct Rust code | Generated code |
| Learning curve | Standard Rust | DSL-specific |

**Existing State Machine Implementation** (`base.rs`):

```rust
pub enum WorkflowState {
    Created, Running, AwaitingApproval, AwaitingEscalation,
    Completed, Failed, Cancelled, Retrying, Archived,
}

pub enum WorkflowEvent {
    Start, AgentComplete, AgentCompletePending, AllAgentsComplete,
    AgentFailed, RetryAttempt, MaxRetriesExceeded,
    Approved, Modified, Rejected, TimeoutEscalated,
    Cancel, Archive,
}

// Guard functions (from context.rs)
pub fn can_proceed(&self) -> bool {
    self.pending_approvals.is_empty()
}

pub fn can_retry_any(&self) -> bool {
    self.failed_agents.values().all(|&count| count < self.max_retries)
}
```

### Human-in-the-Loop UX Patterns

**Core Pattern: Checkpoint-Resume with Approval Events** (from Microsoft Agent Framework):

```
Agent produces work → Creates ApprovalRequest
    ↓
System pauses → Stores checkpoint
    ↓
Human reviews → Sends ApprovalDecision
    ↓
System resumes → Continues from checkpoint
```

**Approval Request Data Structure** (already exists in context.rs):

```rust
pub struct ApprovalRequest {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub agent: AgentType,
    pub result: AgentResult,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub escalated: bool,
    pub approver: Option<Uuid>,
}

pub enum ApprovalDecision {
    Approve { approver: Uuid, comment: Option<String> },
    Modify { approver: Uuid, modifications: Vec<HumanModification>, comment: Option<String> },
    Reject { approver: Uuid, reason: String },
    RequestChanges { approver: Uuid, feedback: String },
}
```

**Key UX Patterns:**

1. **Request-Response Cycle** - Agent → Approval → Decision → Resume
2. **Modification Support** - `HumanModification` tracks path, original/new values, reason
3. **Timeout with Escalation** - Approval deadlines with auto-escalation
4. **Notification System** - WebSocket/SSE real-time, email reminders

**Frontend WebSocket Pattern:**

```typescript
const socket = new WebSocket('/ws/workflows');

socket.on('approval_required', (request) => {
    showApprovalDialog({
        agent: request.agent,
        result: request.result,
        onApprove: () => sendDecision(request.id, 'approve'),
        onModify: (mods) => sendDecision(request.id, 'modify', mods),
        onReject: (reason) => sendDecision(request.id, 'reject', reason),
    });
});
```

### Async Testing Strategies

**loom for Deterministic Concurrency Testing** (already configured):

```toml
[workspace.dependencies]
loom = "0.7"
```

**Loom Use Cases:**
- Channel communication (mpsc, oneshot, watch)
- Concurrent state mutations
- Race conditions in state transitions
- Cancellation behavior

**Loom Testing Pattern:**

```rust
#[test]
fn test_concurrent_approval_processing() {
    loom::model(|| {
        let (tx, rx) = loom::sync::mpsc::channel(1);

        let t1 = loom::thread::spawn(move || {
            tx.send(ApprovalDecision::Approve { ... }).unwrap();
        });

        let t2 = loom::thread::spawn(move || {
            if let Some(decision) = rx.blocking_recv() {
                process_decision(decision);
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();
    });
}
```

**Tokio Test for Async Integration:**

```rust
use tokio_test::{assert_pending, assert_ready, task};

#[tokio::test]
async fn test_agent_execution_with_timeout() {
    let mut executor = task::spawn(run_agent(AgentType::Research));
    assert_pending!(executor.poll());
    let result = executor.await;
    assert!(result.is_ok());
}
```

**Test Organization (recommended):**

```
tests/
├── state_machine_tests.rs     # State transition logic
├── state_machine_recovery.rs  # Checkpoint restore logic
├── agent_execution_tests.rs   # Agent executor logic
├── approval_workflow_tests.rs # Human-in-the-loop integration
└── loom/
    └── concurrent_tests.rs    # Deterministic concurrency tests
```

---

## 3. Integration Points

### Files to Extend

| File | Purpose |
|------|---------|
| `server/crates/iou-orchestrator/src/state_machine/base.rs` | State machine implementation |
| `server/crates/iou-orchestrator/src/context.rs` | Context and approval types |
| `server/crates/iou-orchestrator/src/stage_executor.rs` | Stage execution engine |
| `server/crates/iou-ai/src/agents/pipeline.rs` | Agent pipeline pattern |
| `server/crates/iou-api/src/routes/` | API endpoints |

### Dependencies Already Available

- **tokio** - Async runtime
- **loom** - Concurrency testing
- **duckdb** - Embedded database
- **axum** - Web framework with WebSocket
- **serde** - Serialization
- **thiserror** - Error handling
- **tracing** - Structured logging

### Components to Implement

1. **Checkpoint Persistence** - Implement `CheckpointStorage` trait with DuckDB
2. **WebSocket/Real-time** - Approval notifications via axum WebSocket
3. **Agent Integration** - Connect existing `iou-ai` agents to orchestrator
4. **Audit Log Persistence** - Persist `AuditEntry` types

---

## 4. Key Recommendations

1. **Keep the existing hand-rolled state machine** - Better error handling, native async, well-tested
2. **Implement checkpoint-resume using DuckDB** - Database dependency exists
3. **Use WebSocket for real-time approvals** - axum WebSocket support available
4. **Add loom tests for concurrent scenarios** - Already configured in workspace
5. **Leverage existing Microsoft patterns** - Checkpoint-resume and approval event patterns proven
6. **Implement the audit log** - `AuditEntry` type exists, just needs persistence

---

## 5. Testing Strategy Summary

| Component | Testing Tool | Focus |
|-----------|--------------|-------|
| State transitions | cargo test | Unit tests for all transitions |
| Concurrency | loom | Channel communication, races |
| Async execution | tokio-test | Agent execution, timeouts |
| Integration | Integration tests | Full workflow with mocks |

---

*Research completed via codebase exploration and web research on state machines, human-in-the-loop UX patterns, and async testing strategies.*
