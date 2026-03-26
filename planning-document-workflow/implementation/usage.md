# IOU-Modern Workflow System: Usage Guide

## Overview

This guide covers the complete multi-stage approval workflow system implemented across 11 sections. The system provides:

- **Multi-stage approval workflows** with sequential, parallel (any/all/majority) approvals
- **Delegation system** with temporary, permanent, and bulk delegations
- **SLA monitoring** with business hours calculation and automatic escalation
- **Version control** with diff generation and restoration capabilities
- **HTTP API** for all workflow operations
- **Frontend components** for Dioxus WASM UI

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (Dioxus)                        │
│  ┌──────────────┐ ┌─────────────┐ ┌─────────────────────────┐  │
│  │ Stage Tracker│ │ Diff Viewer │ │ Version History        │  │
│  └──────────────┘ └─────────────┘ └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         HTTP API (Axum)                          │
│  /api/workflow/stages  /api/delegations  /api/versions          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Orchestrator (State Machine)                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │
│  │ Stage Executor   │  │ SLA Calculator   │  │ Escalation   │  │
│  └──────────────────┘  └──────────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Core Domain                              │
│  Workflows  Delegations  Diff  Versions  Config                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Storage (PostgreSQL + S3)                   │
└─────────────────────────────────────────────────────────────────┘
```

## Database Schema

### Key Tables

| Table | Purpose |
|-------|---------|
| `approval_stages` | Workflow stage definitions |
| `delegations` | Delegation records |
| `document_approval_stages` | Document-stage relationships |
| `document_approvals` | Individual approval decisions |
| `approval_escalations` | Escalation history |
| `document_versions` | Version history with S3 references |

### Migration

```bash
# Apply the workflow schema migration
psql -d iou_db -f migrations/040_enhanced_workflow.sql
```

## Core Types

### ApprovalType

```rust
pub enum ApprovalType {
    Sequential,      // Approvers in sequence
    ParallelAny,     // Any one approver
    ParallelAll,     // All approvers required
    ParallelMajority // Majority of approvers
}
```

### StageStatus

```rust
pub enum StageStatus {
    Pending,     // Not yet started
    InProgress,  // Actively being approved
    Completed,   // Approved successfully
    Rejected,    // Rejected by quorum
    Expired,     // SLA deadline passed
    Escalated,   // Escalated to higher authority
    Cancelled,   // Workflow cancelled
    Skipped,     // Skipped via business rule
}
```

### ExpiryAction

```rust
pub enum ExpiryAction {
    AutoApprove,   // Automatically approve on expiry
    AutoReject,    // Automatically reject on expiry
    Escalate,      // Escalate to backup approvers
    Hold,          // Hold for manual review
}
```

## Configuration

### Workflow YAML

```yaml
# config/workflows/defaults.yaml
workflows:
  standard_approval:
    stages:
      - name: "manager_review"
        display_name: "Manager Review"
        type: "sequential"
        approvers:
          - role: "manager"
        sla_hours: 24
        expiry_action: "escalate"
        escalation_targets:
          - role: "director"

      - name: "director_approval"
        display_name: "Director Approval"
        type: "parallel_all"
        approvers:
          - role: "director"
        sla_hours: 48
        expiry_action: "hold"
```

### Hot-Reload Configuration

```rust
use iou_core::config::watcher::WorkflowWatcher;

// Watch for config changes (auto-reloads)
let watcher = WorkflowWatcher::new("/etc/iou/workflows")?;
watcher.start().await?;
```

## HTTP API

### Workflow Stages

```bash
# List stages for a document
GET /api/documents/{id}/stages

# Approve a stage
POST /api/documents/{id}/stages/{stage_id}/approvals
{
  "decision": "approved",
  "comment": "Looks good"
}

# Reject a stage
POST /api/documents/{id}/stages/{stage_id}/approvals
{
  "decision": "rejected",
  "comment": "Needs revision"
}

# Get stage status
GET /api/documents/{id}/stages/{stage_id}
```

### Delegations

```bash
# Create delegation
POST /api/delegations
{
  "delegator": "user-123",
  "delegate": "user-456",
  "start_date": "2026-03-24",
  "end_date": "2026-03-31"
}

# List active delegations
GET /api/delegations?user=user-123&status=active

# Revoke delegation
DELETE /api/delegations/{id}
```

### Versions

```bash
# List versions
GET /api/documents/{id}/versions

# Compare versions
GET /api/documents/{id}/versions/compare?from=v1&to=v2

# Restore version
POST /api/documents/{id}/versions/{version_id}/restore
```

## Frontend Components

### WorkflowStageTracker

```rust
use iou_frontend::components::WorkflowStageTracker;

<WorkflowStageTracker
    document_id={document_id}
    stages={stages}
    on_update={|status| println!("Stage: {:?}", status)}
/>
```

### DiffViewer

```rust
use iou_frontend::components::DiffViewer;

<DiffViewer
    old_content={old_version}
    new_content={new_version}
    format={DiffFormat::Unified}
/>
```

### DelegationManager

```rust
use iou_frontend::components::DelegationManager;

<DelegationManager
    user_id={current_user.id}
    delegations={user_delegations}
/>
```

## Running Tests

### All Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_complete_multi_stage_document_flow
```

### Integration Tests

```bash
# End-to-end workflow tests
cargo test --test workflow_integration

# Security tests
cargo test --test approval_bypass

# Performance tests (requires release mode)
cargo test --test workflow_load --release
```

### Coverage Verification

```bash
cargo test --package iou-core --lib workflows
```

## Performance Benchmarks

| Metric | Threshold |
|--------|-----------|
| 100 documents processed | < 5 seconds |
| 10,000 line diff generation | < 100ms |
| 100 version pagination | < 10ms |
| Stage transition overhead | < 1000μs |

## Deployment

### Environment Variables

```bash
# Database
DATABASE_URL=postgresql://user:pass@host/iou_db

# S3/MinIO (for version storage)
S3_ENDPOINT=http://minio:9000
S3_BUCKET=iou-versions
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin

# Workflow config
WORKFLOW_CONFIG_PATH=/etc/iou/workflows

# SLA Settings
BUSINESS_HOURS_START=09:00
BUSINESS_HOURS_END=17:00
BUSINESS_DAYS=1-5
```

### Running the API Server

```bash
# Build and run
cargo run --bin iou-api

# With custom config
WORKFLOW_CONFIG_PATH=/etc/iou/workflows cargo run --bin iou-api
```

## Implementation Summary

| Section | Description | Files |
|---------|-------------|-------|
| 01 | Database schema | `migrations/040_enhanced_workflow.sql` |
| 02 | Core types | `crates/iou-core/src/workflows/multi_stage.rs` |
| 03 | Config system | `crates/iou-core/src/config/workflow.rs` |
| 04 | Multi-stage engine | `crates/iou-orchestrator/src/state_machine/` |
| 05 | Diff generator | `crates/iou-core/src/diff/generator.rs` |
| 06 | Delegation system | `crates/iou-core/src/delegation/` |
| 07 | SLA escalation | `crates/iou-core/src/sla/`, `crates/iou-orchestrator/src/jobs/` |
| 08 | Version storage | `crates/iou-core/src/versions/service.rs` |
| 09 | API endpoints | `crates/iou-api/src/routes/` |
| 10 | Frontend components | `crates/iou-frontend/src/components/` |
| 11 | Testing | `crates/iou-api/tests/`, `crates/iou-core/tests/` |

## Commit History

| Commit | Section |
|--------|---------|
| 449032b | section-01-database-schema |
| 57a4433 | section-02-core-types |
| 525632c | section-03-config-system |
| 7fbc39c | section-04-multi-stage-engine |
| 26c8e96 | section-05-diff-generator |
| 968b2c7 | section-06-delegation-system |
| 36561a5 | section-07-sla-escalation |
| 58af6d0 | section-08-version-storage |
| 2bdeac2 | section-09-api-endpoints |
| 8921490 | section-10-frontend-components |
| 662ef24 | section-11-testing-integration |

## Next Steps

1. **Deploy database migration**: Apply `migrations/040_enhanced_workflow.sql`
2. **Configure workflows**: Set up YAML files in `config/workflows/`
3. **Run tests**: Verify all tests pass in your environment
4. **Start the API**: Launch the API server with proper configuration
5. **Build frontend**: Compile Dioxus components for WASM deployment
