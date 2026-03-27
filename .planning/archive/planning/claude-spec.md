# Agent-Orchestrated Document Creation - Complete Specification

## Version: 1.0
## Date: 2026-03-10

---

## 1. Executive Summary

Volledige herontwerp van de document creatie flow in IOU-Modern, georkestreerd door AI agents met menselijke tussenkomst na elke agent. De implementatie is volledig Rust-native met behulp van Tokio voor async runtime en smlang voor state machine definitie.

---

## 2. Current State Analysis

### 2.1 Existing Architecture

De huidige implementatie in `crates/iou-ai/src/agents/` bevat vier agents:

| Agent | Verantwoordelijkheid | Status |
|-------|---------------------|--------|
| Research | GraphRAG queries, structuur extractie | ✅ Geïmplementeerd |
| Content | Markdown template verwerking, AI generatie | ✅ Geïmplementeerd |
| Compliance | Woo/GDPR validatie, PII detectie | ✅ Geïmplementeerd |
| Review | Kwaliteitscontrole, goedkeuring logic | ✅ Geïmplementeerd |

### 2.2 Current Pipeline

`crates/iou-ai/src/agents/pipeline.rs` implementeert een sequentiële pipeline:
```
User Request → Research → Content → Compliance → Review → Document
```

### 2.3 What Works Well

- ✅ Solide error handling met `ErrorSeverity` (Transient/Permanent)
- ✅ Modulaire agent design met schone interfaces
- ✅ Configuratie flexibiliteit (`PipelineConfig`)
- ✅ Goede test coverage voor core logic
- ✅ Checkpoint structuur (maar opslag is TODO)

### 2.4 Critical Gaps

- ❌ Geen menselijke tussenkomst tussen agents
- ❀ Synchrone pipeline flow (geen pause/resume)
- ❀ Geen persistenie van workflow state
- ❀ Geen API integratie
- ❀ Beperkte event handling voor UI

---

## 3. Requirements

### 3.1 Functional Requirements

#### FR1: Agent Orchestration
- Document creatie moet uitvoerbaar zijn via state machine
- Ondersteuning voor parallelle agent executie waar mogelijk
- Agents moeten autonoom beslissingen kunnen nemen
- State machine moet gedefinieerd worden met smlang DSL

#### FR2: Human-in-the-Loop Checkpoints
- Goedkeuringspunt NA elke agent
- Mogelijkheid voor menselijke correctie tijdens generatie
- Audit trail van alle menselijke interventies
- UI moet per-agent resultaten tonen

#### FR3: State Management
- In-memory state voor 1-10 workflows
- Persistente checkpoints in DuckDB voor recovery
- Version tracking van document iteraties
- Resume capability na crash

#### FR4: Error Handling
- Auto-retry met exponential backoff voor transient errors
- Permanent failure na max retries
- Configureerbare timeouts per document type
- Optionele alert naar mens bij permanent failure

#### FR5: Integration
- API endpoints voor document aanvraag, status, goedkeuring
- Event-driven architecture voor UI integration
- Browser-native event sourcing (cqrs-es pattern)

### 3.2 Non-Functional Requirements

#### NFR1: Scale
- 1-10 gelijktijdige document workflows
- Geen distributed locking nodig
- Single-worker model is voldoende

#### NFR2: Performance
- Agent timeouts configureerbaar (standaard 5 min)
- Parallel executie waar mogelijk
- UI progress updates voor langlopende taken

#### NFR3: Technology Stack
- **Async Runtime**: Tokio
- **State Machine**: smlang DSL
- **Event Pattern**: cqrs-es style event sourcing
- **Storage**: DuckDB (existing) + in-memory state
- **Channels**: tokio::sync (mpsc, oneshot)

---

## 4. Architecture Design

### 4.1 State Machine Definition

```
┌─────────────┐
│   CREATED   │
└──────┬──────┘
       │
       ▼
┌─────────────┐    human_approval     ┌──────────────┐
│  RESEARCHING │◄──────────────────────│ AWAITING_INPUT│
└──────┬──────┘                      └───────┬──────┘
       │                                     │
       │ completed                           │ input_received
       ▼                                     ▼
┌─────────────┐    human_approval     ┌──────────────┐
│  GENERATING │◄──────────────────────│ AWAITING_INPUT│
└──────┬──────┘                      └───────┬──────┘
       │                                     │
       │ completed                           │ input_received
       ▼                                     ▼
┌─────────────┐    human_approval     ┌──────────────┐
│ COMPLIANCE   │◄──────────────────────│ AWAITING_INPUT│
└──────┬──────┘                      └───────┬──────┘
       │                                     │
       │ completed                           │ input_received
       ▼                                     ▼
┌─────────────┐    human_approval     ┌──────────────┐
│ REVIEWING   │◄──────────────────────│ AWAITING_INPUT│
└──────┬──────┘                      └───────┬──────┘
       │                                     │
       │ approved                           │
       ▼                                     │
┌─────────────┐◄─────────────────────────────┘
│ COMPLETED   │
└─────────────┘
```

### 4.2 Parallel Execution Paths

Sommine agent combinaties kunnen parallel lopen:
- `Compliance` kan starten tijdens `Content` (voorlopige check)
- `Research` voor volgende document kan starten tijdens `Content` van huidige

### 4.3 Event Types

```rust
pub enum WorkflowEvent {
    // Workflow lifecycle
    WorkflowCreated { id: Uuid, document_type: String },
    WorkflowStarted { id: Uuid },
    WorkflowCompleted { id: Uuid },
    
    // Agent execution
    AgentStarted { id: Uuid, agent: AgentType },
    AgentCompleted { id: Uuid, agent: AgentType, result: AgentResult },
    AgentFailed { id: Uuid, agent: AgentType, error: AgentError },
    
    // Human interaction
    ApprovalRequested { id: Uuid, agent: AgentType },
    ApprovalReceived { id: Uuid, agent: AgentType, decision: ApprovalDecision },
    ContentModified { id: Uuid, agent: AgentType, modifications: Vec<Modification> },
    
    // System events
    RetryScheduled { id: Uuid, agent: AgentType, attempt: u32 },
    TimeoutOccurred { id: Uuid, agent: AgentType },
}
```

### 4.4 Data Structures

```rust
pub struct WorkflowState {
    pub id: Uuid,
    pub document_request: DocumentRequest,
    pub current_agent: Option<AgentType>,
    pub status: WorkflowStatus,
    pub agent_results: HashMap<AgentType, AgentResult>,
    pub pending_approvals: Vec<ApprovalRequest>,
    pub audit_log: Vec<AuditEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct AgentResult {
    pub agent: AgentType,
    pub success: bool,
    pub data: serde_json::Value,
    pub execution_time_ms: u64,
    pub can_proceed: bool,  // Human approved this step
    pub human_modifications: Vec<HumanModification>,
}
```

---

## 5. API Endpoints

### 5.1 Workflow Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/workflows` | Start nieuwe workflow |
| GET | `/api/workflows/:id` | Haal workflow status op |
| GET | `/api/workflows` | Lijst alle workflows |
| DELETE | `/api/workflows/:id` | Annuleer workflow |

### 5.2 Human Interaction

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/workflows/:id/pending` | Haal openstaande goedkeuringen |
| POST | `/api/workflows/:id/approve` | Keur agent resultaat goed |
| POST | `/api/workflows/:id/modify` | Wijzig agent resultaat |
| POST | `/api/workflows/:id/reject` | Weiger agent resultaat |

### 5.3 Events

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/workflows/:id/events` | Haal event log op |
| GET | `/api/workflows/:id/stream` | SSE stream voor live updates |

---

## 6. Success Criteria

- [ ] Document creatie is volledig georkestreerd door agents
- [ ] Menselijke goedkeuring is verplicht na elke agent
- [ ] Alle stappen zijn traceerbaar via audit trail
- [ ] Implementatie past binnen bestaande Rust architecture
- [ ] Workflow kan resume na crash
- [ ] Agents kunnen parallel lopen waar mogelijk
- [ ] Timeout is configureerbaar per document type
