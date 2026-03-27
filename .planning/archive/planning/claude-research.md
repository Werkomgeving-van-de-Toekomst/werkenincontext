# Research Report: Agent Orchestration with Human-in-the-Loop

## Date: 2026-03-10

## 1. Codebase Analysis - Existing IOU-Modern Agents

### Current Architecture

The agent system in `crates/iou-ai/src/agents/` uses a **pipeline-based architecture** with four specialized agents:

1. **Research Agent** - Queries GraphRAG, extracts structure patterns
2. **Content Agent** - Generates document content using Markdown templates
3. **Compliance Agent** - Validates against Dutch regulations (Woo), detects PII
4. **Review Agent** - Quality assurance and approval logic

### What Works Well

- **Solid error handling** with severity classification (Transient vs Permanent)
- **Modular agent design** with clean interfaces
- **Configuration flexibility** with sensible defaults
- **Good test coverage** for core logic
- **Pipeline checkpoints** structure (though storage is TODO)

### Critical Limitations for Orchestration

1. **No human-in-the-middle support** - Only approval/rejection at the very end
2. **Synchronous pipeline flow** - Cannot pause/resume between agents
3. **No persistence** - Pipeline results are in-memory only
4. **No API integration** - Agents aren't called from the API layer
5. **Limited event handling** - No event-driven architecture for UI integration

### Code Patterns to Preserve

```rust
// Error classification pattern - excellent
pub enum ErrorSeverity {
    Transient,  // Retry with exponential backoff
    Permanent,  // Fail immediately
}

// Modular configuration
pub struct PipelineConfig {
    pub max_iterations: usize,
    pub max_retries: u32,
    pub enable_checkpoints: bool,
}

// Rich context passing between agents
pub struct ResearchContext {
    pub mandatory_sections: Vec<String>,
    pub suggested_variables: Vec<TemplateVariableSuggestion>,
}
```

### Required Changes for Human-in-the-Loop

1. Add intermediate states: `AwaitingHumanInput`, `HumanReviewInProgress`
2. Support partial document updates and section-level reviews
3. Implement workflow state persistence
4. Add event-driven architecture for UI integration
5. Implement proper storage for pipeline checkpoints

---

## 2. LangGraph Architecture

### Core Concepts

LangGraph is a **low-level orchestration framework** for building stateful agents:

- **StateGraph** - Graph-based execution where nodes process state
- **State** - TypedDict that flows through the graph
- **Conditional edges** - Dynamic routing based on state
- **Checkpointing** - State persistence for durable execution

### Human-in-the-Loop Support

1. **State Inspection** - Inspect agent state at any point
2. **State Modification** - Humans can modify state before resuming
3. **Checkpointing** - Pause/resume patterns with persistent state

### Integration Requirements

- Python service layer for LangGraph execution
- Communication bridge to Rust API (gRPC or HTTP)
- LangSmith for debugging and observability

**Source:** [LangGraph GitHub](https://github.com/langchain-ai/langgraph)

---

## 3. Rust Async Orchestration

### Tokio Runtime

The standard async runtime in Rust:

- `tokio::task` - Task spawning with `spawn` and `JoinHandle`
- `tokio::sync` - Channels: `mpsc`, `oneshot`, `watch`, `broadcast`
- `tokio::time` - Timeouts and intervals

### Actor Model with Actix

Built on Tokio, provides:
- Actor-based orchestration
- Message-based communication
- Supervision for fault tolerance
- Typed messages (no `Any` type)

### State Machine Libraries

1. **smlang** - DSL for state machines with async support
2. **rustfsm** - Lightweight finite state transducers
3. **cqrs-es** - Event sourcing with CQRS pattern

### Observability

- **tracing** crate - Structured logging for async systems
- **loom** - Deterministic testing of concurrent programs

---

## 4. Human-in-the-Loop Patterns

### Breakpoint Patterns

1. **Checkpoint-Based Pausing** - Serialize state at specific points
2. **Interrupt Handling** - Agents emit events when human input needed

### Resume Patterns

1. **State Mutation** - Human modifies agent state directly
2. **Event Injection** - Human input converted to event in stream

### Audit Trail Requirements

- State snapshots before/after intervention
- Decision logging with timestamps and user identity
- Reasoning capture for decisions
- Lineage tracking of decision impact

### UI Patterns

1. **Approval Queues** - Pending decisions with priority ordering
2. **State Diff Visualization** - Show proposed changes
3. **Breakpoint Dashboard** - Real-time view of paused agents

---

## Recommendations

### Architecture Decision: Rust-Native vs LangGraph

| Factor | LangGraph | Rust-Native |
|--------|-----------|-------------|
| Integration | Requires Python bridge | Native Rust integration |
| Dependencies | Python ecosystem | Rust ecosystem |
| Complexity | Higher (multi-language) | Lower (single language) |
| Maintenance | Two codebases | Single codebase |
| Features | Rich built-in HITL | Build custom HITL |

**Recommendation:** Rust-native implementation using:
- **Tokio** for async runtime
- **Actix** or custom channels for agent communication
- **smlang** for state machine definition
- **cqrs-es** pattern for event sourcing and audit trail

### Key Implementation Priorities

1. Implement state machine for document workflow
2. Add persistent checkpoint system
3. Create event-driven architecture for UI integration
4. Build approval queue system
5. Implement comprehensive audit logging
