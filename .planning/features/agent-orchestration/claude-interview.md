# Agent Orchestration - Interview Transcript

## Interview Questions & Answers

### 1. Agent Execution Model

**Q:** How should agents execute - strict sequence, parallel where possible, or dynamic dependency-based?

**A:** Hybrid/DAG - Dynamic dependency-based execution where agents can run in parallel based on their dependencies, with a DAG defining the execution graph.

### 2. Approval UX Pattern

**Q:** How should human-in-the-loop approvals work?

**A:** Inline during creation - The agent proposes output, and human can modify/reject inline during the document creation process. This provides immediate feedback rather than blocking workflows.

### 3. Checkpoint Strategy

**Q:** When should checkpoints be saved for crash recovery?

**A:** Adaptive - More frequent during long workflows, configurable per workflow type. This balances overhead with recovery granularity.

### 4. Notification Channels

**Q:** What notification channels should be used for approvals?

**A:** All available channels:
- WebSocket/SSE - Real-time browser notifications
- Email - For pending approvals
- Dashboard queue - In-app pending approval list
- Mobile push - Via existing mobile infrastructure

### 5. Error Retry Strategy

**Q:** How should agent failures be handled?

**A:** Per-agent configurable - Each agent type can have its own retry configuration (max attempts, backoff strategy), with a global cap.

### 6. Workflow Storage

**Q:** Where should workflow state be stored?

**A:** DuckDB + PostgreSQL hybrid:
- DuckDB for analytics and checkpoint snapshots
- PostgreSQL for active workflow state and queries

### 7. API Design

**Q:** What API style for workflow control?

**A:** GraphQL + WebSocket:
- GraphQL for flexible queries and mutations
- WebSocket for real-time workflow updates and approval signaling

### 8. Workflow Priority

**Q:** Should workflows support priority/queuing?

**A:** Priority queue - Workflows queued by priority, with ability to preempt lower priority workflows if needed.

### 9. Observability

**Q:** What observability features are needed?

**A:** Comprehensive observability:
- Tracing/logs - Structured logging for workflow events
- Prometheus metrics - Agent execution time, success rates
- Dashboard UI - Real-time workflow visualization
- Compliance audit - Full audit trail for regulatory requirements

### 10. Testing Focus

**Q:** What should be the primary testing focus?

**A:** All critical areas:
- State machine logic - Unit tests for transitions and guards
- Concurrency safety - loom tests for concurrent state access
- End-to-end workflows - Integration tests with real agents
- Approval UX - UI tests for human interactions

### 11. Phase 1 Scope (MVP)

**Q:** What should be in Phase 1 (MVP)?

**A:** Comprehensive MVP including:
- Core orchestration - State machine, basic agent execution
- Checkpoint/recovery - Save/restore workflow state after crashes
- Human-in-the-loop - Approval endpoints, notification system
- Workflow scheduling - Priority queue, multi-workflow execution

---

## Summary of Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Execution | Hybrid/DAG | Balance parallelism with dependencies |
| Approval | Inline during creation | Immediate feedback, better UX |
| Checkpoints | Adaptive | Balance performance with recovery |
| Notifications | All channels | Maximum reach for approvals |
| Retry | Per-agent config | Flexibility for different agent behaviors |
| Storage | DuckDB + PostgreSQL | Analytics + operational data |
| API | GraphQL + WebSocket | Flexible queries + real-time |
| Priority | Priority queue | Support for urgent workflows |
| Observability | Full stack | Monitoring + compliance |
| Testing | Comprehensive | Coverage for all critical paths |
| Phase 1 | Full scope | Complete feature set from start |

---

## Key Architecture Implications

1. **DAG-based execution** requires a dependency graph builder and executor
2. **Inline approvals** need WebSocket streaming and UI components
3. **Adaptive checkpoints** require configurable checkpoint policy
4. **Hybrid storage** needs data synchronization between DuckDB and PostgreSQL
5. **GraphQL API** requires schema design and subscription support
6. **Priority queue** requires a workflow scheduler with preemption
7. **Full observability** needs instrumentation throughout the stack

---

*Interview conducted during deep-plan session for Agent Orchestration feature*
