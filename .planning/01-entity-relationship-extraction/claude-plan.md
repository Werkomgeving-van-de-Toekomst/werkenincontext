# Implementation Plan: Entity & Relationship Extraction

**Project:** IOU-Modern GraphRAG - Stakeholder Extraction
**Created:** 2026-03-16
**Split:** 01-entity-relationship-extraction

---

## Overview

This plan describes building a hybrid entity extraction system for Dutch government documents. The system extracts person and organization entities, normalizes them using external APIs, and populates the existing KnowledgeGraph for stakeholder analysis.

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Hybrid extraction (regex + LLM) | Meets <5s latency while maintaining accuracy |
| Claude Sonnet 4.5 for LLM | Best Dutch language support, structured output |
| Rijksoverheid API for normalization | Authoritative source for Dutch government orgs |
| Tiered confidence acceptance | Balances automation with quality control |
| Mentions-only in Phase 1 | Incremental complexity, relationships deferred |

---

## Architecture

### System Context

```
┌─────────────────────────────────────────────────────────────────┐
│                         IOU-Modern                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐      ┌─────────────────┐      ┌───────────┐ │
│  │   Document   │ ───> │  Stakeholder    │ ───> │ Knowledge │ │
│  │   Pipeline   │      │   Extractor     │      │   Graph   │ │
│  └──────────────┘      └─────────────────┘      └───────────┘ │
│                                │                              │
│                                ▼                              │
│                       ┌─────────────────┐                     │
│                       │ Rijksoverheid   │                     │
│                       │     API         │                     │
│                       └─────────────────┘                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Extraction Pipeline

```
Document Text
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│ Stage 1: Fast Baseline (target: <500ms)                      │
├─────────────────────────────────────────────────────────────┤
│ • Enhanced regex for Dutch patterns (titles, roles)          │
│ • rust-bert Dutch NER for persons/organizations             │
│ • Returns entities with confidence scores                    │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│ Stage 2: Uncertainty Filter                                   │
├─────────────────────────────────────────────────────────────┤
│ • If confidence < 0.7 → Stage 3                              │
│ • If missing context (title, role) → Stage 3                 │
│ • If ambiguous entities → Stage 3                            │
│ • Otherwise → Stage 4                                        │
└─────────────────────────────────────────────────────────────┘
     │                    │
     │                    ▼
     │           ┌───────────────────────────────────────────┐
     │           │ Stage 3: LLM Enhancement (Claude API)      │
     │           ├───────────────────────────────────────────┤
     │           │ • Send uncertain text to Claude            │
     │           │ • Tool calling for structured output       │
     │           │ • Returns entities with confidence        │
     │           └───────────────────────────────────────────┘
     │                    │
     └────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Stage 4: Normalization                                       │
├─────────────────────────────────────────────────────────────┤
│ • Rijksoverheid API lookup for organizations                │
│ • String similarity for deduplication (Jaro-Winkler)        │
│ • Canonical name resolution                                 │
│ • Cache API responses (TTL: 24h)                            │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│ Stage 5: Tiered Acceptance                                   │
├─────────────────────────────────────────────────────────────┤
│ • High (≥0.9): Auto-accept                                  │
│ • Medium (0.7-0.9): Accept with verification flag           │
│ • Low (0.5-0.7): Flag for manual review                     │
│ • Very low (<0.5): Reject                                   │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│ Stage 6: Graph Storage                                       │
├─────────────────────────────────────────────────────────────┤
│ • Add entities to KnowledgeGraph (petgraph)                 │
│ • Create MentionRelationship for each document              │
│ • Update PostgreSQL for query/search                        │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Structure

```
iou-ai/
├── src/
│   ├── stakeholder/
│   │   ├── mod.rs              # Public API
│   │   ├── extractor.rs        # Main extraction trait
│   │   ├── baseline.rs         # Regex + rust-bert implementation
│   │   ├── llm_extractor.rs    # Claude API integration
│   │   ├── normalizer.rs       # Entity normalization
│   │   ├── deduplicator.rs     # Entity deduplication
│   │   ├── rijksoverheid.rs    # Dutch gov API client
│   │   └── confidence.rs       # Confidence scoring
│   ├── entities/
│   │   ├── person.rs           # PersonEntity type
│   │   ├── organization.rs     # OrganizationEntity type
│   │   └── mention.rs          # MentionRelationship type
│   └── agents/
│       └── pipeline.rs         # Integration point
```

---

## Data Structures

### Type System Approach

**Note:** The existing `Entity` type in `iou-core/src/graphrag.rs` is used as the foundation. Stakeholder-specific attributes are stored in the `metadata` field, with convenience wrappers providing type-safe access.

### Core Types

```rust
use iou_core::graphrag::{Entity, EntityType, Relationship};

/// Convenience wrapper for person entities with typed metadata access
pub struct PersonStakeholder {
    pub entity: Entity,
}

impl PersonStakeholder {
    // Accessors for metadata fields
    pub fn title(&self) -> Option<&str>;
    pub fn role(&self) -> Option<&str>;
    pub fn department(&self) -> Option<&str>;
    pub fn email(&self) -> Option<&str>;
    pub fn phone(&self) -> Option<&str>;

    // Builder for creating from extraction
    pub fn new(name: String, confidence: f32) -> Self;
}

/// Convenience wrapper for organization entities with typed metadata access
pub struct OrganizationStakeholder {
    pub entity: Entity,
}

impl OrganizationStakeholder {
    // Accessors for metadata fields
    pub fn short_name(&self) -> Option<&str>;
    pub fn org_type(&self) -> Option<OrgType>;
    pub fn parent_org(&self) -> Option<&str>;
    pub fn location(&self) -> Option<&str>;

    // Builder for creating from extraction
    pub fn new(name: String, confidence: f32) -> Self;
}

/// Mention relationship between entity and document
pub struct MentionRelationship {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub document_id: Uuid,
    pub mention_type: MentionType,  // NOW POPULATED during extraction
    pub context: Option<String>,
    pub position: Option<TextPosition>,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

/// How the entity was extracted
pub enum ExtractionMethod {
    Regex,
    RustBert,
    LLM,
    Manual,
}

/// Verification tier based on confidence
pub enum VerificationStatus {
    AutoAccepted,       // confidence ≥ 0.9
    AcceptedFlagged,    // 0.7 ≤ confidence < 0.9
    PendingReview,      // 0.5 ≤ confidence < 0.7
    Rejected,           // confidence < 0.5
}

/// Type of organization (simplified for Phase 1 detection)
pub enum OrgType {
    Ministry,      // Detectable via "ministerie" keyword
    Agency,        // Detectable via "dienst", "agentschap" keywords
    Other,         // Default for undetermined
}

/// How the entity is mentioned (populated during extraction)
pub enum MentionType {
    Subject,       // Primary subject of sentence
    Author,        // Author of document
    Recipient,     // Recipient/recipient
    Referenced,    // Referenced in passing (default)
}
```

### Extraction Result

```rust
/// Result of extraction process
pub struct ExtractionResult {
    pub persons: Vec<PersonEntity>,
    pub organizations: Vec<OrganizationEntity>,
    pub mentions: Vec<MentionRelationship>,
    pub stats: ExtractionStats,
    pub processing_time_ms: u64,
}

pub struct ExtractionStats {
    pub total_entities: usize,
    pub high_confidence: usize,
    pub medium_confidence: usize,
    pub low_confidence: usize,
    pub llm_calls_made: usize,
    pub api_calls_made: usize,
}
```

---

## API Definitions

### Main Extractor Trait

```rust
/// Main stakeholder extraction interface
#[async_trait]
pub trait StakeholderExtractor: Send + Sync {
    /// Extract entities from a document
    async fn extract(
        &self,
        document: &Document,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, ExtractionError>;

    /// Normalize entities using external API
    async fn normalize_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, NormalizationError>;

    /// Deduplicate entities by similarity
    async fn deduplicate_entities(
        &self,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>, DeduplicationError>;
}
```

### Extraction Options

```rust
/// Configuration for extraction process
pub struct ExtractionOptions {
    /// Whether to use LLM for uncertain cases
    pub use_llm: bool,

    /// Minimum confidence threshold
    pub confidence_threshold: f32,

    /// Whether to normalize via external API
    pub enable_normalization: bool,

    /// Maximum LLM calls per document (cost control)
    pub max_llm_calls: usize,

    /// Maximum cost per document in USD (cost control)
    pub max_cost_per_document: f32,

    /// Timeout for LLM API calls
    pub llm_timeout: Duration,
}

impl Default for ExtractionOptions {
    fn default() -> Self {
        Self {
            use_llm: true,
            confidence_threshold: 0.5,
            enable_normalization: true,
            max_llm_calls: 10,
            max_cost_per_document: 0.10, // $0.10 per document max
            llm_timeout: Duration::from_secs(10),
        }
    }
}
```

### Cost Estimation

**Token Count Estimates:**
- Short document (1-3 pages): ~1K-2K tokens
- Medium document (5-10 pages): ~3K-5K tokens
- Long document (15-20 pages): ~6K-8K tokens

**Cost per Document (Claude Sonnet 4.5):**
- Input: ~$3/1M tokens → $0.006-$0.024 per document
- Output: ~$15/1M tokens → ~$0.003 per extraction
- **Total: ~$0.01-$0.05 per document** (within $0.10 budget)

**Cost Mitigation:**
- Aggressive baseline extraction (regex + rust-bert) reduces LLM calls
- API response caching for normalization
- Per-document budget prevents runaway costs

### Claude LLM Extractor

```rust
/// Claude API integration for entity extraction
pub struct ClaudeExtractor {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl ClaudeExtractor {
    /// Create new Claude extractor
    pub fn new(api_key: String) -> Self;

    /// Extract entities from text using Claude tool calling
    pub async fn extract_entities(
        &self,
        text: &str,
        context: &ExtractionContext,
    ) -> Result<Vec<Entity>, ClaudeError>;

    /// Check if extraction is needed based on confidence
    pub fn should_extract(&self, confidence: f32) -> bool {
        confidence < 0.7
    }
}
```

### Rijksoverheid API Client

```rust
/// Client for Dutch government organization API
pub struct RijksoverheidClient {
    base_url: String,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, OrgInfo>>>,
    cache_ttl: Duration,
}

impl RijksoverheidClient {
    /// Create new client with in-memory cache
    pub fn new() -> Self;

    /// Get canonical name for organization
    pub async fn get_canonical_name(
        &self,
        name_or_abbrev: &str,
    ) -> Result<Option<String>, ApiError>;

    /// Get full organization info
    pub async fn get_org_info(
        &self,
        name: &str,
    ) -> Result<Option<OrgInfo>, ApiError>;

    /// Clear expired cache entries
    pub async fn clear_expired_cache(&self);
}

pub struct OrgInfo {
    pub canonical_name: String,
    pub abbreviations: Vec<String>,
    pub org_type: OrgType,
    pub parent_org: Option<String>,
}
```

---

## Document Pipeline Integration

### Integration Point

Location: `crates/iou-ai/src/agents/pipeline.rs`

After Content Agent generates document content, insert stakeholder extraction:

```rust
// In AgentPipelineWithConfig::execute_document_pipeline
// After Content Agent completes (around line 250):

// Extract stakeholders from generated document
let extraction_result = stakeholder_extractor
    .extract(&generated_document, &extraction_options)
    .await?;

// Add extracted entities to knowledge graph
let mut entity_ids = Vec::new();

for person in extraction_result.persons {
    let entity: Entity = person.into();
    knowledge_graph.add_entity(entity.clone()).await;
    entity_ids.push(entity.id);
}

for org in extraction_result.organizations {
    let entity: Entity = org.into();
    knowledge_graph.add_entity(entity.clone()).await;
    entity_ids.push(entity.id);
}

// Create mention relationships
for mention in extraction_result.mentions {
    let relationship: Relationship = mention.into();
    knowledge_graph.add_relationship(relationship).await;
}

// Log extraction stats
tracing::info!(
    "Stakeholder extraction complete: {} persons, {} organizations, {} mentions",
    extraction_result.persons.len(),
    extraction_result.organizations.len(),
    extraction_result.mentions.len()
);
```

### Pipeline Configuration

Add to `PipelineConfig`:

```rust
pub struct PipelineConfig {
    // ... existing fields

    /// Enable stakeholder extraction
    pub enable_stakeholder_extraction: bool,

    /// Stakeholder extraction options
    pub stakeholder_options: ExtractionOptions,
}
```

---

## KnowledgeGraph Extensions

### New Methods

Location: `crates/iou-ai/src/graphrag.rs`

```rust
impl KnowledgeGraph {
    /// Get all stakeholders mentioned in a document
    pub fn get_document_stakeholders(
        &self,
        document_id: Uuid,
    ) -> Vec<&Entity>;

    /// Get all documents mentioning a stakeholder
    pub fn get_stakeholder_documents(
        &self,
        entity_id: Uuid,
    ) -> Vec<Uuid>;

    /// Get influence metrics for a stakeholder
    pub fn get_stakeholder_influence(
        &self,
        entity_id: Uuid,
    ) -> InfluenceMetrics;

    /// Find stakeholders by name (fuzzy search)
    pub fn find_stakeholders_by_name(
        &self,
        name: &str,
        threshold: f32,
    ) -> Vec<&Entity>;
}

pub struct InfluenceMetrics {
    pub entity_id: Uuid,
    pub mention_count: usize,
    pub document_count: usize,
    pub pagerank_score: f32,
    pub betweenness_centrality: f32,
}
```

---

## Implementation Sections

### Section 00: Feasibility Spike (NEW)

**Goal:** Validate external dependencies before committing to architecture

**Tasks:**
- Verify Rijksoverheid API (`api.data.overheid.nl`) capabilities:
  - Test canonical name lookup endpoint
  - Document actual response format
  - Check rate limits and authentication requirements
- Create cost estimation model:
  - Measure average token counts for Dutch government documents
  - Calculate per-document extraction costs
  - Set `max_cost_per_document` threshold
- Implement local fallback:
  - Create dictionary of common Dutch government orgs
  - MinFin → Ministerie van Financiën
  - BZK → Ministerie van Binnenlandse Zaken en Koninkrijksrelaties
- Verify `GeneratedDocument` structure in pipeline

**Dependencies:** None (can run in parallel with Section 01)

**Success Criteria:**
- API capabilities documented or fallback strategy confirmed
- Cost model established
- Local fallback dictionary created

---

### Section 01: Foundation & Types

**Goal:** Define core data structures and public API

**Tasks:**
- Create `iou-ai/src/stakeholder/` module structure
- Define `PersonStakeholder`, `OrganizationStakeholder` convenience wrappers
- Define `StakeholderExtractor` trait
- Define `ExtractionResult` and related types
- Add basic error types
- Implement Dutch name normalization module:
  - Lowercase prefixes (van, van der, de, ten) for comparison
  - Handle title variations (dr. vs dr)
  - Phonetic matching for similar names

**Dependencies:** None

**Success Criteria:**
- Types compile without errors
- Trait definition is complete
- Error types cover expected failure modes
- Name normalization handles Dutch prefixes correctly

---

### Section 02: Baseline Extraction

**Goal:** Implement fast regex + rust-bert extraction

**Tasks:**
- Create `baseline.rs` with regex patterns for Dutch:
  - Person titles (dr., prof., mr., mr. dr., ing., ir.)
  - Government roles (minister, staatssecretaris, ambtenaar, etc.)
  - Department patterns (MinFin, BZK, etc.)
- Integrate rust-bert for Dutch NER:
  - Add `rust-bert` dependency
  - Load Dutch NER model (eager load at startup)
  - Implement entity extraction
- Implement confidence scoring
- Add basic relationship extraction (Pattern-based):
  - "X, minister van Y" → X WorksFor Y
  - "dr. X, directeur Z" → X hasRole at Z
  - Common Dutch government relationship patterns
- Implement mention type detection:
  - Detect author from document headers
  - Detect recipient from "Geachte: ..." patterns
  - Default to Referenced if unclear
- Write unit tests for patterns

**Dependencies:** Section 01

**Success Criteria:**
- Regex patterns extract known Dutch titles/roles
- rust-bert model loads successfully
- Extraction produces entities with confidence scores
- Basic relationships extracted (person ↔ organization)
- Mention types are detected and populated
- Unit tests pass for Dutch sample text

---

### Section 03: Rijksoverheid API Client

**Goal:** Implement Dutch government organization lookup

**Tasks:**
- Create `rijksoverheid.rs` module
- Implement API client with:
  - `get_canonical_name()` method
  - `get_org_info()` method
  - In-memory caching with TTL
- Add integration with `data.overheid.nl` API
- Implement fallback for API failures
- Write tests (with mocked API responses)

**Dependencies:** Section 01

**Success Criteria:**
- API client resolves "MinFin" → "Ministerie van Financiën"
- Cache reduces API calls for repeated lookups
- Graceful degradation when API is unavailable
- Tests verify canonical name resolution

---

### Section 04: LLM Extractor

**Goal:** Implement Claude API integration

**Tasks:**
- Create `llm_extractor.rs` module
- Integrate Anthropic Rust SDK or use reqwest
- Define Claude tool schema for entity extraction
- Implement `extract_entities()` with:
  - Tool calling for structured output
  - Confidence scoring from logprobs
  - Timeout handling
- Add cost tracking (tokens per document)
- Write tests with mocked Claude responses

**Dependencies:** Section 01

**Success Criteria:**
- Claude API returns structured entities
- Confidence scores are calculated from logprobs
- Timeout prevents hanging on API failures
- Cost tracking monitors token usage

---

### Section 05: Normalization & Deduplication

**Goal:** Implement entity normalization and deduplication

**Tasks:**
- Create `normalizer.rs` module:
  - Integrate RijksoverheidClient
  - Apply canonical names to organizations
  - Cache normalized entities
- Create `deduplicator.rs` module:
  - Implement Jaro-Winkler similarity
  - Implement connected components clustering
  - Merge duplicate entities with strategy:
    - Create new canonical UUID for merged entity
    - Store old UUIDs as aliases in metadata
    - Update all MentionRelationships to point to canonical entity
    - Implement entity merge log for auditability
- Write tests for:
  - Known entity variants (MinFin vs Ministerie van Financiën)
  - Duplicate detection
  - Canonical name resolution
  - Merge strategy verification

**Dependencies:** Sections 01, 03

**Success Criteria:**
- Entity variants resolve to canonical form
- Duplicates are detected and merged
- Normalization cache reduces repeated API calls
- Merge log provides audit trail

---

### Section 06: Main Extractor Implementation

**Goal:** Implement the full extraction pipeline

**Tasks:**
- Create `extractor.rs` with `StakeholderExtractor` impl:
  - Coordinate baseline, LLM, normalization, deduplication
  - Apply tiered acceptance logic
  - Track processing time
  - Handle partial failures gracefully
- Implement hybrid logic:
  - Run baseline first
  - Filter uncertain entities
  - Call LLM for uncertain cases (within limits)
  - Merge results
- Write integration tests

**Dependencies:** Sections 01, 02, 03, 04, 05

**Success Criteria:**
- End-to-end extraction produces all entity types
- Processing time <5 seconds for typical documents
- LLM is called only for uncertain cases
- Failures in one stage don't block others

---

### Section 07: Document Pipeline Integration

**Goal:** Hook extractor into document creation workflow

**Tasks:**
- Modify `iou-ai/src/agents/pipeline.rs`:
  - Add `StakeholderExtractor` to `AgentPipeline`
  - Call extraction after Content Agent
  - Add entities to KnowledgeGraph
  - Create mention relationships
- Update `PipelineConfig` with stakeholder options
- Add logging for extraction stats
- Write integration test with full pipeline

**Dependencies:** Sections 01, 06

**Success Criteria:**
- Documents are processed with stakeholder extraction
- Entities appear in KnowledgeGraph after pipeline
- Extraction stats are logged
- Pipeline errors don't lose document content

---

### Section 08: KnowledgeGraph Extensions

**Goal:** Add stakeholder-specific query methods

**Tasks:**
- Extend `iou-ai/src/graphrag.rs`:
  - `get_document_stakeholders()`
  - `get_stakeholder_documents()`
  - `get_stakeholder_influence()`
  - `find_stakeholders_by_name()`
- Implement influence metrics calculation
- Add tests for new methods

**Dependencies:** Sections 01, 07

**Success Criteria:**
- Can query stakeholders by document
- Can query documents by stakeholder
- Influence metrics are calculated
- Fuzzy name search works

---

### Section 09: API Endpoints

**Goal:** Expose stakeholder data via REST API

**Tasks:**
- Add routes to `iou-api/src/routes/stakeholder.rs`:
  - `GET /stakeholders/:id` - Get stakeholder details
  - `GET /stakeholders/:id/documents` - Get documents mentioning stakeholder
  - `GET /documents/:id/stakeholders` - Get stakeholders in document
  - `GET /stakeholders/search?q=` - Search stakeholders
- Implement pagination
- Add authentication/authorization
- Write API tests

**Dependencies:** Sections 01, 08

**Success Criteria:**
- Endpoints return stakeholder data
- Search works with fuzzy matching
- Pagination handles large result sets
- Unauthorized requests are rejected

---

### Section 10: Testing & Validation

**Goal:** Comprehensive test coverage

**Tasks:**
- Unit tests for each module (target: >80% coverage)
- Integration tests for:
  - Full extraction pipeline
  - Document pipeline integration
  - API endpoints
- Performance tests:
  - Baseline extraction <500ms
  - Full pipeline <5s (95th percentile)
- Create test fixtures:
  - Sample Dutch government documents
  - Known stakeholder entities
  - Expected extraction results

**Dependencies:** All previous sections

**Success Criteria:**
- All tests pass
- Coverage target met
- Performance benchmarks pass
- Test fixtures are realistic

---

## Dependencies

### External Crates

```toml
[dependencies]
# Existing
iou-core = { path = "../iou-core" }
petgraph = "0.6"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }

# New
rust-bert = "0.22"          # Dutch NER
reqwest = { version = "0.12", features = ["json"] }
anthropic-rust = "0.1"      # Claude API (or use reqwest)
strsim = "0.11"             # Jaro-Winkler similarity
```

### API Keys Required

- `ANTHROPIC_API_KEY` - Claude Sonnet 4.5 access
- Rijksoverheid API (open, no key required)

---

## Configuration

### Environment Variables

```bash
# Claude API
ANTHROPIC_API_KEY=sk-ant-xxxxx
CLAUDE_MODEL=claude-sonnet-4-20250514

# Rijksoverheid API
RIJKSOVERHEID_API_URL=https://api.data.overheid.nl
RIJKSOVERHEID_CACHE_TTL=86400  # 24 hours

# Extraction behavior
EXTRACTION_CONFIDENCE_THRESHOLD=0.5
EXTRACTION_MAX_LLM_CALLS=10
EXTRACTION_LLM_TIMEOUT=10
```

---

## Error Handling Strategy

| Error Type | Handling | User Impact |
|------------|----------|-------------|
| Baseline extraction fails | Log warning, continue | Reduced entities, not blocking |
| LLM API timeout | Use baseline results only | May miss some entities |
| Rijksoverheid API fails | Use local rules, cache hit | Non-canonical names |
| KnowledgeGraph write fails | Return error, retry | Blocking, requires manual intervention |

---

## PII and Privacy Handling

### GDPR Considerations

This system processes Wet open overheid (Woo) documents which may contain:
- Government officials (public interest, lower privacy expectations)
- Citizens' personal information (high privacy protection required)

### PII Classification

```rust
pub enum PiiClassification {
    /// Government officials (ministers, directors, etc.)
    Official,

    /// Citizens mentioned in documents
    Citizen,

    /// Generic entities, no PII concerns
    None,
}
```

### Handling Rules

| Classification | Retention | Access Control | Redaction |
|----------------|-----------|----------------|-----------|
| Official | 7 years | Role-based | No |
| Citizen | 2 years | Restricted | Yes (API) |
| None | Indefinite | Public | No |

### Implementation

- Flag entities with `pii_classification` in metadata
- Apply RLS policies on `stakeholder_entities` table
- Automatic redaction for citizen PII in API responses
- Audit log for all citizen PII access

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Person extraction precision | >85% | Test set evaluation |
| Organization extraction precision | >90% | Test set evaluation |
| End-to-end latency (95th %ile) | <5 seconds | Performance tests |
| Baseline extraction time | <500ms | Performance tests |
| LLM call reduction | >80% | Production metrics |
| API cache hit rate | >60% | Production metrics |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| LLM API rate limits | Implement queue, exponential backoff |
| Rijksoverheid API downtime | Local fallback cache (24h TTL) |
| Low confidence extractions | Tiered acceptance, manual review queue |
| Dutch language nuances | Training on government document samples |
| Cost overruns (LLM) | Per-document budget, aggressive caching |

---

*Plan created: 2026-03-16*
