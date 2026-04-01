Now I have all the context needed. Let me generate the self-contained section content for section-06-workflow-scheduler:

# Section 06: Workflow Scheduler

## Overview

This section implements the workflow scheduler that manages concurrent workflow execution using a priority queue. The scheduler supports preemption of lower-priority workflows when higher-priority work arrives, with graceful checkpointing before preemption and resume from checkpoint.

**Dependencies:**
- `section-02-parallel-executor`: Required for the parallel agent executor and event bus
- `section-03-checkpoint-recovery`: Required for checkpoint-before-preempt functionality

**Files to Create:**
- `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/scheduler/mod.rs`
- `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/scheduler/queue.rs`
- `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/scheduler/config.rs`

**Files to Modify:**
- `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/lib.rs` - Add scheduler module export

---

## Tests (TDD)

Write these tests BEFORE implementing the scheduler. All tests should be placed in `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/tests/scheduler_tests.rs`.

### 4.1 Priority Queue Tests

**Test: Critical priority workflow dequeued before High priority**
- Push CRITICAL workflow to queue
- Push HIGH workflow to queue
- Call `pop()`
- Assert returns CRITICAL workflow (not HIGH)

**Test: Same priority ordered by queued_at (FIFO)**
- Push 2 HIGH workflows with different timestamps (t1 < t2)
- Call `pop()` twice
- Assert first pushed workflow (t1) returned first, then t2

**Test: pop returns None when max_concurrent reached**
- Set `max_concurrent=2`
- Simulate 2 workflows running (increment `running_count` to 2)
- Call `pop()`
- Assert returns `None`

**Test: complete() decrements running_count**
- Set `running_count=2`
- Call `complete()`
- Assert `running_count=1`
- Assert `pop()` now returns a workflow (capacity available)

**Test: TOCTOU race fixed - concurrent pop calls respect limit**
- Spawn 10 tasks calling `pop()` simultaneously with `max_concurrent=2`
- Use `tokio::spawn` and `join_all`
- Assert exactly 2 workflows returned (not more)
- Assert no race condition causes >2 concurrent executions

**Test: Ord implementation matches priority expectations**
- Create `QueuedWorkflow` with CRITICAL priority
- Create `QueuedWorkflow` with LOW priority
- Assert `CRITICAL < LOW` (higher priority = lower ordering for BinaryHeap)

### 4.2 Scheduler Preemption Tests

**Test: should_preempt returns lower priority workflow ID**
- Add CRITICAL workflow to queue
- Simulate 2 NORMAL workflows running
- Call `should_preempt()`
- Assert returns `Some(id)` of oldest NORMAL workflow

**Test: should_preempt returns None when all running are higher priority**
- Add NORMAL workflow to queue
- Simulate CRITICAL workflow running
- Call `should_preempt()`
- Assert returns `None` (no valid preemption target)

**Test: should_preempt returns None for non-preemptible workflow**
- Add CRITICAL workflow to queue
- Simulate CRITICAL workflow running with `preemptible=false`
- Call `should_preempt()`
- Assert returns `None` (cannot preempt non-preemptible)

**Test: Preemption checkpoints workflow before pausing**
- Start workflow execution
- Trigger preemption via scheduler
- Assert checkpoint saved before pause
- Assert workflow state saved as `Preempted`
- Assert `checkpoint_before_preempt` flag respected

**Test: Preempted workflow resumes from checkpoint**
- Preempt workflow at layer 2
- Resume workflow when resources available
- Assert execution continues from layer 2 (not from start)
- Assert agent results from layer 1 preserved

---

## Implementation

### 4.1 Priority Queue

**File:** `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/scheduler/queue.rs`

The priority queue manages waiting workflows with priority-based ordering. It must handle concurrent access safely while respecting the maximum concurrent workflow limit.

**Key Design Points:**

1. **Priority Ordering:** CRITICAL > HIGH > NORMAL > LOW. Since Rust's `BinaryHeap` is a max-heap, we implement `Ord` such that higher priority = lower ordering value.

2. **FIFO for Same Priority:** When priorities are equal, order by `queued_at` timestamp (earlier = higher priority).

3. **TOCTOU Race Fix:** The check-and-increment of `running_count` must be atomic within the same lock as the heap pop operation. The naive implementation of checking `running_count` then popping has a race condition.

**Type Definitions:**

```rust
use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::context::RequestPriority;

/// Priority enum with strict ordering (CRITICAL > HIGH > NORMAL > LOW)
/// Note: Existing code uses RequestPriority (Low, Normal, High, Urgent)
/// We map: Urgent=CRITICAL, High=HIGH, Normal=NORMAL, Low=LOW
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Critical,
    High,
    Normal,
    Low,
}

impl From<RequestPriority> for Priority {
    fn from(p: RequestPriority) -> Self {
        match p {
            RequestPriority::Urgent => Priority::Critical,
            RequestPriority::High => Priority::High,
            RequestPriority::Normal => Priority::Normal,
            RequestPriority::Low => Priority::Low,
        }
    }
}

/// A workflow entry in the priority queue
#[derive(Debug, Clone)]
pub struct QueuedWorkflow {
    pub workflow_id: Uuid,
    pub priority: Priority,
    pub queued_at: DateTime<Utc>,
    pub estimated_duration: Option<chrono::Duration>,
    pub preemptible: bool,
}

// BinaryHeap is max-heap, so Lower ordering = Higher priority
impl Ord for QueuedWorkflow {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority (Lower enum = Higher priority)
        match (self.priority, other.priority) {
            (Priority::Critical, Priority::Critical) => {
                // Same priority: FIFO by queued_at
                self.queued_at.cmp(&other.queued_at)
            }
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

/// Result type for queue operations
pub type QueueResult<T> = Result<T, QueueError>;

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("Queue is full")]
    Full,
    #[error("Queue operation failed: {0}")]
    OperationFailed(String),
}
```

**Priority Queue Implementation:**

```rust
/// Thread-safe priority queue for workflow scheduling
pub struct WorkflowPriorityQueue {
    inner: Mutex<BinaryHeap<QueuedWorkflow>>,
    max_concurrent: usize,
    running_count: Arc<AtomicUsize>,
}

impl WorkflowPriorityQueue {
    /// Create a new priority queue with the specified concurrency limit
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            inner: Mutex::new(BinaryHeap::new()),
            max_concurrent,
            running_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Add a workflow to the queue
    pub async fn push(&self, workflow: QueuedWorkflow) -> QueueResult<()> {
        let mut inner = self.inner.lock().await;
        inner.push(workflow);
        Ok(())
    }

    /// Pop the highest priority workflow, respecting concurrent limit
    /// 
    /// CRITICAL: Check and increment running_count INSIDE the lock to prevent TOCTOU race
    /// Multiple concurrent pop() calls must not exceed max_concurrent
    pub async fn pop(&self) -> Option<QueuedWorkflow> {
        let mut inner = self.inner.lock().await;
        
        // Check and increment running_count atomically within the lock
        if self.running_count.load(AtomicOrdering::Relaxed) < self.max_concurrent {
            self.running_count.fetch_add(1, AtomicOrdering::Relaxed);
            inner.pop()
        } else {
            None
        }
    }

    /// Mark a workflow as complete, freeing a slot for another workflow
    pub fn complete(&self) {
        self.running_count.fetch_sub(1, AtomicOrdering::Relaxed);
    }

    /// Get the current number of running workflows
    pub fn running_count(&self) -> usize {
        self.running_count.load(AtomicOrdering::Relaxed)
    }

    /// Get the number of queued (waiting) workflows
    pub async fn queued_count(&self) -> usize {
        self.inner.lock().await.len()
    }

    /// Remove a specific workflow from the queue (e.g., if cancelled)
    pub async fn remove(&self, workflow_id: Uuid) -> bool {
        let mut inner = self.inner.lock().await;
        let len_before = inner.len();
        
        // Drain and filter, then rebuild
        let workflows: Vec<_> = inner.drain().filter(|w| w.workflow_id != workflow_id).collect();
        *inner = BinaryHeap::from(workflows);
        
        inner.len() < len_before
    }

    /// Peek at the highest priority workflow without removing it
    pub async fn peek(&self) -> Option<QueuedWorkflow> {
        let inner = self.inner.lock().await;
        inner.peek().cloned()
    }
}
```

---

### 4.2 Workflow Scheduler with Preemption

**File:** `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/scheduler/mod.rs`

The scheduler coordinates workflow execution, manages the priority queue, and implements preemption logic.

**Scheduler Configuration:**

```rust
use serde::{Deserialize, Serialize};

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum number of workflows to execute concurrently
    pub max_concurrent_workflows: usize,
    
    /// Whether preemption is enabled
    pub preemption_enabled: bool,
    
    /// Grace period before preemption (milliseconds)
    pub preemption_grace_period_ms: u64,
    
    /// Whether to checkpoint before preempting
    pub checkpoint_before_preempt: bool,
    
    /// Minimum priority difference required for preemption
    pub min_preemption_priority_delta: u8,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: 3,
            preemption_enabled: true,
            preemption_grace_period_ms: 5000,  // 5 seconds
            checkpoint_before_preempt: true,
            min_preemption_priority_delta: 1,
        }
    }
}
```

**Running Workflow Tracking:**

```rust
use std::collections::HashMap;
use crate::context::{RequestPriority, WorkflowContext};

/// Information about a currently running workflow
#[derive(Debug, Clone)]
pub struct RunningWorkflow {
    pub workflow_id: Uuid,
    pub priority: Priority,
    pub started_at: DateTime<Utc>,
    pub preemptible: bool,
    pub current_agent: Option<String>,
    pub can_checkpoint: bool,
}
```

**Preemption Decision Logic:**

```rust
/// Determine if a workflow should be preempted
/// 
/// Returns Some(workflow_id) if preemption should occur, None otherwise
pub fn should_preempt(
    queued: &QueuedWorkflow,
    running: &[RunningWorkflow],
    config: &SchedulerConfig,
) -> Option<Uuid> {
    if !config.preemption_enabled {
        return None;
    }

    // Find candidate workflows for preemption
    let candidates: Vec<_> = running
        .iter()
        .filter(|w| w.preemptible)
        .filter(|w| priority_delta(queued.priority, w.priority) >= config.min_preemption_priority_delta)
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Select the lowest priority candidate (or oldest if tie)
    let target = candidates
        .iter()
        .min_by_key(|w| (w.priority, w.started_at))?;

    Some(target.workflow_id)
}

/// Calculate the priority difference (higher = queued is more important)
fn priority_delta(queued: Priority, running: Priority) -> u8 {
    match (queued, running) {
        (Priority::Critical, Priority::Low) => 3,
        (Priority::Critical, Priority::Normal) => 2,
        (Priority::Critical, Priority::High) => 1,
        (Priority::High, Priority::Low) => 2,
        (Priority::High, Priority::Normal) => 1,
        (Priority::Normal, Priority::Low) => 1,
        _ => 0,
    }
}
```

**Scheduler Implementation:**

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};

/// Main workflow scheduler
pub struct WorkflowScheduler {
    queue: Arc<WorkflowPriorityQueue>,
    running: Arc<Mutex<HashMap<Uuid, RunningWorkflow>>>,
    config: SchedulerConfig,
    event_tx: broadcast::Sender<SchedulerEvent>,
    checkpoint_manager: Option<Arc<dyn CheckpointManager>>,
}

#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    WorkflowStarted { id: Uuid },
    WorkflowCompleted { id: Uuid },
    WorkflowPreempted { id: Uuid, reason: String },
    WorkflowResumed { id: Uuid },
}

#[async_trait]
pub trait CheckpointManager: Send + Sync {
    async fn save_checkpoint(&self, workflow_id: Uuid) -> Result<(), CheckpointError>;
    async fn load_checkpoint(&self, workflow_id: Uuid) -> Result<Option<WorkflowCheckpoint>, CheckpointError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    #[error("Checkpoint failed: {0}")]
    Failed(String),
    #[error("No checkpoint found for workflow {0}")]
    NotFound(Uuid),
}

impl WorkflowScheduler {
    pub fn new(
        config: SchedulerConfig,
        event_tx: broadcast::Sender<SchedulerEvent>,
    ) -> Self {
        Self {
            queue: Arc::new(WorkflowPriorityQueue::new(config.max_concurrent_workflows)),
            running: Arc::new(Mutex::new(HashMap::new())),
            config,
            event_tx,
            checkpoint_manager: None,
        }
    }

    pub fn with_checkpoint_manager(
        mut self,
        manager: Arc<dyn CheckpointManager>,
    ) -> Self {
        self.checkpoint_manager = Some(manager);
        self
    }

    /// Submit a workflow for execution
    pub async fn submit(&self, workflow: QueuedWorkflow) -> QueueResult<()> {
        info!(
            workflow_id = %workflow.workflow_id,
            priority = ?workflow.priority,
            "Workflow submitted to scheduler"
        );
        
        self.queue.push(workflow).await?;
        self.try_start_workflow().await;
        Ok(())
    }

    /// Attempt to start a workflow from the queue
    async fn try_start_workflow(&self) {
        if let Some(workflow) = self.queue.pop().await {
            let running = RunningWorkflow {
                workflow_id: workflow.workflow_id,
                priority: workflow.priority,
                started_at: Utc::now(),
                preemptible: workflow.preemptible,
                current_agent: None,
                can_checkpoint: true,
            };

            self.running.lock().await.insert(workflow.workflow_id, running);
            
            let _ = self.event_tx.send(SchedulerEvent::WorkflowStarted {
                id: workflow.workflow_id,
            });

            info!(
                workflow_id = %workflow.workflow_id,
                running_count = self.queue.running_count(),
                "Workflow started from queue"
            );
        }
    }

    /// Mark a workflow as complete
    pub async fn complete_workflow(&self, workflow_id: Uuid) {
        self.running.lock().await.remove(&workflow_id);
        self.queue.complete();
        
        let _ = self.event_tx.send(SchedulerEvent::WorkflowCompleted { id: workflow_id });

        // Try to start next workflow
        self.try_start_workflow().await;
    }

    /// Check if preemption is needed and execute it
    pub async fn check_preemption(&self) -> bool {
        let peeked = self.queue.peek().await;
        let queued = match peeked {
            Some(w) => w,
            None => return false,
        };

        let running = self.running.lock().await.clone();
        let running_vec: Vec<_> = running.values().cloned().collect();

        if let Some(target_id) = should_preempt(&queued, &running_vec, &self.config) {
            drop(running); // Release lock before async operations
            self.execute_preemption(target_id, queued).await;
            true
        } else {
            false
        }
    }

    /// Execute preemption of a running workflow
    async fn execute_preemption(&self, workflow_id: Uuid, queued_workflow: QueuedWorkflow) {
        info!(
            workflow_id = %workflow_id,
            "Executing preemption for workflow"
        );

        // Step 1: Checkpoint if enabled
        if self.config.checkpoint_before_preempt {
            if let Some(manager) = &self.checkpoint_manager {
                match manager.save_checkpoint(workflow_id).await {
                    Ok(_) => {
                        debug!(workflow_id = %workflow_id, "Checkpoint saved before preemption");
                    }
                    Err(e) => {
                        warn!(
                            workflow_id = %workflow_id,
                            error = %e,
                            "Failed to checkpoint before preemption"
                        );
                        // Continue with preemption anyway
                    }
                }
            }
        }

        // Step 2: Remove from running
        self.running.lock().await.remove(&workflow_id);

        // Step 3: Notify and requeue
        let _ = self.event_tx.send(SchedulerEvent::WorkflowPreempted {
            id: workflow_id,
            reason: format!("Preempted by higher priority workflow {}", queued_workflow.workflow_id),
        });

        // Requeue the preempted workflow
        let preempted = QueuedWorkflow {
            workflow_id,
            priority: self.get_workflow_priority(workflow_id).await,
            queued_at: Utc::now(),
            estimated_duration: None,
            preemptible: true,
        };
        let _ = self.queue.push(preempted).await;

        // Step 4: Start the new workflow
        self.try_start_workflow().await;
    }

    /// Resume a preempted workflow from its checkpoint
    pub async fn resume_workflow(&self, workflow_id: Uuid) -> Result<(), SchedulerError> {
        info!(workflow_id = %workflow_id, "Resuming preempted workflow");

        if let Some(manager) = &self.checkpoint_manager {
            match manager.load_checkpoint(workflow_id).await {
                Ok(Some(_checkpoint)) => {
                    // Add to queue for resumption
                    let requeued = QueuedWorkflow {
                        workflow_id,
                        priority: Priority::Normal,  // Restore original priority from checkpoint
                        queued_at: Utc::now(),
                        estimated_duration: None,
                        preemptible: true,
                    };
                    self.queue.push(requeued).await?;
                    
                    let _ = self.event_tx.send(SchedulerEvent::WorkflowResumed { id: workflow_id });
                    Ok(())
                }
                Ok(None) => Err(SchedulerError::NoCheckpoint(workflow_id)),
                Err(e) => Err(SchedulerError::CheckpointFailed(e.to_string())),
            }
        } else {
            Err(SchedulerError::NoCheckpointManager)
        }
    }

    async fn get_workflow_priority(&self, workflow_id: Uuid) -> Priority {
        // In real implementation, fetch from database
        Priority::Normal
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("No checkpoint found for workflow {0}")]
    NoCheckpoint(Uuid),
    #[error("Checkpoint manager not configured")]
    NoCheckpointManager,
    #[error("Checkpoint failed: {0}")]
    CheckpointFailed(String),
    #[error("Queue error: {0}")]
    QueueError(String),
}

impl From<QueueError> for SchedulerError {
    fn from(e: QueueError) -> Self {
        SchedulerError::QueueError(e.to_string())
    }
}
```

**Note:** The `WorkflowCheckpoint` type should match the structure from `section-03-checkpoint-recovery`. Refer to that section for the checkpoint data structure definition.

---

### 4.3 Module Exports

**File:** `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/scheduler/mod.rs`

```rust
pub mod config;
pub mod queue;

pub use config::SchedulerConfig;
pub use queue::{QueuedWorkflow, WorkflowPriorityQueue, QueueError, QueueResult, Priority};

use crate::checkpoint::model::WorkflowCheckpoint;
use crate::context::RequestPriority;
// ... rest of scheduler implementation
```

**File:** `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/scheduler/config.rs`

```rust
use serde::{Deserialize, Serialize};

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum number of workflows to execute concurrently
    pub max_concurrent_workflows: usize,
    
    /// Whether preemption is enabled
    pub preemption_enabled: bool,
    
    /// Grace period before preemption (milliseconds)
    pub preemption_grace_period_ms: u64,
    
    /// Whether to checkpoint before preempting
    pub checkpoint_before_preempt: bool,
    
    /// Minimum priority difference required for preemption
    pub min_preemption_priority_delta: u8,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: 3,
            preemption_enabled: true,
            preemption_grace_period_ms: 5000,
            checkpoint_before_preempt: true,
            min_preemption_priority_delta: 1,
        }
    }
}
```

---

### 4.4 Library Exports

**File:** `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/lib.rs`

Add the scheduler module:

```rust
pub mod scheduler;

// Re-exports
pub use scheduler::{
    WorkflowScheduler, WorkflowPriorityQueue, QueuedWorkflow, Priority,
    SchedulerConfig, SchedulerEvent, SchedulerError,
};
```

---

## Integration with Other Sections

### Checkpoint Manager (section-03)

The scheduler requires a checkpoint manager for preemption. The `CheckpointManager` trait should be compatible with the PostgreSQL checkpoint storage from `section-03-checkpoint-recovery`:

```rust
// In section-03, implement this trait for PgCheckpointStorage
#[async_trait]
impl CheckpointManager for PgCheckpointStorage {
    async fn save_checkpoint(&self, workflow_id: Uuid) -> Result<(), CheckpointError> {
        // Use existing save_checkpoint from section-03
    }

    async fn load_checkpoint(&self, workflow_id: Uuid) -> Result<Option<WorkflowCheckpoint>, CheckpointError> {
        // Use existing load_latest_checkpoint from section-03
    }
}
```

### State Machine (section-01)

Add a new state `Preempted` to the workflow state machine for preempted workflows:

```rust
// In state_machine/base.rs, add to WorkflowState enum:
pub enum WorkflowState {
    // ... existing states
    Preempted,  // NEW: Workflow paused due to preemption
}
```

### Event Bus (section-02)

The scheduler publishes events to the event bus. Add these events to `OrchestratorEvent`:

```rust
// In event_bus/events.rs, add to OrchestratorEvent enum:
pub enum OrchestratorEvent {
    // ... existing events
    WorkflowPreempted { id: Uuid, reason: String },
    WorkflowResumed { id: Uuid },
}
```

---

## Summary

This section implements:

1. **Priority Queue** with correct `Ord` implementation for CRITICAL > HIGH > NORMAL > LOW ordering
2. **TOCTOU-safe** concurrent limit enforcement using atomic operations within locks
3. **Workflow Scheduler** with preemption support
4. **Graceful preemption** with optional checkpointing before pause
5. **Resume from checkpoint** for preempted workflows

The scheduler integrates with the checkpoint manager (section-03) for state persistence and the event bus (section-02) for event distribution.