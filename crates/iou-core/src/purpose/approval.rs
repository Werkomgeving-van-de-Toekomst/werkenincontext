//! Purpose Approval Workflow
//!
//! Workflow for requesting and approving new data processing purposes.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::workflows::{ApprovalStage, ApprovalType, WorkflowDefinition};

/// Purpose approval status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PurposeApprovalStatus {
    Draft,
    PendingDpoReview,
    PendingGovernanceReview,
    Approved,
    Rejected,
    Expired,
}

/// Purpose request for a new processing purpose
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurposeRequest {
    pub id: String,
    pub purpose_id: Option<String>,
    pub name: String,
    pub description: String,
    pub lawful_basis: String,
    pub data_categories: Vec<String>,
    pub requested_by: String,
    pub organization_id: String,
    pub justification: String,
    pub expected_duration_days: Option<u32>,
    pub status: PurposeApprovalStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub dpo_notes: Option<String>,
    pub governance_notes: Option<String>,
    pub rejection_reason: Option<String>,
    pub approved_by: Vec<String>,
    pub final_purpose_id: Option<String>,
}

impl PurposeRequest {
    pub fn new(
        id: String,
        name: String,
        description: String,
        lawful_basis: String,
        data_categories: Vec<String>,
        requested_by: String,
        organization_id: String,
        justification: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            purpose_id: None,
            name,
            description,
            lawful_basis,
            data_categories,
            requested_by,
            organization_id,
            justification,
            expected_duration_days: None,
            status: PurposeApprovalStatus::Draft,
            created_at: now,
            updated_at: now,
            submitted_at: None,
            dpo_notes: None,
            governance_notes: None,
            rejection_reason: None,
            approved_by: Vec::new(),
            final_purpose_id: None,
        }
    }

    pub fn submit(&mut self) -> Result<(), PurposeApprovalError> {
        if self.status != PurposeApprovalStatus::Draft {
            return Err(PurposeApprovalError::InvalidStatus);
        }

        self.status = PurposeApprovalStatus::PendingDpoReview;
        self.submitted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn approve_dpo(&mut self, approver: String, notes: Option<String>) -> Result<(), PurposeApprovalError> {
        if self.status != PurposeApprovalStatus::PendingDpoReview {
            return Err(PurposeApprovalError::InvalidStatus);
        }

        self.status = PurposeApprovalStatus::PendingGovernanceReview;
        self.dpo_notes = notes;
        self.approved_by.push(approver);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn approve_governance(
        &mut self,
        approver: String,
        purpose_id: String,
        notes: Option<String>,
    ) -> Result<(), PurposeApprovalError> {
        if self.status != PurposeApprovalStatus::PendingGovernanceReview {
            return Err(PurposeApprovalError::InvalidStatus);
        }

        self.status = PurposeApprovalStatus::Approved;
        self.governance_notes = notes;
        self.approved_by.push(approver);
        self.final_purpose_id = Some(purpose_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn reject(&mut self, reason: String) -> Result<(), PurposeApprovalError> {
        if !matches!(
            self.status,
            PurposeApprovalStatus::PendingDpoReview | PurposeApprovalStatus::PendingGovernanceReview
        ) {
            return Err(PurposeApprovalError::InvalidStatus);
        }

        self.status = PurposeApprovalStatus::Rejected;
        self.rejection_reason = Some(reason);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        if let Some(submitted) = self.submitted_at {
            let elapsed = Utc::now().signed_duration_since(submitted).num_days();
            elapsed > 14
        } else {
            false
        }
    }

    pub fn required_approvers(&self) -> Vec<String> {
        match self.status {
            PurposeApprovalStatus::PendingDpoReview => vec!["dpo@organization".to_string()],
            PurposeApprovalStatus::PendingGovernanceReview => {
                vec!["data-governance@organization".to_string()]
            }
            _ => vec![],
        }
    }
}

/// Purpose approval errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum PurposeApprovalError {
    #[error("Invalid status for this operation")]
    InvalidStatus,

    #[error("Request not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: user {0} cannot approve this request")]
    Unauthorized(String),
}

/// Purpose approval summary for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurposeApprovalSummary {
    pub request_id: String,
    pub purpose_name: String,
    pub status: PurposeApprovalStatus,
    pub requested_by: String,
    pub created_at: DateTime<Utc>,
    pub is_expired: bool,
    pub next_approvers: Vec<String>,
}

impl From<PurposeRequest> for PurposeApprovalSummary {
    fn from(req: PurposeRequest) -> Self {
        let is_expired = req.is_expired();
        let next_approvers = req.required_approvers();
        Self {
            request_id: req.id,
            purpose_name: req.name,
            status: req.status,
            requested_by: req.requested_by,
            created_at: req.created_at,
            is_expired,
            next_approvers,
        }
    }
}

/// Purpose approval workflow definition
/// TODO: Update to match actual WorkflowDefinition structure
pub fn purpose_approval_workflow() -> String {
    "purpose_approval".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purpose_request_creation() {
        let request = PurposeRequest::new(
            "REQ-001".to_string(),
            "TEST_PURPOSE".to_string(),
            "Test purpose".to_string(),
            "wettelijke_verplichting".to_string(),
            vec!["persoonsgegevens".to_string()],
            "user-123".to_string(),
            "org-456".to_string(),
            "Test justification".to_string(),
        );

        assert_eq!(request.status, PurposeApprovalStatus::Draft);
        assert!(request.submitted_at.is_none());
    }

    #[test]
    fn test_submit_request() {
        let mut request = PurposeRequest::new(
            "REQ-001".to_string(),
            "TEST".to_string(),
            "Test".to_string(),
            "wettelijke_verplichting".to_string(),
            vec![],
            "user".to_string(),
            "org".to_string(),
            "Test".to_string(),
        );

        request.submit().unwrap();
        assert_eq!(request.status, PurposeApprovalStatus::PendingDpoReview);
        assert!(request.submitted_at.is_some());
    }

    #[test]
    fn test_approval_workflow() {
        let workflow_name = purpose_approval_workflow();
        assert_eq!(workflow_name, "purpose_approval");
    }
}
