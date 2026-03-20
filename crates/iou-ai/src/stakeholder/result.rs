//! Result types for stakeholder extraction

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::types::PersonStakeholder;
use super::types::OrganizationStakeholder;
use iou_core::graphrag::{Relationship, RelationshipType};

/// Wrapper for MentionRelationship to avoid circular dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionRelationshipWrapper {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub document_id: Uuid,
    pub confidence: f32,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<MentionRelationshipWrapper> for Relationship {
    fn from(wrapper: MentionRelationshipWrapper) -> Self {
        Relationship {
            id: wrapper.id,
            source_entity_id: wrapper.entity_id,
            target_entity_id: wrapper.document_id,
            relationship_type: RelationshipType::RefersTo,
            weight: 1.0,
            confidence: wrapper.confidence,
            context: None,
            source_domain_id: None,
            created_at: wrapper.created_at.unwrap_or_else(Utc::now),
        }
    }
}

/// Result of the stakeholder extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Extracted person entities
    pub persons: Vec<PersonStakeholder>,

    /// Extracted organization entities
    pub organizations: Vec<OrganizationStakeholder>,

    /// Mention relationships linking entities to documents
    /// (Use MentionRelationship from the mention module)
    pub mentions: Vec<MentionRelationshipWrapper>,

    /// Extraction statistics
    pub stats: ExtractionStats,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Statistics from extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionStats {
    /// Total entities extracted
    pub total_entities: usize,

    /// High confidence entities (>= 0.9)
    pub high_confidence: usize,

    /// Medium confidence entities (0.7 - 0.9)
    pub medium_confidence: usize,

    /// Low confidence entities (0.5 - 0.7)
    pub low_confidence: usize,

    /// LLM API calls made
    pub llm_calls_made: usize,

    /// Rijksoverheid API calls made
    pub api_calls_made: usize,
}

impl ExtractionStats {
    /// Create new stats with all counters at zero
    pub fn new() -> Self {
        Self {
            total_entities: 0,
            high_confidence: 0,
            medium_confidence: 0,
            low_confidence: 0,
            llm_calls_made: 0,
            api_calls_made: 0,
        }
    }

    /// Increment the appropriate confidence bucket based on score
    pub fn track_confidence(&mut self, confidence: f32) {
        self.total_entities += 1;
        if confidence >= 0.9 {
            self.high_confidence += 1;
        } else if confidence >= 0.7 {
            self.medium_confidence += 1;
        } else if confidence >= 0.5 {
            self.low_confidence += 1;
        }
    }

    /// Increment LLM call counter
    pub fn track_llm_call(&mut self) {
        self.llm_calls_made += 1;
    }

    /// Increment API call counter
    pub fn track_api_call(&mut self) {
        self.api_calls_made += 1;
    }
}

impl Default for ExtractionStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification status based on confidence score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationStatus {
    /// Auto-accepted (confidence >= 0.9)
    AutoAccepted,

    /// Accepted but flagged for review (0.7 <= confidence < 0.9)
    AcceptedFlagged,

    /// Pending manual review (0.5 <= confidence < 0.7)
    PendingReview,

    /// Rejected (confidence < 0.5)
    Rejected,
}

impl VerificationStatus {
    /// Determine status from confidence score
    pub fn from_confidence(confidence: f32) -> Self {
        if confidence >= 0.9 {
            VerificationStatus::AutoAccepted
        } else if confidence >= 0.7 {
            VerificationStatus::AcceptedFlagged
        } else if confidence >= 0.5 {
            VerificationStatus::PendingReview
        } else {
            VerificationStatus::Rejected
        }
    }

    /// Check if this status represents an accepted entity
    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::AutoAccepted | Self::AcceptedFlagged)
    }

    /// Check if this status requires review
    pub fn requires_review(&self) -> bool {
        matches!(self, Self::AcceptedFlagged | Self::PendingReview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_status_from_confidence() {
        assert_eq!(VerificationStatus::from_confidence(0.95), VerificationStatus::AutoAccepted);
        assert_eq!(VerificationStatus::from_confidence(0.9), VerificationStatus::AutoAccepted);
        assert_eq!(VerificationStatus::from_confidence(0.85), VerificationStatus::AcceptedFlagged);
        assert_eq!(VerificationStatus::from_confidence(0.7), VerificationStatus::AcceptedFlagged);
        assert_eq!(VerificationStatus::from_confidence(0.6), VerificationStatus::PendingReview);
        assert_eq!(VerificationStatus::from_confidence(0.5), VerificationStatus::PendingReview);
        assert_eq!(VerificationStatus::from_confidence(0.4), VerificationStatus::Rejected);
    }

    #[test]
    fn test_verification_status_is_accepted() {
        assert!(VerificationStatus::AutoAccepted.is_accepted());
        assert!(VerificationStatus::AcceptedFlagged.is_accepted());
        assert!(!VerificationStatus::PendingReview.is_accepted());
        assert!(!VerificationStatus::Rejected.is_accepted());
    }

    #[test]
    fn test_verification_status_requires_review() {
        assert!(!VerificationStatus::AutoAccepted.requires_review());
        assert!(VerificationStatus::AcceptedFlagged.requires_review());
        assert!(VerificationStatus::PendingReview.requires_review());
        assert!(!VerificationStatus::Rejected.requires_review());
    }

    #[test]
    fn test_extraction_stats_track_confidence() {
        let mut stats = ExtractionStats::new();

        stats.track_confidence(0.95);
        assert_eq!(stats.total_entities, 1);
        assert_eq!(stats.high_confidence, 1);

        stats.track_confidence(0.8);
        assert_eq!(stats.total_entities, 2);
        assert_eq!(stats.medium_confidence, 1);

        stats.track_confidence(0.6);
        assert_eq!(stats.total_entities, 3);
        assert_eq!(stats.low_confidence, 1);
    }

    #[test]
    fn test_extraction_stats_track_calls() {
        let mut stats = ExtractionStats::new();

        stats.track_llm_call();
        assert_eq!(stats.llm_calls_made, 1);

        stats.track_api_call();
        assert_eq!(stats.api_calls_made, 1);
    }
}
