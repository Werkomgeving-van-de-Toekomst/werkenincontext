# External Review Integration Notes

## Review Summary
**Model:** claude-opus-4
**Date:** 2026-03-27
**Overall Assessment:** 7/10 - Strong foundation with critical gaps to address

## Decisions on Feedback

### Integrating (High Priority)

#### 1. DAG State Transition Table ✅
**Issue:** State transitions for new DAG states (ParallelExecuting, PartialComplete, MergeResults) are underspecified.
**Action:** Add complete state transition matrix to Phase 1.1, including error conditions and edge cases.

#### 2. Shared State Synchronization ✅
**Issue:** Multiple agents writing to WorkflowContext concurrently - no thread-safety defined.
**Action:** Add `Arc<RwLock<WorkflowContext>>` pattern to Phase 1.3 and define ownership rules.

#### 3. Priority Queue Race Conditions ✅
**Issue:** TOCTOU race in `pop()` - check-then-increment outside lock.
**Action:** Move `running_count` increment inside the lock in Phase 4.1.

#### 4. WebSocket Authentication ✅
**Issue:** No auth described for WebSocket connections.
**Action:** Add JWT authentication layer before WebSocket upgrade in Phase 3.2.

#### 5. Agent Trait Interface ✅
**Issue:** No unified agent trait/interface defined.
**Action:** Add `Agent` trait definition to Phase 1.3.

#### 6. Checkpoint Version Migration ✅
**Issue:** No migration strategy for incompatible checkpoint versions.
**Action:** Add version numbering scheme and migration functions to Phase 2.4.

### Integrating (Medium Priority)

#### 7. Storage Strategy ✅
**Issue:** DuckDB not ideal for operational workflow state.
**Action:** Changed to PostgreSQL for operational data (checkpoints, state). DuckDB can be added later for analytics if needed.

#### 8. Event Bus Backpressure ✅
**Issue:** `broadcast` drops slow receivers silently.
**Action:** Specify bounded `mpsc` for critical events (audit) in Phase 1.4.

#### 9. Approval Authorization ✅
**Issue:** `approverId` from user input - anyone can approve as anyone.
**Action:** Get `approverId` from auth context in Phase 3.1.

#### 10. Escalation Flow ✅
**Issue:** Escalation underspecified - who receives? What if escalation times out?
**Action:** Add escalation configuration and timeout handling to Phase 3.

### Not Integrating (With Reasons)

#### GraphQL + WebSocket "Redundancy"
**Reason:** GraphQL subscriptions are for query-driven updates; WebSocket is for push notifications. They serve different use cases. Will clarify distinction.

#### Hybrid Storage "Unjustified"
**Reason:** DuckDB for analytics/checkpoints is a valid architectural choice for analytical workloads. Will add brief justification.

#### Resource Limits
**Reason:** Out of scope for v1 - can be added in Phase 6 (Operations) if needed.

#### Fault Injection Testing
**Reason:** Advanced testing technique - can be added post-MVP. Current plan covers basic recovery testing.

## Plan Updates Summary

1. **Phase 1.1:** Add state transition diagram/table
2. **Phase 1.3:** Add Agent trait, specify shared state sync
3. **Phase 1.4:** Add backpressure handling for critical events
4. **Phase 2.3:** Add spawn_blocking for DuckDB
5. **Phase 2.4:** Add checkpoint versioning and migration
6. **Phase 3.1:** Fix approval authorization
7. **Phase 3.2:** Add WebSocket authentication
8. **Phase 3:** Clarify escalation flow
9. **Phase 4.1:** Fix priority queue race condition
