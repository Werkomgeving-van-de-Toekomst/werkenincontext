<!-- PROJECT_CONFIG
runtime: rust-cargo
test_command: cargo test
END_PROJECT_CONFIG -->

<!-- SECTION_MANIFEST
section-01-database-schema
section-02-core-types
section-03-config-system
section-04-multi-stage-engine
section-05-diff-generator
section-06-delegation-system
section-07-sla-escalation
section-08-version-storage
section-09-api-endpoints
section-10-frontend-components
section-11-testing-integration
END_MANIFEST -->

# Implementation Sections Index

## Dependency Graph

| Section | Depends On | Blocks | Parallelizable |
|---------|------------|--------|----------------|
| section-01-database-schema | - | 02, 03, 04, 05 | No |
| section-02-core-types | 01 | 03, 04 | Yes |
| section-03-config-system | 01, 02 | 04 | No |
| section-04-multi-stage-engine | 01, 02, 03 | 06, 07, 09 | No |
| section-05-diff-generator | 01, 02 | 08 | Yes |
| section-06-delegation-system | 04 | 07, 09 | No |
| section-07-sla-escalation | 04, 06 | 09 | No |
| section-08-version-storage | 05 | 09 | No |
| section-09-api-endpoints | 04, 06, 07, 08 | 10 | No |
| section-10-frontend-components | 09 | 11 | No |
| section-11-testing-integration | All | - | No |

## Execution Order

**Batch 1** (Foundation):
1. section-01-database-schema (no dependencies)

**Batch 2** (Core types):
2. section-02-core-types (after 01)

**Batch 3** (Parallel foundation):
3. section-03-config-system (after 01, 02)
4. section-04-multi-stage-engine (after 01, 02)
5. section-05-diff-generator (after 01, 02)

**Batch 4** (Workflow extensions):
6. section-06-delegation-system (after 04)
7. section-07-sla-escalation (after 04, 06)

**Batch 8** (Version system):
8. section-08-version-storage (after 05)

**Batch 9** (API layer):
9. section-09-api-endpoints (after 04, 06, 07, 08)

**Batch 10** (Frontend):
10. section-10-frontend-components (after 09)

**Batch 11** (Testing):
11. section-11-testing-integration (after all backend + API)

## Section Summaries

### section-01-database-schema
Database migration script creating new tables for multi-stage workflows: `approval_stages`, `delegations`, `document_approval_stages`, `document_approvals`, `approval_escalations`, and extensions to `document_versions`. Includes indexes and foreign key constraints.

**Files:**
- `migrations/040_enhanced_workflow.sql`
- Tests in `crates/iou-api/tests/database/schema_040.rs`

### section-02-core-types
Rust type definitions for multi-stage workflows, delegation, and version tracking. Includes `ApprovalStage`, `StageInstance`, `Delegation`, `ApprovalType`, `ExpiryAction`, and related enums/structs.

**Files:**
- `crates/iou-core/src/workflows/multi_stage.rs`
- `crates/iou-core/src/delegation.rs`
- Tests in `crates/iou-core/tests/workflows/multi_stage_types.rs`

### section-03-config-system
Configuration loading and hot-reload for workflow definitions. Uses YAML files for stage definitions, approvers, SLAs, and expiry actions. Implements file watcher for runtime config updates.

**Files:**
- `crates/iou-core/src/config/workflow.rs`
- `crates/iou-core/src/config/watcher.rs`
- `config/workflows/defaults.yaml`
- Tests in `crates/iou-core/tests/config/workflow_config.rs`

### section-04-multi-stage-engine
Extended state machine and stage executor for multi-stage approvals. Handles sequential progression, parallel approvals (any/all/majority), stage completion evaluation, and transitions between stages.

**Files:**
- `crates/iou-orchestrator/src/state_machine/multi_stage.rs`
- `crates/iou-orchestrator/src/stage_executor.rs`
- Tests in `crates/iou-orchestrator/tests/state_machine/multi_stage_transitions.rs`
- Tests in `crates/iou-orchestrator/tests/stages/execution.rs`

### section-05-diff-generator
Text diff generation using the `similar` crate. Supports unified, side-by-side, and inline diff formats. Computes changes between document versions for visualization.

**Files:**
- `crates/iou-core/src/diff/generator.rs`
- Tests in `crates/iou-core/tests/diff/generator.rs`

### section-06-delegation-system
Delegation CRUD operations and resolution logic. Supports temporary (date-range), permanent, and bulk delegations. Resolves actual approvers considering active delegations, prevents circular chains.

**Files:**
- `crates/iou-core/src/delegation/resolver.rs`
- `crates/iou-core/src/delegation/service.rs`
- Tests in `crates/iou-core/tests/delegation/resolution.rs`
- Tests in `crates/iou-core/tests/delegation/service.rs`

### section-07-sla-escalation
SLA calculator (business hours with weekend/holiday skipping) and escalation service. Scheduled job checks for approaching/past deadlines, sends notifications via WebSocket/email/webhook, executes expiry actions.

**Files:**
- `crates/iou-core/src/sla/calculator.rs`
- `crates/iou-core/src/escalation/service.rs`
- `crates/iou-orchestrator/src/jobs/expiry_checker.rs`
- Tests in `crates/iou-core/tests/sla/calculator.rs`
- Tests in `crates/iou-core/tests/escalation/service.rs`
- Tests in `crates/iou-orchestrator/tests/jobs/expiry_checker.rs`

### section-08-version-storage
Version storage service with S3/MinIO backend. Creates version records on document changes, compresses old versions, supports restoration with audit trail. Implements version numbering and parent tracking.

**Files:**
- `crates/iou-core/src/versions/service.rs`
- Tests in `crates/iou-core/tests/versions/service.rs`

### section-09-api-endpoints
HTTP API routes for workflow stages, delegations, and versions. Endpoints for listing/approving/rejecting stages, managing delegations, comparing/restoring document versions.

**Files:**
- `crates/iou-api/src/routes/workflow_stages.rs`
- `crates/iou-api/src/routes/delegations.rs`
- `crates/iou-api/src/routes/versions.rs`
- Tests in `crates/iou-api/tests/routes/workflow_stages.rs`
- Tests in `crates/iou-api/tests/routes/delegations.rs`
- Tests in `crates/iou-api/tests/routes/versions.rs`

### section-10-frontend-components
Dioxus WASM components for enhanced UI: workflow stage tracker, diff viewer, delegation manager, version history. Updates approval queue with stage progress, countdown timers, and escalation indicators.

**Files:**
- `crates/iou-frontend/src/pages/approval_queue.rs` (modified)
- `crates/iou-frontend/src/components/workflow_stage_tracker.rs`
- `crates/iou-frontend/src/components/diff_viewer.rs`
- `crates/iou-frontend/src/components/delegation_manager.rs`
- `crates/iou-frontend/src/components/version_history.rs`
- Tests in `crates/iou-frontend/tests/components/` (if Dioxus testing added)

### section-11-testing-integration
End-to-end integration tests covering complete workflows: multi-stage approval, delegation, expiry/escalation, version restoration, and diff generation. Performance and security tests.

**Files:**
- `crates/iou-api/tests/workflows/end_to_end.rs`
- Performance and security test modules
