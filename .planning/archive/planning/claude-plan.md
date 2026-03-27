# Implementation Plan: Agent-Orchestrated Document Creation with Human-in-the-Loop

## Overview

Dit plan beschrijft de implementatie van een georkestreerde document creatie flow voor IOU-Modern, waarbij AI agents werken onder menselijk toezicht met goedkeuringspunten na elke agent. De implementatie is volledig Rust-native en integreert met de bestaande codebase.

**Context:** IOU-Modern is een informatiebeheer platform voor Nederlandse overheidsorganisaties, gebouwd met Rust, Axum, en DuckDB. De huidige agent implementatie in `crates/iou-ai/` heeft vier agents (Research, Content, Compliance, Review) maar mist menselijke tussenkomst en workflow state management.

---

## 1. Architecture

### 1.1 Design Philosophy

**Rust-native orchestration** over LangGraph integratie:
- Eén codebase, één taal
- Geen Python service bridge nodig
- Betere integratie met bestaande IOU-Modern architecture

**State machine driven workflow**:
- Gebruik smlang DSL voor state machine definitie
- Compile-time validatie van transitions
- Type-safe state handling
- Fallback plan: Indien smlang problemen geeft, overschakelen naar hand-rolled state machine

**In-memory state met persistente checkpoints**:
- Primaire state in geheugen voor 1-10 workflows
- Periodieke checkpoints naar DuckDB voor crash recovery
- Event sourcing pattern voor audit trail

### 1.2 Relationship to Existing Workflow Systems

IOU-Modern heeft drie workflow-gerelateerde systemen die complementair zijn:

| Systeem | Locatie | Verantwoordelijkheid |
|---------|---------|---------------------|
| `WorkflowEngine` | `iou-api/src/workflows/mod.rs` | Domein workflow definities, goedkeuringsstromen |
| `AgentPipeline` | `iou-ai/src/agents/pipeline.rs` | Sequentiële agent executie (huidig) |
| `WorkflowOrchestrator` | `iou-orchestrator` (nieuw) | State machine met human-in-the-loop |

De nieuwe `WorkflowOrchestrator`:
- Gebruikt bestaande agents uit `iou-ai`
- Integreert met `WorkflowEngine` voor domein-specifieke goedkeuringen
- Voegt state machine en checkpoint functionaliteit toe

### 1.3 Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Async Runtime | Tokio | Standaard Rust async runtime |
| State Machine | smlang (+ fallback) | DSL met compile-time validatie |
| Channels | tokio::sync (mpsc, oneshot) | Inter-agent communicatie |
| Storage | DuckDB (existing) | Bestaande database voor checkpoints |
| Web Framework | Axum (existing) | API integration |
| Observability | tracing | Structured logging voor async |
| Concurrency Testing | loom | Deterministische concurrency tests |

### 1.4 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         API Layer                            │
│  POST /api/workflows   GET /api/workflows/:id/stream        │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                    Workflow Orchestrator                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ State Machine│  │ Event Bus    │  │ Checkpoint   │     │
│  │  (smlang)     │  │  (mpsc)      │  │  Manager     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└────────────────────────┬────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Research   │  │   Content   │  │ Compliance  │
│   Agent     │  │   Agent     │  │   Agent     │
└─────────────┘  └─────────────┘  └─────────────┘
        │                │                │
        └────────────────┼────────────────┘
                         ▼
                  ┌─────────────┐
                  │    Review   │
                  │   Agent     │
                  └─────────────┘
```

---

## 2. State Machine Design

### 2.1 States

De state machine wordt gedefinieerd met smlang DSL:

```rust
state_machine! {
    name: Workflow,
    derive_states: [Debug, Clone, PartialEq],
    derive_events: [Clone, Debug],
    transitions: {
        // Initial state
        *Created + Start = Running,

        // Agent execution states
        Running + AgentComplete(AgentType) [can_proceed] = Running,
        Running + RequireApproval = AwaitingApproval,

        // Human interaction
        AwaitingApproval + Approved = Running,
        AwaitingApproval + Modified = Running,
        AwaitingApproval + Rejected = Failed,
        AwaitingApproval + TimeoutEscalated = AwaitingEscalation,

        // Escalation handling
        AwaitingEscalation + Approved = Running,
        AwaitingEscalation + Rejected = Failed,

        // Completion
        Running + AllAgentsComplete = Completed,
        *Completed + Finalize = Archived,

        // Error handling
        Running + AgentFailed [can_retry] = Retrying,
        Retrying + RetryComplete = Running,
        Retrying + MaxRetriesExceeded = Failed,

        // Cancellation
        *Created + Cancel = Cancelled,
        Running + Cancel = Cancelled,
        AwaitingApproval + Cancel = Cancelled,
    }
}
```

### 2.2 State Context

De state machine draagt context die alle relevante data bevat:

```rust
pub struct WorkflowContext {
    // Workflow identificatie
    pub id: Uuid,
    pub document_request: DocumentRequest,
    pub workflow_version: String,  // Voor versie compatibiliteit

    // Huidige state
    pub current_agent: Option<AgentType>,
    pub completed_agents: HashSet<AgentType>,
    pub failed_agents: HashMap<AgentType, u32>,  // agent -> retry count

    // Resultaten per agent
    pub agent_results: HashMap<AgentType, AgentResult>,

    // Menselijke interacties
    pub pending_approvals: Vec<ApprovalRequest>,
    pub audit_log: Vec<AuditEntry>,

    // Configuratie
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub approval_timeout_hours: u32,

    // Timing
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub approval_deadline: Option<DateTime<Utc>>,
}

pub struct AgentResult {
    pub agent: AgentType,
    pub success: bool,
    pub data: serde_json::Value,
    pub execution_time_ms: u64,
    pub human_modified: bool,
    pub modifications: Vec<HumanModification>,
    pub completed_at: DateTime<Utc>,
}
```

### 2.3 Guards

Guards bepalen of een transition mag plaatsvinden:

```rust
fn can_proceed(_state: &WorkflowState, context: &WorkflowContext) -> bool {
    // Check if human approved the last agent result
    context.pending_approvals.is_empty()
}

fn can_retry(_state: &WorkflowState, context: &WorkflowContext) -> bool {
    // Check if any agent hasn't exceeded max retries
    context.failed_agents.values().all(|&count| count < context.max_retries)
}
```

---

## 3. State Machine Recovery

### 3.1 Recovery Algorithm

Wanneer een workflow geresumed wordt van een checkpoint:

1. **Laatste checkpoint laden** uit DuckDB
2. **State deserialiseren** van string representation
3. **Context valideren** tegen huidige workflow definitie
4. **State machine reconstrueren** met behulp van smlang's `StateMachine::new()`
5. **Doorgaan met volgende agent** of wachten op menselijke goedkeuring

### 3.2 State Deserialization

```rust
impl WorkflowState {
    /// Deserialize state from string (stored in checkpoint)
    pub fn from_str(s: &str) -> Result<Self, StateError> {
        match s {
            "Created" => Ok(WorkflowState::Created),
            "Running" => Ok(WorkflowState::Running),
            "AwaitingApproval" => Ok(WorkflowState::AwaitingApproval),
            // ... all states
            _ => Err(StateError::InvalidState(s.to_string())),
        }
    }

    /// Serialize state to string (for checkpoint storage)
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
```

### 3.3 Version Compatibility

```rust
pub struct WorkflowVersion {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub definition_hash: String,  // Hash van state machine definitie
}

impl CheckpointManager {
    /// Validate that checkpoint is compatible with current workflow
    pub fn validate_compatibility(
        &self,
        checkpoint: &WorkflowCheckpoint,
        current_version: &WorkflowVersion,
    ) -> Result<(), CompatibilityError> {
        // Check if versions match or if migration is possible
        if checkpoint.workflow_version == current_version.version {
            return Ok(());
        }
        // TODO: Implement migration logic for version changes
        Err(CompatibilityError::VersionMismatch)
    }
}
```

### 3.4 Checkpoint Validation

Bij het laden van een checkpoint:

```rust
pub fn validate_checkpoint(checkpoint: &WorkflowCheckpoint) -> Result<(), CheckpointError> {
    // 1. Check that JSON is valid
    let _context: WorkflowContext = serde_json::from_str(&checkpoint.context_json)
        .map_err(|e| CheckpointError::InvalidJson(e))?;

    // 2. Check that state is valid for current workflow definition
    let _state = WorkflowState::from_str(&checkpoint.state)
        .map_err(|e| CheckpointError::InvalidState(e))?;

    // 3. Check sequence number is monotonically increasing
    // 4. Check timestamps are reasonable
    Ok(())
}
```

---

## 4. Agent Execution Engine

### 4.1 Agent Executor

De agent executor voert individuele agents uit met timeout en retry:

```rust
pub struct AgentExecutor {
    research_agent: Arc<ResearchAgent>,
    content_agent: Arc<ContentAgent>,
    compliance_agent: Arc<ComplianceAgent>,
    review_agent: Arc<ReviewAgent>,
    config: ExecutorConfig,
}

impl AgentExecutor {
    /// Execute an agent with timeout and retry handling
    pub async fn execute_agent(
        &self,
        agent: AgentType,
        context: &WorkflowContext,
    ) -> Result<AgentResult, AgentError>;

    /// Check if agents can run in parallel
    pub fn can_run_parallel(&self, agents: &[AgentType]) -> bool;

    /// Execute multiple agents in parallel
    pub async fn execute_parallel(
        &self,
        agents: Vec<AgentType>,
        context: &WorkflowContext,
    ) -> Vec<(AgentType, Result<AgentResult, AgentError>)>;

    /// Cancel a running agent
    pub async fn cancel_agent(&self, workflow_id: Uuid) -> Result<(), AgentError>;
}
```

### 4.2 Parallel Execution Strategy

Sommige agent combinaties kunnen parallel lopen:

- Research voor document N kan parallel met Content van document N-1
- Compliance pre-check kan parallel met Content generatie
- Echter: final Compliance moet wachten op Content completion

```rust
pub fn get_parallel_groups(agent_sequence: &[AgentType]) -> Vec<Vec<AgentType>> {
    // Returns groups of agents that can run in parallel
    // Example: [[Research], [Content, PreCompliance], [FinalCompliance, Review]]
}

/// Parallel execution failure handling:
/// - If any agent in a parallel group fails with Permanent error: entire group fails
/// - If agent fails with Transient error: retry only that agent
/// - If all agents retryable: retry entire group
pub enum ParallelFailureMode {
    FailGroup,
    RetryFailed,
    RetryAll,
}
```

---

## 5. Human-in-the-Loop System

### 5.1 Approval Workflow

Na elke agent execution gaat de workflow naar `AwaitingApproval` state:

```rust
pub struct ApprovalRequest {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub agent: AgentType,
    pub result: AgentResult,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,  // Based on approval_timeout_hours
    pub escalated: bool,
}

pub enum ApprovalDecision {
    Approve,
    Modify { modifications: Vec<HumanModification> },
    Reject { reason: String },
    RequestChanges { feedback: String },
}

pub enum ApprovalTimeoutBehavior {
    EscalateToSupervisor,   // Default: escalate to next approver
    AutoReject,             // Alternative: auto-reject after extended timeout
    ExtendDeadline,         // Alternative: extend deadline (manual)
}
```

### 5.2 Approval Timeout Handling

Wanneer een goedkeuring timeout (standaard 24 uur):

1. **Stuur notificatie** naar supervisor/escalatie contact
2. **Workflow gaat** naar `AwaitingEscalation` state
3. **Supervisor kan** goedkeuren in plaats van oorspronkelijke approver
4. **Na extended timeout** (standaard 72 uur totaal): auto-reject optie

```rust
pub struct TimeoutConfig {
    pub standard_timeout_hours: u32,    // Default: 24
    pub extended_timeout_hours: u32,     // Default: 72
    pub escalation_contacts: Vec<String>, // Email/notify targets
    pub auto_reject_after_extended: bool,
}
```

### 5.3 Event Stream voor UI Updates

De UI ontvangt real-time updates via Server-Sent Events met reconnection support:

```rust
// GET /api/workflows/:id/stream
pub async fn stream_workflow_events(
    Path(id): Path<Uuid>,
    LastEventId(last_id): Option<Header<String>>,
    State(app): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>>;

// Event types
pub enum WorkflowEvent {
    AgentStarted { agent: AgentType, started_at: DateTime<Utc> },
    AgentCompleted { agent: AgentType, result: AgentResult },
    ApprovalRequired { approval: ApprovalRequest },
    StateTransition { from: WorkflowState, to: WorkflowState },
    WorkflowCompleted { result: DocumentResult },
    ErrorOccurred { error: String },
    TimeoutOccurred { workflow_id: Uuid },
}

// SSE event format
pub struct SseEvent {
    pub id: String,        // UUID for event
    pub event_type: String, // "AgentStarted", etc.
    pub data: String,       // JSON payload
    pub retry: u32,         // Retry interval in ms
}
```

### 5.4 SSE Reconnection Strategy

**Server-side:**
- Event buffer met configurable grootte (standaard 100 events)
- Events bewaard voor replay bij reconnection
- Last-Event-ID header ondersteuning

**Client-side (Dioxus):**
- Automatische reconnection met exponential backoff
- Last-Event-ID meesturen bij reconnect
- At-least-once delivery semantics

```rust
pub struct EventBuffer {
    events: VecDeque<SseEvent>,
    max_size: usize,
    workflow_id: Uuid,
}

impl EventBuffer {
    /// Get events since given ID (for replay)
    pub fn events_since(&self, last_id: &str) -> Vec<SseEvent>;

    /// Prune old events beyond max_size
    pub fn prune(&mut self);

    /// Add new event
    pub fn push(&mut self, event: SseEvent);
}
```

### 5.5 API Endpoints voor Menselijke Interactie

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/workflows/:id/pending` | GET | Haal openstaande goedkeuringen op |
| `/api/workflows/:id/approve` | POST | Keur agent resultaat goed |
| `/api/workflows/:id/modify` | POST | Wijzig agent resultaat |
| `/api/workflows/:id/reject` | POST | Weiger agent resultaat |
| `/api/workflows/:id/escalate` | POST | Escaleren timeout goedkeuring |

---

## 6. Checkpoint and Recovery

### 6.1 Checkpoint Data

Checkpoints worden opgeslagen in DuckDB:

```rust
pub struct WorkflowCheckpoint {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub state: String,              // Serialized WorkflowState
    pub context_json: String,       // Serialized WorkflowContext
    pub workflow_version: String,   // Workflow definition version
    pub checkpointed_at: DateTime<Utc>,
    pub sequence_number: u64,
}

// DuckDB table schema
// CREATE TABLE workflow_checkpoints (
//     id UUID PRIMARY KEY,
//     workflow_id UUID NOT NULL,
//     state VARCHAR NOT NULL,
//     context_json TEXT NOT NULL,
//     workflow_version VARCHAR NOT NULL,
//     checkpointed_at TIMESTAMP NOT NULL,
//     sequence_number INTEGER NOT NULL,
//     INDEX (workflow_id, sequence_number)
// );
```

### 6.2 Checkpoint Strategy

Checkpoints worden gemaakt op:
- Na elke agent completion
- Voor menselijke goedkeuring (critical checkpoint)
- Na menselijke modificatie
- Op configurable interval (standaard 30 seconden tijdens lange operaties)

```rust
pub struct CheckpointManager {
    db: Arc<DuckDBConnection>,
    config: CheckpointConfig,
}

impl CheckpointManager {
    /// Save a checkpoint to DuckDB
    pub async fn save_checkpoint(
        &self,
        workflow_id: Uuid,
        state: &WorkflowState,
        context: &WorkflowContext,
    ) -> Result<Checkpoint, CheckpointError>;

    /// Load the latest checkpoint for a workflow
    pub async fn load_checkpoint(
        &self,
        workflow_id: Uuid,
    ) -> Result<Option<WorkflowCheckpoint>, CheckpointError>;

    /// Resume a workflow from checkpoint with validation
    pub async fn resume_workflow(
        &self,
        workflow_id: Uuid,
    ) -> Result<WorkflowContext, CheckpointError>;

    /// Validate checkpoint integrity
    pub fn validate_checkpoint(&self, checkpoint: &WorkflowCheckpoint) -> Result<(), CheckpointError>;
}
```

---

## 7. Error Handling and Retry

### 7.1 Error Classification

Bestaande `ErrorSeverity` wordt uitgebreid:

```rust
pub enum AgentError {
    Transient {
        source: Box<dyn Error + Send + Sync>,
        retry_after: Option<Duration>,
    },
    Permanent {
        source: Box<dyn Error + Send + Sync>,
        user_message: String,
    },
    Timeout {
        agent: AgentType,
        duration_ms: u64,
    },
    Cancelled {
        workflow_id: Uuid,
        reason: String,
    },
}

impl AgentError {
    /// Determine if this error is retryable
    pub fn is_retryable(&self) -> bool;

    /// Get suggested backoff duration
    pub fn backoff_duration(&self, attempt: u32) -> Duration;
}
```

### 7.2 Retry Logic

Exponential backoff voor transient errors:

```rust
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter: bool,  // Add random jitter to avoid thundering herd
}

impl RetryPolicy {
    /// Calculate backoff duration for given attempt
    pub fn backoff(&self, attempt: u32) -> Duration {
        let ms = (self.base_backoff_ms as f64
            * self.backoff_multiplier.powi(attempt as i32))
            .min(self.max_backoff_ms as f64) as u64;

        // Add jitter if enabled
        if self.jitter {
            let jitter_ms = (ms as f64 * 0.1) as u64;  // ±10%
            let ms = ms - jitter_ms + rand::random::<u64>() % (2 * jitter_ms);
            Duration::from_millis(ms)
        } else {
            Duration::from_millis(ms)
        }
    }
}
```

---

## 8. Performance Requirements

### 8.1 Latency Targets

| Operatie | Doel | Maximum |
|----------|------|----------|
| State transition | < 50ms | 100ms |
| Checkpoint save | < 200ms | 500ms |
| Checkpoint load | < 100ms | 300ms |
| Agent execution | Configureerbaar | 5 min (default) |
| SSE event delivery | < 20ms | 50ms |
| API response (non-SSE) | < 100ms | 200ms |

### 8.2 Memory Budget

| Component | Budget |
|-----------|--------|
| Per workflow state | < 10MB |
| Event buffer per workflow | < 5MB (100 events) |
| Channel backlog | < 1MB |
| Totaal voor 10 workflows | < 150MB |

### 8.3 Throughput Targets

- 10 gelijktijdige workflows zonder performance degradation
- 100 state transitions per seconde
- 50 checkpoints per seconde

---

## 9. Directory Structure

```
crates/
├── iou-orchestrator/          # Nieuwe crate voor orchestration
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── state_machine.rs   # smlang definitions
│       ├── executor.rs        # Agent execution logic
│       ├── checkpoint.rs      # Checkpoint management
│       ├── events.rs          # Event types and bus
│       ├── approval.rs        # Human-in-the-loop logic
│       ├── timeout.rs         # Timeout handling (NEW)
│       ├── version.rs         # Workflow versioning (NEW)
│       └── config.rs          # Configuration types
│
├── iou-ai/                    # Bestaande agents (uitbreiden)
│   └── src/
│       └── agents/
│           ├── mod.rs
│           ├── research.rs
│           ├── content.rs
│           ├── compliance.rs
│           └── review.rs
│
└── iou-api/                   # Bestaande API (uitbreiden)
    └── src/
        └── routes/
            └── workflows.rs   # Nieuwe workflow endpoints
```

---

## 10. Configuration

### 10.1 Workflow Configuration

```rust
pub struct OrchestratorConfig {
    // Agent timeouts (configureerbaar per document type)
    pub default_timeout_ms: u64,
    pub agent_timeouts: HashMap<DocumentType, u64>,

    // Retry policy
    pub max_retries: u32,
    pub retry_policy: RetryPolicy,

    // Checkpoint configuration
    pub checkpoint_interval_ms: u64,
    pub checkpoint_enabled: bool,
    pub checkpoint_retention_days: u32,

    // Parallel execution
    pub enable_parallel: bool,
    pub max_parallel_agents: usize,

    // Human interaction
    pub approval_timeout_hours: u32,
    pub extended_timeout_hours: u32,
    pub escalation_contacts: Vec<String>,

    // Event buffer
    pub event_buffer_size: usize,

    // Workflow versioning
    pub current_workflow_version: String,
}
```

### 10.2 Document Type Configuration

```rust
pub struct DocumentTypeConfig {
    pub document_type: String,
    pub timeout_ms: u64,
    pub require_human_review: bool,
    pub parallel_agents: Vec<AgentType>,
    pub approval_chain: Vec<ApprovalLevel>,
    pub approval_timeout_override: Option<u32>,
}
```

---

## 11. Testing Strategy

### 11.1 Unit Tests

- State machine transitions (smlang compile-time validatie)
- Guard conditions
- Agent executor met mock agents
- Checkpoint save/load
- Event serialization
- Version compatibility checks

### 11.2 Integration Tests

- Volledige workflow van start tot completion
- Menselijke goedkeurings flow
- Parallel agent execution
- Crash recovery van checkpoint
- Timeout handling
- SSE reconnection met event replay

### 11.3 Concurrency Tests (NIEUW met loom)

```rust
#[cfg(test)]
mod concurrency_tests {
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn test_concurrent_approval_requests() {
        // Test dat twee gelijktijdige goedkeuringen correct worden afgehandeld
    }

    #[test]
    fn test_checkpoint_race_condition() {
        // Test race condition bij gelijktijdige checkpoint writes
    }

    #[test]
    fn test_approval_during_agent_failure() {
        // test race condition tussen approval en agent failure
    }
}
```

### 11.4 Recovery Tests

```rust
#[cfg(test)]
mod recovery_tests {
    #[test]
    fn test_resume_from_each_state() {
        // Simuleer crash in elke state en test recovery
    }

    #[test]
    fn test_corrupted_checkpoint_handling() {
        // Test gedrag met corrupte checkpoint data
    }

    #[test]
    fn test_version_migration() {
        // Test migratie van oude naar nieuwe workflow versie
    }
}
```

### 11.5 Test Utilities

```rust
#[cfg(test)]
pub mod test_utils {
    /// Mock agent die altijd succeedt
    pub struct MockSucceedingAgent;

    /// Mock agent die altijd faalt (retryable)
    pub struct MockFailingAgent {
        pub succeed_after: u32,
    };

    /// In-memory checkpoint store voor tests
    pub struct InMemoryCheckpointStore;

    /// Test clock voor deterministische time tests
    pub struct TestClock;
}
```

---

## 12. Implementation Phases

### Phase 1: Foundation
1. Maak `iou-orchestrator` crate
2. Definieer state machine met smlang (evalueren, fallback naar hand-rolled)
3. Implementeer basis WorkflowContext types
4. Voeg workflow versioning toe
5. Unit tests voor state transitions

### Phase 2: Agent Execution
1. Implementeer AgentExecutor
2. Voeg timeout handling met Tokio toe
3. Implementeer retry logic met jitter
4. Parallel execution support met failure mode handling
5. Concurrency tests met loom

### Phase 3: Human-in-the-Loop
1. Approval types en workflow
2. API endpoints voor goedkeuring
3. Timeout handling en escalatie
4. SSE event stream met reconnection support
5. UI integration points

### Phase 4: Checkpoint en Recovery
1. DuckDB checkpoint schema
2. CheckpointManager implementatie
3. State machine recovery algorithm
4. Version compatibility validatie
5. Recovery tests

### Phase 5: Integration
1. API koppeling met bestaande routes
2. Relatie met WorkflowEngine verduidelijken
3. Frontend integration (Dioxus)
4. End-to-end testing
5. Performance tuning

---

## 13. Open Questions Resolved

| Vraag | Beslissing | Rationale |
|-------|-----------|-----------|
| LangGraph vs Rust? | Rust-native | Eenvoudiger integratie, minder dependencies |
| Checkpoint UI? | Per agent | Meer controle, betere audit trail |
| Schaal? | 1-10 workflows | In-memory state is voldoende |
| Parallel agents? | Ja waar mogelijk | Snellere throughput |
| Opslag? | In-memory + DuckDB checkpoints | Balans tussen performance en recovery |
| Approval timeout? | Escalatie naar supervisor (24h) | Verantwoordelijke handhaving |
| State machine recovery? | Deserialize + reconstruct + validate | Robuuste recovery procedure |
| Workflow versioning? | Version string in context + compatibility check | Ondersteuning voor in-flight migrations |

---

## 14. Success Criteria

Na implementatie:
1. ✅ Document creatie is volledig georkestreerd door agents
2. ✅ Menselijke goedkeuring is verplicht na elke agent
3. ✅ Alle stappen zijn traceerbaar via audit trail
4. ✅ Workflow kan resume na crash vanuit elke state
5. ✅ Agents kunnen parallel lopen waar mogelijk
6. ✅ Timeout is configureerbaar per document type
7. ✅ Implementatie past binnen bestaande Rust architecture
8. ✅ SSE reconnection werkt met event replay
9. ✅ Approval timeout resulteert in escalatie, niet dataverlies
