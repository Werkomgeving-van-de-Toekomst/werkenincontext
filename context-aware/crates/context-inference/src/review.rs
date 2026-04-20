// =============================================================================
// Review Module
// =============================================================================
//
// Human review workflow for low-confidence inferences.
// Handles review queue, approval/rejection, and confidence tracking.

use crate::{ContextInferenceResult, InferenceError, InferredContext, ConfidenceScores};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Review status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    Modified,
}

/// Review item for human verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub id: Uuid,
    pub inference_id: Uuid,
    pub status: ReviewStatus,
    pub original_context: InferredContext,
    pub modified_context: Option<InferredContext>,
    pub confidence: ConfidenceScores,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewer: Option<String>,
    pub review_notes: Option<String>,
}

impl ReviewItem {
    /// Create a new review item
    pub fn new(inference_id: Uuid, original_context: InferredContext, confidence: ConfidenceScores) -> Self {
        Self {
            id: Uuid::new_v4(),
            inference_id,
            status: ReviewStatus::Pending,
            original_context,
            modified_context: None,
            confidence,
            created_at: Utc::now(),
            reviewed_at: None,
            reviewer: None,
            review_notes: None,
        }
    }

    /// Approve the inference
    pub fn approve(&mut self, reviewer: String, notes: Option<String>) {
        self.status = ReviewStatus::Approved;
        self.reviewed_at = Some(Utc::now());
        self.reviewer = Some(reviewer);
        self.review_notes = notes;
    }

    /// Reject the inference
    pub fn reject(&mut self, reviewer: String, notes: Option<String>) {
        self.status = ReviewStatus::Rejected;
        self.reviewed_at = Some(Utc::now());
        self.reviewer = Some(reviewer);
        self.review_notes = notes;
    }

    /// Approve with modifications
    pub fn modify(&mut self, reviewer: String, modified_context: InferredContext, notes: Option<String>) {
        self.status = ReviewStatus::Modified;
        self.modified_context = Some(modified_context);
        self.reviewed_at = Some(Utc::now());
        self.reviewer = Some(reviewer);
        self.review_notes = notes;
    }

    /// Get the final context after review
    pub fn final_context(&self) -> &InferredContext {
        self.modified_context.as_ref().unwrap_or(&self.original_context)
    }
}

/// Review queue manager
pub struct ReviewQueue {
    items: Vec<ReviewItem>,
}

impl ReviewQueue {
    /// Create a new review queue
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    /// Add an inference to the review queue
    pub fn add(&mut self, inference_id: Uuid, result: ContextInferenceResult) -> Result<(), InferenceError> {
        if !result.metadata.requires_review {
            return Ok(());
        }

        let item = ReviewItem::new(
            inference_id,
            result.context,
            result.confidence,
        );

        self.items.push(item);
        Ok(())
    }

    /// Get pending reviews
    pub fn pending(&self) -> Vec<&ReviewItem> {
        self.items
            .iter()
            .filter(|i| i.status == ReviewStatus::Pending)
            .collect()
    }

    /// Get a review item by ID
    pub fn get(&self, id: Uuid) -> Option<&ReviewItem> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Get mutable reference to review item
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut ReviewItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Remove completed reviews
    pub fn cleanup_completed(&mut self) -> usize {
        let original_len = self.items.len();
        self.items.retain(|i| i.status == ReviewStatus::Pending);
        original_len - self.items.len()
    }

    /// Statistics
    pub fn stats(&self) -> ReviewStats {
        let total = self.items.len();
        let pending = self.items.iter().filter(|i| i.status == ReviewStatus::Pending).count();
        let approved = self.items.iter().filter(|i| i.status == ReviewStatus::Approved).count();
        let rejected = self.items.iter().filter(|i| i.status == ReviewStatus::Rejected).count();
        let modified = self.items.iter().filter(|i| i.status == ReviewStatus::Modified).count();

        ReviewStats {
            total,
            pending,
            approved,
            rejected,
            modified,
        }
    }
}

impl Default for ReviewQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Review statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStats {
    pub total: usize,
    pub pending: usize,
    pub approved: usize,
    pub rejected: usize,
    pub modified: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreContext, Entity, EntityType};

    #[test]
    fn test_review_item_workflow() {
        let context = InferredContext {
            core: Some(CoreContext {
                title: Some("Test".to_string()),
                creator: None,
                classification: None,
                description: None,
            }),
            domain: None,
            semantic: None,
            legal: None,
            entities: vec![],
        };
        let confidence = ConfidenceScores {
            overall: 0.5,
            by_field: vec![],
        };

        let mut item = ReviewItem::new(Uuid::new_v4(), context, confidence);
        assert_eq!(item.status, ReviewStatus::Pending);

        item.approve("user@example.com".to_string(), None);
        assert_eq!(item.status, ReviewStatus::Approved);
    }

    #[test]
    fn test_review_queue() {
        let mut queue = ReviewQueue::new();

        let context = InferredContext {
            core: Some(CoreContext {
                title: Some("Test".to_string()),
                creator: None,
                classification: None,
                description: None,
            }),
            domain: None,
            semantic: None,
            legal: None,
            entities: vec![],
        };
        let confidence = ConfidenceScores {
            overall: 0.5,
            by_field: vec![],
        };

        queue.add(Uuid::new_v4(), ContextInferenceResult {
            context,
            confidence,
            metadata: crate::InferenceMetadata {
                model: "test".to_string(),
                provider: "test".to_string(),
                inferred_at: Utc::now(),
                processing_time_ms: 100,
                text_length: 100,
                requires_review: true,
                review_reason: Some("test".to_string()),
            },
        }).unwrap();

        assert_eq!(queue.pending().len(), 1);
    }
}
