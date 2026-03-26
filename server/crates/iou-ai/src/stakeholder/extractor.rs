//! Main stakeholder extraction interface
//!
//! This module defines the public trait that all extractor implementations
//! must fulfill. The trait is object-safe, allowing for dynamic dispatch
//! and boxed trait objects.

use std::time::Duration;
use async_trait::async_trait;

use crate::agents::content::GeneratedDocument;
use iou_core::graphrag::Entity;

use super::result::ExtractionResult;
use super::error::{ExtractionError, NormalizationError, DeduplicationError};

/// Main stakeholder extraction interface
///
/// This trait defines the contract for stakeholder extraction implementations.
/// It is object-safe, meaning it can be used as a trait object (Box<dyn StakeholderExtractor>).
#[async_trait]
pub trait StakeholderExtractor: Send + Sync {
    /// Extract entities from a document
    ///
    /// # Arguments
    /// * `document` - The document to extract entities from
    /// * `options` - Configuration options for extraction
    ///
    /// # Returns
    /// Extraction result containing entities, mentions, and statistics
    async fn extract(
        &self,
        document: &GeneratedDocument,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, ExtractionError>;

    /// Normalize entities using external API
    ///
    /// This method resolves entity names to their canonical forms,
    /// particularly for Dutch government organizations.
    async fn normalize_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, NormalizationError>;

    /// Deduplicate entities by similarity
    ///
    /// Identifies and merges duplicate entities using string similarity
    /// and clustering algorithms.
    async fn deduplicate_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, DeduplicationError>;
}

/// Configuration options for extraction process
#[derive(Debug, Clone)]
pub struct ExtractionOptions {
    /// Whether to use LLM for uncertain cases
    pub use_llm: bool,

    /// Minimum confidence threshold
    pub confidence_threshold: f32,

    /// Whether to normalize via external API
    pub enable_normalization: bool,

    /// Maximum LLM calls per document (cost control)
    pub max_llm_calls: usize,

    /// Maximum cost per document in USD (cost control)
    pub max_cost_per_document: f32,

    /// Timeout for LLM API calls
    pub llm_timeout: Duration,
}

impl Default for ExtractionOptions {
    fn default() -> Self {
        Self {
            use_llm: true,
            confidence_threshold: 0.5,
            enable_normalization: true,
            max_llm_calls: 10,
            max_cost_per_document: 0.10, // $0.10 per document max
            llm_timeout: Duration::from_secs(10),
        }
    }
}

impl ExtractionOptions {
    /// Create options optimized for cost
    pub fn cost_effective() -> Self {
        Self {
            use_llm: false,
            confidence_threshold: 0.6,
            enable_normalization: true,
            max_llm_calls: 0,
            max_cost_per_document: 0.01,
            llm_timeout: Duration::from_secs(5),
        }
    }

    /// Create options optimized for accuracy
    pub fn high_accuracy() -> Self {
        Self {
            use_llm: true,
            confidence_threshold: 0.4, // Lower threshold, let LLM filter
            enable_normalization: true,
            max_llm_calls: 20,
            max_cost_per_document: 0.50,
            llm_timeout: Duration::from_secs(30),
        }
    }

    /// Check if a cost estimate is within the configured limit
    pub fn is_within_cost_limit(&self, estimated_cost: f32) -> bool {
        estimated_cost <= self.max_cost_per_document
    }

    /// Check if a confidence score meets the threshold
    pub fn meets_threshold(&self, confidence: f32) -> bool {
        confidence >= self.confidence_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_options_default() {
        let opts = ExtractionOptions::default();

        assert!(opts.use_llm);
        assert_eq!(opts.confidence_threshold, 0.5);
        assert!(opts.enable_normalization);
        assert_eq!(opts.max_llm_calls, 10);
        assert_eq!(opts.max_cost_per_document, 0.10);
    }

    #[test]
    fn test_extraction_options_cost_effective() {
        let opts = ExtractionOptions::cost_effective();

        assert!(!opts.use_llm);
        assert_eq!(opts.confidence_threshold, 0.6);
        assert_eq!(opts.max_llm_calls, 0);
        assert_eq!(opts.max_cost_per_document, 0.01);
    }

    #[test]
    fn test_extraction_options_high_accuracy() {
        let opts = ExtractionOptions::high_accuracy();

        assert!(opts.use_llm);
        assert_eq!(opts.confidence_threshold, 0.4);
        assert_eq!(opts.max_llm_calls, 20);
        assert_eq!(opts.max_cost_per_document, 0.50);
    }

    #[test]
    fn test_is_within_cost_limit() {
        let opts = ExtractionOptions::default();

        assert!(opts.is_within_cost_limit(0.05));
        assert!(opts.is_within_cost_limit(0.10));
        assert!(!opts.is_within_cost_limit(0.15));
    }

    #[test]
    fn test_meets_threshold() {
        let opts = ExtractionOptions::default();

        assert!(opts.meets_threshold(0.5));
        assert!(opts.meets_threshold(0.7));
        assert!(!opts.meets_threshold(0.4));
    }
}
