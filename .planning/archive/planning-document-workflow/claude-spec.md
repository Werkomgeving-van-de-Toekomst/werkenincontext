# Enhanced Document Workflow - Complete Specification

## Overview

Extend the existing IOU-Modern document workflow system to support multi-stage approvals, delegation, expiry handling with escalation, and version history with diff visualization. This will enable organizations to implement complex approval processes while maintaining compliance and auditability.

## Current State Analysis

### Existing Workflow Engine

**Location:** `iou-orchestrator/src/state_machine.rs`, `iou-core/src/workflows.rs`

**Current States:**
```
Draft → Submitted → InReview → Approved → Published
         ↓           ↓
       Draft    ChangesRequested
```

**Features:**
- Strict transition validation with `can_transition_to()`
- Trust level system (Low/Medium/High) for auto-approval decisions
- Workflow definitions with steps and timeouts
- Support for Rust (built-in) and Camunda (Zeebe) workflow drivers
- Comprehensive audit trail with 7-year retention

### Current Database Schema

**Key Tables:**
- `documents` - Core metadata with version keys
- `document_audit` - Immutable audit trail
- `document_versions` - Basic version history
- `templates` - Document templates per domain
- `domain_configs` - Per-domain workflow settings

**Storage:** DuckDB (embedded) + Supabase PostgreSQL for realtime

### Current UI

**Approval Queue:** `/documenten/wachtrij`
- Document listing with status badges
- Approve/Reject with comments
- Audit trail preview
- WebSocket auto-refresh

## Requirements

### 1. Multi-Stage Approvals

Documents should flow through multiple approval stages before final publication. Each stage can have different approvers and requirements.

**Use Cases:**
- Woo documents: Legal review → Department head → Communication officer
- Budget documents: Financial control → Management board
- Project documents: Project lead → Steering committee

**Requirements:**
1. **Configurable Stages** (from interview):
   - YAML/JSON configuration files
   - Global defaults with per-domain overrides
   - Automatic hot-reload without service restart

2. **Stage Types:**
   - Sequential: Stages complete in order
   - Parallel: Multiple approvers at same stage
   - Hybrid: Sequential stages with parallel subprocesses

3. **Parallel Approval Quorum** (from interview):
   - Configurable per stage: any/all/majority
   - Track individual approval responses

4. **Stage Configuration:**
   - Stage name and description
   - Required approvers (user IDs or roles)
   - Approval type (sequential/parallel with quorum)
   - SLA deadline (business hours)
   - Expiry action (notify/return/auto-approve)
   - Optional stages based on document properties

**Data Model:**
```rust
pub struct ApprovalStage {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    pub approval_type: ApprovalType,
    pub approvers: Vec<Approver>,
    pub quorum: QuorumType,
    pub sla_hours: i32,
    pub expiry_action: ExpiryAction,
    pub is_optional: bool,
    pub condition: Option<String>,
}

pub enum ApprovalType {
    Sequential,
    Parallel { quorum: QuorumType },
}

pub enum QuorumType {
    Any,
    All,
    Majority,
}

pub enum ExpiryAction {
    NotifyOnly,
    ReturnToDraft,
    AutoApprove,
    EscalateTo,
}
```

### 2. Delegation

Approvers can delegate their approval authority to another user.

**Use Cases:**
- Manager on vacation delegates to deputy
- Department head delegates specific document types to team lead
- Temporary delegation during absences

**Requirements** (from interview):
1. **Three Delegation Types:**
   - Bulk: All approvals during date range
   - Per-document-type: Specific types to another user
   - Per-document: Single document instance

2. **Temporary vs Permanent:**
   - Temporary: Date range with auto-expiry
   - Permanent: Until revoked

3. **Audit Trail:**
   - Always record original approver
   - Display "Approved by X (delegated from Y)"
   - Limit delegation chains (max 3 hops)

**Data Model:**
```rust
pub struct Delegation {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub delegation_type: DelegationType,
    pub document_types: Vec<String>,  // Empty = all
    pub document_id: Option<Uuid>,    // For single-document
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub is_active: bool,
}

pub enum DelegationType {
    Temporary,
    Permanent,
    Bulk,
}
```

### 3. Approval Expiry & Escalation

Approvals have deadlines to prevent documents from stalling.

**Use Cases:**
- Legal review within 5 business days
- Escalation to manager if no response in 48 hours
- Auto-return to draft after 30 days

**Requirements** (from interview):
1. **SLA Calculation:**
   - Business days = skip weekends only
   - Simple 24-hour cycles excluding Saturday/Sunday
   - Configurable per document type

2. **Deadline Display:**
   - Countdown timer in approval queue UI
   - Color-coded (green → yellow → red)

3. **Escalation Actions** (from interview):
   - All channels: Push notification, Webhook, UI indicator, Email
   - Configurable per document type

4. **Expiry Actions** (from interview):
   - Configurable per document type
   - Options: Notify and wait, Return to draft, Auto-approve

**SLA Calculator:**
```rust
pub struct SlaCalculator {
    pub weekend_days: Vec<Weekday>,  // [Saturday, Sunday]
}

impl SlaCalculator {
    pub fn calculate_deadline(&self, submitted: DateTime<Utc>, hours: i32) -> DateTime<Utc> {
        // Add hours, skipping weekends
    }
    
    pub fn is_overdue(&self, submitted: DateTime<Utc>, hours: i32) -> bool {
        // Check if current time > deadline
    }
    
    pub fn business_hours_until(&self, submitted: DateTime<Utc>, deadline: DateTime<Utc>) -> i32 {
        // Return business hours remaining
    }
}
```

### 4. Version History with Diff View

Track document versions and show what changed between them.

**Use Cases:**
- Reviewer sees changes before approving
- Audit trail shows exact changes per revision
- Compare any two versions

**Requirements** (from interview):
1. **Version Storage:**
   - Hybrid approach: Configurable N full versions + compressed older
   - Version numbering: v1, v2, v3...
   - Metadata: who changed, when, why

2. **Diff Formats** (from interview):
   - All formats with user preference: Unified, Side-by-side, Inline
   - Use `similar` crate for backend diff generation

3. **Restore:**
   - In-place revert (from interview)
   - Creates new audit entry

**Data Model:**
```rust
pub struct DocumentVersion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version_number: String,
    pub storage_key: String,
    pub is_compressed: bool,
    pub change_summary: String,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
    pub is_current: bool,
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

## Technical Architecture

### New Database Tables

```sql
-- Approval stage definitions
CREATE TABLE approval_stages (
    id VARCHAR(50) PRIMARY KEY,
    domain_id VARCHAR(50),
    document_type VARCHAR(100),
    stage_name VARCHAR(200) NOT NULL,
    stage_order INTEGER NOT NULL,
    approval_type VARCHAR(20) NOT NULL,  -- sequential, parallel_any, parallel_all, parallel_majority
    approvers JSONB NOT NULL,
    sla_hours INTEGER NOT NULL DEFAULT 72,
    expiry_action VARCHAR(50) NOT NULL DEFAULT 'notify_only',
    is_optional BOOLEAN DEFAULT false,
    condition TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_approval_stages_domain_type ON approval_stages(domain_id, document_type);

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

CREATE INDEX idx_delegations_active ON delegations(from_user_id, is_active) WHERE is_active = true;
CREATE INDEX idx_delegations_temporal ON delegations(starts_at, ends_at);

-- Approval stage instances (tracking state per document)
CREATE TABLE document_approval_stages (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id),
    stage_id VARCHAR(50) NOT NULL,
    stage_status VARCHAR(20) NOT NULL,  -- pending, in_progress, completed, skipped, expired
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    deadline TIMESTAMPTZ,
    approvers JSONB NOT NULL,
    approvals_received JSONB DEFAULT '[]',
    UNIQUE(document_id, stage_id)
);

CREATE INDEX idx_doc_approval_stages_doc ON document_approval_stages(document_id);
CREATE INDEX idx_doc_approval_stages_status ON document_approval_stages(stage_status);

-- Individual approval responses
CREATE TABLE document_approvals (
    id UUID PRIMARY KEY,
    stage_instance_id UUID NOT NULL REFERENCES document_approval_stages(id),
    approver_id UUID NOT NULL,
    delegated_from UUID,
    decision VARCHAR(20),  -- approved, rejected, delegated
    comment TEXT,
    responded_at TIMESTAMPTZ,
    UNIQUE(stage_instance_id, approver_id)
);

CREATE INDEX idx_doc_approvals_stage ON document_approvals(stage_instance_id);

-- Extended version history
ALTER TABLE document_versions ADD COLUMN is_compressed BOOLEAN DEFAULT false;
ALTER TABLE document_versions ADD COLUMN parent_version_id UUID REFERENCES document_versions(id);
ALTER TABLE document_versions ADD COLUMN diff_summary JSONB;

-- Escalation log
CREATE TABLE approval_escalations (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id),
    stage_instance_id UUID REFERENCES document_approval_stages(id),
    escalation_type VARCHAR(50) NOT NULL,
    notification_channel VARCHAR(50) NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL,
    acknowledged_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'pending'  -- pending, acknowledged, resolved
);

CREATE INDEX idx_approval_escalations_doc ON approval_escalations(document_id);
```

### Configuration Files

**Location:** `/config/workflows/`

**Global Defaults:** `defaults.yaml`
```yaml
approval_stages:
  - stage_id: "single_default"
    stage_name: "Standard Approval"
    stage_order: 1
    approval_type: sequential
    approvers: []  # Empty = document submitter's manager
    sla_hours: 72
    expiry_action: notify_only

version_storage:
  full_versions_keep: 5
  compress_after_days: 30

sla:
  weekends: [Saturday, Sunday]
  escalation_hours: [48, 72]  # Escalate at 48h, 72h
```

**Domain Override:** `/config/workflows/domains/{domain_id}.yaml`
```yaml
approval_stages:
  - stage_id: "woo_legal"
    stage_name: "Legal Review"
    stage_order: 1
    approval_type:
      parallel:
        quorum: all
    approvers:
      - role: "legal_officer"
    sla_hours: 120  # 5 business days
    expiry_action: escalate_to
    escalate_to: "legal_director"

  - stage_id: "woo_dept"
    stage_name: "Department Head"
    stage_order: 2
    approval_type: sequential
    approvers:
      - role: "department_head"
    sla_hours: 48
    expiry_action: return_to_draft
```

### API Endpoints

**Multi-Stage Workflow:**
```
GET    /api/documents/{id}/stages          - Get approval stages for document
GET    /api/documents/{id}/stages/{stage_id} - Get stage status
POST   /api/documents/{id}/stages/{stage_id}/approve - Approve stage
POST   /api/documents/{id}/stages/{stage_id}/reject - Reject stage
```

**Delegation:**
```
GET    /api/delegations                    - List my delegations
POST   /api/delegations                    - Create delegation
DELETE /api/delegations/{id}               - Revoke delegation
GET    /api/users/{id}/delegations          - View user's active delegations
```

**Version History:**
```
GET    /api/documents/{id}/versions        - List all versions
GET    /api/documents/{id}/versions/{version_id} - Get version details
GET    /api/documents/{id}/versions/diff?from=v1&to=v2 - Get diff
POST   /api/documents/{id}/versions/{version_id}/restore - Restore version
```

### Frontend Components

**Approval Queue Enhancements:**
- Multi-stage progress indicator
- Countdown timer per stage
- Delegation badge on assigned approvers
- Escalation status indicators

**New Components:**
- `WorkflowStageTracker` - Visual stage progress
- `DiffViewer` - Version comparison with format toggle
- `DelegationManager` - Create/manage delegations
- `VersionHistory` - Version list with restore action

## Success Criteria

1. Multi-stage workflow can be configured via YAML files
2. Documents flow through all stages correctly
3. Parallel approvals support configurable quorum
4. User can delegate approval authority (bulk/per-type/per-document)
5. Approval queue shows deadline countdown
6. Escalations occur automatically via all configured channels
7. Version comparison shows visual diff in multiple formats
8. Previous versions can be restored
9. All actions logged in audit trail
10. Existing single-stage workflows continue to work
11. Config changes hot-reload without service restart

## Dependencies

**Rust Crates:**
- `similar` - Text diffing
- `notify` - File system watcher for config hot-reload
- `chrono` - Date/time handling (already in use)

**Frontend:**
- Consider Dioxus testing framework for component tests

## Open Questions Resolved

| Question | Resolution |
|----------|------------|
| Approval stages definition | Config file based with hot-reload |
| Business day calculation | Skip weekends only (simple) |
| Diff format | All formats with user preference |
| Diff calculation location | Backend (Rust with `similar`) |
| Parallel approval quorum | Configurable per stage |
| Expiry action | Configurable per document type |
| Version storage | Hybrid (configurable threshold) |
| Restore behavior | In-place revert |
| Config location | Hybrid (defaults + domain overrides) |
| Escalation channels | All: push, webhook, UI, email |
