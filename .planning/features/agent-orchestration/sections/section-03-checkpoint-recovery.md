# Section 03: Checkpoint & Recovery (PostgreSQL)

## Overview

This section implements checkpoint and recovery functionality for workflow state persistence. The system uses PostgreSQL for operational data storage, providing ACID compliance for reliable state transitions and crash recovery.

**Dependencies:**
- `section-02-parallel-executor` - Uses `WorkflowContext` and `Arc<RwLock<T>>` pattern

**Files to Create:**
- `/server/crates/iou-orchestrator/src/checkpoint/mod.rs`
- `/server/crates/iou-orchestrator/src/checkpoint/policy.rs`
- `/server/crates/iou-orchestrator/src/checkpoint/model.rs`
- `/server/crates/iou-orchestrator/src/checkpoint/storage.rs`
- `/server/crates/iou-orchestrator/src/checkpoint/recovery.rs`

---

## Tests (TDD)

### 2.1 Adaptive Checkpoint Policy

**Test: CheckpointPolicy returns true for agent_interval when threshold reached**
- Create policy with `agent_interval=Some(2)`
- Call `should_checkpoint` after 2 agents complete
- Assert returns `true`

**Test: CheckpointPolicy returns true at approval points**
- Create policy with `at_approvals=true`
- Call `should_checkpoint` when entering `AwaitingApproval`
- Assert returns `true`

**Test: CheckpointPolicy returns false before threshold**
- Create policy with `agent_interval=Some(5)`
- Call `should_checkpoint` after 1 agent completes
- Assert returns `false`

**Test: CheckpointPolicy respects time_interval**
- Create policy with `time_interval=60000ms`
- Call `should_checkpoint` after 30 seconds
- Assert returns `false`
- Call after 61 seconds
- Assert returns `true`

### 2.2 Checkpoint Data Structures

**Test: WorkflowCheckpoint serializes to JSON correctly**
- Create `WorkflowCheckpoint` with all fields populated
- Serialize to JSON
- Assert all fields present and correctly typed

**Test: WorkflowCheckpoint deserializes from JSON**
- Create JSON checkpoint string
- Deserialize to `WorkflowCheckpoint`
- Assert all fields match expected values

**Test: Checkpoint captures minimal state for recovery**
- Create checkpoint after 2 agents complete
- Assert `completed_agents` has 2 entries
- Assert `pending_agents` has remaining agents
- Assert `context_snapshot` contains agent outputs

### 2.3 PostgreSQL Checkpoint Storage

**Test: save_checkpoint inserts new row**
- Create `PgCheckpointStorage` with test pool
- Call `save_checkpoint` with new checkpoint
- Query `workflow_checkpoints` table
- Assert row exists with matching `workflow_id`

**Test: save_checkpoint upserts existing workflow checkpoint**
- Save checkpoint for workflow W
- Save updated checkpoint for same workflow W
- Query `workflow_checkpoints` table for W
- Assert only 1 row exists with updated data

**Test: load_latest_checkpoint returns most recent by timestamp**
- Save 3 checkpoints for same workflow with different timestamps
- Call `load_latest_checkpoint`
- Assert returns checkpoint with latest timestamp

**Test: load_latest_checkpoint returns None for non-existent workflow**
- Call `load_latest_checkpoint` with random UUID
- Assert returns `None`

**Test: list_checkpoints returns all checkpoints ordered by timestamp**
- Save 3 checkpoints for workflow
- Call `list_checkpoints`
- Assert returns 3 checkpoints in timestamp DESC order

**Test: delete_old_checkpoints keeps only N most recent**
- Save 5 checkpoints for workflow
- Call `delete_old_checkpoints` with `keep_last=2`
- Query `workflow_checkpoints` table
- Assert only 2 most recent rows remain

**Test: PostgreSQL transaction rollback on save failure**
- Mock connection that fails on insert
- Call `save_checkpoint`
- Assert transaction rolled back
- Assert no partial data written

### 2.4 Recovery Workflow

**Test: recover_workflow restores state from checkpoint**
- Save checkpoint with `state=ParallelExecuting`
- Call `recover_workflow`
- Assert `RecoveryPlan` has `restored_state=ParallelExecuting`
- Assert `pending_agents` matches checkpoint

**Test: recover_workflow returns NotFound for non-existent workflow**
- Call `recover_workflow` with random UUID
- Assert returns `RecoveryError::NotFound`

**Test: recover_workflow applies migration for old version**
- Save checkpoint with `version=1`
- Set `CURRENT_CHECKPOINT_VERSION=2`
- Call `recover_workflow`
- Assert checkpoint migrated to version 2
- Assert `RecoveryPlan.was_migrated=true`

**Test: recover_workflow returns IncompatibleVersion for unsupported version**
- Save checkpoint with `version=99`
- Call `recover_workflow`
- Assert returns `RecoveryError::IncompatibleVersion(99)`

**Test: recover_workflow rebuilds context from snapshot**
- Create checkpoint with complex `context_snapshot`
- Call `recover_workflow`
- Assert context restored with all agent outputs
- Assert `pending_agents` correctly identified

**Test: migrate_checkpoint adds new field with default**
- Create v1 checkpoint without `new_field`
- Call `migrate_checkpoint` from 1 to 2
- Assert returned checkpoint has `new_field` with default value
- Assert version incremented to 2

---

## Implementation

### 1. Checkpoint Policy

**File:** `server/crates/iou-orchestrator/src/checkpoint/policy.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// When checkpoints should be saved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointPolicy {
    /// Checkpoint after every N agents
    pub agent_interval: Option<usize>,

    /// Checkpoint at approval points
    pub at_approvals: bool,

    /// Checkpoint after duration (ms)
    pub time_interval: Option<u64>,

    /// Minimum progress percentage for checkpoint
    pub min_progress_pct: Option<u8>,
}

impl Default for CheckpointPolicy {
    fn default() -> Self {
        Self {
            agent_interval: Some(1),
            at_approvals: true,
            time_interval: Some(60000),
            min_progress_pct: Some(10),
        }
    }
}

/// Workflow-specific policy lookup
pub fn get_policy_for_workflow(workflow_type: WorkflowType) -> CheckpointPolicy {
    match workflow_type {
        WorkflowType::DocumentCreation => CheckpointPolicy {
            agent_interval: Some(1),
            at_approvals: true,
            time_interval: Some(60000),
            min_progress_pct: Some(10),
        },
    }
}

/// Decision function for when to checkpoint
pub fn should_checkpoint(
    policy: &CheckpointPolicy,
    context: &WorkflowContext,
    last_checkpoint: DateTime<Utc>,
) -> bool {
    // Agent interval check
    if let Some(interval) = policy.agent_interval {
        if context.completed_agents_count() >= interval {
            return true;
        }
    }

    // Approval point check
    if policy.at_approvals && context.is_awaiting_approval() {
        return true;
    }

    // Time interval check
    if let Some(interval_ms) = policy.time_interval {
        let elapsed = Utc::now().signed_duration_since(last_checkpoint);
        if elapsed.num_milliseconds() >= interval_ms as i64 {
            return true;
        }
    }

    // Progress threshold check
    if let Some(min_pct) = policy.min_progress_pct {
        if context.progress_pct() >= min_pct {
            return true;
        }
    }

    false
}
```

### 2. Checkpoint Data Model

**File:** `server/crates/iou-orchestrator/src/checkpoint/model.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Current checkpoint schema version
pub const CURRENT_CHECKPOINT_VERSION: u32 = 1;

/// Workflow checkpoint for crash recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub version: u32,
}

/// Recovery plan output
#[derive(Debug, Clone)]
pub struct RecoveryPlan {
    pub workflow_id: Uuid,
    pub restored_state: WorkflowState,
    pub pending_agents: Vec<AgentType>,
    pub restored_results: HashMap<AgentType, AgentResult>,
    pub next_dag_layer: usize,
    pub was_migrated: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error("Checkpoint not found for workflow {0}")]
    NotFound(Uuid),

    #[error("Checkpoint version {0} incompatible with current version {CURRENT_CHECKPOINT_VERSION}")]
    IncompatibleVersion(u32),

    #[error("Migration from version {0} to {1} failed: {2}")]
    MigrationFailed(u32, u32, String),
}
```

### 3. PostgreSQL Storage

**File:** `server/crates/iou-orchestrator/src/checkpoint/storage.rs`

```rust
use sqlx::PgPool;
use async_trait::async_trait;

#[async_trait]
pub trait CheckpointStorage: Send + Sync {
    async fn save_checkpoint(&self, checkpoint: &WorkflowCheckpoint) -> Result<(), StorageError>;
    async fn load_latest_checkpoint(&self, workflow_id: Uuid) -> Result<Option<WorkflowCheckpoint>, StorageError>;
    async fn list_checkpoints(&self, workflow_id: Uuid) -> Result<Vec<WorkflowCheckpoint>, StorageError>;
    async fn delete_old_checkpoints(&self, workflow_id: Uuid, keep_last: usize) -> Result<(), StorageError>;
}

pub struct PgCheckpointStorage {
    pool: PgPool,
}

impl PgCheckpointStorage {
    pub async fn new(pool: PgPool) -> Result<Self, StorageError> {
        sqlx::query("
            CREATE TABLE IF NOT EXISTS workflow_checkpoints (
                id UUID PRIMARY KEY,
                workflow_id UUID NOT NULL,
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
        ")
        .execute(&pool)
        .await?;
        Ok(Self { pool })
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
        .bind(&checkpoint.completed_agents.iter().map(|a| a.to_string()).collect::<Vec<_>>())
        .bind(&checkpoint.pending_agents.iter().map(|a| a.to_string()).collect::<Vec<_>>())
        .bind(serde_json::to_value(&checkpoint.agent_results)?)
        .bind(checkpoint.current_dag_layer.map(|v| v as i32))
        .bind(serde_json::to_value(&checkpoint.context_snapshot)?)
        .bind(CURRENT_CHECKPOINT_VERSION)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load_latest_checkpoint(&self, workflow_id: Uuid) -> Result<Option<WorkflowCheckpoint>, StorageError> {
        sqlx::query_as!(
            "SELECT * FROM workflow_checkpoints
             WHERE workflow_id = $1
             ORDER BY timestamp DESC
             LIMIT 1",
            WorkflowCheckpointRow
        )
        .bind(workflow_id)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| row.to_checkpoint())
        .transpose()
    }
}
```

### 4. Recovery with Migration

**File:** `server/crates/iou-orchestrator/src/checkpoint/recovery.rs`

```rust
pub async fn recover_workflow(
    workflow_id: Uuid,
    storage: &dyn CheckpointStorage,
    state_machine: &mut WorkflowStateMachine,
) -> Result<RecoveryPlan, RecoveryError> {
    // Load checkpoint
    let mut checkpoint = storage.load_latest_checkpoint(workflow_id)
        .await?
        .ok_or(RecoveryError::NotFound(workflow_id))?;

    // Check version compatibility
    if checkpoint.version != CURRENT_CHECKPOINT_VERSION {
        checkpoint = migrate_checkpoint(checkpoint, CURRENT_CHECKPOINT_VERSION)?;
    }

    // Restore state machine
    state_machine.restore_state(checkpoint.state, checkpoint.pending_agents.clone());

    // Rebuild context
    // ...

    Ok(RecoveryPlan {
        workflow_id,
        restored_state: checkpoint.state,
        pending_agents: checkpoint.pending_agents,
        restored_results: checkpoint.agent_results,
        next_dag_layer: checkpoint.current_dag_layer.map(|v| v + 1).unwrap_or(0),
        was_migrated: checkpoint.version != CURRENT_CHECKPOINT_VERSION,
    })
}

fn migrate_checkpoint(mut checkpoint: WorkflowCheckpoint, target_version: u32) -> Result<WorkflowCheckpoint, RecoveryError> {
    match (checkpoint.version, target_version) {
        (1, 2) => {
            // Add new field with default
            checkpoint.version = 2;
            Ok(checkpoint)
        }
        (from, to) => Err(RecoveryError::MigrationFailed(from, to, "No migration path".to_string())),
    }
}
```
