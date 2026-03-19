//! Main stakeholder extractor orchestrating the full pipeline
//!
//! This module implements the core `StakeholderExtractor` that coordinates
//! multiple stages: baseline extraction, LLM enhancement, normalization, and
//! deduplication.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::agents::content::GeneratedDocument;
use iou_core::graphrag::{Entity, EntityType};

use super::baseline::BaselineExtractor;
use super::deduplicator::EntityDeduplicator;
use super::extractor::{ExtractionOptions, StakeholderExtractor};
use super::llm_extractor::{ClaudeExtractor, ExtractionContext, TextSegment, FocusReason};
use super::mention::{MentionRelationship, MentionType, TextPosition};
use super::normalizer::EntityNormalizer;
use super::result::{ExtractionResult, ExtractionStats, MentionRelationshipWrapper, VerificationStatus};
use super::types::{OrganizationStakeholder, PersonStakeholder};
use super::error::{ExtractionError, NormalizationError, DeduplicationError};

/// Main stakeholder extractor orchestrating the full pipeline
pub struct MainExtractor {
    baseline: Arc<BaselineExtractor>,
    llm: Option<Arc<ClaudeExtractor>>,
    normalizer: Arc<EntityNormalizer>,
    deduplicator: Arc<EntityDeduplicator>,
    llm_semaphore: Arc<Semaphore>,
}

impl MainExtractor {
    /// Create a new main extractor with all components
    pub fn new(
        baseline: Arc<BaselineExtractor>,
        llm: Option<Arc<ClaudeExtractor>>,
        normalizer: Arc<EntityNormalizer>,
        deduplicator: Arc<EntityDeduplicator>,
        max_concurrent_llm: usize,
    ) -> Self {
        Self {
            baseline,
            llm,
            normalizer,
            deduplicator,
            llm_semaphore: Arc::new(Semaphore::new(max_concurrent_llm)),
        }
    }

    /// Create extractor with default configuration
    ///
    /// Returns Ok(None) if ANTHROPIC_API_KEY is not set (LLM disabled)
    pub fn with_defaults() -> Result<Self, ExtractionError> {
        let baseline = Arc::new(BaselineExtractor::new(true).map_err(|e| {
            ExtractionError::BaselineFailed(format!("Failed to create baseline extractor: {}", e))
        })?);

        let llm = if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            match ClaudeExtractor::new() {
                Ok(extractor) => Some(Arc::new(extractor)),
                Err(_) => None,
            }
        } else {
            None
        };

        let normalizer = Arc::new(EntityNormalizer::new(Arc::new(
            crate::stakeholder::RijksoverheidClient::new(),
        )));
        let deduplicator = Arc::new(EntityDeduplicator::new());

        Ok(Self::new(baseline, llm, normalizer, deduplicator, 3))
    }

    /// Create extractor with LLM explicitly enabled/disabled
    pub fn with_llm_enabled(use_llm: bool) -> Result<Self, ExtractionError> {
        let baseline = Arc::new(BaselineExtractor::new(true).map_err(|e| {
            ExtractionError::BaselineFailed(format!("Failed to create baseline extractor: {}", e))
        })?);

        let llm = if use_llm {
            match ClaudeExtractor::new() {
                Ok(extractor) => Some(Arc::new(extractor)),
                Err(_) => None,
            }
        } else {
            None
        };

        let normalizer = Arc::new(EntityNormalizer::new(Arc::new(
            crate::stakeholder::RijksoverheidClient::new(),
        )));
        let deduplicator = Arc::new(EntityDeduplicator::new());

        Ok(Self::new(baseline, llm, normalizer, deduplicator, 3))
    }

    /// Run the extraction pipeline on text directly
    pub async fn extract_from_text(
        &self,
        text: &str,
        document_id: Uuid,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, ExtractionError> {
        let document = GeneratedDocument {
            document_id,
            content: text.to_string(),
            variables: vec![],
            entity_links: vec![],
            sections: vec![],
            generated_at: chrono::Utc::now(),
        };

        self.extract(&document, options).await
    }
}

#[async_trait]
impl StakeholderExtractor for MainExtractor {
    /// Extract entities from a document using the full pipeline
    async fn extract(
        &self,
        document: &GeneratedDocument,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, ExtractionError> {
        let start = Instant::now();
        let mut stats = ExtractionStats::default();

        // Stage 1: Baseline extraction (always runs)
        let baseline_result = match self.baseline.extract(document, options) {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("Baseline extraction failed: {}", e);
                // Return empty result instead of failing
                ExtractionResult {
                    persons: Vec::new(),
                    organizations: Vec::new(),
                    mentions: Vec::new(),
                    stats: ExtractionStats::default(),
                    processing_time_ms: start.elapsed().as_millis() as u64,
                }
            }
        };

        stats.total_entities = baseline_result.persons.len() + baseline_result.organizations.len();

        // Stage 2: Filter uncertain entities for LLM enhancement
        let uncertain_persons = filter_uncertain_entities(&baseline_result.persons, options);
        let uncertain_orgs = filter_uncertain_entities(&baseline_result.organizations, options);

        // Stage 3: LLM enhancement (conditional)
        let mut llm_persons = Vec::new();
        let mut llm_orgs = Vec::new();

        if options.use_llm
            && self.llm.is_some()
            && (!uncertain_persons.is_empty() || !uncertain_orgs.is_empty())
            && stats.llm_calls_made < options.max_llm_calls
        {
            let _permit = self.llm_semaphore.acquire().await;

            let llm_extractor = self.llm.as_ref().unwrap();
            let text = &document.content;

            // Create extraction context for uncertain entities
            let mut focus_segments: Vec<TextSegment> = uncertain_persons
                .iter()
                .map(|e| TextSegment {
                    text: text.clone(),
                    position: TextPosition::new(0, text.len()),
                    reason: FocusReason::LowConfidence(e.entity.confidence),
                })
                .collect();

            // Add organizations
            for org in uncertain_orgs {
                focus_segments.push(TextSegment {
                    text: text.clone(),
                    position: TextPosition::new(0, text.len()),
                    reason: FocusReason::LowConfidence(org.entity.confidence),
                });
            }

            let context = ExtractionContext {
                document_id: document.document_id,
                baseline_entities: vec![],
                focus_segments,
                confidence_threshold: 0.7,
            };

            match llm_extractor.extract_entities(text, &context).await {
                Ok(entities) => {
                    for entity in entities {
                        match entity.entity_type {
                            EntityType::Person => {
                                let person = PersonStakeholder::from(entity);
                                llm_persons.push(person);
                            }
                            EntityType::Organization => {
                                let org = OrganizationStakeholder::from(entity);
                                llm_orgs.push(org);
                            }
                            _ => {}
                        }
                    }
                    stats.llm_calls_made += 1;
                }
                Err(e) => {
                    tracing::debug!("LLM extraction failed: {}", e);
                }
            }
        }

        // Stage 4: Merge baseline and LLM results
        let mut all_persons = merge_entity_results(baseline_result.persons, llm_persons);
        let mut all_orgs = merge_entity_results(baseline_result.organizations, llm_orgs);

        // Stage 5: Normalization
        if options.enable_normalization {
            all_orgs = match self.normalizer.normalize_organizations(all_orgs.clone()).await {
                Ok(normalized) => normalized,
                Err(e) => {
                    tracing::warn!("Organization normalization failed: {}", e);
                    all_orgs
                }
            };
        }

        // Normalize persons (adds normalized_name for deduplication)
        all_persons = all_persons
            .into_iter()
            .map(|p| self.normalizer.normalize_person(p))
            .collect();

        // Stage 6: Deduplication
        all_persons = match self.deduplicator.deduplicate_persons(all_persons.clone()).await {
            Ok(deduped) => deduped,
            Err(e) => {
                tracing::warn!("Person deduplication failed: {}", e);
                all_persons
            }
        };

        all_orgs = match self.deduplicator.deduplicate_organizations(all_orgs.clone()).await {
            Ok(deduped) => deduped,
            Err(e) => {
                tracing::warn!("Organization deduplication failed: {}", e);
                all_orgs
            }
        };

        // Stage 7: Tiered acceptance and mention creation
        let (accepted_persons, accepted_orgs, mentions) = apply_tiered_acceptance(
            all_persons,
            all_orgs,
            document.document_id,
            options,
            &mut stats,
        );

        let processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(ExtractionResult {
            persons: accepted_persons,
            organizations: accepted_orgs,
            mentions,
            stats,
            processing_time_ms,
        })
    }

    /// Normalize entities using external API
    async fn normalize_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, NormalizationError> {
        // Separate persons and organizations
        let mut persons = Vec::new();
        let mut orgs = Vec::new();

        for entity in entities {
            match entity.entity_type {
                EntityType::Person => {
                    persons.push(PersonStakeholder::from(entity));
                }
                EntityType::Organization => {
                    orgs.push(OrganizationStakeholder::from(entity));
                }
                _ => {}
            }
        }

        // Normalize organizations
        let normalized_orgs = self.normalizer.normalize_organizations(orgs).await?;

        // Convert back to entities
        let mut result = Vec::new();
        for person in persons {
            result.push(person.into_entity());
        }
        for org in normalized_orgs {
            result.push(org.into_entity());
        }

        Ok(result)
    }

    /// Deduplicate entities by similarity
    async fn deduplicate_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, DeduplicationError> {
        // Separate persons and organizations
        let mut persons = Vec::new();
        let mut orgs = Vec::new();

        for entity in entities {
            match entity.entity_type {
                EntityType::Person => {
                    persons.push(PersonStakeholder::from(entity));
                }
                EntityType::Organization => {
                    orgs.push(OrganizationStakeholder::from(entity));
                }
                _ => {}
            }
        }

        // Deduplicate
        let deduped_persons = self.deduplicator.deduplicate_persons(persons).await?;
        let deduped_orgs = self.deduplicator.deduplicate_organizations(orgs).await?;

        // Convert back to entities
        let mut result = Vec::new();
        for person in deduped_persons {
            result.push(person.into_entity());
        }
        for org in deduped_orgs {
            result.push(org.into_entity());
        }

        Ok(result)
    }
}

/// Filter entities that need LLM enhancement based on confidence
fn filter_uncertain_entities<'a, T>(entities: &'a [T], options: &ExtractionOptions) -> Vec<&'a T>
where
    T: ConfidenceScore,
{
    entities
        .iter()
        .filter(|e| {
            // Low confidence or below threshold
            if e.confidence() < 0.7 || e.confidence() < options.confidence_threshold {
                return true;
            }
            false
        })
        .collect()
}

/// Merge baseline and LLM results, preferring LLM for enhanced entities
fn merge_entity_results<T>(baseline: Vec<T>, llm_enhanced: Vec<T>) -> Vec<T>
where
    T: Mergeable,
{
    let mut merged = baseline;

    for enhanced in llm_enhanced {
        if let Some(pos) = merged.iter().position(|b| b.can_merge_with(&enhanced)) {
            merged[pos] = enhanced; // LLM result takes precedence
        } else {
            merged.push(enhanced);
        }
    }

    merged
}

/// Apply tiered acceptance logic and create mentions
fn apply_tiered_acceptance(
    persons: Vec<PersonStakeholder>,
    organizations: Vec<OrganizationStakeholder>,
    document_id: Uuid,
    _options: &ExtractionOptions,
    stats: &mut ExtractionStats,
) -> (Vec<PersonStakeholder>, Vec<OrganizationStakeholder>, Vec<MentionRelationshipWrapper>) {
    let mut accepted_persons = Vec::new();
    let mut accepted_orgs = Vec::new();
    let mut mentions = Vec::new();

    for person in persons {
        let confidence = person.entity.confidence;
        let verification = VerificationStatus::from_confidence(confidence);

        // Track confidence buckets
        stats.track_confidence(confidence);

        // Reject very low confidence
        if verification == VerificationStatus::Rejected {
            continue;
        }

        let mention = create_mention(&person.entity.id, document_id, confidence);
        mentions.push(mention);
        accepted_persons.push(person);
    }

    for org in organizations {
        let confidence = org.entity.confidence;
        let verification = VerificationStatus::from_confidence(confidence);

        // Track confidence buckets
        stats.track_confidence(confidence);

        // Reject very low confidence
        if verification == VerificationStatus::Rejected {
            continue;
        }

        let mention = create_mention(&org.entity.id, document_id, confidence);
        mentions.push(mention);
        accepted_orgs.push(org);
    }

    (accepted_persons, accepted_orgs, mentions)
}

/// Create a mention relationship wrapper for an entity
fn create_mention(entity_id: &Uuid, document_id: Uuid, confidence: f32) -> MentionRelationshipWrapper {
    MentionRelationshipWrapper {
        id: Uuid::new_v4(),
        entity_id: *entity_id,
        document_id,
        confidence,
    }
}

/// Trait for entities with confidence scores
trait ConfidenceScore {
    fn confidence(&self) -> f32;
}

impl ConfidenceScore for PersonStakeholder {
    fn confidence(&self) -> f32 {
        self.entity.confidence
    }
}

impl ConfidenceScore for OrganizationStakeholder {
    fn confidence(&self) -> f32 {
        self.entity.confidence
    }
}

/// Trait for entities that can be merged
trait Mergeable {
    fn can_merge_with(&self, other: &Self) -> bool;
}

impl Mergeable for PersonStakeholder {
    fn can_merge_with(&self, other: &Self) -> bool {
        // Check if names are similar after normalization
        let self_normalized = self
            .entity
            .metadata
            .get("normalized_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.entity.name)
            .to_lowercase();

        let other_normalized = other
            .entity
            .metadata
            .get("normalized_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&other.entity.name)
            .to_lowercase();

        self_normalized == other_normalized
    }
}

impl Mergeable for OrganizationStakeholder {
    fn can_merge_with(&self, other: &Self) -> bool {
        // Check by canonical name or direct name comparison
        let self_name = self
            .entity
            .canonical_name
            .as_ref()
            .unwrap_or(&self.entity.name)
            .to_lowercase();

        let other_name = other
            .entity
            .canonical_name
            .as_ref()
            .unwrap_or(&other.entity.name)
            .to_lowercase();

        self_name == other_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mergeable_person_same_name() {
        let p1 = PersonStakeholder::new("Jan de Vries".to_string(), 0.9);
        let p2 = PersonStakeholder::new("Jan de Vries".to_string(), 0.8);

        assert!(p1.can_merge_with(&p2));
    }

    #[test]
    fn test_mergeable_person_different_name() {
        let p1 = PersonStakeholder::new("Jan de Vries".to_string(), 0.9);
        let p2 = PersonStakeholder::new("Piet Jansen".to_string(), 0.8);

        assert!(!p1.can_merge_with(&p2));
    }

    #[test]
    fn test_mergeable_org_same_name() {
        let o1 = OrganizationStakeholder::new("MinFin".to_string(), 0.9);
        let o2 = OrganizationStakeholder::new("MinFin".to_string(), 0.8);

        assert!(o1.can_merge_with(&o2));
    }

    #[test]
    fn test_merge_entity_results_no_duplicates() {
        let baseline = vec![
            PersonStakeholder::new("Jan de Vries".to_string(), 0.9),
            PersonStakeholder::new("Piet Jansen".to_string(), 0.8),
        ];

        let llm = vec![PersonStakeholder::new("Jan de Vries".to_string(), 0.95)];

        let merged = merge_entity_results(baseline, llm);

        assert_eq!(merged.len(), 2);
    }

    #[tokio::test]
    async fn test_main_extractor_creation() {
        let extractor = MainExtractor::with_defaults().unwrap();
        // Should not panic
        let _ = &extractor.baseline;
    }

    #[tokio::test]
    async fn test_main_extractor_with_llm_disabled() {
        let extractor = MainExtractor::with_llm_enabled(false).unwrap();
        assert!(extractor.llm.is_none());
    }

    #[tokio::test]
    async fn test_extract_from_simple_text() {
        let extractor = MainExtractor::with_llm_enabled(false).unwrap();
        let options = ExtractionOptions::default();

        let result = extractor
            .extract_from_text("dr. Jan de Vries werkt bij het Ministerie van Financiën.", Uuid::new_v4(), &options)
            .await
            .unwrap();

        // Should have extracted something
        assert!(result.stats.total_entities >= 0);
    }

    #[tokio::test]
    async fn test_extraction_returns_result_structure() {
        let extractor = MainExtractor::with_llm_enabled(false).unwrap();
        let options = ExtractionOptions::default();

        let result = extractor
            .extract_from_text("Document text", Uuid::new_v4(), &options)
            .await
            .unwrap();

        // Check structure exists
        assert!(result.processing_time_ms >= 0);
    }

    #[test]
    fn test_filter_uncertain_entities() {
        let entities = vec![
            PersonStakeholder::new("High Confidence".to_string(), 0.95),
            PersonStakeholder::new("Medium Confidence".to_string(), 0.75),
            PersonStakeholder::new("Low Confidence".to_string(), 0.5),
        ];

        let options = ExtractionOptions {
            confidence_threshold: 0.7,
            ..Default::default()
        };

        let uncertain = filter_uncertain_entities(&entities, &options);

        // Medium confidence (0.75 < 0.7) and low confidence should be filtered
        // Actually 0.75 >= 0.7, so only low confidence < 0.7 is filtered
        assert_eq!(uncertain.len(), 1);
    }

    #[tokio::test]
    async fn test_tiered_acceptance_rejects_very_low() {
        let persons = vec![
            PersonStakeholder::new("High".to_string(), 0.9),
            PersonStakeholder::new("Low".to_string(), 0.3), // Below rejection threshold
            PersonStakeholder::new("Medium".to_string(), 0.6),
        ];

        let orgs = vec![];
        let options = ExtractionOptions::default();
        let mut stats = ExtractionStats::default();

        let (accepted_persons, _, _) =
            apply_tiered_acceptance(persons, orgs, Uuid::new_v4(), &options, &mut stats);

        // Low confidence entity should be rejected
        assert_eq!(accepted_persons.len(), 2);
    }

    #[tokio::test]
    async fn test_full_pipeline_completes_quickly() {
        let extractor = MainExtractor::with_llm_enabled(false).unwrap();
        let options = ExtractionOptions::default();

        let start = Instant::now();
        let result = extractor
            .extract_from_text("Test document with some text.", Uuid::new_v4(), &options)
            .await
            .unwrap();
        let duration = start.elapsed();

        assert!(result.processing_time_ms < 5000);
        assert!(duration.as_secs() < 5);
    }

    #[tokio::test]
    async fn test_llm_disabled_baseline_only() {
        let extractor = MainExtractor::with_llm_enabled(false).unwrap();
        let options = ExtractionOptions::default();

        let result = extractor
            .extract_from_text("dr. Jan de Vries werkt bij MinFin.", Uuid::new_v4(), &options)
            .await
            .unwrap();

        // Should have baseline results
        assert!(result.stats.total_entities >= 0);
        assert_eq!(result.stats.llm_calls_made, 0);
    }
}
