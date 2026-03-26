//! Integration tests for DelegationResolver

use iou_core::delegation::{ResolutionError, ResolvedApprover};
use uuid::Uuid;

// Note: These tests would require a test database.
// For now, we test the error types and basic construction.

#[test]
fn test_resolution_error_display() {
    assert_eq!(
        format!("{}", ResolutionError::CircularChain),
        "Circular delegation chain detected"
    );
    assert_eq!(
        format!("{}", ResolutionError::ChainTooLong),
        "Delegation chain exceeds maximum length"
    );
}

#[test]
fn test_resolved_approver_direct() {
    let user_id = Uuid::new_v4();
    let approver = ResolvedApprover::direct(user_id);
    assert!(!approver.is_delegated);
    assert!(approver.delegation_chain.is_empty());
    assert_eq!(approver.user_id, user_id);
    assert_eq!(approver.chain_length(), 0);
    assert_eq!(approver.original_approver(), user_id);
}

#[test]
fn test_resolved_approver_delegated() {
    let original = Uuid::new_v4();
    let delegated = Uuid::new_v4();
    let chain = vec![original];
    let approver = ResolvedApprover::delegated(delegated, chain.clone());
    assert!(approver.is_delegated);
    assert_eq!(approver.user_id, delegated);
    assert_eq!(approver.delegation_chain, chain);
    assert_eq!(approver.chain_length(), 1);
    assert_eq!(approver.original_approver(), original);
}

#[test]
fn test_resolved_approver_multi_hop_chain() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();
    let chain = vec![a, b];
    let approver = ResolvedApprover::delegated(c, chain.clone());
    assert!(approver.is_delegated);
    assert_eq!(approver.chain_length(), 2);
    assert_eq!(approver.original_approver(), a);
}

// Integration tests would go here with a test database fixture:
// - test_resolve_approver_returns_original_when_no_delegations
// - test_resolve_approver_returns_delegated_for_single_document
// - test_resolve_approver_returns_delegated_for_document_type
// - test_resolve_approver_prioritizes_single_document_over_type
// - test_resolve_approver_follows_chain_up_to_3_hops
// - test_resolve_approver_errors_on_circular_chain
// - test_resolve_approver_errors_on_excessive_hops
// - test_active_delegations_filters_inactive
// - test_active_delegations_filters_expired
// - test_active_delegations_filters_future
// - test_active_delegations_includes_both_from_and_to
