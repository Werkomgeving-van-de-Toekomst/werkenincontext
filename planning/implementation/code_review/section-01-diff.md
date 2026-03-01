diff --git a/Cargo.toml b/Cargo.toml
index f1db8e1..19930b1 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -6,6 +6,7 @@ members = [
     "crates/iou-ai",
     "crates/iou-regels",
     "crates/iou-frontend",
+    "crates/iou-storage",
 ]
 
 [workspace.package]
@@ -50,6 +51,7 @@ tracing-subscriber = { version = "0.3", features = ["env-filter"] }
 iou-core = { path = "crates/iou-core" }
 iou-ai = { path = "crates/iou-ai" }
 iou-regels = { path = "crates/iou-regels" }
+iou-storage = { path = "crates/iou-storage" }
 
 [profile.release]
 lto = true
diff --git a/crates/iou-ai/src/document_entity.rs b/crates/iou-ai/src/document_entity.rs
new file mode 100644
index 0000000..915118b
--- /dev/null
+++ b/crates/iou-ai/src/document_entity.rs
@@ -0,0 +1,127 @@
+//! GraphRAG Document entity schema for document-related knowledge graph operations
+
+use serde::{Deserialize, Serialize};
+use uuid::Uuid;
+
+/// Document entity for GraphRAG knowledge graph
+///
+/// This entity represents stored documents in the knowledge graph,
+/// enabling the Research Agent to query similar documents and extract patterns.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DocumentEntity {
+    pub id: Uuid,
+    pub domain_id: String,
+    pub document_type: String,
+    pub title: String,
+    pub content: String,
+    pub sections: Vec<DocumentSection>,
+    pub metadata: DocumentEntityMetadata,
+    pub embeddings: Option<Vec<f32>>,  // For semantic similarity
+    pub created_at: chrono::DateTime<chrono::Utc>,
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DocumentSection {
+    pub name: String,
+    pub content: String,
+    pub is_mandatory: bool,
+    pub order: i32,
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DocumentEntityMetadata {
+    pub author: Option<String>,
+    pub department: Option<String>,
+    pub tags: Vec<String>,
+    pub language: String,  // Default: "nl"
+    pub woo_relevant: bool,
+    pub compliance_score: Option<f32>,
+}
+
+impl DocumentEntity {
+    /// Create a new document entity
+    pub fn new(
+        domain_id: String,
+        document_type: String,
+        title: String,
+        content: String,
+    ) -> Self {
+        Self {
+            id: Uuid::new_v4(),
+            domain_id,
+            document_type,
+            title,
+            content,
+            sections: Vec::new(),
+            metadata: DocumentEntityMetadata {
+                author: None,
+                department: None,
+                tags: Vec::new(),
+                language: "nl".to_string(),
+                woo_relevant: false,
+                compliance_score: None,
+            },
+            embeddings: None,
+            created_at: chrono::Utc::now(),
+        }
+    }
+
+    /// Add a section to the document
+    pub fn with_section(mut self, name: String, content: String, is_mandatory: bool, order: i32) -> Self {
+        self.sections.push(DocumentSection {
+            name,
+            content,
+            is_mandatory,
+            order,
+        });
+        self
+    }
+
+    /// Check if document is Woo-relevant
+    pub fn is_woo_relevant(&self) -> bool {
+        self.metadata.woo_relevant || self.document_type.starts_with("woo_")
+    }
+}
+
+/// Schema definition for Document entity in GraphRAG
+pub struct DocumentSchema;
+
+impl DocumentSchema {
+    /// Entity name in GraphRAG
+    pub const ENTITY_NAME: &'static str = "Document";
+
+    /// Required fields for Document entity
+    pub fn required_fields() -> Vec<&'static str> {
+        vec![
+            "id",
+            "domain_id",
+            "document_type",
+            "content",
+            "created_at",
+        ]
+    }
+
+    /// Optional fields for Document entity
+    pub fn optional_fields() -> Vec<&'static str> {
+        vec![
+            "title",
+            "sections",
+            "embeddings",
+            "woo_relevant",
+            "compliance_score",
+        ]
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_document_entity_schema_is_defined() {
+        assert_eq!(DocumentSchema::ENTITY_NAME, "Document");
+        assert!(DocumentSchema::required_fields().contains(&"id"));
+        assert!(DocumentSchema::required_fields().contains(&"domain_id"));
+        assert!(DocumentSchema::required_fields().contains(&"document_type"));
+    }
+}
diff --git a/crates/iou-ai/src/lib.rs b/crates/iou-ai/src/lib.rs
index 5718de7..19e4ad2 100644
--- a/crates/iou-ai/src/lib.rs
+++ b/crates/iou-ai/src/lib.rs
@@ -20,12 +20,14 @@
 
 pub mod ner;
 pub mod graphrag;
+pub mod document_entity;
 pub mod compliance;
 pub mod suggestions;
 pub mod semantic;
 
 pub use ner::DutchNerExtractor;
 pub use graphrag::KnowledgeGraph;
+pub use document_entity::{DocumentEntity, DocumentSection, DocumentEntityMetadata, DocumentSchema};
 pub use compliance::ComplianceAssessor;
 pub use suggestions::MetadataSuggester;
 pub use semantic::{SemanticSearchService, cosine_similarity};
diff --git a/crates/iou-core/src/document.rs b/crates/iou-core/src/document.rs
new file mode 100644
index 0000000..6c42154
--- /dev/null
+++ b/crates/iou-core/src/document.rs
@@ -0,0 +1,319 @@
+//! Document domain types for the multi-agent document creation system.
+//!
+//! This module defines the core data structures used throughout the document
+//! creation pipeline, including states, requests, and metadata.
+
+use chrono::{DateTime, Utc};
+use serde::{Deserialize, Serialize};
+use std::collections::HashMap;
+use uuid::Uuid;
+
+// Reuse existing WorkflowStatus as DocumentState
+pub use crate::workflows::WorkflowStatus as DocumentState;
+
+/// Unique identifier for a document generation request
+pub type DocumentId = Uuid;
+
+/// Trust level determines auto-approval behavior
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum TrustLevel {
+    /// Always requires human approval, regardless of compliance score
+    Low,
+    /// Requires approval if compliance_score < required_approval_threshold
+    Medium,
+    /// Auto-approval ONLY for non-Woo documents with high confidence.
+    /// ALL Woo-relevant documents require human approval.
+    High,
+}
+
+impl TrustLevel {
+    /// Check if this trust level requires human approval for the given context
+    pub fn requires_approval(
+        self,
+        is_woo_document: bool,
+        compliance_score: f32,
+        threshold: f32,
+    ) -> bool {
+        match self {
+            TrustLevel::Low => true,
+            TrustLevel::Medium => compliance_score < threshold,
+            TrustLevel::High => {
+                // Woo documents ALWAYS require approval regardless of confidence
+                if is_woo_document {
+                    true
+                } else {
+                    compliance_score < threshold
+                }
+            }
+        }
+    }
+
+    pub fn requires_approval_for_all(self) -> bool {
+        matches!(self, TrustLevel::Low)
+    }
+
+    pub fn requires_approval_if_compliance_below(self, _threshold: f32) -> bool {
+        matches!(self, TrustLevel::Medium)
+    }
+
+    pub fn requires_approval_for_woo(self) -> bool {
+        matches!(self, TrustLevel::High)
+    }
+}
+
+/// IMPORTANT SECURITY NOTE:
+/// - ALL Woo-relevant documents require human approval regardless of confidence score
+/// - Auto-approval only applies to internal, non-sensitive documents where legal compliance is not a concern
+/// - A "dry run" mode should be available for testing auto-approval before enabling it in production
+
+/// Configuration per information domain
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DomainConfig {
+    pub domain_id: String,
+    pub trust_level: TrustLevel,
+    pub required_approval_threshold: f32,  // For Medium trust
+    pub auto_approval_threshold: f32,      // For High trust
+}
+
+impl DomainConfig {
+    /// Validate that threshold values are within valid range (0.0 - 1.0)
+    pub fn validate_thresholds(&self) -> Result<(), String> {
+        if !(0.0..=1.0).contains(&self.required_approval_threshold) {
+            return Err(format!(
+                "required_approval_threshold must be between 0.0 and 1.0, got {}",
+                self.required_approval_threshold
+            ));
+        }
+        if !(0.0..=1.0).contains(&self.auto_approval_threshold) {
+            return Err(format!(
+                "auto_approval_threshold must be between 0.0 and 1.0, got {}",
+                self.auto_approval_threshold
+            ));
+        }
+        Ok(())
+    }
+
+    /// Check if a document in this domain requires human approval
+    pub fn requires_approval(
+        &self,
+        is_woo_document: bool,
+        compliance_score: f32,
+    ) -> bool {
+        self.trust_level.requires_approval(
+            is_woo_document,
+            compliance_score,
+            self.auto_approval_threshold,
+        )
+    }
+}
+
+/// Document generation request
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DocumentRequest {
+    pub id: DocumentId,
+    pub domain_id: String,
+    pub document_type: String,
+    pub context: HashMap<String, String>,
+    pub requested_at: DateTime<Utc>,
+}
+
+impl DocumentRequest {
+    pub fn new(domain_id: String, document_type: String, context: HashMap<String, String>) -> Self {
+        Self {
+            id: Uuid::new_v4(),
+            domain_id,
+            document_type,
+            context,
+            requested_at: Utc::now(),
+        }
+    }
+}
+
+/// Document metadata stored in DuckDB
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DocumentMetadata {
+    pub id: DocumentId,
+    pub domain_id: String,
+    pub document_type: String,
+    pub state: DocumentState,
+    pub current_version_key: String,    // S3 object key
+    pub previous_version_key: Option<String>,
+    pub compliance_score: f32,
+    pub confidence_score: f32,
+    pub created_at: DateTime<Utc>,
+    pub updated_at: DateTime<Utc>,
+}
+
+/// Agent execution result
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct AgentResult {
+    pub agent_name: String,
+    pub success: bool,
+    pub data: serde_json::Value,
+    pub errors: Vec<String>,
+    pub execution_time_ms: u64,
+}
+
+impl AgentResult {
+    pub fn success(agent_name: String, data: serde_json::Value, execution_time_ms: u64) -> Self {
+        Self {
+            agent_name,
+            success: true,
+            data,
+            errors: Vec::new(),
+            execution_time_ms,
+        }
+    }
+
+    pub fn failure(agent_name: String, errors: Vec<String>, execution_time_ms: u64) -> Self {
+        Self {
+            agent_name,
+            success: false,
+            data: serde_json::Value::Null,
+            errors,
+            execution_time_ms,
+        }
+    }
+}
+
+/// Audit trail entry for observability
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct AuditEntry {
+    pub id: Uuid,
+    pub document_id: DocumentId,
+    pub agent_name: String,
+    pub action: String,
+    pub details: serde_json::Value,
+    pub timestamp: DateTime<Utc>,
+    pub execution_time_ms: Option<u64>,
+}
+
+impl AuditEntry {
+    pub fn new(
+        document_id: DocumentId,
+        agent_name: String,
+        action: String,
+        details: serde_json::Value,
+    ) -> Self {
+        Self {
+            id: Uuid::new_v4(),
+            document_id,
+            agent_name,
+            action,
+            details,
+            timestamp: Utc::now(),
+            execution_time_ms: None,
+        }
+    }
+}
+
+/// S3 object reference
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct StorageRef {
+    pub bucket: String,
+    pub key: String,
+    pub version_id: Option<String>,
+    pub content_type: String,
+    pub size_bytes: u64,
+    pub etag: String,
+}
+
+/// Document version in S3
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct DocumentVersion {
+    pub storage_ref: StorageRef,
+    pub format: DocumentFormat,
+    pub created_at: DateTime<Utc>,
+    pub created_by: String,  // Agent or User ID
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+pub enum DocumentFormat {
+    Markdown,
+    ODF,   // OpenDocument Format
+    PDF,
+}
+
+impl DocumentFormat {
+    pub fn extension(&self) -> &str {
+        match self {
+            DocumentFormat::Markdown => "md",
+            DocumentFormat::ODF => "odt",
+            DocumentFormat::PDF => "pdf",
+        }
+    }
+
+    pub fn content_type(&self) -> &str {
+        match self {
+            DocumentFormat::Markdown => "text/markdown",
+            DocumentFormat::ODF => "application/vnd.oasis.opendocument.text",
+            DocumentFormat::PDF => "application/pdf",
+        }
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::workflows::WorkflowStatus;
+
+    #[test]
+    fn test_document_id_generates_valid_uuid() {
+        let id = Uuid::new_v4();
+        assert_ne!(id, Uuid::nil());
+    }
+
+    #[test]
+    fn test_document_state_maps_to_workflow_status() {
+        let state: DocumentState = WorkflowStatus::Draft;
+        assert_eq!(state, WorkflowStatus::Draft);
+    }
+
+    #[test]
+    fn test_trust_level_determines_approval_requirements() {
+        let low = TrustLevel::Low;
+        let medium = TrustLevel::Medium;
+        let high = TrustLevel::High;
+
+        assert!(low.requires_approval_for_all());
+        assert!(medium.requires_approval_if_compliance_below(0.8));
+        assert!(high.requires_approval_for_woo());
+    }
+
+    #[test]
+    fn test_domain_config_validates_threshold_ranges() {
+        let config = DomainConfig {
+            domain_id: "test".to_string(),
+            trust_level: TrustLevel::Medium,
+            required_approval_threshold: 0.8,
+            auto_approval_threshold: 0.95,
+        };
+        assert!(config.validate_thresholds().is_ok());
+
+        let invalid_config = DomainConfig {
+            domain_id: "test".to_string(),
+            trust_level: TrustLevel::Medium,
+            required_approval_threshold: 1.5,  // Invalid: > 1.0
+            auto_approval_threshold: 0.95,
+        };
+        assert!(invalid_config.validate_thresholds().is_err());
+    }
+
+    #[test]
+    fn test_document_request_serialization() {
+        let request = DocumentRequest {
+            id: Uuid::new_v4(),
+            domain_id: "test-domain".to_string(),
+            document_type: "woo_besluit".to_string(),
+            context: HashMap::from([
+                ("reference".to_string(), "REF-001".to_string())
+            ]),
+            requested_at: Utc::now(),
+        };
+
+        let json = serde_json::to_string(&request).unwrap();
+        let deserialized: DocumentRequest = serde_json::from_str(&json).unwrap();
+        assert_eq!(deserialized.domain_id, request.domain_id);
+    }
+}
diff --git a/crates/iou-core/src/lib.rs b/crates/iou-core/src/lib.rs
index be11936..9b373b3 100644
--- a/crates/iou-core/src/lib.rs
+++ b/crates/iou-core/src/lib.rs
@@ -19,6 +19,7 @@ pub mod organization;
 pub mod graphrag;
 pub mod api_types;
 pub mod workflows;
+pub mod document;
 
 // Re-exports voor gemakkelijk gebruik
 pub use domain::{DomainType, InformationDomain, Case, Project, PolicyTopic};
@@ -26,3 +27,10 @@ pub use objects::{ObjectType, InformationObject};
 pub use compliance::{Classification, WooMetadata, AvgMetadata, RetentionPolicy};
 pub use organization::{Organization, Department, User, Role};
 pub use workflows::{WorkflowStatus, WorkflowDefinition, WorkflowExecution};
+
+// Document creation types
+pub use document::{
+    DocumentId, DocumentState, TrustLevel, DomainConfig,
+    DocumentRequest, DocumentMetadata, AgentResult, AuditEntry,
+    StorageRef, DocumentVersion, DocumentFormat,
+};
diff --git a/crates/iou-storage/Cargo.toml b/crates/iou-storage/Cargo.toml
new file mode 100644
index 0000000..1515e47
--- /dev/null
+++ b/crates/iou-storage/Cargo.toml
@@ -0,0 +1,23 @@
+[package]
+name = "iou-storage"
+version = "0.1.0"
+edition = "2021"
+
+[dependencies]
+iou-core = { path = "../iou-core" }
+async-trait = "0.1"
+tokio = { version = "1", features = ["full"] }
+serde = { version = "1", features = ["derive"] }
+serde_json = "1"
+thiserror = "2"
+anyhow = "1"
+chrono = { version = "0.4", features = ["serde"] }
+uuid = { version = "1", features = ["serde", "v4"] }
+
+# AWS SDK for S3 (works with MinIO and other S3-compatible storage)
+aws-config = { version = "1.5", features = ["behavior-version-latest"] }
+aws-sdk-s3 = "1.65"
+aws-smithy-types = "1.2"
+
+[dev-dependencies]
+tokio-test = "0.4"
diff --git a/crates/iou-storage/src/config.rs b/crates/iou-storage/src/config.rs
new file mode 100644
index 0000000..6c85596
--- /dev/null
+++ b/crates/iou-storage/src/config.rs
@@ -0,0 +1,76 @@
+//! Storage configuration
+
+use serde::{Deserialize, Serialize};
+
+/// Storage configuration loaded from environment or config file
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct StorageConfig {
+    /// S3-compatible endpoint URL
+    pub endpoint: String,
+    /// Access key ID
+    pub access_key_id: String,
+    /// Secret access key
+    pub secret_access_key: String,
+    /// Bucket name for document storage
+    pub bucket: String,
+    /// Region (optional for MinIO)
+    pub region: Option<String>,
+    /// Whether to use path-style addressing (required for MinIO)
+    pub force_path_style: bool,
+}
+
+impl StorageConfig {
+    /// Load configuration from environment variables
+    ///
+    /// # Security
+    /// This method requires STORAGE_ACCESS_KEY_ID and STORAGE_SECRET_ACCESS_KEY
+    /// to be set. Default credentials are never used in production.
+    ///
+    /// For development/testing, use `minio_local()` instead.
+    pub fn from_env() -> Result<Self, anyhow::Error> {
+        Ok(Self {
+            endpoint: std::env::var("STORAGE_ENDPOINT")
+                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
+            access_key_id: std::env::var("STORAGE_ACCESS_KEY_ID")
+                .expect("STORAGE_ACCESS_KEY_ID must be set"),
+            secret_access_key: std::env::var("STORAGE_SECRET_ACCESS_KEY")
+                .expect("STORAGE_SECRET_ACCESS_KEY must be set"),
+            bucket: std::env::var("STORAGE_BUCKET")
+                .unwrap_or_else(|_| "iou-documents".to_string()),
+            region: std::env::var("STORAGE_REGION").ok(),
+            force_path_style: std::env::var("STORAGE_FORCE_PATH_STYLE")
+                .unwrap_or_else(|_| "true".to_string())
+                .parse()
+                .unwrap_or(true),
+        })
+    }
+
+    /// Create a config for local MinIO development
+    ///
+    /// # Warning
+    /// Only use this for local development. Never deploy with default credentials.
+    pub fn minio_local() -> Self {
+        Self {
+            endpoint: "http://localhost:9000".to_string(),
+            access_key_id: "minioadmin".to_string(),
+            secret_access_key: "minioadmin".to_string(),
+            bucket: "iou-documents".to_string(),
+            region: None,
+            force_path_style: true,
+        }
+    }
+}
+
+#[cfg(test)]
+impl StorageConfig {
+    pub fn test_mock() -> Self {
+        Self {
+            endpoint: "http://localhost:9000".to_string(),
+            access_key_id: "test-key".to_string(),
+            secret_access_key: "test-secret".to_string(),
+            bucket: "test-bucket".to_string(),
+            region: None,
+            force_path_style: true,
+        }
+    }
+}
diff --git a/crates/iou-storage/src/lib.rs b/crates/iou-storage/src/lib.rs
new file mode 100644
index 0000000..3880f45
--- /dev/null
+++ b/crates/iou-storage/src/lib.rs
@@ -0,0 +1,12 @@
+//! Storage abstraction layer for IOU-Modern document system.
+//!
+//! Provides a unified interface for S3/MinIO storage operations and
+//! document metadata persistence.
+
+pub mod config;
+pub mod s3;
+pub mod metadata;
+
+pub use config::StorageConfig;
+pub use s3::{S3Client, S3Error, StorageOperations};
+pub use metadata::{MetadataStore, MetadataError};
diff --git a/crates/iou-storage/src/metadata.rs b/crates/iou-storage/src/metadata.rs
new file mode 100644
index 0000000..5aa2f89
--- /dev/null
+++ b/crates/iou-storage/src/metadata.rs
@@ -0,0 +1,196 @@
+//! Metadata storage operations using DuckDB
+
+use chrono::Utc;
+use std::collections::HashMap;
+
+use iou_core::document::{DocumentMetadata, AuditEntry, DocumentId, DocumentState};
+use thiserror::Error;
+
+#[derive(Error, Debug)]
+pub enum MetadataError {
+    #[error("Database error: {0}")]
+    DatabaseError(String),
+
+    #[error("Not found: {0}")]
+    NotFound(String),
+
+    #[error("Serialization error: {0}")]
+    SerializationError(#[from] serde_json::Error),
+
+    #[error("Invalid state: {0}")]
+    InvalidState(String),
+
+    #[error("Connection error: {0}")]
+    ConnectionError(String),
+}
+
+pub type Result<T> = std::result::Result<T, MetadataError>;
+
+/// Metadata store for document-related database operations
+///
+/// This is a stub implementation for now. In production, this would use
+/// DuckDB for persistent storage.
+pub struct MetadataStore {
+    // In-memory storage for development
+    documents: std::sync::Arc<std::sync::Mutex<HashMap<DocumentId, DocumentMetadata>>>,
+    audit_trail: std::sync::Arc<std::sync::Mutex<Vec<AuditEntry>>>,
+}
+
+impl MetadataStore {
+    /// Create a new metadata store with in-memory storage
+    pub fn new() -> Result<Self> {
+        Ok(Self {
+            documents: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
+            audit_trail: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
+        })
+    }
+
+    /// Create a new document metadata record
+    pub fn create_document(&self, metadata: &DocumentMetadata) -> Result<()> {
+        let mut docs = self.documents.lock().unwrap();
+        if docs.contains_key(&metadata.id) {
+            return Err(MetadataError::DatabaseError(format!(
+                "Document with id {} already exists",
+                metadata.id
+            )));
+        }
+        docs.insert(metadata.id, metadata.clone());
+        Ok(())
+    }
+
+    /// Get document metadata by ID
+    pub fn get_document(&self, id: DocumentId) -> Result<DocumentMetadata> {
+        let docs = self.documents.lock().unwrap();
+        docs.get(&id)
+            .cloned()
+            .ok_or_else(|| MetadataError::NotFound(id.to_string()))
+    }
+
+    /// Update document state
+    pub fn update_state(&self, id: DocumentId, new_state: DocumentState) -> Result<()> {
+        let mut docs = self.documents.lock().unwrap();
+        let metadata = docs
+            .get_mut(&id)
+            .ok_or_else(|| MetadataError::NotFound(id.to_string()))?;
+
+        // Validate state transition
+        if !metadata.state.can_transition_to(&new_state) {
+            return Err(MetadataError::InvalidState(format!(
+                "Cannot transition from {:?} to {:?}",
+                metadata.state, new_state
+            )));
+        }
+
+        metadata.state = new_state;
+        metadata.updated_at = Utc::now();
+        Ok(())
+    }
+
+    /// Add audit trail entry
+    pub fn add_audit_entry(&self, entry: &AuditEntry) -> Result<()> {
+        let mut audit = self.audit_trail.lock().unwrap();
+        audit.push(entry.clone());
+        Ok(())
+    }
+
+    /// Get audit trail for document
+    pub fn get_audit_trail(&self, document_id: DocumentId) -> Result<Vec<AuditEntry>> {
+        let audit = self.audit_trail.lock().unwrap();
+        Ok(audit
+            .iter()
+            .filter(|e| e.document_id == document_id)
+            .cloned()
+            .collect())
+    }
+
+    /// List documents by state
+    pub fn list_by_state(&self, state: DocumentState) -> Result<Vec<DocumentMetadata>> {
+        let docs = self.documents.lock().unwrap();
+        Ok(docs
+            .values()
+            .filter(|d| d.state == state)
+            .cloned()
+            .collect())
+    }
+}
+
+impl Default for MetadataStore {
+    fn default() -> Self {
+        Self::new().unwrap()
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use iou_core::workflows::WorkflowStatus;
+    use uuid::Uuid;
+
+    #[test]
+    fn test_metadata_store_creation() {
+        let store = MetadataStore::new().unwrap();
+        assert_eq!(store.documents.lock().unwrap().len(), 0);
+    }
+
+    #[test]
+    fn test_create_and_get_document() {
+        let store = MetadataStore::new().unwrap();
+        let metadata = DocumentMetadata {
+            id: Uuid::new_v4(),
+            domain_id: "test".to_string(),
+            document_type: "woo_besluit".to_string(),
+            state: WorkflowStatus::Draft,
+            current_version_key: "key1".to_string(),
+            previous_version_key: None,
+            compliance_score: 0.0,
+            confidence_score: 0.0,
+            created_at: Utc::now(),
+            updated_at: Utc::now(),
+        };
+
+        store.create_document(&metadata).unwrap();
+        let retrieved = store.get_document(metadata.id).unwrap();
+        assert_eq!(retrieved.domain_id, "test");
+    }
+
+    #[test]
+    fn test_update_state_valid_transition() {
+        let store = MetadataStore::new().unwrap();
+        let id = Uuid::new_v4();
+
+        let metadata = DocumentMetadata {
+            id,
+            domain_id: "test".to_string(),
+            document_type: "woo_besluit".to_string(),
+            state: WorkflowStatus::Draft,
+            current_version_key: "key1".to_string(),
+            previous_version_key: None,
+            compliance_score: 0.0,
+            confidence_score: 0.0,
+            created_at: Utc::now(),
+            updated_at: Utc::now(),
+        };
+
+        store.create_document(&metadata).unwrap();
+        assert!(store.update_state(id, WorkflowStatus::Submitted).is_ok());
+        assert!(store.update_state(id, WorkflowStatus::Approved).is_err()); // Can't skip Submitted
+    }
+
+    #[test]
+    fn test_audit_trail() {
+        let store = MetadataStore::new().unwrap();
+        let doc_id = Uuid::new_v4();
+
+        let entry = AuditEntry::new(
+            doc_id,
+            "TestAgent".to_string(),
+            "TestAction".to_string(),
+            serde_json::json!({"test": "data"}),
+        );
+
+        store.add_audit_entry(&entry).unwrap();
+        let trail = store.get_audit_trail(doc_id).unwrap();
+        assert_eq!(trail.len(), 1);
+        assert_eq!(trail[0].agent_name, "TestAgent");
+    }
+}
diff --git a/crates/iou-storage/src/s3.rs b/crates/iou-storage/src/s3.rs
new file mode 100644
index 0000000..3eabfe6
--- /dev/null
+++ b/crates/iou-storage/src/s3.rs
@@ -0,0 +1,250 @@
+//! S3 client wrapper for document storage operations
+//!
+//! This module provides an async S3 client using AWS SDK for Rust,
+//! compatible with AWS S3, MinIO, Garage, and other S3-compatible storage.
+
+use async_trait::async_trait;
+use aws_config::BehaviorVersion;
+use aws_config::Region;
+use aws_sdk_s3::{Client, Config};
+use aws_smithy_types::byte_stream::ByteStream;
+use thiserror::Error;
+use std::sync::Arc;
+
+pub use crate::config::StorageConfig as S3Config;
+
+#[derive(Error, Debug)]
+pub enum S3Error {
+    #[error("IO error: {0}")]
+    IoError(#[from] std::io::Error),
+
+    #[error("Serialization error: {0}")]
+    SerializationError(#[from] serde_json::Error),
+
+    #[error("Not found: {bucket}/{key}")]
+    NotFound { bucket: String, key: String },
+
+    #[error("S3 operation failed: {0}")]
+    OperationFailed(String),
+
+    #[error("Configuration error: {0}")]
+    ConfigError(String),
+}
+
+pub type Result<T> = std::result::Result<T, S3Error>;
+
+/// S3 client wrapper with convenient methods for document operations
+pub struct S3Client {
+    config: S3Config,
+    client: Arc<Client>,
+    bucket: String,
+}
+
+impl S3Client {
+    /// Create a new S3 client from configuration
+    pub async fn new(config: S3Config) -> Result<Self> {
+        let region_str = config.region.as_deref().unwrap_or("us-east-1").to_string();
+        let region = Region::new(region_str);
+
+        // Create credentials provider
+        let creds = aws_sdk_s3::config::Credentials::new(
+            &config.access_key_id,
+            &config.secret_access_key,
+            None,
+            None,
+            "iou-storage",
+        );
+
+        // Build S3 config
+        let s3_config = Config::builder()
+            .behavior_version(BehaviorVersion::latest())
+            .region(region)
+            .endpoint_url(config.endpoint.clone())
+            .credentials_provider(creds)
+            .force_path_style(config.force_path_style)
+            .build();
+
+        let client = Arc::new(Client::from_conf(s3_config));
+        let bucket = config.bucket.clone();
+
+        Ok(Self {
+            config,
+            client,
+            bucket,
+        })
+    }
+
+    /// Create a new S3 client synchronously (for testing/development)
+    ///
+    /// NOTE: This creates a client without async config loading.
+    /// Use `new()` for production.
+    pub fn new_sync(config: S3Config) -> Result<Self> {
+        let region_str = config.region.as_deref().unwrap_or("us-east-1").to_string();
+        let region = Region::new(region_str);
+
+        let creds = aws_sdk_s3::config::Credentials::new(
+            &config.access_key_id,
+            &config.secret_access_key,
+            None,
+            None,
+            "iou-storage",
+        );
+
+        let s3_config = Config::builder()
+            .behavior_version(BehaviorVersion::latest())
+            .region(region)
+            .endpoint_url(config.endpoint.clone())
+            .credentials_provider(creds)
+            .force_path_style(config.force_path_style)
+            .build();
+
+        let client = Arc::new(Client::from_conf(s3_config));
+        let bucket = config.bucket.clone();
+
+        Ok(Self {
+            config,
+            client,
+            bucket,
+        })
+    }
+
+    /// Check if client is properly configured
+    pub fn is_ready(&self) -> bool {
+        !self.config.access_key_id.is_empty() && !self.config.secret_access_key.is_empty()
+    }
+
+    /// Generate a storage key for a document
+    pub fn document_key(document_id: &str, version: i32, format: &str) -> String {
+        format!("documents/{}/v{}.{}", document_id, version, format)
+    }
+
+    /// Generate a storage key for a redacted document
+    pub fn redacted_document_key(document_id: &str, version: i32, format: &str) -> String {
+        format!("documents/{}/v{}.redacted.{}", document_id, version, format)
+    }
+
+    /// Get the bucket name
+    pub fn bucket_name(&self) -> &str {
+        &self.bucket
+    }
+}
+
+/// Async storage operations trait
+#[async_trait]
+pub trait StorageOperations: Send + Sync {
+    /// Put data to S3
+    async fn put(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<String>;
+
+    /// Get data from S3
+    async fn get(&self, key: &str) -> Result<Vec<u8>>;
+
+    /// Delete data from S3
+    async fn delete(&self, key: &str) -> Result<()>;
+
+    /// Check if key exists in S3
+    async fn exists(&self, key: &str) -> Result<bool>;
+}
+
+#[async_trait]
+impl StorageOperations for S3Client {
+    async fn put(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<String> {
+        let body = ByteStream::from(data);
+
+        self.client
+            .put_object()
+            .bucket(&self.bucket)
+            .key(key)
+            .content_type(content_type)
+            .body(body)
+            .send()
+            .await
+            .map_err(|e| S3Error::OperationFailed(format!("put failed: {}", e)))?;
+
+        Ok(format!("{}/{}", self.bucket, key))
+    }
+
+    async fn get(&self, key: &str) -> Result<Vec<u8>> {
+        let output = self
+            .client
+            .get_object()
+            .bucket(&self.bucket)
+            .key(key)
+            .send()
+            .await
+            .map_err(|e| {
+                // Check for not found variants
+                if e.to_string().contains("NoSuchKey") || e.to_string().contains("NotFound") {
+                    S3Error::NotFound {
+                        bucket: self.bucket.clone(),
+                        key: key.to_string(),
+                    }
+                } else {
+                    S3Error::OperationFailed(format!("get failed: {}", e))
+                }
+            })?;
+
+        let data = output
+            .body
+            .collect()
+            .await
+            .map_err(|e| S3Error::OperationFailed(format!("read body failed: {}", e)))?
+            .into_bytes();
+
+        Ok(data.to_vec())
+    }
+
+    async fn delete(&self, key: &str) -> Result<()> {
+        self.client
+            .delete_object()
+            .bucket(&self.bucket)
+            .key(key)
+            .send()
+            .await
+            .map_err(|e| S3Error::OperationFailed(format!("delete failed: {}", e)))?;
+
+        Ok(())
+    }
+
+    async fn exists(&self, key: &str) -> Result<bool> {
+        match self
+            .client
+            .head_object()
+            .bucket(&self.bucket)
+            .key(key)
+            .send()
+            .await
+        {
+            Ok(_) => Ok(true),
+            Err(e) => {
+                let err_str = e.to_string();
+                if err_str.contains("NotFound") || err_str.contains("NoSuchKey") {
+                    Ok(false)
+                } else {
+                    Err(S3Error::OperationFailed(format!("exists check failed: {}", e)))
+                }
+            }
+        }
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_document_key_generation() {
+        let key = S3Client::document_key("uuid-here", 1, "md");
+        assert_eq!(key, "documents/uuid-here/v1.md");
+
+        let redacted_key = S3Client::redacted_document_key("uuid-here", 1, "md");
+        assert_eq!(redacted_key, "documents/uuid-here/v1.redacted.md");
+    }
+
+    #[test]
+    fn test_s3_client_sync_creation() {
+        let config = S3Config::test_mock();
+        let client = S3Client::new_sync(config).unwrap();
+        assert!(client.is_ready());
+        assert_eq!(client.bucket_name(), "test-bucket");
+    }
+}
diff --git a/migrations/030_documents.sql b/migrations/030_documents.sql
new file mode 100644
index 0000000..98d7a1d
--- /dev/null
+++ b/migrations/030_documents.sql
@@ -0,0 +1,105 @@
+-- Migration: Document metadata schema
+-- Version: 030
+-- Description: Creates tables for document creation agents system
+
+-- Documents table
+CREATE TABLE IF NOT EXISTS documents (
+    id UUID PRIMARY KEY,
+    domain_id VARCHAR NOT NULL,
+    document_type VARCHAR NOT NULL,
+    state VARCHAR NOT NULL,  -- Uses WorkflowStatus values: Draft, Submitted, Approved, Rejected, Published
+    trust_level VARCHAR NOT NULL,  -- Low, Medium, High
+
+    -- Storage references
+    current_version_key VARCHAR NOT NULL,
+    previous_version_key VARCHAR,
+
+    -- Scores
+    compliance_score FLOAT NOT NULL DEFAULT 0.0,
+    confidence_score FLOAT NOT NULL DEFAULT 0.0,
+
+    -- Request context (JSON)
+    request_context JSON,
+
+    -- Audit timestamps
+    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
+    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
+    published_at TIMESTAMP,
+
+    -- Approval information
+    approved_by VARCHAR,
+    approval_notes TEXT
+);
+
+-- Indexes for common queries
+CREATE INDEX IF NOT EXISTS idx_documents_domain ON documents(domain_id);
+CREATE INDEX IF NOT EXISTS idx_documents_state ON documents(state);
+CREATE INDEX IF NOT EXISTS idx_documents_domain_state ON documents(domain_id, state);
+CREATE INDEX IF NOT EXISTS idx_documents_created ON documents(created_at DESC);
+
+-- Audit trail table
+CREATE TABLE IF NOT EXISTS document_audit (
+    id UUID PRIMARY KEY,
+    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
+    agent_name VARCHAR NOT NULL,
+    action VARCHAR NOT NULL,
+    details JSON,
+    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
+    execution_time_ms INTEGER
+);
+
+CREATE INDEX IF NOT EXISTS idx_audit_document ON document_audit(document_id);
+CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON document_audit(timestamp DESC);
+
+-- Document versions table for full history and rollback support
+CREATE TABLE IF NOT EXISTS document_versions (
+    id UUID PRIMARY KEY,
+    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
+    version_number INTEGER NOT NULL,
+    storage_key VARCHAR NOT NULL,
+    format VARCHAR NOT NULL,  -- Markdown, ODF, PDF
+    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
+    created_by VARCHAR,  -- Agent name or User ID
+    change_summary TEXT,
+    is_current BOOLEAN NOT NULL DEFAULT FALSE,
+    compliance_score FLOAT,
+    UNIQUE(document_id, version_number)
+);
+
+CREATE INDEX IF NOT EXISTS idx_versions_document ON document_versions(document_id);
+CREATE INDEX IF NOT EXISTS idx_versions_current ON document_versions(document_id, is_current);
+
+-- Templates table
+CREATE TABLE IF NOT EXISTS templates (
+    id UUID PRIMARY KEY,
+    name VARCHAR NOT NULL,
+    domain_id VARCHAR NOT NULL,
+    document_type VARCHAR NOT NULL,
+    content TEXT NOT NULL,
+    required_variables JSON,  -- Array of strings
+    optional_sections JSON,    -- Array of strings
+    version INTEGER NOT NULL DEFAULT 1,
+    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
+    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
+    is_active BOOLEAN NOT NULL DEFAULT TRUE
+);
+
+CREATE UNIQUE INDEX IF NOT EXISTS idx_templates_domain_type
+ON templates(domain_id, document_type)
+WHERE is_active = TRUE;
+
+-- Domain configuration table
+CREATE TABLE IF NOT EXISTS domain_configs (
+    domain_id VARCHAR PRIMARY KEY,
+    trust_level VARCHAR NOT NULL,  -- Low, Medium, High
+    required_approval_threshold FLOAT DEFAULT 0.8,
+    auto_approval_threshold FLOAT DEFAULT 0.95,
+    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
+    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
+);
+
+-- Insert default domain configurations
+INSERT INTO domain_configs (domain_id, trust_level, required_approval_threshold, auto_approval_threshold)
+VALUES
+    ('default', 'Low', 0.8, 0.95)
+ON CONFLICT (domain_id) DO NOTHING;
diff --git a/planning/implementation/code_review/section-01-diff.md b/planning/implementation/code_review/section-01-diff.md
new file mode 100644
index 0000000..a9f9899
--- /dev/null
+++ b/planning/implementation/code_review/section-01-diff.md
@@ -0,0 +1,823 @@
+diff --git a/Cargo.toml b/Cargo.toml
+index f1db8e1..19930b1 100644
+--- a/Cargo.toml
++++ b/Cargo.toml
+@@ -6,6 +6,7 @@ members = [
+     "crates/iou-ai",
+     "crates/iou-regels",
+     "crates/iou-frontend",
++    "crates/iou-storage",
+ ]
+ 
+ [workspace.package]
+@@ -50,6 +51,7 @@ tracing-subscriber = { version = "0.3", features = ["env-filter"] }
+ iou-core = { path = "crates/iou-core" }
+ iou-ai = { path = "crates/iou-ai" }
+ iou-regels = { path = "crates/iou-regels" }
++iou-storage = { path = "crates/iou-storage" }
+ 
+ [profile.release]
+ lto = true
+diff --git a/crates/iou-ai/src/document_entity.rs b/crates/iou-ai/src/document_entity.rs
+new file mode 100644
+index 0000000..915118b
+--- /dev/null
++++ b/crates/iou-ai/src/document_entity.rs
+@@ -0,0 +1,127 @@
++//! GraphRAG Document entity schema for document-related knowledge graph operations
++
++use serde::{Deserialize, Serialize};
++use uuid::Uuid;
++
++/// Document entity for GraphRAG knowledge graph
++///
++/// This entity represents stored documents in the knowledge graph,
++/// enabling the Research Agent to query similar documents and extract patterns.
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct DocumentEntity {
++    pub id: Uuid,
++    pub domain_id: String,
++    pub document_type: String,
++    pub title: String,
++    pub content: String,
++    pub sections: Vec<DocumentSection>,
++    pub metadata: DocumentEntityMetadata,
++    pub embeddings: Option<Vec<f32>>,  // For semantic similarity
++    pub created_at: chrono::DateTime<chrono::Utc>,
++}
++
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct DocumentSection {
++    pub name: String,
++    pub content: String,
++    pub is_mandatory: bool,
++    pub order: i32,
++}
++
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct DocumentEntityMetadata {
++    pub author: Option<String>,
++    pub department: Option<String>,
++    pub tags: Vec<String>,
++    pub language: String,  // Default: "nl"
++    pub woo_relevant: bool,
++    pub compliance_score: Option<f32>,
++}
++
++impl DocumentEntity {
++    /// Create a new document entity
++    pub fn new(
++        domain_id: String,
++        document_type: String,
++        title: String,
++        content: String,
++    ) -> Self {
++        Self {
++            id: Uuid::new_v4(),
++            domain_id,
++            document_type,
++            title,
++            content,
++            sections: Vec::new(),
++            metadata: DocumentEntityMetadata {
++                author: None,
++                department: None,
++                tags: Vec::new(),
++                language: "nl".to_string(),
++                woo_relevant: false,
++                compliance_score: None,
++            },
++            embeddings: None,
++            created_at: chrono::Utc::now(),
++        }
++    }
++
++    /// Add a section to the document
++    pub fn with_section(mut self, name: String, content: String, is_mandatory: bool, order: i32) -> Self {
++        self.sections.push(DocumentSection {
++            name,
++            content,
++            is_mandatory,
++            order,
++        });
++        self
++    }
++
++    /// Check if document is Woo-relevant
++    pub fn is_woo_relevant(&self) -> bool {
++        self.metadata.woo_relevant || self.document_type.starts_with("woo_")
++    }
++}
++
++/// Schema definition for Document entity in GraphRAG
++pub struct DocumentSchema;
++
++impl DocumentSchema {
++    /// Entity name in GraphRAG
++    pub const ENTITY_NAME: &'static str = "Document";
++
++    /// Required fields for Document entity
++    pub fn required_fields() -> Vec<&'static str> {
++        vec![
++            "id",
++            "domain_id",
++            "document_type",
++            "content",
++            "created_at",
++        ]
++    }
++
++    /// Optional fields for Document entity
++    pub fn optional_fields() -> Vec<&'static str> {
++        vec![
++            "title",
++            "sections",
++            "embeddings",
++            "woo_relevant",
++            "compliance_score",
++        ]
++    }
++}
++
++#[cfg(test)]
++mod tests {
++    use super::*;
++
++    #[test]
++    fn test_document_entity_schema_is_defined() {
++        assert_eq!(DocumentSchema::ENTITY_NAME, "Document");
++        assert!(DocumentSchema::required_fields().contains(&"id"));
++        assert!(DocumentSchema::required_fields().contains(&"domain_id"));
++        assert!(DocumentSchema::required_fields().contains(&"document_type"));
++    }
++}
+diff --git a/crates/iou-ai/src/lib.rs b/crates/iou-ai/src/lib.rs
+index 5718de7..19e4ad2 100644
+--- a/crates/iou-ai/src/lib.rs
++++ b/crates/iou-ai/src/lib.rs
+@@ -20,12 +20,14 @@
+ 
+ pub mod ner;
+ pub mod graphrag;
++pub mod document_entity;
+ pub mod compliance;
+ pub mod suggestions;
+ pub mod semantic;
+ 
+ pub use ner::DutchNerExtractor;
+ pub use graphrag::KnowledgeGraph;
++pub use document_entity::{DocumentEntity, DocumentSection, DocumentEntityMetadata, DocumentSchema};
+ pub use compliance::ComplianceAssessor;
+ pub use suggestions::MetadataSuggester;
+ pub use semantic::{SemanticSearchService, cosine_similarity};
+diff --git a/crates/iou-core/src/document.rs b/crates/iou-core/src/document.rs
+new file mode 100644
+index 0000000..6c42154
+--- /dev/null
++++ b/crates/iou-core/src/document.rs
+@@ -0,0 +1,319 @@
++//! Document domain types for the multi-agent document creation system.
++//!
++//! This module defines the core data structures used throughout the document
++//! creation pipeline, including states, requests, and metadata.
++
++use chrono::{DateTime, Utc};
++use serde::{Deserialize, Serialize};
++use std::collections::HashMap;
++use uuid::Uuid;
++
++// Reuse existing WorkflowStatus as DocumentState
++pub use crate::workflows::WorkflowStatus as DocumentState;
++
++/// Unique identifier for a document generation request
++pub type DocumentId = Uuid;
++
++/// Trust level determines auto-approval behavior
++#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
++#[serde(rename_all = "lowercase")]
++pub enum TrustLevel {
++    /// Always requires human approval, regardless of compliance score
++    Low,
++    /// Requires approval if compliance_score < required_approval_threshold
++    Medium,
++    /// Auto-approval ONLY for non-Woo documents with high confidence.
++    /// ALL Woo-relevant documents require human approval.
++    High,
++}
++
++impl TrustLevel {
++    /// Check if this trust level requires human approval for the given context
++    pub fn requires_approval(
++        self,
++        is_woo_document: bool,
++        compliance_score: f32,
++        threshold: f32,
++    ) -> bool {
++        match self {
++            TrustLevel::Low => true,
++            TrustLevel::Medium => compliance_score < threshold,
++            TrustLevel::High => {
++                // Woo documents ALWAYS require approval regardless of confidence
++                if is_woo_document {
++                    true
++                } else {
++                    compliance_score < threshold
++                }
++            }
++        }
++    }
++
++    pub fn requires_approval_for_all(self) -> bool {
++        matches!(self, TrustLevel::Low)
++    }
++
++    pub fn requires_approval_if_compliance_below(self, _threshold: f32) -> bool {
++        matches!(self, TrustLevel::Medium)
++    }
++
++    pub fn requires_approval_for_woo(self) -> bool {
++        matches!(self, TrustLevel::High)
++    }
++}
++
++/// IMPORTANT SECURITY NOTE:
++/// - ALL Woo-relevant documents require human approval regardless of confidence score
++/// - Auto-approval only applies to internal, non-sensitive documents where legal compliance is not a concern
++/// - A "dry run" mode should be available for testing auto-approval before enabling it in production
++
++/// Configuration per information domain
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct DomainConfig {
++    pub domain_id: String,
++    pub trust_level: TrustLevel,
++    pub required_approval_threshold: f32,  // For Medium trust
++    pub auto_approval_threshold: f32,      // For High trust
++}
++
++impl DomainConfig {
++    /// Validate that threshold values are within valid range (0.0 - 1.0)
++    pub fn validate_thresholds(&self) -> Result<(), String> {
++        if !(0.0..=1.0).contains(&self.required_approval_threshold) {
++            return Err(format!(
++                "required_approval_threshold must be between 0.0 and 1.0, got {}",
++                self.required_approval_threshold
++            ));
++        }
++        if !(0.0..=1.0).contains(&self.auto_approval_threshold) {
++            return Err(format!(
++                "auto_approval_threshold must be between 0.0 and 1.0, got {}",
++                self.auto_approval_threshold
++            ));
++        }
++        Ok(())
++    }
++
++    /// Check if a document in this domain requires human approval
++    pub fn requires_approval(
++        &self,
++        is_woo_document: bool,
++        compliance_score: f32,
++    ) -> bool {
++        self.trust_level.requires_approval(
++            is_woo_document,
++            compliance_score,
++            self.auto_approval_threshold,
++        )
++    }
++}
++
++/// Document generation request
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct DocumentRequest {
++    pub id: DocumentId,
++    pub domain_id: String,
++    pub document_type: String,
++    pub context: HashMap<String, String>,
++    pub requested_at: DateTime<Utc>,
++}
++
++impl DocumentRequest {
++    pub fn new(domain_id: String, document_type: String, context: HashMap<String, String>) -> Self {
++        Self {
++            id: Uuid::new_v4(),
++            domain_id,
++            document_type,
++            context,
++            requested_at: Utc::now(),
++        }
++    }
++}
++
++/// Document metadata stored in DuckDB
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct DocumentMetadata {
++    pub id: DocumentId,
++    pub domain_id: String,
++    pub document_type: String,
++    pub state: DocumentState,
++    pub current_version_key: String,    // S3 object key
++    pub previous_version_key: Option<String>,
++    pub compliance_score: f32,
++    pub confidence_score: f32,
++    pub created_at: DateTime<Utc>,
++    pub updated_at: DateTime<Utc>,
++}
++
++/// Agent execution result
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct AgentResult {
++    pub agent_name: String,
++    pub success: bool,
++    pub data: serde_json::Value,
++    pub errors: Vec<String>,
++    pub execution_time_ms: u64,
++}
++
++impl AgentResult {
++    pub fn success(agent_name: String, data: serde_json::Value, execution_time_ms: u64) -> Self {
++        Self {
++            agent_name,
++            success: true,
++            data,
++            errors: Vec::new(),
++            execution_time_ms,
++        }
++    }
++
++    pub fn failure(agent_name: String, errors: Vec<String>, execution_time_ms: u64) -> Self {
++        Self {
++            agent_name,
++            success: false,
++            data: serde_json::Value::Null,
++            errors,
++            execution_time_ms,
++        }
++    }
++}
++
++/// Audit trail entry for observability
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct AuditEntry {
++    pub id: Uuid,
++    pub document_id: DocumentId,
++    pub agent_name: String,
++    pub action: String,
++    pub details: serde_json::Value,
++    pub timestamp: DateTime<Utc>,
++    pub execution_time_ms: Option<u64>,
++}
++
++impl AuditEntry {
++    pub fn new(
++        document_id: DocumentId,
++        agent_name: String,
++        action: String,
++        details: serde_json::Value,
++    ) -> Self {
++        Self {
++            id: Uuid::new_v4(),
++            document_id,
++            agent_name,
++            action,
++            details,
++            timestamp: Utc::now(),
++            execution_time_ms: None,
++        }
++    }
++}
++
++/// S3 object reference
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct StorageRef {
++    pub bucket: String,
++    pub key: String,
++    pub version_id: Option<String>,
++    pub content_type: String,
++    pub size_bytes: u64,
++    pub etag: String,
++}
++
++/// Document version in S3
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct DocumentVersion {
++    pub storage_ref: StorageRef,
++    pub format: DocumentFormat,
++    pub created_at: DateTime<Utc>,
++    pub created_by: String,  // Agent or User ID
++}
++
++#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
++pub enum DocumentFormat {
++    Markdown,
++    ODF,   // OpenDocument Format
++    PDF,
++}
++
++impl DocumentFormat {
++    pub fn extension(&self) -> &str {
++        match self {
++            DocumentFormat::Markdown => "md",
++            DocumentFormat::ODF => "odt",
++            DocumentFormat::PDF => "pdf",
++        }
++    }
++
++    pub fn content_type(&self) -> &str {
++        match self {
++            DocumentFormat::Markdown => "text/markdown",
++            DocumentFormat::ODF => "application/vnd.oasis.opendocument.text",
++            DocumentFormat::PDF => "application/pdf",
++        }
++    }
++}
++
++#[cfg(test)]
++mod tests {
++    use super::*;
++    use crate::workflows::WorkflowStatus;
++
++    #[test]
++    fn test_document_id_generates_valid_uuid() {
++        let id = Uuid::new_v4();
++        assert_ne!(id, Uuid::nil());
++    }
++
++    #[test]
++    fn test_document_state_maps_to_workflow_status() {
++        let state: DocumentState = WorkflowStatus::Draft;
++        assert_eq!(state, WorkflowStatus::Draft);
++    }
++
++    #[test]
++    fn test_trust_level_determines_approval_requirements() {
++        let low = TrustLevel::Low;
++        let medium = TrustLevel::Medium;
++        let high = TrustLevel::High;
++
++        assert!(low.requires_approval_for_all());
++        assert!(medium.requires_approval_if_compliance_below(0.8));
++        assert!(high.requires_approval_for_woo());
++    }
++
++    #[test]
++    fn test_domain_config_validates_threshold_ranges() {
++        let config = DomainConfig {
++            domain_id: "test".to_string(),
++            trust_level: TrustLevel::Medium,
++            required_approval_threshold: 0.8,
++            auto_approval_threshold: 0.95,
++        };
++        assert!(config.validate_thresholds().is_ok());
++
++        let invalid_config = DomainConfig {
++            domain_id: "test".to_string(),
++            trust_level: TrustLevel::Medium,
++            required_approval_threshold: 1.5,  // Invalid: > 1.0
++            auto_approval_threshold: 0.95,
++        };
++        assert!(invalid_config.validate_thresholds().is_err());
++    }
++
++    #[test]
++    fn test_document_request_serialization() {
++        let request = DocumentRequest {
++            id: Uuid::new_v4(),
++            domain_id: "test-domain".to_string(),
++            document_type: "woo_besluit".to_string(),
++            context: HashMap::from([
++                ("reference".to_string(), "REF-001".to_string())
++            ]),
++            requested_at: Utc::now(),
++        };
++
++        let json = serde_json::to_string(&request).unwrap();
++        let deserialized: DocumentRequest = serde_json::from_str(&json).unwrap();
++        assert_eq!(deserialized.domain_id, request.domain_id);
++    }
++}
+diff --git a/crates/iou-core/src/lib.rs b/crates/iou-core/src/lib.rs
+index be11936..9b373b3 100644
+--- a/crates/iou-core/src/lib.rs
++++ b/crates/iou-core/src/lib.rs
+@@ -19,6 +19,7 @@ pub mod organization;
+ pub mod graphrag;
+ pub mod api_types;
+ pub mod workflows;
++pub mod document;
+ 
+ // Re-exports voor gemakkelijk gebruik
+ pub use domain::{DomainType, InformationDomain, Case, Project, PolicyTopic};
+@@ -26,3 +27,10 @@ pub use objects::{ObjectType, InformationObject};
+ pub use compliance::{Classification, WooMetadata, AvgMetadata, RetentionPolicy};
+ pub use organization::{Organization, Department, User, Role};
+ pub use workflows::{WorkflowStatus, WorkflowDefinition, WorkflowExecution};
++
++// Document creation types
++pub use document::{
++    DocumentId, DocumentState, TrustLevel, DomainConfig,
++    DocumentRequest, DocumentMetadata, AgentResult, AuditEntry,
++    StorageRef, DocumentVersion, DocumentFormat,
++};
+diff --git a/crates/iou-storage/Cargo.toml b/crates/iou-storage/Cargo.toml
+new file mode 100644
+index 0000000..e2a1dac
+--- /dev/null
++++ b/crates/iou-storage/Cargo.toml
+@@ -0,0 +1,18 @@
++[package]
++name = "iou-storage"
++version = "0.1.0"
++edition = "2021"
++
++[dependencies]
++iou-core = { path = "../iou-core" }
++async-trait = "0.1"
++tokio = { version = "1", features = ["full"] }
++serde = { version = "1", features = ["derive"] }
++serde_json = "1"
++thiserror = "2"
++anyhow = "1"
++chrono = { version = "0.4", features = ["serde"] }
++uuid = { version = "1", features = ["serde", "v4"] }
++
++[dev-dependencies]
++tokio-test = "0.4"
+diff --git a/crates/iou-storage/src/config.rs b/crates/iou-storage/src/config.rs
+new file mode 100644
+index 0000000..c56e9b1
+--- /dev/null
++++ b/crates/iou-storage/src/config.rs
+@@ -0,0 +1,66 @@
++//! Storage configuration
++
++use serde::{Deserialize, Serialize};
++
++/// Storage configuration loaded from environment or config file
++#[derive(Debug, Clone, Serialize, Deserialize)]
++pub struct StorageConfig {
++    /// S3-compatible endpoint URL
++    pub endpoint: String,
++    /// Access key ID
++    pub access_key_id: String,
++    /// Secret access key
++    pub secret_access_key: String,
++    /// Bucket name for document storage
++    pub bucket: String,
++    /// Region (optional for MinIO)
++    pub region: Option<String>,
++    /// Whether to use path-style addressing (required for MinIO)
++    pub force_path_style: bool,
++}
++
++impl StorageConfig {
++    /// Load configuration from environment variables
++    pub fn from_env() -> Result<Self, anyhow::Error> {
++        Ok(Self {
++            endpoint: std::env::var("STORAGE_ENDPOINT")
++                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
++            access_key_id: std::env::var("STORAGE_ACCESS_KEY_ID")
++                .unwrap_or_else(|_| "minioadmin".to_string()),
++            secret_access_key: std::env::var("STORAGE_SECRET_ACCESS_KEY")
++                .unwrap_or_else(|_| "minioadmin".to_string()),
++            bucket: std::env::var("STORAGE_BUCKET")
++                .unwrap_or_else(|_| "iou-documents".to_string()),
++            region: std::env::var("STORAGE_REGION").ok(),
++            force_path_style: std::env::var("STORAGE_FORCE_PATH_STYLE")
++                .unwrap_or_else(|_| "true".to_string())
++                .parse()
++                .unwrap_or(true),
++        })
++    }
++
++    pub fn minio_local() -> Self {
++        Self {
++            endpoint: "http://localhost:9000".to_string(),
++            access_key_id: "minioadmin".to_string(),
++            secret_access_key: "minioadmin".to_string(),
++            bucket: "iou-documents".to_string(),
++            region: None,
++            force_path_style: true,
++        }
++    }
++}
++
++#[cfg(test)]
++impl StorageConfig {
++    pub fn test_mock() -> Self {
++        Self {
++            endpoint: "http://localhost:9000".to_string(),
++            access_key_id: "test-key".to_string(),
++            secret_access_key: "test-secret".to_string(),
++            bucket: "test-bucket".to_string(),
++            region: None,
++            force_path_style: true,
++        }
++    }
++}
+diff --git a/crates/iou-storage/src/lib.rs b/crates/iou-storage/src/lib.rs
+new file mode 100644
+index 0000000..2556e36
+--- /dev/null
++++ b/crates/iou-storage/src/lib.rs
+@@ -0,0 +1,9 @@
++//! Storage abstraction layer for IOU-Modern document system.
++//!
++//! Provides a unified interface for S3/MinIO storage operations.
++
++pub mod config;
++pub mod s3;
++
++pub use config::StorageConfig;
++pub use s3::{S3Client, S3Error};
+diff --git a/crates/iou-storage/src/s3.rs b/crates/iou-storage/src/s3.rs
+new file mode 100644
+index 0000000..cddd337
+--- /dev/null
++++ b/crates/iou-storage/src/s3.rs
+@@ -0,0 +1,75 @@
++//! S3 client wrapper for document storage operations
++
++use thiserror::Error;
++
++pub use crate::config::StorageConfig as S3Config;
++
++#[derive(Error, Debug)]
++pub enum S3Error {
++    #[error("IO error: {0}")]
++    IoError(#[from] std::io::Error),
++
++    #[error("Serialization error: {0}")]
++    SerializationError(#[from] serde_json::Error),
++
++    #[error("Not found: {bucket}/{key}")]
++    NotFound { bucket: String, key: String },
++
++    #[error("S3 operation failed: {0}")]
++    OperationFailed(String),
++}
++
++pub type Result<T> = std::result::Result<T, S3Error>;
++
++/// S3 client wrapper with convenient methods for document operations
++pub struct S3Client {
++    config: S3Config,
++}
++
++impl S3Client {
++    /// Create a new S3 client from configuration
++    pub fn new(config: S3Config) -> Result<Self> {
++        Ok(Self { config })
++    }
++
++    /// Check if client is properly configured
++    pub fn is_ready(&self) -> bool {
++        true
++    }
++
++    /// Generate a storage key for a document
++    pub fn document_key(document_id: &str, version: i32, format: &str) -> String {
++        format!("documents/{}/v{}.{}", document_id, version, format)
++    }
++
++    /// Generate a storage key for a redacted document
++    pub fn redacted_document_key(document_id: &str, version: i32, format: &str) -> String {
++        format!("documents/{}/v{}.redacted.{}", document_id, version, format)
++    }
++
++    /// Get the bucket name
++    pub fn bucket(&self) -> &str {
++        &self.config.bucket
++    }
++}
++
++#[cfg(test)]
++mod tests {
++    use super::*;
++
++    #[test]
++    fn test_document_key_generation() {
++        let key = S3Client::document_key("uuid-here", 1, "md");
++        assert_eq!(key, "documents/uuid-here/v1.md");
++
++        let redacted_key = S3Client::redacted_document_key("uuid-here", 1, "md");
++        assert_eq!(redacted_key, "documents/uuid-here/v1.redacted.md");
++    }
++
++    #[test]
++    fn test_s3_client_compiles() {
++        let config = S3Config::test_mock();
++        let client = S3Client::new(config).unwrap();
++        assert!(client.is_ready());
++    }
++}
+diff --git a/migrations/030_documents.sql b/migrations/030_documents.sql
+new file mode 100644
+index 0000000..98d7a1d
+--- /dev/null
++++ b/migrations/030_documents.sql
+@@ -0,0 +1,105 @@
++-- Migration: Document metadata schema
++-- Version: 030
++-- Description: Creates tables for document creation agents system
++
++-- Documents table
++CREATE TABLE IF NOT EXISTS documents (
++    id UUID PRIMARY KEY,
++    domain_id VARCHAR NOT NULL,
++    document_type VARCHAR NOT NULL,
++    state VARCHAR NOT NULL,  -- Uses WorkflowStatus values: Draft, Submitted, Approved, Rejected, Published
++    trust_level VARCHAR NOT NULL,  -- Low, Medium, High
++
++    -- Storage references
++    current_version_key VARCHAR NOT NULL,
++    previous_version_key VARCHAR,
++
++    -- Scores
++    compliance_score FLOAT NOT NULL DEFAULT 0.0,
++    confidence_score FLOAT NOT NULL DEFAULT 0.0,
++
++    -- Request context (JSON)
++    request_context JSON,
++
++    -- Audit timestamps
++    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
++    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
++    published_at TIMESTAMP,
++
++    -- Approval information
++    approved_by VARCHAR,
++    approval_notes TEXT
++);
++
++-- Indexes for common queries
++CREATE INDEX IF NOT EXISTS idx_documents_domain ON documents(domain_id);
++CREATE INDEX IF NOT EXISTS idx_documents_state ON documents(state);
++CREATE INDEX IF NOT EXISTS idx_documents_domain_state ON documents(domain_id, state);
++CREATE INDEX IF NOT EXISTS idx_documents_created ON documents(created_at DESC);
++
++-- Audit trail table
++CREATE TABLE IF NOT EXISTS document_audit (
++    id UUID PRIMARY KEY,
++    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
++    agent_name VARCHAR NOT NULL,
++    action VARCHAR NOT NULL,
++    details JSON,
++    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
++    execution_time_ms INTEGER
++);
++
++CREATE INDEX IF NOT EXISTS idx_audit_document ON document_audit(document_id);
++CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON document_audit(timestamp DESC);
++
++-- Document versions table for full history and rollback support
++CREATE TABLE IF NOT EXISTS document_versions (
++    id UUID PRIMARY KEY,
++    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
++    version_number INTEGER NOT NULL,
++    storage_key VARCHAR NOT NULL,
++    format VARCHAR NOT NULL,  -- Markdown, ODF, PDF
++    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
++    created_by VARCHAR,  -- Agent name or User ID
++    change_summary TEXT,
++    is_current BOOLEAN NOT NULL DEFAULT FALSE,
++    compliance_score FLOAT,
++    UNIQUE(document_id, version_number)
++);
++
++CREATE INDEX IF NOT EXISTS idx_versions_document ON document_versions(document_id);
++CREATE INDEX IF NOT EXISTS idx_versions_current ON document_versions(document_id, is_current);
++
++-- Templates table
++CREATE TABLE IF NOT EXISTS templates (
++    id UUID PRIMARY KEY,
++    name VARCHAR NOT NULL,
++    domain_id VARCHAR NOT NULL,
++    document_type VARCHAR NOT NULL,
++    content TEXT NOT NULL,
++    required_variables JSON,  -- Array of strings
++    optional_sections JSON,    -- Array of strings
++    version INTEGER NOT NULL DEFAULT 1,
++    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
++    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
++    is_active BOOLEAN NOT NULL DEFAULT TRUE
++);
++
++CREATE UNIQUE INDEX IF NOT EXISTS idx_templates_domain_type
++ON templates(domain_id, document_type)
++WHERE is_active = TRUE;
++
++-- Domain configuration table
++CREATE TABLE IF NOT EXISTS domain_configs (
++    domain_id VARCHAR PRIMARY KEY,
++    trust_level VARCHAR NOT NULL,  -- Low, Medium, High
++    required_approval_threshold FLOAT DEFAULT 0.8,
++    auto_approval_threshold FLOAT DEFAULT 0.95,
++    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
++    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
++);
++
++-- Insert default domain configurations
++INSERT INTO domain_configs (domain_id, trust_level, required_approval_threshold, auto_approval_threshold)
++VALUES
++    ('default', 'Low', 0.8, 0.95)
++ON CONFLICT (domain_id) DO NOTHING;
