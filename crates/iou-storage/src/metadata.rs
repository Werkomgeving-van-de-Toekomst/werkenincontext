//! Metadata storage operations using DuckDB

use chrono::Utc;
use std::collections::HashMap;

use iou_core::document::{DocumentMetadata, AuditEntry, DocumentId, DocumentState};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}

pub type Result<T> = std::result::Result<T, MetadataError>;

/// Metadata store for document-related database operations
///
/// This is a stub implementation for now. In production, this would use
/// DuckDB for persistent storage.
pub struct MetadataStore {
    // In-memory storage for development
    documents: std::sync::Arc<std::sync::Mutex<HashMap<DocumentId, DocumentMetadata>>>,
    audit_trail: std::sync::Arc<std::sync::Mutex<Vec<AuditEntry>>>,
}

impl MetadataStore {
    /// Create a new metadata store with in-memory storage
    pub fn new() -> Result<Self> {
        Ok(Self {
            documents: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            audit_trail: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        })
    }

    /// Create a new document metadata record
    pub fn create_document(&self, metadata: &DocumentMetadata) -> Result<()> {
        let mut docs = self.documents.lock().unwrap();
        if docs.contains_key(&metadata.id) {
            return Err(MetadataError::DatabaseError(format!(
                "Document with id {} already exists",
                metadata.id
            )));
        }
        docs.insert(metadata.id, metadata.clone());
        Ok(())
    }

    /// Get document metadata by ID
    pub fn get_document(&self, id: DocumentId) -> Result<DocumentMetadata> {
        let docs = self.documents.lock().unwrap();
        docs.get(&id)
            .cloned()
            .ok_or_else(|| MetadataError::NotFound(id.to_string()))
    }

    /// Update document state
    pub fn update_state(&self, id: DocumentId, new_state: DocumentState) -> Result<()> {
        let mut docs = self.documents.lock().unwrap();
        let metadata = docs
            .get_mut(&id)
            .ok_or_else(|| MetadataError::NotFound(id.to_string()))?;

        // Validate state transition
        if !metadata.state.can_transition_to(&new_state) {
            return Err(MetadataError::InvalidState(format!(
                "Cannot transition from {:?} to {:?}",
                metadata.state, new_state
            )));
        }

        metadata.state = new_state;
        metadata.updated_at = Utc::now();
        Ok(())
    }

    /// Add audit trail entry
    pub fn add_audit_entry(&self, entry: &AuditEntry) -> Result<()> {
        let mut audit = self.audit_trail.lock().unwrap();
        audit.push(entry.clone());
        Ok(())
    }

    /// Get audit trail for document
    pub fn get_audit_trail(&self, document_id: DocumentId) -> Result<Vec<AuditEntry>> {
        let audit = self.audit_trail.lock().unwrap();
        Ok(audit
            .iter()
            .filter(|e| e.document_id == document_id)
            .cloned()
            .collect())
    }

    /// List documents by state
    pub fn list_by_state(&self, state: DocumentState) -> Result<Vec<DocumentMetadata>> {
        let docs = self.documents.lock().unwrap();
        Ok(docs
            .values()
            .filter(|d| d.state == state)
            .cloned()
            .collect())
    }
}

impl Default for MetadataStore {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iou_core::workflows::WorkflowStatus;
    use uuid::Uuid;

    #[test]
    fn test_metadata_store_creation() {
        let store = MetadataStore::new().unwrap();
        assert_eq!(store.documents.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_create_and_get_document() {
        let store = MetadataStore::new().unwrap();
        let metadata = DocumentMetadata {
            id: Uuid::new_v4(),
            domain_id: "test".to_string(),
            document_type: "woo_besluit".to_string(),
            state: WorkflowStatus::Draft,
            current_version_key: "key1".to_string(),
            previous_version_key: None,
            compliance_score: 0.0,
            confidence_score: 0.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        store.create_document(&metadata).unwrap();
        let retrieved = store.get_document(metadata.id).unwrap();
        assert_eq!(retrieved.domain_id, "test");
    }

    #[test]
    fn test_update_state_valid_transition() {
        let store = MetadataStore::new().unwrap();
        let id = Uuid::new_v4();

        let metadata = DocumentMetadata {
            id,
            domain_id: "test".to_string(),
            document_type: "woo_besluit".to_string(),
            state: WorkflowStatus::Draft,
            current_version_key: "key1".to_string(),
            previous_version_key: None,
            compliance_score: 0.0,
            confidence_score: 0.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        store.create_document(&metadata).unwrap();
        assert!(store.update_state(id, WorkflowStatus::Submitted).is_ok());
        assert!(store.update_state(id, WorkflowStatus::Approved).is_err()); // Can't skip Submitted
    }

    #[test]
    fn test_audit_trail() {
        let store = MetadataStore::new().unwrap();
        let doc_id = Uuid::new_v4();

        let entry = AuditEntry::new(
            doc_id,
            "TestAgent".to_string(),
            "TestAction".to_string(),
            serde_json::json!({"test": "data"}),
        );

        store.add_audit_entry(&entry).unwrap();
        let trail = store.get_audit_trail(doc_id).unwrap();
        assert_eq!(trail.len(), 1);
        assert_eq!(trail[0].agent_name, "TestAgent");
    }
}
