//! Multi-stage approval workflow types
//!
//! Defines types for approval stage definitions, per-document stage instances,
//! and approval responses within the enhanced workflow system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
            (None, None) => Err("Approver must have either user_id or role".to_string()),
            (Some(_), Some(_)) => Err("Approver cannot have both user_id and role".to_string()),
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
                Err("Cannot transition from Pending to Completed without InProgress".to_string())
            }
            (StageStatus::Completed, _) | (StageStatus::Expired, _) => {
                Err("Cannot transition from terminal state".to_string())
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
            return Err("Approver has already responded to this stage".to_string());
        }
        self.approvals_received.push(approval);
        Ok(())
    }

    /// Check if all required approvers have responded
    ///
    /// Note: This counts ALL responses (approved, rejected, or delegated).
    /// Use `approved_count()` and `rejected_count()` to determine outcome.
    /// The workflow engine should check both `is_complete()` AND the decision
    /// counts to determine if a stage should transition.
    pub fn is_complete(&self, approval_type: ApprovalType) -> bool {
        let total_approvers = self.approvers.len() as i32;
        let received = self.approvals_received.len() as i32;

        match approval_type {
            ApprovalType::Sequential => {
                // For sequential, we need all approvals in order
                received == total_approvers
            }
            ApprovalType::ParallelAny => {
                // Any single approval is enough
                received >= 1
            }
            ApprovalType::ParallelAll => {
                // All must approve
                received == total_approvers
            }
            ApprovalType::ParallelMajority => {
                // More than half must approve
                let required = (total_approvers / 2) + 1;
                received >= required
            }
        }
    }

    /// Get the count of approved responses
    pub fn approved_count(&self) -> usize {
        self.approvals_received
            .iter()
            .filter(|a| matches!(a.decision, ApprovalDecision::Approved))
            .count()
    }

    /// Get the count of rejected responses
    pub fn rejected_count(&self) -> usize {
        self.approvals_received
            .iter()
            .filter(|a| matches!(a.decision, ApprovalDecision::Rejected))
            .count()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_type_serializes_correctly() {
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

    #[test]
    fn test_expiry_action_escalate_to_includes_target() {
        let action = ExpiryAction::EscalateTo {
            target: "manager".to_string(),
        };
        let serialized = serde_json::to_string(&action).unwrap();
        assert!(serialized.contains("manager"));
        let deserialized: ExpiryAction = serde_json::from_str(&serialized).unwrap();
        matches!(deserialized, ExpiryAction::EscalateTo { .. });
    }

    #[test]
    fn test_stage_instance_status_transitions() {
        let mut instance = StageInstance::new(Uuid::new_v4(), "review".to_string(), vec![]);
        assert_eq!(instance.status, StageStatus::Pending);

        instance.transition_to(StageStatus::InProgress).unwrap();
        assert_eq!(instance.status, StageStatus::InProgress);
        assert!(instance.started_at.is_some());

        instance.transition_to(StageStatus::Completed).unwrap();
        assert_eq!(instance.status, StageStatus::Completed);
        assert!(instance.completed_at.is_some());
    }

    #[test]
    fn test_stage_instance_cannot_skip_to_completed() {
        let mut instance = StageInstance::new(Uuid::new_v4(), "review".to_string(), vec![]);
        let result = instance.transition_to(StageStatus::Completed);
        assert!(result.is_err());
    }

    #[test]
    fn test_approval_response_includes_delegated_from() {
        let original = Uuid::new_v4();
        let delegate = Uuid::new_v4();
        let response = ApprovalResponse::new(delegate, ApprovalDecision::Approved, None)
            .with_delegation(original);
        assert_eq!(response.delegated_from, Some(original));
    }

    #[test]
    fn test_approver_requires_either_user_id_or_role() {
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

    #[test]
    fn test_stage_instance_tracks_approvals_in_order() {
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

    #[test]
    fn test_stage_instance_rejects_duplicate_approval() {
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

    #[test]
    fn test_is_complete_parallel_all() {
        let mut instance = StageInstance::new(
            Uuid::new_v4(),
            "review".to_string(),
            vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
        );
        assert!(!instance.is_complete(ApprovalType::ParallelAll));

        instance.add_approval(ApprovalResponse::new(
            instance.approvers[0],
            ApprovalDecision::Approved,
            None,
        )).unwrap();
        assert!(!instance.is_complete(ApprovalType::ParallelAll));

        instance.add_approval(ApprovalResponse::new(
            instance.approvers[1],
            ApprovalDecision::Approved,
            None,
        )).unwrap();
        assert!(!instance.is_complete(ApprovalType::ParallelAll));

        instance.add_approval(ApprovalResponse::new(
            instance.approvers[2],
            ApprovalDecision::Approved,
            None,
        )).unwrap();
        assert!(instance.is_complete(ApprovalType::ParallelAll));
    }

    #[test]
    fn test_is_complete_parallel_any() {
        let mut instance = StageInstance::new(
            Uuid::new_v4(),
            "review".to_string(),
            vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
        );
        assert!(!instance.is_complete(ApprovalType::ParallelAny));

        instance.add_approval(ApprovalResponse::new(
            instance.approvers[0],
            ApprovalDecision::Approved,
            None,
        )).unwrap();
        assert!(instance.is_complete(ApprovalType::ParallelAny));
    }

    #[test]
    fn test_is_complete_parallel_majority() {
        let approvers = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let mut instance = StageInstance::new(Uuid::new_v4(), "review".to_string(), approvers.clone());
        // Need 3 out of 5 for majority

        instance.add_approval(ApprovalResponse::new(
            approvers[0],
            ApprovalDecision::Approved,
            None,
        )).unwrap();
        instance.add_approval(ApprovalResponse::new(
            approvers[1],
            ApprovalDecision::Approved,
            None,
        )).unwrap();
        assert!(!instance.is_complete(ApprovalType::ParallelMajority));

        instance.add_approval(ApprovalResponse::new(
            approvers[2],
            ApprovalDecision::Approved,
            None,
        )).unwrap();
        assert!(instance.is_complete(ApprovalType::ParallelMajority));
    }

    #[test]
    fn test_approved_rejected_counts() {
        let mut instance = StageInstance::new(
            Uuid::new_v4(),
            "review".to_string(),
            vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
        );

        instance.add_approval(ApprovalResponse::new(
            instance.approvers[0],
            ApprovalDecision::Approved,
            None,
        )).unwrap();
        instance.add_approval(ApprovalResponse::new(
            instance.approvers[1],
            ApprovalDecision::Rejected,
            None,
        )).unwrap();
        instance.add_approval(ApprovalResponse::new(
            instance.approvers[2],
            ApprovalDecision::Approved,
            None,
        )).unwrap();

        assert_eq!(instance.approved_count(), 2);
        assert_eq!(instance.rejected_count(), 1);
    }
}
