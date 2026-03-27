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

**In-memory state met persistente checkpoints**:
- Primaire state in geheugen voor 1-10 workflows
- Periodieke checkpoints naar DuckDB voor crash recovery
- Event sourcing pattern voor audit trail

### 1.2 Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Async Runtime | Tokio | Standaard Rust async runtime |
| State Machine | smlang | DSL met compile-time validatie |
| Channels | tokio::sync (mpsc, oneshot) | Inter-agent communicatie |
| Storage | DuckDB (existing) | Bestaande database voor checkpoints |
| Web Framework | Axum (existing) | API integration |
| Observability | tracing | Structured logging voor async |

### 1.3 High-Level Architecture

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

    // Timing
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

pub struct AgentResult {
    pub agent: AgentType,
    pub success: bool,
    pub data: serde_json::Value,
    pub execution_time_ms: u64,
    pub human_modified: bool,
    pub modifications: Vec<HumanModification>,
}
```

### 2.3 Guards

Guards bepalen of een transition mag plaatsvinden:

```rust
fn can_proceed(_state: &WorkflowState, context: &WorkflowContext) -> bool {
    // Check if human approved the last agent result
    true  // Placeholder - actual implementation checks pending_approvals
}

fn can_retry(_state: &WorkflowState, context: &WorkflowContext) -> bool {
    // Check if retry count is below max
    true  // Placeholder - actual implementation checks retry count
}
```

---

## 3. Agent Execution Engine

### 3.1 Agent Executor

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
}
```

### 3.2 Parallel Execution Strategy

Sommige agent combinaties kunnen parallel lopen:

- Research voor document N kan parallel met Content van document N-1
- Compliance pre-check kan parallel met Content generatie
- Echter: final Compliance moet wachten op Content completion

```rust
pub fn get_parallel_groups(agent_sequence: &[AgentType]) -> Vec<Vec<AgentType>> {
    // Returns groups of agents that can run in parallel
    // Example: [[Research], [Content, PreCompliance], [FinalCompliance, Review]]
}
```

---

## 4. Human-in-the-Loop System

### 4.1 Approval Workflow

Na elke agent execution gaat de workflow naar `AwaitingApproval` state:

```rust
pub struct ApprovalRequest {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub agent: AgentType,
    pub result: AgentResult,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub enum ApprovalDecision {
    Approve,
    Modify { modifications: Vec<HumanModification> },
    Reject { reason: String },
    RequestChanges { feedback: String },
}
```

### 4.2 Event Stream voor UI Updates

De UI ontvangt real-time updates via Server-Sent Events:

```rust
// GET /api/workflows/:id/stream
pub async fn stream_workflow_events(
    Path(id): Path<Uuid>,
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
}
```

### 4.3 API Endpoints voor Menselijke Interactie

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/workflows/:id/pending` | GET | Haal openstaande goedkeuringen op |
| `/api/workflows/:id/approve` | POST | Keur agent resultaat goed |
| `/api/workflows/:id/modify` | POST | Wijzig agent resultaat |
| `/api/workflows/:id/reject` | POST | Weiger agent resultaat |

---

## 5. Checkpoint and Recovery

### 5.1 Checkpoint Data

Checkpoints worden opgeslagen in DuckDB:

```rust
pub struct WorkflowCheckpoint {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub state: WorkflowState,
    pub context_json: String,  // Serialized WorkflowContext
    pub checkpointed_at: DateTime<Utc>,
    pub sequence_number: u64,
}

// DuckDB table schema
// CREATE TABLE workflow_checkpoints (
//     id UUID PRIMARY KEY,
//     workflow_id UUID NOT NULL,
//     state VARCHAR NOT NULL,
//     context_json TEXT NOT NULL,
//     checkpointed_at TIMESTAMP NOT NULL,
//     sequence_number INTEGER NOT NULL
// );
```

### 5.2 Checkpoint Strategy

Checkpoints worden gemaakt op:
- Na elke agent completion
- Voor menselijke goedkeuring
- Na menselijke modificatie
- Op configurable interval (bijv. elke 30 seconden)

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

    /// Resume a workflow from checkpoint
    pub async fn resume_workflow(
        &self,
        workflow_id: Uuid,
    ) -> Result<WorkflowContext, CheckpointError>;
}
```

---

## 6. Error Handling and Retry

### 6.1 Error Classification

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
}

impl AgentError {
    /// Determine if this error is retryable
    pub fn is_retryable(&self) -> bool;

    /// Get suggested backoff duration
    pub fn backoff_duration(&self, attempt: u32) -> Duration;
}
```

### 6.2 Retry Logic

Exponential backoff voor transient errors:

```rust
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub backoff_multiplier: f64,
}

impl RetryPolicy {
    /// Calculate backoff duration for given attempt
    pub fn backoff(&self, attempt: u32) -> Duration {
        let ms = (self.base_backoff_ms as f64
            * self.backoff_multiplier.powi(attempt as i32))
            .min(self.max_backoff_ms as f64) as u64;
        Duration::from_millis(ms)
    }
}
```

---

## 7. Directory Structure

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

## 8. Configuration

### 8.1 Workflow Configuration

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

    // Parallel execution
    pub enable_parallel: bool,

    // Human interaction
    pub approval_timeout_hours: u32,
}
```

### 8.2 Document Type Configuration

```rust
pub struct DocumentTypeConfig {
    pub document_type: String,
    pub timeout_ms: u64,
    pub require_human_review: bool,
    pub parallel_agents: Vec<AgentType>,
    pub approval_chain: Vec<ApprovalLevel>,
}
```

---

## 9. Testing Strategy

### 9.1 Unit Tests

- State machine transitions (smlang compile-time validatie)
- Guard conditions
- Agent executor met mock agents
- Checkpoint save/load
- Event serialization

### 9.2 Integration Tests

- Volledige workflow van start tot completion
- Menselijke goedkeurings flow
- Parallel agent execution
- Crash recovery van checkpoint
- Timeout handling

### 9.3 Test Utilities

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
}
```

---

## 10. Implementation Phases

### Phase 1: Foundation
1. Maak `iou-orchestrator` crate
2. Definieer state machine met smlang
3. Implementeer basis WorkflowContext types
4. Unit tests voor state transitions

### Phase 2: Agent Execution
1. Implementeer AgentExecutor
2. Voeg timeout handling met Tokio
3. Implementeer retry logic
4. Parallel execution support

### Phase 3: Human-in-the-Loop
1. Approval types en workflow
2. API endpoints voor goedkeuring
3. SSE event stream
4. UI integration points

### Phase 4: Checkpoint en Recovery
1. DuckDB checkpoint schema
2. CheckpointManager implementatie
3. Resume van checkpoint logic
4. Recovery tests

### Phase 5: Integration
1. API koppeling met bestaande routes
2. Frontend integration (Dioxus)
3. End-to-end testing
4. Performance tuning

---

## 11. Open Questions Resolved

| Vraag | Beslissing | Rationale |
|-------|-----------|-----------|
| LangGraph vs Rust? | Rust-native | Eenvoudiger integratie, minder dependencies |
| Checkpoint UI? | Per agent | Meer controle, betere audit trail |
| Schaal? | 1-10 workflows | In-memory state is voldoende |
| Parallel agents? | Ja waar mogelijk | Snellere throughput |
| Opslag? | In-memory + DuckDB checkpoints | Balans tussen performance en recovery |

---

## 12. Success Criteria

Na implementatie:
1. ✅ Document creatie is volledig georkestreerd door agents
2. ✅ Menselijke goedkeuring is verplicht na elke agent
3. ✅ Alle stappen zijn traceerbaar via audit trail
4. ✅ Workflow kan resume na crash
5. ✅ Agents kunnen parallel lopen waar mogelijk
6. ✅ Timeout is configureerbaar per document type
7. ✅ Implementatie past binnen bestaande Rust architecture
