# Specification: Entity & Relationship Extraction for Stakeholder GraphRAG

**Created:** 2026-03-16
**Split:** 01-entity-relationship-extraction
**Project:** IOU-Modern GraphRAG Integration

---

## Overview

Extract persons, organizations, and their mentions from Dutch government documents. Build a stakeholder knowledge graph incrementally using a hybrid approach that combines fast baseline extraction with LLM-enhanced accuracy.

## Context

### Problem Statement

IOU-Modern processes Dutch government documents (Woo documents, policy briefs, meeting notes) but lacks automated stakeholder entity extraction. Currently:
- Basic NER exists for government bodies (via regex)
- No extraction of person names or roles
- No tracking of stakeholder mentions across documents
- Knowledge graph exists but is manually populated

### Solution Approach

Implement a hybrid extraction pipeline:
1. **Fast baseline:** Enhanced regex + rust-bert Dutch NER
2. **LLM enhancement:** Claude API for uncertain cases only
3. **Normalization:** External API lookup (Rijksoverheid) for canonical names
4. **Tiered acceptance:** Auto-accept high confidence, flag low for review

### Non-Goals

- Phase 1: Relationship classification (reports_to, collaborates_with, etc.) - deferred to Phase 2
- Real-time entity updates from external sources
- Advanced entity linking (same person across different documents)
- Social media or web scraping

---

## Requirements

### Functional Requirements

| ID | Requirement | Description | Priority |
|----|-------------|-------------|----------|
| ERE-01 | Person Extraction | Extract person names with titles, roles, departments | P0 |
| ERE-02 | Organization Extraction | Extract org names with types and abbreviations | P0 |
| ERE-03 | Mention Detection | Track which stakeholders are mentioned in each document | P0 |
| ERE-04 | Entity Normalization | Resolve variants using Rijksoverheid API lookup | P0 |
| ERE-05 | Confidence Scoring | Score each extraction for quality filtering | P0 |
| ERE-06 | Tiered Acceptance | Auto-accept high confidence, flag low for review | P0 |
| ERE-07 | Pipeline Integration | Extract during document creation workflow | P0 |
| ERE-08 | Real-time Performance | Complete extraction in <5 seconds per document | P0 |

### Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | Extraction Precision | >85% for persons, >90% for organizations |
| NFR-02 | Extraction Recall | >70% for entities |
| NFR-03 | Latency | <5 seconds per document (95th percentile) |
| NFR-04 | Availability | 99.5% (handle Rijksoverheid API failures) |
| NFR-05 | GDPR Compliance | All extractions logged for audit trail |

---

## Data Structures

### Person Entity

```rust
pub struct PersonEntity {
    pub id: Uuid,
    pub name: String,           // "Jan de Vries"
    pub title: Option<String>,  // "Dr.", "Prof.", "mr.", "mr. dr."
    pub role: Option<String>,   // "beleidsmedewerker", "minister", "ambtenaar"
    pub department: Option<String>, // "MinFin", "Directie Algemene Financiële Politiek"
    pub email: Option<String>,
    pub phone: Option<String>,
    pub confidence: f32,        // 0.0 to 1.0
    pub source_document_id: Uuid,
    pub extraction_method: ExtractionMethod, // Regex, RustBert, LLM
    pub created_at: DateTime<Utc>,
}
```

### Organization Entity

```rust
pub struct OrganizationEntity {
    pub id: Uuid,
    pub name: String,           // "Ministerie van Financiën"
    pub short_name: Option<String>, // "MinFin"
    pub org_type: OrgType,      // Ministry, Agency, Municipal, Province, Other
    pub canonical_name: Option<String>, // From Rijksoverheid API
    pub parent_org: Option<String>,
    pub location: Option<String>,
    pub confidence: f32,
    pub source_document_id: Uuid,
    pub extraction_method: ExtractionMethod,
    pub created_at: DateTime<Utc>,
}
```

### Mention Relationship

```rust
pub struct MentionRelationship {
    pub id: Uuid,
    pub entity_id: Uuid,        // Person or Organization
    pub document_id: Uuid,
    pub mention_type: MentionType, // Subject, Author, Recipient, Referenced
    pub context: Option<String>, // Surrounding text
    pub position: Option<TextPosition>,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}
```

### Extraction Method

```rust
pub enum ExtractionMethod {
    Regex,          // Fast pattern matching
    RustBert,       // Transformer-based NER
    LLM,            // Claude API extraction
    Manual,         // User-entered
}
```

---

## Architecture

### Extraction Pipeline

```
Document Input
    ↓
┌─────────────────────────────────────────────────┐
│ Stage 1: Fast Baseline (<500ms)                  │
├─────────────────────────────────────────────────┤
│ • Enhanced regex patterns (existing DutchNer)    │
│ • rust-bert Dutch NER for persons/orgs          │
│ • Pattern-based title/role extraction            │
└─────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────┐
│ Stage 2: Uncertainty Detection                  │
├─────────────────────────────────────────────────┤
│ • Low confidence (<0.7) → LLM review            │
│ • Missing context → LLM review                  │
│ • Ambiguous entities → LLM review               │
└─────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────┐
│ Stage 3: LLM Enhancement (for uncertain cases)   │
├─────────────────────────────────────────────────┤
│ • Claude Sonnet 4.5 API                         │
│ • Tool calling for structured output            │
│ • Returns entities with confidence scores        │
└─────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────┐
│ Stage 4: Normalization & Deduplication           │
├─────────────────────────────────────────────────┤
│ • Rijksoverheid API lookup for orgs             │
│ • String similarity matching                     │
│ • Canonical name resolution                      │
└─────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────┐
│ Stage 5: Tiered Acceptance                       │
├─────────────────────────────────────────────────┤
│ • High (≥0.9): Auto-accept                      │
│ • Medium (0.7-0.9): Accept with verification    │
│ • Low (0.5-0.7): Flag for review                │
│ • Very low (<0.5): Reject                        │
└─────────────────────────────────────────────────┘
    ↓
Knowledge Graph (petgraph) + PostgreSQL
```

### API Design

```rust
/// Main extraction service
pub trait StakeholderExtractor {
    /// Extract entities from document
    async fn extract(
        &self,
        document: &Document,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult>;

    /// Normalize entities using external API
    async fn normalize_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>>;

    /// Detect and deduplicate similar entities
    async fn deduplicate_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>>;
}

pub struct ExtractionResult {
    pub persons: Vec<PersonEntity>,
    pub organizations: Vec<OrganizationEntity>,
    pub mentions: Vec<MentionRelationship>,
    pub confidence_stats: ConfidenceStats,
    pub processing_time_ms: u64,
}

pub struct ExtractionOptions {
    pub use_llm: bool,
    pub confidence_threshold: f32,
    pub enable_normalization: bool,
    pub max_llm_calls: usize,
}
```

---

## Integration Points

### Document Pipeline

**Location:** `crates/iou-ai/src/agents/pipeline.rs`

**Hook:** After Content Agent generates document content

```rust
// In AgentPipeline::execute_document_pipeline
// After Content Agent completes:

let extraction_result = stakeholder_extractor
    .extract(document, &extraction_options)
    .await?;

// Add entities to knowledge graph
for person in extraction_result.persons {
    knowledge_graph.add_entity(person.into());
}
for org in extraction_result.organizations {
    knowledge_graph.add_entity(org.into());
}

// Store mention relationships
for mention in extraction_result.mentions {
    knowledge_graph.add_relationship(mention.into());
}
```

### KnowledgeGraph Extension

**Location:** `crates/iou-ai/src/graphrag.rs`

**Add stakeholder-specific methods:**

```rust
impl KnowledgeGraph {
    /// Get all stakeholders mentioned in a document
    pub fn get_document_stakeholders(&self, document_id: Uuid) -> Vec<&Entity>;

    /// Find documents mentioning a stakeholder
    pub fn get_stakeholder_documents(&self, entity_id: Uuid) -> Vec<&Entity>;

    /// Get stakeholder influence metrics
    pub fn get_stakeholder_influence(&self, entity_id: Uuid) -> InfluenceMetrics;
}
```

### Rijksoverheid API Client

**New module:** `crates/iou-ai/src/rijksoverheid.rs`

```rust
/// Client for Dutch government organization API
pub struct RijksoverheidClient {
    base_url: String,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, OrgInfo>>>,
}

impl RijksoverheidClient {
    /// Get canonical name for organization abbreviation
    pub async fn get_canonical_name(&self, abbrev: &str) -> Option<String>;

    /// Get organization details
    pub async fn get_org_info(&self, name: &str) -> Option<OrgInfo>;
}
```

---

## Testing Strategy

### Unit Tests

- Regex pattern tests (existing: `test_extract_province`, etc.)
- rust-bert integration tests with Dutch sample text
- Normalization logic tests
- Confidence calculation tests

### Integration Tests

- End-to-end extraction pipeline
- Rijksoverheid API client (mocked)
- KnowledgeGraph integration
- Document pipeline hook

### Test Data

- Sample Woo documents
- Policy briefs with known stakeholders
- Meeting notes with person/organization mentions

### Performance Tests

- Benchmark extraction latency (target: <5s 95th percentile)
- Load test with concurrent document processing
- Cache effectiveness metrics

---

## Success Criteria

- [ ] Person names extracted with >85% precision
- [ ] Organization names extracted with >90% precision
- [ ] Entities normalized using Rijksoverheid API
- [ ] Extraction completes in <5 seconds (95th percentile)
- [ ] Tiered acceptance filters low-confidence extractions
- [ ] Integration with document pipeline complete
- [ ] KnowledgeGraph populated with extracted entities

---

## References

- Original spec: `.planning/01-entity-relationship-extraction/spec.md`
- Research findings: `claude-research.md`
- Interview transcript: `claude-interview.md`
- Existing NER: `crates/iou-ai/src/ner.rs`
- KnowledgeGraph: `crates/iou-ai/src/graphrag.rs`
- Document pipeline: `crates/iou-ai/src/agents/pipeline.rs`
- rust-bert: https://github.com/guillaume-be/rust-bert
- Rijksoverheid API: https://api.data.overheid.nl/

---

*Specification created: 2026-03-16*
