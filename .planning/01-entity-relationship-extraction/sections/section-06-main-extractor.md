Now I have all the context. Let me locate the specific content for section-06-main-extractor from both plan documents to extract the relevant information.

From the index.md, section-06-main-extractor is described as:
> Implements the full extraction pipeline: coordinates baseline, LLM, normalization, and deduplication stages; applies tiered acceptance logic; handles partial failures; tracks processing time.

Dependencies: Sections 02, 04, 05
Blocks: Section 07

From the plan (Section 06: Main Extractor Implementation), I need to extract the implementation details, and from the TDD plan, I need to extract the test stubs.

Let me now generate the complete, self-contained section content:

# Section 06: Main Extractor Implementation

## Overview

This section implements the core `StakeholderExtractor` that orchestrates the full entity extraction pipeline. The extractor coordinates multiple stages - baseline extraction, LLM enhancement, normalization, and deduplication - while applying tiered acceptance logic and maintaining resilience through partial failure handling.

**File:** `crates/iou-ai/src/stakeholder/extractor.rs`

## Dependencies

This section depends on the following completed sections:

- **Section 01 (Foundation & Types):** Core data structures including `PersonStakeholder`, `OrganizationStakeholder`, `ExtractionResult`, `ExtractionOptions`, and error types
- **Section 02 (Baseline Extraction):** Fast regex + rust-bert extraction implementation
- **Section 04 (LLM Extractor):** Claude API integration for enhanced extraction
- **Section 05 (Normalization & Deduplication):** Entity normalization and deduplication logic

## Architecture

The main extractor follows a hybrid pipeline architecture:

```
Document Input
     │
     ▼
┌─────────────────────────────────────────┐
│ 1. Baseline Extraction (<500ms target)   │
│    - Regex patterns for Dutch entities   │
│    - rust-bert NER                       │
└─────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────┐
│ 2. Uncertainty Filter                   │
│    - confidence < 0.7 → LLM              │
│    - missing context → LLM               │
└─────────────────────────────────────────┘
     │              │
     │              ▼
     │     ┌───────────────────────┐
     │     │ 3. LLM Enhancement    │
     │     │    (optional)         │
     │     └───────────────────────┘
     │              │
     └──────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│ 4. Normalization                        │
│    - Rijksoverheid API lookup           │
│    - Canonical name resolution          │
└─────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────┐
│ 5. Deduplication                        │
│    - Jaro-Winkler similarity            │
│    - Connected components clustering    │
└─────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────┐
│ 6. Tiered Acceptance                    │
│    - High (≥0.9): Auto-accept           │
│    - Medium (0.7-0.9): Flag             │
│    - Low (0.5-0.7): Manual review       │
│    - Very low (<0.5): Reject            │
└─────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────┐
│ 7. Result Assembly                      │
│    - Merge persons/organizations        │
│    - Create mention relationships       │
│    - Track stats                        │
└─────────────────────────────────────────┘
```

## Implementation

### Main Extractor Structure

**File:** `crates/iou-ai/src/stakeholder/extractor.rs`

```rust
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

use iou_core::graphrag::{Entity, EntityType};
use crate::stakeholder::{
    PersonStakeholder, OrganizationStakeholder,
    ExtractionResult, ExtractionOptions, ExtractionError,
    ExtractionStats, VerificationStatus,
};
use crate::stakeholder::baseline::BaselineExtractor;
use crate::stakeholder::llm_extractor::ClaudeExtractor;
use crate::stakeholder::normalizer::EntityNormalizer;
use crate::stakeholder::deduplicator::EntityDeduplicator;

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
    pub fn with_defaults(api_key: Option<String>) -> Result<Self, ExtractionError> {
        let baseline = Arc::new(BaselineExtractor::new()?);
        
        let llm = api_key.map(|key| {
            Arc::new(ClaudeExtractor::new(key))
        });
        
        let normalizer = Arc::new(EntityNormalizer::new()?);
        let deduplicator = Arc::new(EntityDeduplicator::new());
        
        Ok(Self::new(baseline, llm, normalizer, deduplicator, 3))
    }
}

#[async_trait]
impl StakeholderExtractor for MainExtractor {
    /// Extract entities from a document using the full pipeline
    async fn extract(
        &self,
        document: &Document,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, ExtractionError> {
        let start = Instant::now();
        let mut stats = ExtractionStats::default();
        
        // Stage 1: Baseline extraction (always runs)
        let baseline_result = self
            .baseline
            .extract(document, options)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Baseline extraction failed: {}", e);
                BaselineResult::empty()
            });
        
        stats.total_entities = baseline_result.persons.len() + baseline_result.organizations.len();
        
        // Stage 2: Filter uncertain entities for LLM enhancement
        let uncertain_persons = filter_uncertain_entities(&baseline_result.persons, options);
        let uncertain_orgs = filter_uncertain_entities(&baseline_result.organizations, options);
        
        // Stage 3: LLM enhancement (conditional)
        let mut llm_persons = Vec::new();
        let mut llm_orgs = Vec::new();
        
        if options.use_llm && self.llm.is_some() && (!uncertain_persons.is_empty() || !uncertain_orgs.is_empty()) {
            let _permit = self.llm_semaphore.acquire().await.unwrap();
            
            if stats.llm_calls_made < options.max_llm_calls {
                let llm_extractor = self.llm.as_ref().unwrap();
                
                // Enhance uncertain persons
                for person in &uncertain_persons {
                    match llm_extractor.enhance_person(person, document).await {
                        Ok(enhanced) => {
                            llm_persons.push(enhanced);
                            stats.llm_calls_made += 1;
                        }
                        Err(e) => {
                            tracing::debug!("LLM enhancement failed for person: {}", e);
                        }
                    }
                    
                    if stats.llm_calls_made >= options.max_llm_calls {
                        break;
                    }
                }
                
                // Enhance uncertain organizations
                for org in &uncertain_orgs {
                    match llm_extractor.enhance_organization(org, document).await {
                        Ok(enhanced) => {
                            llm_orgs.push(enhanced);
                            stats.llm_calls_made += 1;
                        }
                        Err(e) => {
                            tracing::debug!("LLM enhancement failed for organization: {}", e);
                        }
                    }
                    
                    if stats.llm_calls_made >= options.max_llm_calls {
                        break;
                    }
                }
            }
        }
        
        // Stage 4: Merge baseline and LLM results
        let mut all_persons = merge_entity_results(baseline_result.persons, llm_persons);
        let mut all_orgs = merge_entity_results(baseline_result.organizations, llm_orgs);
        
        // Stage 5: Normalization
        if options.enable_normalization {
            all_persons = self.normalizer.normalize_persons(all_persons).await.unwrap_or(all_persons);
            all_orgs = self.normalizer.normalize_organizations(all_orgs).await.unwrap_or(all_orgs);
            stats.api_calls_made = self.normalizer.api_call_count();
        }
        
        // Stage 6: Deduplication
        all_persons = self.deduplicator.deduplicate_persons(all_persons)?;
        all_orgs = self.deduplicator.deduplicate_organizations(all_orgs)?;
        
        // Stage 7: Tiered acceptance and mention creation
        let (accepted_persons, mentions) = apply_tiered_acceptance(
            all_persons,
            all_orgs,
            document.id(),
            options,
            &mut stats,
        );
        
        let processing_time_ms = start.elapsed().as_millis() as u64;
        
        Ok(ExtractionResult {
            persons: accepted_persons.persons,
            organizations: accepted_persons.organizations,
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
        self.normalizer.normalize_entities(entities).await
    }

    /// Deduplicate entities by similarity
    async fn deduplicate_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, DeduplicationError> {
        self.deduplicator.deduplicate_entities(entities).await
    }
}

/// Filter entities that need LLM enhancement based on confidence and context
fn filter_uncertain_entities<T>(entities: &[T], options: &ExtractionOptions) -> Vec<&T>
where
    T: ConfidenceScore + HasContext,
{
    entities
        .iter()
        .filter(|e| {
            // Low confidence
            if e.confidence() < 0.7 {
                return true;
            }
            // Missing important context
            if e.missing_context() {
                return true;
            }
            // Below threshold
            if e.confidence() < options.confidence_threshold {
                return true;
            }
            false
        })
        .collect()
}

/// Merge baseline and LLM results, preferring LLM for enhanced entities
fn merge_entity_results<T>(
    baseline: Vec<T>,
    llm_enhanced: Vec<T>,
) -> Vec<T>
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
    options: &ExtractionOptions,
    stats: &mut ExtractionStats,
) -> (AcceptedEntities, Vec<MentionRelationship>) {
    let mut accepted_persons = Vec::new();
    let mut accepted_orgs = Vec::new();
    let mut mentions = Vec::new();
    
    for person in persons {
        let confidence = person.confidence();
        let verification = match confidence {
            c if c >= 0.9 => VerificationStatus::AutoAccepted,
            c if c >= 0.7 => VerificationStatus::AcceptedFlagged,
            c if c >= 0.5 => VerificationStatus::PendingReview,
            _ => VerificationStatus::Rejected,
        };
        
        match verification {
            VerificationStatus::Rejected => {
                stats.low_confidence += 1;
                continue;
            }
            VerificationStatus::PendingReview => {
                stats.low_confidence += 1;
                // Still include but flagged
            }
            VerificationStatus::AcceptedFlagged => {
                stats.medium_confidence += 1;
            }
            VerificationStatus::AutoAccepted => {
                stats.high_confidence += 1;
            }
        }
        
        let mention = create_mention(&person, document_id, confidence);
        mentions.push(mention);
        accepted_persons.push(person);
    }
    
    for org in organizations {
        let confidence = org.confidence();
        let verification = match confidence {
            c if c >= 0.9 => VerificationStatus::AutoAccepted,
            c if c >= 0.7 => VerificationStatus::AcceptedFlagged,
            c if c >= 0.5 => VerificationStatus::PendingReview,
            _ => VerificationStatus::Rejected,
        };
        
        match verification {
            VerificationStatus::Rejected => {
                stats.low_confidence += 1;
                continue;
            }
            VerificationStatus::PendingReview => {
                stats.low_confidence += 1;
            }
            VerificationStatus::AcceptedFlagged => {
                stats.medium_confidence += 1;
            }
            VerificationStatus::AutoAccepted => {
                stats.high_confidence += 1;
            }
        }
        
        let mention = create_mention(&org, document_id, confidence);
        mentions.push(mention);
        accepted_orgs.push(org);
    }
    
    (
        AcceptedEntities {
            persons: accepted_persons,
            organizations: accepted_orgs,
        },
        mentions,
    )
}
```

## Tests

### Test Stubs

Write the following tests in `crates/iou-ai/src/stakeholder/extractor_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::stakeholder::test_fixtures::{mock_document, mock_options};

    #[tokio::test]
    async fn test_full_pipeline_returns_all_entity_types() {
        let extractor = MainExtractor::with_defaults(None).unwrap();
        let document = mock_document();
        let options = mock_options();
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        assert!(!result.persons.is_empty() || !result.organizations.is_empty());
        assert!(!result.mentions.is_empty());
    }

    #[tokio::test]
    async fn test_baseline_extraction_runs_before_llm() {
        let extractor = MainExtractor::with_defaults(None).unwrap();
        let document = mock_document();
        let mut options = mock_options();
        options.use_llm = false;
        
        let result_no_llm = extractor.extract(&document, &options).await.unwrap();
        
        options.use_llm = true;
        let result_with_llm = extractor.extract(&document, &options).await.unwrap();
        
        // Baseline should produce results regardless of LLM setting
        assert!(!result_no_llm.persons.is_empty() || !result_no_llm.organizations.is_empty());
    }

    #[tokio::test]
    async fn test_llm_only_called_for_low_confidence_entities() {
        let extractor = MainExtractor::with_defaults(Some("test-key".to_string())).unwrap();
        let document = mock_document_with_low_confidence_entities();
        let options = mock_options();
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        // LLM should have been called
        assert!(result.stats.llm_calls_made > 0);
    }

    #[tokio::test]
    async fn test_llm_only_called_when_missing_context() {
        let extractor = MainExtractor::with_defaults(Some("test-key".to_string())).unwrap();
        let document = mock_document_without_context();
        let options = mock_options();
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        // LLM should have been called to fill in missing context
        assert!(result.stats.llm_calls_made > 0);
    }

    #[tokio::test]
    async fn test_results_from_baseline_and_llm_merged_correctly() {
        let extractor = MainExtractor::with_defaults(Some("test-key".to_string())).unwrap();
        let document = mock_document();
        let options = mock_options();
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        // Results should not have duplicates
        assert_eq!(result.persons.len(), unique_person_count(&result.persons));
    }

    #[tokio::test]
    async fn test_tiered_acceptance_accepts_high_confidence() {
        let extractor = MainExtractor::with_defaults(None).unwrap();
        let document = mock_document_with_high_confidence();
        let options = mock_options();
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        assert!(result.stats.high_confidence > 0);
    }

    #[tokio::test]
    async fn test_tiered_acceptance_flags_medium_confidence() {
        let extractor = MainExtractor::with_defaults(None).unwrap();
        let document = mock_document_with_medium_confidence();
        let options = mock_options();
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        assert!(result.stats.medium_confidence > 0);
    }

    #[tokio::test]
    async fn test_tiered_acceptance_rejects_low_confidence() {
        let extractor = MainExtractor::with_defaults(None).unwrap();
        let document = mock_document_with_low_confidence();
        let options = mock_options();
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        // Low confidence entities should be rejected
        let accepted_count = result.stats.high_confidence + result.stats.medium_confidence;
        assert!(accepted_count < result.stats.total_entities);
    }

    #[tokio::test]
    async fn test_processing_time_is_tracked() {
        let extractor = MainExtractor::with_defaults(None).unwrap();
        let document = mock_document();
        let options = mock_options();
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        assert!(result.processing_time_ms > 0);
    }

    #[tokio::test]
    async fn test_llm_failure_doesnt_lose_baseline_results() {
        let extractor = MainExtractor::with_defaults(Some("invalid-key".to_string())).unwrap();
        let document = mock_document();
        let mut options = mock_options();
        options.use_llm = true;
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        // Should still have baseline results
        assert!(!result.persons.is_empty() || !result.organizations.is_empty());
    }

    #[tokio::test]
    async fn test_extraction_stats_counts_correctly() {
        let extractor = MainExtractor::with_defaults(None).unwrap();
        let document = mock_document_mixed_confidence();
        let options = mock_options();
        
        let result = extractor.extract(&document, &options).await.unwrap();
        
        assert_eq!(
            result.stats.total_entities,
            result.stats.high_confidence + result.stats.medium_confidence + result.stats.low_confidence
        );
    }

    #[tokio::test]
    async fn test_full_pipeline_completes_quickly() {
        let extractor = MainExtractor::with_defaults(None).unwrap();
        let document = typical_government_document();
        let options = mock_options();
        
        let start = Instant::now();
        let result = extractor.extract(&document, &options).await.unwrap();
        let duration = start.elapsed();
        
        assert!(result.processing_time_ms < 5000);
        assert!(duration.as_secs() < 5);
    }
}
```

## Success Criteria

- End-to-end extraction produces all entity types (persons and organizations)
- Processing time <5 seconds for typical Dutch government documents
- LLM is called only for uncertain cases (confidence <0.7 or missing context)
- Failures in LLM stage do not lose baseline results
- Tiered acceptance correctly categorizes entities by confidence
- Extraction stats accurately count high/medium/low confidence entities

## Integration Points

This section produces the main `StakeholderExtractor` implementation that will be used by:

- **Section 07 (Pipeline Integration):** The `AgentPipeline` will call this extractor after document generation
- The extractor internally uses components from sections 02, 04, and 05

## Error Handling

The main extractor implements resilient error handling:

| Stage | Failure Behavior | Fallback |
|-------|------------------|----------|
| Baseline extraction | Log warning, continue with empty result | Skip to next document |
| LLM enhancement | Log debug, use baseline results | Baseline entities only |
| Normalization | Use original entities | Non-canonical names |
| Deduplication | Return error | Fail fast (data integrity) |

## Configuration

The extractor behavior is controlled via `ExtractionOptions`:

```rust
pub struct ExtractionOptions {
    /// Whether to use LLM for uncertain cases
    pub use_llm: bool,
    
    /// Minimum confidence threshold for acceptance
    pub confidence_threshold: f32,
    
    /// Whether to normalize via external API
    pub enable_normalization: bool,
    
    /// Maximum LLM calls per document (cost control)
    pub max_llm_calls: usize,
    
    /// Maximum cost per document in USD
    pub max_cost_per_document: f32,
    
    /// Timeout for LLM API calls
    pub llm_timeout: Duration,
}
```

Default values provide a balanced configuration suitable for production use.