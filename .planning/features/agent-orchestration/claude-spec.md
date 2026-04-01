# Agent Orchestration - Complete Specification

## Overview

Georkestreerde document creatie flow voor IOU-Modern, waarbij AI agents werken onder menselijk toezicht met goedkeuringspunten. Het systeem ondersteunt dynamische agent-executie gebaseerd op dependency graphs, met menselijke interactie inline tijdens het creatieproces.

## Status

📋 **In Planning** - Architecture designed, requirements gathered

## Problem Context

Document creation (like PROVISA documents in the Woo context) is a complex, multi-step process that involves:
- Understanding requirements and legal frameworks
- Gathering information from various sources
- Structuring content according to templates
- Validating compliance and completeness
- Reviewing and refining output

Het huidige systeem heeft al een multi-agent pipeline, maar mist:
- Dynamische executie op basis van dependencies
- Robuuste checkpoint/recovery na crashes
- Compleet human-in-the-loop systeem
- Workflow scheduling met priority queues

## Proposed Solution

### Core Architecture

```
GraphQL API + WebSocket (Real-time)
    ↓
Workflow Scheduler (Priority Queue + Preemption)
    ↓
Orchestrator (State Machine + Event Bus + Checkpoint Manager)
    ↓
Agent Executor (DAG-based execution)
    ↓
Agents (Research → Content → Compliance → Review)
```

### Key Features

1. **DAG-based Agent Execution** - Agents kunnen parallel lopen op basis van dependencies
2. **Inline Human-in-the-Loop** - Menselijke interactie tijdens document creatie
3. **Adaptive Checkpointing** - Configureerbare checkpoints per workflow type
4. **Priority Queue Scheduling** - Workflows met priority, preemption support
5. **Hybrid Storage** - DuckDB (analytics/checkpoints) + PostgreSQL (operational state)
6. **Full Observability** - Logging, metrics, dashboard, compliance audit

## Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Async Runtime | Tokio 1.43 | Existing, full-featured |
| State Machine | Hand-rolled (existing) | Better async integration than smlang |
| Channels | tokio::sync (mpsc, oneshot) | Async agent coordination |
| Storage | DuckDB + PostgreSQL | Analytics + operational data |
| API | GraphQL + WebSocket | Flexible queries + real-time |
| Testing | loom + tokio-test | Deterministic concurrency testing |
| Observability | tracing + Prometheus | Structured logs + metrics |

## Agents

| Agent | Responsibility | Output | Dependencies |
|-------|---------------|--------|--------------|
| Research | Haalt relevante wetgeving, precedenten | ResearchResult | - |
| Content | Genereert document concept | ContentDraft | ResearchResult |
| Compliance | Checkt tegen regels (PROVISA, Woo, etc.) | ComplianceReport | ContentDraft |
| Review | Eindredactie en kwaliteitstoets | FinalDocument | ComplianceReport |

## State Machine

### States

```
Created → Running → AwaitingApproval → Running → ... → Completed
                              ↓
                         AwaitingEscalation
                              ↓
                           Completed
```

**Terminal states:** `Completed`, `Failed`, `Cancelled`, `Archived`
**Retry state:** `Retrying`

### Events

- `Start` - Begin workflow execution
- `AgentComplete` - Agent finished successfully
- `AgentCompletePending` - Agent finished, awaiting approval
- `AllAgentsComplete` - All agents in DAG completed
- `AgentFailed` - Agent failed with retryable error
- `RetryAttempt` - Retry initiated
- `MaxRetriesExceeded` - Retry limit reached
- `Approved` - Human approved pending work
- `Modified` - Human modified pending work
- `Rejected` - Human rejected pending work
- `TimeoutEscalated` - Approval timeout, escalated
- `Cancel` - Workflow cancelled
- `Archive` - Workflow archived

### Guards

- `can_proceed` - No pending approvals
- `can_retry_any` - At least one agent can retry
- `can_escalate` - Approval timeout exceeded

## DAG-Based Execution

### Dependency Graph

```
        Research
           ↓
        Content
           ↓
       Compliance ← (parallel validation agents possible)
           ↓
        Review
```

### Execution Model

1. Build DAG from agent dependencies
2. Topological sort for execution order
3. Execute independent agents in parallel
4. Wait at approval points
5. Continue after approval/modification
6. Repeat until all agents complete

## Human-in-the-Loop

### Approval Flow

```
Agent produces output → Show in UI (inline)
    ↓
Human reviews inline
    ↓
Options: Approve | Modify | Reject | Request Changes
    ↓
Workflow continues with decision
```

### Approval Types

- **Approve** - Accept output as-is
- **Modify** - Edit output inline, track changes
- **Reject** - Reject output, require regeneration
- **Request Changes** - Provide feedback for next iteration

### Notification Channels

1. **WebSocket/SSE** - Real-time browser notifications
2. **Email** - Pending approval notifications
3. **Dashboard Queue** - In-app approval list
4. **Mobile Push** - Via existing mobile infrastructure

### Approval Data Structures

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

pub struct HumanModification {
    pub path: String,           // JSON path to modified field
    pub original_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub reason: String,
}
```

## Checkpoint & Recovery

### Adaptive Checkpointing

- **Configurable per workflow type**
- **More frequent during long workflows**
- **Balance between overhead and recovery granularity**

### Checkpoint Points

1. After each agent completes
2. Before approval points
3. After approval decisions
4. On workflow pause/cancel

### Storage Strategy

- **PostgreSQL** - Active workflow state, fast queries
- **DuckDB** - Checkpoint snapshots, analytics

### Recovery Flow

```
Crash detected → Find latest checkpoint
    ↓
Restore state from DuckDB snapshot
    ↓
Rebuild workflow from PostgreSQL logs
    ↓
Resume from last completed agent
```

## Workflow Scheduling

### Priority Queue

- Workflows queued by priority level
- Higher priority workflows can preempt
- Preemption saves state, pauses, resumes later

### Priority Levels

1. **Critical** - Immediate execution, preempts all
2. **High** - Executes next, preempts normal/low
3. **Normal** - Default priority
4. **Low** - Background execution

### Scheduler Behavior

```
New workflow arrives → Check priority
    ↓
If higher than running → Preempt lowest priority
    ↓
Queue workflow by priority
    ↓
Execute when resources available
```

## API Design

### GraphQL Schema

```graphql
type Workflow {
  id: ID!
  status: WorkflowStatus!
  priority: Priority!
  currentAgent: AgentType
  pendingApprovals: [ApprovalRequest!]!
  createdAt: DateTime!
  updatedAt: DateTime!
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
  reject(input: RejectionInput!): Workflow!
  cancelWorkflow(id: ID!): Workflow!
}

type Subscription {
  workflowUpdated(id: ID!): Workflow!
  approvalRequired: ApprovalRequest!
}
```

### WebSocket Events

```typescript
// Client → Server
{ type: "approve", requestId: string, comment?: string }
{ type: "modify", requestId: string, modifications: Modification[], comment?: string }
{ type: "reject", requestId: string, reason: string }

// Server → Client
{ type: "approval_required", request: ApprovalRequest }
{ type: "workflow_updated", workflow: Workflow }
{ type: "agent_started", agent: AgentType }
{ type: "agent_completed", agent: AgentType, result: AgentResult }
```

## Error Handling

### Per-Agent Retry Configuration

```rust
pub struct AgentRetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

// Default configurations
const RESEARCH_RETRY: AgentRetryConfig = AgentRetryConfig {
    max_attempts: 3,
    base_delay_ms: 1000,
    max_delay_ms: 30000,
    backoff_multiplier: 2.0,
};
```

### Error Severity

- **Transient** - Retry with backoff (network, temporary API issues)
- **Permanent** - Fail immediately (invalid input, configuration errors)

## Observability

### Logging

- Structured logging with `tracing`
- Workflow ID in all logs
- State transitions logged
- Agent execution timed

### Metrics

- Agent execution time (p50, p95, p99)
- Success rate per agent type
- Approval response time
- Workflow completion time
- Queue depth by priority

### Dashboard UI

- Real-time workflow status
- Pending approval queue
- Agent execution visualization
- Error rate monitoring

### Compliance Audit

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
    AgentStarted { agent: AgentType },
    AgentCompleted { agent: AgentType },
    ApprovalRequested { agent: AgentType },
    ApprovalDecision { decision: ApprovalDecision },
    WorkflowCompleted,
    // ... more events
}

pub enum AuditActor {
    System,
    User { id: Uuid, name: String },
    Agent { agent_type: AgentType },
}
```

## Testing Strategy

### State Machine Tests

- All valid state-event transitions
- Invalid transitions return errors
- Guard functions tested
- Serialization/deserialization

### Concurrency Tests (loom)

- Concurrent state mutations
- Channel communication
- Race conditions in transitions
- Cancellation behavior

### Integration Tests

- End-to-end workflow execution
- Agent integration with real backends
- Approval flow with WebSocket
- Checkpoint save/restore

### Approval UX Tests

- UI interactions
- Notification delivery
- Modification tracking
- Escalation on timeout

## Implementation Phases

### Phase 1 (MVP) - Complete Feature Set

1. **Core Orchestration**
   - State machine enhancement for DAG execution
   - Agent executor with parallel execution
   - Event bus for agent coordination

2. **Checkpoint/Recovery**
   - Adaptive checkpoint policy
   - DuckDB snapshot storage
   - PostgreSQL operational state
   - Recovery workflow

3. **Human-in-the-Loop**
   - GraphQL API with subscriptions
   - WebSocket real-time updates
   - Approval endpoints
   - Notification system (all channels)

4. **Workflow Scheduling**
   - Priority queue implementation
   - Scheduler with preemption
   - Multi-workflow execution

### Phase 2 - Production Hardening

5. **Monitoring & Observability**
   - Prometheus metrics
   - Dashboard UI
   - Compliance audit logging

6. **Performance Optimization**
   - Connection pooling
   - Caching strategies
   - Database optimization

### Phase 3 - Advanced Features

7. **Advanced Workflows**
   - Conditional execution
   - Loop/retry patterns
   - Sub-workflows

8. **Agent Marketplace**
   - Dynamic agent registration
   - Agent versioning
   - A/B testing

## Non-Functional Requirements

### Performance

- Workflow creation: < 100ms
- Agent start: < 500ms
- Approval notification: < 1s
- Checkpoint save: < 100ms
- Recovery time: < 5s

### Scalability

- Support 1000+ concurrent workflows
- 10+ agents executing in parallel
- Sub-second approval notifications

### Reliability

- 99.9% uptime for orchestrator
- Zero data loss (durable checkpoints)
- Graceful degradation on agent failure

### Security

- Authentication for all approvals
- Authorization checks per workflow
- Audit trail for all decisions
- Encrypted data at rest

## Dependencies

### Internal

- `iou-ai` - Agent implementations
- `iou-api` - Existing API structure
- `iou-core` - Workflow types

### External

- Tokio - Async runtime
- DuckDB - Embedded database
- PostgreSQL - Relational database
- Axum - Web framework
- async-graphql - GraphQL implementation

---

*This specification synthesizes the original feature README, codebase research, web research on state machines and human-in-the-loop patterns, and detailed interview responses.*
