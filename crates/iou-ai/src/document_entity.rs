//! GraphRAG Document entity schema for document-related knowledge graph operations

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Document entity for GraphRAG knowledge graph
///
/// This entity represents stored documents in the knowledge graph,
/// enabling the Research Agent to query similar documents and extract patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEntity {
    pub id: Uuid,
    pub domain_id: String,
    pub document_type: String,
    pub title: String,
    pub content: String,
    pub sections: Vec<DocumentSection>,
    pub metadata: DocumentEntityMetadata,
    pub embeddings: Option<Vec<f32>>,  // For semantic similarity
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    pub name: String,
    pub content: String,
    pub is_mandatory: bool,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEntityMetadata {
    pub author: Option<String>,
    pub department: Option<String>,
    pub tags: Vec<String>,
    pub language: String,  // Default: "nl"
    pub woo_relevant: bool,
    pub compliance_score: Option<f32>,
}

impl DocumentEntity {
    /// Create a new document entity
    pub fn new(
        domain_id: String,
        document_type: String,
        title: String,
        content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            domain_id,
            document_type,
            title,
            content,
            sections: Vec::new(),
            metadata: DocumentEntityMetadata {
                author: None,
                department: None,
                tags: Vec::new(),
                language: "nl".to_string(),
                woo_relevant: false,
                compliance_score: None,
            },
            embeddings: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Add a section to the document
    pub fn with_section(mut self, name: String, content: String, is_mandatory: bool, order: i32) -> Self {
        self.sections.push(DocumentSection {
            name,
            content,
            is_mandatory,
            order,
        });
        self
    }

    /// Check if document is Woo-relevant
    pub fn is_woo_relevant(&self) -> bool {
        self.metadata.woo_relevant || self.document_type.starts_with("woo_")
    }
}

/// Schema definition for Document entity in GraphRAG
pub struct DocumentSchema;

impl DocumentSchema {
    /// Entity name in GraphRAG
    pub const ENTITY_NAME: &'static str = "Document";

    /// Required fields for Document entity
    pub fn required_fields() -> Vec<&'static str> {
        vec![
            "id",
            "domain_id",
            "document_type",
            "content",
            "created_at",
        ]
    }

    /// Optional fields for Document entity
    pub fn optional_fields() -> Vec<&'static str> {
        vec![
            "title",
            "sections",
            "embeddings",
            "woo_relevant",
            "compliance_score",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_entity_schema_is_defined() {
        assert_eq!(DocumentSchema::ENTITY_NAME, "Document");
        assert!(DocumentSchema::required_fields().contains(&"id"));
        assert!(DocumentSchema::required_fields().contains(&"domain_id"));
        assert!(DocumentSchema::required_fields().contains(&"document_type"));
    }
}
