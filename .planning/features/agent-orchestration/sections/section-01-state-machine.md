Now I have all the context needed. Let me generate the self-contained section content for `section-01-state-machine`.

# Section 01: State Machine Enhancement for DAG Execution

## Overview

This section implements the core state machine enhancements required to support dynamic DAG-based agent execution. The state machine tracks workflow state, manages parallel agent execution, and handles approval checkpoints with escalation support.

**Key Files:**
- `/server/crates/iou-orchestrator/src/state_machine/mod.rs`
- `/server/crates/iou-orchestrator/src/state_machine/base.rs`
- `/server/crates/iou-orchestrator/src/state_machine/events.rs`
- `/server/crates/iou-orchestrator/src/dag/mod.rs`
- `/server/crates/iou-orchestrator/src/dag/builder.rs`
- `/server/crates/iou-orchestrator/src/dag/model.rs`

**Dependencies:** None (this section has no dependencies and can be implemented first)

**Blocks:** `section-02-parallel-executor`

---

## Tests (TDD)

Write these tests FIRST in `/server/crates/iou-orchestrator/tests/state_machine_tests.rs`:

### State Transition Tests

```rust
#[tokio::test]
async fn test_state_transition_created_to_parallel_executing_after_dag_built() {
    // Create state machine in Created state
    // Send DagBuilt event with valid DAG
    // Assert state is ParallelExecuting
    // Assert DAG layers are stored in state machine
}

#[tokio::test]
async fn test_state_transition_parallel_executing_to_partial_complete_on_agent_completion() {
    // Start in ParallelExecuting with 2-agent layer
    // Send PartialAgentComplete event for one agent
    // Assert state is PartialComplete
    // Assert completed agent tracked in DagLayer
}

#[tokio::test]
async fn test_state_transition_partial_complete_to_merge_results_when_layer_complete() {
    // Start in PartialComplete with all agents complete
    // Send DagLayerComplete event
    // Assert state is MergeResults
    // Assert all layer results available
}

#[tokio::test]
async fn test_agent_failure_during_parallel_executing_cancels_all_and_fails_workflow() {
    // Start in ParallelExecuting with 2-agent layer
    // Send AgentFailed event for one agent
    // Assert state is Failed
    // Assert cancellation signal was sent
}
```

### Guard Function Tests

```rust
#[tokio::test]
async fn test_guard_can_execute_parallel_returns_true_when_no_dependencies_pending() {
    // Set up state with all dependencies satisfied
    // Call can_execute_parallel guard
    // Assert returns true
}

#[tokio::test]
async fn test_guard_can_execute_parallel_returns_false_when_dependencies_pending() {
    // Set up state with incomplete dependencies
    // Call can_execute_parallel guard
    // Assert returns false
}
```

### Concurrency Tests (loom)

```rust
#[test]
fn test_concurrent_state_transitions() {
    loom::model(|| {
        // Use loom to simulate multiple threads sending events simultaneously
        // Assert state machine remains consistent
        // Assert no lost events
    });
}
```

### DAG Builder Tests

Write these tests in `/server/crates/iou-orchestrator/tests/dag_tests.rs`:

```rust
#[tokio::test]
async fn test_build_execution_dag_creates_valid_layers_from_sequential_dependencies() {
    // Provide sequential agent dependencies (A -> B -> C)
    // Call build_execution_dag
    // Assert 3 layers created with one agent each
    // Assert layer 0 has A, layer 1 has B, layer 2 has C
}

#[tokio::test]
async fn test_build_execution_dag_groups_independent_agents_into_same_layer() {
    // Provide dependencies where B and C both depend on A
    // Call build_execution_dag
    // Assert layer 0 has A, layer 1 has both B and C
}

#[tokio::test]
async fn test_build_execution_dag_returns_error_for_circular_dependencies() {
    // Provide circular dependency (A -> B -> A)
    // Call build_execution_dag
    // Assert returns DagError::CircularDependency
}

#[tokio::test]
async fn test_build_execution_dag_handles_complex_dependency_graph() {
    // Provide complex multi-branch dependency tree
    // Call build_execution_dag
    // Assert all agents assigned to correct layers
    // Assert no layer has agents with unmet dependencies
}

#[tokio::test]
async fn test_build_execution_dag_handles_empty_agent_list() {
    // Provide empty dependency list
    // Call build_execution_dag
    // Assert returns empty DAG with no layers
}
```

---

## Implementation Details

### 1. Extended WorkflowState Enum

**File:** `/server/crates/iou-orchestrator/src/state_machine/events.rs`

Extend the existing `WorkflowState` enum with new DAG-specific states:

```rust
/// The complete set of workflow states supporting both sequential and DAG execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowState {
    // Existing states preserved
    Created,
    Running,
    AwaitingApproval,
    AwaitingEscalation,
    Completed,
    Failed,
    Cancelled,
    Retrying,
    Archived,
    
    // New DAG states for parallel execution
    /// Multiple agents running concurrently within a DAG layer
    ParallelExecuting,
    /// Some agents in the current layer have completed, awaiting others
    PartialComplete,
    /// All agents in layer complete, merging results before next layer
    MergeResults,
}

impl WorkflowState {
    /// Returns true if this state represents a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled | Self::Archived)
    }
    
    /// Returns true if this state allows agent execution
    pub fn can_execute_agents(&self) -> bool {
        matches!(self, Self::Running | Self::ParallelExecuting)
    }
}

impl Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParallelExecuting => write!(f, "PARALLEL_EXECUTING"),
            Self::PartialComplete => write!(f, "PARTIAL_COMPLETE"),
            Self::MergeResults => write!(f, "MERGE_RESULTS"),
            // ... existing states
        }
    }
}
```

### 2. DAG Data Structures

**File:** `/server/crates/iou-orchestrator/src/dag/model.rs`

```rust
use std::collections::HashSet;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Represents a single layer in the execution DAG
/// Agents within a layer can execute in parallel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagLayer {
    /// Zero-based index of this layer in the DAG
    pub layer_index: usize,
    /// Agents in this layer (all can run concurrently)
    pub agents: Vec<AgentType>,
    /// Track which agents in this layer have completed
    pub completed: HashSet<AgentType>,
}

impl DagLayer {
    pub fn new(layer_index: usize, agents: Vec<AgentType>) -> Self {
        Self {
            layer_index,
            agents,
            completed: HashSet::new(),
        }
    }
    
    /// Returns true if all agents in this layer are complete
    pub fn is_complete(&self) -> bool {
        self.completed.len() == self.agents.len()
    }
    
    /// Returns true if any agent in this layer has completed
    pub fn has_any_complete(&self) -> bool {
        !self.completed.is_empty()
    }
    
    /// Mark an agent as completed
    pub fn mark_complete(&mut self, agent: AgentType) -> Result<(), DagError> {
        if !self.agents.contains(&agent) {
            return Err(DagError::AgentNotInLayer { agent, layer: self.layer_index });
        }
        self.completed.insert(agent);
        Ok(())
    }
}

/// The complete execution DAG for a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDag {
    /// Ordered layers of agents (layers execute sequentially, agents within layer execute in parallel)
    pub layers: Vec<DagLayer>,
    /// Current layer being executed (None = not started)
    pub current_layer: Option<usize>,
}

impl ExecutionDag {
    pub fn new(layers: Vec<DagLayer>) -> Self {
        Self {
            layers,
            current_layer: None,
        }
    }
    
    /// Start execution from the first layer
    pub fn start(&mut self) -> Result<(), DagError> {
        if self.layers.is_empty() {
            return Err(DagError::EmptyDag);
        }
        self.current_layer = Some(0);
        Ok(())
    }
    
    /// Get the current layer
    pub fn current_layer(&self) -> Option<&DagLayer> {
        self.current_layer.and_then(|idx| self.layers.get(idx))
    }
    
    /// Advance to the next layer
    pub fn advance_layer(&mut self) -> Result<(), DagError> {
        match self.current_layer {
            Some(idx) if idx + 1 < self.layers.len() => {
                self.current_layer = Some(idx + 1);
                Ok(())
            }
            Some(_) => Err(DagError::NoMoreLayers),
            None => Err(DagError::NotStarted),
        }
    }
    
    /// Returns true if all layers are complete
    pub fn is_complete(&self) -> bool {
        self.layers.iter().all(DagLayer::is_complete)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DagError {
    #[error("Circular dependency detected involving agent: {0}")]
    CircularDependency(String),
    
    #[error("Agent {agent:?} not found in layer {layer}")]
    AgentNotInLayer { agent: AgentType, layer: usize },
    
    #[error("DAG is empty, no agents to execute")]
    EmptyDag,
    
    #[error("DAG execution not started")]
    NotStarted,
    
    #[error("No more layers to execute")]
    NoMoreLayers,
    
    #[error("Invalid agent dependency: {0}")]
    InvalidDependency(String),
}
```

### 3. Agent Dependency Definition

**File:** `/server/crates/iou-orchestrator/src/dag/model.rs` (continued)

```rust
/// Defines agent dependencies for DAG construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDependency {
    pub agent: AgentType,
    pub depends_on: Vec<AgentType>,
}

/// Known agent types in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Research,
    Content,
    Compliance,
    Review,
}

// Document creation workflow dependencies (sequential for now, parallel-ready)
const DOCUMENT_WORKFLOW_DEPS: &[AgentDependency] = &[
    AgentDependency { agent: AgentType::Research, depends_on: vec![] },
    AgentDependency { agent: AgentType::Content, depends_on: vec![AgentType::Research] },
    AgentDependency { agent: AgentType::Compliance, depends_on: vec![AgentType::Content] },
    AgentDependency { agent: AgentType::Review, depends_on: vec![AgentType::Compliance] },
];
```

### 4. DAG Builder

**File:** `/server/crates/iou-orchestrator/src/dag/builder.rs`

```rust
use std::collections::{HashMap, HashSet, VecDeque};

/// Builds an execution DAG from agent dependency definitions
pub fn build_execution_dag(agents: &[AgentDependency]) -> Result<ExecutionDag, DagError> {
    if agents.is_empty() {
        return Ok(ExecutionDag::new(vec![]));
    }
    
    // Step 1: Validate no circular dependencies using Kahn's algorithm
    let mut graph: HashMap<AgentType, Vec<AgentType>> = HashMap::new();
    let mut in_degree: HashMap<AgentType, usize> = HashMap::new();
    
    for dep in agents {
        graph.insert(dep.agent, dep.depends_on.clone());
        *in_degree.entry(dep.agent).or_insert(0) += dep.depends_on.len();
    }
    
    // Check for cycles and perform topological sort
    let mut queue: VecDeque<AgentType> = in_degree
        .iter()
        .filter(|(_, &degree)| degree == 0)
        .map(|(&agent, _)| agent)
        .collect();
    
    let mut sorted: Vec<AgentType> = Vec::new();
    
    while let Some(agent) = queue.pop_front() {
        sorted.push(agent);
        
        // Find all agents that depend on this one
        for dependent in agents.iter().filter_map(|dep| {
            if dep.depends_on.contains(&agent) {
                Some(dep.agent)
            } else {
                None
            }
        }) {
            if let Some(degree) = in_degree.get_mut(&dependent) {
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(dependent);
                }
            }
        }
    }
    
    // If not all agents were processed, there's a cycle
    if sorted.len() != agents.len() {
        return Err(DagError::CircularDependency(
            "Cycle detected in agent dependencies".to_string()
        ));
    }
    
    // Step 2: Group independent agents into layers
    let mut layers: Vec<DagLayer> = Vec::new();
    let mut placed: HashSet<AgentType> = HashSet::new();
    
    for agent in sorted {
        // Find which layer this agent belongs to
        // An agent can be in a layer if all its dependencies are in previous layers
        
        // Get dependencies
        let deps = agents.iter()
            .find(|d| d.agent == agent)
            .map(|d| &d.depends_on)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        
        // Find the earliest layer that can accommodate this agent
        // (all dependencies must be in layers < this layer)
        let mut target_layer = 0;
        
        for (layer_idx, layer) in layers.iter().enumerate() {
            // Check if all dependencies are satisfied by this layer
            let deps_satisfied = deps.iter().all(|dep| {
                layers[..=layer_idx].iter().any(|l| l.agents.contains(dep))
            });
            
            if deps_satisfied && target_layer == 0 {
                target_layer = layer_idx;
            }
        }
        
        // Add to existing layer or create new one
        if target_layer < layers.len() {
            layers[target_layer].agents.push(agent);
        } else {
            layers.push(DagLayer::new(layers.len(), vec![agent]));
        }
        
        placed.insert(agent);
    }
    
    Ok(ExecutionDag::new(layers))
}
```

### 5. DAG-Specific Events

**File:** `/server/crates/iou-orchestrator/src/state_machine/events.rs` (extend existing events)

```rust
/// Events that trigger state transitions in the workflow state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    // Existing events (preserved)
    Start,
    AgentComplete { agent: AgentType },
    ApprovalReceived,
    ApprovalRejected,
    EscalationReceived,
    Timeout,
    Cancel,
    Retry,
    
    // New DAG-specific events
    /// DAG has been constructed and is ready for execution
    DagBuilt { dag: ExecutionDag },
    
    /// Starting multiple agents in parallel
    AgentParallelStart { agents: Vec<AgentType> },
    
    /// One agent in a parallel set has completed
    PartialAgentComplete { agent: AgentType },
    
    /// All agents in the current DAG layer have completed
    DagLayerComplete,
    
    /// Results from parallel agents have been merged
    MergeComplete,
    
    /// An agent failed during execution
    AgentFailed { agent: AgentType, error: String },
}
```

### 6. Extended State Machine

**File:** `/server/crates/iou-orchestrator/src/state_machine/base.rs`

Extend the existing `WorkflowStateMachine` struct:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct WorkflowStateMachine {
    pub workflow_id: Uuid,
    pub state: WorkflowState,
    pub dag: Option<ExecutionDag>,
    pub context: Arc<RwLock<WorkflowContext>>,
    // ... existing fields
}

impl WorkflowStateMachine {
    /// Handle a workflow event and transition state
    pub async fn handle_event(&mut self, event: WorkflowEvent) -> Result<StateTransition, StateMachineError> {
        let transition = StateTransition {
            from: self.state,
            event: event.clone(),
            to: self.state, // Will be updated
        };
        
        let new_state = match (&self.state, &event) {
            // DAG construction
            (WorkflowState::Created, WorkflowEvent::DagBuilt { dag }) => {
                self.dag = Some(dag.clone());
                WorkflowState::ParallelExecuting
            }
            
            // Parallel execution
            (WorkflowState::ParallelExecuting, WorkflowEvent::PartialAgentComplete { .. }) => {
                WorkflowState::PartialComplete
            }
            
            (WorkflowState::ParallelExecuting, WorkflowEvent::DagLayerComplete) => {
                WorkflowState::MergeResults
            }
            
            // Partial completion handling
            (WorkflowState::PartialComplete, WorkflowEvent::PartialAgentComplete { .. }) => {
                // Stay in PartialComplete, waiting for more agents
                WorkflowState::PartialComplete
            }
            
            (WorkflowState::PartialComplete, WorkflowEvent::DagLayerComplete) => {
                WorkflowState::MergeResults
            }
            
            // Merge results
            (WorkflowState::MergeResults, WorkflowEvent::MergeComplete) => {
                // Advance to next layer or complete
                if let Some(dag) = &mut self.dag {
                    if let Err(e) = dag.advance_layer() {
                        tracing::warn!("Failed to advance DAG layer: {}", e);
                        return Ok(transition.to(WorkflowState::Completed));
                    }
                    if dag.current_layer().is_some() {
                        WorkflowState::ParallelExecuting
                    } else {
                        WorkflowState::Completed
                    }
                } else {
                    WorkflowState::Completed
                }
            }
            
            // Agent failure during parallel execution
            (WorkflowState::ParallelExecuting, WorkflowEvent::AgentFailed { .. }) => {
                // Cancel all agents in current layer
                WorkflowState::Failed
            }
            
            // ... preserve existing state transitions
            
            _ => {
                return Err(StateMachineError::InvalidTransition {
                    from: self.state,
                    event: format!("{:?}", event),
                });
            }
        };
        
        let transition = transition.to(new_state);
        self.state = new_state;
        Ok(transition)
    }
    
    /// Guard: Check if parallel execution can proceed
    pub fn can_execute_parallel(&self) -> bool {
        match &self.dag {
            Some(dag) => {
                if let Some(layer) = dag.current_layer() {
                    // Check if all dependencies for this layer's agents are satisfied
                    // For now, if we're in ParallelExecuting state, we can proceed
                    self.state == WorkflowState::ParallelExecuting
                } else {
                    false
                }
            }
            None => false,
        }
    }
    
    /// Guard: Check if all dependent agents are complete
    pub fn all_dependents_complete(&self, agent: AgentType) -> bool {
        // Check if all agents that `agent` depends on are complete
        self.dag.as_ref().and_then(|dag| {
            dag.layers.iter().find(|layer| layer.agents.contains(&agent))
        }).map(|layer| {
            layer.agents.iter().all(|a| {
                // All agents in previous layers should be complete
                self.dag.as_ref().map(|dag| {
                    dag.layers.iter().take_while(|l| l.layer_index < layer.layer_index)
                        .all(|l| l.completed.contains(a))
                }).unwrap_or(false)
            })
        }).unwrap_or(false)
    }
}

struct StateTransition {
    from: WorkflowState,
    event: WorkflowEvent,
    to: WorkflowState,
}

impl StateTransition {
    fn to(mut self, to: WorkflowState) -> Self {
        self.to = to;
        self
    }
}
```

---

## State Transition Table

| Current State | Event | Next State | Guard | Notes |
|--------------|-------|------------|-------|-------|
| Created | DagBuilt | ParallelExecuting | DAG valid | DAG constructed from dependencies |
| ParallelExecuting | PartialAgentComplete | PartialComplete | any_complete | Some agents done |
| ParallelExecuting | DagLayerComplete | MergeResults | all_complete | All layer agents done |
| ParallelExecuting | AgentFailed | Failed | - | Cancel all agents in layer |
| PartialComplete | PartialAgentComplete | PartialComplete | more_pending | Waiting for more agents |
| PartialComplete | DagLayerComplete | MergeResults | all_complete | Ready to merge results |
| MergeResults | MergeComplete | ParallelExecuting | has_next_layer | Continue to next layer |
| MergeResults | MergeComplete | Completed | no_next_layer | All layers complete |
| Any | Cancel | Cancelled | - | User cancelled workflow |
| Any | FatalError | Failed | - | Unrecoverable error |

---

## Error Handling

**File:** `/server/crates/iou-orchestrator/src/state_machine/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum StateMachineError {
    #[error("Invalid state transition from {from:?} with event {event}")]
    InvalidTransition { from: WorkflowState, event: String },
    
    #[error("Cannot execute parallel: {0}")]
    CannotExecuteParallel(String),
    
    #[error("DAG not built for workflow {0}")]
    DagNotBuilt(Uuid),
    
    #[error("Agent {0:?} failed: {1}")]
    AgentExecutionFailed(AgentType, String),
    
    #[error("Workflow stuck in state {0:?} - no progress for configured timeout")]
    WorkflowStuck(WorkflowState),
}
```

---

## Module Exports

**File:** `/server/crates/iou-orchestrator/src/state_machine/mod.rs`

```rust
mod base;
mod events;
mod error;

pub use base::{WorkflowStateMachine, StateTransition};
pub use events::{WorkflowState, WorkflowEvent};
pub use error::{StateMachineError};

// Re-export DAG types
pub use crate::dag::{ExecutionDag, DagLayer, AgentType, DagError};
```

**File:** `/server/crates/iou-orchestrator/src/dag/mod.rs`

```rust
mod builder;
mod model;

pub use builder::build_execution_dag;
pub use model::{ExecutionDag, DagLayer, AgentDependency, AgentType, DagError};
```

---

## Dependencies to Add

Update `/server/crates/iou-orchestrator/Cargo.toml`:

```toml
[dependencies]
# Existing dependencies preserved

# New for this section
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
uuid = { version = "1.0", features = ["serde"] }
```

---

## Integration Notes

This section is completely self-contained with no dependencies on other sections. However, it establishes the foundation for:

1. **Section 02 (Parallel Executor):** The `ExecutionDag` and `DagLayer` structures are used by the parallel executor to determine which agents can run concurrently.

2. **Section 03 (Checkpoint & Recovery):** The `WorkflowState` enum and `ExecutionDag` are serialized into checkpoints for recovery.

3. **Section 04 (GraphQL API):** The state is exposed via GraphQL queries for workflow status.

The state machine uses `Arc<RwLock<WorkflowContext>>` for shared state, which will be populated by the parallel executor in the next section.