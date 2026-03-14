# Interview Transcript: Agent Orchestration with Human-in-the-Loop

## Date: 2026-03-10

---

## Q1: Voorkeursarchitectuur voor agent orchestratie?

**Answer:** Rust-native (aanbevolen)

**Rationale:** User prefers full Rust implementation using Tokio + smlang/cqrs-es patterns over LangGraph integration. This avoids Python bridge complexity and keeps everything in a single codebase.

**Implications:**
- Use Tokio for async runtime
- Use smlang for state machine DSL
- Implement cqrs-es pattern for event sourcing
- Native integration with existing IOU-Modern codebase

---

## Q2: Hoe moet menselijke tussenkomst werken?

**Answer:** Per agent checkpoint

**Rationale:** Users want to approve/review each agent's output before proceeding to the next agent, not just a single approval at the end.

**Implications:**
- Implement breakpoints after each agent
- UI must show agent-specific results
- Human can modify/annotate agent output
- Workflow state must persist across human interaction

---

## Q3: Hoeveel document creatie workflows verwacht je tegelijk?

**Answer:** 1-10 documenten

**Rationale:** Small scale, enterprise team usage (not mass public usage).

**Implications:**
- Simple in-memory state management sufficient
- No distributed locking needed
- Simple queue system (if any)
- Can use single-worker model

---

## Q4: Moeten agents parallel kunnen werken?

**Answer:** Ja waar mogelijk

**Rationale:** Some agents can run in parallel (e.g., compliance check during content generation).

**Implications:**
- Need to identify agent dependencies
- Implement parallel execution paths in state machine
- Use Tokio tasks for concurrent agent execution
- Channel-based communication between parallel agents

---

## Q5: Waar moet workflow state worden opgeslagen?

**Answer:** In-memory primary

**Rationale:** For 1-10 concurrent workflows, in-memory state is sufficient with periodic persistence to DuckDB for recovery.

**Implications:**
- Use in-memory state machine with periodic checkpoints
- Persist to DuckDB for crash recovery
- Simpler architecture than full event sourcing

---

## Q6: Wat moet er gebeuren als een agent faalt?

**Answer:** Auto retry

**Rationale:** Use existing retry configuration (max_retries in PipelineConfig) before escalating.

**Implications:**
- Preserve existing ErrorSeverity classification
- Implement exponential backoff for transient errors
- Fail permanently after max retries
- Optional: alert human after permanent failure

---

## Q7: Hoe omgaan met langlopende agent taken?

**Answer:** Configureerbaar

**Rationale:** Different document types may need different timeouts (simple letter vs complex legal document).

**Implications:**
- Add per-document-type timeout configuration
- Support agent-specific timeouts
- Implement Tokio timeout handling
- UI should show progress for long-running tasks

---

## Summary of Key Requirements

1. **Rust-native orchestration** using Tokio + smlang
2. **Per-agent human checkpoints** with review/edit capability
3. **Small scale** (1-10 concurrent workflows)
4. **Parallel agent execution** where possible
5. **In-memory state** with DuckDB persistence for recovery
6. **Auto-retry** with existing error classification
7. **Configurable timeouts** per document type
