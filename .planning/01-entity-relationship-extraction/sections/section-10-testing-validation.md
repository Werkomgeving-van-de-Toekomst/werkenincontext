Now I have all the context needed. Let me generate the section content for `section-10-testing-validation.md`. Based on the manifest, this section should contain comprehensive test coverage including unit tests, integration tests, performance benchmarks, GDPR/privacy tests, and test fixtures with Dutch government document samples.</think># Section 10: Testing & Validation

## Overview

This section implements comprehensive testing and validation for the entity and relationship extraction system. It covers unit tests for each module, integration tests for the full pipeline, performance benchmarks, GDPR/privacy compliance tests, and realistic test fixtures.

**Dependencies:** All previous sections (00-09) must be implemented first.

**File Location:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/tests/`

---

## Test Structure

```
iou-ai/
├── tests/
│   ├── stakeholder_unit.rs          # Unit tests for each module
│   ├── stakeholder_integration.rs   # Integration tests
│   ├── stakeholder_performance.rs   # Performance benchmarks
│   ├── stakeholder_gdpr.rs          # Privacy and GDPR tests
│   └── fixtures/
│       ├── documents/                # Sample Dutch government documents
│       ├── expected/                 # Expected extraction results
│       └── mod.rs                    # Fixture helpers
└── src/stakeholder/
    └── tests/                        # Module-specific unit tests
        ├── baseline_tests.rs
        ├── llm_tests.rs
        ├── normalizer_tests.rs
        └── deduplicator_tests.rs
```

---

## Test Fixtures

### Document Samples

Create test fixture files at `/Users/marc/Projecten/iou-modern/crates/iou-ai/tests/fixtures/documents/`:

#### 1. Government Document with Ministries (`ministry_document.txt`)

A realistic Dutch government document containing:
- Multiple ministry references (MinFin, BZK, VWS)
- Person entities with titles (dr., minister, staatssecretaris)
- Organization abbreviations
- Standard Dutch government correspondence patterns

#### 2. Document with Citizen PII (`citizen_document.txt`)

A document containing:
- Citizen names and contact information (for PII testing)
- Government official references
- Mixed PII classifications

#### 3. Name Variations Document (`name_variations.txt`)

A document testing Dutch name handling:
- Names with prefixes (van, van der, de, ten)
- Title variations (dr. vs dr)
- Name case variations
- Phonetic similarities

#### 4. Organization Abbreviations Document (`org_abbreviations.txt`)

A document with:
- Common Dutch government abbreviations
- Full and short form variants
- Acronym usage

### Expected Results

Create expected result files at `/Users/marc/Projecten/iou-modern/crates/iou-ai/tests/fixtures/expected/`:

- `ministry_expected.json` - Expected entities and relationships for ministry_document.txt
- `citizen_expected.json` - Expected entities with PII classifications
- `name_variations_expected.json` - Expected normalized name results
- `org_abbreviations_expected.json` - Expected canonical organization names

### Fixture Helper Module

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/tests/fixtures/mod.rs`

```rust
/// Test fixture helpers for stakeholder extraction tests

use std::sync::Arc;
use iou_ai::stakeholder::{StakeholderExtractor, ExtractionOptions};
use iou_core::graphrag::GeneratedDocument;

/// Load a document fixture by name
pub fn load_document(name: &str) -> String {
    // Implementation reads from fixtures/documents/{name}.txt
}

/// Load expected extraction results
pub fn load_expected(name: &str) -> ExpectedResult {
    // Implementation reads from fixtures/expected/{name}.json
}

/// Create a GeneratedDocument from fixture text
pub fn create_test_document(text: &str) -> GeneratedDocument {
    // Implementation creates a valid GeneratedDocument for testing
}

/// Default extraction options for testing
pub fn test_extraction_options() -> ExtractionOptions {
    ExtractionOptions {
        use_llm: false,  // Disable LLM for fast unit tests
        confidence_threshold: 0.5,
        enable_normalization: false,  // Disable for isolated tests
        max_llm_calls: 0,
        max_cost_per_document: 0.0,
        llm_timeout: Duration::from_secs(1),
    }
}
```

---

## Unit Tests

### Test Coverage Requirements

Each module must achieve >80% code coverage.

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/tests/stakeholder_unit.rs`

#### Regex Pattern Tests

```rust
#[cfg(test)]
mod regex_tests {
    use iou_ai::stakeholder::baseline::BaselineExtractor;
    
    #[test]
    fn test_extract_title_dr() {
        // Verify "dr." is extracted as title
    }
    
    #[test]
    fn test_extract_title_prof() {
        // Verify "prof." is extracted as title
    }
    
    #[test]
    fn test_extract_role_minister() {
        // Verify "minister" is extracted as government role
    }
    
    #[test]
    fn test_extract_role_staatsecretaris() {
        // Verify "staatssecretaris" is extracted as government role
    }
    
    #[test]
    fn test_extract_department_minfin() {
        // Verify "MinFin" is extracted as department abbreviation
    }
    
    #[test]
    fn test_extract_department_bzk() {
        // Verify "BZK" is extracted as department abbreviation
    }
    
    #[test]
    fn test_no_false_positives() {
        // Verify patterns don't match non-government text
    }
}
```

#### NER Model Tests

```rust
#[cfg(test)]
mod ner_tests {
    use iou_ai::stakeholder::baseline::BaselineExtractor;
    
    #[tokio::test]
    async fn test_dutch_ner_model_loads() {
        // Verify rust-bert Dutch NER model loads without error
    }
    
    #[tokio::test]
    async fn test_ner_person_extraction() {
        // Verify NER returns Person entities from Dutch text
    }
    
    #[tokio::test]
    async fn test_ner_organization_extraction() {
        // Verify NER returns Organization entities from Dutch text
    }
}
```

#### Confidence Scoring Tests

```rust
#[cfg(test)]
mod confidence_tests {
    use iou_ai::stakeholder::confidence::ConfidenceScorer;
    
    #[test]
    fn test_confidence_in_valid_range() {
        // Verify confidence scores are between 0.0 and 1.0
    }
    
    #[test]
    fn test_high_confidence_threshold() {
        // Verify >= 0.9 is classified as high
    }
    
    #[test]
    fn test_medium_confidence_threshold() {
        // Verify 0.7-0.9 is classified as medium
    }
    
    #[test]
    fn test_low_confidence_threshold() {
        // Verify 0.5-0.7 is classified as low
    }
    
    #[test]
    fn test_reject_confidence_threshold() {
        // Verify < 0.5 is rejected
    }
}
```

#### Relationship Pattern Tests

```rust
#[cfg(test)]
mod relationship_tests {
    use iou_ai::stakeholder::baseline::RelationshipExtractor;
    
    #[test]
    fn test_worksfor_pattern_minister_van() {
        // Verify "X, minister van Y" creates WorksFor relationship
    }
    
    #[test]
    fn test_worksfor_pattern_directeur() {
        // Verify "dr. X, directeur Z" creates WorksFor with role
    }
}
```

#### Mention Type Detection Tests

```rust
#[cfg(test)]
mod mention_tests {
    use iou_ai::stakeholder::baseline::MentionDetector;
    
    #[test]
    fn test_detect_author_from_header() {
        // Verify MentionType::Author detected from header patterns
    }
    
    #[test]
    fn test_detect_recipient_from_geachte() {
        // Verify MentionType::Recipient detected from "Geachte:" patterns
    }
    
    #[test]
    fn test_defaults_to_referenced() {
        // Verify MentionType::Referenced is default
    }
}
```

#### Name Normalization Tests

```rust
#[cfg(test)]
mod normalization_tests {
    use iou_ai::stakeholder::normalizer::DutchNameNormalizer;
    
    #[test]
    fn test_lowercase_prefix_van() {
        // Verify "van" prefix is lowercased
    }
    
    #[test]
    fn test_lowercase_prefix_van_der() {
        // Verify "van der" prefix is lowercased
    }
    
    #[test]
    fn test_lowercase_prefix_de() {
        // Verify "de" prefix is lowercased
    }
    
    #[test]
    fn test_lowercase_prefix_ten() {
        // Verify "ten" prefix is lowercased
    }
    
    #[test]
    fn test_case_insensitive_matching() {
        // Verify "Jan de Vries" and "jan de vries" match
    }
}
```

#### Jaro-Winkler Similarity Tests

```rust
#[cfg(test)]
mod similarity_tests {
    use iou_ai::stakeholder::deduplicator::JaroWinkler;
    
    #[test]
    fn test_identical_strings() {
        // Verify similarity returns 1.0 for identical strings
    }
    
    #[test]
    fn test_similar_strings() {
        // Verify similarity > 0.9 for similar strings
    }
    
    #[test]
    fn test_dissimilar_strings() {
        // Verify similarity < 0.5 for dissimilar strings
    }
    
    #[test]
    fn test_symmetric() {
        // Verify sim(a,b) == sim(b,a)
    }
    
    #[test]
    fn test_reflexive() {
        // Verify sim(a,a) == 1.0
    }
}
```

#### Entity Merge Tests

```rust
#[cfg(test)]
mod merge_tests {
    use iou_ai::stakeholder::deduplicator::EntityMerger;
    
    #[tokio::test]
    async fn test_merge_creates_canonical_uuid() {
        // Verify merge creates new canonical UUID
    }
    
    #[tokio::test]
    async fn test_merge_stores_old_uuids_as_aliases() {
        // Verify old UUIDs stored in metadata
    }
    
    #[tokio::test]
    async fn test_merge_updates_mention_relationships() {
        // Verify MentionRelationships updated to canonical entity
    }
    
    #[tokio::test]
    async fn test_merge_log_audit_trail() {
        // Verify merge log contains audit trail
    }
}
```

#### Rijksoverheid API Tests

```rust
#[cfg(test)]
mod rijksoverheid_tests {
    use iou_ai::stakeholder::rijksoverheid::RijksoverheidClient;
    use wiremock::{MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};
    
    #[tokio::test]
    async fn test_get_canonical_name_minfin() {
        // Verify returns "Ministerie van Financiën" for "MinFin"
    }
    
    #[tokio::test]
    async fn test_get_canonical_name_full_name() {
        // Verify returns same name for full input
    }
    
    #[tokio::test]
    async fn test_get_canonical_name_unknown() {
        // Verify returns None for unknown organization
    }
    
    #[tokio::test]
    async fn test_cache_returns_without_api_call() {
        // Verify cache returns cached result
    }
    
    #[tokio::test]
    async fn test_cache_expires_after_ttl() {
        // Verify cache respects TTL
    }
    
    #[tokio::test]
    async fn test_handles_timeout_gracefully() {
        // Verify timeout doesn't crash
    }
}
```

#### LLM Extractor Tests

```rust
#[cfg(test)]
mod llm_tests {
    use iou_ai::stakeholder::llm_extractor::ClaudeExtractor;
    
    #[tokio::test]
    async fn test_returns_structured_entities() {
        // Verify Claude API returns structured Entity list
    }
    
    #[tokio::test]
    async fn test_tool_calling_produces_valid_json() {
        // Verify tool calling produces valid JSON
    }
    
    #[tokio::test]
    async fn test_confidence_from_logprobs() {
        // Verify confidence calculated from logprobs is in [0.0, 1.0]
    }
    
    #[tokio::test]
    async fn test_timeout_prevents_hanging() {
        // Verify timeout prevents hanging on API failure
    }
    
    #[tokio::test]
    async fn test_cost_tracking() {
        // Verify cost tracking returns token count
    }
    
    #[tokio::test]
    async fn test_max_llm_calls_limit() {
        // Verify limit prevents runaway API usage
    }
    
    #[tokio::test]
    async fn test_max_cost_limit() {
        // Verify cost limit prevents overruns
    }
}
```

#### Error Path Tests

```rust
#[cfg(test)]
mod error_tests {
    use iou_ai::stakeholder::{ExtractionError, ExtractionErrorKind};
    
    #[test]
    fn test_api_timeout_error() {
        // Verify timeout produces appropriate error
    }
    
    #[test]
    fn test_authentication_error() {
        // Verify auth failure returns appropriate error
    }
    
    #[test]
    fn test_invalid_document_error() {
        // Verify invalid document handled
    }
    
    #[test]
    fn test_parse_error() {
        // Verify parse errors handled
    }
}
```

---

## Integration Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/tests/stakeholder_integration.rs`

### Full Pipeline Tests

```rust
#[cfg(test)]
mod integration_tests {
    use iou_ai::stakeholder::MainExtractor;
    use iou_ai::agents::AgentPipeline;
    use crate::fixtures::{load_document, create_test_document, test_extraction_options};
    
    #[tokio::test]
    async fn test_full_extraction_pipeline() {
        // Test complete extraction with real Dutch government document
        // Verify:
        // - All entity types returned
        // - Mention relationships created
        // - Stats are correct
    }
    
    #[tokio::test]
    async fn test_baseline_before_llm() {
        // Verify baseline extraction runs before LLM
    }
    
    #[tokio::test]
    async fn test_llm_only_for_low_confidence() {
        // Verify LLM only called when confidence < 0.7
    }
    
    #[tokio::test]
    async fn test_llm_only_for_missing_context() {
        // Verify LLM called when title/role missing
    }
    
    #[tokio::test]
    async fn test_results_merged_correctly() {
        // Verify baseline and LLM results merged
    }
    
    #[tokio::test]
    async fn test_partial_failure_handling() {
        // Verify LLM failure doesn't lose baseline results
    }
    
    #[tokio::test]
    async fn test_processing_time_tracked() {
        // Verify processing time is returned
    }
}
```

### Document Pipeline Integration Tests

```rust
#[tokio::test]
async fn test_pipeline_calls_extractor_after_content_agent() {
    // Verify StakeholderExtractor called after Content Agent
}

#[tokio::test]
async fn test_entities_added_to_knowledge_graph() {
    // Verify extracted entities appear in KnowledgeGraph
}

#[tokio::test]
async fn test_mention_relationships_created() {
    // Verify MentionRelationships created for each entity
}

#[tokio::test]
async fn test_extraction_stats_logged() {
    // Verify stats logged with correct counts
}

#[tokio::test]
async fn test_pipeline_continues_on_extraction_failure() {
    // Verify non-blocking failure handling
}

#[tokio::test]
async fn test_generated_document_text_accessible() {
    // Verify GeneratedDocument text is accessible
}

#[tokio::test]
async fn test_pipeline_config_controls_execution() {
    // Verify enable_stakeholder_extraction controls execution
}

#[tokio::test]
async fn test_pipeline_options_passed_to_extractor() {
    // Verify stakeholder_options passed correctly
}
```

### Cross-Document Deduplication Tests

```rust
#[tokio::test]
async fn test_entity_deduplication_across_documents() {
    // Verify entities from different documents are deduplicated
}
```

### KnowledgeGraph Extension Tests

```rust
#[tokio::test]
async fn test_get_document_stakeholders() {
    // Verify returns all entities for document
}

#[tokio::test]
async fn test_get_document_stakeholders_empty() {
    // Verify returns empty vec for document with no entities
}

#[tokio::test]
async fn test_get_stakeholder_documents() {
    // Verify returns all document IDs for entity
}

#[tokio::test]
async fn test_get_stakeholder_documents_unknown_entity() {
    // Verify returns empty vec for unknown entity
}

#[tokio::test]
async fn test_get_stakeholder_influence() {
    // Verify returns mention_count, document_count, pagerank_score
}

#[tokio::test]
async fn test_find_stakeholders_by_name() {
    // Verify returns matches above threshold
}

#[tokio::test]
async fn test_find_stakeholders_by_name_no_matches() {
    // Verify returns empty vec when no matches
}
```

---

## Performance Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/tests/stakeholder_performance.rs`

```rust
#[cfg(test)]
mod performance_tests {
    use iou_ai::stakeholder::MainExtractor;
    use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
    use crate::fixtures::{load_document, test_extraction_options};
    
    fn bench_baseline_extraction(c: &mut Criterion) {
        let mut group = c.benchmark_group("baseline_extraction");
        
        for size in [1000, 5000, 10000].iter() {
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
                let text = generate_dutch_text(size);
                b.iter(|| {
                    // Benchmark baseline extraction
                    // Target: <500ms p50, <750ms p95
                });
            });
        }
        group.finish();
    }
    
    fn bench_full_pipeline(c: &mut Criterion) {
        let mut group = c.benchmark_group("full_pipeline");
        
        for size in [1000, 5000, 10000].iter() {
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
                let text = generate_dutch_text(size);
                b.iter(|| {
                    // Benchmark full extraction pipeline
                    // Target: <3s p50, <5s p95
                });
            });
        }
        group.finish();
    }
    
    fn bench_model_load_time(c: &mut Criterion) {
        c.bench_function("model_load", |b| {
            b.iter(|| {
                // Benchmark NER model loading
                // Document the cold start time
            });
        });
    }
    
    criterion_group!(benches, bench_baseline_extraction, bench_full_pipeline, bench_model_load_time);
    criterion_main!(benches);
}
```

### Performance Targets

| Metric | Target (p50) | Target (p95) |
|--------|-------------|-------------|
| Baseline extraction | <500ms | <750ms |
| Full pipeline (no LLM) | <500ms | <750ms |
| Full pipeline (with LLM) | <3s | <5s |
| Model load (cold) | <2s | <3s |

---

## GDPR and Privacy Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/tests/stakeholder_gdpr.rs`

```rust
#[cfg(test)]
mod gdpr_tests {
    use iou_core::graphrag::PiiClassification;
    use iou_ai::stakeholder::MainExtractor;
    
    #[tokio::test]
    async fn test_citizen_entities_flagged() {
        // Verify citizen entities get PiiClassification::Citizen
    }
    
    #[tokio::test]
    async fn test_official_entities_flagged() {
        // Verify official entities get PiiClassification::Official
    }
    
    #[tokio::test]
    async fn test_api_redacts_citizen_pii() {
        // Verify API redacts email, phone for citizens
    }
    
    #[tokio::test]
    async fn test_api_does_not_redact_official_pii() {
        // Verify official PII is NOT redacted
    }
    
    #[tokio::test]
    async fn test_pii_access_logged() {
        // Verify all citizen PII access is logged
    }
    
    #[tokio::test]
    async fn test_rls_policy_restricts_citizen_access() {
        // Verify RLS policy restricts citizen entity access
    }
    
    #[tokio::test]
    async fn test_retention_policy_citizen() {
        // Verify citizen entities marked for 2-year retention
    }
    
    #[tokio::test]
    async fn test_retention_policy_official() {
        // Verify official entities marked for 7-year retention
    }
}
```

---

## Property-Based Tests (proptest)

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/tests/stakeholder_proptest.rs`

```rust
#[cfg(test)]
mod proptest_tests {
    use proptest::prelude::*;
    use iou_ai::stakeholder::normalizer::DutchNameNormalizer;
    use iou_ai::stakeholder::deduplicator::JaroWinkler;
    
    proptest! {
        #[test]
        fn test_normalization_is_idempotent(name in "[a-zA-Z]{1,50}") {
            // normalize(normalize(x)) == normalize(x)
        }
        
        #[test]
        fn test_confidence_always_in_range(score in 0.0f32..1.0) {
            // Verify confidence always in [0.0, 1.0]
        }
        
        #[test]
        fn test_similarity_symmetric(a in "[a-z]{1,20}", b in "[a-z]{1,20}") {
            // sim(a,b) == sim(b,a)
        }
        
        #[test]
        fn test_similarity_reflexive(s in "[a-z]{1,20}") {
            // sim(s,s) == 1.0
        }
    }
}
```

---

## API Endpoint Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/tests/stakeholder_api_tests.rs`

```rust
#[cfg(test)]
mod api_tests {
    use reqwest::Client;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_get_stakeholder_by_id() {
        // GET /stakeholders/:id returns stakeholder for valid ID
        // Returns 404 for invalid ID
    }
    
    #[tokio::test]
    async fn test_get_stakeholder_documents() {
        // GET /stakeholders/:id/documents returns all documents
    }
    
    #[tokio::test]
    async fn test_get_document_stakeholders() {
        // GET /documents/:id/stakeholders returns all stakeholders
    }
    
    #[tokio::test]
    async fn test_search_stakeholders() {
        // GET /stakeholders/search?q= returns results for partial match
        // Returns empty for no match
    }
    
    #[tokio::test]
    async fn test_pagination() {
        // Verify correct page size
        // Verify next/prev page links
    }
    
    #[tokio::test]
    async fn test_unauthorized_returns_401() {
        // Verify authentication required
    }
}
```

---

## Implementation Checklist

1. **Create test fixture structure**
   - [ ] Create `tests/fixtures/` directory
   - [ ] Create `documents/` subdirectory with 4 sample documents
   - [ ] Create `expected/` subdirectory with JSON expected results
   - [ ] Implement `fixtures/mod.rs` helper module

2. **Implement unit tests**
   - [ ] Regex pattern tests (positive and negative cases)
   - [ ] NER model tests
   - [ ] Confidence scoring tests
   - [ ] Relationship pattern tests
   - [ ] Mention type detection tests
   - [ ] Name normalization tests
   - [ ] Jaro-Winkler similarity tests
   - [ ] Entity merge tests
   - [ ] Rijksoverheid API tests (with mocked responses)
   - [ ] LLM extractor tests (with mocked responses)
   - [ ] Error path tests for all modules

3. **Implement integration tests**
   - [ ] Full extraction pipeline test
   - [ ] Baseline before LLM verification
   - [ ] Conditional LLM calling tests
   - [ ] Result merging tests
   - [ ] Partial failure handling tests
   - [ ] Document pipeline integration tests
   - [ ] KnowledgeGraph extension tests
   - [ ] Cross-document deduplication tests

4. **Implement performance tests**
   - [ ] Baseline extraction benchmarks
   - [ ] Full pipeline benchmarks
   - [ ] Model load time measurement
   - [ ] Document performance targets

5. **Implement GDPR/privacy tests**
   - [ ] Citizen/official classification tests
   - [ ] PII redaction tests
   - [ ] Access logging tests
   - [ ] RLS policy tests
   - [ ] Retention policy tests

6. **Implement property-based tests**
   - [ ] Normalization idempotence
   - [ ] Confidence range validation
   - [ ] Similarity properties

7. **Implement API endpoint tests**
   - [ ] GET /stakeholders/:id
   - [ ] GET /stakeholders/:id/documents
   - [ ] GET /documents/:id/stakeholders
   - [ ] GET /stakeholders/search
   - [ ] Pagination tests
   - [ ] Authentication tests

8. **Verify coverage**
   - [ ] Run `cargo tarpaulin` or similar
   - [ ] Confirm >80% coverage for all stakeholder modules
   - [ ] Address any coverage gaps

---

## Success Criteria

- All tests pass (`cargo test`)
- >80% code coverage for stakeholder modules
- Performance benchmarks meet targets (p50 and p95)
- GDPR/privacy tests verify PII handling
- Test fixtures represent realistic Dutch government documents
- Integration tests verify end-to-end functionality

---

## Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --test stakeholder_unit

# Run integration tests
cargo test --test stakeholder_integration

# Run performance benchmarks
cargo test --test stakeholder_performance -- --nocapture

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage/

# Run GDPR tests
cargo test --test stakeholder_gdpr
```