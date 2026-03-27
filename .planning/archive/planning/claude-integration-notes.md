# Integration Notes: Opus Review Feedback

## Date: 2026-03-10

---

## Recommendations Accepted

### 1. State Machine Recovery (MUST FIX)

**Issue:** Plan underspecified how state machine reconstructs from DuckDB checkpoint.

**Integration:** Add "State Machine Recovery" section to plan:
- Deserialize `WorkflowState` from string/enum representation
- Reconstruct state machine using smlang's `StateMachine::new()` with initial state
- Validate deserialized state against current workflow definition version
- Add state validation step on checkpoint load

### 2. Workflow Versioning (MUST FIX)

**Issue:** No handling for workflow definition changes during in-flight workflows.

**Integration:** Add workflow versioning:
- `WorkflowContext` includes `workflow_version: String`
- Checkpoints capture version at save time
- Resume validates version compatibility
- Support for "grandfathering" in-flight workflows with original definition

### 3. SSE Reconnection Strategy (MUST FIX)

**Issue:** No specification for client reconnection and event replay.

**Integration:** Add SSE implementation details:
- Last-Event-ID header support for reconnection
- Event buffer with configurable size (default 100 events)
- At-least-once delivery semantics
- Client-side reconnection logic in Dioxus frontend

### 4. Approval Timeout Behavior (MUST FIX)

**Issue:** Plan mentions timeout but doesn't specify behavior.

**Integration:** Specify timeout behavior:
- Default: 24 hours, then escalate to supervisor
- Configurable per document type
- Notification system for pending approvals
- Optional: auto-reject after extended timeout (72h)

### 5. Concurrency Testing (SHOULD FIX)

**Issue:** No concurrency testing specified.

**Integration:** Add to testing strategy:
- Use `loom` for deterministic concurrency testing
- Tests for concurrent approval requests
- Tests for state race conditions
- Tests for concurrent checkpoint writes

### 6. Performance Requirements (SHOULD FIX)

**Issue:** No performance targets specified.

**Integration:** Add "Performance Requirements" section:
- State transition: <100ms
- Checkpoint save: <500ms
- Agent execution: configurable (default 5 min)
- Memory per workflow: <10MB
- SSE event latency: <50ms

---

## Recommendations Not Accepted (with Rationale)

### 1. Reconsider smlang → Keep smlang

**Reviewer concern:** smlang may be unmaintained, limited async support.

**Counter-rationale:** 
- smlang provides compile-time validation which is valuable
- We can wrap smlang in async-friendly interface
- If issues arise, we can refactor to hand-rolled state machine
- The DSL improves code readability significantly

**Mitigation:** Add fallback plan in Phase 1 to evaluate smlang and switch to hand-rolled if needed.

### 2. New Crate → Keep separate crate

**Reviewer concern:** New `iou-orchestrator` crate may be unnecessary.

**Counter-rationale:**
- Orchestration is a distinct concern from agent implementation
- Separate crate allows independent evolution
- Cleaner dependency management for testing
- Existing `iou-ai` already does pipeline execution; orchestration is different

**Clarification in plan:** Document relationship between `WorkflowEngine` (existing), `AgentPipeline` (existing), and `WorkflowOrchestrator` (new):
- `WorkflowEngine`: Domain workflow definitions (what approvals needed)
- `AgentPipeline`: Sequential agent execution (current implementation)
- `WorkflowOrchestrator`: State machine with human-in-the-loop (new)
- These are complementary systems with different responsibilities

### 3. Checkpoint Table → Keep separate table

**Reviewer concern:** Duplicates patterns in existing `document_audit`.

**Counter-rationale:**
- `document_audit`: Human-facing audit trail (what changed, who, when)
- `workflow_checkpoints`: Machine-readable state snapshots for recovery
- Different access patterns, different schemas
- Checkpoints need efficient serialization/deserialization, not human-readable audit

---

## Recommendations Deferred

### Metrics and Observability

Deferred to post-MVP integration with existing monitoring.

### Workflow Visualization UI

Deferred to separate frontend project.

### Advanced Retry Policies

Deferred to Phase 2 after basic retry is proven.

---

## Summary of Plan Updates

1. **New Section:** "State Machine Recovery" - recovery algorithm, validation
2. **New Section:** "Workflow Versioning" - version compatibility, migration
3. **New Section:** "SSE Implementation" - reconnection, event replay
4. **New Section:** "Performance Requirements" - latency, memory targets
5. **Updated:** Testing strategy - added concurrency testing with loom
6. **Updated:** Architecture - clarified relationship to existing workflow systems
7. **Updated:** Approval timeout - specified behavior and escalation
