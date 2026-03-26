//! Mock stakeholder extractor for testing
//!
//! This module provides a mock implementation of the StakeholderExtractor
//! trait for use in tests, particularly in pipeline integration tests.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use async_trait::async_trait;

use crate::stakeholder::{
    StakeholderExtractor, ExtractionResult, ExtractionOptions, ExtractionError,
    extractor::ExtractionOptions,
    result::{ExtractionStats, MentionRelationshipWrapper},
    types::{PersonStakeholder, OrganizationStakeholder},
};
use crate::agents::content::GeneratedDocument;
use iou_core::graphrag::Entity;

/// Mock stakeholder extractor for testing
pub struct MockStakeholderExtractor {
    called: Arc<AtomicBool>,
    call_count: Arc<AtomicUsize>,
    expected_result: Option<ExtractionResult>,
    fail: bool,
    received_document_content: Arc<std::sync::RwLock<Option<String>>>,
    received_options: Arc<std::sync::RwLock<Option<ExtractionOptions>>>,
}

impl MockStakeholderExtractor {
    /// Create a new mock extractor
    pub fn new() -> Self {
        Self {
            called: Arc::new(AtomicBool::new(false)),
            call_count: Arc::new(AtomicUsize::new(0)),
            expected_result: None,
            fail: false,
            received_document_content: Arc::new(std::sync::RwLock::new(None)),
            received_options: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    /// Create a mock extractor with predefined entities
    pub fn with_entities(persons: usize, organizations: usize) -> Self {
        let mut result = ExtractionResult::default();

        // Create mock persons
        for i in 0..persons {
            result.persons.push(PersonStakeholder::new(
                format!("Person {}", i),
                0.9,
            ));
        }

        // Create mock organizations
        for i in 0..organizations {
            result.organizations.push(OrganizationStakeholder::new(
                format!("Organization {}", i),
                0.9,
            ));
        }

        // Create mock mentions
        let total = persons + organizations;
        for i in 0..total {
            result.mentions.push(MentionRelationshipWrapper {
                id: Uuid::new_v4(),
                entity_id: Uuid::new_v4(),
                document_id: Uuid::new_v4(),
                confidence: 0.9,
                created_at: Some(chrono::Utc::now()),
            });
        }

        result.stats = ExtractionStats {
            total_entities: total as usize,
            high_confidence: total as usize,
            medium_confidence: 0,
            low_confidence: 0,
            llm_calls_made: 0,
            api_calls_made: 0,
        };

        Self {
            called: Arc::new(AtomicBool::new(false)),
            call_count: Arc::new(AtomicUsize::new(0)),
            expected_result: Some(result),
            fail: false,
            received_document_content: Arc::new(std::sync::RwLock::new(None)),
            received_options: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    /// Create a mock extractor that always fails
    pub fn failing() -> Self {
        Self {
            called: Arc::new(AtomicBool::new(false)),
            call_count: Arc::new(AtomicUsize::new(0)),
            expected_result: None,
            fail: true,
            received_document_content: Arc::new(std::sync::RwLock::new(None)),
            received_options: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    /// Set the expected result for the next call
    pub fn set_expected_result(&self, result: ExtractionResult) {
        let _ = self
            .received_document_content
            .write()
            .map(|_| {
                self.expected_result = Some(result);
            });
    }

    /// Check if the extractor was called
    pub fn was_called(&self) -> bool {
        self.called.load(Ordering::SeqCst)
    }

    /// Get the number of times the extractor was called
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    /// Check if a document was received
    pub fn received_document(&self) -> bool {
        if let Ok(content) = self.received_document_content.try_read() {
            content.is_some()
        } else {
            false
        }
    }

    /// Get the received document content
    pub fn get_document_content(&self) -> Option<String> {
        self.received_document_content
            .try_read()
            .ok()
            .and_then(|opt| opt.as_ref().cloned())
    }

    /// Get the received options
    pub fn get_received_options(&self) -> Option<ExtractionOptions> {
        self.received_options
            .try_read()
            .ok()
            .and_then(|opt| opt.as_ref().cloned())
    }

    /// Reset the mock state
    pub fn reset(&self) {
        self.called.store(false, Ordering::SeqCst);
        self.call_count.store(0, Ordering::SeqCst);
        let _ = self.received_document_content.write().map(|mut v| *v = None);
        let _ = self.received_options.write().map(|mut v| *v = None);
    }
}

impl Default for MockStakeholderExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StakeholderExtractor for MockStakeholderExtractor {
    async fn extract(
        &self,
        document: &GeneratedDocument,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, ExtractionError> {
        self.called.store(true, Ordering::SeqCst);
        self.call_count.fetch_add(1, Ordering::SeqCst);

        // Store received document content
        let _ = self
            .received_document_content
            .write()
            .map(|mut v| *v = Some(document.content.clone()));

        // Store received options
        let _ = self
            .received_options
            .write()
            .map(|mut v| *v = Some(options.clone()));

        if self.fail {
            return Err(ExtractionError::ExtractionFailed(
                "Mock extraction failure".to_string(),
            ));
        }

        Ok(self
            .expected_result
            .clone()
            .unwrap_or_default())
    }

    async fn normalize_entities(
        &self,
        _entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, ExtractionError> {
        unimplemented!("Mock extractor does not implement normalize_entities")
    }

    async fn deduplicate_entities(
        &self,
        _entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, ExtractionError> {
        unimplemented!("Mock extractor does not implement deduplicate_entities")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_extractor_tracks_calls() {
        let extractor = MockStakeholderExtractor::new();

        assert!(!extractor.was_called());
        assert_eq!(extractor.call_count(), 0);

        // Call extract
        let _ = extractor
            .extract(
                &GeneratedDocument {
                    document_id: Uuid::new_v4(),
                    content: "Test content".to_string(),
                    variables: vec![],
                    entity_links: vec![],
                    sections: vec![],
                    generated_at: chrono::Utc::now(),
                },
                &ExtractionOptions::default(),
            )
            .await
            .unwrap();

        assert!(extractor.was_called());
        assert_eq!(extractor.call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_extractor_returns_content() {
        let extractor = MockStakeholderExtractor::new();

        let _ = extractor
            .extract(
                &GeneratedDocument {
                    document_id: Uuid::new_v4(),
                    content: "dr. Jan de Vries werkt bij MinFin".to_string(),
                    variables: vec![],
                    entity_links: vec![],
                    sections: vec![],
                    generated_at: chrono::Utc::now(),
                },
                &ExtractionOptions::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            extractor.get_document_content(),
            Some("dr. Jan de Vries werkt bij MinFin".to_string())
        );
    }

    #[tokio::test]
    async fn test_mock_extractor_with_entities() {
        let extractor = MockStakeholderExtractor::with_entities(2, 1);

        let result = extractor
            .extract(
                &GeneratedDocument {
                    document_id: Uuid::new_v4(),
                    content: "Test".to_string(),
                    variables: vec![],
                    entity_links: vec![],
                    sections: vec![],
                    generated_at: chrono::Utc::now(),
                },
                &ExtractionOptions::default(),
            )
            .await
            .unwrap();

        assert_eq!(result.persons.len(), 2);
        assert_eq!(result.organizations.len(), 1);
        assert_eq!(result.mentions.len(), 3);
    }

    #[tokio::test]
    async fn test_mock_extractor_failing() {
        let extractor = MockStakeholderExtractor::failing();

        let result = extractor
            .extract(
                &GeneratedDocument {
                    document_id: Uuid::new_v4(),
                    content: "Test".to_string(),
                    variables: vec![],
                    entity_links: vec![],
                    sections: vec![],
                    generated_at: chrono::Utc::now(),
                },
                &ExtractionOptions::default(),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokioio::test]
    async fn test_mock_extractor_reset() {
        let extractor = MockStakeholderExtractor::new();

        let _ = extractor
            .extract(
                &GeneratedDocument {
                    document_id: Uuid::new_v4(),
                    content: "Test".to_string(),
                    variables: vec![],
                    entity_links: vec![],
                    sections: vec![],
                    generated_at: chrono::Utc::now(),
                },
                &ExtractionOptions::default(),
            )
            .await
            .unwrap();

        assert!(extractor.was_called());
        assert_eq!(extractor.call_count(), 1);

        extractor.reset();

        assert!(!extractor.was_called());
        assert_eq!(extractor.call_count(), 0);
    }
}
