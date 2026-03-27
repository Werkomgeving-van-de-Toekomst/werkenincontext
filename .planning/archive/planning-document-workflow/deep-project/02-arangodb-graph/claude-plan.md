# Implementation Plan: ArangoDB Graph Persistence

## Executive Summary

Implement a native graph database persistence layer using ArangoDB to replace PostgreSQL-based graph storage. This enables efficient multi-hop graph traversals, better performance on complex relationship queries, and native graph algorithms.

The implementation uses:
- **arangors** crate for ArangoDB client
- **mobc-arangors** for connection pooling
- **testcontainers** for integration testing
- Separate vertex collections per entity type
- Separate edge collections per relationship type

---

## Background

### Current State

The iou-modern project has a PostgreSQL-based graph storage approach (attempted in previous iteration) with type definitions in `crates/iou-core/src/graphrag.rs`. The existing types include:

- `Entity` with `EntityType` enum (Person, Organization, Location, Law, Date, Money, Policy, Miscellaneous)
- `Relationship` with `RelationshipType` enum (WorksFor, LocatedIn, SubjectTo, etc.)
- `Community` with hierarchical structure

### Why ArangoDB

PostgreSQL recursive CTEs work for simple graphs but have performance limitations for multi-hop traversals. ArangoDB provides:
- Native graph traversal with AQL
- Sub-100ms single-hop, sub-500ms 3-hop queries
- Built-in shortest path algorithms
- Flexible schema (no migrations needed)

---

## Architecture Overview

### Collection Structure

#### Vertex Collections (4 total)

| Collection | Stores | Key Indexes |
|------------|--------|-------------|
| `persons` | Person entities | `name` (persistent), `canonical_name` (hash, unique), `source_domain_id` |
| `organizations` | Organization entities | `name` (persistent), `canonical_name` (hash, unique), `source_domain_id` |
| `locations` | Location entities | `name` (persistent), `source_domain_id` |
| `laws` | Law/legislation entities | `name` (persistent), `source_domain_id` |

Each vertex contains: `_key` (uuid), `name`, `entity_type`, `canonical_name`, `description`, `confidence`, `source_domain_id`, `metadata`, `created_at`.

#### Edge Collections (10+ total)

| Collection | From | To | Purpose |
|------------|------|-----|---------|
| `edge_works_for` | persons | organizations | Employment relationships |
| `edge_located_in` | persons, orgs | locations | Geographic relationships |
| `edge_subject_to` | entities | laws | Legal compliance |
| `edge_refers_to` | entities | entities | Document references |
| `edge_relates_to` | entities | entities | General associations |
| `edge_owner_of` | persons, orgs | entities | Ownership |
| `edge_reports_to` | persons | persons | Organizational hierarchy |
| `edge_collaborates_with` | entities | entities | Collaboration |
| `edge_follows` | entities | entities | Sequential/temporal |
| `edge_part_of` | entities | entities | Group membership |

Each edge contains: `_from`, `_to`, `relationship_type`, `weight`, `confidence`, `context`, `source_domain_id`, `created_at`.

#### Community Collections (3 total)

| Collection | Purpose |
|------------|---------|
| `communities` | Community vertices (name, level, summary, keywords) |
| `edge_member_of` | Entity → Community membership |
| `edge_subcommunity` | Community → Community hierarchy |

### Named Graph Definition

Create `knowledge_graph` in ArangoDB with all edge collections for cleaner AQL syntax using `GRAPH 'knowledge_graph'` in queries.

---

## Module Structure

```
crates/iou-core/src/graphrag/
├── mod.rs              # Public API, re-exports, type definitions
├── connection.rs       # Connection pooling with mobc
├── store.rs            # GraphStore struct with CRUD operations
├── queries.rs          # AQL query builders and utilities
└── error.rs            # StoreError enum and conversions

crates/iou-core/tests/graphrag/
├── arangodb_integration.rs  # Integration tests with testcontainers
└── common.rs                # Test utilities
```

---

## Implementation Sections

### Section 1: Dependencies and Setup

**Add to `crates/iou-core/Cargo.toml`:**

```toml
arangors = { version = "0.6", features = ["reqwest_async"] }
mobc-arangors = "0.2"
testcontainers = "0.15"
```

**Purpose:** Establish dependencies for ArangoDB client, connection pooling, and testing.

### Section 2: Connection Module

**File:** `crates/iou-core/src/graphrag/connection.rs`

**Struct to implement:**

```rust
pub struct ArangorsConnectionManager {
    connection_url: String,
    username: String,
    credential: String,
    database: String,
}

impl ArangorsConnectionManager {
    pub async fn connect(&self) -> Result<Connection<ReqwestClient>, ClientError>;
}
```

**Function to implement:**

```rust
pub async fn create_pool(config: &ArangoConfig) -> Result<Pool<ArangorsConnectionManager>, StoreError>;
```

**Purpose:** Establish connection pooling with mobc for efficient database access.

### Section 3: Error Types

**File:** `crates/iou-core/src/graphrag/error.rs`

**Enum to define:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(Uuid),

    #[error("Relationship not found: {0}")]
    RelationshipNotFound(Uuid),

    #[error("Community not found: {0}")]
    CommunityNotFound(Uuid),

    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("ArangoDB error [{code}]: {message}")]
    Arango { code: u16, message: String },
}
```

**Conversion impl:** `From<ClientError> for StoreError`

**Purpose:** Centralized error handling with thiserror for ergonomics.

### Section 4: GraphStore - Entity Operations

**File:** `crates/iou-core/src/graphrag/store.rs`

**Struct to define:**

```rust
pub struct GraphStore {
    pool: Pool<ArangorsConnectionManager>,
    db_name: String,
}
```

**Methods to implement:**

```rust
impl GraphStore {
    pub async fn new(config: &ArangoConfig) -> Result<Self, StoreError>;

    pub async fn ensure_collections(&self) -> Result<(), StoreError>;

    pub async fn ensure_indexes(&self) -> Result<(), StoreError>;

    // Entity CRUD
    pub async fn create_entity(&self, entity: &Entity) -> Result<Entity, StoreError>;

    pub async fn get_entity(&self, id: Uuid) -> Result<Option<Entity>, StoreError>;

    pub async fn update_entity(&self, id: Uuid, updates: EntityUpdate) -> Result<Entity, StoreError>;

    pub async fn delete_entity(&self, id: Uuid, cascade: bool) -> Result<bool, StoreError>;

    pub async fn list_entities(&self, filters: EntityFilters, pagination: PaginationOptions) -> Result<PaginatedEntities, StoreError>;

    // Upsert patterns for idempotency
    pub async fn get_or_create_entity(&self, entity: &Entity) -> Result<Entity, StoreError>;

    pub async fn upsert_entity(&self, entity: &Entity) -> Result<Entity, StoreError>;
}
```

**PaginationOptions struct:**

```rust
pub struct PaginationOptions {
    pub limit: usize,
    pub cursor: Option<String>,
}
```

**Purpose:** Core CRUD operations for entities with type-specific collection routing, cascade delete support, and pagination.

### Section 4A: Data Integrity & Cascade Deletes

When `cascade: true` is passed to `delete_entity`, execute AQL to remove all edges:

```aql
FOR e IN edge_works_for
  FILTER e._from == @entity_id OR e._to == @entity_id
  REMOVE e IN edge_works_for
```

Repeat for all edge collections that could reference the deleted entity.

### Section 5: GraphStore - Relationship Operations

**Additional methods on `GraphStore`:**

```rust
pub async fn create_relationship(&self, rel: &Relationship) -> Result<Relationship, StoreError>;

pub async fn get_relationship(&self, id: Uuid) -> Result<Option<Relationship>, StoreError>;

pub async fn delete_relationship(&self, id: Uuid) -> Result<bool, StoreError>;

pub async fn get_entity_relationships(&self, entity_id: Uuid, options: RelationshipQueryOptions) -> Result<Vec<Relationship>, StoreError>;
```

**Purpose:** Relationship CRUD with proper edge collection routing based on `relationship_type`.

### Section 6: GraphStore - Graph Traversals

**Additional methods on `GraphStore`:**

```rust
pub async fn find_shortest_path(&self, from: Uuid, to: Uuid) -> Result<Option<GraphPath>, StoreError>;

pub async fn traverse(&self, request: TraversalRequest) -> Result<TraversalResult, StoreError>;

pub async fn get_neighbors(&self, entity_id: Uuid, filters: NeighborFilters) -> Result<Vec<Neighbor>, StoreError>;
```

**TraversalRequest struct:**

```rust
pub struct TraversalRequest {
    pub start_entity: Uuid,
    pub direction: TraversalDirection,
    pub min_depth: u32,
    pub max_depth: u32,
    pub filters: TraversalFilters,
}
```

**Purpose:** High-performance graph traversals using AQL graph syntax.

### Section 7: GraphStore - Community Operations

**Additional methods on `GraphStore`:**

```rust
pub async fn create_community(&self, community: &Community) -> Result<Community, StoreError>;

pub async fn get_community(&self, id: Uuid) -> Result<Option<Community>, StoreError>;

pub async fn add_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError>;

pub async fn remove_community_member(&self, community_id: Uuid, entity_id: Uuid) -> Result<bool, StoreError>;

pub async fn get_community_members(&self, community_id: Uuid) -> Result<Vec<Entity>, StoreError>;
```

**Purpose:** Community management with flexible membership via edges.

### Section 8: AQL Query Builders

**File:** `crates/iou-core/src/graphrag/queries.rs`

**Functions to implement:**

```rust
pub fn build_neighbors_aql(entity_id: Uuid, filters: &NeighborFilters) -> String;

pub fn build_shortest_path_aql(from: Uuid, to: Uuid) -> String;

pub fn build_traversal_aql(request: &TraversalRequest) -> String;

pub fn build_community_members_aql(community_id: Uuid) -> String;
```

**Purpose:** Centralized AQL query construction for maintainability.

### Section 9: Module Exports

**File:** `crates/iou-core/src/graphrag/mod.rs`

**Update to include:**

```rust
pub mod connection;
pub mod store;
pub mod queries;
pub mod error;

pub use store::{GraphStore, TraversalRequest, TraversalResult};
pub use error::StoreError;
pub use connection::ArangoConfig;
```

**Preserve existing types:** Entity, EntityType, Relationship, RelationshipType, Community, etc.

**Purpose:** Public API surface with clean re-exports.

### Section 10: Integration Tests

**File:** `crates/iou-core/tests/graphrag/arangodb_integration.rs`

**Test functions to implement:**

```rust
#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_entity_crud();

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_relationship_crud();

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_graph_traversal();

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_shortest_path();

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_community_operations();
```

**Test setup function:**

```rust
async fn setup_test_store() -> GraphStore;
```

**Purpose:** Verify all operations work correctly with real ArangoDB instance.

### Section 10A: Concurrency Tests

**Additional test functions:**

```rust
#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_concurrent_entity_creation_resolves_to_single();

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_pool_exhaustion_recovery();
```

**Purpose:** Verify thread safety and connection pool behavior under load.

### Section 10B: Performance Tests

**Benchmark functions:**

```rust
#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_single_hop_under_100ms();

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_three_hop_under_500ms();
```

**Purpose:** Validate performance targets are met.

---

### Section 11: Transaction Support

**File:** `crates/iou-core/src/graphrag/store.rs`

**Add transaction wrapper method:**

```rust
impl GraphStore {
    pub async fn transaction<F, R>(&self, f: F) -> Result<R, StoreError>
    where
        F: FnOnce(&Transaction) -> futures::future::BoxFuture<'_, Result<R, StoreError>>;
}
```

**Transaction struct:**

```rust
pub struct Transaction {
    db: Database<Client>,
}
```

**Purpose:** Enable atomic multi-step operations using ArangoDB stream transactions.

---

### Section 12: Bulk Operations

**File:** `crates/iou-core/src/graphrag/store.rs`

**Methods to implement:**

```rust
impl GraphStore {
    pub async fn bulk_create_entities(&self, entities: Vec<Entity>) -> Result<Vec<Entity>, StoreError>;

    pub async fn bulk_create_relationships(&self, relationships: Vec<Relationship>) -> Result<Vec<Relationship>, StoreError>;

    pub async fn bulk_delete_entities(&self, ids: Vec<Uuid>) -> Result<u64, StoreError>;
}
```

**Purpose:** Efficient batch operations for large-scale data import and cleanup.

---

### Section 13: Migration Strategy

#### Phase 1: Dual-Write Period

- Keep PostgreSQL operational
- Add ArangoDB alongside
- Write all graph data to both databases
- Verify data consistency between systems

#### Phase 2: Read Migration

- Switch read operations to ArangoDB
- Keep PostgreSQL as backup
- Monitor performance and correctness

#### Phase 3: Cutover

- Deprecate PostgreSQL graph storage
- Remove dual-write code
- Keep PostgreSQL for non-graph operations

#### Validation Queries

**Compare entity counts:**

```sql
-- PostgreSQL
SELECT entity_type, COUNT(*) FROM graph_entities GROUP BY entity_type;
```

```aql
-- ArangoDB
FOR e IN entities
  COLLECT type = e.entity_type WITH COUNT INTO count
  RETURN {type, count}
```

---

## AQL Query Reference

### Neighbors Query

```aql
FOR v, e IN 1..1 ANY @entity_id GRAPH 'knowledge_graph'
  FILTER e.relationship_type IN @types
  RETURN {entity: v, relationship: e}
```

### Shortest Path Query

```aql
FOR v, e, p IN OUTBOUND SHORTEST_PATH @from TO @to GRAPH 'knowledge_graph'
  RETURN p
```

### Multi-hop Traversal

```aql
FOR v, e, p IN @min_depth..@max_depth OUTBOUND @entity_id GRAPH 'knowledge_graph'
  FILTER v.entity_type == @entity_type
  OPTIONS {uniqueVertices: 'global', bfs: true}
  RETURN {vertex: v, edge: e, path: p}
```

### Community Members Query

```aql
FOR v, e IN 1..1 INBOUND 'communities/@id' edge_member_of
  RETURN v
```

---

## Testing Strategy

### Testcontainers Setup

Use ArangoDB image: `arangodb/arangodb:3.12`

### Test Categories

1. **Unit Tests**: Query builders, error handling logic
2. **Integration Tests**: Full CRUD with real database
3. **Performance Tests**: Traversal depth benchmarks

### Test Organization

```rust
// Test utilities in tests/common.rs
pub async fn setup_test_store() -> GraphStore {
    let container = ArangoDbContainer::default().start().await?;
    // Configure pool from container
}

// Cleanup between tests
pub async fn clean_collections(store: &GraphStore);
```

---

## Key Decisions & Rationale

| Decision | Rationale |
|----------|-----------|
| Separate vertex collections | Better scalability, type-specific indexes, cleaner shard keys |
| Separate edge collections | Targeted traversals, type-specific attributes, better performance |
| Vertex + edges for communities | Hierarchical support, dynamic membership, future extensibility |
| mobc for pooling | Production-ready, proven pattern for database connection pools |
| testcontainers for testing | Real database behavior, no mocking complexity |

---

## Migration Notes

1. **No schema migrations**: ArangoDB is schemaless, collections created on first use
2. **Backward compatibility**: Preserve existing type definitions in graphrag.rs
3. **Incremental adoption**: Can run alongside existing PostgreSQL during transition
4. **Index creation**: Async operation, handled in `ensure_indexes()`

---

## Success Criteria

- [ ] All entity CRUD operations working
- [ ] All relationship CRUD operations working
- [ ] Graph traversals (1-5 hops) sub-500ms
- [ ] Shortest path queries working
- [ ] Community operations working
- [ ] Integration tests passing
- [ ] Connection pooling configured
- [ ] Error handling comprehensive
