# Agent Orchestration - TDD Plan

This document defines tests to write BEFORE implementing each section of the implementation plan. Tests follow the project's existing patterns: `#[tokio::test]`, loom for concurrency, and mocks in `tests/mocks/`.

---

## Phase 1: Core Orchestration

### 1.1 Enhance State Machine for DAG Execution

**Test: State transition from Created to ParallelExecuting after DAG built**
- Create state machine in Created state
- Send DagBuilt event with valid DAG
- Assert state is ParallelExecuting
- Assert DAG layers are stored in state machine

**Test: State transition from ParallelExecuting to PartialComplete on agent completion**
- Start in ParallelExecuting with 2-agent layer
- Send PartialAgentComplete event for one agent
- Assert state is PartialComplete
- Assert completed agent tracked in DagLayer

**Test: State transition from PartialComplete to MergeResults when layer complete**
- Start in PartialComplete with all agents complete
- Send DagLayerComplete event
- Assert state is MergeResults
- Assert all layer results available

**Test: Agent failure during ParallelExecuting cancels all agents and fails workflow**
- Start in ParallelExecuting with 2-agent layer
- Send AgentFailed event for one agent
- Assert state is Failed
- Assert cancellation signal was sent

**Test: Guard can_execute_parallel returns true when no dependencies pending**
- Set up state with all dependencies satisfied
- Call can_execute_parallel guard
- Assert returns true

**Test: Guard can_execute_parallel returns false when dependencies pending**
- Set up state with incomplete dependencies
- Call can_execute_parallel guard
- Assert returns false

**Test: loom test for concurrent state transitions**
- Use loom to simulate multiple threads sending events simultaneously
- Assert state machine remains consistent
- Assert no lost events

### 1.2 Build DAG from Agent Dependencies

**Test: build_execution_dag creates valid layers from sequential dependencies**
- Provide sequential agent dependencies (A → B → C)
- Call build_execution_dag
- Assert 3 layers created with one agent each
- Assert layer 0 has A, layer 1 has B, layer 2 has C

**Test: build_execution_dag groups independent agents into same layer**
- Provide dependencies where B and C both depend on A
- Call build_execution_dag
- Assert layer 0 has A, layer 1 has both B and C

**Test: build_execution_dag returns error for circular dependencies**
- Provide circular dependency (A → B → A)
- Call build_execution_dag
- Assert returns DagError::CircularDependency

**Test: build_execution_dag handles complex dependency graph**
- Provide complex multi-branch dependency tree
- Call build_execution_dag
- Assert all agents assigned to correct layers
- Assert no layer has agents with unmet dependencies

**Test: build_execution_dag handles empty agent list**
- Provide empty dependency list
- Call build_execution_dag
- Assert returns empty DAG with no layers

### 1.3 Parallel Agent Executor

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

**Test: CancellationToken cancels running agents**
- Start execute_layer_parallel with 2 agents
- Send cancellation signal
- Assert both agents receive cancellation
- Assert execution returns early with Cancelled error

**Test: RwLock context allows concurrent reads**
- Spawn multiple tasks reading from WorkflowContext
- Assert no deadlocks occur
- Assert all tasks read consistent state

**Test: Agent trait implementation for Research agent**
- Create ResearchAgent instance
- Call execute() with mock input
- Assert returns AgentOutput with expected structure
- Assert agent_type() returns AgentType::Research

**Test: loom test for concurrent result writes**
- Use loom to simulate multiple agents writing results simultaneously
- Assert all results written correctly
- Assert no data races

### 1.4 Event Bus

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

---

## Phase 2: Checkpoint & Recovery (PostgreSQL)

### 2.1 Adaptive Checkpoint Policy

**Test: CheckpointPolicy returns true for agent_interval when threshold reached**
- Create policy with agent_interval=Some(2)
- Call should_checkpoint after 2 agents complete
- Assert returns true

**Test: CheckpointPolicy returns true at approval points**
- Create policy with at_approvals=true
- Call should_checkpoint when entering AwaitingApproval
- Assert returns true

**Test: CheckpointPolicy returns false before threshold**
- Create policy with agent_interval=Some(5)
- Call should_checkpoint after 1 agent completes
- Assert returns false

**Test: CheckpointPolicy respects time_interval**
- Create policy with time_interval=60000ms
- Call should_checkpoint after 30 seconds
- Assert returns false
- Call after 61 seconds
- Assert returns true

### 2.2 Checkpoint Data Structure

**Test: WorkflowCheckpoint serializes to JSON correctly**
- Create WorkflowCheckpoint with all fields populated
- Serialize to JSON
- Assert all fields present and correctly typed

**Test: WorkflowCheckpoint deserializes from JSON**
- Create JSON checkpoint string
- Deserialize to WorkflowCheckpoint
- Assert all fields match expected values

**Test: Checkpoint captures minimal state for recovery**
- Create checkpoint after 2 agents complete
- Assert completed_agents has 2 entries
- Assert pending_agents has remaining agents
- Assert context_snapshot contains agent outputs

### 2.3 PostgreSQL Checkpoint Storage

**Test: save_checkpoint inserts new row**
- Create PgCheckpointStorage with test pool
- Call save_checkpoint with new checkpoint
- Query checkpoints table
- Assert row exists with matching workflow_id

**Test: save_checkpoint upserts existing workflow checkpoint**
- Save checkpoint for workflow W
- Save updated checkpoint for same workflow W
- Query checkpoints table for W
- Assert only 1 row exists with updated data

**Test: load_latest_checkpoint returns most recent by timestamp**
- Save 3 checkpoints for same workflow with different timestamps
- Call load_latest_checkpoint
- Assert returns checkpoint with latest timestamp

**Test: load_latest_checkpoint returns None for non-existent workflow**
- Call load_latest_checkpoint with random UUID
- Assert returns None

**Test: list_checkpoints returns all checkpoints ordered by timestamp**
- Save 3 checkpoints for workflow
- Call list_checkpoints
- Assert returns 3 checkpoints in timestamp DESC order

**Test: delete_old_checkpoints keeps only N most recent**
- Save 5 checkpoints for workflow
- Call delete_old_checkpoints with keep_last=2
- Query checkpoints table
- Assert only 2 most recent rows remain

**Test: PostgreSQL transaction rollback on save failure**
- Mock connection that fails on insert
- Call save_checkpoint
- Assert transaction rolled back
- Assert no partial data written

### 2.4 Recovery Workflow

**Test: recover_workflow restores state from checkpoint**
- Save checkpoint with state=ParallelExecuting
- Call recover_workflow
- Assert RecoveryPlan has restored_state=ParallelExecuting
- Assert pending_agents matches checkpoint

**Test: recover_workflow returns NotFound for non-existent workflow**
- Call recover_workflow with random UUID
- Assert returns RecoveryError::NotFound

**Test: recover_workflow applies migration for old version**
- Save checkpoint with version=1
- Set CURRENT_CHECKPOINT_VERSION=2
- Call recover_workflow
- Assert checkpoint migrated to version 2
- Assert RecoveryPlan.was_migrated=true

**Test: recover_workflow returns IncompatibleVersion for unsupported version**
- Save checkpoint with version=99
- Call recover_workflow
- Assert returns RecoveryError::IncompatibleVersion(99)

**Test: recover_workflow rebuilds context from snapshot**
- Create checkpoint with complex context_snapshot
- Call recover_workflow
- Assert context restored with all agent outputs
- Assert pending_agents correctly identified

**Test: migrate_checkpoint adds new field with default**
- Create v1 checkpoint without new_field
- Call migrate_checkpoint from 1 to 2
- Assert returned checkpoint has new_field with default value
- Assert version incremented to 2

---

## Phase 3: Human-in-the-Loop

### 3.1 GraphQL API Schema

**Test: createWorkflow mutation creates workflow in database**
- Send GraphQL createWorkflow mutation with valid input
- Query workflows table
- Assert row created with matching document_type and priority

**Test: approve mutation fails without authentication**
- Send GraphQL approve mutation without auth header
- Assert returns UNAUTHORIZED error
- Assert approval not applied in database

**Test: approve mutation gets approverId from JWT context**
- Send GraphQL approve mutation with valid JWT
- Assert approval record has approver_id matching JWT subject
- Assert workflow advanced to next state

**Test: workflow query returns pending approvals for user's role**
- Create workflow with pending approval
- Send workflow query with user JWT
- Assert response includes approval requests user can approve
- Assert excludes requests for other roles

**Test: approve mutation validates user can approve this workflow**
- Create approval request requiring "supervisor" role
- Send approve mutation with JWT for "user" role
- Assert returns FORBIDDEN error
- Assert approval not applied

**Test: modify input does not include approverId (from auth context)**
- Send GraphQL modify mutation
- Assert approverId extracted from JWT, not input
- Assert modification attributed to authenticated user

**Test: Subscription sends workflow updates to subscribed clients**
- Create GraphQL subscription for workflow ID
- Trigger workflow state change
- Assert subscription receives update event
- Assert event contains new state

### 3.2 WebSocket Server

**Test: WebSocket handler rejects unauthenticated connections**
- Attempt WebSocket upgrade without auth header
- Assert connection closed with 401
- Assert no socket created

**Test: WebSocket handler accepts valid JWT**
- Send WebSocket upgrade with valid Bearer token
- Assert connection upgraded
- Assert socket receives initial message

**Test: WebSocket handler validates JWT before upgrade**
- Send WebSocket upgrade with expired JWT
- Assert connection rejected
- Assert error message about expired credentials

**Test: WebSocket subscription receives workflow events**
- Connect authenticated WebSocket
- Subscribe to workflow ID
- Trigger agent completion event
- Assert WebSocket receives AgentCompleted message

**Test: WebSocket approve message uses authenticated user ID**
- Connect authenticated WebSocket as user "alice"
- Send Approve message with request_id (no approver_id)
- Assert approval recorded with approver="alice"
- Assert other users cannot approve as "alice"

**Test: WebSocket rejects approve for unauthorized workflow**
- Connect authenticated WebSocket
- Send Approve message for workflow user cannot access
- Assert receives Error message
- Assert approval not applied

**Test: WebSocket handles subscribe/unsubscribe messages**
- Send Subscribe message for workflow ID
- Send Unsubscribe message
- Trigger workflow event
- Assert no message received (unsubscribed)

**Test: Multiple WebSocket subscribers receive same event**
- Connect 3 WebSocket clients to same workflow
- Trigger workflow event
- Assert all 3 clients receive message

### 3.3 Notification System

**Test: Email notification sent on approval required**
- Create approval request
- Call NotificationDispatcher.dispatch_approval_required
- Assert email channel notified
- Assert email contains approval URL and timeout

**Test: Multiple notification channels all receive notification**
- Configure email, Slack, and FCM channels
- Dispatch approval required event
- Assert all 3 channels notified
- Assert no channel errors block others

**Test: Notification channel failure logged but doesn't block dispatch**
- Mock one channel to return error
- Dispatch approval required
- Assert other channels still notified
- Assert error logged

**Test: Escalation notification includes escalation level**
- Escalate approval to level 2
- Dispatch escalation notification
- Assert notification includes "Escalated to: Manager"
- Assert notification marked as urgent

### 3.4 Escalation Configuration

**Test: Escalation timeout triggers after configured minutes**
- Create approval request with timeout=60 minutes
- Set current time to +61 minutes
- Run escalation checker task
- Assert approval escalated to next level

**Test: Escalation chain progresses through levels**
- Configure 3-level escalation chain
- Let first level timeout
- Let second level timeout
- Assert escalated to level 3 (final)
- Assert third level timeout marks workflow as failed

**Test: Escalation notification sends to correct role**
- Escalate to supervisor level
- Assert notification sent to all users with "supervisor" role
- Assert users without "supervisor" role don't receive notification

**Test: Max escalations reached fails workflow**
- Create approval request with max_escalations=2
- Escalate through all levels
- Let final level timeout
- Assert workflow state = Failed
- Assert administrators notified of failure

**Test: Approval at escalated level clears escalation**
- Create escalated approval request
- Approve at level 2 (manager)
- Assert workflow resumes normal execution
- Assert no further escalation notifications sent

---

## Phase 4: Workflow Scheduling

### 4.1 Priority Queue Implementation

**Test: Critical priority workflow dequeued before High priority**
- Push CRITICAL workflow
- Push HIGH workflow
- Call pop()
- Assert returns CRITICAL workflow

**Test: Same priority ordered by queued_at (FIFO)**
- Push 2 HIGH workflows with different timestamps
- Call pop() twice
- Assert first pushed workflow returned first

**Test: pop returns None when max_concurrent reached**
- Set max_concurrent=2
- Simulate 2 workflows running
- Call pop()
- Assert returns None

**Test: complete() decrements running_count**
- Set running_count=2
- Call complete()
- Assert running_count=1
- Assert pop() now returns workflow

**Test: TOCTOU race fixed - concurrent pop calls respect limit**
- Spawn 10 tasks calling pop() simultaneously with max_concurrent=2
- Assert exactly 2 workflows returned
- Assert no race condition causes >2 concurrent

**Test: Ord implementation matches priority expectations**
- Create QueuedWorkflow with CRITICAL priority
- Create QueuedWorkflow with LOW priority
- Assert CRITICAL < LOW (higher priority = lower ordering)

### 4.2 Workflow Scheduler with Preemption

**Test: should_preempt returns lower priority workflow ID**
- Add CRITICAL workflow to queue
- Simulate 2 NORMAL workflows running
- Call should_preempt()
- Assert returns ID of oldest NORMAL workflow

**Test: should_preempt returns None when all running are higher priority**
- Add NORMAL workflow to queue
- Simulate CRITICAL workflow running
- Call should_preempt()
- Assert returns None

**Test: should_preempt returns None for non-preemptible workflow**
- Add CRITICAL workflow to queue
- Simulate CRITICAL workflow running with preemptible=false
- Call should_preempt()
- Assert returns None

**Test: Preemption checkpoints workflow before pausing**
- Start workflow execution
- Trigger preemption
- Assert checkpoint saved before pause
- Assert workflow state saved as Preempted

**Test: Preempted workflow resumes from checkpoint**
- Preempt workflow at layer 2
- Resume workflow when resources available
- Assert execution continues from layer 2 (not from start)
- Assert agent results from layer 1 preserved

---

## Phase 5: Observability

### 5.1 Structured Logging

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

### 5.2 Metrics

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

### 5.3 Audit Logging

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

---

## Integration Tests

### End-to-End Workflow Execution

**Test: Complete document creation workflow from start to finish**
- Create workflow via GraphQL
- Connect WebSocket client
- Wait for completion
- Assert all 4 agents executed in order
- Assert 3 approval points passed
- Assert WebSocket received all state updates
- Assert final document generated

**Test: Workflow recovery after crash**
- Start workflow execution
- Simulate crash after layer 2
- Restart orchestrator
- Call recover_workflow
- Assert workflow resumes at layer 3
- Assert final document generated correctly

**Test: Parallel execution with dependencies**
- Create workflow with agents that have parallel branches
- Execute workflow
- Assert independent agents ran concurrently
- Assert dependent agents waited for dependencies

**Test: Human-in-the-loop approval with modification**
- Execute workflow to approval point
- Approve with modifications to agent output
- Assert next agent received modified output
- Assert audit log records modifications

**Test: Escalation flow for timeout approval**
- Create approval request
- Wait for timeout
- Assert escalation notification sent
- Assert escalated approver can approve
- Assert workflow resumes after escalation approval

**Test: Priority queue orders execution correctly**
- Submit LOW, CRITICAL, HIGH workflows
- Assert CRITICAL executes first
- Assert HIGH executes before LOW

**Test: Preemption of low priority workflow**
- Start LOW priority workflow
- Submit CRITICAL workflow
- Assert LOW workflow preempted
- Assert CRITICAL workflow completes
- Assert LOW workflow resumes and completes

---

## Property-Based Tests

**Test: State machine transitions never reach invalid states**
- Use proptest to generate random event sequences
- Assert state machine always in valid state
- Assert no stuck states (no progress for N events)

**Test: DAG builder produces valid topological sort**
- Use proptest to generate random dependency graphs
- Assert no cycles in output DAG
- Assert dependencies always satisfied

**Test: Priority queue maintains heap property**
- Randomly push/pop workflows
- Assert heap property maintained after each operation
- Assert pop() always returns highest priority
