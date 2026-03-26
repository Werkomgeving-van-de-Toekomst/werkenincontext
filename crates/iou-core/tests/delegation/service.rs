//! Integration tests for DelegationService

use iou_core::delegation::DelegationError;
use std::error::Error;

// Note: These tests would require a test database.
// For now, we test the error types and basic construction.

#[test]
fn test_delegation_error_display() {
    assert_eq!(
        format!("{}", DelegationError::SelfDelegation),
        "Cannot delegate to self"
    );
    assert_eq!(
        format!("{}", DelegationError::InvalidDateRange),
        "End date must be after start date"
    );
    assert_eq!(
        format!("{}", DelegationError::CircularDelegation),
        "Circular delegation detected"
    );
    assert_eq!(
        format!("{}", DelegationError::TooManyActiveDelegations),
        "Maximum active delegations reached"
    );
    assert_eq!(format!("{}", DelegationError::NotFound), "Delegation not found");
    assert_eq!(
        format!("{}", DelegationError::UnauthorizedRevocation),
        "Not authorized to revoke this delegation"
    );
}

#[test]
fn test_delegation_error_source() {
    let err = DelegationError::NotFound;
    assert!(err.source().is_none());
}

// Integration tests would go here with a test database fixture:
// - test_create_delegation_creates_record
// - test_create_delegation_validates_no_self_delegation
// - test_create_delegation_validates_date_range
// - test_create_delegation_detects_circular_delegation
// - test_create_delegation_limits_active_delegations
// - test_revoke_delegation_sets_inactive
// - test_revoke_delegation_creates_audit_entry
// - test_revoke_delegation_authorizes_only_creator_or_from_user
// - test_auto_expire_delegations_finds_expired
// - test_auto_expire_delegations_sets_inactive
// - test_auto_expire_delegations_returns_expired_ids
// - test_list_user_delegations_includes_from_and_to
