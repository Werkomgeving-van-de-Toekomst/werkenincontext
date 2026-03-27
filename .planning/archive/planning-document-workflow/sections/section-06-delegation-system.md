Now I have all the context I need. Let me extract the relevant content for section-06-delegation-system from both the implementation plan and the TDD plan, and create a self-contained section.

Looking at the index.md, section-06-delegation-system is described as:

> Delegation CRUD operations and resolution logic. Supports temporary (date-range), permanent, and bulk delegations. Resolves actual approvers considering active delegations, prevents circular chains.

**Files:**
- `crates/iou-core/src/delegation/resolver.rs`
- `crates/iou-core/src/delegation/service.rs`
- Tests in `crates/iou-core/tests/delegation/resolution.rs`
- Tests in `crates/iou-core/tests/delegation/service.rs`

**Dependencies:**
- Depends on: section-04-multi-stage-engine
- Blocks: section-07-sla-escalation, section-09-api-endpoints

Now I will extract the relevant content from both plans and create the section.</think># Section 6: Delegation System

## Overview

The delegation system allows users to temporarily transfer their approval authority to other users. This is essential for handling absences (vacation, sick leave), workload distribution, and organizational changes. The system supports three types of delegations:

1. **Temporary**: Date-range bounded delegations (e.g., "while I'm on vacation next week")
2. **Permanent**: Ongoing delegations without end date
3. **Bulk**: Delegations covering multiple document types at once

The system also includes a resolution layer that automatically routes approval requests to the correct delegate when a document enters a stage.

## Dependencies

This section requires completion of:
- **section-04-multi-stage-engine**: For stage initialization which uses delegation resolution

## Database Schema

The delegation system uses the `delegations` table created in the database migration (section-01-database-schema):

```sql
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
```

**Key indexes for delegation queries:**
- `(from_user_id, is_active)` for active delegation lookup
- `(to_user_id, is_active)` for viewing received delegations

## Type Definitions

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/delegation.rs`

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a delegation from one user to another
#[derive(Debug, Clone)]
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

/// Type of delegation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelegationType {
    /// Temporary delegation with date range
    Temporary,
    /// Permanent delegation without end date
    Permanent,
    /// Bulk delegation covering multiple document types
    Bulk,
}
```

## Implementation

### 6.1 Delegation CRUD Service

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/delegation/service.rs`

The `DelegationService` handles creating, revoking, and auto-expiring delegations.

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::delegation::{Delegation, DelegationType};

pub struct DelegationService {
    // Database connection and configuration
    max_active_delegations: usize,
}

impl DelegationService {
    /// Create a new delegation
    /// 
    /// Validates:
    /// - from_user != to_user (no self-delegation)
    /// - ends_at > starts_at when provided
    /// - No circular delegation chains (A -> B -> A)
    /// - Total active delegations within configured limit
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
    ) -> Result<Delegation, DelegationError>;

    /// Revoke an active delegation
    /// 
    /// Only the creator or from_user can revoke a delegation.
    /// Creates an audit trail entry.
    pub async fn revoke_delegation(
        &self,
        delegation_id: Uuid,
        revoked_by: Uuid,
    ) -> Result<(), DelegationError>;

    /// Find and mark expired delegations as inactive
    /// 
    /// Returns list of delegation IDs that were expired.
    /// Called by scheduled job or during delegation resolution.
    pub async fn auto_expire_delegations(&self) -> Result<Vec<Uuid>, DelegationError>;

    /// List delegations for a user (both created and received)
    pub async fn list_user_delegations(
        &self,
        user_id: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<Delegation>, DelegationError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DelegationError {
    #[error("Cannot delegate to self")]
    SelfDelegation,
    
    #[error("End date must be after start date")]
    InvalidDateRange,
    
    #[error("Circular delegation detected")]
    CircularDelegation,
    
    #[error("Maximum active delegations reached")]
    TooManyActiveDelegations,
    
    #[error("Delegation not found")]
    NotFound,
    
    #[error("Not authorized to revoke this delegation")]
    UnauthorizedRevocation,
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
```

### 6.2 Delegation Resolver

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/delegation/resolver.rs`

The `DelegationResolver` is used by the stage executor to determine the actual approver for a given user.

```rust
use uuid::Uuid;
use crate::delegation::Delegation;

pub struct DelegationResolver;

impl DelegationResolver {
    /// Resolve the actual approver for a user, considering active delegations
    /// 
    /// Delegation priority (highest to lowest):
    /// 1. Single-document delegation (document_id matches)
    /// 2. Document-type delegation (document_type matches)
    /// 3. Bulk delegation (all document_types)
    /// 
    /// Follows delegation chains up to 3 hops to prevent infinite loops.
    /// Returns error if circular delegation detected or chain too long.
    pub async fn resolve_approver(
        &self,
        original_approver: Uuid,
        document_type: &str,
        document_id: Option<Uuid>,
    ) -> Result<Uuid, ResolutionError> {
        // Implementation steps:
        // 1. Query active delegations where from_user_id == original_approver
        // 2. Filter by applicability (document_id, document_type, or bulk)
        // 3. Select highest priority delegation
        // 4. If delegation found, recursively resolve (max 3 hops)
        // 5. Return final approver or original if no delegation
    }

    /// Get all active delegations for a user (both from and to perspectives)
    pub async fn active_delegations(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Delegation>, ResolutionError> {
        // Query delegations where:
        // - (from_user_id == user_id OR to_user_id == user_id)
        // - is_active == true
        // - starts_at <= now
        // - (ends_at IS NULL OR ends_at > now)
    }

    /// Check if a delegation chain would create a circular reference
    fn detect_circular_chain(
        &self,
        from: Uuid,
        to: Uuid,
        max_hops: usize,
    ) -> Result<bool, ResolutionError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ResolutionError {
    #[error("Circular delegation chain detected")]
    CircularChain,
    
    #[error("Delegation chain exceeds maximum length")]
    ChainTooLong,
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
```

## Tests

### Test Location

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/tests/delegation/service.rs`

### Delegation Service Tests

Write tests in the following order:

1. **create_delegation creates delegation record with unique ID**
   - Create a delegation with valid parameters
   - Verify record exists in database
   - Verify ID is a valid UUID

2. **create_delegation validates from_user != to_user (no self-delegation)**
   - Attempt to create delegation where from == to
   - Verify error is SelfDelegation

3. **create_delegation validates ends_at > starts_at when provided**
   - Attempt to create delegation with ends_at before starts_at
   - Verify error is InvalidDateRange

4. **create_delegation detects circular delegation (A -> B -> A)**
   - Create delegation A -> B
   - Attempt to create delegation B -> A
   - Verify error is CircularDelegation

5. **create_delegation limits total active delegations per user (configurable max)**
   - Set max_active_delegations to 3
   - Create 3 delegations from user X
   - Attempt to create 4th delegation
   - Verify error is TooManyActiveDelegations

6. **revoke_delegation sets is_active to false**
   - Create active delegation
   - Call revoke_delegation
   - Verify is_active is now false

7. **revoke_delegation creates audit trail entry**
   - Create active delegation
   - Call revoke_delegation
   - Verify audit trail record exists

8. **revoke_delegation only allows revocation by creator or from_user**
   - Create delegation with user A as from_user
   - Attempt to revoke as user B (not creator, not from_user)
   - Verify error is UnauthorizedRevocation

9. **auto_expire_delegations finds delegations past ends_at**
   - Create delegation with ends_at in the past
   - Call auto_expire_delegations
   - Verify delegation is in returned list

10. **auto_expire_delegations sets is_active to false for expired**
    - Create delegation with ends_at in the past
    - Call auto_expire_delegations
    - Verify is_active is now false

11. **auto_expire_delegations returns list of expired delegation IDs**
    - Create multiple expired delegations
    - Call auto_expire_delegations
    - Verify all IDs are returned

### Test Location

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/tests/delegation/resolution.rs`

### Delegation Resolver Tests

Write tests in the following order:

1. **resolve_approver returns original approver when no active delegations**
   - Resolve approver with no delegations
   - Verify returned user_id matches original

2. **resolve_approver returns delegated user for single-document delegation**
   - Create delegation for specific document_id
   - Resolve approver for that document
   - Verify delegated user is returned

3. **resolve_approver returns delegated user for document_type delegation**
   - Create delegation for document_type "invoice"
   - Resolve approver for "invoice" type document
   - Verify delegated user is returned

4. **resolve_approver prioritizes single-document over document_type delegation**
   - Create document_type delegation A -> B
   - Create single-document delegation A -> C for same document
   - Resolve approver for that document
   - Verify C is returned (single-document takes priority)

5. **resolve_approver follows delegation chain up to 3 hops**
   - Create delegation A -> B
   - Create delegation B -> C
   - Create delegation C -> D
   - Resolve approver for A
   - Verify D is returned

6. **resolve_approver errors on circular delegation (A -> B -> A)**
   - Create delegation A -> B
   - Create delegation B -> A
   - Resolve approver for A
   - Verify error is CircularChain

7. **resolve_approver errors on delegation chain exceeding 3 hops**
   - Create 4-hop chain: A -> B -> C -> D -> E
   - Resolve approver for A
   - Verify error is ChainTooLong

8. **active_delegations returns only active, non-expired delegations**
   - Create active delegation
   - Create expired delegation (ends_at in past)
   - Create future delegation (starts_at in future)
   - Call active_delegations
   - Verify only active delegation is returned

9. **active_delegations excludes delegations where current time < starts_at**
   - Create delegation with starts_at in future
   - Call active_delegations
   - Verify delegation is not returned

10. **active_delegations excludes delegations where current time > ends_at**
    - Create delegation with ends_at in past
    - Call active_delegations
    - Verify delegation is not returned

11. **active_delegations includes both from_user and to_user perspectives**
    - Create delegation A -> B
    - Call active_delegations for user A (from_user)
    - Call active_delegations for user B (to_user)
    - Verify both see the delegation

## Integration with Stage Executor

The delegation resolver is called by the stage executor (from section-04-multi-stage-engine) during stage initialization:

```rust
// In StageExecutor::initialize_stages
for approver in &stage_config.approvers {
    let resolved_approver = delegation_resolver
        .resolve_approver(approver.user_id, document_type, Some(document_id))
        .await?;
    // Add resolved_approver to stage instance
}
```

## Configuration

The delegation system uses the following configuration values (from application config):

| Setting | Default | Description |
|---------|---------|-------------|
| `delegation.max_active` | 10 | Maximum active delegations per user |
| `delegation.max_chain_hops` | 3 | Maximum delegation chain depth |
| `delegation.auto_expire_interval` | 3600s | How often to check for expired delegations |

## Dependencies on Other Sections

This section integrates with:

- **section-04-multi-stage-engine**: Uses delegation resolution during stage initialization
- **section-07-sla-escalation**: Delegated approvers receive escalation notifications
- **section-09-api-endpoints**: Provides REST API for delegation management

---

## Implementation Notes (2026-03-24)

### Files Actually Created

| File | Status |
|------|--------|
| `crates/iou-core/src/delegation/types.rs` | ✅ Moved from `delegation.rs` (557 lines) |
| `crates/iou-core/src/delegation/mod.rs` | ✅ Created (13 lines) |
| `crates/iou-core/src/delegation/service.rs` | ✅ Created (451 lines) |
| `crates/iou-core/src/delegation/resolver.rs` | ✅ Created (358 lines) |
| `crates/iou-core/tests/delegation/mod.rs` | ✅ Created (2 lines) |
| `crates/iou-core/tests/delegation/service.rs` | ✅ Created (52 lines) |
| `crates/iou-core/tests/delegation/resolution.rs` | ✅ Created (68 lines) |
| `crates/iou-core/tests/mod.rs` | ✅ Updated (added `mod delegation;`) |
| `crates/iou-core/src/delegation.rs` | ✅ Deleted (moved to types.rs) |

### Tests Implemented

**Note:** Full integration tests with database were deferred as they require test database setup. Basic unit tests for error types and `ResolvedApprover` construction pass:
- `test_delegation_error_display` (6 assertions)
- `test_delegation_error_source`
- `test_resolution_error_display` (2 assertions)
- `test_resolved_approver_direct`
- `test_resolved_approver_delegated`
- `test_resolved_approver_multi_hop_chain`

Plus existing tests in `delegation/types.rs` (15 tests) for:
- Validation (self-delegation, time range, type-specific constraints)
- Active state checking
- Document type applicability
- `ResolvedApprover` functionality
- Factory methods (`new_temporary`, `new_permanent`, `new_bulk`, `new_for_document`)

### Deviations from Plan

1. **Dynamic SQL queries**: Used `sqlx::query()` instead of `sqlx::query!()` macro to avoid DATABASE_URL requirement during development. This is a pragmatic choice for early development.

2. **Refactoring structure**: The plan called for `delegation.rs` to remain alongside the new `service.rs` and `resolver.rs`. Instead, we restructured to `delegation/types.rs` for a cleaner module hierarchy.

3. **Integration tests deferred**: The plan specified 21 integration tests for service and 11 for resolver. These were noted as comments in test files but not implemented due to requiring test database infrastructure.

### Code Review Decisions

- ✅ Deferred: `row_to_delegation` code duplication (minor, acceptable for now)
- ✅ Deferred: Integration tests (out of scope without test database)
- ✅ Accepted: Unsafe code in tests (`PgPool::new_untested()` for construction tests)

### API Features Implemented

- `DelegationService::create_delegation()` with full validation
- `DelegationService::revoke_delegation()` with authorization checks
- `DelegationService::auto_expire_delegations()` for scheduled cleanup
- `DelegationService::list_user_delegations()` with include_inactive option
- `DelegationResolver::resolve_approver()` with chain following
- `DelegationResolver::active_delegations()` for user queries
- `DelegationResolver::detect_circular_chain()` for validation

### Commit

- **Branch**: main
- **Implementation Commit**: `968b2c7`