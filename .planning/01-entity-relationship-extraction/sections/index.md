<!-- PROJECT_CONFIG
runtime: rust-cargo
test_command: cargo test
END_PROJECT_CONFIG -->

<!-- SECTION_MANIFEST
section-00-feasibility-spike
section-01-foundation-types
section-02-baseline-extraction
section-03-rijksoverheid-api
section-04-llm-extractor
section-05-normalization-deduplication
section-06-main-extractor
section-07-pipeline-integration
section-08-knowledgegraph-extensions
section-09-api-endpoints
section-10-testing-validation
END_MANIFEST -->

# Implementation Sections Index

## Dependency Graph

| Section | Depends On | Blocks | Parallelizable |
|---------|------------|--------|----------------|
| section-00-feasibility-spike | - | 03 | Yes |
| section-01-foundation-types | - | 02, 04, 05 | Yes |
| section-02-baseline-extraction | 01 | 06 | Yes |
| section-03-rijksoverheid-api | 00 | 05 | Yes |
| section-04-llm-extractor | 01 | 06 | Yes |
| section-05-normalization-deduplication | 01, 03 | 06 | Yes |
| section-06-main-extractor | 02, 04, 05 | 07 | No |
| section-07-pipeline-integration | 01, 06 | 08 | No |
| section-08-knowledgegraph-extensions | 01, 07 | 09 | No |
| section-09-api-endpoints | 01, 08 | 10 | No |
| section-10-testing-validation | All | - | No |

## Execution Order

**Phase 1: Foundation & Validation (Parallel)**
1. section-00-feasibility-spike (no dependencies - validates external APIs)
2. section-01-foundation-types (no dependencies - core data structures)

**Phase 2: Extraction Components (Parallel after Phase 1)**
3. section-02-baseline-extraction (after 01)
4. section-03-rijksoverheid-api (after 00)
5. section-04-llm-extractor (after 01)
6. section-05-normalization-deduplication (after 01, 03)

**Phase 3: Integration (Sequential)**
7. section-06-main-extractor (after 02, 04, 05)
8. section-07-pipeline-integration (after 01, 06)
9. section-08-knowledgegraph-extensions (after 01, 07)

**Phase 4: Exposure & Validation (Sequential)**
10. section-09-api-endpoints (after 01, 08)
11. section-10-testing-validation (after all)

## Section Summaries

### section-00-feasibility-spike
Validates external dependencies before main implementation: Rijksoverheid API capabilities, cost estimation model, local fallback dictionary, and GeneratedDocument structure verification.

### section-01-foundation-types
Defines core data structures and public API: PersonStakeholder and OrganizationStakeholder convenience wrappers, StakeholderExtractor trait, ExtractionResult types, and Dutch name normalization module.

### section-02-baseline-extraction
Implements fast regex + rust-bert extraction: Dutch title/role patterns, NER model loading, confidence scoring, basic pattern-based relationship extraction, and mention type detection.

### section-03-rijksoverheid-api
Implements Dutch government organization lookup: API client with canonical name resolution, in-memory caching with TTL, and graceful degradation with local fallback.

### section-04-llm-extractor
Implements Claude API integration: tool calling for structured entity extraction, confidence scoring from logprobs, timeout handling, cost tracking, and budget controls.

### section-05-normalization-deduplication
Implements entity normalization and deduplication: Rijksoverheid API integration for canonical names, Jaro-Winkler similarity, connected components clustering, and entity merge strategy with audit logging.

### section-06-main-extractor
Implements the full extraction pipeline: coordinates baseline, LLM, normalization, and deduplication stages; applies tiered acceptance logic; handles partial failures; tracks processing time.

### section-07-pipeline-integration
Hooks extractor into document creation workflow: modifies AgentPipeline to call extraction after Content Agent, adds entities to KnowledgeGraph, creates mention relationships, updates PipelineConfig.

### section-08-knowledgegraph-extensions
Adds stakeholder-specific query methods: get_document_stakeholders, get_stakeholder_documents, get_stakeholder_influence, and find_stakeholders_by_name with fuzzy search.

### section-09-api-endpoints
Exposes stakeholder data via REST API: routes for getting stakeholder details, documents by stakeholder, stakeholders by document, and search with fuzzy matching; includes authentication and pagination.

### section-10-testing-validation
Comprehensive test coverage: unit tests for each module (>80% coverage), integration tests for full pipeline, performance benchmarks, GDPR/privacy tests, and test fixtures with Dutch government document samples.
