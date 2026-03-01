//! Document domain types for the multi-agent document creation system.
//!
//! This module defines the core data structures used throughout the document
//! creation pipeline, including states, requests, and metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Reuse existing WorkflowStatus as DocumentState
pub use crate::workflows::WorkflowStatus as DocumentState;

/// Unique identifier for a document generation request
pub type DocumentId = Uuid;

/// Trust level determines auto-approval behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustLevel {
    /// Always requires human approval, regardless of compliance score
    Low,
    /// Requires approval if compliance_score < required_approval_threshold
    Medium,
    /// Auto-approval ONLY for non-Woo documents with high confidence.
    /// ALL Woo-relevant documents require human approval.
    High,
}

impl TrustLevel {
    /// Check if this trust level requires human approval for the given context
    pub fn requires_approval(
        self,
        is_woo_document: bool,
        compliance_score: f32,
        threshold: f32,
    ) -> bool {
        match self {
            TrustLevel::Low => true,
            TrustLevel::Medium => compliance_score < threshold,
            TrustLevel::High => {
                // Woo documents ALWAYS require approval regardless of confidence
                if is_woo_document {
                    true
                } else {
                    compliance_score < threshold
                }
            }
        }
    }

    pub fn requires_approval_for_all(self) -> bool {
        matches!(self, TrustLevel::Low)
    }

    pub fn requires_approval_if_compliance_below(self, _threshold: f32) -> bool {
        matches!(self, TrustLevel::Medium)
    }

    pub fn requires_approval_for_woo(self) -> bool {
        matches!(self, TrustLevel::High)
    }
}

/// IMPORTANT SECURITY NOTE:
/// - ALL Woo-relevant documents require human approval regardless of confidence score
/// - Auto-approval only applies to internal, non-sensitive documents where legal compliance is not a concern
/// - A "dry run" mode should be available for testing auto-approval before enabling it in production

/// Configuration per information domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub domain_id: String,
    pub trust_level: TrustLevel,
    pub required_approval_threshold: f32,  // For Medium trust
    pub auto_approval_threshold: f32,      // For High trust
}

impl DomainConfig {
    /// Validate that threshold values are within valid range (0.0 - 1.0)
    pub fn validate_thresholds(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.required_approval_threshold) {
            return Err(format!(
                "required_approval_threshold must be between 0.0 and 1.0, got {}",
                self.required_approval_threshold
            ));
        }
        if !(0.0..=1.0).contains(&self.auto_approval_threshold) {
            return Err(format!(
                "auto_approval_threshold must be between 0.0 and 1.0, got {}",
                self.auto_approval_threshold
            ));
        }
        Ok(())
    }

    /// Check if a document in this domain requires human approval
    pub fn requires_approval(
        &self,
        is_woo_document: bool,
        compliance_score: f32,
    ) -> bool {
        self.trust_level.requires_approval(
            is_woo_document,
            compliance_score,
            self.auto_approval_threshold,
        )
    }
}

/// Document generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRequest {
    pub id: DocumentId,
    pub domain_id: String,
    pub document_type: String,
    pub context: HashMap<String, String>,
    pub requested_at: DateTime<Utc>,
}

impl DocumentRequest {
    pub fn new(domain_id: String, document_type: String, context: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            domain_id,
            document_type,
            context,
            requested_at: Utc::now(),
        }
    }
}

/// Document metadata stored in DuckDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: DocumentId,
    pub domain_id: String,
    pub document_type: String,
    pub state: DocumentState,
    pub current_version_key: String,    // S3 object key
    pub previous_version_key: Option<String>,
    pub compliance_score: f32,
    pub confidence_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub agent_name: String,
    pub success: bool,
    pub data: serde_json::Value,
    pub errors: Vec<String>,
    pub execution_time_ms: u64,
}

impl AgentResult {
    pub fn success(agent_name: String, data: serde_json::Value, execution_time_ms: u64) -> Self {
        Self {
            agent_name,
            success: true,
            data,
            errors: Vec::new(),
            execution_time_ms,
        }
    }

    pub fn failure(agent_name: String, errors: Vec<String>, execution_time_ms: u64) -> Self {
        Self {
            agent_name,
            success: false,
            data: serde_json::Value::Null,
            errors,
            execution_time_ms,
        }
    }
}

/// Audit trail entry for observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub document_id: DocumentId,
    pub agent_name: String,
    pub action: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub execution_time_ms: Option<u64>,
}

impl AuditEntry {
    pub fn new(
        document_id: DocumentId,
        agent_name: String,
        action: String,
        details: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            document_id,
            agent_name,
            action,
            details,
            timestamp: Utc::now(),
            execution_time_ms: None,
        }
    }
}

/// S3 object reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRef {
    pub bucket: String,
    pub key: String,
    pub version_id: Option<String>,
    pub content_type: String,
    pub size_bytes: u64,
    pub etag: String,
}

/// Document version in S3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub storage_ref: StorageRef,
    pub format: DocumentFormat,
    pub created_at: DateTime<Utc>,
    pub created_by: String,  // Agent or User ID
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentFormat {
    Markdown,
    ODF,   // OpenDocument Format
    PDF,
}

impl DocumentFormat {
    pub fn extension(&self) -> &str {
        match self {
            DocumentFormat::Markdown => "md",
            DocumentFormat::ODF => "odt",
            DocumentFormat::PDF => "pdf",
        }
    }

    pub fn content_type(&self) -> &str {
        match self {
            DocumentFormat::Markdown => "text/markdown",
            DocumentFormat::ODF => "application/vnd.oasis.opendocument.text",
            DocumentFormat::PDF => "application/pdf",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflows::WorkflowStatus;

    #[test]
    fn test_document_id_generates_valid_uuid() {
        let id = Uuid::new_v4();
        assert_ne!(id, Uuid::nil());
    }

    #[test]
    fn test_document_state_maps_to_workflow_status() {
        let state: DocumentState = WorkflowStatus::Draft;
        assert_eq!(state, WorkflowStatus::Draft);
    }

    #[test]
    fn test_trust_level_determines_approval_requirements() {
        let low = TrustLevel::Low;
        let medium = TrustLevel::Medium;
        let high = TrustLevel::High;

        assert!(low.requires_approval_for_all());
        assert!(medium.requires_approval_if_compliance_below(0.8));
        assert!(high.requires_approval_for_woo());
    }

    #[test]
    fn test_domain_config_validates_threshold_ranges() {
        let config = DomainConfig {
            domain_id: "test".to_string(),
            trust_level: TrustLevel::Medium,
            required_approval_threshold: 0.8,
            auto_approval_threshold: 0.95,
        };
        assert!(config.validate_thresholds().is_ok());

        let invalid_config = DomainConfig {
            domain_id: "test".to_string(),
            trust_level: TrustLevel::Medium,
            required_approval_threshold: 1.5,  // Invalid: > 1.0
            auto_approval_threshold: 0.95,
        };
        assert!(invalid_config.validate_thresholds().is_err());
    }

    #[test]
    fn test_document_request_serialization() {
        let request = DocumentRequest {
            id: Uuid::new_v4(),
            domain_id: "test-domain".to_string(),
            document_type: "woo_besluit".to_string(),
            context: HashMap::from([
                ("reference".to_string(), "REF-001".to_string())
            ]),
            requested_at: Utc::now(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: DocumentRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.domain_id, request.domain_id);
    }
}
