//! WebSocket message types for document status updates

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Document status update message
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DocumentStatus {
    /// Workflow has started
    #[serde(rename = "started")]
    Started {
        document_id: Uuid,
        agent: String,
        timestamp: i64,
    },

    /// Progress update during workflow execution
    #[serde(rename = "progress")]
    Progress {
        document_id: Uuid,
        agent: String,
        percent: u8,
        message: Option<String>,
        timestamp: i64,
    },

    /// Workflow completed successfully
    #[serde(rename = "completed")]
    Completed {
        document_id: Uuid,
        timestamp: i64,
    },

    /// Workflow failed
    #[serde(rename = "failed")]
    Failed {
        document_id: Uuid,
        error: String,
        timestamp: i64,
    },

    /// Stage status changed (workflow-specific)
    #[serde(rename = "stage_status_changed")]
    StageStatusChanged {
        document_id: Uuid,
        stage_id: String,
        stage_name: String,
        old_status: String,
        new_status: String,
        timestamp: i64,
    },

    /// Approval received (workflow-specific)
    #[serde(rename = "approval_received")]
    ApprovalReceived {
        document_id: Uuid,
        stage_id: String,
        approver_id: Uuid,
        decision: String,
        timestamp: i64,
    },

    /// Delegation added (workflow-specific)
    #[serde(rename = "delegation_added")]
    DelegationAdded {
        document_id: Uuid,
        stage_id: String,
        from_user_id: Uuid,
        to_user_id: Uuid,
        timestamp: i64,
    },
}

impl DocumentStatus {
    /// Get the document ID for this status update
    pub fn document_id(&self) -> Uuid {
        match self {
            DocumentStatus::Started { document_id, .. } => *document_id,
            DocumentStatus::Progress { document_id, .. } => *document_id,
            DocumentStatus::Completed { document_id, .. } => *document_id,
            DocumentStatus::Failed { document_id, .. } => *document_id,
            DocumentStatus::StageStatusChanged { document_id, .. } => *document_id,
            DocumentStatus::ApprovalReceived { document_id, .. } => *document_id,
            DocumentStatus::DelegationAdded { document_id, .. } => *document_id,
        }
    }

    /// Check if this status indicates workflow completion (success or failure)
    pub fn is_terminal(&self) -> bool {
        matches!(self, DocumentStatus::Completed { .. } | DocumentStatus::Failed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_status_serialization() {
        let status = DocumentStatus::Started {
            document_id: Uuid::new_v4(),
            agent: "research".to_string(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"type\":\"started\""));
        assert!(json.contains("research"));
    }

    #[test]
    fn test_document_status_extracts_id() {
        let id = Uuid::new_v4();
        let status = DocumentStatus::Progress {
            document_id: id,
            agent: "content".to_string(),
            percent: 50,
            message: None,
            timestamp: 0,
        };

        assert_eq!(status.document_id(), id);
    }

    #[test]
    fn test_terminal_status_detection() {
        let id = Uuid::new_v4();

        assert!(DocumentStatus::Completed { document_id: id, timestamp: 0 }.is_terminal());
        assert!(DocumentStatus::Failed {
            document_id: id,
            error: "err".to_string(),
            timestamp: 0
        }
        .is_terminal());
        assert!(!DocumentStatus::Started {
            document_id: id,
            agent: "test".to_string(),
            timestamp: 0
        }
        .is_terminal());
    }
}
