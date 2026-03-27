# Code Review Interview: Section 03 - Research Agent

## Date
2026-03-01

## Triage Summary

### Auto-Fixes Applied

#### 1. Config Validation (Medium)
**Issue:** `ResearchAgentConfig` has no validation for threshold ranges (0.0-1.0) or max_similar_documents > 0.

**Decision:** Add validation method to `ResearchAgentConfig` that returns `Result<(), String>` for invalid configurations.

#### 2. Add TODO Comments for Stubs (Medium)
**Issue:** PROVISA guidelines and similarity scoring are stubbed without documentation.

**Decision:** Add prominent TODO comments indicating these are placeholder implementations.

#### 3. Case-Insensitive Entity Search (Low)
**Issue:** Entity search is case-sensitive and may miss matches.

**Decision:** Convert to case-insensitive search using `to_lowercase()`.

#### 4. Lazy Domain Name (Low)
**Issue:** `format!("Domain {}", domain_config.domain_id)` produces unhelpful output.

**Decision:** Add comment noting this should be fetched from a domain registry in the future.

## User Interview Items

### 1. EntityReference.entity_id Type Mismatch (High Priority)

**Issue:** The plan specifies `entity_id: String` but implementation uses `Uuid`.

```rust
// Current implementation (research.rs:70)
pub struct EntityReference {
    pub entity_id: Uuid,  // Plan says: String
    pub entity_type: String,
    pub name: String,
    pub relevance_score: f32,
}
```

**Options:**
1. Change to `String` to match the plan (allows storing non-UUID entity IDs)
2. Keep `Uuid` and update the plan documentation (more type-safe for internal entities)

**Recommendation:** Change to `String` - the plan is correct for flexibility with external knowledge graphs.

### 2. Incomplete GraphRAG Integration (High Priority)

**Issue:** `query_similar_documents` doesn't actually perform similarity search. It:
- Ignores `similarity_threshold` config
- Returns hardcoded `similarity_score: 0.8`
- Only filters by entity type, not semantic similarity

**Options:**
1. Leave as stub for future section (GraphRAG integration may be section-07 or later)
2. Implement basic similarity using cosine similarity on document titles
3. Add TODO comment with expected behavior

**Recommendation:** Leave as stub with prominent TODO - full semantic search requires embedding infrastructure not yet built.

## Decisions Made

### Decision 1: EntityReference.entity_id → Change to String
**Rationale:** More flexible for external knowledge graphs that may use non-UUID entity identifiers.

**Action Required:** Change `entity_id: Uuid` to `entity_id: String` in EntityReference struct and update all usages.

### Decision 2: GraphRAG Stub → Add TODO Comments
**Rationale:** Full semantic similarity search requires embedding infrastructure that will be built in later sections.

**Action Required:** Add prominent TODO comments to `query_similar_documents` and related functions.
