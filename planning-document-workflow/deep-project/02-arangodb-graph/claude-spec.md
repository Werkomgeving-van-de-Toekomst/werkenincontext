# Implementation Specification: ArangoDB Graph Persistence

## Overview

Implement a graph persistence layer for the iou-modern knowledge graph system using ArangoDB. This replaces the previous PostgreSQL-based approach with a native graph database for better performance on complex graph traversals and relationship discovery.

## Requirements

### Functional Requirements

1. **Entity CRUD**: Create, read, update, delete entities with proper type handling
2. **Relationship CRUD**: Create, read, delete relationships between entities
3. **Graph Traversal**: Support 1-5 hop traversals for relationship discovery
4. **Community Storage**: Store communities with flexible membership via edges
5. **Snapshots**: Track graph state over time

### Non-Functional Requirements

- **Performance**: Sub-100ms for 1-hop queries, sub-500ms for 3-hop traversals
- **Scalability**: Support 100K+ entities, 500K+ relationships
- **Reliability**: ACID transactions for multi-document operations
- **Testability**: Integration tests with testcontainers

---

## Architecture

### Collection Design

#### Vertex Collections (Separate per Type)

| Collection | Purpose | Key Indexes |
|------------|---------|-------------|
| `persons` | Person entities | name, canonical_name, source_domain_id |
| `organizations` | Organization entities | name, canonical_name, source_domain_id |
| `locations` | Location entities | name, source_domain_id |
| `laws` | Law/legislation entities | name, source_domain_id |

Each vertex document structure:
```javascript
{
  "_key": "<uuid>",
  "name": "string",
  "entity_type": "PERSON|ORGANIZATION|LOCATION|LAW",
  "canonical_name": "string|null",
  "description": "string|null",
  "confidence": 0.95,
  "source_domain_id": "uuid|null",
  "metadata": {},
  "created_at": "2024-01-01T00:00:00Z"
}
```

#### Edge Collections (Separate per Type)

| Collection | From | To | Purpose |
|------------|------|-----|---------|
| `edge_works_for` | persons | organizations | Employment |
| `edge_located_in` | persons, orgs | locations | Geographic |
| `edge_subject_to` | entities | laws | Legal compliance |
| `edge_refers_to` | entities | entities | References |
| `edge_relates_to` | entities | entities | General relations |
| `edge_owner_of` | persons, orgs | entities | Ownership |
| `edge_reports_to` | persons | persons | Reporting |
| `edge_collaborates_with` | entities | entities | Collaboration |
| `edge_follows` | entities | entities | Sequential |
| `edge_part_of` | entities | entities | Membership |

Each edge document structure:
```javascript
{
  "_from": "<collection>/<key>",
  "_to": "<collection>/<key>",
  "relationship_type": "WORKS_FOR|LOCATED_IN|...",
  "weight": 1.0,
  "confidence": 0.9,
  "context": "string|null",
  "source_domain_id": "uuid|null",
  "created_at": "2024-01-01T00:00:00Z"
}
```

#### Community Collections

| Collection | Purpose |
|------------|---------|
| `communities` | Community vertices |
| `edge_member_of` | Entity → Community membership |
| `edge_subcommunity` | Community → Community hierarchy |

### Named Graph

Define `knowledge_graph` grouping all edge collections for cleaner AQL syntax.

---

## Implementation Plan

### Phase 1: Setup & Connection

**File:** `crates/iou-core/Cargo.toml`

Add dependencies:
```toml
[dependencies]
arangors = "0.6"
mobc-arangors = "0.2"  # Connection pooling
testcontainers = "0.15"  # Integration testing
```

**File:** `crates/iou-core/src/graphrag/connection.rs`

- Connection pooling with mobc
- Database and collection initialization
- Health check functionality

### Phase 2: GraphStore Implementation

**File:** `crates/iou-core/src/graphrag/store.rs`

**Struct:**
```rust
pub struct GraphStore {
    pool: mobc::Pool<ArangorsConnectionManager>,
    db: Database<Client>,
}
```

**Methods:**
- `new()` - Create store with connection pool
- `ensure_collections()` - Create collections if not exist
- `ensure_indexes()` - Create indexes if not exist

#### Entity Operations

- `create_entity(entity_type, entity) -> Result<Entity>`
- `get_entity(id) -> Result<Option<Entity>>`
- `update_entity(id, updates) -> Result<Entity>`
- `delete_entity(id) -> Result<bool>`
- `list_entities(filters) -> Result<Vec<Entity>>`

#### Relationship Operations

- `create_relationship(rel) -> Result<Relationship>`
- `get_relationships(entity_id, direction, depth) -> Result<Vec<Relationship>>`
- `delete_relationship(id) -> Result<bool>`

#### Community Operations

- `create_community(community) -> Result<Community>`
- `get_community(id) -> Result<Option<Community>>`
- `add_member(community_id, entity_id) -> Result<bool>`
- `remove_member(community_id, entity_id) -> Result<bool>`

#### Graph Traversal

- `find_shortest_path(from, to) -> Result<Option<Path>>`
- `traverse(from, direction, min_depth, max_depth, filters) -> Result<TraversalResult>`
- `get_neighbors(entity_id, filters) -> Result<Vec<Neighbor>>`

### Phase 3: Module Structure

**Directory:**
```
crates/iou-core/src/graphrag/
├── mod.rs           # Re-exports, types
├── connection.rs    # Connection pooling
├── store.rs         # GraphStore implementation
├── queries.rs       # AQL query builders
├── error.rs         # Error types
└── types.rs         # Shared types (if splitting from mod.rs)
```

### Phase 4: Testing

**File:** `crates/iou-core/tests/graphrag/arangodb_integration.rs`

```rust
#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_entity_crud() {
    // Testcontainers setup
    // Create entity
    // Verify retrieval
    // Update entity
    // Delete entity
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn test_relationship_traversal() {
    // Create test entities and relationships
    // Execute traversal query
    // Verify results
}
```

---

## AQL Query Patterns

### Get Neighbors (1-hop)

```aql
FOR v, e IN 1..1 ANY @entity_id GRAPH 'knowledge_graph'
  FILTER e.relationship_type IN @types
  RETURN {entity: v, relationship: e}
```

### Shortest Path

```aql
FOR v, e, p IN OUTBOUND SHORTEST_PATH
  @from TO @to
  GRAPH 'knowledge_graph'
  RETURN p
```

### Multi-hop Traversal with Filters

```aql
FOR v, e, p IN 1..@depth OUTBOUND @entity_id GRAPH 'knowledge_graph'
  FILTER v.entity_type == @entity_type
  OPTIONS {uniqueVertices: 'global', bfs: true}
  RETURN {vertex: v, path: p}
```

### Community Members

```aql
FOR v, e IN 1..1 INBOUND 'communities/@id' edge_member_of
  RETURN v
```

---

## Error Handling

**File:** `crates/iou-core/src/graphrag/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(Uuid),

    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("ArangoDB error: {code:?} - {message}")]
    Arango { code: u16, message: String },
}
```

---

## Testing Strategy

### Unit Tests

- Mock arangors client
- Test query builder logic
- Test error handling

### Integration Tests

- Testcontainers with ArangoDB image
- Full CRUD operations
- Graph traversal queries
- Transaction rollback

### Test Setup

```rust
async fn setup_test_store() -> GraphStore {
    let container = ArangoDbContainer::default().start().await?;
    let conn = container.connection().await?;
    GraphStore::new(conn).await
}
```

---

## Migration Path

1. Add dependencies to Cargo.toml
2. Create connection module with pooling
3. Implement GraphStore with basic CRUD
4. Add AQL query builders
5. Write integration tests
6. Update module exports in lib.rs
7. Run tests and verify

---

## Sources

- arangors crate: https://github.com/fMeow/arangors
- ArangoDB AQL Graphs: https://docs.arangodb.com/3.12/aql/graphs/
- Testcontainers Rust: https://docs.rs/testcontainers/
