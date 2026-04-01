<!-- PROJECT_CONFIG
runtime: rust-cargo
test_command: cargo test
END_PROJECT_CONFIG -->

<!-- SECTION_MANIFEST
section-01-state-machine
section-02-parallel-executor
section-03-checkpoint-recovery
section-04-graphql-api
section-05-websocket-notifications
section-06-workflow-scheduler
section-07-observability
section-08-integration-tests
END_MANIFEST -->

# Implementation Sections Index

## Dependency Graph

| Section | Depends On | Blocks | Parallelizable |
|---------|------------|--------|----------------|
| section-01-state-machine | - | section-02 | Yes |
| section-02-parallel-executor | 01 | section-03 | No |
| section-03-checkpoint-recovery | 02 | section-04, section-05 | Yes |
| section-04-graphql-api | 03 | section-08 | Yes |
| section-05-websocket-notifications | 03 | section-08 | Yes |
| section-06-workflow-scheduler | 02 | section-08 | No |
| section-07-observability | 02 | section-08 | Yes |
| section-08-integration-tests | 03, 04, 05, 06, 07 | - | No |

## Execution Order

1. **section-01-state-machine** (no dependencies)
2. **section-02-parallel-executor** (after 01)
3. **section-03-checkpoint-recovery** (after 02)
4. **section-04-graphql-api, section-05-websocket-notifications, section-06-workflow-scheduler, section-07-observability** (parallel after 03)
5. **section-08-integration-tests** (after all others)

## Section Summaries

### section-01-state-machine
Enhance existing state machine with DAG-specific states (ParallelExecuting, PartialComplete, MergeResults), state transition table, DAG builder from agent dependencies, and guard functions for parallel execution validation.

### section-02-parallel-executor
Agent trait definition, parallel agent executor with Tokio JoinSet, shared state synchronization using Arc<RwLock<WorkflowContext>>, cancellation token pattern, event bus with broadcast and bounded mpsc channels for critical events.

### section-03-checkpoint-recovery
Adaptive checkpoint policy, PostgreSQL checkpoint storage with async sqlx, checkpoint data structures, recovery workflow with version validation and migration strategy for incompatible checkpoints.

### section-04-graphql-api
GraphQL schema definitions for Workflow, ApprovalRequest, mutations (createWorkflow, approve, modify, reject), queries, subscriptions, authentication integration (approverId from JWT context), DataLoader for batch loading to prevent N+1 queries.

### section-05-websocket-notifications
WebSocket server with JWT authentication before upgrade, real-time workflow updates and approval notifications, multi-channel notification system (email, Slack, FCM), escalation configuration with timeout chains and role-based authorization.

### section-06-workflow-scheduler
Priority queue implementation with fixed TOCTOU race condition, Ord implementation for CRITICAL > HIGH > NORMAL > LOW ordering, workflow scheduler with preemption support, graceful checkpoint-before-preempt, resume from checkpoint.

### section-07-observability
Structured logging with tracing crate, Prometheus metrics (counters, gauges, histograms) for workflow lifecycle, audit logging to PostgreSQL with append-only constraint, critical events via bounded channel with backpressure handling.

### section-08-integration-tests
End-to-end workflow execution tests, crash recovery simulation, parallel execution with dependencies, human-in-the-loop approval with modification, escalation flow, priority queue ordering, preemption scenarios, property-based tests for state machine and DAG builder.
