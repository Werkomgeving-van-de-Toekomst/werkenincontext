# TDD Plan: Enhanced Document Workflow

This document defines test requirements for each section of the implementation plan. Tests should be written BEFORE implementing the corresponding feature.

**Testing Framework:** Rust built-in with `#[tokio::test]` for async tests
**Test Organization:** Co-located in `tests/` directories at crate level
**Key Helpers:** `MockS3Client`, `WebSocketTestClient`, `random_document_id()`, `random_user_id()`

---

## Phase 1: Foundation & Configuration

### 1.1 Database Schema Extensions

**File:** `migrations/040_enhanced_workflow.sql`

**Tests to write:**
- Test: approval_stages table can be created and queried
- Test: delegations table enforces unique constraint on (from_user_id, to_user_id, document_types)
- Test: document_approval_stages enforces uniqueness on (document_id, stage_id)
- Test: document_approvals enforces uniqueness on (stage_instance_id, approver_id)
- Test: approval_escalations can track escalation history
- Test: document_versions extended columns (is_compressed, parent_version_id, diff_summary) are nullable
- Test: foreign key constraints prevent orphan records
- Test: indexes improve query performance for stage lookups

**Location:** `crates/iou-api/tests/database/schema_040.rs`

### 1.2 Core Type Definitions

**File:** `crates/iou-core/src/workflows/multi_stage.rs`

**Tests to write:**
- Test: ApprovalStage validates required fields (stage_id, stage_name, stage_order, approval_type)
- Test: ApprovalType variants serialize/deserialize correctly
- Test: ExpiryAction::EscalateTo includes target in serialized form
- Test: StageInstance status transitions follow valid state machine (Pending -> InProgress -> Completed)
- Test: StageInstance cannot skip from Pending to Completed
- Test: ApprovalResponse includes delegated_from when approval was delegated
- Test: Approver struct requires either user_id or role, not both
- Test: StageInstance tracks all approvals_received in order

**Location:** `crates/iou-core/tests/workflows/multi_stage_types.rs`

### 1.3 Configuration System

**File:** `crates/iou-core/src/config/workflow.rs`

**Tests to write:**
- Test: WorkflowConfig deserializes from valid YAML
- Test: StageConfig validates approval_type is one of: sequential, parallel_any, parallel_all, parallel_majority
- Test: StageConfig validates sla_hours is positive
- Test: StageConfig with is_optional=true can have empty approvers
- Test: VersionStorageConfig defaults full_versions_keep to 5
- Test: SlaConfig weekend_days accepts valid weekday names
- Test: ConfigWatcher emits event on file modification
- Test: ConfigWatcher merges domain overrides with defaults
- Test: ConfigWatcher reloads configuration after file change
- Test: Invalid YAML in config file returns error, not panic

**Location:** `crates/iou-core/tests/config/workflow_config.rs`

---

## Phase 2: Multi-Stage Approval Workflow

### 2.1 Extended State Machine

**File:** `crates/iou-orchestrator/src/state_machine/multi_stage.rs`

**Tests to write:**
- Test: transition_to_next_stage returns next sequential stage when current completes
- Test: transition_to_next_stage returns Completed state when all stages done
- Test: transition_to_next_stage preserves document_id across transitions
- Test: evaluate_stage_completion returns Complete when all required approvals received
- Test: evaluate_stage_completion returns InProgress when partial approvals received
- Test: evaluate_stage_completion returns Failed when rejection received
- Test: evaluate_stage_completion returns Expired when deadline passed
- Test: evaluate_stage_completion handles ParallelAny with single approval
- Test: evaluate_stage_completion handles ParallelAll requiring all approvers
- Test: evaluate_stage_completion handles ParallelMajority with >50% threshold
- Test: State machine rejects invalid transition (e.g., Completed -> InProgress)

**Location:** `crates/iou-orchestrator/tests/state_machine/multi_stage_transitions.rs`

### 2.2 Stage Execution Engine

**File:** `crates/iou-orchestrator/src/stage_executor.rs`

**Tests to write:**
- Test: initialize_stages creates stage instances from config for document_type
- Test: initialize_stages resolves approvers using delegation lookup
- Test: initialize_stages calculates deadlines using SLA calculator
- Test: initialize_stages skips optional stages when condition not met
- Test: start_stage updates status to InProgress and sets started_at
- Test: start_stage sends WebSocket notification to all approvers
- Test: record_approval creates ApprovalResponse record
- Test: record_approval updates stage's approvals_received list
- Test: record_approval returns Complete when quorum met
- Test: record_approval triggers next stage transition on completion
- Test: meets_quorum returns true for ParallelAny with single approval
- Test: meets_quorum returns true for ParallelAll only when all approved
- Test: meets_quorum calculates majority correctly for odd/even approver counts
- Test: record_approval rejects duplicate approval from same approver

**Location:** `crates/iou-orchestrator/tests/stages/execution.rs`

### 2.3 Delegation Resolution

**File:** `crates/iou-core/src/delegation/resolver.rs`

**Tests to write:**
- Test: resolve_approver returns original approver when no active delegations
- Test: resolve_approver returns delegated user for single-document delegation
- Test: resolve_approver returns delegated user for document_type delegation
- Test: resolve_approver prioritizes single-document over document_type delegation
- Test: resolve_approver follows delegation chain up to 3 hops
- Test: resolve_approver errors on circular delegation (A -> B -> A)
- Test: resolve_approver errors on delegation chain exceeding 3 hops
- Test: active_delegations returns only active, non-expired delegations
- Test: active_delegations excludes delegations where current time < starts_at
- Test: active_delegations excludes delegations where current time > ends_at
- Test: active_delegations includes both from_user and to_user perspectives

**Location:** `crates/iou-core/tests/delegation/resolution.rs`

### 2.4 API Endpoints

**File:** `crates/iou-api/src/routes/workflow_stages.rs`

**Tests to write:**
- Test: GET /api/documents/:id/stages returns all stages for document
- Test: GET /api/documents/:id/stages includes stage status and approvers
- Test: GET /api/documents/:id/stages/:stage_id returns detailed stage info
- Test: GET /api/documents/:id/stages/:stage_id includes approvals_received array
- Test: POST /api/documents/:id/stages/:stage_id/approve records approval
- Test: POST /api/documents/:id/stages/:stage_id/approve requires authentication
- Test: POST /api/documents/:id/stages/:stage_id/approve rejects non-approver
- Test: POST /api/documents/:id/stages/:stage_id/reject records rejection
- Test: POST /api/documents/:id/stages/:stage_id/reject prevents further approvals
- Test: POST approval returns 409 when approver has already voted
- Test: POST approval with delegation records delegated_from in audit trail

**Location:** `crates/iou-api/tests/routes/workflow_stages.rs`

---

## Phase 3: Delegation System

### 3.1 Delegation CRUD Operations

**File:** `crates/iou-core/src/delegation/service.rs`

**Tests to write:**
- Test: create_delegation creates delegation record with unique ID
- Test: create_delegation validates from_user != to_user (no self-delegation)
- Test: create_delegation validates ends_at > starts_at when provided
- Test: create_delegation detects circular delegation (A -> B -> A)
- Test: create_delegation limits total active delegations per user (configurable max)
- Test: revoke_delegation sets is_active to false
- Test: revoke_delegation creates audit trail entry
- Test: revoke_delegation only allows revocation by creator or from_user
- Test: auto_expire_delegations finds delegations past ends_at
- Test: auto_expire_delegations sets is_active to false for expired
- Test: auto_expire_delegations returns list of expired delegation IDs

**Location:** `crates/iou-core/tests/delegation/service.rs`

### 3.2 Delegation API

**File:** `crates/iou-api/src/routes/delegations.rs`

**Tests to write:**
- Test: GET /api/delegations returns user's created and received delegations
- Test: GET /api/delegations filters by is_active status
- Test: POST /api/delegations creates delegation with valid input
- Test: POST /api/delegations requires authentication
- Test: POST /api/delegations validates to_user exists
- Test: POST /api/delegations returns 400 for invalid date range
- Test: DELETE /api/delegations/:id revokes delegation
- Test: DELETE /api/delegations/:id returns 403 for non-creator
- Test: GET /api/users/:id/delegations returns user's delegations (admin only)
- Test: GET /api/users/:id/delegations returns 403 for non-admin

**Location:** `crates/iou-api/tests/routes/delegations.rs`

---

## Phase 4: Approval Expiry & Escalation

### 4.1 SLA Calculator

**File:** `crates/iou-core/src/sla/calculator.rs`

**Tests to write:**
- Test: calculate_deadline adds business_hours to start time
- Test: calculate_deadline skips Saturday and Sunday
- Test: calculate_deadline skips configured holidays
- Test: calculate_deadline handles partial days (e.g., 2 hours from Friday 4pm)
- Test: is_overdue returns true when current time > deadline
- Test: is_overdue returns false when current time <= deadline
- Test: hours_until_deadline returns positive hours for future deadline
- Test: hours_until_deadline returns negative hours for past deadline
- Test: hours_until_deadline counts only business hours
- Test: SlaCalculator configured with Dutch weekend (Sat, Sun)
- Test: SlaCalculator configured with custom weekend days

**Location:** `crates/iou-core/tests/sla/calculator.rs`

### 4.2 Escalation Service

**File:** `crates/iou-core/src/escalation/service.rs`

**Tests to write:**
- Test: check_overdue_stages finds stages approaching deadline (< 24 hours)
- Test: check_overdue_stages finds stages past deadline
- Test: check_overdue_stages respects configured escalation thresholds
- Test: send_escalation logs to approval_escalations table
- Test: send_escalation sends WebSocket notification
- Test: send_escalation sends email notification
- Test: send_escalation sends webhook notification
- Test: send_escalation updates escalation status to sent when successful
- Test: send_escalation updates escalation status to failed when retry exhausted
- Test: NotificationChannel::WebSocket sends via Supabase realtime
- Test: NotificationChannel::Email uses existing email service
- Test: NotificationChannel::Webhook handles HTTP errors and retries

**Location:** `crates/iou-core/tests/escalation/service.rs`

### 4.3 Scheduled Expiry Check

**File:** `crates/iou-orchestrator/src/jobs/expiry_checker.rs`

**Tests to write:**
- Test: ExpiryChecker runs check_and_escalate on each interval tick
- Test: check_and_escalate queries all InProgress stages
- Test: check_and_escalate calculates hours_until_deadline for each stage
- Test: check_and_escalate sends escalation for stages < threshold hours
- Test: check_and_escalate executes expiry_action for expired stages
- Test: check_and_escalate handles NotifyOnly action (only logs, no state change)
- Test: check_and_escalate handles ReturnToDraft action (returns document to draft)
- Test: check_and_escalate handles AutoApprove action (auto-approves stage)
- Test: check_and_escalate handles EscalateTo action (notifies target)
- Test: ExpiryChecker continues running after single check
- Test: ExpiryChecker handles errors without stopping loop

**Location:** `crates/iou-orchestrator/tests/jobs/expiry_checker.rs`

---

## Phase 5: Version History & Diff Visualization

### 5.1 Version Storage Service

**File:** `crates/iou-core/src/versions/service.rs`

**Tests to write:**
- Test: create_version stores document content in S3
- Test: create_version creates document_versions record with metadata
- Test: create_version sets parent_version_id to previous current version
- Test: create_version increments version number (v1, v2, v3...)
- Test: create_version compresses old versions when threshold exceeded
- Test: list_versions returns versions ordered by created_at DESC
- Test: list_versions includes version number, created_by, change_summary
- Test: restore_version fetches version content from S3
- Test: restore_version updates document with restored content
- Test: restore_version creates new version recording the restore
- Test: restore_version creates audit trail entry
- Test: restore_version requires authentication
- Test: compress_old_versions compresses versions beyond full_versions_keep
- Test: compress_old_versions sets is_compressed flag
- Test: compressed_versions can be decompressed for diff generation

**Location:** `crates/iou-core/tests/versions/service.rs`

### 5.2 Diff Generation

**File:** `crates/iou-core/src/diff/generator.rs`

**Tests to write:**
- Test: generate_diff returns DocumentDiff with from_version and to_version
- Test: generate_diff with Unified format produces git-style output
- Test: generate_diff with SideBySide format produces aligned changes
- Test: generate_diff with Inline format produces highlighted changes
- Test: DiffChange::Unchanged contains text present in both versions
- Test: DiffChange::Inserted contains text only in new version
- Test: DiffChange::Deleted contains text only in old version
- Test: DiffChange::Replaced contains both old and new text
- Test: unified_diff marks additions with + prefix
- Test: unified_diff marks deletions with - prefix
- Test: side_by_side_diff aligns unchanged lines
- Test: side_by_side_diff shows additions and deletions in parallel
- Test: inline_diff wraps changes in highlight markers
- Test: generate_diff handles empty old_content (all insertions)
- Test: generate_diff handles empty new_content (all deletions)
- Test: generate_diff handles identical content (no changes)

**Location:** `crates/iou-core/tests/diff/generator.rs`

### 5.3 Version API

**File:** `crates/iou-api/src/routes/versions.rs`

**Tests to write:**
- Test: GET /api/documents/:id/versions returns all versions for document
- Test: GET /api/documents/:id/versions includes version metadata (number, created_at, created_by)
- Test: GET /api/documents/:id/versions/diff with from and to params returns diff
- Test: GET /api/documents/:id/versions/diff defaults to comparing last two versions
- Test: GET /api/documents/:id/versions/diff supports format parameter (unified, side_by_side, inline)
- Test: GET /api/documents/:id/versions/:version_id returns single version
- Test: GET /api/documents/:id/versions/:version_id includes change_summary
- Test: POST /api/documents/:id/versions/:version_id/restore restores version
- Test: POST /api/documents/:id/versions/:version_id/restore requires authentication
- Test: POST /api/documents/:id/versions/:version_id/restore creates new version
- Test: POST restore returns 403 for non-authorized users

**Location:** `crates/iou-api/tests/routes/versions.rs`

---

## Phase 6: Frontend Components

### 6.1 Approval Queue Enhancements

**File:** `crates/iou-frontend/src/pages/approval_queue.rs`

**Tests to write (consider Dioxus testing framework):**
- Test: WorkflowStageTracker renders all stages
- Test: WorkflowStageTracker marks completed stages with checkmark
- Test: WorkflowStageTracker highlights current stage
- Test: WorkflowStageTracker shows pending stages as dimmed
- Test: Countdown timer displays hours remaining for stage deadline
- Test: Countdown timer color codes (green > 24h, yellow 8-24h, red < 8h)
- Test: Delegation badge appears next to delegated approver names
- Test: Escalation icon appears when stage has been escalated
- Test: Clicking stage expands to show approvers and their status

**Location:** `crates/iou-frontend/tests/components/approval_queue.rs` (if Dioxus testing added)

### 6.2 Diff Viewer Component

**File:** `crates/iou-frontend/src/components/diff_viewer.rs`

**Tests to write:**
- Test: DiffViewer renders diff with appropriate styling
- Test: DiffViewer shows additions in green
- Test: DiffViewer shows deletions in red
- Test: DiffViewer shows unchanged text in default color
- Test: Format toggle switches between unified, side-by-side, inline
- Test: DiffViewer handles large diffs without performance issues
- Test: DiffViewer shows "no changes" message for identical versions

**Location:** `crates/iou-frontend/tests/components/diff_viewer.rs` (if Dioxus testing added)

### 6.3 Delegation Manager

**File:** `crates/iou-frontend/src/components/delegation_manager.rs`

**Tests to write:**
- Test: DelegationManager lists active delegations
- Test: DelegationManager shows delegation type (temporary, permanent, bulk)
- Test: DelegationManager shows date range for temporary delegations
- Test: CreateDelegationForm validates to_user is required
- Test: CreateDelegationForm validates ends_at > starts_at
- Test: CreateDelegationForm allows document_types selection
- Test: Revoke button removes delegation
- Test: Revoke button requires confirmation

**Location:** `crates/iou-frontend/tests/components/delegation_manager.rs` (if Dioxus testing added)

### 6.4 Version History Component

**File:** `crates/iou-frontend/src/components/version_history.rs`

**Tests to write:**
- Test: VersionHistory lists all versions with metadata
- Test: VersionHistory shows version number, created_by, created_at
- Test: VersionHistory allows selecting two versions for comparison
- Test: Compare button opens diff viewer with selected versions
- Test: Restore button requires confirmation
- Test: Restore button shows warning about overwriting current version
- Test: Restoring creates new version (not replacing previous)

**Location:** `crates/iou-frontend/tests/components/version_history.rs` (if Dioxus testing added)

---

## Phase 7: Testing

### 7.1 Unit Tests

**File:** `crates/iou-core/tests/workflows/multi_stage.rs`

**Coverage confirmation:**
- Verify all stage transition logic has tests
- Verify all quorum evaluations have tests
- Verify SLA calculation edge cases have tests
- Verify delegation resolution paths have tests
- Verify diff generation formats have tests

### 7.2 Integration Tests

**File:** `crates/iou-api/tests/workflows/end_to_end.rs`

**Test scenarios:**
- Test: Complete multi-stage document flow (submit -> stage1 -> stage2 -> approved)
- Test: Parallel approval with quorum (3 approvers, 2 required)
- Test: Delegation during approval (original approver delegates, new approver approves)
- Test: Expiry and escalation (stage expires, escalation sent, auto-action executed)
- Test: Version creation and restoration (create doc, edit, restore previous version)
- Test: Diff between versions (create versions, compare any two)
- Test: Rejection at any stage (document rejected, returns to draft)
- Test: Optional stage skipped (condition not met, stage marked as skipped)

### 7.3 Frontend Tests

**If Dioxus testing framework is added:**
- Test: Stage tracker renders correctly for multi-stage documents
- Test: Diff viewer format toggle works
- Test: Delegation form validation prevents invalid submissions
- Test: Version history compare and restore actions

---

## Test Organization Summary

```
crates/
├── iou-core/
│   └── tests/
│       ├── workflows/
│       │   └── multi_stage_types.rs       # Phase 1.2, 2.1
│       ├── config/
│       │   └── workflow_config.rs          # Phase 1.3
│       ├── delegation/
│       │   ├── resolution.rs               # Phase 2.3
│       │   └── service.rs                  # Phase 3.1
│       ├── sla/
│       │   └── calculator.rs               # Phase 4.1
│       ├── escalation/
│       │   └── service.rs                  # Phase 4.2
│       ├── versions/
│       │   └── service.rs                  # Phase 5.1
│       └── diff/
│           └── generator.rs                # Phase 5.2
├── iou-orchestrator/
│   └── tests/
│       ├── state_machine/
│       │   └── multi_stage_transitions.rs  # Phase 2.1
│       ├── stages/
│       │   └── execution.rs                # Phase 2.2
│       └── jobs/
│           └── expiry_checker.rs           # Phase 4.3
├── iou-api/
│   └── tests/
│       ├── database/
│       │   └── schema_040.rs               # Phase 1.1
│       ├── routes/
│       │   ├── workflow_stages.rs          # Phase 2.4
│       │   ├── delegations.rs              # Phase 3.2
│       │   └── versions.rs                 # Phase 5.3
│       └── workflows/
│           └── end_to_end.rs               # Phase 7.2
└── iou-frontend/
    └── tests/                              # Phase 6 (if Dioxus testing added)
        ├── components/
        │   ├── approval_queue.rs
        │   ├── diff_viewer.rs
        │   ├── delegation_manager.rs
        │   └── version_history.rs
```

---

## Implementation Sequence with Test-First Order

**Phase 1 (Foundation):**
1. Write schema tests → Create migration
2. Write type definition tests → Add core types
3. Write config tests → Implement configuration system

**Phase 2 (Multi-Stage):**
4. Write state machine tests → Extend state machine
5. Write executor tests → Implement stage executor
6. Write resolver tests → Add delegation resolution
7. Write API tests → Create stage API endpoints

**Phase 3 (Delegation):**
8. Write service tests → Implement delegation CRUD
9. Write API tests → Create delegation API endpoints
10. Write UI tests → Add delegation UI components

**Phase 4 (Expiry & Escalation):**
11. Write SLA tests → Implement SLA calculator
12. Write escalation tests → Create escalation service
13. Write job tests → Add scheduled expiry checker
14. Write UI tests → Update approval queue with timers

**Phase 5 (Versions & Diff):**
15. Write version service tests → Implement version storage
16. Write diff generator tests → Add diff generator
17. Write API tests → Create version API endpoints
18. Write UI tests → Add diff viewer and version history

**Phase 6 (Testing & Polish):**
19. Run all unit tests → Verify coverage
20. Run integration tests → Verify end-to-end flows
21. Run performance tests → Verify large document handling
22. Run security tests → Verify authorization and audit logging
