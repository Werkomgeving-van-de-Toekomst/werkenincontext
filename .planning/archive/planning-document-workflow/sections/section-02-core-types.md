Now I have all the context needed to generate the section content for `section-02-core-types`. Let me extract the relevant content for this section.

# Section 02: Core Type Definitions

## Overview

This section defines the Rust type system for multi-stage approval workflows, delegation, and version tracking. These types serve as the foundation for the enhanced workflow system and are used across the orchestrator, API, and core services.

## Dependencies

This section depends on:
- **section-01-database-schema**: The database tables (`approval_stages`, `delegations`, `document_approval_stages`, `document_approvals`) must exist before these types can be persisted

## Files to Create

1. `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows/multi_stage.rs` - Multi-stage workflow types
2. `/Users/marc/Projecten/iou-modern/crates/iou-core/src/delegation.rs` - Delegation types
3. `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows/mod.rs` - Module exports (may need modification)

## Implementation

### 1. Multi-Stage Workflow Types

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows/multi_stage.rs`

Create the core types for approval stage definitions and per-document stage instances.

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an approval stage definition (configured, not per-document)
#[derive(Debug, Clone, Deserialize, Serialize)]
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

/// An approver definition - can be a specific user or a role
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Approver {
    pub user_id: Option<Uuid>,
    pub role: Option<String>,
}

impl Approver {
    /// Validate that exactly one of user_id or role is set
    pub fn validate(&self) -> Result<(), String> {
        match (&self.user_id, &self.role) {
            (None, None) => Err("Approver must have either user_id or role"),
            (Some(_), Some(_)) => Err("Approver cannot have both user_id and role"),
            _ => Ok(()),
        }
    }
}

/// How approvals within a stage are counted
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    Sequential,
    ParallelAny,
    ParallelAll,
    ParallelMajority,
}

/// What happens when a stage's deadline passes
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExpiryAction {
    NotifyOnly,
    ReturnToDraft,
    AutoApprove,
    EscalateTo { target: String },
}

/// Per-document stage instance state
#[derive(Debug, Clone, Deserialize, Serialize)]
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

impl StageInstance {
    /// Create a new pending stage instance
    pub fn new(document_id: Uuid, stage_id: String, approvers: Vec<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            document_id,
            stage_id,
            status: StageStatus::Pending,
            started_at: None,
            completed_at: None,
            deadline: None,
            approvers,
            approvals_received: Vec::new(),
        }
    }

    /// Transition the stage status
    pub fn transition_to(&mut self, new_status: StageStatus) -> Result<(), String> {
        match (&self.status, &new_status) {
            (StageStatus::Pending, StageStatus::Completed) => {
                Err("Cannot transition from Pending to Completed without InProgress")
            }
            (StageStatus::Completed, _) | (StageStatus::Expired, _) => {
                Err("Cannot transition from terminal state")
            }
            _ => {
                self.status = new_status;
                match new_status {
                    StageStatus::InProgress => {
                        self.started_at = Some(Utc::now());
                    }
                    StageStatus::Completed | StageStatus::Expired => {
                        self.completed_at = Some(Utc::now());
                    }
                    _ => {}
                }
                Ok(())
            }
        }
    }

    /// Add an approval response
    pub fn add_approval(&mut self, approval: ApprovalResponse) -> Result<(), String> {
        // Check for duplicate approval from same approver
        if self.approvals_received.iter().any(|a| a.approver_id == approval.approver_id) {
            return Err("Approver has already responded to this stage");
        }
        self.approvals_received.push(approval);
        Ok(())
    }
}

/// Status of a stage instance
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
    Expired,
}

/// An individual approval response within a stage
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApprovalResponse {
    pub approver_id: Uuid,
    pub delegated_from: Option<Uuid>,
    pub decision: ApprovalDecision,
    pub comment: Option<String>,
    pub responded_at: DateTime<Utc>,
}

impl ApprovalResponse {
    /// Create a new approval response
    pub fn new(approver_id: Uuid, decision: ApprovalDecision, comment: Option<String>) -> Self {
        Self {
            approver_id,
            delegated_from: None,
            decision,
            comment,
            responded_at: Utc::now(),
        }
    }

    /// Create a delegated approval response
    pub fn with_delegation(mut self, delegated_from: Uuid) -> Self {
        self.delegated_from = Some(delegated_from);
        self
    }
}

/// The decision made by an approver
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    Delegated { to: Uuid },
}
```

### 2. Delegation Types

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/delegation.rs`

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A delegation of approval authority from one user to another
#[derive(Debug, Clone, Deserialize, Serialize)]
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

impl Delegation {
    /// Check if this delegation is currently active
    pub fn is_currently_active(&self) -> bool {
        if !self.is_active {
            return false;
        }
        let now = Utc::now();
        if now < self.starts_at {
            return false;
        }
        if let Some(ends_at) = self.ends_at {
            if now > ends_at {
                return false;
            }
        }
        true
    }

    /// Validate the delegation configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.from_user_id == self.to_user_id {
            return Err("Cannot delegate to self");
        }
        if let Some(ends_at) = self.ends_at {
            if ends_at <= self.starts_at {
                return Err("End time must be after start time");
            }
        }
        // Validate delegation type matches document configuration
        match self.delegation_type {
            DelegationType::Bulk => {
                if self.document_id.is_some() {
                    return Err("Bulk delegations cannot specify a single document");
                }
            }
            DelegationType::Permanent => {
                if self.ends_at.is_some() {
                    return Err("Permanent delegations cannot have an end time");
                }
            }
            DelegationType::Temporary => {
                if self.ends_at.is_none() {
                    return Err("Temporary delegations must have an end time");
                }
            }
        }
        Ok(())
    }
}

/// The scope/type of delegation
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DelegationType {
    /// Temporary delegation with a specific date range
    Temporary,
    /// Permanent delegation (no end date)
    Permanent,
    /// Bulk delegation for multiple document types
    Bulk,
}

/// A resolved approver with delegation chain information
#[derive(Debug, Clone)]
pub struct ResolvedApprover {
    pub user_id: Uuid,
    pub delegation_chain: Vec<Uuid>,
    pub is_delegated: bool,
}

impl ResolvedApprover {
    /// Create a non-delegated approver
    pub fn direct(user_id: Uuid) -> Self {
        Self {
            user_id,
            delegation_chain: Vec::new(),
            is_delegated: false,
        }
    }

    /// Create a delegated approver
    pub fn delegated(user_id: Uuid, chain: Vec<Uuid>) -> Self {
        Self {
            user_id,
            delegation_chain: chain,
            is_delegated: true,
        }
    }
}
```

### 3. Module Exports

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows/mod.rs`

Ensure the workflow module properly exports the multi-stage types:

```rust
mod multi_stage;

pub use multi_stage::{
    ApprovalStage,
    Approver,
    ApprovalType,
    ExpiryAction,
    StageInstance,
    StageStatus,
    ApprovalResponse,
    ApprovalDecision,
};
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/lib.rs`

Ensure the delegation module is exported at the crate root:

```rust
pub mod delegation;

pub use delegation::{
    Delegation,
    DelegationType,
    ResolvedApprover,
};
```

## Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/tests/workflows/multi_stage_types.rs`

Write tests FIRST following TDD principles:

```rust
use iou_core::workflows::*;
use iou_core::delegation::*;
use uuid::Uuid;

#[tokio::test]
async fn test_approval_stage_validates_required_fields() {
    // Test that ApprovalStage requires stage_id, stage_name, stage_order, approval_type
    let stage = ApprovalStage {
        stage_id: "review".to_string(),
        stage_name: "Legal Review".to_string(),
        stage_order: 1,
        approval_type: ApprovalType::ParallelAll,
        approvers: vec![],
        sla_hours: 72,
        expiry_action: ExpiryAction::NotifyOnly,
        is_optional: false,
        condition: None,
    };
    assert_eq!(stage.stage_id, "review");
}

#[tokio::test]
async fn test_approval_type_serializes_correctly() {
    // Test serialization/deserialization of all ApprovalType variants
    let types = vec![
        ApprovalType::Sequential,
        ApprovalType::ParallelAny,
        ApprovalType::ParallelAll,
        ApprovalType::ParallelMajority,
    ];
    for t in types {
        let serialized = serde_json::to_string(&t).unwrap();
        let deserialized: ApprovalType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(t, deserialized);
    }
}

#[tokio::test]
async fn test_expiry_action_escalate_to_includes_target() {
    // Test that EscalateTo serializes with target field
    let action = ExpiryAction::EscalateTo {
        target: "manager".to_string(),
    };
    let serialized = serde_json::to_string(&action).unwrap();
    assert!(serialized.contains("manager"));
    let deserialized: ExpiryAction = serde_json::from_str(&serialized).unwrap();
    matches!(deserialized, ExpiryAction::EscalateTo { .. });
}

#[tokio::test]
async fn test_stage_instance_status_transitions() {
    // Test valid transitions: Pending -> InProgress -> Completed
    let mut instance = StageInstance::new(Uuid::new_v4(), "review".to_string(), vec![]);
    assert_eq!(instance.status, StageStatus::Pending);
    
    instance.transition_to(StageStatus::InProgress).unwrap();
    assert_eq!(instance.status, StageStatus::InProgress);
    assert!(instance.started_at.is_some());
    
    instance.transition_to(StageStatus::Completed).unwrap();
    assert_eq!(instance.status, StageStatus::Completed);
    assert!(instance.completed_at.is_some());
}

#[tokio::test]
async fn test_stage_instance_cannot_skip_to_completed() {
    // Test that Pending -> Completed is rejected
    let mut instance = StageInstance::new(Uuid::new_v4(), "review".to_string(), vec![]);
    let result = instance.transition_to(StageStatus::Completed);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_approval_response_includes_delegated_from() {
    // Test that delegated_from is tracked when set
    let original = Uuid::new_v4();
    let delegate = Uuid::new_v4();
    let response = ApprovalResponse::new(delegate, ApprovalDecision::Approved, None)
        .with_delegation(original);
    assert_eq!(response.delegated_from, Some(original));
}

#[tokio::test]
async fn test_approver_requires_either_user_id_or_role() {
    // Test validation: exactly one of user_id or role must be set
    let approver_both = Approver {
        user_id: Some(Uuid::new_v4()),
        role: Some("manager".to_string()),
    };
    assert!(approver_both.validate().is_err());
    
    let approver_neither = Approver {
        user_id: None,
        role: None,
    };
    assert!(approver_neither.validate().is_err());
    
    let approver_user = Approver {
        user_id: Some(Uuid::new_v4()),
        role: None,
    };
    assert!(approver_user.validate().is_ok());
}

#[tokio::test]
async fn test_stage_instance_tracks_approvals_in_order() {
    // Test that approvals_received maintains insertion order
    let mut instance = StageInstance::new(
        Uuid::new_v4(),
        "review".to_string(),
        vec![Uuid::new_v4(), Uuid::new_v4()],
    );
    
    let approval1 = ApprovalResponse::new(Uuid::new_v4(), ApprovalDecision::Approved, None);
    let approval2 = ApprovalResponse::new(Uuid::new_v4(), ApprovalDecision::Approved, None);
    
    instance.add_approval(approval1.clone()).unwrap();
    instance.add_approval(approval2.clone()).unwrap();
    
    assert_eq!(instance.approvals_received.len(), 2);
    assert_eq!(instance.approvals_received[0].approver_id, approval1.approver_id);
    assert_eq!(instance.approvals_received[1].approver_id, approval2.approver_id);
}

#[tokio::test]
async fn test_delegation_validates_no_self_delegation() {
    // Test that from_user cannot equal to_user
    let user_id = Uuid::new_v4();
    let delegation = Delegation {
        id: Uuid::new_v4(),
        from_user_id: user_id,
        to_user_id: user_id,
        delegation_type: DelegationType::Temporary,
        document_types: vec![],
        document_id: None,
        starts_at: Utc::now(),
        ends_at: Some(Utc::now() + chrono::Duration::hours(24)),
        is_active: true,
        created_at: Utc::now(),
        created_by: user_id,
    };
    assert!(delegation.validate().is_err());
}

#[tokio::test]
async fn test_delegation_is_currently_active() {
    // Test active, not started, and expired scenarios
    let now = Utc::now();
    
    // Active delegation
    let active = Delegation {
        id: Uuid::new_v4(),
        from_user_id: Uuid::new_v4(),
        to_user_id: Uuid::new_v4(),
        delegation_type: DelegationType::Temporary,
        document_types: vec![],
        document_id: None,
        starts_at: now - chrono::Duration::hours(1),
        ends_at: Some(now + chrono::Duration::hours(1)),
        is_active: true,
        created_at: now,
        created_by: Uuid::new_v4(),
    };
    assert!(active.is_currently_active());
    
    // Not yet started
    let not_started = Delegation { starts_at: now + chrono::Duration::hours(1), ..active.clone() };
    assert!(!not_started.is_currently_active());
    
    // Expired
    let expired = Delegation { ends_at: Some(now - chrono::Duration::hours(1)), ..active.clone() };
    assert!(!expired.is_currently_active());
    
    // Marked inactive
    let inactive = Delegation { is_active: false, ..active };
    assert!(!inactive.is_currently_active());
}

#[tokio::test]
async fn test_stage_instance_rejects_duplicate_approval() {
    // Test that the same approver cannot approve twice
    let mut instance = StageInstance::new(
        Uuid::new_v4(),
        "review".to_string(),
        vec![Uuid::new_v4()],
    );
    
    let approver = Uuid::new_v4();
    let approval1 = ApprovalResponse::new(approver, ApprovalDecision::Approved, None);
    let approval2 = ApprovalResponse::new(approver, ApprovalDecision::Approved, None);
    
    instance.add_approval(approval1).unwrap();
    let result = instance.add_approval(approval2);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resolved_approver_delegated() {
    // Test ResolvedApprover::direct and ResolvedApprover::delegated
    let user_id = Uuid::new_v4();
    
    let direct = ResolvedApprover::direct(user_id);
    assert!(!direct.is_delegated);
    assert!(direct.delegation_chain.is_empty());
    assert_eq!(direct.user_id, user_id);
    
    let chain = vec![Uuid::new_v4(), Uuid::new_v4()];
    let delegated = ResolvedApprover::delegated(user_id, chain.clone());
    assert!(delegated.is_delegated);
    assert_eq!(delegated.delegation_chain, chain);
    assert_eq!(delegated.user_id, user_id);
}
```

## Implementation Checklist

1. Create `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows/multi_stage.rs` with all type definitions
2. Create `/Users/marc/Projecten/iou-modern/crates/iou-core/src/delegation.rs` with delegation types
3. Update module exports in `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows/mod.rs`
4. Update crate exports in `/Users/marc/Projecten/iou-modern/crates/iou-core/src/lib.rs`
5. Create test file `/Users/marc/Projecten/iou-modern/crates/iou-core/tests/workflows/multi_stage_types.rs`
6. Run `cargo test` to verify all tests pass
7. Verify `cargo clippy` produces no warnings

## Verification

After implementation, verify:

- All types serialize/deserialize correctly with `serde`
- `StageInstance` state machine only allows valid transitions
- `Approver` validation enforces exactly one of user_id or role
- `Delegation` validation prevents self-delegation and invalid date ranges
- `is_currently_active()` correctly handles all delegation states
- Duplicate approvals are rejected by `StageInstance::add_approval()`
## Implementation Notes

### Files Created/Modified

- `crates/iou-core/src/workflows/multi_stage.rs` - New: Multi-stage approval types with 12 unit tests
- `crates/iou-core/src/delegation.rs` - New: Delegation types with 17 unit tests
- `crates/iou-core/src/workflows/mod.rs` - New: Workflow module (replaces workflows.rs)
- `crates/iou-core/src/lib.rs` - Modified: Added delegation module and new exports
- `crates/iou-core/tests/workflows/mod.rs` - New: Enables test discovery for nested test files
- `crates/iou-core/tests/workflows/multi_stage_types.rs` - New: Integration tests (11 tests)
- `crates/iou-core/src/workflows.rs` - Deleted: Replaced by workflows/mod.rs

### Changes from Original Plan

1. **Removed workflows.rs** - Converted to `workflows/mod.rs` module structure
2. **Added helper methods** to `StageInstance`:
   - `approved_count()` - Count approved responses
   - `rejected_count()` - Count rejected responses
   - `is_complete()` - Check if all approvers responded (with clarifying documentation)
3. **Added factory methods** to `Delegation`:
   - `new_temporary()` - Create temporary delegation
   - `new_permanent()` - Create permanent delegation
   - `new_bulk()` - Create bulk delegation
   - `new_for_document()` - Create document-specific delegation
4. **Added helper methods** to `ResolvedApprover`:
   - `original_approver()` - Get the original approver before delegations
   - `chain_length()` - Get delegation chain length

### Code Review Fixes Applied

1. Created `tests/workflows/mod.rs` to enable integration test discovery
2. Added clarifying documentation to `StageInstance::is_complete()` explaining it counts ALL responses
3. Noted delegation cycle detection as a concern for section-06 (delegation resolver)

### Test Results

All 66 tests pass (29 new tests for section-02):
- 12 multi_stage unit tests
- 17 delegation unit tests
- 2 existing workflow tests
- 11 integration tests (now properly discovered)
- 24 existing tests from other modules

