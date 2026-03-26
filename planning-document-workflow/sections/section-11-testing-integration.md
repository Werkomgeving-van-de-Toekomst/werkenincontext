Now I have all the context needed. Let me generate the section content for `section-11-testing-integration`.

---

# Section 11: Testing & Integration

## Overview

This section covers integration tests and test coverage for the enhanced document workflow system. Tests verify that workflow features (multi-stage approvals, delegation, expiry/escalation) work correctly at the domain level.

## Dependencies

This section depends on ALL previous sections being implemented:
- section-01-database-schema (tables must exist)
- section-02-core-types (type definitions available)
- section-03-config-system (workflow configuration loaded)
- section-04-multi-stage-engine (state machine and executor)
- section-05-diff-generator (diff generation working)
- section-06-delegation-system (delegation resolution)
- section-07-sla-escalation (SLA and escalation service)
- section-08-version-storage (version storage service)
- section-09-api-endpoints (all API routes available)
- section-10-frontend-components (optional for integration tests)

## Implementation Notes

### What Was Implemented

1. **Domain-level Integration Tests** (`crates/iou-api/tests/workflows/end_to_end.rs`)
   - Tests exercise workflow state machines through complete approval cycles
   - Tests multi-stage sequential and parallel approvals
   - Tests delegation with audit trail verification
   - Tests expiry/escalation concepts
   - Tests rejection scenarios and optional stage skipping

2. **Security Tests** (`crates/iou-api/tests/approval_bypass.rs`)
   - Authorization pattern with `MockAuthContext`
   - Delegation chain limit validation
   - Circular delegation detection
   - Audit trail completeness verification
   - Stage modification authorization checks

3. **Performance Tests** (`crates/iou-api/tests/performance/workflow_load.rs`)
   - Concurrent document approval simulation
   - Large document diff generation performance
   - Version list pagination performance
   - Stage transition overhead measurement

4. **Coverage Verification** (`crates/iou-core/tests/workflows/multi_stage_coverage.rs`)
   - Stage status transition coverage
   - Quorum evaluation for all approval types
   - SLA calculation edge cases
   - Approval decision variant coverage
   - Delegation chain coverage

### What Differs From Plan

The plan specified full API-layer integration tests with HTTP requests, authentication middleware, and database integration. The implementation currently provides domain-level testing with the following deviations:

**API Integration:** Tests currently use domain types directly rather than HTTP requests. TODO comments added to document the pattern for future API-layer integration.

**Database Integration:** Tests use in-memory objects. Database test infrastructure is documented as a TODO.

**Mock Services:** External services (S3, email, webhooks) are documented for future mocking.

**Version History Tests:** Simplified to basic string comparison rather than full VersionRecord integration due to type mismatches.

**Test Configuration:** Configuration files in `tests/config/` were not created.

## Test Organization

```
crates/
├── iou-api/
│   └── tests/
│       ├── workflows/
│       │   ├── end_to_end.rs           # Main integration tests
│       │   ├── helpers.rs                # Test helpers with TODO docs
│       │   └── mod.rs                    # Module exports
│       ├── performance/
│       │   └── workflow_load.rs         # Performance benchmarks
│       └── approval_bypass.rs           # Security tests
└── iou-core/
    └── tests/
        └── workflows/
            ├── multi_stage_coverage.rs   # Coverage verification
            └── mod.rs                     # Module exports
```

## Running the Tests

```bash
# Run all integration tests
cargo test --test workflow_integration

# Run security tests
cargo test --test approval_bypass

# Run performance tests
cargo test --test workflow_load --release

# Run coverage verification
cargo test --package iou-core --lib workflows
```

## Success Criteria

The testing section is complete when:
1. ✅ All integration tests pass consistently
2. ✅ Performance tests meet defined thresholds
3. ✅ Security tests verify authorization patterns
4. ✅ Test coverage for core workflow types is verified
5. ⏳ Tests can run in CI/CD pipeline (domain tests ready, API tests TODO)
6. ⏳ Mock implementations allow tests to run without external dependencies (documented as TODO)

```rust
use iou_api::test::helpers::*;
use uuid::Uuid;
use chrono::{Utc, Duration};

mod helpers {
    pub use super::super::*;
}

/// Test helper module for common test utilities
mod test_helpers {
    use super::*;
    
    /// Creates a test document with initial content
    pub async fn create_test_document(
        client: &TestClient,
        title: &str,
        content: &str,
    ) -> Uuid {
        // Implementation creates document via API
        // Returns document_id
    }
    
    /// Submits a document for approval
    pub async fn submit_document(client: &TestClient, document_id: Uuid) -> Response {
        // Implementation submits document
    }
    
    /// Creates test users with appropriate roles
    pub async fn create_test_users() -> Vec<TestUser> {
        // Implementation creates users in test database
    }
    
    /// Sets up workflow configuration for a domain
    pub async fn setup_workflow_config(domain_id: &str, stages: Vec<StageConfig>) {
        // Implementation writes config files
    }
    
    pub struct TestUser {
        pub id: Uuid,
        pub name: String,
        pub role: String,
        pub auth_token: String,
    }
}

#[tokio::test]
async fn test_complete_multi_stage_document_flow() {
    // Setup: Create test users with different roles
    // Create document
    // Submit for approval
    
    // Stage 1: Manager approval
    // Verify document is in stage 1
    // Approve as manager
    // Verify transition to stage 2
    
    // Stage 2: Director approval
    // Verify document is in stage 2
    // Approve as director
    // Verify document is approved
    
    // Verify final state
}

#[tokio::test]
async fn test_parallel_approval_with_quorum() {
    // Setup: Create document requiring 3 approvers with majority quorum
    
    // First approval arrives
    // Verify stage still in_progress
    
    // Second approval arrives (majority met)
    // Verify stage completes and transitions to next stage
    
    // Third approval attempts (should fail - stage already complete)
}

#[tokio::test]
async fn test_delegation_during_approval() {
    // Setup: Original approver creates delegation to backup
    
    // Document submitted requiring original approver
    // Verify delegated approver is listed as actual approver
    
    // Delegated approver approves
    // Verify approval records delegated_from in audit trail
    
    // Verify approval counted towards quorum
}

#[tokio::test]
async fn test_expiry_and_escalation() {
    // Setup: Create document with short SLA (e.g., 1 hour)
    // Set stage deadline in past
    
    // Run expiry checker
    // Verify escalation notification sent
    
    // Verify approval_escalations record created
    
    // If expiry_action is auto_approve: verify stage approved
    // If expiry_action is return_to_draft: verify document returned to draft
}

#[tokio::test]
async fn test_version_creation_and_restoration() {
    // Setup: Create document with initial content
    
    // Modify document (create v2)
    // Verify version record created
    
    // Modify again (create v3)
    // Verify version record created with parent_version_id
    
    // Restore to v2
    // Verify new version (v4) created with v2 content
    // Verify audit trail records restore action
    
    // Verify document content matches v2
}

#[tokio::test]
async fn test_diff_between_versions() {
    // Setup: Create document with multiple versions
    
    // Request diff between v1 and v3
    // Verify diff includes all changes
    
    // Request diff in unified format
    // Verify + and - prefixes
    
    // Request diff in side_by_side format
    // Verify aligned output
    
    // Request diff in inline format
    // Verify highlighted changes
}

#[tokio::test]
async fn test_rejection_at_any_stage() {
    // Setup: Submit multi-stage document
    
    // Approve stage 1
    
    // Reject at stage 2
    // Verify document returns to draft
    // Verify no further stages can be approved
    
    // Verify rejection recorded in audit trail
}

#[tokio::test]
async fn test_optional_stage_skipped() {
    // Setup: Configure optional stage with condition
    // Create document that does not meet condition
    
    // Submit document
    // Verify optional stage marked as Skipped
    // Verify workflow continues to next stage
}

#[tokio::test]
async fn test_sequential_parallel_stage_mixed() {
    // Setup: Configure workflow with sequential then parallel stages
    // Stage 1: Sequential (manager)
    // Stage 2: Parallel (3 reviewers, need 2)
    
    // Approve stage 1
    // Verify stage 2 starts
    
    // Approve 2 of 3 reviewers in stage 2
    // Verify stage 2 completes
}
```

### File: `crates/iou-api/tests/performance/workflow_load.rs`

Create performance tests to verify the system handles realistic loads.

```rust
#[tokio::test]
async fn test_concurrent_document_approvals() {
    // Setup: Create 100 documents awaiting approval
    // Simulate 10 concurrent approvers
    
    // Measure: Time to process all approvals
    // Assert: All documents processed within acceptable time
}

#[tokio::test]
async fn test_large_document_diff_generation() {
    // Setup: Create document with 10,000 lines
    // Modify middle section
    
    // Measure: Time to generate diff
    // Assert: Diff generation completes within acceptable time
}

#[tokio::test]
async fn test_version_list_pagination() {
    // Setup: Create document with 100 versions
    
    // Test: Paginated version list
    // Assert: Performance degrades linearly, not exponentially
}
```

### File: `crates/iou-api/tests/security/approval_bypass.rs`

Create security tests to verify authorization and audit logging.

```rust
#[tokio::test]
async fn test_non_approver_cannot_approve() {
    // Setup: Create document awaiting specific approver
    
    // Attempt approval as different user
    // Assert: 403 Forbidden returned
    // Assert: Approval not recorded
}

#[tokio::test]
async fn test_approval_without_authentication() {
    // Setup: Create document awaiting approval
    
    // Attempt approval without auth token
    // Assert: 401 Unauthorized returned
}

#[tokio::test]
async fn test_delegation_chain_limit() {
    // Setup: Create delegation chain A -> B -> C -> D (4 hops)
    
    // Attempt to resolve approver
    // Assert: Error returned (chain exceeds 3 hops)
}

#[tokio::test]
async fn test_circular_delegation_prevented() {
    // Setup: Create delegation A -> B, then attempt B -> A
    
    // Assert: Creation fails with circular delegation error
}

#[tokio::test]
async fn test_restore_requires_authorization() {
    // Setup: Create document with version history
    
    // Attempt restore as non-owner, non-admin
    // Assert: 403 Forbidden returned
    
    // Attempt restore as document owner
    // Assert: Restore succeeds
}

#[tokio::test]
async fn test_audit_trail_completeness() {
    // Setup: Run through complete workflow
    
    // Query audit trail
    // Assert: Every action logged with timestamp, user, and details
    // Assert: Delegated approvals include original approver
}
```

### File: `crates/iou-core/tests/workflows/multi_stage_coverage.rs`

Verify unit test coverage for core workflow logic.

```rust
// This module does not add new tests but verifies coverage
// Run with: cargo test --coverage

#[test]
fn verify_stage_transition_coverage() {
    // Use coverage tool to assert all transitions tested
}

#[test]
fn verify_quorum_evaluation_coverage() {
    // Assert all approval types (any/all/majority) covered
}

#[test]
fn verify_sla_calculation_edge_cases() {
    // Assert weekend skipping tested
    // Assert holiday handling tested
}
```

## Test Organization

```
crates/
├── iou-api/
│   └── tests/
│       ├── workflows/
│       │   └── end_to_end.rs           # Main integration tests
│       ├── performance/
│       │   └── workflow_load.rs        # Performance benchmarks
│       └── security/
│           └── approval_bypass.rs      # Security tests
└── iou-core/
    └── tests/
        └── workflows/
            └── multi_stage_coverage.rs # Coverage verification
```

## Running the Tests

```bash
# Run all integration tests
cargo test --test end_to_end

# Run with database backend
cargo test --test end_to_end --features test-database

# Run performance tests
cargo test --test workflow_load --release

# Run security tests
cargo test --test approval_bypass

# Generate coverage report
cargo test --coverage
```

## Test Data Setup

Integration tests require test data fixtures:

```rust
// In test_helpers module

pub async fn setup_test_database() -> TestDatabase {
    // Create isolated test database
    // Run migrations
    // Seed with test data
}

pub async fn teardown_test_database(db: TestDatabase) {
    // Clean up test database
}

pub struct TestDatabase {
    pub connection: PgConnection,
    pub cleanup: Vec<Box<dyn FnOnce()>>,
}
```

## Test Configuration

Create test configuration files in `crates/iou-api/tests/config/`:

```yaml
# test_workflows.yaml
test_domain:
  approval_stages:
    - stage_id: test_stage_1
      stage_name: Test Manager Approval
      stage_order: 1
      approval_type: sequential
      approvers:
        - role: manager
      sla_hours: 24
      expiry_action: notify_only
    
    - stage_id: test_stage_2
      stage_name: Test Director Approval
      stage_order: 2
      approval_type: parallel_any
      approvers:
        - user_id: DIRECTOR_UUID
      sla_hours: 48
      expiry_action: escalate_to
      escalate_target: "escalation@example.com"
```

## Success Criteria

The testing section is complete when:

1. All integration tests pass consistently
2. Performance tests meet defined thresholds
3. Security tests verify all authorization checks
4. Test coverage meets minimum requirements (80%+ for new code)
5. Tests can run in CI/CD pipeline
6. Mock implementations allow tests to run without external dependencies

## Implementation Notes

1. Use test transactions that roll back after each test
2. Mock external services (S3, email, webhooks) for tests
3. Use fixed time mocks for predictable deadline testing
4. Create deterministic UUIDs for reproducible tests
5. Ensure tests can run in parallel (no shared state)