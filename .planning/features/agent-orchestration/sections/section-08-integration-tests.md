Now I have all the context needed. Let me extract the relevant content for section-08-integration-tests.

Looking at the index.md, section-08-integration-tests is described as:
"End-to-end workflow execution tests, crash recovery simulation, parallel execution with dependencies, human-in-the-loop approval with modification, escalation flow, priority queue ordering, preemption scenarios, property-based tests for state machine and DAG builder."

And from the dependency graph, it depends on sections 03, 04, 05, 06, 07.

From the TDD plan, the integration tests section is at the end:
- End-to-End Workflow Execution tests
- Property-Based Tests

From the main plan, the relevant sections are in the Testing Strategy section.

# Section 08: Integration Tests

## Overview

This section implements comprehensive integration tests for the agent orchestration system. These tests verify end-to-end workflows, component interactions, and system-wide behaviors that cannot be validated through unit tests alone. All integration tests depend on sections 03 through 07 being complete.

**Dependencies:** section-03-checkpoint-recovery, section-04-graphql-api, section-05-websocket-notifications, section-06-workflow-scheduler, section-07-observability

## File Structure

```
server/crates/iou-orchestrator/tests/
├── integration_tests.rs       # Main integration test module
├── e2e/
│   ├── workflow_execution.rs  # End-to-end workflow tests
│   ├── crash_recovery.rs      # Crash recovery simulation tests
│   ├── parallel_execution.rs  # Parallel agent execution tests
│   ├── approval_flow.rs       # Human-in-the-loop approval tests
│   ├── escalation.rs          # Escalation flow tests
│   └── scheduler.rs           # Priority queue and preemption tests
├── property_based/
│   ├── state_machine.rs       # State machine proptests
│   └── dag_builder.rs         # DAG builder proptests
└── fixtures/
    ├── test_workflows.rs      # Test workflow definitions
    └── mock_agents.rs         # Mock agent implementations
```

## Test Environment Setup

**Location:** `server/crates/iou-orchestrator/tests/common/mod.rs`

The integration test environment requires:

1. **Test Database:** PostgreSQL test container or database
2. **Test Orchestrator:** Minimal orchestrator instance with mock agents
3. **Test Clock:** Controllable time for timeout/escalation testing
4. **Mock Notification Channels:** In-memory notification capture

```rust
// Common test infrastructure
pub struct TestEnvironment {
    pub db_pool: PgPool,
    pub orchestrator: TestOrchestrator,
    pub mock_notifications: Arc<RwLock<Vec<Notification>>>,
    pub test_clock: Arc<TestClock>,
}

pub struct TestClock {
    pub now: Arc<RwLock<DateTime<Utc>>>,
}

impl TestClock {
    pub fn advance(&self, duration: Duration) {
        *self.now.write().unwrap() += chrono::Duration::from_std(duration).unwrap();
    }
}
```

## End-to-End Workflow Execution Tests

**Location:** `server/crates/iou-orchestrator/tests/e2e/workflow_execution.rs`

### Test Cases

**Test: Complete document creation workflow from start to finish**

Create a workflow via GraphQL, connect a WebSocket client, wait for completion, and verify:
- All 4 agents executed in order (Research, Content, Compliance, Review)
- 3 approval points passed successfully
- WebSocket received all state updates
- Final document was generated correctly
- Audit log contains all expected entries

**Test: Workflow execution with modified approval**

Execute workflow to first approval point, approve with modifications to agent output, and verify:
- Next agent received the modified output (not original)
- Audit log records the modification with diff
- WebSocket subscribers received the modified data
- Final document incorporates the modifications

**Test: Workflow cancellation during execution**

Start workflow execution, send cancellation via GraphQL, and verify:
- Running agents received cancellation signal
- Workflow state transitioned to Cancelled
- WebSocket received cancellation notification
- No further agents were started
- Partial results were checkpointed if configured

### Implementation Notes

- Use `tokio::test` with `#[serial]` attribute for database-dependent tests
- Use `testcontainers` for PostgreSQL test database
- Mock external agent calls to avoid real API calls
- Verify both GraphQL and WebSocket paths for each operation

## Crash Recovery Tests

**Location:** `server/crates/iou-orchestrator/tests/e2e/crash_recovery.rs`

### Test Cases

**Test: Workflow recovery after simulated crash**

1. Start workflow execution
2. Execute through layer 2 (Research and Content agents complete)
3. Simulate crash (drop orchestrator instance)
4. Create new orchestrator instance
5. Call `recover_workflow` for the workflow ID
6. Verify:
   - Workflow resumes at layer 3 (Compliance agent)
   - Agent results from layer 1 and 2 are preserved
   - State machine restored to correct state
   - Final document generated correctly

**Test: Recovery from checkpoint at approval point**

1. Execute workflow to approval point
2. Save checkpoint with AwaitingApproval state
3. Simulate crash and restart
4. Recover workflow
5. Verify workflow is still awaiting approval (not re-executed)

**Test: Recovery with migrated checkpoint version**

1. Create checkpoint with old version format
2. Apply migration during recovery
3. Verify `RecoveryPlan.was_migrated` is true
4. Verify workflow resumes successfully

**Test: Multiple checkpoints - latest is used**

1. Execute workflow and create 3 checkpoints
2. Modify database to have different timestamps
3. Call recover_workflow
4. Verify most recent checkpoint is used

### Implementation Notes

- Use PostgreSQL transaction rollback to simulate crash
- Test with `PgPool` disconnect to simulate connection loss
- Verify checkpoint cleanup (old checkpoints deleted after recovery)

## Parallel Execution Tests

**Location:** `server/crates/iou-orchestrator/tests/e2e/parallel_execution.rs`

### Test Cases

**Test: Parallel execution with independent agents**

Create workflow with agents that have parallel branches:
```
        Research
         /     \
    ContentA  ContentB
         \     /
          Review
```
Execute and verify:
- ContentA and ContentB ran concurrently (verify timing overlap)
- Research completed before Content agents started
- Review started after both Content agents completed
- Results from both Content branches available to Review

**Test: Sequential agents do not run in parallel**

Create workflow with sequential dependencies (A -> B -> C)
Execute and verify:
- Only one agent running at a time
- Each agent starts after previous completes
- Total execution time equals sum of individual times

**Test: Agent failure cancels parallel layer**

1. Create layer with 2 agents where one fails
2. Execute layer
3. Verify:
   - Both agents cancelled (or one failed, one cancelled)
   - Workflow state transitioned to Failed
   - No subsequent layers executed
   - WebSocket received failure notification

**Test: Optional agent failure does not cancel layer**

1. Create layer with required and optional agents
2. Mock optional agent to fail
3. Execute layer
4. Verify:
   - Required agent succeeded
   - Workflow continued with partial results
   - Optional failure logged but not fatal

### Implementation Notes

- Use `tokio::time::sleep` to verify timing overlap
- Record agent start/completion timestamps
- Use `Arc<AtomicUsize>` for concurrent agent tracking

## Human-in-the-Loop Approval Tests

**Location:** `server/crates/iou-orchestrator/tests/e2e/approval_flow.rs`

### Test Cases

**Test: Approval via WebSocket message**

1. Connect authenticated WebSocket as user "alice"
2. Execute workflow to approval point
3. Send Approve message via WebSocket (no approver_id in message)
4. Verify:
   - Approval recorded with approver="alice"
   - Workflow resumed execution
   - WebSocket subscriber received approval confirmation
   - Other WebSocket clients did not receive approval as "alice"

**Test: Approval via GraphQL mutation**

1. Create approval request
2. Send GraphQL `approve` mutation with JWT for user "bob"
3. Verify:
   - Approval record has approver_id matching JWT subject
   - Workflow advanced to next state
   - Audit log contains approval event with actor

**Test: Unauthorized approval rejected**

1. Create approval request requiring "supervisor" role
2. Send approve mutation with JWT for "user" role
3. Verify:
   - Mutation returns FORBIDDEN error
   - Approval not applied in database
   - Workflow still awaiting approval
   - WebSocket not notified of approval

**Test: Approval with modification**

1. Execute workflow to approval point
2. Submit approval with modifications to agent output JSON
3. Verify:
   - Next agent received modified output
   - Audit log includes full modification diff
   - Modification fields recorded in audit details
   - WebSocket received modified approval event

**Test: Reject workflow**

1. Execute workflow to approval point
2. Send GraphQL `reject` mutation with reason
3. Verify:
   - Workflow state transitioned to Failed (or Cancelled)
   - Rejection reason recorded
   - WebSocket received rejection notification
   - Audit log includes rejection event

### Implementation Notes

- Use `jsonwebtoken` to create test JWTs with custom claims
- Mock user roles in test database
- Verify both WebSocket and GraphQL approval paths

## Escalation Flow Tests

**Location:** `server/crates/iou-orchestrator/tests/e2e/escalation.rs`

### Test Cases

**Test: Escalation timeout triggers after configured minutes**

1. Create approval request with timeout=60 minutes
2. Advance test clock by 61 minutes
3. Run escalation checker task
4. Verify:
   - Approval escalated to next level
   - Escalation notification sent to supervisor role
   - Approval request `escalated` flag is true
   - New expiration time set

**Test: Escalation chain progresses through levels**

1. Configure 3-level escalation chain
2. Let first level timeout (advance clock)
3. Let second level timeout (advance clock)
4. Verify:
   - Escalated to level 2 after first timeout
   - Escalated to level 3 after second timeout
   - Third level timeout marks workflow as Failed
   - Administrators notified of final failure

**Test: Escalation notification sends to correct role**

1. Escalate to supervisor level
2. Create test users with various roles
3. Verify:
   - Notification sent to all users with "supervisor" role
   - Users without "supervisor" role did not receive notification
   - Notification includes "Escalated to: Supervisor"
   - Notification marked as urgent

**Test: Approval at escalated level clears escalation**

1. Create escalated approval request (level 2)
2. Approve at level 2 (manager role)
3. Verify:
   - Workflow resumes normal execution
   - No further escalation notifications sent
   - Escalation level reset to 0

**Test: Max escalations reached fails workflow**

1. Create approval request with max_escalations=2
2. Escalate through all levels
3. Let final level timeout
4. Verify:
   - Workflow state = Failed
   - Administrators notified of failure
   - Audit log records escalation failure

### Implementation Notes

- Use `TestClock` for controllable time advancement
- Mock notification channels to capture sent notifications
- Verify notification content and recipients

## Priority Queue and Scheduler Tests

**Location:** `server/crates/iou-orchestrator/tests/e2e/scheduler.rs`

### Test Cases

**Test: Priority queue orders execution correctly**

1. Submit LOW priority workflow
2. Submit CRITICAL priority workflow
3. Submit HIGH priority workflow
4. Wait for all to complete
5. Verify:
   - CRITICAL executed first
   - HIGH executed before LOW
   - All workflows completed successfully

**Test: Same priority ordered by queued_at (FIFO)**

1. Submit 3 HIGH workflows with 100ms delay between each
2. Verify execution order matches submission order

**Test: Preemption of low priority workflow**

1. Start LOW priority workflow (verify it's running)
2. Submit CRITICAL workflow
3. Verify:
   - LOW workflow preempted (checkpointed and paused)
   - CRITICAL workflow completes
   - LOW workflow resumes from checkpoint
   - LOW workflow completes successfully

**Test: Non-preemptible workflow not preempted**

1. Start CRITICAL workflow with preemptible=false
2. Submit another CRITICAL workflow
3. Verify:
   - Second workflow queued (not preempted)
   - First workflow continues to completion
   - Second workflow starts after first completes

**Test: Max concurrent limit respected**

1. Set max_concurrent=2
2. Submit 5 workflows
3. Verify:
   - Only 2 workflows running at any time
   - As each completes, next starts
   - Total completion time reflects limit

### Implementation Notes

- Use `tokio::spawn` to submit workflows concurrently
- Track running state via metrics or state queries
- Verify checkpoint existence for preempted workflows

## Property-Based Tests

**Location:** `server/crates/iou-orchestrator/tests/property_based/`

### State Machine Properties

**Test: State machine transitions never reach invalid states**

Use `proptest` to generate random event sequences and verify:
- State machine always in valid state after any event
- No stuck states (state never changes despite N events)
- Terminal states (Completed, Failed, Cancelled) never transition out

```rust
proptest! {
    #[test]
    fn prop_state_machine_valid(events in prop::collection::vec(event_strategy(), 0..100)) {
        let mut sm = WorkflowStateMachine::new();
        for event in events {
            sm.handle(event);
            assert!(sm.is_valid_state());
        }
    }
}
```

**Test: State machine is deterministic**

Same event sequence always produces same final state.

### DAG Builder Properties

**Test: DAG builder produces valid topological sort**

Use `proptest` to generate random dependency graphs and verify:
- No cycles in output DAG
- Dependencies always satisfied (agent only appears after its dependencies)
- All input agents appear in output
- Independent agents grouped into same layer when possible

```rust
proptest! {
    #[test]
    fn prop_dag_topological_sort(deps in prop::collection::vec(dependency_strategy(), 1..20)) {
        let dag = build_execution_dag(&deps)?;
        // Verify no cycles
        // Verify dependencies satisfied
        // Verify all agents present
    }
}
```

**Test: DAG builder handles duplicate dependencies**

Duplicate dependency definitions do not cause duplicate agents in output.

### Priority Queue Properties

**Test: Priority queue maintains heap property**

Randomly push/pop workflows and verify:
- Heap property maintained after each operation
- `pop()` always returns highest priority (CRITICAL > HIGH > NORMAL > LOW)
- Same priority returns FIFO (earlier queued_at first)

## Test Execution

**Location:** `server/crates/iou-orchestrator/Cargo.toml`

Configure test profiles and dependencies:

```toml
[dev-dependencies]
testcontainers = "0.23"
proptest = "1.5"
tokio = { version = "1.43", features = ["test-util"] }
serial_test = "3.2"
```

Run integration tests:

```bash
# All integration tests
cargo test --test integration_tests

# Specific test suite
cargo test --test workflow_execution

# Property-based tests (more runs)
PROPTEST_CASES=1000 cargo test --test property_based
```

## Success Criteria

- [ ] All end-to-end workflow tests pass
- [ ] Crash recovery tests verify successful resume from checkpoint
- [ ] Parallel execution tests verify concurrent and sequential behavior
- [ ] Approval tests verify WebSocket and GraphQL paths
- [ ] Escalation tests verify timeout and notification behavior
- [ ] Scheduler tests verify priority ordering and preemption
- [ ] Property-based tests run with 1000+ cases without failure
- [ ] All tests are deterministic (no flaky runs)
- [ ] Test execution time is reasonable (< 5 minutes total)