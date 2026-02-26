//! Document workflow types
//!
//! Defines workflow states, transitions, and related types for document
//! processing within IOU-Modern.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Workflow status for documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    /// Document is being drafted
    Draft,

    /// Submitted for review
    Submitted,

    /// Under review
    InReview,

    /// Changes requested
    ChangesRequested,

    /// Approved for publication
    Approved,

    /// Published
    Published,

    /// Rejected
    Rejected,

    /// Archived
    Archived,
}

impl WorkflowStatus {
    /// Get all possible next statuses
    pub fn next_statuses(&self) -> Vec<WorkflowStatus> {
        match self {
            WorkflowStatus::Draft => vec![
                WorkflowStatus::Submitted,
                WorkflowStatus::Archived,
            ],
            WorkflowStatus::Submitted => vec![
                WorkflowStatus::InReview,
                WorkflowStatus::Draft,
            ],
            WorkflowStatus::InReview => vec![
                WorkflowStatus::Approved,
                WorkflowStatus::ChangesRequested,
                WorkflowStatus::Rejected,
            ],
            WorkflowStatus::ChangesRequested => vec![
                WorkflowStatus::Draft,
                WorkflowStatus::Submitted,
            ],
            WorkflowStatus::Approved => vec![
                WorkflowStatus::Published,
            ],
            WorkflowStatus::Published => vec![
                WorkflowStatus::Archived,
            ],
            WorkflowStatus::Rejected => vec![
                WorkflowStatus::Draft,
                WorkflowStatus::Archived,
            ],
            WorkflowStatus::Archived => vec![],
        }
    }

    /// Check if transition to another status is allowed
    pub fn can_transition_to(&self, target: &WorkflowStatus) -> bool {
        self.next_statuses().contains(target)
    }
}

/// Workflow transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    pub id: Uuid,
    pub document_id: Uuid,
    pub from_status: WorkflowStatus,
    pub to_status: WorkflowStatus,
    pub triggered_by: Uuid,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Uuid,
    pub domain_type: Option<String>,
    pub steps: Vec<WorkflowStep>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: WorkflowStatus,
    pub required_role: Option<String>,
    pub auto_assign_to: Option<Uuid>, // User or role
    pub timeout_hours: Option<i32>,
    pub order_index: i32,
}

/// Workflow execution instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub document_id: Uuid,
    pub current_status: WorkflowStatus,
    pub current_step_id: Option<Uuid>,
    pub started_by: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// Workflow action that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAction {
    pub id: Uuid,
    pub name: String,
    pub action_type: WorkflowActionType,
    pub target_status: WorkflowStatus,
    pub require_comment: bool,
    pub require_approval: bool,
    pub allowed_roles: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowActionType {
    Submit,
    Approve,
    Reject,
    RequestChanges,
    Publish,
    Archive,
    Withdraw,
}

/// Workflow notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNotification {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub recipient_id: Uuid,
    pub notification_type: WorkflowNotificationType,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowNotificationType {
    Assignment,
    ApprovalRequired,
    ChangesRequested,
    Approved,
    Rejected,
    Published,
    Overdue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_transitions() {
        assert!(WorkflowStatus::Draft.can_transition_to(&WorkflowStatus::Submitted));
        assert!(!WorkflowStatus::Draft.can_transition_to(&WorkflowStatus::Approved));
    }

    #[test]
    fn test_next_statuses() {
        let next = WorkflowStatus::InReview.next_statuses();
        assert!(next.contains(&WorkflowStatus::Approved));
        assert!(next.contains(&WorkflowStatus::ChangesRequested));
        assert!(next.contains(&WorkflowStatus::Rejected));
    }
}
