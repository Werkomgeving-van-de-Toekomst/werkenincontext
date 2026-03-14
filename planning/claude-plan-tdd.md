# TDD Plan: Agent-Orchestrated Document Creation

## Testing Approach

**Framework:** Rust's built-in test framework (`cargo test`)
**Concurrency Testing:** loom for deterministic concurrency testing
**Organization:** Unit tests in module `mod tests;` blocks, integration tests in `tests/` directory
**Mocks:** Hand-rolled mock traits for agent interfaces

---

## 1. State Machine Tests

### State Transitions (smlang compile-time validatie)
- Test: Created + Start → Running
- Test: Running + AgentComplete (with approval) → Running
- Test: Running + RequireApproval → AwaitingApproval
- Test: AwaitingApproval + Approved → Running
- Test: AwaitingApproval + Rejected → Failed
- Test: AwaitingApproval + TimeoutEscalated → AwaitingEscalation
- Test: Running + AllAgentsComplete → Completed
- Test: Completed + Finalize → Archived
- Test: Running + AgentFailed (can_retry=true) → Retrying
- Test: Running + AgentFailed (can_retry=false) → Failed
- Test: All states + Cancel → Cancelled

### Guard Conditions
- Test: can_proceed returns true when pending_approvals is empty
- Test: can_proceed returns false when pending_approvals has items
- Test: can_retry returns true when all retry counts below max
- Test: can_retry returns false when any agent exceeded max retries

### State Serialization
- Test: WorkflowState::from_str parses all valid state names
- Test: WorkflowState::from_str returns error for invalid state
- Test: WorkflowState::to_string produces parseable output

---

## 2. State Machine Recovery Tests

### Checkpoint Validation
- Test: validate_checkpoint passes for valid checkpoint data
- Test: validate_checkpoint fails with invalid JSON
- Test: validate_checkpoint fails with invalid state string
- Test: validate_checkpoint checks sequence number monotonicity

### Version Compatibility
- Test: validate_compatibility passes when versions match
- Test: validate_compatibility fails when versions differ (no migration)
- Test: validate_compatibility passes with compatible version bump

### Recovery Algorithm
- Test: resume_workflow reconstructs state from checkpoint
- Test: resume_workflow fails with corrupted checkpoint
- Test: resume_workflow validates version before resume

---

## 3. Agent Execution Tests

### Single Agent Execution
- Test: execute_agent returns AgentResult on success
- Test: execute_agent returns AgentError::Timeout when timeout exceeded
- Test: execute_agent retries Transient errors
- Test: execute_agent fails immediately on Permanent error
- Test: execute_agent respects agent-specific timeout configuration

### Parallel Execution
- Test: execute_parallel runs independent agents concurrently
- Test: execute_parallel returns results for all agents
- Test: execute_parallel handles partial failures correctly
- Test: can_run_parallel returns true for compatible agents
- Test: can_run_parallel returns false for dependent agents

### Parallel Failure Modes
- Test: Permanent error in parallel group fails entire group
- Test: Transient error triggers retry for failed agent only
- Test: All Transient errors trigger group retry

### Agent Cancellation
- Test: cancel_agent stops running agent
- Test: cancel_agent cleans up partial results
- Test: cancel_agent is idempotent

---

## 4. Human-in-the-Loop Tests

### Approval Workflow
- Test: AgentComplete with pending_approval creates ApprovalRequest
- Test: ApprovalRequest expires_at is set correctly from config
- Test: Approve decision clears pending_approvals
- Test: Modify decision updates AgentResult with modifications
- Test: Reject decision transitions workflow to Failed

### Timeout Handling
- Test: Approval timeout after standard_timeout_hours triggers escalation
- Test: Escalation creates notification to escalation_contacts
- Test: Supervisor approval in AwaitingEscalation resumes workflow
- Test: Extended timeout triggers auto-reject when configured

### SSE Event Stream
- Test: stream_workflow_events sends AgentStarted event
- Test: stream_workflow_events sends AgentCompleted event
- Test: stream_workflow_events sends ApprovalRequired event
- Test: stream_workflow_events supports Last-Event-ID header
- Test: stream_workflow_events replays events since last ID

### Event Buffer
- Test: EventBuffer stores events up to max_size
- Test: EventBuffer::events_since returns correct events
- Test: EventBuffer::prune removes old events
- Test: EventBuffer handles concurrent push/prune

---

## 5. Checkpoint Tests

### Save Checkpoint
- Test: save_checkpoint stores state and context in DuckDB
- Test: save_checkpoint increments sequence_number
- Test: save_checkpoint includes workflow_version
- Test: save_checkpoint handles concurrent saves (last write wins)

### Load Checkpoint
- Test: load_checkpoint returns latest checkpoint for workflow
- Test: load_checkpoint returns None for non-existent workflow
- Test: load_checkpoint deserializes context correctly

### Checkpoint Strategy
- Test: Checkpoint saved after each agent completion
- Test: Checkpoint saved before awaiting approval
- Test: Checkpoint saved after human modification
- Test: Periodic checkpoint during long-running operations

---

## 6. Error Handling Tests

### Error Classification
- Test: AgentError::is_retryable returns true for Transient
- Test: AgentError::is_retryable returns false for Permanent
- Test: AgentError::is_retryable returns false for Timeout
- Test: AgentError::is_retryable returns false for Cancelled

### Retry Logic
- Test: RetryPolicy::backoff increases exponentially
- Test: RetryPolicy::backoff respects max_backoff_ms
- Test: RetryPolicy::backoff adds jitter when enabled
- Test: RetryPolicy::backoff is deterministic when jitter disabled

---

## 7. Concurrency Tests (loom)

### Concurrent Approval
- Test: Two simultaneous approvals - last one wins
- Test: Approval during agent failure - consistent state
- Test: Modification during retry - modification wins

### Concurrent Checkpoints
- Test: Parallel checkpoint writes don't corrupt data
- Test: Load during save returns consistent checkpoint

### State Races
- Test: Cancel during agent execution stops agent
- Test: Cancel during approval works correctly

---

## 8. Performance Tests

### Latency Targets
- Test: State transition completes in <100ms
- Test: Checkpoint save completes in <500ms
- Test: Checkpoint load completes in <300ms
- Test: SSE event delivery in <50ms

### Memory Budget
- Test: Single workflow state <10MB
- Test: Event buffer <5MB for 100 events
- Test: 10 concurrent workflows <150MB total

### Throughput
- Test: 100 state transitions per second
- Test: 50 checkpoint saves per second

---

## 9. Integration Tests

### Full Workflow
- Test: Complete workflow from Created to Archived
- Test: Workflow with all human approval steps
- Test: Workflow with modification and re-execution
- Test: Workflow handles rejection and retry

### Recovery Scenarios
- Test: Resume from checkpoint after crash
- Test: Resume from each possible state
- Test: Resume with version mismatch fails appropriately

### API Integration
- Test: POST /api/workflows creates workflow
- Test: GET /api/workflows/:id returns status
- Test: POST /api/workflows/:id/approve approves step
- Test: GET /api/workflows/:id/stream streams events

---

## 10. Test Utilities

### Mock Agents
- Test: MockSucceedingAgent always returns success
- Test: MockFailingAgent fails N times then succeeds
- Test: MockTimingAgent simulates configurable delay

### Test Infrastructure
- Test: InMemoryCheckpointStore for testing
- Test: TestClock for deterministic time-based tests
- Test: TestEventBuffer for SSE testing
