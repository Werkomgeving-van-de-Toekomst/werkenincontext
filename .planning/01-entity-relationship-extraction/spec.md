# Spec: Entity & Relationship Extraction

**Split:** 01-entity-relationship-extraction
**Created:** 2026-03-16
**Estimated Duration:** 1-2 sprints

## Overview

Extract persons, organizations, and their relationships from Dutch government documents. Build the stakeholder knowledge graph incrementally as documents are processed.

## Context

### Existing Infrastructure

IOU-Modern has basic NER (`crates/iou-ai/src/ner.rs`) using regex patterns for:
- Case numbers (zaaknummers)
- BSN (citizen service numbers)
- KvK (Chamber of Commerce numbers)

**Gap:** Need to extract:
- Person names with roles/titles
- Organization names with types
- Relationships between entities
- Contextual information (department, contact info)

### Document Sources

- Woo documents (besluiten, inventarisaties)
- Policy briefs (beleidsnotities)
- Meeting notes (notulen)
- Correspondence (brieven, emails)

## Requirements

### Core Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| ERE-01 | Person Entity Extraction | Extract person names, roles, departments, contact info |
| ERE-02 | Organization Extraction | Extract org names, types, parent orgs, locations |
| ERE-03 | Relationship Extraction | Classify relationships: reports_to, collaborates_with, advises |
| ERE-04 | Entity Normalization | Deduplicate entities, resolve variants (MinFin vs Ministerie van Financiën) |
| ERE-05 | Confidence Scoring | Score each extraction for quality filtering |
| ERE-06 | Pipeline Integration | Extract entities during document creation pipeline |

### Entity Schema

**Person:**
```rust
struct PersonEntity {
    name: String,           // "Jan de Vries"
    title: Option<String>,  // "Dr.", "Prof."
    role: Option<String>,   // "beleidsmedewerker", "minister"
    department: Option<String>, // "MinFin", "Directie Algemene Financiële Politiek"
    email: Option<String>,
    phone: Option<String>,
    confidence: f32,
}
```

**Organization:**
```rust
struct OrganizationEntity {
    name: String,           // "Ministerie van Financiën"
    short_name: Option<String>, // "MinFin"
    org_type: OrgType,      // Ministry, Agency, Municipal, etc.
    parent_org: Option<String>,
    location: Option<String>,
    confidence: f32,
}
```

### Relationship Schema

```rust
enum StakeholderRelationship {
    ReportsTo { person: PersonId, supervisor: PersonId },
    Manages { person: PersonId, org: OrganizationId },
    CollaboratesWith { entity1: EntityId, entity2: EntityId },
    Advises { advisor: EntityId, advised: EntityId },
    MentionedIn { entity: EntityId, document: DocumentId },
}
```

## Technical Approach

### NER Options

| Approach | Pros | Cons |
|----------|------|------|
| spaCy Dutch | Good accuracy, ready to use | External dependency, Python runtime |
| Fine-tuned transformer | Best accuracy | Complex, GPU needed |
| Enhanced regex | Simple, fast | Lower accuracy |
| Pattern-based + LLM | Good balance | Requires LLM API |

**Recommended:** Start with pattern-based rules + regex, enhance with LLM for context.

### Extraction Pipeline

```
Document → Tokenize → Sentence Split → Entity Recognition →
Relationship Extraction → Normalization → Storage
```

### Integration Points

- **Document Pipeline:** Hook into `AgentPipeline` after Content Agent
- **KnowledgeGraph:** Use existing `KnowledgeGraph::add_entity()`
- **Database:** Store entities/relationships in PostgreSQL

## API Design

```rust
/// Extract entities from document text
async fn extract_stakeholders(
    document: &Document,
    context: &ExtractionContext,
) -> Result<ExtractionResult>

/// Normalize and deduplicate entities
async fn normalize_entities(
    entities: Vec<Entity>,
    graph: &KnowledgeGraph,
) -> Vec<Entity>

/// Extract relationships between entities
async fn extract_relationships(
    entities: &[Entity],
    document: &Document,
) -> Vec<StakeholderRelationship>
```

## Acceptance Criteria

- [ ] Person names extracted with >85% precision
- [ ] Organization names extracted with >90% precision
- [ ] Relationships classified with >75% precision
- [ ] Entities normalized (variants resolved)
- [ ] Extraction runs in background during document processing
- [ ] Low-quality extractions filtered by confidence threshold

## Success Metrics

| Metric | Target |
|--------|--------|
| Entity extraction precision | >85% |
| Entity extraction recall | >70% |
| Relationship classification | >75% |
| Extraction speed | <5 seconds per document |

## Out of Scope

- Real-time entity updates from external sources
- Social media scraping
- Face recognition from images
- Advanced entity linking (same person across docs)

## Constraints

- Dutch language primary
- Must comply with AVG/GDPR
- Must handle government terminology correctly
- Must integrate with existing KnowledgeGraph

---

## References

- Requirements: `.planning/graphrag-stakeholder-requirements.md`
- Existing NER: `crates/iou-ai/src/ner.rs`
- KnowledgeGraph: `crates/iou-ai/src/graphrag.rs`
