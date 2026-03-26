# Implementation Plan: Enhanced Document Workflow

## Overview

This plan extends the IOU-Modern document workflow system with four major features: multi-stage approvals, delegation, approval expiry with escalation, and version history with diff visualization. The implementation builds on the existing workflow state machine, audit trail system, and dual database architecture (DuckDB + Supabase).

**Context for Implementation:**
- Existing workflow engine is in `iou-orchestrator/src/state_machine.rs` and `iou-core/src/workflows.rs`
- Current states: Draft → Submitted → InReview → Approved → Published
- Database uses DuckDB for embedded analytics and Supabase PostgreSQL for realtime
- Approval queue UI is at `/documenten/wachtrij` (Dutch interface)

**Backward Compatibility Requirement:** All changes must preserve existing single-stage workflow behavior without modification.

---

## Phase 1: Foundation & Configuration

### 1.1 Database Schema Extensions

Create new tables to support multi-stage workflows, delegation, and escalation tracking.

**File:** `migrations/040_enhanced_workflow.sql`

**New Tables:**

```sql
-- Approval stage definitions (configured, not per-document)
CREATE TABLE approval_stages (
    id VARCHAR(50) PRIMARY KEY,
    domain_id VARCHAR(50),
    document_type VARCHAR(100),
    stage_name VARCHAR(200) NOT NULL,
    stage_order INTEGER NOT NULL,
    approval_type VARCHAR(20) NOT NULL,
    approvers JSONB NOT NULL,
    sla_hours INTEGER NOT NULL DEFAULT 72,
    expiry_action VARCHAR(50) NOT NULL DEFAULT 'notify_only',
    is_optional BOOLEAN DEFAULT false,
    condition TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Active delegations
CREATE TABLE delegations (
    id UUID PRIMARY KEY,
    from_user_id UUID NOT NULL,
    to_user_id UUID NOT NULL,
    delegation_type VARCHAR(20) NOT NULL,
    document_types JSONB DEFAULT '[]',
    document_id UUID,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL
);

-- Approval stage instances (per-document state tracking)
CREATE TABLE document_approval_stages (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id),
    stage_id VARCHAR(50) NOT NULL,
    stage_status VARCHAR(20) NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    deadline TIMESTAMPTZ,
    approvers JSONB NOT NULL,
    approvals_received JSONB DEFAULT '[]',
    UNIQUE(document_id, stage_id)
);

-- Individual approval responses
CREATE TABLE document_approvals (
    id UUID PRIMARY KEY,
    stage_instance_id UUID NOT NULL REFERENCES document_approval_stages(id),
    approver_id UUID NOT NULL,
    delegated_from UUID,
    decision VARCHAR(20),
    comment TEXT,
    responded_at TIMESTAMPTZ,
    UNIQUE(stage_instance_id, approver_id)
);

-- Escalation log
CREATE TABLE approval_escalations (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id),
    stage_instance_id UUID REFERENCES document_approval_stages(id),
    escalation_type VARCHAR(50) NOT NULL,
    notification_channel VARCHAR(50) NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL,
    acknowledged_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'pending'
);

-- Extend existing document_versions table
ALTER TABLE document_versions ADD COLUMN is_compressed BOOLEAN DEFAULT false;
ALTER TABLE document_versions ADD COLUMN parent_version_id UUID REFERENCES document_versions(id);
ALTER TABLE document_versions ADD COLUMN diff_summary JSONB;
```

**Indexes:**
- `approval_stages`: (domain_id, document_type) lookup
- `delegations`: (from_user_id, is_active) for active delegation lookup
- `document_approval_stages`: (document_id) and (stage_status) for workflow queries
- `document_approvals`: (stage_instance_id) for approval aggregation
- `approval_escalations`: (document_id) for escalation history

### 1.2 Core Type Definitions

**File:** `crates/iou-core/src/workflows/multi_stage.rs` (new)

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents an approval stage definition
pub struct ApprovalStage {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    pub approval_type: ApprovalType,
    pub approvers: Vec<Approver>,
    pub sla_hours: i32,
    pub expiry_action: ExpiryAction,
    pub is_optional: bool,
    pub condition: Option<String>,
}

pub struct Approver {
    pub user_id: Option<Uuid>,
    pub role: Option<String>,
}

pub enum ApprovalType {
    Sequential,
    ParallelAny,
    ParallelAll,
    ParallelMajority,
}

pub enum ExpiryAction {
    NotifyOnly,
    ReturnToDraft,
    AutoApprove,
    EscalateTo { target: String },
}

/// Per-document stage instance state
pub struct StageInstance {
    pub id: Uuid,
    pub document_id: Uuid,
    pub stage_id: String,
    pub status: StageStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub approvers: Vec<Uuid>,
    pub approvals_received: Vec<ApprovalResponse>,
}

pub enum StageStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
    Expired,
}

pub struct ApprovalResponse {
    pub approver_id: Uuid,
    pub delegated_from: Option<Uuid>,
    pub decision: ApprovalDecision,
    pub comment: Option<String>,
    pub responded_at: DateTime<Utc>,
}

pub enum ApprovalDecision {
    Approved,
    Rejected,
    Delegated { to: Uuid },
}
```

**File:** `crates/iou-core/src/delegation.rs` (new)

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct Delegation {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub delegation_type: DelegationType,
    pub document_types: Vec<String>,
    pub document_id: Option<Uuid>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

pub enum DelegationType {
    Temporary,
    Permanent,
    Bulk,
}
```

### 1.3 Configuration System

**File:** `crates/iou-core/src/config/workflow.rs` (new)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowConfig {
    pub approval_stages: Vec<StageConfig>,
    pub version_storage: VersionStorageConfig,
    pub sla: SlaConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StageConfig {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    pub approval_type: ApprovalTypeConfig,
    pub approvers: Vec<ApproverConfig>,
    pub sla_hours: i32,
    pub expiry_action: String,
    pub is_optional: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionStorageConfig {
    pub full_versions_keep: i32,
    pub compress_after_days: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlaConfig {
    pub weekends: Vec<String>,
    pub escalation_hours: Vec<i32>,
}
```

**Configuration directory structure:**
```
config/
└── workflows/
    ├── defaults.yaml           # Global defaults
    └── domains/
        └── {domain_id}.yaml    # Per-domain overrides
```

**File:** `crates/iou-core/src/config/watcher.rs` (new)

```rust
use notify::{Watcher, RecursiveMode, EventKind};
use std::path::Path;

pub struct ConfigWatcher {
    config_path: &'static Path,
}

impl ConfigWatcher {
    /// Start watching configuration files for changes
    pub fn watch(&self) -> impl Stream<Item = ConfigChangeEvent> {
        // Use notify crate to watch config directory
        // Emit events when files change
    }
    
    /// Load configuration from files
    pub fn load_config(&self, domain_id: &str) -> WorkflowConfig {
        // Merge defaults with domain override
    }
}

pub enum ConfigChangeEvent {
    Updated { domain_id: String },
    Created { domain_id: String },
    Deleted { domain_id: String },
}
```

---

## Phase 2: Multi-Stage Approval Workflow

### 2.1 Extended State Machine

**File:** `crates/iou-orchestrator/src/state_machine/multi_stage.rs` (new)

```rust
use crate::state_machine::WorkflowState;

/// Extended state transitions for multi-stage approval
pub fn transition_to_next_stage(
    current: WorkflowState,
    stage_completed: StageInstance,
) -> WorkflowTransition {
    // Determine next state based on stage completion
    // Handle sequential stage progression
}

pub fn evaluate_stage_completion(
    stage: &StageInstance,
) -> StageCompletionStatus {
    // Check if all required approvals received
    // Evaluate quorum for parallel stages
}

pub enum StageCompletionStatus {
    Complete,
    InProgress,
    Failed,
    Expired,
}
```

### 2.2 Stage Execution Engine

**File:** `crates/iou-orchestrator/src/stage_executor.rs` (new)

```rust
use crate::state_machine::multi_stage::StageInstance;

pub struct StageExecutor;

impl StageExecutor {
    /// Initialize stage instances for a submitted document
    pub async fn initialize_stages(
        &self,
        document_id: Uuid,
        domain_id: &str,
        document_type: &str,
    ) -> Result<Vec<StageInstance>> {
        // Load applicable stages from config
        // Resolve approvers (check for delegations)
        // Create stage instances with calculated deadlines
    }
    
    /// Start a stage (mark as in_progress, notify approvers)
    pub async fn start_stage(
        &self,
        stage: &StageInstance,
    ) -> Result<()> {
        // Update status, send notifications via WebSocket
    }
    
    /// Record an approval decision
    pub async fn record_approval(
        &self,
        stage_id: Uuid,
        approver_id: Uuid,
        decision: ApprovalDecision,
        comment: Option<String>,
    ) -> Result<StageCompletionStatus> {
        // Record approval, check if stage complete
        // If complete, transition to next stage or final state
    }
    
    /// Check if stage meets quorum requirements
    fn meets_quorum(&self, stage: &StageInstance) -> bool {
        // Evaluate based on approval type (any/all/majority)
    }
}
```

### 2.3 Delegation Resolution

**File:** `crates/iou-core/src/delegation/resolver.rs` (new)

```rust
use crate::delegation::Delegation;

pub struct DelegationResolver;

impl DelegationResolver {
    /// Find the actual approver for a user, considering active delegations
    pub async fn resolve_approver(
        &self,
        original_approver: Uuid,
        document_type: &str,
        document_id: Option<Uuid>,
    ) -> Result<Uuid> {
        // Check for active delegations
        // Priority: single-document > per-type > bulk
        // Follow delegation chains (max 3 hops)
    }
    
    /// Get all active delegations for a user
    pub async fn active_delegations(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Delegation>> {
        // Query delegations table for active, non-expired
    }
}
```

### 2.4 API Endpoints

**File:** `crates/iou-api/src/routes/workflow_stages.rs` (new)

```rust
use axum::{Router, Json, extract::{Path, State}};

pub fn workflow_stages_router() -> Router {
    Router::new()
        .route("/api/documents/:id/stages", get(get_stages))
        .route("/api/documents/:id/stages/:stage_id", get(get_stage))
        .route("/api/documents/:id/stages/:stage_id/approve", post(approve_stage))
        .route("/api/documents/:id/stages/:stage_id/reject", post(reject_stage))
}

/// Get all approval stages for a document
async fn get_stages(
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<StageView>>> {
    // Return all stages with current status
}

/// Get detailed status of a specific stage
async fn get_stage(
    Path((id, stage_id)): Path<(Uuid, String)>,
) -> Result<Json<StageDetailView>> {
    // Return stage details including approvals received
}

/// Approve a stage
async fn approve_stage(
    Path((id, stage_id)): Path<(Uuid, String)>,
    Json(req): Json<ApprovalRequest>,
) -> Result<StatusCode> {
    // Record approval, check stage completion
}
```

---

## Phase 3: Delegation System

### 3.1 Delegation CRUD Operations

**File:** `crates/iou-core/src/delegation/service.rs`

```rust
pub struct DelegationService;

impl DelegationService {
    pub async fn create_delegation(
        &self,
        from: Uuid,
        to: Uuid,
        delegation_type: DelegationType,
        document_types: Vec<String>,
        document_id: Option<Uuid>,
        starts_at: DateTime<Utc>,
        ends_at: Option<DateTime<Utc>>,
        created_by: Uuid,
    ) -> Result<Delegation> {
        // Validate delegation (no circular, within limits)
        // Create delegation record
    }
    
    pub async fn revoke_delegation(
        &self,
        delegation_id: Uuid,
        revoked_by: Uuid,
    ) -> Result<()> {
        // Mark as inactive, audit log
    }
    
    pub async fn auto_expire_delegations(&self) -> Result<Vec<Uuid>> {
        // Find and expire delegations past ends_at
    }
}
```

### 3.2 Delegation API

**File:** `crates/iou-api/src/routes/delegations.rs` (new)

```rust
pub fn delegations_router() -> Router {
    Router::new()
        .route("/api/delegations", get(list_delegations).post(create_delegation))
        .route("/api/delegations/:id", delete(revoke_delegation))
        .route("/api/users/:id/delegations", get(user_delegations))
}

/// List current user's delegations (created and received)
async fn list_delegations(
    claims: Claims,
) -> Result<Json<Vec<DelegationView>>> {
    // Return both from_user and to_user delegations
}

/// Create a new delegation
async fn create_delegation(
    claims: Claims,
    Json(req): Json<CreateDelegationRequest>,
) -> Result<Json<Delegation>> {
    // Validate and create
}
```

---

## Phase 4: Approval Expiry & Escalation

### 4.1 SLA Calculator

**File:** `crates/iou-core/src/sla/calculator.rs` (new)

```rust
use chrono::{DateTime, Utc, Weekday};

pub struct SlaCalculator {
    pub weekend_days: Vec<Weekday>,
}

impl SlaCalculator {
    /// Calculate deadline by adding business hours, skipping weekends
    pub fn calculate_deadline(
        &self,
        start: DateTime<Utc>,
        business_hours: i32,
    ) -> DateTime<Utc> {
        // Add hours, skip Saturday/Sunday
        // Simple implementation: no partial day handling
    }
    
    /// Check if deadline has passed
    pub fn is_overdue(&self, deadline: DateTime<Utc>) -> bool {
        Utc::now() > deadline
    }
    
    /// Calculate business hours until deadline
    pub fn hours_until_deadline(&self, deadline: DateTime<Utc>) -> i32 {
        // Count hours excluding weekends
    }
}
```

### 4.2 Escalation Service

**File:** `crates/iou-core/src/escalation/service.rs` (new)

```rust
use crate::sla::calculator::SlaCalculator;

pub struct EscalationService {
    sla_calculator: SlaCalculator,
    notification_channels: Vec<Box<dyn NotificationChannel>>,
}

impl EscalationService {
    /// Check for stages approaching or past deadline
    pub async fn check_overdue_stages(&self) -> Result<Vec<Escalation>> {
        // Query stages in document_approval_stages
        // Calculate hours until deadline
        // Trigger escalations based on configured thresholds
    }
    
    /// Send escalation via configured channels
    pub async fn send_escalation(
        &self,
        stage: &StageInstance,
        escalation_type: EscalationType,
    ) -> Result<()> {
        // Iterate through notification channels
        // Log to approval_escalations table
    }
}

pub trait NotificationChannel {
    async fn send(
        &self,
        recipient: Uuid,
        message: &EscalationMessage,
    ) -> Result<bool>;
}

// Channel implementations:
// - WebSocket push (Supabase realtime)
// - Email (via existing email service)
// - Webhook (HTTP POST)
// - UI indicator (database flag)
```

### 4.3 Scheduled Expiry Check

**File:** `crates/iou-orchestrator/src/jobs/expiry_checker.rs` (new)

```rust
use tokio::time::interval;

pub struct ExpiryChecker {
    interval: Duration,
    escalation_service: EscalationService,
}

impl ExpiryChecker {
    pub async fn run(&self) -> Infallible {
        let mut timer = interval(self.interval);
        loop {
            timer.tick().await;
            self.check_and_escalate().await;
        }
    }
    
    async fn check_and_escalate(&self) {
        // Check all in_progress stages
        // Calculate overdue status
        // Send escalations
        // Execute expiry actions
    }
}
```

---

## Phase 5: Version History & Diff Visualization

### 5.1 Version Storage Service

**File:** `crates/iou-core/src/versions/service.rs` (new)

```rust
use crate::storage::StorageClient;

pub struct VersionService {
    storage: StorageClient,
    full_versions_keep: i32,
}

impl VersionService {
    /// Create a new version when document is modified
    pub async fn create_version(
        &self,
        document_id: Uuid,
        content: &str,
        changed_by: Uuid,
        change_summary: &str,
    ) -> Result<DocumentVersion> {
        // Store content in S3
        // Check if we need to compress old versions
        // Create version record
    }
    
    /// Get version history for a document
    pub async fn list_versions(
        &self,
        document_id: Uuid,
    ) -> Result<Vec<DocumentVersion>> {
        // Query document_versions, ordered by created_at DESC
    }
    
    /// Restore a previous version (in-place)
    pub async fn restore_version(
        &self,
        document_id: Uuid,
        version_id: Uuid,
        restored_by: Uuid,
    ) -> Result<DocumentVersion> {
        // Fetch version content
        // Update document with restored content
        // Create new version recording the restore
        // Audit log entry
    }
    
    /// Compress old versions if threshold exceeded
    async fn compress_old_versions(&self, document_id: Uuid) -> Result<()> {
        // Count versions beyond full_versions_keep
        // Compress using gzip or similar
        // Update is_compressed flag
    }
}
```

### 5.2 Diff Generation

**File:** `crates/iou-core/src/diff/generator.rs` (new)

```rust
use similar::{ChangeTag, TextDiff};

pub struct DiffGenerator;

impl DiffGenerator {
    /// Generate diff between two document versions
    pub fn generate_diff(
        &self,
        old_content: &str,
        new_content: &str,
        format: DiffFormat,
    ) -> DocumentDiff {
        match format {
            DiffFormat::Unified => self.unified_diff(old_content, new_content),
            DiffFormat::SideBySide => self.side_by_side_diff(old_content, new_content),
            DiffFormat::Inline => self.inline_diff(old_content, new_content),
        }
    }
    
    fn unified_diff(&self, old: &str, new: &str) -> DocumentDiff {
        // Use similar::TextDiff for unified format
    }
    
    fn side_by_side_diff(&self, old: &str, new: &str) -> DocumentDiff {
        // Generate aligned changes for side-by-side view
    }
    
    fn inline_diff(&self, old: &str, new: &str) -> DocumentDiff {
        // Generate inline highlighted changes
    }
}

pub struct DocumentDiff {
    pub from_version: String,
    pub to_version: String,
    pub format: DiffFormat,
    pub changes: Vec<DiffChange>,
}

pub enum DiffFormat {
    Unified,
    SideBySide,
    Inline,
}

pub enum DiffChange {
    Unchanged(String),
    Inserted(String),
    Deleted(String),
    Replaced { old: String, new: String },
}
```

### 5.3 Version API

**File:** `crates/iou-api/src/routes/versions.rs` (new)

```rust
pub fn versions_router() -> Router {
    Router::new()
        .route("/api/documents/:id/versions", get(list_versions))
        .route("/api/documents/:id/versions/diff", get(get_diff))
        .route("/api/documents/:id/versions/:version_id", get(get_version))
        .route("/api/documents/:id/versions/:version_id/restore", post(restore_version))
}

/// List all versions of a document
async fn list_versions(
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VersionView>>> {
    // Return version metadata
}

/// Get diff between two versions
async fn get_diff(
    Path(id): Path<Uuid>,
    Query(params): Query<DiffParams>,
) -> Result<Json<DocumentDiff>> {
    // params.from, params.to, params.format
    // Generate and return diff
}

/// Restore a previous version
async fn restore_version(
    Path((id, version_id)): Path<(Uuid, Uuid)>,
    claims: Claims,
) -> Result<Json<VersionView>> {
    // Restore and return new version
}
```

---

## Phase 6: Frontend Components

### 6.1 Approval Queue Enhancements

**File:** `crates/iou-frontend/src/pages/approval_queue.rs` (modify)

Add to existing approval queue:
- Stage progress indicator (horizontal stepper)
- Countdown timer for each stage's deadline
- Color-coded urgency (green > 24h, yellow 8-24h, red < 8h)
- Delegation badge on approver names
- Escalation status icon

**Component:** `WorkflowStageTracker`

```rust
pub fn WorkflowStageTracker(cx: Scope, props: StageTrackerProps) -> Element {
    // Render horizontal stepper
    // Show completed, current, and pending stages
    // Display approvers and their status
}
```

### 6.2 Diff Viewer Component

**File:** `crates/iou-frontend/src/components/diff_viewer.rs` (new)

```rust
pub fn DiffViewer(cx: Scope, props: DiffViewerProps) -> Element {
    // Toggle for diff format (unified/side-by-side/inline)
    // Render diff changes with appropriate styling
    // Green for additions, red for deletions
}

#[derive(Props, PartialEq)]
pub struct DiffViewerProps {
    pub document_id: Uuid,
    pub from_version: String,
    pub to_version: String,
    pub format: DiffFormat,
}
```

### 6.3 Delegation Manager

**File:** `crates/iou-frontend/src/components/delegation_manager.rs` (new)

```rust
pub fn DelegationManager(cx: Scope) -> Element {
    // Form to create delegation
    // List of active delegations
    // Revoke button for each
}

pub fn CreateDelegationForm(cx: Scope) -> Element {
    // Fields: to_user, type, document_types, date_range
    // Validation and submission
}
```

### 6.4 Version History Component

**File:** `crates/iou-frontend/src/components/version_history.rs` (new)

```rust
pub fn VersionHistory(cx: Scope, props: VersionHistoryProps) -> Element {
    // List versions with metadata
    // Compare button (select two versions)
    // Restore button with confirmation
}
```

---

## Phase 7: Testing

### 7.1 Unit Tests

**File:** `crates/iou-core/tests/workflows/multi_stage.rs` (new)

Test coverage:
- Stage transition logic
- Quorum evaluation (any/all/majority)
- SLA calculation (weekend skipping)
- Delegation resolution
- Diff generation

### 7.2 Integration Tests

**File:** `crates/iou-api/tests/workflows/end_to_end.rs` (new)

Test scenarios:
- Multi-stage document flow (sequential stages)
- Parallel approval with quorum
- Delegation during approval
- Expiry and escalation
- Version creation and restoration
- Diff between versions

### 7.3 Frontend Tests

Consider adding Dioxus testing framework for:
- Stage tracker rendering
- Diff viewer format toggle
- Delegation form validation

---

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
# For configuration hot-reload
notify = "6.0"

# For diff generation
similar = "2.4"

# For async file watching (already has tokio)
```

---

## Directory Structure

```
crates/
├── iou-core/
│   ├── src/
│   │   ├── workflows/
│   │   │   └── multi_stage.rs          # Stage type definitions
│   │   ├── delegation/
│   │   │   ├── mod.rs                  # Delegation types
│   │   │   └── resolver.rs             # Delegation resolution
│   │   ├── sla/
│   │   │   └── calculator.rs           # SLA deadline calculation
│   │   ├── escalation/
│   │   │   └── service.rs              # Escalation orchestration
│   │   ├── versions/
│   │   │   └── service.rs              # Version storage & retrieval
│   │   ├── diff/
│   │   │   └── generator.rs            # Diff generation
│   │   └── config/
│   │       ├── workflow.rs             # Config types
│   │       └── watcher.rs              # File system watcher
│   └── tests/
│       └── workflows/
│           └── multi_stage.rs          # Unit tests
├── iou-orchestrator/
│   ├── src/
│   │   ├── state_machine/
│   │   │   └── multi_stage.rs          # Multi-stage transitions
│   │   ├── stage_executor.rs           # Stage execution logic
│   │   └── jobs/
│   │       └── expiry_checker.rs       # Scheduled expiry checks
├── iou-api/
│   ├── src/
│   │   └── routes/
│   │       ├── workflow_stages.rs      # Stage API endpoints
│   │       ├── delegations.rs          # Delegation API
│   │       └── versions.rs             # Version API
│   └── tests/
│       └── workflows/
│           └── end_to_end.rs           # Integration tests
└── iou-frontend/
    └── src/
        ├── pages/
        │   └── approval_queue.rs       # Enhanced with stages
        └── components/
            ├── workflow_stage_tracker.rs
            ├── diff_viewer.rs
            ├── delegation_manager.rs
            └── version_history.rs

config/
└── workflows/
    ├── defaults.yaml                   # Global workflow config
    └── domains/                        # Per-domain overrides
        └── example.yaml

migrations/
└── 040_enhanced_workflow.sql           # Database schema changes
```

---

## Implementation Sequence

**Phase 1 (Foundation):**
1. Create database migration
2. Add core type definitions
3. Implement configuration system with hot-reload

**Phase 2 (Multi-Stage):**
4. Extend state machine for multi-stage
5. Implement stage executor
6. Add delegation resolution
7. Create stage API endpoints

**Phase 3 (Delegation):**
8. Implement delegation CRUD service
9. Create delegation API endpoints
10. Add delegation UI components

**Phase 4 (Expiry & Escalation):**
11. Implement SLA calculator
12. Create escalation service with notification channels
13. Add scheduled expiry checker job
14. Update approval queue with deadline timers

**Phase 5 (Versions & Diff):**
15. Implement version storage service
16. Add diff generator with `similar` crate
17. Create version API endpoints
18. Add diff viewer and version history UI components

**Phase 6 (Testing & Polish):**
19. Add unit tests for new components
20. Create integration tests for end-to-end workflows
21. Performance testing for large document sets
22. Security audit for delegation and approval bypass risks

---

## Success Criteria

Each feature is complete when:

**Multi-Stage Approvals:**
- Documents flow through configured stages sequentially
- Parallel stages evaluate quorum correctly
- Stage progress visible in approval queue UI
- Config changes hot-reload without restart

**Delegation:**
- Users can create temporary/permanent/bulk delegations
- Delegated approvers show original approver in audit trail
- Delegation chains limited to 3 hops
- Auto-expiry of temporary delegations

**Expiry & Escalation:**
- Countdown timers display in approval queue
- Escalations sent via all configured channels
- Expiry actions execute correctly
- SLA calculation skips weekends

**Version History & Diff:**
- Versions stored with configurable compression
- Diff generated in all three formats
- Previous versions can be restored
- Audit trail records all restores

**Backward Compatibility:**
- Existing single-stage workflows function unchanged
- No breaking changes to existing API endpoints
- Existing tests continue to pass
