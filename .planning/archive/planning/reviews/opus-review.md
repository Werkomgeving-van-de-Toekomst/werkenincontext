# Review: Agent-Orchestrated Document Creation with Human-in-the-Loop Implementation Plan

**Reviewer:** Claude Opus 4.6
**Date:** 2026-03-10
**Review Type:** Technical Architecture Review

---

## Executive Summary

The implementation plan presents a thoughtful, well-structured approach to adding human-in-the-loop capabilities to IOU-Modern's agent system. The decision to pursue a Rust-native orchestration approach rather than LangGraph integration is well-justified and aligned with the existing codebase. However, there are several areas requiring clarification, additional detail, or reconsideration before implementation should proceed.

**Overall Assessment:** **Conditionally Approved with Recommendations**

---

## 1. Completeness Analysis

### 1.1 Strengths

The plan demonstrates strong completeness in several areas:

- **State Machine Design**: The smlang DSL definition is comprehensive with well-defined states and transitions
- **Data Structures**: `WorkflowContext` and `AgentResult` types are well-specified
- **Error Classification**: Building on the existing `ErrorSeverity` pattern from `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/error.rs` is appropriate
- **API Contract**: Endpoints are clearly specified with appropriate HTTP semantics

### 1.2 Gaps and Missing Components

**Critical Gap: State Machine Initialization**

The plan shows a state machine definition using `smlang` but does not address:

1. **State reconstruction on recovery**: How does the state machine get reconstructed from a DuckDB checkpoint? The `WorkflowContext` is serialized as JSON, but the state machine itself needs to be rehydrated to the correct state.

```rust
// The plan shows checkpoint data:
pub struct WorkflowCheckpoint {
    pub state: WorkflowState,  // How is this reconstructed?
    pub context_json: String,
    // ...
}
```

2. **No specification for state persistence during `AwaitingApproval`**: The workflow sits in this state potentially for hours/days. What happens if the server restarts? The plan mentions checkpoints are made "for human approval" but the resume flow is underspecified.

**Recommendation:** Add a section on "State Machine Recovery" that details:
- How to deserialize `WorkflowState` from string representation
- How to reconstruct the smlang state machine from a checkpoint
- Validation that the deserialized state is valid for the current workflow definition

**Missing: Workflow Versioning**

The plan does not address what happens when the workflow definition changes while workflows are in-flight:

- What if a new agent is added to the pipeline?
- What if state transitions are modified?
- Should workflows complete with their original definition or migrate?

**Recommendation:** Add workflow versioning with either:
- Capture workflow definition version in `WorkflowContext`
- Support for "grandfathering" in-flight workflows

**Missing: SSE Reconnection Strategy**

The plan specifies SSE events for UI updates but does not address:

- Client reconnection after network failure
- Event replay for missed events
- How to handle "last event ID" in SSE implementation

**Missing: Parallel Execution Coordination**

Section 3.2 mentions parallel execution groups but lacks:

- Dependency resolution between parallel agents
- How partial failures in parallel groups are handled
- Whether all agents in a parallel group must succeed for the group to succeed

---

## 2. Feasibility Assessment

### 2.1 Technology Stack Feasibility

**smlang State Machine Library**

The plan proposes using `smlang` for state machine definition. Research shows:

- **Concern**: `smlang` has limited async support and may not be well-maintained (last release >2 years ago based on typical Rust ecosystem patterns)
- **Alternative**: Consider `rustfsm` or hand-rolled state machine using enums and pattern matching, which would give more control

**DuckDB for Checkpoints**

The use of DuckDB for checkpoint storage is feasible but with caveats:

- DuckDB's JSON storage is adequate for `WorkflowContext` serialization
- However, DuckDB is single-writer by design; concurrent checkpoint writes from multiple workflows could contend
- The plan notes "1-10 workflows" which likely avoids this issue

**Recommendation:** Consider batching checkpoint writes or using a queue pattern if scaling beyond 10 workflows is anticipated.

### 2.2 Integration with Existing Codebase

**Strong Integration Points:**

The plan correctly identifies and leverages existing patterns:

- Error handling via `ErrorSeverity` from `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/error.rs`
- `WorkflowStatus` enum from `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows.rs`
- DuckDB database layer from `/Users/marc/Projecten/iou-modern/crates/iou-api/src/db.rs`
- Existing agent pipeline from `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs`

**Integration Concern: Two Workflow Systems**

The codebase currently has TWO workflow-related systems:

1. **`WorkflowEngine` in `/Users/marc/Projecten/iou-modern/crates/iou-api/src/workflows/mod.rs`**: A general workflow engine for document approvals and routing
2. **`AgentPipeline` in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs`**: The agent pipeline for document generation

The plan proposes a THIRD system: `iou-orchestrator` crate with its own state machine.

**Critical Question:** Why not extend the existing `WorkflowEngine` rather than creating a new orchestration crate?

**Recommendation:** Clarify the relationship between:
- The existing `WorkflowEngine` (workflow definitions, steps, routing)
- The proposed `WorkflowOrchestrator` (agent coordination, human-in-the-loop)
- Whether they should be unified or remain separate

**Database Schema Concern**

The plan proposes a new `workflow_checkpoints` table, but the existing schema already has:

- `documents` table (for document metadata)
- `document_audit` table (for audit trail)
- `document_versions` table (for version history)

It's unclear why workflow checkpoints need a separate table rather than being stored in `document_audit` or a new `workflow_state` table linked to documents.

---

## 3. Edge Cases and Failure Modes

### 3.1 Unaddressed Edge Cases

**1. Approval Timeout Handling**

The plan mentions `approval_timeout_hours` in config but doesn't specify:
- What happens when approval times out?
- Does the workflow fail? Get auto-approved? Escalate to a supervisor?
- How are timeout notifications sent?

**2. Concurrent Modification Conflicts**

Scenario: A human is modifying agent output while the agent is retrying in the background.

- What happens if the modification completes after the agent succeeds?
- Is there optimistic locking on `AgentResult.modifications`?

**3. Mid-Agent Cancellation**

If a workflow is cancelled during agent execution:
- How is the agent task stopped? (Tokio task cancellation?)
- What cleanup is needed for partial results?
- How are cancelled workflows represented in the audit trail?

**4. Checkpoint Corruption**

If a checkpoint cannot be deserialized:
- Does the workflow fail? Start over?
- Is there a checkpoint validation step on load?
- Are there checkpoints-of-checkpoints for recovery?

**5. SSE Client Disconnection During Critical Updates**

If the UI client disconnects during a state transition:
- Are critical events queued for replay?
- Is there at-least-once or at-most-once delivery guarantee?

---

## 4. Testing Strategy Assessment

### 4.1 Strengths

The plan includes:
- Unit tests for state machine transitions
- Integration tests for full workflows
- Mock utilities for testing

### 4.2 Gaps

**Missing: Concurrency Testing**

Given the async, multi-agent nature, the plan should include:

- Tests for concurrent approval requests (what if two users approve simultaneously?)
- Tests for concurrent checkpoint writes
- Tests for state race conditions (e.g., approval arrives during agent failure)

**Missing: Recovery Testing**

The plan mentions crash recovery but doesn't specify testing for:

- Simulating process kill during each state
- Validating checkpoint consistency after recovery
- Testing with corrupted checkpoint data

**Missing: Performance Testing**

While the plan targets 1-10 workflows, there should be:

- Baseline performance metrics for each agent
- Target latencies for state transitions
- Memory profiling for long-running workflows

**Recommendation:** Add a "Performance Requirements" section with:
- Max acceptable latency for each transition
- Memory budget per workflow
- Target checkpoint size

---

## 5. Specific Technical Recommendations

### 5.1 State Machine Implementation

**Issue:** The smlang DSL shown in the plan has syntax issues:

```rust
// From plan:
Running + AgentComplete(AgentType) [can_proceed] = Running,
```

**Concern:** The `AgentComplete(AgentType)` event carries data, but guards in smlang typically don't have access to event payload. The `can_proceed` guard needs to check `pending_approvals`, which is in context, not the event.

**Recommendation:** Clarify whether:
1. The state machine processes the event first (updates context), then checks guard
2. The guard needs access to both state and event payload

### 5.2 Channel Architecture

The plan shows `tokio::sync::mpsc` for inter-agent communication but doesn't specify:

- Channel capacity (bounded vs unbounded?)
- Backpressure strategy when channels are full
- How channels are cleaned up after workflow completion

**Recommendation:** Use bounded channels with capacity specified in config, and document the backpressure behavior (block? drop? fail?).

### 5.3 SSE Implementation Detail

The plan shows SSE for workflow events but doesn't specify the Axum implementation:

```rust
// From plan:
pub async fn stream_workflow_events(
    Path(id): Path<Uuid>,
    State(app): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>>;
```

**Issue:** `Sse` type and axum SSE integration need clarification. Axum 0.8 has changed SSE handling.

**Recommendation:** Specify using `axum::response::Sse` with `tokio_stream` wrappers, and include a complete example in the implementation phase.

---

## 6. Architecture Concerns

### 6.1 New Crate vs Extension

The plan proposes a new `iou-orchestrator` crate. Consider:

- **Alternative:** Extend `iou-ai` with an `orchestration` module
- **Benefit:** Reduces crate complexity, keeps agent-related code together
- **Trade-off:** Larger crate, but cleaner dependency graph

**Recommendation:** Reconsider whether a separate crate is necessary vs. adding modules to existing crates.

### 6.2 Directory Structure

The proposed structure:
```
crates/
├── iou-orchestrator/          # Nieuwe crate voor orchestration
├── iou-ai/                    # Bestaande agents (uitbreiden)
└── iou-api/                   # Bestaande API (uitbreiden)
```

**Concern:** This separates orchestration from the agents it orchestrates, creating potential synchronization issues.

**Alternative:**
```
crates/
├── iou-ai/
│   ├── src/agents/            # Existing agents
│   └── src/orchestration/     # NEW: Orchestration logic
└── iou-api/                   # Existing API
```

---

## 7. Priority Recommendations

### Must Address Before Implementation

1. **Clarify relationship between existing `WorkflowEngine` and new orchestrator**
2. **Specify state machine recovery from checkpoints**
3. **Add workflow versioning strategy**
4. **Detail SSE reconnection and event replay**
5. **Specify behavior on approval timeout**

### Should Address in Phase 1

1. **Reconsider smlang vs. simpler state machine**
2. **Evaluate whether new crate is necessary**
3. **Add concurrency testing strategy**
4. **Define performance requirements**
5. **Specify parallel execution failure handling**

### Can Defer to Later

1. **Metrics and observability integration**
2. **Workflow visualization/debugging UI**
3. **Advanced retry policies (jitter, circuit breakers)**

---

## 8. Integration Checklist

Before implementation begins, ensure:

- [ ] Decision on smlang vs. alternative state machine approach
- [ ] Clarification on `WorkflowEngine` vs `WorkflowOrchestrator` relationship
- [ ] Database migration for `workflow_checkpoints` table (or alternative)
- [ ] SSE implementation specification for Axum 0.8
- [ ] Approval timeout behavior specification
- [ ] State machine recovery algorithm documented
- [ ] Parallel execution failure mode specified
- [ ] Channel capacity and backpressure strategy defined

---

## 9. Conclusion

The implementation plan is well-conceived and demonstrates strong understanding of both the requirements and the existing codebase. The Rust-native approach is appropriate and the state machine pattern is well-suited to the problem domain.

However, before implementation begins, the following should be addressed:

1. **Resolve architectural ambiguity** around multiple workflow systems
2. **Add missing specifications** for recovery, timeouts, and concurrency
3. **Reconsider the new crate** vs. extending existing crates
4. **Validate technology choices** (smlang maintenance status)

With these addressed, the plan provides a solid foundation for implementation.

---

## Appendix: Code References

### Existing State Machine Pattern
The codebase already has a state machine pattern in `WorkflowStatus`:
- File: `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows.rs`
- Pattern: Enum with `can_transition_to()` method

### Existing Error Handling
The error classification pattern is already established:
- File: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/error.rs`
- Pattern: `PipelineError` with `severity()` method returning `ErrorSeverity`

### Existing Database Schema
Document and audit tables already exist:
- Files: `/Users/marc/Projecten/iou-modern/migrations/030_documents.sql`, `/Users/marc/Projecten/iou-modern/migrations/031_templates.sql`
- Tables: `documents`, `document_audit`, `templates`

### Existing API Integration
Document creation endpoints already stubbed:
- File: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/documents.rs`
- Status: TODO comments indicate where orchestration should be integrated
