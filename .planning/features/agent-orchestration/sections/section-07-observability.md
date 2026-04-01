Now I have all the context I need. Let me generate the section content for section-07-observability.

# Section 07: Observability

## Overview

This section implements comprehensive observability for the agent orchestration system including structured logging, Prometheus metrics, and compliance audit logging. These capabilities enable monitoring, debugging, and regulatory compliance for workflow execution.

## Dependencies

This section depends on:
- **section-02-parallel-executor** - Agent execution events and lifecycle hooks
- **section-03-checkpoint-recovery** - Checkpoint events for recovery tracking

## File Locations

- `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/metrics.rs` - Prometheus metrics definitions
- `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/audit.rs` - Audit logging structures and storage
- `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/observability/` - New module for observability coordination

## Tests

### 5.1 Structured Logging Tests

**Test: State transition logged with workflow_id and state**
- Trigger state transition
- Assert log entry contains workflow_id
- Assert log entry contains old_state and new_state

**Test: Agent completion logged with duration**
- Complete agent execution
- Assert log entry contains agent_type
- Assert log entry contains duration_ms

**Test: Error logs include stack trace context**
- Trigger error during agent execution
- Assert log entry at ERROR level
- Assert log entry includes error message
- Assert log entry includes workflow_id for correlation

### 5.2 Metrics Tests

**Test: Counter increments on workflow completion**
- Complete 3 workflows
- Query workflow_completed_total metric
- Assert value = 3

**Test: Histogram records workflow duration**
- Complete workflow with 500ms duration
- Query workflow_duration_seconds histogram
- Assert bucket for 0.5s incremented

**Test: Gauge updates for concurrent workflows**
- Start 2 workflows
- Query workflows_active gauge
- Assert value = 2
- Complete 1 workflow
- Assert value = 1

**Test: Priority queue depth gauge updates**
- Add 5 workflows to queue
- Assert queue_depth gauge = 5
- Process 2 workflows
- Assert queue_depth gauge = 3

### 5.3 Audit Logging Tests

**Test: Audit entry written for approval decision**
- Make approval decision via GraphQL
- Query audit_log table
- Assert entry exists with event_type="approval_decision"
- Assert entry contains approver_id and decision

**Test: Audit entries are immutable**
- Insert audit entry
- Attempt to UPDATE audit entry
- Assert query fails (append-only constraint)

**Test: Audit log includes all approval modifications**
- Approve with modifications to agent output
- Query audit_log for approval
- Assert audit entry includes full modification diff
- Assert modification fields recorded

**Test: Critical events written via bounded channel**
- Publish 20 critical events rapidly (channel capacity=10)
- Assert first 10 written to audit
- Assert next 10 handled via backpressure (retried/queued)

## Implementation

### 5.1 Structured Logging

The project already uses the `tracing` crate for structured logging. This section adds instrumentation points throughout the orchestrator.

**Log Instrumentation Points:**

1. **Workflow State Transitions** - Log every state change with workflow_id, old_state, new_state
2. **Agent Lifecycle** - Log agent start, completion, and failures with timing
3. **Checkpoint Operations** - Log checkpoint save/load with duration
4. **Approval Events** - Log approval requests and decisions
5. **Error Context** - Log errors with workflow_id for correlation

**Logging Pattern:**

```rust
// State transition logging
tracing::info!(
    workflow_id = %workflow_id,
    old_state = %old_state,
    new_state = %new_state,
    "Workflow state transition"
);

// Agent completion logging
tracing::info!(
    workflow_id = %workflow_id,
    agent = ?agent_type,
    duration_ms = execution_time.as_millis(),
    result_count = result.len(),
    "Agent completed successfully"
);

// Error logging with context
tracing::error!(
    workflow_id = %workflow_id,
    agent = ?agent_type,
    error = %error,
    "Agent execution failed"
);
```

### 5.2 Prometheus Metrics

**File:** `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/metrics.rs`

Create a metrics module that defines and exposes Prometheus metrics using the `prometheus` crate.

**Metric Definitions:**

```rust
use prometheus::{Counter, Gauge, Histogram, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Registry, default_registry};
use lazy_static::lazy_static;

// Workflow lifecycle metrics
lazy_static! {
    // Counters
    static ref WORKFLOW_CREATED_TOTAL: IntCounter = IntCounter::new(
        "workflow_created_total",
        "Total number of workflows created"
    ).unwrap();
    
    static ref WORKFLOW_STARTED_TOTAL: IntCounter = IntCounter::new(
        "workflow_started_total",
        "Total number of workflows started"
    ).unwrap();
    
    static ref WORKFLOW_COMPLETED_TOTAL: IntCounter = IntCounter::new(
        "workflow_completed_total",
        "Total number of workflows completed successfully"
    ).unwrap();
    
    static ref WORKFLOW_FAILED_TOTAL: IntCounterVec = IntCounterVec::new(
        "workflow_failed_total",
        "Total number of workflows failed",
        &["reason"]
    ).unwrap();
    
    // Gauges
    static ref WORKFLOWS_ACTIVE: IntGauge = IntGauge::new(
        "workflows_active",
        "Number of workflows currently active"
    ).unwrap();
    
    static ref WORKFLOWS_QUEUED: IntGauge = IntGauge::new(
        "workflows_queued",
        "Number of workflows waiting in queue"
    ).unwrap();
    
    static ref PENDING_APPROVALS: IntGauge = IntGauge::new(
        "pending_approvals_current",
        "Current number of pending approvals"
    ).unwrap();
    
    // Histograms
    static ref WORKFLOW_DURATION_SECONDS: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "workflow_duration_seconds",
            "Workflow execution time in seconds"
        )
        .buckets(vec![0.1, 1.0, 10.0, 60.0, 300.0, 1800.0])
    ).unwrap();
    
    // Agent metrics
    static ref AGENT_EXECUTION_TIME_SECONDS: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "agent_execution_time_seconds",
            "Agent execution time in seconds"
        )
        .buckets(vec![0.5, 1.0, 5.0, 15.0, 30.0, 60.0]),
        &["agent_type"]
    ).unwrap();
    
    static ref AGENT_EXECUTIONS_TOTAL: IntCounterVec = IntCounterVec::new(
        "agent_executions_total",
        "Total number of agent executions",
        &["agent_type", "status"]
    ).unwrap();
    
    // Approval metrics
    static ref APPROVAL_RESPONSE_TIME_SECONDS: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "approval_response_time_seconds",
            "Time from approval request to decision in seconds"
        )
        .buckets(vec![60.0, 300.0, 900.0, 3600.0, 86400.0])
    ).unwrap();
    
    static ref APPROVAL_DECISIONS_TOTAL: IntCounterVec = IntCounterVec::new(
        "approval_decisions_total",
        "Total number of approval decisions",
        &["decision", "escalated"]
    ).unwrap();
    
    // Checkpoint metrics
    static ref CHECKPOINT_SAVE_DURATION_SECONDS: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "checkpoint_save_duration_seconds",
            "Time to save checkpoint in seconds"
        )
        .buckets(vec![0.01, 0.1, 0.5, 1.0, 5.0])
    ).unwrap();
    
    static ref CHECKPOINT_RECOVERIES_TOTAL: IntCounter = IntCounter::new(
        "checkpoint_recoveries_total",
        "Total number of workflow recoveries from checkpoint"
    ).unwrap();
}
```

**Metric Recording Helper Functions:**

```rust
/// Record workflow creation
pub fn record_workflow_created(priority: &str) {
    WORKFLOW_CREATED_TOTAL.inc();
    tracing::debug!("Metric recorded: workflow_created");
}

/// Record workflow start
pub fn record_workflow_started() {
    WORKFLOW_STARTED_TOTAL.inc();
    WORKFLOWS_ACTIVE.inc();
}

/// Record workflow completion
pub fn record_workflow_completed(duration_secs: f64) {
    WORKFLOW_COMPLETED_TOTAL.inc();
    WORKFLOWS_ACTIVE.dec();
    WORKFLOW_DURATION_SECONDS.observe(duration_secs);
}

/// Record workflow failure
pub fn record_workflow_failed(reason: &str) {
    WORKFLOW_FAILED_TOTAL.with_label_values(&[reason]).inc();
    WORKFLOWS_ACTIVE.dec();
}

/// Record agent start
pub fn record_agent_started(agent_type: &str) {
    // Track start time in a map for duration calculation
}

/// Record agent completion
pub fn record_agent_completed(agent_type: &str, duration_secs: f64, status: &str) {
    AGENT_EXECUTION_TIME_SECONDS.with_label_values(&[agent_type]).observe(duration_secs);
    AGENT_EXECUTIONS_TOTAL.with_label_values(&[agent_type, status]).inc();
}

/// Record approval request
pub fn record_approval_requested() {
    PENDING_APPROVALS.inc();
}

/// Record approval decision
pub fn record_approval_decision(decision: &str, escalated: bool, duration_secs: f64) {
    let escalated_label = if escalated { "true" } else { "false" };
    APPROVAL_DECISIONS_TOTAL.with_label_values(&[decision, escalated_label]).inc();
    PENDING_APPROVALS.dec();
    APPROVAL_RESPONSE_TIME_SECONDS.observe(duration_secs);
}

/// Record checkpoint save
pub fn record_checkpoint_saved(duration_secs: f64) {
    CHECKPOINT_SAVE_DURATION_SECONDS.observe(duration_secs);
}

/// Record workflow recovery
pub fn record_workflow_recovered() {
    CHECKPOINT_RECOVERIES_TOTAL.inc();
}

/// Record queue depth
pub fn record_queue_depth(depth: i64) {
    WORKFLOWS_QUEUED.set(depth);
}
```

**Metrics Export Endpoint:**

The metrics are exposed via an HTTP endpoint (typically `/metrics`) that Prometheus scrapes. This is integrated into the main server Axum router.

### 5.3 Compliance Audit Logging

**File:** `/Users/marc/Projecten/iou-modern/server/crates/iou-orchestrator/src/audit.rs`

Implement immutable audit logging for compliance requirements.

**Audit Data Structures:**

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub actor: AuditActor,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    WorkflowCreated,
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowCancelled,
    WorkflowFailed,
    WorkflowEscalated,
    WorkflowRecovered,
    AgentStarted,
    AgentCompleted,
    AgentFailed,
    ApprovalRequested,
    ApprovalDecision,
    ApprovalModified,
    CheckpointSaved,
    CheckpointLoaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActor {
    System,
    User { id: Uuid, name: String, email: String },
    Agent { agent_type: String },
}
```

**Audit Storage Trait:**

```rust
#[async_trait::async_trait]
pub trait AuditStorage: Send + Sync {
    async fn write_entry(&self, entry: &AuditEntry) -> Result<(), AuditError>;
    async fn query_workflow_audit(
        &self,
        workflow_id: Uuid,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<AuditEntry>, AuditError>;
    async fn query_by_actor(
        &self,
        actor_id: Uuid,
        from: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditError>;
}
```

**PostgreSQL Audit Storage Implementation:**

```rust
pub struct PgAuditStorage {
    pool: sqlx::PgPool,
    critical_tx: mpsc::Sender<AuditEntry>,
}

impl PgAuditStorage {
    pub async fn new(pool: sqlx::PgPool) -> Result<Self, AuditError> {
        // Create audit_log table with append-only constraint
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id UUID PRIMARY KEY,
                workflow_id UUID NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                event_type TEXT NOT NULL,
                actor_type TEXT NOT NULL,
                actor_id UUID,
                actor_name TEXT,
                actor_email TEXT,
                agent_type TEXT,
                details JSONB NOT NULL,
                CONSTRAINT no_updates CHECK (1 = 0)  -- Prevents UPDATEs
            );
            
            CREATE INDEX IF NOT EXISTS idx_audit_log_workflow_id 
                ON audit_log(workflow_id, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_audit_log_actor 
                ON audit_log(actor_id, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp 
                ON audit_log(timestamp DESC);
            
            -- Grant SELECT only (no UPDATE/DELETE)
            REVOKE UPDATE, DELETE ON audit_log FROM PUBLIC;
            "#
        )
        .execute(&pool)
        .await?;
        
        let (critical_tx, mut critical_rx) = mpsc::channel(1000);
        
        // Spawn audit writer task
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            while let Some(entry) = critical_rx.recv().await {
                if let Err(e) = Self::persist_entry(&pool_clone, &entry).await {
                    tracing::error!("Failed to write audit entry: {}", e);
                }
            }
        });
        
        Ok(Self { pool, critical_tx })
    }
    
    async fn persist_entry(
        pool: &sqlx::PgPool,
        entry: &AuditEntry,
    ) -> Result<(), AuditError> {
        let (actor_type, actor_id, actor_name, actor_email, agent_type) = match &entry.actor {
            AuditActor::System => (
                "system".to_string(),
                None,
                None,
                None,
                None,
            ),
            AuditActor::User { id, name, email } => (
                "user".to_string(),
                Some(*id),
                Some(name.as_str()),
                Some(email.as_str()),
                None,
            ),
            AuditActor::Agent { agent_type: at } => (
                "agent".to_string(),
                None,
                None,
                None,
                Some(at.as_str()),
            ),
        };
        
        sqlx::query(
            r#"
            INSERT INTO audit_log 
                (id, workflow_id, timestamp, event_type, 
                 actor_type, actor_id, actor_name, actor_email, 
                 agent_type, details)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(entry.id)
        .bind(entry.workflow_id)
        .bind(entry.timestamp)
        .bind(&format!("{:?}", entry.event_type).to_lowercase())
        .bind(actor_type)
        .bind(actor_id)
        .bind(actor_name)
        .bind(actor_email)
        .bind(agent_type)
        .bind(&entry.details)
        .execute(pool)
        .await?;
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl AuditStorage for PgAuditStorage {
    async fn write_entry(&self, entry: &AuditEntry) -> Result<(), AuditError> {
        // Use bounded channel with backpressure handling
        self.critical_tx
            .send(entry.clone())
            .await
            .map_err(|_| AuditError::ChannelFull)?;
        Ok(())
    }
    
    async fn query_workflow_audit(
        &self,
        workflow_id: Uuid,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<AuditEntry>, AuditError> {
        let query = sqlx::query_as::<_, AuditRow>(
            "SELECT * FROM audit_log 
             WHERE workflow_id = $1 
             AND ($2::timestamptz IS NULL OR timestamp >= $2)
             AND ($3::timestamptz IS NULL OR timestamp <= $3)
             ORDER BY timestamp ASC"
        )
        .bind(workflow_id)
        .bind(from)
        .bind(to);
        
        // Parse rows into AuditEntry...
        Ok(vec![])
    }
    
    async fn query_by_actor(
        &self,
        actor_id: Uuid,
        from: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditError> {
        // Implementation...
        Ok(vec![])
    }
}
```

**Audit Logger:**

```rust
pub struct AuditLogger {
    storage: Arc<dyn AuditStorage>,
}

impl AuditLogger {
    pub fn new(storage: Arc<dyn AuditStorage>) -> Self {
        Self { storage }
    }
    
    pub async fn log_event(
        &self,
        workflow_id: Uuid,
        event_type: AuditEventType,
        actor: AuditActor,
        details: serde_json::Value,
    ) -> Result<(), AuditError> {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            workflow_id,
            timestamp: Utc::now(),
            event_type,
            actor,
            details,
        };
        
        self.storage.write_entry(&entry).await
    }
}
```

### Integration with Event Bus

The audit logger subscribes to the event bus's critical channel to ensure all important events are captured.

```rust
pub async fn spawn_audit_subscriber(
    event_bus: Arc<EventBus>,
    audit_logger: Arc<AuditLogger>,
) {
    tokio::spawn(async move {
        let mut critical_rx = event_bus.subscribe_critical();
        
        while let Ok(event) = critical_rx.recv().await {
            match event {
                OrchestratorEvent::WorkflowCreated { id } => {
                    let _ = audit_logger.log_event(
                        id,
                        AuditEventType::WorkflowCreated,
                        AuditActor::System,
                        serde_json::json!({}),
                    ).await;
                }
                OrchestratorEvent::ApprovalDecision { workflow_id, decision } => {
                    let details = serde_json::to_value(decision).unwrap_or_default();
                    // Extract actor from decision...
                    let _ = audit_logger.log_event(
                        workflow_id,
                        AuditEventType::ApprovalDecision,
                        // Actor from auth context
                        AuditActor::System,
                        details,
                    ).await;
                }
                // Handle other events...
                _ => {}
            }
        }
    });
}
```

## Success Criteria

- [ ] All workflow state transitions are logged with workflow_id
- [ ] Agent execution includes duration_ms in logs
- [ ] Error logs include stack trace and workflow_id for correlation
- [ ] workflow_completed_total counter increments on workflow completion
- [ ] workflow_duration_seconds histogram records execution time
- [ ] workflows_active gauge updates correctly during execution
- [ ] queue_depth gauge reflects current queue size
- [ ] Audit entries written for all approval decisions
- [ ] Audit log table is append-only (UPDATEs blocked)
- [ ] Approval modifications captured with full diff
- [ ] Critical events handled via bounded channel with backpressure