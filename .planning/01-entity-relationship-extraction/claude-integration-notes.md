# Integration Notes: External Review Feedback

**Plan Version:** iteration-1
**Review Date:** 2026-03-16
**Reviewer:** Claude Opus (via subagent)

---

## Summary

The Opus review identified 20 issues across critical gaps, significant gaps, security concerns, and minor suggestions. This document records what was integrated and why.

---

## Critical Issues

### ✅ Issue 1: Type System Integration (INTEGRATED)

**Problem:** Plan defined `PersonEntity` and `OrganizationEntity` structs, but `Entity` type already exists in `iou-core/src/graphrag.rs` with `EntityType::Person` and `EntityType::Organization`.

**Action:** Revised the data structures approach:
- Use existing `Entity` type as the foundation
- Store stakeholder-specific attributes in `metadata` field:
  - Persons: `title`, `role`, `department`, `email`, `phone`
  - Organizations: `short_name`, `org_type`, `parent_org`, `location`
- Create convenience wrapper types `PersonStakeholder` and `OrganizationStakeholder` that provide typed access to metadata
- These wrappers implement `From<Entity>` and `Into<Entity>` for easy conversion

**Rationale:** Eliminates type duplication while maintaining type-safe access to stakeholder-specific fields.

### ✅ Issue 2: Missing Relationships in Phase 1 (PARTIALLY INTEGRATED)

**Problem:** Phase 1 only extracts mentions, missing the primary value of stakeholder extraction.

**Action:** Added basic relationship extraction to Phase 1:
- Pattern-based extraction: "X, minister van Y" → X WorksFor Y
- Title-based: "dr. X, directeur Z" → X hasRole at Z
- Simple regex patterns for common Dutch government relationship patterns

**Rationale:** Baseline relationship extraction provides immediate value without LLM complexity.

### ✅ Issue 3: LLM Cost Control (INTEGRATED)

**Problem:** No cost estimation, only `max_llm_calls` limit.

**Action:** Added cost estimation section:
- Estimated token counts for Dutch government documents (5-20 pages ≈ 2K-8K tokens)
- Per-document cost calculation at ~$0.01-$0.05 per doc with Claude Sonnet 4.5
- Added `max_cost_per_document` threshold in addition to call limits
- Added cost tracking metrics

**Rationale:** Prevents unexpected operational costs.

### ✅ Issue 4: Rijksoverheid API Assumptions (INTEGRATED)

**Problem:** Assumed API capabilities without verification.

**Action:** Added Phase 0 (Feasibility Spike):
- Verify Rijksoverheid API capabilities before main implementation
- Document rate limits and actual endpoints
- Implement local fallback dictionary for common Dutch gov orgs

**Rationale:** Validates external dependencies before committing to architecture.

---

## Significant Gaps

### ✅ Issue 5: ID Management for Deduplication (INTEGRATED)

**Problem:** No merge strategy for duplicate entity UUIDs.

**Action:** Specified merge strategy:
- Create new canonical UUID for merged entity
- Store old UUIDs as aliases in metadata
- Update all relationships to point to canonical entity
- Implement entity merge log for auditability

**Rationale:** Ensures graph consistency and auditability.

### ✅ Issue 6: MentionRelationship Directionality (INTEGRATED)

**Problem:** `mention_type` field exists but isn't populated.

**Action:** Added mention type detection:
- Detect if entity is author, recipient, subject, or referenced
- Pattern-based detection from document structure
- Default to `Referenced` if unclear

**Rationale:** Enables queries like "who wrote this document?"

### ✅ Issue 7: Pipeline Integration Point (NOTED)

**Problem:** Need to verify `GeneratedDocument` structure.

**Action:** Added verification task in Section 07:
- Inspect `GeneratedDocument` to ensure text field exists
- Add fallback to document content retrieval if needed

**Rationale:** Prevents integration failures due to type mismatches.

### ✅ Issue 8: Dutch Name Variations (INTEGRATED)

**Problem:** No handling of Dutch name prefixes, titles, etc.

**Action:** Added Dutch name normalization module:
- Lowercase prefixes (van, van der, de, ten) for comparison
- Handle title variations (dr. vs dr)
- Phonetic matching for similar names

**Rationale:** Critical for deduplication quality.

### ✅ Issue 9: Performance Benchmarks (INTEGRATED)

**Problem:** No consideration for model loading time.

**Action:** Clarified performance targets:
- Model loading: eager load at startup (~1-2s one-time)
- Baseline extraction: <500ms (warm model)
- Full pipeline: <5s (95th percentile, warm model)
- Added cold start documentation

**Rationale:** Realistic performance expectations.

---

## Security & Privacy

### ✅ Issue 10: PII Handling (INTEGRATED)

**Problem:** No discussion of PII retention or access control.

**Action:** Added PII handling section:
- Flag entities as PII based on role (citizen vs official)
- Different retention policies for citizens vs government officials
- RLS policies on stakeholder tables
- Automatic redaction for citizen PII in API responses

**Rationale:** GDPR compliance for Woo documents.

### ✅ Issue 11: API Key Management (NOTED)

**Problem:** No secure key management strategy.

**Action:** Noted that the project should use existing key management infrastructure (documented in ops runbooks).

**Rationale:** Leverages existing security practices.

---

## Architecture Concerns

### ✅ Issue 12: petgraph Duplication (NOTED)

**Problem:** Potential duplication of existing graph operations.

**Action:** Added verification task to review existing `KnowledgeGraph` methods before implementing new ones.

**Rationale:** Avoids code duplication.

### ⏸️ Issue 13: Entity Evolution (DEFERRED)

**Problem:** No temporal tracking of entity changes.

**Action:** Deferred to future phase.
- Temporal entity tracking adds significant complexity
- Out of scope for MVP

**Rationale:** Incremental development approach.

---

## Minor Issues & Suggestions

### ✅ Issue 14: Test Fixtures (INTEGRATED)

**Action:** Specified test fixture content:
- Common Dutch title patterns
- Government organization abbreviations
- Edge cases (similar names, orgs)

### ✅ Issue 15: Error Recovery (INTEGRATED)

**Action:** Added transaction-like pattern:
- Collect all changes in batch
- Apply atomically or roll back on failure

### ✅ Issue 16: Confidence Calibration (INTEGRATED)

**Action:** Added calibration phase using labeled test set.

### ⏸️ Issue 17: Metrics Dashboard (DEFERRED)

**Action:** Basic metrics collection added, full dashboard deferred to Phase 2.

### ⏸️ Issue 18: rust-bert Model Size (NOTED)

**Action:** Noted for implementation phase - models are large (~500MB) and need storage strategy.

### ⏸️ Issue 19: anthropic-rust Crate (NOTED)

**Action:** Will use `reqwest` directly if crate is unmaintained.

### ✅ Issue 20: Unused Type Enum Values (INTEGRATED)

**Action:** Simplified `OrgType` to only include values we can detect in Phase 1.

---

## Additional Additions

### Phase 0: Feasibility Spike (NEW)

Added new section before main implementation:
- Verify Rijksoverheid API capabilities
- Document actual API endpoints and rate limits
- Create cost estimation model
- Validate external dependencies

### Privacy Impact Assessment (NEW)

Added section on GDPR considerations:
- PII flagging
- Retention policies
- Access control
- Redaction rules

---

## Items NOT Integrated

1. **Temporal Entity Tracking** - Deferred to future phase
2. **Full Metrics Dashboard** - Basic metrics only for MVP
3. **Advanced Entity Linking** - Deferred (same person across documents)

**Reason:** These add significant complexity and are out of scope for the MVP focused on single-document extraction.
