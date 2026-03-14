//! Workflow context and related types
//!
//! Defines the data structures that carry state through the workflow execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// The main context carried through the workflow state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    // Workflow identification
    pub id: Uuid,
    pub document_request: DocumentRequest,
    pub workflow_version: String,

    // Current state tracking
    pub current_agent: Option<AgentType>,
    pub completed_agents: HashSet<AgentType>,
    pub failed_agents: HashMap<AgentType, u32>,

    // Results per agent
    pub agent_results: HashMap<AgentType, AgentResult>,

    // Human interactions
    pub pending_approvals: Vec<ApprovalRequest>,
    pub audit_log: Vec<AuditEntry>,

    // Configuration (can be overridden per workflow)
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub approval_timeout_hours: u32,

    // Timing
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub approval_deadline: Option<DateTime<Utc>>,
}

impl WorkflowContext {
    /// Create a new workflow context
    pub fn new(
        id: Uuid,
        document_request: DocumentRequest,
        workflow_version: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            document_request,
            workflow_version,
            current_agent: None,
            completed_agents: HashSet::new(),
            failed_agents: HashMap::new(),
            agent_results: HashMap::new(),
            pending_approvals: Vec::new(),
            audit_log: Vec::new(),
            timeout_ms: 300_000, // 5 minutes default
            max_retries: 3,
            approval_timeout_hours: 24,
            created_at: now,
            last_activity: now,
            approval_deadline: None,
        }
    }

    /// Check if a specific agent has completed
    pub fn is_agent_complete(&self, agent: AgentType) -> bool {
        self.completed_agents.contains(&agent)
    }

    /// Get the retry count for a failed agent
    pub fn get_retry_count(&self, agent: &AgentType) -> u32 {
        self.failed_agents.get(agent).copied().unwrap_or(0)
    }

    /// Check if workflow can proceed (no pending approvals)
    pub fn can_proceed(&self) -> bool {
        self.pending_approvals.is_empty()
    }

    /// Check if any agent can still be retried
    pub fn can_retry_any(&self) -> bool {
        self.failed_agents.values().all(|&count| count < self.max_retries)
    }

    /// Record an agent result
    pub fn record_agent_result(&mut self, result: AgentResult) {
        self.completed_agents.insert(result.agent);
        self.agent_results.insert(result.agent, result.clone());
        self.last_activity = Utc::now();
    }

    /// Record an agent failure
    pub fn record_agent_failure(&mut self, agent: AgentType) {
        *self.failed_agents.entry(agent).or_insert(0) += 1;
        self.last_activity = Utc::now();
    }

    /// Add an audit log entry
    pub fn add_audit_entry(&mut self, entry: AuditEntry) {
        self.audit_log.push(entry);
        self.last_activity = Utc::now();
    }

    /// Check if approval has timed out
    pub fn is_approval_timeout(&self) -> bool {
        if let Some(deadline) = self.approval_deadline {
            Utc::now() > deadline
        } else {
            false
        }
    }

    /// Set approval deadline
    pub fn set_approval_deadline(&mut self, hours: u32) {
        self.approval_deadline = Some(Utc::now() + chrono::Duration::hours(hours as i64));
    }

    /// Get next agent to execute based on completed agents
    pub fn next_agent(&self) -> Option<AgentType> {
        for agent in AgentType::execution_order() {
            if !self.is_agent_complete(*agent) {
                return Some(*agent);
            }
        }
        None
    }
}

/// Agent types in the workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Research,
    Content,
    Compliance,
    Review,
}

impl AgentType {
    /// Get the execution order of agents
    pub fn execution_order() -> &'static [AgentType] {
        &[AgentType::Research, AgentType::Content, AgentType::Compliance, AgentType::Review]
    }

    /// Get agent display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::Research => "Onderzoeksagent",
            AgentType::Content => "Contentagent",
            AgentType::Compliance => "Complianceagent",
            AgentType::Review => "Reviewagent",
        }
    }

    /// Check if this agent can run in parallel with another
    pub fn can_run_parallel_with(&self, _other: &AgentType) -> bool {
        // Current implementation: only sequential execution
        // Future: allow Research(N) parallel with Content(N-1)
        false
    }
}

/// Result from a single agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub agent: AgentType,
    pub success: bool,
    pub data: serde_json::Value,
    pub execution_time_ms: u64,
    pub human_modified: bool,
    pub modifications: Vec<HumanModification>,
    pub completed_at: DateTime<Utc>,
}

impl AgentResult {
    /// Create a new successful agent result
    pub fn success(agent: AgentType, data: serde_json::Value, execution_time_ms: u64) -> Self {
        Self {
            agent,
            success: true,
            data,
            execution_time_ms,
            human_modified: false,
            modifications: Vec::new(),
            completed_at: Utc::now(),
        }
    }

    /// Create a new failed agent result
    pub fn failure(agent: AgentType, error: String) -> Self {
        Self {
            agent,
            success: false,
            data: serde_json::json!({ "error": error }),
            execution_time_ms: 0,
            human_modified: false,
            modifications: Vec::new(),
            completed_at: Utc::now(),
        }
    }
}

/// Approval request for human intervention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub agent: AgentType,
    pub result: AgentResult,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub escalated: bool,
    pub approver: Option<Uuid>,
}

impl ApprovalRequest {
    /// Create a new approval request
    pub fn new(
        workflow_id: Uuid,
        agent: AgentType,
        result: AgentResult,
        timeout_hours: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            agent,
            result,
            created_at: now,
            expires_at: now + chrono::Duration::hours(timeout_hours as i64),
            escalated: false,
            approver: None,
        }
    }

    /// Check if this approval request has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Human decision on an approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum ApprovalDecision {
    Approve {
        approver: Uuid,
        comment: Option<String>,
    },
    Modify {
        approver: Uuid,
        modifications: Vec<HumanModification>,
        comment: Option<String>,
    },
    Reject {
        approver: Uuid,
        reason: String,
    },
    RequestChanges {
        approver: Uuid,
        feedback: String,
    },
}

/// A modification made by a human to agent output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanModification {
    pub id: Uuid,
    pub agent: AgentType,
    pub path: String,
    pub original_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub modified_by: Uuid,
    pub modified_at: DateTime<Utc>,
    pub reason: Option<String>,
}

/// Audit log entry for tracking workflow actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub actor: Option<AuditActor>,
    pub details: serde_json::Value,
}

/// Actions that can be audited
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    WorkflowCreated,
    AgentStarted,
    AgentCompleted,
    AgentFailed,
    ApprovalRequested,
    ApprovalGranted,
    ApprovalRejected,
    ApprovalTimeout,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowCancelled,
    ModificationApplied,
    StateTransition,
}

/// Who or what performed an action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActor {
    User { id: Uuid, name: Option<String> },
    Agent { agent_type: AgentType },
    System,
}

/// Document request from a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRequest {
    pub id: Uuid,
    pub domain_id: String,
    pub document_type: DocumentType,
    pub context: HashMap<String, String>,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
    pub priority: RequestPriority,
}

/// Type of document to generate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    WooBesluit,
    WooInformatie,
    WooBesluitBeroep,
    WooInformatieBeroep,
    Beleidsnotitie,
    Raadsvoorstel,
    Ambtsbericht,
    Persbericht,
    InterneMemo,
    Custom(String),
}

impl DocumentType {
    /// Get the base category of this document type
    pub fn category(&self) -> DocumentCategory {
        match self {
            DocumentType::WooBesluit | DocumentType::WooInformatie
            | DocumentType::WooBesluitBeroep | DocumentType::WooInformatieBeroep => {
                DocumentCategory::Woo
            }
            DocumentType::Beleidsnotitie | DocumentType::Raadsvoorstel => {
                DocumentCategory::Beleid
            }
            DocumentType::Persbericht => DocumentCategory::Communicatie,
            DocumentType::Ambtsbericht | DocumentType::InterneMemo => {
                DocumentCategory::Intern
            }
            DocumentType::Custom(_) => DocumentCategory::Overig,
        }
    }

    /// Check if this document type requires mandatory human review
    pub fn requires_mandatory_review(&self) -> bool {
        matches!(self.category(), DocumentCategory::Woo)
    }
}

/// Document categories for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentCategory {
    Woo,
    Beleid,
    Communicatie,
    Intern,
    Overig,
}

/// Request priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Default for RequestPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Final document result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResult {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub document_type: DocumentType,
    pub content: String,
    pub metadata: DocumentMetadata,
    pub created_at: DateTime<Utc>,
}

/// Metadata about the generated document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub word_count: usize,
    pub section_count: usize,
    pub compliance_score: f32,
    pub confidence_score: f32,
    pub has_refusal_grounds: bool,
    pub pii_detected: bool,
    pub human_modifications: usize,
    pub agent_iterations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_context_creation() {
        let request = DocumentRequest {
            id: Uuid::new_v4(),
            domain_id: "test_domain".to_string(),
            document_type: DocumentType::WooBesluit,
            context: HashMap::new(),
            requested_by: Uuid::new_v4(),
            requested_at: Utc::now(),
            priority: RequestPriority::Normal,
        };

        let context = WorkflowContext::new(
            Uuid::new_v4(),
            request,
            "1.0.0".to_string(),
        );

        assert_eq!(context.completed_agents.len(), 0);
        assert!(context.can_proceed());
        assert!(context.next_agent() == Some(AgentType::Research));
    }

    #[test]
    fn test_agent_execution_order() {
        let order = AgentType::execution_order();
        assert_eq!(order[0], AgentType::Research);
        assert_eq!(order[1], AgentType::Content);
        assert_eq!(order[2], AgentType::Compliance);
        assert_eq!(order[3], AgentType::Review);
    }

    #[test]
    fn test_agent_result_success() {
        let result = AgentResult::success(
            AgentType::Research,
            serde_json::json!({"findings": 42}),
            1000,
        );
        assert!(result.success);
        assert_eq!(result.agent, AgentType::Research);
    }

    #[test]
    fn test_approval_request_expiration() {
        let result = AgentResult::success(
            AgentType::Content,
            serde_json::json!({}),
            100,
        );

        // Create request with 1 hour timeout
        let mut request = ApprovalRequest::new(Uuid::new_v4(), AgentType::Content, result, 1);
        // Manually set to past to test expiration (1 hour ago)
        request.expires_at = chrono::Utc::now() - chrono::Duration::hours(1);

        assert!(request.is_expired());
    }

    #[test]
    fn test_document_type_requires_review() {
        assert!(DocumentType::WooBesluit.requires_mandatory_review());
        assert!(!DocumentType::InterneMemo.requires_mandatory_review());
    }

    #[test]
    fn test_workflow_context_agent_completion() {
        let request = DocumentRequest {
            id: Uuid::new_v4(),
            domain_id: "test".to_string(),
            document_type: DocumentType::InterneMemo,
            context: HashMap::new(),
            requested_by: Uuid::new_v4(),
            requested_at: Utc::now(),
            priority: RequestPriority::Normal,
        };

        let mut context = WorkflowContext::new(Uuid::new_v4(), request, "1.0.0".to_string());

        // Initially, Research is next
        assert_eq!(context.next_agent(), Some(AgentType::Research));

        // Complete Research
        let result = AgentResult::success(AgentType::Research, serde_json::json!({}), 100);
        context.record_agent_result(result);

        // Now Content is next
        assert_eq!(context.next_agent(), Some(AgentType::Content));
        assert!(context.is_agent_complete(AgentType::Research));
    }
}
