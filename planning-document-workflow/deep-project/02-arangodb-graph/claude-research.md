# Research Report: ArangoDB Graph Persistence

## Codebase Analysis

### Existing Types to Preserve

From `crates/iou-core/src/graphrag.rs`:

**Entity struct:**
- `id: Uuid`, `name: String`, `entity_type: EntityType`
- `canonical_name: Option<String>`, `description: Option<String>`
- `confidence: f32`, `source_domain_id: Option<Uuid>`
- `metadata: serde_json::Value`, `created_at: DateTime<Utc>`

**EntityType enum** (SCREAMING_SNAKE_CASE serialization):
- Person, Organization, Location, Law, Date, Money, Policy, Miscellaneous

**Relationship struct:**
- `id: Uuid`, `source_entity_id: Uuid`, `target_entity_id: Uuid`
- `relationship_type: RelationshipType`, `weight: f32`, `confidence: f32`
- `context: Option<String>`, `source_domain_id: Option<Uuid>`, `created_at: DateTime<Utc>`

**RelationshipType enum:**
- WorksFor, LocatedIn, SubjectTo, RefersTo, RelatesTo, OwnerOf, ReportsTo, CollaboratesWith, Follows, PartOf, Unknown

**Community struct:**
- `id: Uuid`, `name: String`, `description: Option<String>`
- `level: i32`, `parent_community_id: Option<Uuid>`
- `member_entity_ids: Vec<Uuid>`, `summary: Option<String>`
- `keywords: Vec<String>`, `created_at: DateTime<Utc>`

### Test Patterns

- Use `#[tokio::test]` for async tests
- Tests marked with `#[ignore = "Requires database connection"]` when needing real DB
- Test utilities in `/tests/common.rs`
- Integration tests in separate files with fixtures

### Module Structure

- Modules declared in `lib.rs` with `pub mod name;`
- Re-exports at bottom of lib.rs for public API
- No separate `mod.rs` files needed (simple structure)

### Error Handling

- Custom error types with `#[derive(thiserror::Error)]`
- `anyhow::Result` for internal errors
- Conversion traits for error chain with `#[from]` attribute

### Dependencies

- **petgraph**: Graph algorithms (already used in iou-ai)
- **sqlx**: PostgreSQL client
- **tokio**: Async runtime
- **serde**: JSON serialization
- **uuid**: UUID generation
- **chrono**: DateTime operations
- **thiserror/anyhow**: Error handling

---

## Arangors Crate Research

### Version & Repository
- **v0.6.0**: https://github.com/fMeow/arangors
- Ergonomic Rust client for ArangoDB
- Async (reqwest/surf) and sync APIs available

### Connection Management

```rust
use arangors::Connection;

// JWT authentication (recommended)
let conn = Connection::establish_jwt("http://localhost:8529", "username", "password").await?;

// Basic auth alternative
let conn = Connection::establish_basic_auth("http://localhost:8529", "username", "password").await?;

// No auth (evaluation only)
let conn = Connection::establish_without_auth("http://localhost:8529").await?;
```

### Database & Collection Operations

```rust
let db = conn.db("my_db").await?;

// Regular collection
let collection = db.create_collection("entities").await?;
let collection = db.collection("entities").await?;

// Edge collection
let edge_collection = db.create_edge_collection("relationships").await?;
```

### CRUD Operations

```rust
use arangors::document::options::*;

// CREATE
let response = collection
    .create_document(entity, InsertOptions::builder().return_new(true).build())
    .await?;

// READ
let doc: Document<Entity> = collection.document(key).await?;

// UPDATE (partial)
let updated = collection
    .update_document(key, patch, UpdateOptions::builder().return_new(true).build())
    .await?;

// DELETE
let deleted = collection
    .remove_document(key, RemoveOptions::builder().return_old(true).build(), None)
    .await?;
```

### AQL Queries

```rust
use arangors::AqlQuery;
use std::collections::HashMap;

// Simple query
let results: Vec<Entity> = db.aql_str("FOR e IN entities RETURN e").await?;

// With bind vars
let mut vars = HashMap::new();
vars.insert("@coll", "entities");
vars.insert("type", "PERSON");

let aql = AqlQuery::builder()
    .query("FOR e IN @@coll FILTER e.entity_type == @type RETURN e")
    .bind_var("@coll", "entities")
    .bind_var("type", "PERSON")
    .batch_size(100)
    .build();

let results: Vec<Entity> = db.aql_query(aql).await?;
```

### Graph Operations

```rust
use arangors::graph::{Graph, EdgeDefinition};

let graph = Graph::builder()
    .name("knowledge_graph".to_string())
    .edge_definitions(vec![
        EdgeDefinition {
            collection: "relationships".to_string(),
            from: vec!["entities".to_string()],
            to: vec!["entities".to_string()],
        }
    ])
    .build();

db.create_graph(graph, true).await?;
```

### Important Notes

- **No built-in connection pooling** in arangors - use `mobc-arangors` or `bb8-arangodb` for pooling
- Use `async` feature with tokio for this codebase
- Handle `ClientError` with variants: `InsufficientPermission`, `Arango`, `Serde`, `HttpClient`

---

## ArangoDB Graph Modeling

### Vertex Collections

**One collection per entity type recommended:**

```javascript
// persons collection
{
  "_key": "person-12345",
  "type": "person",
  "name": "Jane Doe",
  "canonical_name": "Doe, Jane",
  "description": "Software engineer",
  "confidence": 0.95,
  "source_domain_id": "uuid-123",
  "metadata": {},
  "created_at": "2024-01-01T00:00:00Z"
}
```

### Edge Collections

**One edge collection per relationship type:**

```javascript
// edge_works_for collection
{
  "_from": "persons/person-123",
  "_to": "organizations/org-456",
  "relationship_type": "WORKS_FOR",
  "weight": 1.0,
  "confidence": 0.9,
  "context": "Employment since 2020",
  "source_domain_id": "uuid-123",
  "created_at": "2024-01-01T00:00:00Z"
}
```

### Named Graph Definition

```javascript
{
  "_key": "knowledge_graph",
  "edgeDefinitions": [
    {
      "collection": "edge_works_for",
      "from": ["persons"],
      "to": ["organizations"]
    },
    {
      "collection": "edge_located_in",
      "from": ["persons", "organizations"],
      "to": ["locations"]
    }
  ]
}
```

### Indexing Strategy

- **Edge indexes**: Automatic on `_from`, `_to`
- **Persistent indexes**: On `type`, `name`, `created_at`
- **Hash indexes**: On `email`, `canonical_name` (unique)
- **Geo indexes**: On locations with coordinates

### Community Modeling

**Two approaches:**
1. **Explicit community vertices** + membership edges
2. **Computed communities** stored as `community_id` attribute on vertices

**Hybrid recommended:**
- Explicit communities for defined groups
- Computed attributes for discovered clusters

---

## AQL Graph Queries

### Basic Traversal

```aql
FOR v, e, p IN 1..3 OUTBOUND 'entities/123' 'relationships'
  RETURN {vertex: v, edge: e, path: p}
```

### Shortest Path

```aql
FOR v, e, p IN OUTBOUND SHORTEST_PATH
  'entities/source' TO 'entities/target'
  'relationships'
  RETURN p
```

### Get Neighbors

```aql
FOR v, e IN 1..1 ANY 'entities/123' 'relationships'
  RETURN {entity: v, relationship_type: e.relationship_type}
```

### Filter in Traversal

```aql
FOR v, e, p IN 1..3 OUTBOUND 'entities/123' 'relationships'
  FILTER v.entity_type == 'ORGANIZATION'
  FILTER e.confidence >= 0.8
  RETURN v
```

### PRUNE for Performance

```aql
FOR v, e, p IN 1..5 OUTBOUND 'entities/123' 'relationships'
  PRUNE v.entity_type == 'LOCATION'
  RETURN v
```

### Aggregation

```aql
FOR v, e IN 1..1 OUTBOUND 'entities/123' 'relationships'
  COLLECT type = e.relationship_type WITH COUNT INTO count
  RETURN {type, count}
```

### Performance Options

```aql
FOR v, e, p IN 1..3 OUTBOUND 'entities/123' 'relationships'
  OPTIONS {
    bfs: true,
    uniqueVertices: 'global',
    parallelism: 4
  }
  RETURN v
```

### Comparison with SQL

| Feature | AQL | SQL Recursive CTE |
|---------|-----|------------------|
| Syntax | Declarative traversal | Recursive WITH |
| Performance | Optimized for graphs | General-purpose |
| Path extraction | Built-in `p.vertices`/`p.edges` | Manual tracking |
| Direction | OUTBOUND/INBOUND/ANY | Separate queries |

---

## Implementation Recommendations

### Collection Design

**Vertex Collections:**
- `entities` (unified) or separate per type: `persons`, `organizations`, `locations`, `laws`
- Unified simpler for queries, separate better for scalability

**Edge Collections:**
- Single `relationships` with `relationship_type` attribute
- Or separate per type: `edge_works_for`, `edge_located_in`, etc.

**Decision:** Start with unified `entities` + `relationships` for simplicity, add specific collections if performance issues arise.

### Key Arangors Patterns

1. **Use typed queries** for type safety
2. **Use batch queries** for large result sets
3. **Handle revisions** for concurrent updates
4. **Use transactions** for multi-document operations

### ArangoDB vs PostgreSQL for Graph

| Aspect | ArangoDB | PostgreSQL |
|--------|----------|------------|
| Graph queries | Native AQL traversals | Recursive CTEs |
| Performance | Optimized for graphs | General purpose |
| Schema | Flexible | Strict |
| Multi-hop | Sub-100ms typical | Slower with depth |
| Maturity | Mature for graphs | Mature for relations |

---

## Sources

- arangors repository: https://github.com/fMeow/arangors
- ArangoDB Graph Traversals: https://docs.arangodb.com/3.12/aql/graphs/traversals/
- ArangoDB AQL Tutorial: https://www.arangodb.com/docs/devel/aql/tutorial-traversal.html
