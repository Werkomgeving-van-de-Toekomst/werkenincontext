//! Audit log models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit log entry
///
/// Records every action performed in the system for compliance purposes.
/// Entries are immutable and retained for 7 years as per BIO requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier for this audit entry
    pub id: Uuid,

    /// Timestamp when the action occurred (UTC)
    pub timestamp: DateTime<Utc>,

    /// Tenant (municipality) context
    pub tenant_id: String,

    /// User DID who performed the action
    pub user_did: String,

    /// Type of action performed
    pub action: AuditAction,

    /// Resource that was acted upon
    pub resource_type: String,

    /// Specific resource identifier
    pub resource_id: String,

    /// Outcome of the action
    pub outcome: AuditOutcome,

    /// IP address of the client
    pub ip_address: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Additional context (JSON)
    pub context: Option<serde_json::Value>,

    /// Session ID for correlation
    pub session_id: Option<String>,

    /// Related audit entries (for chained operations)
    pub parent_id: Option<Uuid>,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new<S: Into<String>>(
        tenant_id: S,
        user_did: S,
        action: AuditAction,
        resource_type: S,
        resource_id: S,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            tenant_id: tenant_id.into(),
            user_did: user_did.into(),
            action,
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            outcome: AuditOutcome::Success,
            ip_address: None,
            user_agent: None,
            context: None,
            session_id: None,
            parent_id: None,
        }
    }

    /// Add outcome
    pub fn with_outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Add request metadata
    pub fn with_metadata(
        mut self,
        ip: Option<String>,
        ua: Option<String>,
    ) -> Self {
        self.ip_address = ip;
        self.user_agent = ua;
        self
    }

    /// Add context
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Set parent audit entry
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

/// Type of action performed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Document actions
    DocumentCreated,
    DocumentViewed,
    DocumentUpdated,
    DocumentDeleted,
    DocumentApproved,
    DocumentRejected,

    // Process/Rule actions
    ProcessStarted,
    ProcessCompleted,
    ProcessFailed,
    ProcessCancelled,
    RuleEvaluated,

    // Authentication actions
    UserLogin,
    UserLogout,
    VCPresented,

    // Admin actions
    TenantCreated,
    TenantUpdated,
    UserInvited,
    UserRemoved,

    // Calculation actions
    CalculationStarted,
    CalculationCompleted,

    // Generic action
    Custom(String),
}

impl From<String> for AuditAction {
    fn from(s: String) -> Self {
        match s.as_str() {
            "document_created" => AuditAction::DocumentCreated,
            "document_viewed" => AuditAction::DocumentViewed,
            "document_updated" => AuditAction::DocumentUpdated,
            "document_deleted" => AuditAction::DocumentDeleted,
            "document_approved" => AuditAction::DocumentApproved,
            "document_rejected" => AuditAction::DocumentRejected,
            "process_started" => AuditAction::ProcessStarted,
            "process_completed" => AuditAction::ProcessCompleted,
            "process_failed" => AuditAction::ProcessFailed,
            "process_cancelled" => AuditAction::ProcessCancelled,
            "rule_evaluated" => AuditAction::RuleEvaluated,
            "user_login" => AuditAction::UserLogin,
            "user_logout" => AuditAction::UserLogout,
            "vc_presented" => AuditAction::VCPresented,
            "calculation_started" => AuditAction::CalculationStarted,
            "calculation_completed" => AuditAction::CalculationCompleted,
            _ => AuditAction::Custom(s),
        }
    }
}

/// Outcome of the action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Action succeeded
    Success,

    /// Action failed (business logic)
    Failed,

    /// Action denied (authorization)
    Denied,

    /// Action errored (technical)
    Error,
}

/// Filter for querying audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub start_timestamp: Option<DateTime<Utc>>,
    pub end_timestamp: Option<DateTime<Utc>>,
    pub user_did: Option<String>,
    pub action: Option<AuditAction>,
    pub resource_type: Option<String>,
    pub limit: u32,
}

impl Default for AuditQuery {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            start_timestamp: Some(now - chrono::Duration::days(30)),
            end_timestamp: Some(now),
            user_did: None,
            action: None,
            resource_type: None,
            limit: 100,
        }
    }
}

/// Result type for audit queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQueryResult {
    pub entries: Vec<AuditEntry>,
    pub total_count: u64,
    pub has_more: bool,
}

/// Type alias for AuditFilter (for backward compatibility)
pub type AuditFilter = AuditQuery;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            "utrecht",
            "did:example:user123",
            AuditAction::DocumentCreated,
            "document",
            "doc-123",
        );

        assert_eq!(entry.resource_type, "document");
        assert_eq!(entry.resource_id, "doc-123");
        assert_eq!(entry.outcome, AuditOutcome::Success);
    }

    #[test]
    fn test_audit_entry_builder() {
        let entry = AuditEntry::new(
            "amsterdam",
            "did:example:user456",
            AuditAction::ProcessStarted,
            "process",
            "proc-789",
        )
        .with_outcome(AuditOutcome::Failed)
        .with_metadata(Some("127.0.0.1".to_string()), Some("test-agent".to_string()))
        .with_context(serde_json::json!({"test": "data"}));

        assert_eq!(entry.outcome, AuditOutcome::Failed);
        assert_eq!(entry.ip_address, Some("127.0.0.1".to_string()));
        assert!(entry.context.is_some());
    }

    #[test]
    fn test_audit_query_default() {
        let query = AuditQuery::default();
        assert_eq!(query.limit, 100);
        assert!(query.start_timestamp.is_some());
        assert!(query.end_timestamp.is_some());
    }
}
