# Agent Orchestration - Implementation Plan

## Introduction

This plan describes implementing an agent orchestration system for IOU-Modern that coordinates AI agents in a document creation workflow with human-in-the-loop approvals. The system supports dynamic DAG-based execution, adaptive checkpointing, priority queue scheduling, and comprehensive observability.

**What we're building:** A production-ready workflow orchestrator that manages AI agents (Research, Content, Compliance, Review) with human approval checkpoints, crash recovery, and real-time status updates.

**Why:** The existing multi-agent pipeline lacks dynamic execution, robust recovery, and complete human-in-the-loop integration. This system fills those gaps while leveraging existing state machine and agent infrastructure.

**How:** Extend the existing orchestrator crate with DAG execution, enhance the state machine for new approval states, use PostgreSQL for operational data, build GraphQL + WebSocket API, and add comprehensive observability.

---

## Architecture Overview

### System Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Layer                            │
│  (GraphQL + WebSocket for real-time updates)                │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    API Layer                                 │
│  (GraphQL queries/mutations + WebSocket subscriptions)      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                Workflow Scheduler                            │
│  (Priority queue, preemption, multi-workflow execution)     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Orchestrator                              │
│  ┌─────────────┐  ┌──────────┐  ┌─────────────────┐        │
│  │ State       │  │ Event    │  │ Checkpoint      │        │
│  │ Machine     │  │ Bus      │  │ Manager         │        │
│  └─────────────┘  └──────────┘  └─────────────────┘        │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                 Agent Executor                               │
│  (DAG builder, parallel execution, agent coordination)      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      Agents                                  │
│  (Research → Content → Compliance → Review)                 │
│  (Existing iou-ai agents)                                    │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility |
|-----------|---------------|
| GraphQL API | Queries, mutations, subscriptions for workflows |
| WebSocket Server | Real-time workflow updates, approval notifications |
| Workflow Scheduler | Priority queue, workflow preemption, resource allocation |
| State Machine | Workflow state transitions, guard evaluation, audit logging |
| Event Bus | Async event distribution between components |
| Checkpoint Manager | State snapshots, crash recovery |
| Agent Executor | DAG building, parallel agent execution |
| Agents | Domain-specific document processing (existing) |

---

## Phase 1: Core Orchestration

### 1.1 Enhance State Machine for DAG Execution

**Location:** `server/crates/iou-orchestrator/src/state_machine/`

**Current State:** The existing state machine (`base.rs`) defines states and events for sequential agent execution with approval points.

**Required Changes:**

1. **Add DAG-specific states:**
   - `ParallelExecuting` - Multiple agents running concurrently
   - `PartialComplete` - Some agents complete, awaiting others
   - `MergeResults` - Combining results from parallel agents

2. **Add DAG-specific events:**
   - `DagBuilt` - DAG constructed from dependencies
   - `AgentParallelStart` - Start multiple independent agents
   - `PartialAgentComplete` - One agent in parallel set completes
   - `DagLayerComplete` - All agents in a DAG layer complete

3. **Extend guard functions:**
   - `can_execute_parallel()` - Check if agents can run in parallel
   - `all_dependents_complete()` - Verify dependencies satisfied
   - `can_merge_results()` - Check if all parallel results ready

**Type Definitions:**

```rust
// Extended state enum
pub enum WorkflowState {
    // Existing states...
    Created,
    Running,
    AwaitingApproval,
    AwaitingEscalation,
    Completed,
    Failed,
    Cancelled,
    Retrying,
    Archived,

    // New DAG states
    ParallelExecuting,
    PartialComplete,
    MergeResults,
}

// DAG layer tracking
pub struct DagLayer {
    pub layer_index: usize,
    pub agents: Vec<AgentType>,
    pub completed: HashSet<AgentType>,
}

// DAG structure
pub struct ExecutionDag {
    pub layers: Vec<DagLayer>,
    pub current_layer: Option<usize>,
}
```

**Integration Point:** Extend existing `WorkflowStateMachine` struct to include `ExecutionDag` field.

**State Transition Table:**

| Current State | Event | Next State | Guard | Notes |
|--------------|-------|------------|-------|-------|
| Created | Start | Running | - | Workflow started |
| Running | DagBuilt | ParallelExecuting | DAG valid | DAG constructed |
| Running | AgentComplete | AwaitingApproval | requires_approval | Agent output needs approval |
| ParallelExecuting | PartialAgentComplete | PartialComplete | any_complete | Some agents done |
| ParallelExecuting | DagLayerComplete | MergeResults | all_complete | All layer agents done |
| PartialComplete | PartialAgentComplete | PartialComplete | more_pending | Waiting for more agents |
| PartialComplete | DagLayerComplete | MergeResults | all_complete | Ready to merge |
| MergeResults | MergeComplete | Running | merge_success | Continue next layer |
| MergeResults | AgentComplete | AwaitingApproval | requires_approval | Merged result needs approval |
| AwaitingApproval | ApprovalDecision | Running | approved | Continue execution |
| AwaitingApproval | ApprovalTimeout | AwaitingEscalation | timeout_elapsed | Escalate to supervisor |
| AwaitingEscalation | EscalationDecision | Running | approved | Resume after escalation |
| AwaitingEscalation | EscalationTimeout | Failed | timeout_max | Max retries exceeded |
| Running | AllAgentsComplete | Completed | - | Workflow finished |
| Any | Cancel | Cancelled | - | User cancelled |
| Any | FatalError | Failed | - | Unrecoverable error |

**Error Handling:**
- If an agent fails during `ParallelExecuting`, cancel all agents in the layer and transition to `Failed`
- If DAG execution gets stuck (no progress for configured timeout), transition to `Failed`
- Partial results cannot be approved while other agents are still running

---

### 1.2 Build DAG from Agent Dependencies

**Location:** `server/crates/iou-orchestrator/src/dag/` (new module)

**Purpose:** Convert agent dependency definitions into executable layers.

**Agent Dependency Definition:**

```rust
pub struct AgentDependency {
    pub agent: AgentType,
    pub depends_on: Vec<AgentType>,
}

// Known dependencies for document creation workflow
const DOCUMENT_WORKFLOW_DEPS: &[AgentDependency] = &[
    AgentDependency { agent: AgentType::Research, depends_on: vec![] },
    AgentDependency { agent: AgentType::Content, depends_on: vec![AgentType::Research] },
    AgentDependency { agent: AgentType::Compliance, depends_on: vec![AgentType::Content] },
    AgentDependency { agent: AgentType::Review, depends_on: vec![AgentType::Compliance] },
];
```

**Core Function:**

```rust
pub fn build_execution_dag(agents: &[AgentDependency]) -> Result<ExecutionDag, DagError>
```

**Algorithm:**
1. Validate no circular dependencies
2. Perform topological sort
3. Group independent agents into layers
4. Return `ExecutionDag` with ordered layers

**Example DAG for document workflow:**
- Layer 0: Research (no dependencies)
- Layer 1: Content (depends on Research)
- Layer 2: Compliance (depends on Content)
- Layer 3: Review (depends on Compliance)

---

### 1.3 Parallel Agent Executor

**Location:** `server/crates/iou-orchestrator/src/executor/` (new module)

**Purpose:** Execute agents within a DAG layer in parallel using Tokio tasks.

**Agent Trait Definition:**

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&self, input: AgentInput) -> Result<AgentOutput, AgentError>;
    fn agent_type(&self) -> AgentType;
    fn dependencies(&self) -> Vec<AgentType>;
}

pub struct AgentInput {
    pub workflow_id: Uuid,
    pub context: WorkflowContext,
    pub dependencies: HashMap<AgentType, AgentOutput>,
}

pub struct AgentOutput {
    pub agent_type: AgentType,
    pub status: ExecutionStatus,
    pub result: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

pub enum ExecutionStatus {
    Success,
    PartialSuccess,
    Failed,
}
```

**Core Function:**

```rust
pub async fn execute_layer_parallel(
    agents: Vec<AgentType>,
    context: Arc<RwLock<WorkflowContext>>,
) -> Result<HashMap<AgentType, AgentResult>, ExecutorError>
```

**Shared State Synchronization:**
- `WorkflowContext` wrapped in `Arc<RwLock<T>>` for concurrent access
- Each agent task gets a read-only clone of the context for input
- Agent outputs are written through a dedicated channel to serialize writes
- Results are collected and merged by the executor task

**Execution Flow:**
1. Spawn Tokio task for each agent in the layer
2. Use `tokio::spawn` with `JoinSet` for managed concurrency
3. Each agent reads from `context` (read lock) and sends output via channel
4. Executor task collects outputs and writes to `context` (write lock)
5. Collect results as tasks complete
6. Return map of agent → result
7. Handle per-agent failures with configured retry policy

**Cancellation:**
- If one agent fails permanently, send cancellation signal to all agents in layer
- Use `tokio_util::sync::CancellationToken` for coordinated cancellation
- Agents must check cancellation token periodically

**Error Handling:**
- Transient errors: Retry with exponential backoff
- Permanent errors: Cancel layer, fail workflow, mark agent as failed
- Partial failures: Continue if agent marked optional

**Retry Configuration:**

```rust
pub struct AgentRetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

// Per-agent default configurations
impl Default for AgentRetryConfig {
    fn default() -> Self {
        AgentRetryConfig {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}
```

---

### 1.4 Event Bus for Component Communication

**Location:** `server/crates/iou-orchestrator/src/event_bus/` (new module)

**Purpose:** Async communication between orchestrator components using tokio channels.

**Event Types:**

```rust
pub enum OrchestratorEvent {
    WorkflowCreated { id: Uuid },
    WorkflowStarted { id: Uuid },
    AgentStarted { workflow_id: Uuid, agent: AgentType },
    AgentCompleted { workflow_id: Uuid, agent: AgentType, result: AgentResult },
    AgentFailed { workflow_id: Uuid, agent: AgentType, error: String },
    ApprovalRequired { workflow_id: Uuid, request: ApprovalRequest },
    ApprovalDecision { workflow_id: Uuid, decision: ApprovalDecision },
    WorkflowCompleted { id: Uuid },
    WorkflowFailed { id: Uuid, reason: String },
    CheckpointSaved { workflow_id: Uuid },
}
```

**Bus Structure:**

```rust
pub struct EventBus {
    // Broadcast to all subscribers (may drop slow receivers)
    broadcast_tx: broadcast::Sender<OrchestratorEvent>,
    // Bounded channel for critical events (audit, compliance)
    critical_tx: mpsc::Sender<OrchestratorEvent>,
    // Command channel for workflow control
    command_tx: mpsc::Sender<WorkflowCommand>,
}

pub struct EventBusSubscriber {
    rx: broadcast::Receiver<OrchestratorEvent>,
}

// Critical event channel settings
const CRITICAL_CHANNEL_SIZE: usize = 1000;
```

**Backpressure Handling:**
- **Broadcast channel** (`broadcast::Sender`): Used for non-critical updates (UI notifications, metrics). Slow receivers are dropped - they miss events but don't block the system.
- **Critical channel** (`mpsc::Sender`): Bounded channel for events that must not be lost (audit, compliance). When full, `send()` returns an error - the caller must handle backpressure (queue, retry, or fail fast).
- **Audit logger** subscribes to critical channel with a dedicated task that batches writes to storage

**Usage Pattern:**
1. Components subscribe to event types
2. Orchestrator publishes events on state transitions
3. Critical events (approval decisions, state changes) go to `critical_tx`
4. Non-critical events (agent progress) go to `broadcast_tx`
5. Subscribers react asynchronously (notifier, checkpoint manager, audit logger)

---

## Phase 2: Checkpoint & Recovery (PostgreSQL)

### 2.1 Adaptive Checkpoint Policy

**Location:** `server/crates/iou-orchestrator/src/checkpoint/policy.rs`

**Purpose:** Define when checkpoints should be saved based on workflow configuration and progress.

**Policy Configuration:**

```rust
pub struct CheckpointPolicy {
    // Checkpoint after every N agents
    pub agent_interval: Option<usize>,
    // Checkpoint at approval points
    pub at_approvals: bool,
    // Checkpoint after duration (ms)
    pub time_interval: Option<u64>,
    // Minimum progress percentage for checkpoint
    pub min_progress_pct: Option<u8>,
}

// Workflow-specific policies
pub fn get_policy_for_workflow(workflow_type: WorkflowType) -> CheckpointPolicy {
    match workflow_type {
        WorkflowType::DocumentCreation => CheckpointPolicy {
            agent_interval: Some(1),  // After every agent
            at_approvals: true,
            time_interval: Some(60000),  // Every minute
            min_progress_pct: Some(10),  // At least 10% progress
        },
        // ... other workflow types
    }
}
```

**Decision Function:**

```rust
pub fn should_checkpoint(
    policy: &CheckpointPolicy,
    context: &WorkflowContext,
    last_checkpoint: DateTime<Utc>,
) -> bool
```

**Check conditions:**
- Agent count since last checkpoint ≥ interval
- At approval point and policy enabled
- Time elapsed ≥ interval
- Progress threshold exceeded

---

### 2.2 Checkpoint Data Structures

**Location:** `server/crates/iou-orchestrator/src/checkpoint/model.rs`

**Checkpoint Snapshot:**

```rust
pub struct WorkflowCheckpoint {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub state: WorkflowState,
    pub completed_agents: HashSet<AgentType>,
    pub pending_agents: HashSet<AgentType>,
    pub agent_results: HashMap<AgentType, AgentResult>,
    pub current_dag_layer: Option<usize>,
    pub context_snapshot: serde_json::Value,
}
```

---

### 2.3 PostgreSQL Checkpoint Storage

**Location:** `server/crates/iou-orchestrator/src/checkpoint/storage.rs`

**Purpose:** Persist workflow checkpoints to PostgreSQL for crash recovery.

**Note on Storage Choice:** PostgreSQL is used for operational workflow state because:
- Already in the technology stack
- ACID compliant for reliable state transitions
- Excellent for transactional operations (create, update, query workflows)
- Supports JSONB for flexible metadata storage
- Proven operational reliability

DuckDB (optional, future): Can be added later for analytical queries on workflow history, performance metrics, and patterns.

**Storage Trait:**

```rust
#[async_trait]
pub trait CheckpointStorage: Send + Sync {
    async fn save_checkpoint(&self, checkpoint: &WorkflowCheckpoint) -> Result<(), StorageError>;
    async fn load_latest_checkpoint(&self, workflow_id: Uuid) -> Result<Option<WorkflowCheckpoint>, StorageError>;
    async fn list_checkpoints(&self, workflow_id: Uuid) -> Result<Vec<WorkflowCheckpoint>, StorageError>;
    async fn delete_old_checkpoints(&self, workflow_id: Uuid, keep_last: usize) -> Result<(), StorageError>;
}
```

**PostgreSQL Implementation:**

```rust
pub struct PgCheckpointStorage {
    pool: PgPool,
}

impl PgCheckpointStorage {
    pub async fn new(pool: PgPool) -> Result<Self, StorageError> {
        // Run migration to create table if not exists
        sqlx::query("
            CREATE TABLE IF NOT EXISTS workflow_checkpoints (
                id UUID PRIMARY KEY,
                workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                state TEXT NOT NULL,
                completed_agents TEXT[] NOT NULL,
                pending_agents TEXT[] NOT NULL,
                agent_results JSONB NOT NULL,
                current_dag_layer INTEGER,
                context_snapshot JSONB NOT NULL,
                version INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS idx_workflow_checkpoints_workflow_id
                ON workflow_checkpoints(workflow_id, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_workflow_checkpoints_state
                ON workflow_checkpoints(state);
        ").execute(&pool).await?;
        Ok(PgCheckpointStorage { pool })
    }

    async fn save_checkpoint(&self, checkpoint: &WorkflowCheckpoint) -> Result<(), StorageError> {
        sqlx::query("
            INSERT INTO workflow_checkpoints
                (id, workflow_id, state, completed_agents, pending_agents,
                 agent_results, current_dag_layer, context_snapshot, version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (workflow_id) DO UPDATE SET
                timestamp = NOW(),
                state = EXCLUDED.state,
                completed_agents = EXCLUDED.completed_agents,
                pending_agents = EXCLUDED.pending_agents,
                agent_results = EXCLUDED.agent_results,
                current_dag_layer = EXCLUDED.current_dag_layer,
                context_snapshot = EXCLUDED.context_snapshot,
                version = EXCLUDED.version
        ")
        .bind(checkpoint.id)
        .bind(checkpoint.workflow_id)
        .bind(&checkpoint.state.to_string())
        .bind(&checkpoint.completed_agents)
        .bind(&checkpoint.pending_agents)
        .bind(serde_json::to_value(&checkpoint.agent_results)?)
        .bind(checkpoint.current_dag_layer)
        .bind(serde_json::to_value(&checkpoint.context_snapshot)?)
        .bind(CURRENT_CHECKPOINT_VERSION)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load_latest_checkpoint(&self, workflow_id: Uuid) -> Result<Option<WorkflowCheckpoint>, StorageError> {
        let row = sqlx::query_as::<_, (Uuid, Uuid, String, Vec<String>, Vec<String>, serde_json::Value, Option<i32>, serde_json::Value, i32)>(
            "SELECT id, workflow_id, state, completed_agents, pending_agents,
                    agent_results, current_dag_layer, context_snapshot, version
             FROM workflow_checkpoints
             WHERE workflow_id = $1
             ORDER BY timestamp DESC
             LIMIT 1"
        )
        .bind(workflow_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|(id, workflow_id, state, completed_agents, pending_agents, agent_results, current_dag_layer, context_snapshot, version)| {
            Ok(WorkflowCheckpoint {
                id, workflow_id,
                timestamp: Utc::now(), // Fetch from query if needed
                state: state.parse()?,
                completed_agents: completed_agents.into_iter().filter_map(|s| s.parse().ok()).collect(),
                pending_agents: pending_agents.into_iter().filter_map(|s| s.parse().ok()).collect(),
                agent_results: serde_json::from_value(agent_results)?,
                current_dag_layer: current_dag_layer.map(|v| v as usize),
                context_snapshot,
                version: version as u32,
            })
        }).transpose()
    }
}
```

**Benefits of PostgreSQL:**
- Native async support with `sqlx` - no `spawn_blocking` needed
- ACID transactions ensure checkpoint integrity
- Array types for `completed_agents`/`pending_agents`
- JSONB for flexible metadata with indexing support
- Foreign key constraints to workflows table
- Upsert with `ON CONFLICT` for idempotent saves

---

### 2.4 Recovery Workflow

**Location:** `server/crates/iou-orchestrator/src/checkpoint/recovery.rs`

**Purpose:** Restore workflow state after crash or restart.

**Recovery Function:**

```rust
pub async fn recover_workflow(
    workflow_id: Uuid,
    storage: &dyn CheckpointStorage,
    state_machine: &mut WorkflowStateMachine,
) -> Result<RecoveryPlan, RecoveryError>
```

**Recovery Steps:**
1. Load latest checkpoint from PostgreSQL
2. Validate checkpoint version compatibility
3. **If version incompatible, apply migration or return error**
4. Restore state machine to checkpointed state
5. Rebuild in-memory context from snapshot
6. Determine next agents to execute
7. Generate recovery plan

**Version Management:**

```rust
// Current checkpoint schema version
const CURRENT_CHECKPOINT_VERSION: u32 = 1;

pub struct WorkflowCheckpoint {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub state: WorkflowState,
    pub completed_agents: Vec<AgentType>,
    pub pending_agents: Vec<AgentType>,
    pub agent_results: HashMap<AgentType, AgentResult>,
    pub current_dag_layer: Option<usize>,
    pub context_snapshot: serde_json::Value,
    pub version: u32,  // Checkpoint schema version
}

#[derive(Error, Debug)]
pub enum RecoveryError {
    #[error("Checkpoint not found for workflow {0}")]
    NotFound(Uuid),
    #[error("Checkpoint version {0} incompatible with current version {CURRENT_CHECKPOINT_VERSION}")]
    IncompatibleVersion(u32),
    #[error("Migration from version {0} to {1} failed: {2}")]
    MigrationFailed(u32, u32, String),
}
```

**Migration Strategy:**

When `CURRENT_CHECKPOINT_VERSION` is incremented:
1. Add a migration function for the old version
2. Attempt migration during recovery
3. If migration fails, return `MigrationFailed` error
4. Document what data is preserved vs. discarded

```rust
fn migrate_checkpoint(mut checkpoint: WorkflowCheckpoint, target_version: u32) -> Result<WorkflowCheckpoint, MigrationError> {
    match (checkpoint.version, target_version) {
        (1, 2) => {
            // Add new field with default value
            checkpoint.version = 2;
            Ok(checkpoint)
        }
        (from, to) => Err(MigrationError::NoMigrationPath { from, to }),
    }
}
```

**Recovery Plan:**

```rust
pub struct RecoveryPlan {
    pub workflow_id: Uuid,
    pub restored_state: WorkflowState,
    pub pending_agents: Vec<AgentType>,
    pub restore_results: HashMap<AgentType, AgentResult>,
    pub next_dag_layer: usize,
    pub was_migrated: bool,
}
```

---

## Phase 3: Human-in-the-Loop

### 3.1 GraphQL API Schema

**Location:** `server/crates/iou-orchestrator/src/graphql/schema.rs`

**Schema Definition:**

```graphql
type Workflow {
    id: ID!
    status: WorkflowStatus!
    priority: Priority!
    currentAgent: AgentType
    pendingApprovals: [ApprovalRequest!]!
    completedAgents: [AgentType!]!
    createdAt: DateTime!
    updatedAt: DateTime!
}

type ApprovalRequest {
    id: ID!
    workflowId: ID!
    agent: AgentType!
    result: AgentResult!
    createdAt: DateTime!
    expiresAt: DateTime!
    escalated: Boolean!
    approver: String
}

type AgentResult {
    agent: AgentType!
    status: String!
    output: String
    metadata: JSON
}

enum WorkflowStatus {
    CREATED
    RUNNING
    AWAITING_APPROVAL
    AWAITING_ESCALATION
    COMPLETED
    FAILED
    CANCELLED
    RETRYING
    ARCHIVED
}

enum Priority {
    CRITICAL
    HIGH
    NORMAL
    LOW
}

enum AgentType {
    RESEARCH
    CONTENT
    COMPLIANCE
    REVIEW
}

input CreateWorkflowInput {
    documentType: String!
    priority: Priority
    metadata: JSON
}

input ApprovalInput {
    requestId: ID!
    comment: String
}

input ModificationInput {
    requestId: ID!
    modifications: [ModificationInput!]!
    comment: String
}

input ModificationFieldInput {
    path: String!
    newValue: JSON!
}

type Query {
    workflow(id: ID!): Workflow
    workflows(filter: WorkflowFilter): [Workflow!]!
    pendingApprovals: [ApprovalRequest!]!
}

type Mutation {
    createWorkflow(input: CreateWorkflowInput!): Workflow!
    approve(input: ApprovalInput!): Workflow!
    modify(input: ModificationInput!): Workflow!
    reject(requestId: ID!, approverId: String!, reason: String!): Workflow!
    cancelWorkflow(id: ID!): Workflow!
}

type Subscription {
    workflowUpdated(id: ID!): Workflow!
    approvalRequired: ApprovalRequest!
    agentCompleted(workflowId: ID!): AgentResult!
}

input WorkflowFilter {
    status: WorkflowStatus
    priority: Priority
    createdBy: String
    dateFrom: DateTime
    dateTo: DateTime
}
```

---

### 3.2 WebSocket Server

**Location:** `server/crates/iou-orchestrator/src/websocket/`

**Purpose:** Real-time delivery of workflow updates and approval notifications.

**WebSocket Message Types:**

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    Approve { request_id: String, comment: Option<String> },
    Modify { request_id: String, modifications: Vec<Modification>, comment: Option<String> },
    Reject { request_id: String, reason: String },
    Subscribe { workflow_id: String },
    Unsubscribe { workflow_id: String },
}

#[derive(Serialize, Deserialize)]
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

**WebSocket Handler with Authentication:**

```rust
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
) -> Response {
    // Validate JWT credentials before upgrade
    let credentials = auth_header.token();
    match validate_jwt_credentials(credentials, &state.jwt_secret).await {
        Ok(user_id) => {
            ws.on_upgrade(move |socket| handle_socket(socket, state, user_id))
        }
        Err(e) => {
            tracing::warn!("WebSocket auth failed: {}", e);
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Invalid authentication".into())
                .unwrap();
        }
    }
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<AppState>,
    user_id: String,  // Authenticated user
) {
    // Subscribe to event bus
    let mut event_rx = state.event_bus.subscribe();

    // Handle incoming messages and outgoing events
    loop {
        tokio::select! {
            Some(msg_result) = socket.next() => {
                match msg_result {
                    Ok(msg) => handle_client_message(msg, &state).await,
                    Err(e) => break,
                }
            }
            Ok(event) = event_rx.recv() => {
                let ws_msg = event_to_ws_message(event);
                if let Some(msg) = ws_msg {
                    let _ = socket.send(Message::Text(json!(msg).to_string())).await;
                }
            }
        }
    }
}
```

---

### 3.3 Notification System

**Location:** `server/crates/iou-orchestrator/src/notification/`

**Purpose:** Multi-channel notifications for approval requests.

**Notification Channels:**

```rust
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    async fn notify_approval_required(&self, request: &ApprovalRequest) -> Result<(), NotificationError>;
    async fn notify_approval_decision(&self, request_id: Uuid, decision: &ApprovalDecision) -> Result<(), NotificationError>;
    async fn notify_escalation(&self, request: &ApprovalRequest) -> Result<(), NotificationError>;
}

// WebSocket notification (via WebSocket subscribers)
pub struct WebSocketNotifier {
    event_bus: Arc<EventBus>,
}

// Email notification
pub struct EmailNotifier {
    smtp_config: SmtpConfig,
    template_engine: Arc<TemplateEngine>,
}

// Dashboard queue (in-app notification)
pub struct DashboardNotifier {
    // Store notifications in database for dashboard polling
    db: Arc<PgPool>,
}

// Mobile push notification
pub struct MobilePushNotifier {
    fcm_client: Arc<FcmClient>,
}
```

**Notification Dispatcher:**

```rust
pub struct NotificationDispatcher {
    channels: Vec<Box<dyn NotificationChannel>>,
}

impl NotificationDispatcher {
    pub async fn dispatch_approval_required(&self, request: &ApprovalRequest) {
        for channel in &self.channels {
            if let Err(e) = channel.notify_approval_required(request).await {
                tracing::error!("Notification channel failed: {}", e);
            }
        }
    }
}
```

---

### 3.4 Escalation Configuration

**Location:** `server/crates/iou-orchestrator/src/escalation/`

**Purpose:** Configure timeout-based escalation for unapproved approvals.

**Escalation Configuration:**

```rust
pub struct EscalationConfig {
    pub timeout_minutes: u64,
    pub escalation_chain: Vec<EscalationLevel>,
    pub max_escalations: usize,
}

pub struct EscalationLevel {
    pub level: u32,
    pub approver_role: String,  // e.g., "supervisor", "manager", "admin"
    pub notification_channels: Vec<String>,  // e.g., ["email", "slack"]
    pub timeout_minutes: Option<u64>,  // None = no further escalation
}

// Default configuration for document creation workflow
impl Default for EscalationConfig {
    fn default() -> Self {
        EscalationConfig {
            timeout_minutes: 60,  // 1 hour before first escalation
            escalation_chain: vec![
                EscalationLevel {
                    level: 1,
                    approver_role: "supervisor".to_string(),
                    notification_channels: vec!["email".to_string(), "slack".to_string()],
                    timeout_minutes: Some(120),  // 2 hours to next level
                },
                EscalationLevel {
                    level: 2,
                    approver_role: "manager".to_string(),
                    notification_channels: vec!["email".to_string(), "slack".to_string(), "sms".to_string()],
                    timeout_minutes: None,  // Final level
                },
            ],
            max_escalations: 2,
        }
    }
}
```

**Escalation Flow:**

1. Approval request created with `expires_at = now + timeout_minutes`
2. Background task checks for expired approvals every minute
3. On expiration:
   - Find current escalation level from request
   - If `max_escalations` not reached, escalate to next level
   - Update request with new `expires_at` and `escalated = true`
   - Send notifications to escalation level approvers
4. If max escalations reached:
   - Transition workflow to `Failed` state
   - Notify administrators of failed escalation

**Authorization:**
- Only users with matching `approver_role` can approve escalated requests
- Role membership stored in PostgreSQL `user_roles` table
- Checked during approval mutation via auth context

---

## Phase 4: Workflow Scheduling

### 4.1 Priority Queue Implementation

**Location:** `server/crates/iou-orchestrator/src/scheduler/queue.rs`

**Purpose:** Manage workflow queue with priority-based execution.

**Queue Entry:**

```rust
pub struct QueuedWorkflow {
    pub workflow_id: Uuid,
    pub priority: Priority,
    pub queued_at: DateTime<Utc>,
    pub estimated_duration: Option<Duration>,
    pub preemptible: bool,
}

// Priority ordering (CRITICAL > HIGH > NORMAL > LOW)
// BinaryHeap is max-heap, so Lower ordering = Higher priority
impl Ord for QueuedWorkflow {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.priority, other.priority) {
            (Priority::Critical, Priority::Critical) => self.queued_at.cmp(&other.queued_at),
            (Priority::Critical, _) => Ordering::Less,  // Critical is higher priority
            (_, Priority::Critical) => Ordering::Greater,
            (Priority::High, Priority::High) => self.queued_at.cmp(&other.queued_at),
            (Priority::High, Priority::Normal | Priority::Low) => Ordering::Less,
            (Priority::Normal, Priority::High) => Ordering::Greater,
            (Priority::Normal, Priority::Normal) => self.queued_at.cmp(&other.queued_at),
            (Priority::Normal, Priority::Low) => Ordering::Less,
            (Priority::Low, Priority::High | Priority::Normal) => Ordering::Greater,
            (Priority::Low, Priority::Low) => self.queued_at.cmp(&other.queued_at),
        }
    }
}

impl PartialOrd for QueuedWorkflow {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueuedWorkflow {
    fn eq(&self, other: &Self) -> bool {
        self.workflow_id == other.workflow_id
    }
}

impl Eq for QueuedWorkflow {}
```

**Priority Queue:**

```rust
pub struct WorkflowPriorityQueue {
    inner: BinaryHeap<QueuedWorkflow>,
    max_concurrent: usize,
    running_count: AtomicUsize,
}

impl WorkflowPriorityQueue {
    pub async fn push(&self, workflow: QueuedWorkflow) -> QueueResult<()> {
        self.inner.lock().await.push(workflow);
        Ok(())
    }

    pub async fn pop(&self) -> Option<QueuedWorkflow> {
        let mut inner = self.inner.lock().await;
        // Check and increment running_count INSIDE the lock to prevent TOCTOU race
        if self.running_count.load(Ordering::Relaxed) < self.max_concurrent {
            self.running_count.fetch_add(1, Ordering::Relaxed);
            inner.pop()
        } else {
            None
        }
    }

    pub fn complete(&self) {
        self.running_count.fetch_sub(1, Ordering::Relaxed);
    }
}
```

---

### 4.2 Workflow Scheduler with Preemption

**Location:** `server/crates/iou-orchestrator/src/scheduler/mod.rs`

**Purpose:** Coordinate workflow execution with preemption support.

**Scheduler Configuration:**

```rust
pub struct SchedulerConfig {
    pub max_concurrent_workflows: usize,
    pub preemption_enabled: bool,
    pub preemption_grace_period_ms: u64,
    pub checkpoint_before_preempt: bool,
}
```

**Preemption Decision:**

```rust
pub fn should_preempt(
    queued: &QueuedWorkflow,
    running: &[RunningWorkflow],
    config: &SchedulerConfig,
) -> Option<Uuid>  // Returns workflow ID to preempt
```

**Preemption Rules:**
- CRITICAL can preempt NORMAL and LOW
- HIGH can preempt LOW
- Preemptible workflows are preferred targets
- Must checkpoint before preempting (if configured)

**Preemption Flow:**
1. New high-priority workflow arrives
2. Check if capacity available
3. If full, find lowest priority preemptible workflow
4. Send checkpoint request to running workflow
5. Wait for checkpoint to complete
6. Pause workflow, save state
7. Start new workflow
8. Preempted workflow added back to queue

---

## Phase 5: Observability

### 5.1 Structured Logging

**Location:** Already integrated via `tracing` crate

**Log Points:**
- Workflow state transitions (with workflow_id)
- Agent start/completion (with timing)
- Checkpoint save/load (with duration)
- Approval request/decision
- Errors (with full context)

**Example:**

```rust
tracing::info!(
    workflow_id = %workflow_id,
    agent = ?agent_type,
    duration_ms = execution_time.as_millis(),
    "Agent completed successfully"
);
```

---

### 5.2 Prometheus Metrics

**Location:** `server/crates/iou-orchestrator/src/metrics.rs`

**Metrics to Track:**

```rust
// Workflow metrics
lazy_static! {
    static ref WORKFLOW_CREATED: IntCounter = IntCounter::new(
        "workflow_created_total", "Total workflows created"
    ).unwrap();
    static ref WORKFLOW_COMPLETED: IntCounter = IntCounter::new(
        "workflow_completed_total", "Total workflows completed"
    ).unwrap();
    static ref WORKFLOW_FAILED: IntCounter = IntCounter::new(
        "workflow_failed_total", "Total workflows failed"
    ).unwrap();
    static ref WORKFLOW_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new("workflow_duration_seconds", "Workflow execution time")
            .buckets(vec![0.1, 1.0, 10.0, 60.0, 300.0])
    ).unwrap();

    // Agent metrics
    static ref AGENT_EXECUTION_TIME: HistogramVec = HistogramVec::new(
        HistogramOpts::new("agent_execution_time_seconds", "Agent execution time"),
        &["agent_type"]
    ).unwrap();
    static ref AGENT_SUCCESS_RATE: GaugeVec = GaugeVec::new(
        "agent_success_rate", "Agent success rate",
        &["agent_type"]
    ).unwrap();

    // Approval metrics
    static ref APPROVAL_RESPONSE_TIME: Histogram = Histogram::new(
        "approval_response_time_seconds", "Time to approval decision"
    ).unwrap();
    static ref PENDING_APPROVALS: IntGauge = IntGauge::new(
        "pending_approvals_current", "Current pending approvals"
    ).unwrap();
}
```

---

### 5.3 Compliance Audit Log

**Location:** `server/crates/iou-orchestrator/src/audit.rs`

**Audit Entry:**

```rust
pub struct AuditEntry {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event: AuditEvent,
    pub actor: AuditActor,
    pub details: serde_json::Value,
}

pub enum AuditEvent {
    WorkflowCreated,
    WorkflowStarted,
    AgentStarted { agent: AgentType },
    AgentCompleted { agent: AgentType },
    AgentFailed { agent: AgentType, error: String },
    ApprovalRequested { agent: AgentType },
    ApprovalDecision { decision: ApprovalDecision },
    WorkflowEscalated { reason: String },
    WorkflowCompleted,
    WorkflowCancelled,
    CheckpointSaved,
    WorkflowRecovered,
}

pub enum AuditActor {
    System,
    User { id: Uuid, name: String },
    Agent { agent_type: AgentType },
}
```

**Audit Storage:**

```rust
#[async_trait]
pub trait AuditStorage: Send + Sync {
    async fn write_entry(&self, entry: &AuditEntry) -> Result<(), AuditError>;
    async fn query_workflow_audit(&self, workflow_id: Uuid) -> Result<Vec<AuditEntry>, AuditError>;
    async fn query_by_actor(&self, actor_id: Uuid, from: DateTime<Utc>) -> Result<Vec<AuditEntry>, AuditError>;
}
```

---

## Directory Structure

```
server/crates/iou-orchestrator/
├── src/
│   ├── state_machine/
│   │   ├── mod.rs              # State machine exports
│   │   ├── base.rs             # Core state machine (existing, extend)
│   │   └── events.rs           # Event definitions (extend)
│   ├── dag/
│   │   ├── mod.rs              # DAG module exports
│   │   ├── builder.rs          # DAG construction from dependencies
│   │   └── model.rs            # DAG data structures
│   ├── executor/
│   │   ├── mod.rs              # Executor module exports
│   │   ├── parallel.rs         # Parallel agent execution
│   │   └── retry.rs            # Retry logic with backoff
│   ├── event_bus/
│   │   ├── mod.rs              # Event bus exports
│   │   ├── bus.rs              # Event bus implementation
│   │   └── events.rs           # Event type definitions
│   ├── checkpoint/
│   │   ├── mod.rs              # Checkpoint module exports
│   │   ├── policy.rs           # Adaptive checkpoint policy
│   │   ├── model.rs            # Checkpoint data structures
│   │   ├── storage.rs          # DuckDB storage implementation
│   │   └── recovery.rs         # Recovery workflow
│   ├── graphql/
│   │   ├── mod.rs              # GraphQL module exports
│   │   ├── schema.rs           # GraphQL schema definition
│   │   ├── mutations.rs        # Mutation resolvers
│   │   ├── queries.rs          # Query resolvers
│   │   └── subscriptions.rs    # Subscription resolvers
│   ├── websocket/
│   │   ├── mod.rs              # WebSocket module exports
│   │   ├── handler.rs          # WebSocket connection handler
│   │   └── messages.rs         # Message type definitions
│   ├── notification/
│   │   ├── mod.rs              # Notification module exports
│   │   ├── channel.rs          # Notification channel trait
│   │   ├── dispatcher.rs       # Multi-channel dispatcher
│   │   ├── websocket.rs        # WebSocket notifications
│   │   ├── email.rs            # Email notifications
│   │   ├── dashboard.rs        # Dashboard notifications
│   │   └── mobile.rs           # Mobile push notifications
│   ├── scheduler/
│   │   ├── mod.rs              # Scheduler module exports
│   │   ├── queue.rs            # Priority queue
│   │   ├── scheduler.rs        # Main scheduler with preemption
│   │   └── config.rs           # Scheduler configuration
│   ├── metrics.rs              # Prometheus metrics
│   ├── audit.rs                # Compliance audit logging
│   ├── context.rs              # Workflow context (existing)
│   ├── error.rs                # Error types (existing, extend)
│   └── lib.rs                  # Library exports
├── Cargo.toml
└── tests/
    ├── state_machine_tests.rs
    ├── dag_tests.rs
    ├── executor_tests.rs
    ├── checkpoint_tests.rs
    └── loom_tests.rs           # Concurrency tests
```

---

## Integration Points

### Existing Code to Extend

| File | Purpose | Changes |
|------|---------|---------|
| `state_machine/base.rs` | Core state machine | Add DAG states, events, guards |
| `context.rs` | Workflow context | Add DAG tracking, checkpoint metadata |
| `stage_executor.rs` | Stage execution | Replace with parallel executor |
| `error.rs` | Error types | Add DAG, checkpoint, scheduler errors |

### Dependencies to Add

```toml
[dependencies]
# Existing
tokio = { version = "1.43", features = ["full"] }
duckdb = "1"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }

# New
async-graphql = { version = "7", features = ["tokio", "uuid"] }
async-graphql-axum = "7"
tokio-tungstenite = "0.24"
prometheus = "0.13"
lettre = "0.11"  # Email
fcm = "0.10"     # Mobile push (or use existing)
```

---

## Testing Strategy

### Unit Tests

- **State machine transitions:** All valid and invalid transitions
- **DAG builder:** Dependency validation, layer construction
- **Checkpoint policy:** Decision logic for various scenarios
- **Retry logic:** Backoff calculation, attempt counting

### Concurrency Tests (loom)

```rust
#[test]
fn test_concurrent_state_transitions() {
    loom::model(|| {
        // Test concurrent state access
    });
}

#[test]
fn test_concurrent_checkpoint_writes() {
    loom::model(|| {
        // Test checkpoint write contention
    });
}
```

### Integration Tests

- **End-to-end workflow:** Create → Execute → Approve → Complete
- **Checkpoint/recovery:** Save checkpoint, simulate crash, recover
- **WebSocket communication:** Client sends approval, receives updates
- **Preemption:** High-priority workflow preempts low-priority

### Approval UX Tests

- **Inline approval:** UI renders agent output inline
- **Modification tracking:** Changes tracked with diffs
- **Notification delivery:** All channels receive notifications
- **Escalation:** Timeout triggers escalation

---

## Success Criteria

### Phase 1 Completion

- [ ] DAG builder constructs valid execution layers
- [ ] Parallel executor runs independent agents concurrently
- [ ] Event bus delivers events to all subscribers
- [ ] State machine handles new DAG states correctly

### Phase 2 Completion

- [ ] Checkpoints saved at configured intervals
- [ ] Workflow recovers from crash to exact state
- [ ] DuckDB storage performs efficiently

### Phase 3 Completion

- [ ] GraphQL API serves queries/mutations/subscriptions
- [ ] WebSocket delivers real-time updates
- [ ] All notification channels functioning

### Phase 4 Completion

- [ ] Priority queue orders workflows correctly
- [ ] Preemption gracefully pauses/resumes workflows
- [ ] Multiple workflows execute concurrently

### Phase 5 Completion

- [ ] All metrics exposed to Prometheus
- [ ] Audit trail captures all compliance events
- [ ] Dashboard visualizes workflow status

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| DAG cycles prevent execution | High | Validate dependencies during workflow creation |
| Checkpoint corruption prevents recovery | High | Version checkpoints, validate on load |
| Parallel agents conflict on shared state | Medium | Design stateless agents, use immutable data |
| Preemption causes data loss | High | Always checkpoint before preempt |
| WebSocket connection drops | Medium | Auto-reconnect with state sync |
| Notification delivery failure | Low | Log failures, provide dashboard fallback |

---

*This plan synthesizes requirements from the feature README, findings from codebase and web research, and detailed interview responses. The implementation extends existing infrastructure rather than replacing it, following established patterns in the IOU-Modern codebase.*
