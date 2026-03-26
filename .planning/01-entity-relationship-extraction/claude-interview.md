# Interview Transcript: Entity & Relationship Extraction

**Date:** 2026-03-16
**Spec:** 01-entity-relationship-extraction

---

## Q1: LLM Provider Selection

**Question:** Which LLM provider should be used for entity extraction?

**Answer:** Claude API (Sonnet 4.5)

**Notes:**
- User prefers Claude over OpenAI or local models
- Good for Dutch language understanding
- Strong tool calling support for structured output

---

## Q2: Entity Normalization Approach

**Question:** How should entity normalization handle Dutch government abbreviations?

**Answer:** External API lookup

**Notes:**
- Use Rijksoverheid (Dutch government) API for canonical names
- Resolves variants like MinFin → Ministerie van Financiën
- More authoritative than maintaining local abbreviation lists

**Implementation Consideration:**
- Need to cache API responses for performance
- Handle API failures gracefully with fallback to local rules

---

## Q3: Latency Requirements

**Question:** What are the latency requirements for entity extraction during document processing?

**Answer:** Real-time required (<5 seconds)

**Notes:**
- Users expect responsive UI
- LLM API calls may introduce latency
- Hybrid approach recommended to meet this requirement

---

## Q4: Hybrid Extraction Strategy

**Question:** Should we use a hybrid extraction strategy to meet real-time requirements?

**Answer:** Hybrid recommended

**Approach:**
1. **Stage 1:** Regex/rust-bert NER (fast, <500ms)
2. **Stage 2:** LLM extraction only for uncertain cases
3. **Stage 3:** Merge and boost confidence for overlapping entities

**Benefits:**
- 80-90% of extractions from fast baseline
- LLM only for ~10-20% ambiguous cases
- Meets <5s requirement while maintaining quality

---

## Q5: Relationship Types Scope

**Question:** Which stakeholder relationship types should be extracted in Phase 1?

**Answer:** Mentions first, then expand

**Approach:**
- **Phase 1:** Extract which stakeholders are mentioned in documents
- **Phase 2:** Add relationship classification (reports_to, collaborates_with, etc.)
- **Rationale:** Simpler to implement, provides immediate value

**Relationship Types Planned for Later:**
- Hierarchical: reports_to, manages
- Collaborative: collaborates_with, advises
- Organizational: works_for, located_in

---

## Q6: Confidence Handling

**Question:** How should low-confidence entity extractions be handled?

**Answer:** Tiered acceptance

**Tiers:**
- **High confidence (≥0.9):** Auto-accept, add to graph
- **Medium confidence (0.7-0.9):** Accept with verification flag
- **Low confidence (0.5-0.7):** Flag for manual review
- **Very low (<0.5):** Reject

**Implementation:**
- Show confidence scores in UI
- Allow users to manually approve/reject
- Learn from corrections over time

---

## Summary of Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| LLM Provider | Claude Sonnet 4.5 API | Best Dutch support, tool calling |
| Normalization | External API (Rijksoverheid) | Authoritative source |
| Latency Target | <5 seconds | Real-time UI requirement |
| Strategy | Hybrid (regex + LLM) | Balance speed and accuracy |
| Relationships | Mentions first, expand later | Incremental complexity |
| Confidence | Tiered acceptance | Balance automation and quality |

---

## Technical Implications

1. **Performance:** Hybrid approach required for <5s target
2. **Caching:** External API calls must be cached
3. **Phasing:** Start simple, add relationship classification later
4. **UX:** Show confidence scores, allow manual review
5. **API Integration:** Need Rijksoverheid API client

---

*Interview completed: 2026-03-16*
