//! Mention relationship types
//!
//! A mention relationship connects an entity to a document where it was
//! referenced, capturing the context of the mention.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use iou_core::graphrag::{Relationship, RelationshipType};

/// A mention of an entity within a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionRelationship {
    /// Unique identifier for this mention
    pub id: Uuid,

    /// ID of the entity being mentioned
    pub entity_id: Uuid,

    /// ID of the document containing the mention
    pub document_id: Uuid,

    /// How the entity is mentioned in this document
    pub mention_type: MentionType,

    /// Surrounding text context (optional)
    pub context: Option<String>,

    /// Position within document text
    pub position: Option<TextPosition>,

    /// Confidence of this mention extraction
    pub confidence: f32,

    /// When this mention was created
    pub created_at: DateTime<Utc>,
}

/// Position of text within a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    /// Character offset from start
    pub offset: usize,

    /// Length of mention in characters
    pub length: usize,

    /// Line number (if available)
    pub line: Option<usize>,
}

impl TextPosition {
    /// Create a new text position
    pub fn new(offset: usize, length: usize) -> Self {
        Self {
            offset,
            length,
            line: None,
        }
    }

    /// Create with line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

/// How the entity is mentioned (populated during extraction)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MentionType {
    /// Primary subject of sentence/document
    Subject,

    /// Author of document
    Author,

    /// Recipient of document
    Recipient,

    /// Referenced in passing (default)
    Referenced,
}

impl Default for MentionType {
    fn default() -> Self {
        MentionType::Referenced
    }
}

/// Method used to extract the entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExtractionMethod {
    /// Regex-based extraction
    Regex,

    /// rust-bert NER model
    RustBert,

    /// LLM extraction
    LLM,

    /// Manually added
    Manual,
}

impl MentionRelationship {
    /// Create a new mention relationship
    pub fn new(
        entity_id: Uuid,
        document_id: Uuid,
        mention_type: MentionType,
        confidence: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_id,
            document_id,
            mention_type,
            context: None,
            position: None,
            confidence,
            created_at: Utc::now(),
        }
    }

    /// Add context to the mention
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Add position information
    pub fn with_position(mut self, position: TextPosition) -> Self {
        self.position = Some(position);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mention_relationship_serialization() {
        let mention = MentionRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            MentionType::Author,
            0.95,
        );

        let json = serde_json::to_string(&mention).unwrap();
        let deserialized: MentionRelationship = serde_json::from_str(&json).unwrap();

        assert_eq!(mention.id, deserialized.id);
        assert_eq!(mention.mention_type, deserialized.mention_type);
        assert_eq!(mention.confidence, deserialized.confidence);
    }

    #[test]
    fn test_mention_with_context() {
        let entity_id = Uuid::new_v4();
        let doc_id = Uuid::new_v4();

        let mention = MentionRelationship::new(entity_id, doc_id, MentionType::Subject, 0.9)
            .with_context("Jan de Vries signed this document".to_string());

        assert_eq!(mention.context, Some("Jan de Vries signed this document".to_string()));
        assert_eq!(mention.entity_id, entity_id);
        assert_eq!(mention.document_id, doc_id);
    }

    #[test]
    fn test_mention_with_position() {
        let mention = MentionRelationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            MentionType::Referenced,
            0.8,
        )
        .with_position(TextPosition::new(100, 15).with_line(5));

        assert!(mention.position.is_some());
        let pos = mention.position.unwrap();
        assert_eq!(pos.offset, 100);
        assert_eq!(pos.length, 15);
        assert_eq!(pos.line, Some(5));
    }

    #[test]
    fn test_text_position_new() {
        let pos = TextPosition::new(50, 10);
        assert_eq!(pos.offset, 50);
        assert_eq!(pos.length, 10);
        assert_eq!(pos.line, None);
    }

    #[test]
    fn test_text_position_with_line() {
        let pos = TextPosition::new(50, 10).with_line(3);
        assert_eq!(pos.line, Some(3));
    }

    #[test]
    fn test_mention_type_default() {
        assert_eq!(MentionType::default(), MentionType::Referenced);
    }
}

impl From<MentionRelationship> for Relationship {
    fn from(mention: MentionRelationship) -> Self {
        Relationship {
            id: mention.id,
            source_entity_id: mention.entity_id,
            target_entity_id: mention.document_id,
            relationship_type: RelationshipType::RefersTo,
            weight: 1.0,
            confidence: mention.confidence,
            context: mention.context,
            source_domain_id: None,
            created_at: mention.created_at,
        }
    }
}
