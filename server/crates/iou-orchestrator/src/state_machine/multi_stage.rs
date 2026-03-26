//! Multi-stage approval workflow state machine
//!
//! Extends the base state machine to handle sequential progression through
//! multiple approval stages with support for parallel approval patterns.

use super::base::WorkflowState;
use iou_core::workflows::multi_stage::{StageInstance, StageStatus, ApprovalType, ApprovalDecision};

/// Result of a stage completion evaluation
#[derive(Debug, PartialEq, Clone)]
pub enum StageCompletionStatus {
    /// All required approvals received
    Complete,
    /// Still waiting for more approvals
    InProgress,
    /// A rejection was received
    Failed,
    /// Stage deadline passed
    Expired,
}

/// Represents a workflow transition result
#[derive(Debug, Clone)]
pub struct WorkflowTransition {
    /// The new current state
    pub current_state: WorkflowState,
    /// The next stage to execute, if any
    pub next_stage: Option<StageInstance>,
    /// Whether this is the final state (no more stages)
    pub is_final: bool,
}

/// Transition to the next stage in the workflow
///
/// # Arguments
/// * `current` - The current workflow state
/// * `completed_stage` - The stage instance that just completed
/// * `all_stages` - All configured stages for this document type
///
/// # Returns
/// A `WorkflowTransition` indicating the next state and stage
pub fn transition_to_next_stage(
    current: WorkflowState,
    completed_stage: &StageInstance,
    all_stages: &[StageInstance],
) -> WorkflowTransition {
    // Find the index of the completed stage
    let completed_index = all_stages
        .iter()
        .position(|s| s.stage_id == completed_stage.stage_id);

    match completed_index {
        Some(idx) if idx + 1 < all_stages.len() => {
            // There is a next stage
            WorkflowTransition {
                current_state: WorkflowState::AwaitingApproval,
                next_stage: Some(all_stages[idx + 1].clone()),
                is_final: false,
            }
        }
        Some(_) => {
            // This was the last stage - document is fully approved
            WorkflowTransition {
                current_state: WorkflowState::Completed,
                next_stage: None,
                is_final: true,
            }
        }
        None => {
            // Stage not found - should not happen in normal flow
            WorkflowTransition {
                current_state: current,
                next_stage: None,
                is_final: false,
            }
        }
    }
}

/// Evaluate whether a stage has completed based on received approvals
///
/// # Arguments
/// * `stage` - The stage instance to evaluate
/// * `approval_type` - The type of approval (sequential, parallel_any, etc.)
///
/// # Returns
/// A `StageCompletionStatus` indicating the stage's completion state
pub fn evaluate_stage_completion(
    stage: &StageInstance,
    approval_type: &ApprovalType,
) -> StageCompletionStatus {
    use StageStatus as SS;

    // Check if expired first
    if stage.status == SS::Expired {
        return StageCompletionStatus::Expired;
    }

    // Check for any rejection
    let has_rejection = stage.approvals_received.iter().any(|a| {
        matches!(a.decision, ApprovalDecision::Rejected)
    });
    if has_rejection {
        return StageCompletionStatus::Failed;
    }

    // Count total approvers and approvals received
    let total_approvers = stage.approvers.len() as i32;
    let approvals_count = stage.approvals_received.len() as i32;

    match approval_type {
        ApprovalType::Sequential => {
            // Sequential requires all approvers in order
            if approvals_count == total_approvers {
                StageCompletionStatus::Complete
            } else {
                StageCompletionStatus::InProgress
            }
        }
        ApprovalType::ParallelAny => {
            // Any single approval completes the stage
            if approvals_count >= 1 {
                StageCompletionStatus::Complete
            } else {
                StageCompletionStatus::InProgress
            }
        }
        ApprovalType::ParallelAll => {
            // All approvers must approve
            if approvals_count == total_approvers {
                StageCompletionStatus::Complete
            } else {
                StageCompletionStatus::InProgress
            }
        }
        ApprovalType::ParallelMajority => {
            // More than 50% must approve
            let required = (total_approvers / 2) + 1;
            if approvals_count >= required {
                StageCompletionStatus::Complete
            } else {
                StageCompletionStatus::InProgress
            }
        }
    }
}

/// Validate that a state transition is allowed
pub fn is_valid_transition(from: StageStatus, to: StageStatus) -> bool {
    use StageStatus as S;

    matches!(
        (from, to),
        (S::Pending, S::InProgress) |
        (S::InProgress, S::Completed) |
        (S::InProgress, S::Expired) |
        (S::Pending, S::Skipped) |
        // Allow staying in same state
        (S::Pending, S::Pending) |
        (S::InProgress, S::InProgress) |
        (S::Completed, S::Completed) |
        (S::Skipped, S::Skipped) |
        (S::Expired, S::Expired)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use iou_core::workflows::multi_stage::ApprovalResponse;

    fn create_test_stage(
        stage_id: &str,
        approvers: Vec<Uuid>,
        status: StageStatus,
    ) -> StageInstance {
        StageInstance {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            stage_id: stage_id.to_string(),
            status,
            started_at: None,
            completed_at: None,
            deadline: None,
            approvers,
            approvals_received: Vec::new(),
        }
    }

    #[test]
    fn test_transition_to_next_stage_with_more_stages() {
        let stage1 = create_test_stage("stage_1", vec![], StageStatus::Completed);
        let stage2 = create_test_stage("stage_2", vec![], StageStatus::Pending);
        let stage3 = create_test_stage("stage_3", vec![], StageStatus::Pending);

        let all_stages = vec![stage1.clone(), stage2.clone(), stage3];

        let result = transition_to_next_stage(WorkflowState::AwaitingApproval, &stage1, &all_stages);

        assert!(!result.is_final);
        assert_eq!(result.next_stage.unwrap().stage_id, "stage_2");
    }

    #[test]
    fn test_transition_to_next_stage_final_stage() {
        let stage1 = create_test_stage("stage_1", vec![], StageStatus::Completed);
        let stage2 = create_test_stage("stage_2", vec![], StageStatus::Completed);
        let stage3 = create_test_stage("stage_3", vec![], StageStatus::Completed);

        let all_stages = vec![stage1, stage2, stage3.clone()];

        let result = transition_to_next_stage(WorkflowState::AwaitingApproval, &stage3, &all_stages);

        assert!(result.is_final);
        assert!(result.next_stage.is_none());
        assert_eq!(result.current_state, WorkflowState::Completed);
    }

    #[test]
    fn test_is_valid_transition_accepts_valid_transitions() {
        assert!(is_valid_transition(StageStatus::Pending, StageStatus::InProgress));
        assert!(is_valid_transition(StageStatus::InProgress, StageStatus::Completed));
        assert!(is_valid_transition(StageStatus::InProgress, StageStatus::Expired));
        assert!(is_valid_transition(StageStatus::Pending, StageStatus::Skipped));
    }

    #[test]
    fn test_is_valid_transition_rejects_invalid_transitions() {
        assert!(!is_valid_transition(StageStatus::Completed, StageStatus::InProgress));
        assert!(!is_valid_transition(StageStatus::Expired, StageStatus::InProgress));
        assert!(!is_valid_transition(StageStatus::Skipped, StageStatus::InProgress));
    }

    #[test]
    fn test_evaluate_stage_completion_parallel_any() {
        let approvers = vec![Uuid::new_v4(), Uuid::new_v4()];
        let mut stage = create_test_stage("review", approvers, StageStatus::InProgress);
        stage.approvals_received = vec![
            ApprovalResponse {
                approver_id: stage.approvers[0],
                delegated_from: None,
                decision: ApprovalDecision::Approved,
                comment: None,
                responded_at: Utc::now(),
            }
        ];

        let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelAny);
        assert_eq!(result, StageCompletionStatus::Complete);
    }

    #[test]
    fn test_evaluate_stage_completion_with_rejection() {
        let approvers = vec![Uuid::new_v4(), Uuid::new_v4()];
        let mut stage = create_test_stage("review", approvers, StageStatus::InProgress);
        stage.approvals_received = vec![
            ApprovalResponse {
                approver_id: stage.approvers[0],
                delegated_from: None,
                decision: ApprovalDecision::Approved,
                comment: None,
                responded_at: Utc::now(),
            },
            ApprovalResponse {
                approver_id: stage.approvers[1],
                delegated_from: None,
                decision: ApprovalDecision::Rejected,
                comment: None,
                responded_at: Utc::now(),
            },
        ];

        let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelAll);
        assert_eq!(result, StageCompletionStatus::Failed);
    }

    #[test]
    fn test_evaluate_stage_completion_expired() {
        let approver = Uuid::new_v4();
        let mut stage = create_test_stage("review", vec![approver], StageStatus::Expired);

        let result = evaluate_stage_completion(&stage, &ApprovalType::ParallelAny);
        assert_eq!(result, StageCompletionStatus::Expired);
    }
}
