//! Integration tests for multi-stage workflow and delegation types
//!
//! Tests the interaction between different type modules and validates
//! serialization/deserialization for API compatibility.

use iou_core::workflows::*;
use iou_core::delegation::*;
use uuid::Uuid;

#[tokio::test]
async fn test_approval_stage_creates_with_all_fields() {
    let stage = ApprovalStage {
        stage_id: "legal_review".to_string(),
        stage_name: "Legal Review".to_string(),
        stage_order: 1,
        approval_type: ApprovalType::ParallelAll,
        approvers: vec![
            Approver {
                user_id: Some(Uuid::new_v4()),
                role: None,
            }
        ],
        sla_hours: 72,
        expiry_action: ExpiryAction::NotifyOnly,
        is_optional: false,
        condition: None,
    };
    assert_eq!(stage.stage_id, "legal_review");
    assert_eq!(stage.approvers.len(), 1);
}

#[tokio::test]
async fn test_stage_instance_roundtrip_serialization() {
    let instance = StageInstance::new(
        Uuid::new_v4(),
        "review".to_string(),
        vec![Uuid::new_v4(), Uuid::new_v4()],
    );

    let serialized = serde_json::to_string(&instance).unwrap();
    let deserialized: StageInstance = serde_json::from_str(&serialized).unwrap();

    assert_eq!(instance.id, deserialized.id);
    assert_eq!(instance.document_id, deserialized.document_id);
    assert_eq!(instance.stage_id, deserialized.stage_id);
    assert_eq!(instance.status, deserialized.status);
}

#[tokio::test]
async fn test_delegation_serialization_preserves_all_fields() {
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let delegation = Delegation::new_permanent(
        from,
        to,
        vec!["invoice".to_string()],
        from,
    );

    let serialized = serde_json::to_string(&delegation).unwrap();
    let deserialized: Delegation = serde_json::from_str(&serialized).unwrap();

    assert_eq!(delegation.id, deserialized.id);
    assert_eq!(delegation.from_user_id, deserialized.from_user_id);
    assert_eq!(delegation.to_user_id, deserialized.to_user_id);
    assert_eq!(delegation.delegation_type, deserialized.delegation_type);
    assert_eq!(delegation.document_types, deserialized.document_types);
}

#[tokio::test]
async fn test_resolved_approver_serialization() {
    let user_id = Uuid::new_v4();
    let chain = vec![Uuid::new_v4(), Uuid::new_v4()];
    let approver = ResolvedApprover::delegated(user_id, chain.clone());

    let serialized = serde_json::to_string(&approver).unwrap();
    let deserialized: ResolvedApprover = serde_json::from_str(&serialized).unwrap();

    assert_eq!(approver.user_id, deserialized.user_id);
    assert_eq!(approver.is_delegated, deserialized.is_delegated);
    assert_eq!(approver.delegation_chain, deserialized.delegation_chain);
}

#[tokio::test]
async fn test_expiry_action_all_variants_serialize() {
    let actions = vec![
        ExpiryAction::NotifyOnly,
        ExpiryAction::ReturnToDraft,
        ExpiryAction::AutoApprove,
        ExpiryAction::EscalateTo { target: "manager".to_string() },
    ];

    for action in actions {
        let serialized = serde_json::to_string(&action).unwrap();
        let deserialized: ExpiryAction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(action, deserialized);
    }
}

#[tokio::test]
async fn test_approval_decision_with_delegation_serializes() {
    let to = Uuid::new_v4();
    let decision = ApprovalDecision::Delegated { to };

    let serialized = serde_json::to_string(&decision).unwrap();
    let deserialized: ApprovalDecision = serde_json::from_str(&serialized).unwrap();

    match (decision, deserialized) {
        (ApprovalDecision::Delegated { to: t1 }, ApprovalDecision::Delegated { to: t2 }) => {
            assert_eq!(t1, t2);
        }
        _ => panic!("Delegation decision not preserved"),
    }
}

#[tokio::test]
async fn test_stage_status_all_transitions_valid() {
    let statuses = vec![
        StageStatus::Pending,
        StageStatus::InProgress,
        StageStatus::Completed,
        StageStatus::Skipped,
        StageStatus::Expired,
    ];

    for status in statuses {
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: StageStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }
}

#[tokio::test]
async fn test_delegation_type_all_variants_serialize() {
    let types = vec![
        DelegationType::Temporary,
        DelegationType::Permanent,
        DelegationType::Bulk,
    ];

    for dt in types {
        let serialized = serde_json::to_string(&dt).unwrap();
        let deserialized: DelegationType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(dt, deserialized);
    }
}

#[tokio::test]
async fn test_workflow_exports_available() {
    // Verify all multi-stage types are exported from workflows module
    let _stage = ApprovalStage {
        stage_id: "test".to_string(),
        stage_name: "Test".to_string(),
        stage_order: 1,
        approval_type: ApprovalType::Sequential,
        approvers: vec![],
        sla_hours: 24,
        expiry_action: ExpiryAction::NotifyOnly,
        is_optional: false,
        condition: None,
    };

    let _instance = StageInstance::new(Uuid::new_v4(), "test".to_string(), vec![]);
    let _status = StageStatus::Pending;
    let _decision = ApprovalDecision::Approved;
}

#[tokio::test]
async fn test_delegation_exports_available() {
    // Verify all delegation types are exported from crate root
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();

    let _delegation = Delegation::new_permanent(from, to, vec![], from);
    let _resolved = ResolvedApprover::direct(to);
    let _type = DelegationType::Temporary;
}
