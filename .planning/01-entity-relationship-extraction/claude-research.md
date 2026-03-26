# Research Findings: Entity & Relationship Extraction

**Date:** 2026-03-16
**Planning:** IOU-Modern Stakeholder GraphRAG

---

## 1. Codebase Analysis: IOU-Modern

### 1.1 Existing NER Implementation

Location: `crates/iou-ai/src/ner.rs`

**Current Approach:** Regex-based extraction for Dutch government entities

**Entity Types Supported:**
- Organizations (provinces, municipalities, ministries)
- Laws (Dutch legislation: Woo, AVG, Archiefwet)
- Dates (Dutch date formats)
- Money (Euro amounts)
- Policy domain terms

**Key Pattern:** Lazy static regex compilation with Dutch-specific patterns

```rust
pub struct DutchNerExtractor;
impl DutchNerExtractor {
    pub fn extract_entities(&self, text: &str) -> Vec<Entity>
    pub fn extract_with_positions(&self, text: &str) -> Vec<(Entity, usize, usize)>
}
```

**Gap:** No Person or Organization extraction for stakeholders (only government bodies and locations)

### 1.2 KnowledgeGraph Implementation

Location: `crates/iou-ai/src/graphrag.rs`

**Graph Backend:** petgraph (DiGraph<Entity, Relationship>)

**Entity Schema:**
```rust
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub entity_type: EntityType,  // Person, Organization, Location, Law, etc.
    pub canonical_name: Option<String>,
    pub confidence: f32,
    pub source_domain_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
```

**Relationship Schema:**
```rust
pub struct Relationship {
    pub relationship_type: RelationshipType,  // WorksFor, LocatedIn, SubjectTo, RefersTo
    pub weight: f32,
    pub confidence: f32,
    pub context: Option<String>,
}
```

**Existing Features:**
- Community detection (SCC, Louvain)
- Path finding (Dijkstra)
- Neighbor queries
- Domain relation discovery

**Gap:** No stakeholder-specific relationship types (reports_to, collaborates_with, advises)

### 1.3 Document Pipeline Integration

Location: `crates/iou-ai/src/agents/pipeline.rs`

**Flow:** Research → Content → Compliance → Review

**Integration Point:** Extract entities during or after Content Agent execution

**Existing TODOs:**
- Line 288: "TODO: Store document in S3"
- Line 289: "TODO: Update database state"
- Line 290: "TODO: Add audit trail logging"

### 1.4 Testing Setup

- Rust built-in testing (`#[test]`, `#[tokio::test]`)
- No external test frameworks
- Integration tests in `crates/iou-api/tests/`
- Unit tests for each NER entity type

### 1.5 Crate Architecture

```
iou-core (domain models)
├── graphrag.rs (Entity, Relationship types)

iou-ai (AI/ML services)
├── ner.rs (Dutch NER - regex based)
├── graphrag.rs (KnowledgeGraph with petgraph)
├── agents/ (Document pipeline)

iou-api (REST API)
├── routes/graphrag.rs (GraphRAG endpoints)
```

---

## 2. Best Practices Research: 2024-2025

### 2.1 Rust NER for Dutch

**Recommended: rust-bert**

- Native Rust transformer-based NLP
- Dutch BERT models for NER
- Entity types: Person, Location, Organization, Miscellaneous
- No Python dependency
- Source: [rust-bert GitHub](https://github.com/guillaume-be/rust-bert)

**Alternative: Hugging Face Candle**

- Official Hugging Face Rust framework (2024)
- Pure Rust, no Python
- Access to all HF models including Dutch

**spaCy Integration:** No official Rust spaCy exists. Options:
1. FFI via pyo3 (complex, Python runtime required)
2. Export spaCy to ONNX, run with tract in Rust
3. Use native Rust alternatives (recommended: rust-bert)

### 2.2 GraphRAG Patterns

**Microsoft GraphRAG (2024)** - Leading reference architecture

**Core Components:**
1. **Entity-Relationship Extraction:** LLM-based extraction from text
2. **Hierarchical Knowledge Graphs:** Multi-level structure
3. **Hybrid Retrieval:** Vector similarity + graph traversal
4. **Community Detection:** Leiden algorithm for clustering

**Workflow:**
```
Extract entities/relationships → Build graph → Detect communities →
Generate summaries → Create embeddings → Hybrid retrieval
```

**Key Benefit:** Better handling of multi-hop reasoning vs pure vector RAG

Source: [Microsoft GraphRAG GitHub](https://github.com/microsoft/graphrag)

### 2.3 Entity Normalization & Deduplication

**Best Practices (2024-2025):**

1. **Blocking Methods:** Reduce comparison space (LSH, sorted neighborhood)
2. **String Similarity:** Jaro-Winkler for names, Levenshtein for variants
3. **Semantic Similarity:** Transformer embeddings (Sentence-BERT)
4. **Phonetic Matching:** Double Metaphone for spelling variations
5. **Graph-Based:** Connected components for transitive relationships

**Pipeline Pattern:**
```
Normalize → Block → Compare → Classify → Cluster → Canonicalize
```

**Hybrid Similarity (Recommended):**
```rust
fn entity_similarity(e1, e2) -> f32 {
    let string_score = jaro_winkler(e1.name, e2.name);
    let semantic_score = cosine_similarity(
        get_embedding(e1.name + e1.context),
        get_embedding(e2.name + e2.context)
    );
    0.4 * string_score + 0.6 * semantic_score
}
```

### 2.4 LLM Entity Extraction with Confidence

**Structured Output with Tool Calling (2024 Best Practice):**

Use Anthropic tool use or OpenAI function calling for guaranteed JSON structure.

**Confidence Scoring Techniques:**

1. **Logprob-Based:** Use token-level probabilities
2. **Self-Reflection:** Ask LLM to evaluate its own extraction
3. **Ensemble Methods:** Multiple runs with voting
4. **Calibration:** Isotonic regression for post-hoc calibration

**Multi-Stage Validation Pattern:**
```
LLM Extraction → Rule Validation → KB Verification → Human Review (low confidence)
```

**Hybrid Approach (Recommended):**
- Stage 1: rust-bert NER (fast baseline)
- Stage 2: LLM extraction for uncertain cases
- Stage 3: Merge and boost confidence for overlaps

**Benefits:** 80-90% cost reduction while maintaining quality

---

## 3. Recommended Architecture for IOU-Modern

```
Text Input
    ↓
[Stage 1] Enhanced Regex (existing patterns) + rust-bert Dutch NER
    ↓
[Stage 2] LLM Entity Extraction (Claude with tool calling)
    ↓
[Stage 3] Entity Normalization & Deduplication
    │   ├── String similarity (Jaro-Winkler)
    │   ├── Semantic similarity (embeddings)
    │   └── Graph clustering (connected components)
    ↓
[Stage 4] Graph Construction (GraphRAG pattern)
    │   ├── Person entities
    │   ├── Organization entities
    │   └── Stakeholder relationships
    ↓
Knowledge Graph (petgraph + PostgreSQL)
```

---

## 4. Testing Recommendations

Based on existing codebase patterns:

- Continue using Rust's built-in testing
- Add `#[tokio::test]` for async extraction functions
- Test with Dutch government document samples
- Property-based tests for normalization (using proptest)
- Integration tests for full pipeline

---

## 5. Key Sources

1. [rust-bert GitHub](https://github.com/guillaume-be/rust-bert)
2. [Microsoft GraphRAG](https://github.com/microsoft/graphrag)
3. [Hugging Face Candle](https://github.com/huggingface/candle)
4. [tract ONNX Runtime](https://github.com/snipsco/tract)
5. [NER Complete Guide 2026](https://www.articsledge.com/post/named-entity-recognition-ner)

---

*Research completed: 2026-03-16*
