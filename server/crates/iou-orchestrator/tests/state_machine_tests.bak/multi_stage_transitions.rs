//! Multi-stage state transition tests
//!
//! Tests for the multi-stage approval workflow state machine transitions.

use iou_orchestrator::state_machine::multi_stage::{
    transition_to_next_stage, evaluate_stage_completion, is_valid_transition,
    StageCompletionStatus, WorkflowTransition,
};
use iou_core::workflows::multi_stage::{
    StageInstance, StageStatus, ApprovalType, ApprovalDecision, ApprovalResponse,
};
use uuid::Uuid;
use chrono::Utc;

fn create_test_stage(
    stage_id: String,
    approvers: Vec<Uuid>,
    status: StageStatus,
) -> StageInstance {
    StageInstance {
        id: Uuid::new_v4(),
        document_id: Uuid::new_v4(),
        stage_id,
        status,
        started_at: None,
        completed_at: None,
        deadline: None,
        approvers,
        approvals_received: Vec::new(),
    }
}

#[test]
fn test_transition_to_next_stage_returns_next_stage_when_current_completes() {
    let document_id = Uuid::new_v4();
    let stage1 = create_test_stage(
        "stage_1".to_string(),
        vec![Uuid::new_v4()],
        StageStatus::Completed,
    );
    let stage2 = create_test_stage(
        "stage_2".to_string(),
        vec![Uuid::new_v4()],
        StageStatus::Pending,
    );
    let stage3 = create_test_stage(
        "stage_3".to_string(),
        vec![Uuid::new_v4()],
        StageStatus::Pending,
    );

    let all_stages = vec![stage1.clone(), stage2.clone(), stage3];

    let result = transition_to_next_stage(
        iou_orchestrator::state_machine::WorkflowState::AwaitingApproval,
        &stage1,
        &all_stages,
    );

    assert!(!result.is_final);
    assert_eq!(result.next_stage.unwrap().stage_id, "stage_2");
}

#[test]
fn test_transition_to_next_stage_returns_completed_when_all_stages_done() {
    let stage1 = create_test_stage(
        "stage_1".to_string(),
        vec![Uuid::new_v4()],
        StageStatus::Completed,
    );
    let stage2 = create_test_stage(
        "stage_2".to_string(),
        vec![Uuid::new_v4()],
        StageStatus::Completed,
    );
    let stage3 = create_test_stage(
        "stage_3".to_string(),
        vec![Uuid::new_v4()],
        StageStatus::Completed,
    );

    let all_stages = vec![stage1.clone(), stage2, stage3];

    let result = transition_to_next_stage(
        iou_orchestrator::state_machine::WorkflowState::InReview,
        &stage3,
        &all_stages,
    );

    assert!(result.is_final);
    assert!(result.next_stage.is_none());
    assert_eq!(
        result.current_state,
        iou_orchestrator::state_machine::WorkflowState::Completed
    );
}

#[test]
fn test_transition_to_next_stage_preserves_document_id() {
    let document_id = Uuid::new_v4();
    let stage1 = StageInstance {
        id: Uuid::new_v4(),
        document_id,
        stage_id: "stage_1".to_string(),
        status: StageStatus::Completed,
        started_at: None,
        completed_at: None,
        deadline: None,
        approvers: vec![],
        approvals_received: vec![],
    };
    let stage2 = StageInstance {
        id: Uuid::new_v4(),
        document_id,
        stage_id: "stage_2".to_string(),
        status: StageStatus::Pending,
        started_at: None,
        completed_at: None,
        deadline: None,
        approvers: vec![],
        approvals_received: vec![],
    };

    let all_stages = vec![stage1.clone(), stage2];

    let result = transition_to_next_stage(
        iou_orchestrator::state_machine::WorkflowState::AwaitingApproval,
        &stage1,
        &all_stages,
    );

    assert_eq!(result.next_stage.unwrap().document_id, document_id);
}

#[test]
fn test_evaluate_stage_completion_returns_complete_when_all_required_approvals_received() {
    let approver1 = Uuid::new_v4();
    let approver2 = Uuid::new_v4();
    let mut stage = create_test_stage(
        "review".to_string(),
        vec![approver1, approver2],
        StageStatus::InProgress,
    );
    stage.approvals_received = vec![
        ApprovalResponse::new(approver1, ApprovalDecision::Approved, None),
        ApprovalResponse::new(approver2, ApprovalDecision::Approved, None),
    ];

    let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelAll);

    assert_eq!(result, StageCompletionStatus::Complete);
}

#[test]
fn test_evaluate_stage_completion_returns_in_progress_when_partial_approvals() {
    let approver1 = Uuid::new_v4();
    let approver2 = Uuid::new_v4();
    let mut stage = create_test_stage(
        "review".to_string(),
        vec![approver1, approver2],
        StageStatus::InProgress,
    );
    stage.approvals_received = vec![ApprovalResponse::new(
        approver1,
        ApprovalDecision::Approved,
        None,
    )];

    let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelAll);

    assert_eq!(result, StageCompletionStatus::InProgress);
}

#[test]
fn test_evaluate_stage_completion_returns_failed_when_rejection_received() {
    let approver1 = Uuid::new_v4();
    let approver2 = Uuid::new_v4();
    let mut stage = create_test_stage(
        "review".to_string(),
        vec![approver1, approver2],
        StageStatus::InProgress,
    );
    stage.approvals_received = vec![
        ApprovalResponse::new(approver1, ApprovalDecision::Approved, None),
        ApprovalResponse::new(approver2, ApprovalDecision::Rejected, None),
    ];

    let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelAll);

    assert_eq!(result, StageCompletionStatus::Failed);
}

#[test]
fn test_evaluate_stage_completion_returns_expired_when_deadline_passed() {
    let approver = Uuid::new_v4();
    let mut stage = create_test_stage(
        "review".to_string(),
        vec![approver],
        StageStatus::Expired,
    );
    stage.approvals_received = vec![ApprovalResponse::new(
        approver,
        ApprovalDecision::Approved,
        None,
    )];

    let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelAny);

    assert_eq!(result, StageCompletionStatus::Expired);
}

#[test]
fn test_evaluate_stage_completion_handles_parallel_any_with_single_approval() {
    let approver1 = Uuid::new_v4();
    let approver2 = Uuid::new_v4();
    let mut stage = create_test_stage(
        "review".to_string(),
        vec![approver1, approver2],
        StageStatus::InProgress,
    );
    stage.approvals_received = vec![ApprovalResponse::new(
        approver1,
        ApprovalDecision::Approved,
        None,
    )];

    let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelAny);

    assert_eq!(result, StageCompletionStatus::Complete);
}

#[test]
fn test_evaluate_stage_completion_handles_parallel_all_requiring_all_approvers() {
    let approver1 = Uuid::new_v4();
    let approver2 = Uuid::new_v4();
    let mut stage = create_test_stage(
        "review".to_string(),
        vec![approver1, approver2],
        StageStatus::InProgress,
    );
    stage.approvals_received = vec![ApprovalResponse::new(
        approver1,
        ApprovalDecision::Approved,
        None,
    )];

    let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelAll);

    assert_eq!(result, StageCompletionStatus::InProgress);
}

#[test]
fn test_evaluate_stage_completion_handles_parallel_majority_with_50_percent_threshold() {
    // With 5 approvers, need 3 for majority
    let approvers: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
    let mut stage = create_test_stage(
        "review".to_string(),
        approvers.clone(),
        StageStatus::InProgress,
    );

    // Add 2 approvals - should be InProgress
    stage.approvals_received = vec![
        ApprovalResponse::new(approvers[0], ApprovalDecision::Approved, None),
        ApprovalResponse::new(approvers[1], ApprovalDecision::Approved, None),
    ];

    let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelMajority);
    assert_eq!(result, StageCompletionStatus::InProgress);

    // Add third approval - should be Complete
    stage.approvals_received.push(ApprovalResponse::new(
        approvers[2],
        ApprovalDecision::Approved,
        None,
    ));

    let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelMajority);
    assert_eq!(result, StageCompletionStatus::Complete);
}

#[test]
fn test_is_valid_transition_allows_valid_transitions() {
    assert!(is_valid_transition(StageStatus::Pending, StageStatus::InProgress));
    assert!(is_valid_transition(StageStatus::InProgress, StageStatus::Completed));
    assert!(is_valid_transition(StageStatus::InProgress, StageStatus::Expired));
    assert!(is_valid_transition(StageStatus::Pending, StageStatus::Skipped));
}

#[test]
fn test_state_machine_rejects_invalid_transition() {
    // Cannot go from Completed back to InProgress
    assert!(!is_valid_transition(StageStatus::Completed, StageStatus::InProgress));

    // Cannot go from Completed to Pending
    assert!(!is_valid_transition(StageStatus::Completed, StageStatus::Pending));

    // Cannot go from Expired to InProgress
    assert!(!is_valid_transition(StageStatus::Expired, StageStatus::InProgress));

    // Cannot go from Skipped to InProgress
    assert!(!is_valid_transition(StageStatus::Skipped, StageStatus::InProgress));
}

#[test]
fn test_is_valid_transition_allows_same_state() {
    assert!(is_valid_transition(StageStatus::Pending, StageStatus::Pending));
    assert!(is_valid_transition(StageStatus::InProgress, StageStatus::InProgress));
    assert!(is_valid_transition(StageStatus::Completed, StageStatus::Completed));
    assert!(is_valid_transition(StageStatus::Skipped, StageStatus::Skipped));
    assert!(is_valid_transition(StageStatus::Expired, StageStatus::Expired));
}

#[test]
fn test_evaluate_stage_completion_sequential_requires_all_approvals() {
    let approvers: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
    let mut stage = create_test_stage(
        "review".to_string(),
        approvers.clone(),
        StageStatus::InProgress,
    );

    // With 2 out of 3 approvals, still in progress
    stage.approvals_received = vec![
        ApprovalResponse::new(approvers[0], ApprovalDecision::Approved, None),
        ApprovalResponse::new(approvers[1], ApprovalDecision::Approved, None),
    ];

    let result = evaluate_stage_completion(&stage, &ApprovalType::Sequential);
    assert_eq!(result, StageCompletionStatus::InProgress);

    // With all 3, complete
    stage.approvals_received.push(ApprovalResponse::new(
        approvers[2],
        ApprovalDecision::Approved,
        None,
    ));

    let result = evaluate_stage_completion(&stage, &ApprovalType::Sequential);
    assert_eq!(result, StageCompletionStatus::Complete);
}

#[test]
fn test_transition_to_next_stage_handles_stage_not_found() {
    let stage1 = create_test_stage(
        "stage_1".to_string(),
        vec![Uuid::new_v4()],
        StageStatus::Completed,
    );
    let stage2 = create_test_stage(
        "stage_2".to_string(),
        vec![Uuid::new_v4()],
        StageStatus::Pending,
    );

    let all_stages = vec![stage1.clone(), stage2];

    // Try to transition using a stage that's not in the list
    let unknown_stage = create_test_stage(
        "unknown".to_string(),
        vec![Uuid::new_v4()],
        StageStatus::Completed,
    );

    let result = transition_to_next_stage(
        iou_orchestrator::state_machine::WorkflowState::AwaitingApproval,
        &unknown_stage,
        &all_stages,
    );

    // Should return current state without changing
    assert!(!result.is_final);
    assert!(result.next_stage.is_none());
    assert_eq!(
        result.current_state,
        iou_orchestrator::state_machine::WorkflowState::AwaitingApproval
    );
}
