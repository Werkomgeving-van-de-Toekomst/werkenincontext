Now I have all the context needed. Let me generate the section content for `section-02-parallel-executor`. Based on the index.md, this section covers:

**section-02-parallel-executor**
Agent trait definition, parallel agent executor with Tokio JoinSet, shared state synchronization using Arc<RwLock<WorkflowContext>>, cancellation token pattern, event bus with broadcast and bounded mpsc channels for critical events.

Let me extract the relevant content from the plan and TDD documents.

---

# Section 02: Parallel Agent Executor

## Overview

This section implements the core parallel execution infrastructure for AI agents. It defines the `Agent` trait that all AI agents must implement, builds a parallel executor using Tokio `JoinSet`, manages shared state synchronization with `Arc<RwLock<WorkflowContext>>`, implements coordinated cancellation, and creates an event bus for component communication.

**Dependencies:** This section depends on `section-01-state-machine` being completed first.

## Files to Create/Modify

### New Files

| File | Purpose |
|------|---------|
| `server/crates/iou-orchestrator/src/executor/mod.rs` | Executor module exports |
| `server/crates/iou-orchestrator/src/executor/parallel.rs` | Parallel agent execution with JoinSet |
| `server/crates/iou-orchestrator/src/executor/retry.rs` | Retry logic with exponential backoff |
| `server/crates/iou-orchestrator/src/agent/trait.rs` | Agent trait definition |
| `server/crates/iou-orchestrator/src/agent/types.rs` | Agent input/output types |
| `server/crates/iou-orchestrator/src/event_bus/mod.rs` | Event bus module exports |
| `server/crates/iou-orchestrator/src/event_bus/bus.rs` | Event bus implementation |
| `server/crates/iou-orchestrator/src/event_bus/events.rs` | Event type definitions |

### Files to Modify

| File | Changes |
|------|---------|
| `server/crates/iou-orchestrator/src/lib.rs` | Export new modules |
| `server/crates/iou-orchestrator/Cargo.toml` | Add `tokio-util` dependency for `CancellationToken` |

## Tests

Write these tests in `server/crates/iou-orchestrator/tests/executor_tests.rs` before implementing:

### execute_layer_parallel Tests

**Test: execute_layer_parallel runs all agents in layer**
- Create layer with 2 mock agents
- Mock agent executor returns success results
- Call execute_layer_parallel
- Assert HashMap contains results for both agents
- Assert both agents marked as completed

**Test: execute_layer_parallel retries transient failures**
- Create layer with agent that fails twice then succeeds
- Configure retry policy with max_attempts=3
- Call execute_layer_parallel
- Assert agent result is success after retries
- Assert 3 execution attempts were made

**Test: execute_layer_parallel fails workflow on permanent error**
- Create layer with agent that returns permanent error
- Call execute_layer_parallel
- Assert returns ExecutorError::PermanentFailure
- Assert workflow marked as failed

**Test: execute_layer_parallel continues with optional agent failure**
- Create layer with required and optional agents
- Mock optional agent fails
- Call execute_layer_parallel
- Assert returns success with partial results
- Assert required agent result present

### Cancellation Tests

**Test: CancellationToken cancels running agents**
- Start execute_layer_parallel with 2 agents
- Send cancellation signal
- Assert both agents receive cancellation
- Assert execution returns early with Cancelled error

### Concurrency Tests

**Test: RwLock context allows concurrent reads**
- Spawn multiple tasks reading from WorkflowContext
- Assert no deadlocks occur
- Assert all tasks read consistent state

**Test: loom test for concurrent result writes**
- Use loom to simulate multiple agents writing results simultaneously
- Assert all results written correctly
- Assert no data races

### Agent Trait Tests

**Test: Agent trait implementation for Research agent**
- Create ResearchAgent instance
- Call execute() with mock input
- Assert returns AgentOutput with expected structure
- Assert agent_type() returns AgentType::Research

### Event Bus Tests

**Test: EventBus broadcasts events to all subscribers**
- Create EventBus with 3 subscribers
- Publish WorkflowCreated event
- Assert all 3 subscribers receive the event

**Test: Critical events use bounded channel with backpressure**
- Create EventBus with critical channel capacity 10
- Send 15 events rapidly
- Assert first 10 succeed, next 5 return error
- Assert no events lost

**Test: Slow broadcast subscriber is dropped**
- Create EventBus with 1 slow subscriber (doesn't consume)
- Publish 100 events rapidly (channel size < 100)
- Assert slow subscriber misses events
- Assert other subscribers unaffected

**Test: Command channel processes workflow commands**
- Send StartWorkflow command
- Assert command received and processed
- Assert workflow started

**Test: Audit logger subscribes to critical channel**
- Create audit logger subscribing to critical events
- Publish ApprovalDecision event
- Assert audit logger writes to PostgreSQL
- Assert audit entry contains all event fields

## Implementation Details

### 1. Agent Trait Definition

**Location:** `server/crates/iou-orchestrator/src/agent/trait.rs`

Define the core abstraction that all AI agents implement:

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Core trait that all AI agents must implement
#[async_trait]
pub trait Agent: Send + Sync {
    /// Execute the agent with the given input
    async fn execute(&self, input: AgentInput) -> Result<AgentOutput, AgentError>;
    
    /// Return the agent type identifier
    fn agent_type(&self) -> AgentType;
    
    /// Return list of agent types this agent depends on
    fn dependencies(&self) -> Vec<AgentType>;
    
    /// Whether this agent is required for workflow success
    fn is_required(&self) -> bool {
        true
    }
}
```

### 2. Agent Types

**Location:** `server/crates/iou-orchestrator/src/agent/types.rs`

Define input, output, and error types:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Input provided to an agent for execution
pub struct AgentInput {
    pub workflow_id: Uuid,
    pub context: WorkflowContext,
    pub dependencies: HashMap<AgentType, AgentOutput>,
}

/// Output returned by an agent after execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub agent_type: AgentType,
    pub status: ExecutionStatus,
    pub result: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

/// Execution status for an agent output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    PartialSuccess,
    Failed,
}

/// Agent type identifiers
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Research,
    Content,
    Compliance,
    Review,
}

/// Error types from agent execution
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Transient error: {0}")]
    Transient(String),
    
    #[error("Permanent error: {0}")]
    Permanent(String),
    
    #[error("Unmet dependencies: {0:?}")]
    UnmetDependencies(Vec<AgentType>),
}
```

### 3. Parallel Agent Executor

**Location:** `server/crates/iou-orchestrator/src/executor/parallel.rs`

Core function signature:

```rust
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Execute all agents in a DAG layer in parallel
pub async fn execute_layer_parallel(
    agents: Vec<AgentType>,
    context: Arc<RwLock<WorkflowContext>>,
    cancel_token: CancellationToken,
    retry_config: AgentRetryConfig,
) -> Result<HashMap<AgentType, AgentResult>, ExecutorError>
```

**Execution Flow:**

1. Create a `JoinSet` to manage spawned agent tasks
2. For each agent in the layer:
   - Read agent inputs from context (read lock)
   - Spawn a Tokio task with `cancel_token.clone()` for child tasks
   - Each task wraps execution with retry logic
3. Wait for tasks to complete with `join_all` or process `JoinSet` as tasks finish
4. Collect results into a `HashMap<AgentType, AgentResult>`
5. Handle failure scenarios:
   - If any required agent fails permanently, cancel remaining tasks
   - If optional agent fails, continue and record partial result
6. Write collected results to context (write lock)

**Shared State Pattern:**

- `WorkflowContext` is wrapped in `Arc<RwLock<T>>`
- Agent tasks take read guards to access dependencies
- Executor task takes write guard to merge results
- Use `try_read()` / `try_write()` with timeouts to prevent deadlocks

### 4. Retry Logic

**Location:** `server/crates/iou-orchestrator/src/executor/retry.rs`

```rust
/// Configuration for agent retry behavior
pub struct AgentRetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

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

/// Execute agent with retry on transient errors
pub async fn execute_with_retry(
    agent: &dyn Agent,
    input: AgentInput,
    config: &AgentRetryConfig,
    cancel_token: &CancellationToken,
) -> Result<AgentOutput, AgentError>
```

**Retry Logic:**

1. Attempt agent execution
2. On `AgentError::Transient`:
   - Calculate delay: `min(base_delay * multiplier^attempt, max_delay)`
   - Sleep with `tokio::time::sleep`
   - Check cancellation token before retry
   - Retry if attempts remaining
3. On `AgentError::Permanent`: Fail immediately
4. Return success or last error

### 5. Event Bus

**Location:** `server/crates/iou-orchestrator/src/event_bus/bus.rs`

**Event Types:**

```rust
/// Events emitted by the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
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
use tokio::sync::{broadcast, mpsc};

pub struct EventBus {
    // Broadcast to all subscribers (may drop slow receivers)
    broadcast_tx: broadcast::Sender<OrchestratorEvent>,
    
    // Bounded channel for critical events (audit, compliance)
    critical_tx: mpsc::Sender<OrchestratorEvent>,
    
    // Command channel for workflow control
    command_tx: mpsc::Sender<WorkflowCommand>,
}

impl EventBus {
    pub fn new(critical_channel_size: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        let (critical_tx, _) = mpsc::channel(critical_channel_size);
        let (command_tx, _) = mpsc::channel(100);
        
        Self { broadcast_tx, critical_tx, command_tx }
    }
    
    /// Subscribe to broadcast events (non-critical)
    pub fn subscribe(&self) -> EventBusSubscriber {
        EventBusSubscriber {
            rx: self.broadcast_tx.subscribe(),
        }
    }
    
    /// Publish a non-critical event (broadcast)
    pub async fn publish(&self, event: OrchestratorEvent) {
        let _ = self.broadcast_tx.send(event);
    }
    
    /// Publish a critical event (bounded channel, may return error)
    pub async fn publish_critical(&self, event: OrchestratorEvent) -> Result<(), EventBusError> {
        self.critical_tx
            .send(event)
            .await
            .map_err(|_| EventBusError::CriticalChannelFull)
    }
    
    /// Send a command to the orchestrator
    pub async fn send_command(&self, cmd: WorkflowCommand) -> Result<(), EventBusError> {
        self.command_tx
            .send(cmd)
            .await
            .map_err(|_| EventBusError::CommandChannelFull)
    }
}

pub struct EventBusSubscriber {
    rx: broadcast::Receiver<OrchestratorEvent>,
}

impl EventBusSubscriber {
    pub async fn recv(&mut self) -> Result<OrchestratorEvent, broadcast::error::RecvError> {
        self.rx.recv().await
    }
}
```

**Backpressure Handling:**

| Channel Type | Purpose | Backpressure Strategy |
|--------------|---------|----------------------|
| `broadcast::Sender` | Non-critical updates (UI, metrics) | Slow receivers are dropped - they miss events |
| `mpsc::Sender` (critical) | Audit, compliance events | Returns error when full - caller must retry/fail |
| `mpsc::Sender` (command) | Workflow control commands | Returns error when full - caller must retry/fail |

### 6. Cancellation Token Pattern

**Location:** `server/crates/iou-orchestrator/src/executor/parallel.rs`

Use `tokio_util::sync::CancellationToken` for coordinated cancellation:

```rust
use tokio_util::sync::CancellationToken;

// In execute_layer_parallel:
let parent_token = cancel_token;
let child_token = parent_token.clone();

// Spawn agent task
task::spawn(async move {
    tokio::select! {
        result = agent.execute(input) => result,
        _ = child_token.cancelled() => {
            Err(AgentError::Cancelled)
        }
    }
});

// To cancel all agents on permanent failure:
if let Err(ExecutorError::PermanentFailure) = agent_result {
    parent_token.cancel();  // All child tasks receive cancellation
    break;
}
```

### 7. Agent Result Type

**Location:** `server/crates/iou-orchestrator/src/agent/types.rs`

```rust
/// Result of an agent execution attempt
#[derive(Debug, Clone)]
pub struct AgentResult {
    pub agent_type: AgentType,
    pub status: ExecutionStatus,
    pub output: Option<serde_json::Value>,
    pub attempts: u32,
    pub duration_ms: u64,
    pub error: Option<String>,
}
```

## Dependencies on Other Sections

This section depends on:

- **section-01-state-machine**: Uses the `WorkflowState` enum and `DagLayer` structure. The state machine must already define `ParallelExecuting`, `PartialComplete`, and `MergeResults` states.

## Error Types

Add to `server/crates/iou-orchestrator/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Permanent failure from agent: {0}")]
    PermanentFailure(String),
    
    #[error("All agents in layer failed")]
    AllAgentsFailed,
    
    #[error("Execution cancelled")]
    Cancelled,
    
    #[error("Context lock poisoned")]
    ContextLockPoisoned,
    
    #[error("No agents in layer")]
    EmptyLayer,
}

#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Critical channel full - backpressure")]
    CriticalChannelFull,
    
    #[error("Command channel full - backpressure")]
    CommandChannelFull,
}
```

## Cargo.toml Additions

Add to `server/crates/iou-orchestrator/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
tokio-util = { version = "0.7", features = ["sync"] }
```

## Implementation Checklist

1. Create `agent/types.rs` with `AgentInput`, `AgentOutput`, `AgentType`, `AgentError`
2. Create `agent/trait.rs` with `Agent` trait definition
3. Create `executor/retry.rs` with `AgentRetryConfig` and `execute_with_retry`
4. Create `executor/parallel.rs` with `execute_layer_parallel` using `JoinSet`
5. Create `event_bus/events.rs` with `OrchestratorEvent` enum
6. Create `event_bus/bus.rs` with `EventBus` struct and backpressure handling
7. Update `lib.rs` to export all new modules
8. Write all tests before implementation
9. Implement with cancellation token support
10. Verify concurrent reads work without deadlock