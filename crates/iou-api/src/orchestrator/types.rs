//! Type mappings between orchestrator and API types

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use iou_core::workflows::WorkflowStatus as DocumentState;
use iou_orchestrator::{WorkflowState, AgentType};

/// Status message for WebSocket broadcast
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum StatusMessage {
    #[serde(rename = "started")]
    Started { document_id: Uuid, agent: String, timestamp: i64 },

    #[serde(rename = "progress")]
    Progress { document_id: Uuid, agent: String, percent: u8, timestamp: i64 },

    #[serde(rename = "completed")]
    Completed { document_id: Uuid, timestamp: i64 },

    #[serde(rename = "failed")]
    Failed { document_id: Uuid, error: String, timestamp: i64 },
}

impl StatusMessage {
    pub fn document_id(&self) -> Uuid {
        match self {
            StatusMessage::Started { document_id, .. } => *document_id,
            StatusMessage::Progress { document_id, .. } => *document_id,
            StatusMessage::Completed { document_id, .. } => *document_id,
            StatusMessage::Failed { document_id, .. } => *document_id,
        }
    }

    /// Convert to WebSocket DocumentStatus
    pub fn to_document_status(&self) -> crate::websockets::types::DocumentStatus {
        let timestamp = match self {
            StatusMessage::Started { timestamp, .. } => *timestamp,
            StatusMessage::Progress { timestamp, .. } => *timestamp,
            StatusMessage::Completed { timestamp, .. } => *timestamp,
            StatusMessage::Failed { timestamp, .. } => *timestamp,
        };

        match self {
            StatusMessage::Started { document_id, agent, .. } => {
                crate::websockets::types::DocumentStatus::Started {
                    document_id: *document_id,
                    agent: agent.clone(),
                    timestamp,
                }
            }
            StatusMessage::Progress { document_id, agent, percent, .. } => {
                crate::websockets::types::DocumentStatus::Progress {
                    document_id: *document_id,
                    agent: agent.clone(),
                    percent: *percent,
                    message: None,
                    timestamp,
                }
            }
            StatusMessage::Completed { document_id, .. } => {
                crate::websockets::types::DocumentStatus::Completed {
                    document_id: *document_id,
                    timestamp,
                }
            }
            StatusMessage::Failed { document_id, error, .. } => {
                crate::websockets::types::DocumentStatus::Failed {
                    document_id: *document_id,
                    error: error.clone(),
                    timestamp,
                }
            }
        }
    }
}

/// Convert orchestrator WorkflowState to DocumentState
pub fn workflow_state_to_document_state(workflow_state: &WorkflowState) -> DocumentState {
    match workflow_state {
        WorkflowState::Created => DocumentState::Draft,
        WorkflowState::Running => DocumentState::InReview,
        WorkflowState::AwaitingApproval => DocumentState::InReview,
        WorkflowState::AwaitingEscalation => DocumentState::InReview,
        WorkflowState::Completed => DocumentState::Approved,
        WorkflowState::Failed => DocumentState::Rejected,
        WorkflowState::Cancelled => DocumentState::Rejected,
        WorkflowState::Retrying => DocumentState::InReview,
        WorkflowState::Archived => DocumentState::Archived,
    }
}

/// Agent display name for status messages (Dutch)
pub fn agent_display_name(agent: &AgentType) -> &'static str {
    match agent {
        AgentType::Research => "onderzoeksagent",
        AgentType::Content => "contentagent",
        AgentType::Compliance => "complianceagent",
        AgentType::Review => "reviewagent",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state_mapping() {
        assert_eq!(
            workflow_state_to_document_state(&WorkflowState::Created),
            DocumentState::Draft
        );
        assert_eq!(
            workflow_state_to_document_state(&WorkflowState::Running),
            DocumentState::InReview
        );
        assert_eq!(
            workflow_state_to_document_state(&WorkflowState::Completed),
            DocumentState::Approved
        );
        assert_eq!(
            workflow_state_to_document_state(&WorkflowState::Failed),
            DocumentState::Rejected
        );
    }

    #[test]
    fn test_status_message_document_id() {
        let id = Uuid::new_v4();
        let ts = chrono::Utc::now().timestamp();

        let msg = StatusMessage::Started { document_id: id, agent: "test".to_string(), timestamp: ts };
        assert_eq!(msg.document_id(), id);

        let msg = StatusMessage::Progress { document_id: id, agent: "test".to_string(), percent: 50, timestamp: ts };
        assert_eq!(msg.document_id(), id);

        let msg = StatusMessage::Completed { document_id: id, timestamp: ts };
        assert_eq!(msg.document_id(), id);

        let msg = StatusMessage::Failed { document_id: id, error: "test".to_string(), timestamp: ts };
        assert_eq!(msg.document_id(), id);
    }

    #[test]
    fn test_agent_display_names() {
        assert_eq!(agent_display_name(&AgentType::Research), "onderzoeksagent");
        assert_eq!(agent_display_name(&AgentType::Content), "contentagent");
        assert_eq!(agent_display_name(&AgentType::Compliance), "complianceagent");
        assert_eq!(agent_display_name(&AgentType::Review), "reviewagent");
    }
}
