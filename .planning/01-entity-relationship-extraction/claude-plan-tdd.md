# TDD Plan: Entity & Relationship Extraction

**Project:** IOU-Modern GraphRAG - Stakeholder Extraction
**Created:** 2026-03-16

**Testing Framework:** Rust built-in (`#[test]`, `#[tokio::test]`)
**Additional:** proptest for property-based testing

---

## Overview

This document defines the tests to write BEFORE implementing each section of the plan. Each test stub describes what to test, not how to implement it.

---

## Section 00: Feasibility Spike

### Test Stubs

- Test: Rijksoverheid API returns canonical name for "MinFin"
- Test: Rijksoverheid API handles unknown organization gracefully
- Test: Local fallback dictionary returns canonical name when API unavailable
- Test: Cost estimation calculates correctly for 1K, 5K, 10K token documents
- Test: GeneratedDocument contains accessible text field

---

## Section 01: Foundation & Types

### Test Stubs

- Test: PersonStakeholder wrapper creates valid Entity with EntityType::Person
- Test: PersonStakeholder metadata contains title, role, department when set
- Test: OrganizationStakeholder wrapper creates valid Entity with EntityType::Organization
- Test: OrganizationStakeholder metadata contains short_name, org_type when set
- Test: MentionRelationship serializes/deserializes correctly
- Test: StakeholderExtractor trait is object-safe (can be boxed)
- Test: Dutch name normalization lowercases prefixes (van, van der, de, ten)
- Test: Dutch name normalization treats "Jan de Vries" and "jan de vries" as same
- Test: Phonetic matching handles common Dutch name variations
- Test: ExtractionError covers all expected failure modes

---

## Section 02: Baseline Extraction

### Test Stubs

- Test: Regex extracts "dr." as title
- Test: Regex extracts "prof." as title
- Test: Regex extracts "minister" as government role
- Test: Regex extracts "staatssecretaris" as government role
- Test: Regex extracts "MinFin" as department abbreviation
- Test: Regex extracts "BZK" as department abbreviation
- Test: rust-bert Dutch NER model loads without error
- Test: rust-bert returns Person entities from Dutch text
- Test: rust-bert returns Organization entities from Dutch text
- Test: Confidence scores are between 0.0 and 1.0
- Test: Pattern "X, minister van Y" creates WorksFor relationship
- Test: Pattern "dr. X, directeur Z" creates WorksFor relationship with role
- Test: Mention type Author detected from document header patterns
- Test: Mention type Recipient detected from "Geachte:" patterns
- Test: Mention type defaults to Referenced when unclear
- Test: Baseline extraction completes in <500ms for typical document

---

## Section 03: Rijksoverheid API Client

### Test Stubs

- Test: get_canonical_name returns "Ministerie van Financiën" for "MinFin"
- Test: get_canonical_name returns "Ministerie van Financiën" for "Ministerie van Financiën"
- Test: get_canonical_name returns None for unknown organization
- Test: get_org_info returns OrgInfo with abbreviations for known ministry
- Test: Cache returns cached result without API call
- Test: Cache expires after TTL
- Test: Client handles API timeout gracefully
- Test: Client handles API error responses gracefully
- Test: clear_expired_cache removes only expired entries

---

## Section 04: LLM Extractor

### Test Stubs

- Test: Claude API call returns structured Entity list
- Test: Tool calling produces valid JSON for entities
- Test: Confidence score calculated from logprobs is between 0.0 and 1.0
- Test: Low-confidence entities trigger retry with different prompt
- Test: Timeout prevents hanging on API failure
- Test: Cost tracking returns token count and estimated cost
- Test: max_llm_calls limit prevents runaway API usage
- Test: max_cost_per_document limit prevents cost overruns
- Test: API key authentication failure returns appropriate error

---

## Section 05: Normalization & Deduplication

### Test Stubs

- Test: Jaro-Winkler similarity returns 1.0 for identical strings
- Test: Jaro-Winkler similarity >0.9 for similar strings
- Test: Jaro-Winkler similarity <0.5 for dissimilar strings
- Test: Normalization applies canonical name from Rijksoverheid API
- Test: Normalization uses local fallback when API unavailable
- Test: Connected components clustering groups duplicate entities
- Test: Merge creates new canonical UUID
- Test: Merge stores old UUIDs as aliases in metadata
- Test: Merge updates all MentionRelationships to canonical entity
- Test: Merge log contains audit trail of entity changes
- Test: Cache reduces API calls for repeated lookups

---

## Section 06: Main Extractor Implementation

### Test Stubs

- Test: Full pipeline returns ExtractionResult with all entity types
- Test: Baseline extraction runs before LLM enhancement
- Test: LLM only called for entities with confidence <0.7
- Test: LLM only called when missing context (title, role)
- Test: Results from baseline and LLM are merged correctly
- Test: Tiered acceptance accepts high confidence (≥0.9) entities
- Test: Tiered acceptance flags medium confidence (0.7-0.9) entities
- Test: Tiered acceptance rejects low confidence (<0.5) entities
- Test: Processing time is tracked and returned
- Test: Failure in LLM stage doesn't lose baseline results
- Test: ExtractionStats correctly counts high/medium/low confidence entities
- Test: Full pipeline completes in <5s for typical document

---

## Section 07: Document Pipeline Integration

### Test Stubs

- Test: StakeholderExtractor called after Content Agent
- Test: Extracted entities added to KnowledgeGraph
- Test: MentionRelationships created for each entity
- Test: Extraction stats logged with correct counts
- Test: Pipeline continues if extraction fails (non-blocking)
- Test: GeneratedDocument text is accessible to extractor
- Test: PipelineConfig enable_stakeholder_extraction controls execution
- Test: PipelineConfig stakeholder_options passed to extractor

---

## Section 08: KnowledgeGraph Extensions

### Test Stubs

- Test: get_document_stakeholders returns all entities for document
- Test: get_document_stakeholders returns empty vec for document with no entities
- Test: get_stakeholder_documents returns all document IDs for entity
- Test: get_stakeholder_documents returns empty vec for unknown entity
- Test: get_stakeholder_influence returns mention_count
- Test: get_stakeholder_influence returns document_count
- Test: get_stakeholder_influence calculates pagerank_score
- Test: find_stakeholders_by_name returns matches above threshold
- Test: find_stakeholders_by_name returns empty vec when no matches
- Test: Methods don't duplicate existing KnowledgeGraph functionality

---

## Section 09: API Endpoints

### Test Stubs

- Test: GET /stakeholders/:id returns stakeholder for valid ID
- Test: GET /stakeholders/:id returns 404 for invalid ID
- Test: GET /stakeholders/:id/documents returns all documents mentioning stakeholder
- Test: GET /documents/:id/stakeholders returns all stakeholders in document
- Test: GET /stakeholders/search?q= returns results for partial name match
- Test: GET /stakeholders/search?q= returns empty results for no match
- Test: Pagination returns correct page size
- Test: Pagination returns correct next/prev page links
- Test: Unauthorized requests return 401
- Test: Citizen PII redacted in API responses
- Test: Official PII not redacted in API responses

---

## Section 10: Testing & Validation

### Test Stubs

#### Unit Tests

- Test coverage >80% for all stakeholder modules
- Test: All regex patterns tested with positive/negative cases
- Test: All error paths tested

#### Integration Tests

- Test: Full extraction pipeline with real Dutch government document
- Test: Document pipeline integration end-to-end
- Test: API endpoints with authenticated client
- Test: Entity deduplication across multiple documents

#### Performance Tests

- Test: Baseline extraction <500ms (p50)
- Test: Baseline extraction <750ms (p95)
- Test: Full pipeline <3s (p50)
- Test: Full pipeline <5s (p95)
- Test: Model load time measured and documented

#### Test Fixtures

- Sample Dutch government document with ministries
- Sample Dutch government document with citizens (PII)
- Sample document with name variations (van, van der, de)
- Sample document with organization abbreviations
- Expected extraction results for each fixture

---

## Property-Based Tests (proptest)

### Test Stubs

- Test: Dutch name normalization is idempotent (normalize(normalize(x)) == normalize(x))
- Test: Confidence scores always in range [0.0, 1.0]
- Test: Entity merge produces single entity with combined aliases
- Test: Similarity function is symmetric (sim(a,b) == sim(b,a))
- Test: Similarity function reflexive (sim(a,a) == 1.0)

---

## GDPR/Privacy Tests

### Test Stubs

- Test: Citizen entities flagged with PiiClassification::Citizen
- Test: Official entities flagged with PiiClassification::Official
- Test: API redacts citizen PII (email, phone) in responses
- Test: API does not redact official PII
- Test: PII access logged for audit trail
- Test: RLS policy restricts citizen entity access

---

*TDD plan created: 2026-03-16*
