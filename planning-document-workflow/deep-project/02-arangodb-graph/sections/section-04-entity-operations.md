# Section 4: GraphStore - Entity Operations

## Overview

This section implements the core CRUD operations for entities in the graph store. The GraphStore struct provides methods to create, read, update, delete, and query entities, with intelligent routing to type-specific collections in ArangoDB.

## Dependencies

This section depends on:
- **Section 1** (Dependencies): arangors, mobc-arangors crates must be added
- **Section 2** (Connection): ArangorsConnectionManager and connection pool must be implemented
- **Section 3** (Errors): StoreError enum and conversion traits must be defined

Ensure these sections are complete before implementing this section.

## Implementation Plan

### File Structure

**File to create:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/store.rs`

**File to modify:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/mod.rs`

### Collection Routing

Entities are routed to type-specific collections based on their `EntityType`:

| EntityType | Collection Name | Collection Type |
|------------|-----------------|-----------------|
| Person | `persons` | Vertex collection |
| Organization | `organizations` | Vertex collection |
| Location | `locations` | Vertex collection |
| Law | `laws` | Vertex collection |
| Date, Money, Policy, Miscellaneous | `entities` | Vertex collection (fallback) |

### Core Struct Definition

```rust
/// Graph store for ArangoDB persistence layer
/// 
/// Provides CRUD operations for entities, relationships, and communities
/// with intelligent routing to type-specific collections.
pub struct GraphStore {
    pool: Pool<ArangorsConnectionManager>,
    db_name: String,
}

impl GraphStore {
    /// Create a new GraphStore instance
    /// 
    /// # Arguments
    /// * `config` - ArangoDB connection configuration
    /// 
    /// # Returns
    /// Result containing GraphStore or StoreError
    pub async fn new(config: &ArangoConfig) -> Result<Self, StoreError>;
    
    /// Ensure all required collections exist
    /// 
    /// Creates vertex collections for each entity type and edge collections
    /// for each relationship type if they don't already exist.
    pub async fn ensure_collections(&self) -> Result<(), StoreError>;
    
    /// Ensure indexes are created on collections
    /// 
    /// Creates persistent and hash indexes for efficient querying:
    /// - Persistent index on `name` field (for text search)
    /// - Hash index on `canonical_name` (for deduplication)
    /// - Hash index on `source_domain_id` (for domain queries)
    pub async fn ensure_indexes(&self) -> Result<(), StoreError>;
}
```

### Entity CRUD Operations

#### Create Entity

```rust
/// Create a new entity in the graph
/// 
/// Routes the entity to the appropriate collection based on its type.
/// Generates a new UUID if not provided.
/// 
/// # Arguments
/// * `entity` - Entity to create (id field optional, will be generated)
/// 
/// # Returns
/// Result containing the created entity with assigned ID
/// 
/// # Errors
/// - StoreError::UniqueViolation if entity with same canonical_name exists
/// - StoreError::Connection if database connection fails
/// - StoreError::Query if AQL execution fails
pub async fn create_entity(&self, entity: &Entity) -> Result<Entity, StoreError>;
```

**Implementation Notes:**
- Generate UUID if `entity.id` is nil
- Convert `entity_type` to collection name using routing logic
- Insert document with arangors `create_document()` method
- Handle unique constraint violations on `canonical_name`
- Return entity with generated `_key` and metadata

#### Get Entity

```rust
/// Get an entity by ID
/// 
/// Queries all vertex collections to find the entity.
/// 
/// # Arguments
/// * `id` - UUID of the entity to retrieve
/// 
/// # Returns
/// - Ok(Some(Entity)) if entity found
/// - Ok(None) if entity not found
/// - Err(StoreError) on database error
pub async fn get_entity(&self, id: Uuid) -> Result<Option<Entity>, StoreError>;
```

**Implementation Notes:**
- Must query all 5 vertex collections (persons, organizations, locations, laws, entities)
- Use AQL UNION approach or sequential collection queries
- Return first non-None result
- Handle document deserialization from ArangoDB format

#### Update Entity

```rust
/// Update an existing entity
/// 
/// # Arguments
/// * `id` - UUID of the entity to update
/// * `updates` - EntityUpdate struct with fields to modify
/// 
/// # Returns
/// Result containing the updated entity
/// 
/// # Errors
/// - StoreError::EntityNotFound if entity doesn't exist
/// - StoreError::Query if update fails
pub async fn update_entity(&self, id: Uuid, updates: EntityUpdate) -> Result<Entity, StoreError>;
```

**EntityUpdate struct:**

```rust
/// Partial update for an entity
/// 
/// All fields are optional; only provided fields are updated.
#[derive(Debug, Clone, Serialize)]
pub struct EntityUpdate {
    pub name: Option<String>,
    pub canonical_name: Option<String>,
    pub description: Option<String>,
    pub confidence: Option<f32>,
    pub metadata: Option<serde_json::Value>,
}
```

#### Delete Entity

```rust
/// Delete an entity from the graph
/// 
/// # Arguments
/// * `id` - UUID of the entity to delete
/// * `cascade` - If true, also delete all edges connected to this entity
/// 
/// # Returns
/// - Ok(true) if entity was deleted
/// - Ok(false) if entity was not found
/// - Err(StoreError) on database error
pub async fn delete_entity(&self, id: Uuid, cascade: bool) -> Result<bool, StoreError>;
```

**Cascade Delete Implementation:**

When `cascade: true`, execute AQL to remove all edges:

```aql
// For each edge collection
FOR e IN edge_works_for
  FILTER e._from == @entity_id OR e._to == @entity_id
  REMOVE e IN edge_works_for
```

Repeat for all edge collections (edge_located_in, edge_subject_to, etc.).

#### List Entities

```rust
/// List entities with filtering and pagination
/// 
/// # Arguments
/// * `filters` - EntityFilters to apply
/// * `pagination` - PaginationOptions for result set
/// 
/// # Returns
/// PaginatedEntities containing results and pagination metadata
pub async fn list_entities(
    &self, 
    filters: EntityFilters, 
    pagination: PaginationOptions
) -> Result<PaginatedEntities, StoreError>;
```

**Filter and Pagination Types:**

```rust
/// Filters for entity queries
#[derive(Debug, Clone, Default)]
pub struct EntityFilters {
    pub entity_type: Option<EntityType>,
    pub source_domain_id: Option<Uuid>,
    pub name_contains: Option<String>,
    pub min_confidence: Option<f32>,
}

/// Pagination options
#[derive(Debug, Clone)]
pub struct PaginationOptions {
    pub limit: usize,
    pub cursor: Option<String>,
}

impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            limit: 50,
            cursor: None,
        }
    }
}

/// Paginated entity results
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedEntities {
    pub entities: Vec<Entity>,
    pub total_count: u64,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}
```

**Implementation Notes:**
- Build AQL query with FILTER clauses based on non-None filters
- Use LIMIT for pagination
- Implement cursor-based pagination using `_id` or created_at timestamp
- Execute query across all relevant collections
- Aggregate results and compute pagination metadata

### Upsert Patterns

#### Get or Create Entity

```rust
/// Get an existing entity or create a new one
/// 
/// Idempotent operation that prevents duplicate entities.
/// Uses canonical_name for uniqueness check.
/// 
/// # Arguments
/// * `entity` - Entity to get or create
/// 
/// # Returns
/// Result containing existing or newly created entity
/// 
/// # Use Cases
/// - Ingesting entities from external sources
/// - Ensuring no duplicates when processing multiple documents
pub async fn get_or_create_entity(&self, entity: &Entity) -> Result<Entity, StoreError>;
```

**Implementation Notes:**
- First attempt to find existing entity by canonical_name
- If found, return existing entity
- If not found, create new entity
- **Critical:** Handle race conditions where concurrent calls might both try to create
- Consider using ArangoDB's UPSERT operation for atomicity

#### Upsert Entity

```rust
/// Update an entity if it exists, or create it if it doesn't
/// 
/// Uses ArangoDB's UPSERT operation for atomicity.
/// 
/// # Arguments
/// * `entity` - Entity to upsert (must have ID)
/// 
/// # Returns
/// Result containing the created or updated entity
pub async fn upsert_entity(&self, entity: &Entity) -> Result<Entity, StoreError>;
```

**AQL UPSERT Pattern:**

```aql
UPSERT { canonical_name: @canonical_name }
INSERT @entity
UPDATE @update_data
IN @@collection
RETURN NEW
```

## Helper Functions

### Collection Name Resolution

```rust
/// Get the collection name for an entity type
/// 
/// # Arguments
/// * `entity_type` - EntityType enum value
/// 
/// # Returns
/// Collection name as string
fn collection_name_for_entity_type(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Person => "persons",
        EntityType::Organization => "organizations",
        EntityType::Location => "locations",
        EntityType::Law => "laws",
        _ => "entities", // Date, Money, Policy, Miscellaneous
    }
}
```

### Entity ID to Document Handle

```rust
/// Convert entity UUID to ArangoDB document handle
/// 
/// Document handle format: "collection_name/_key"
/// 
/// # Arguments
/// * `id` - Entity UUID
/// 
/// # Returns
/// Document handle string for AQL queries
fn entity_id_to_handle(id: Uuid) -> String {
    // Implementation will query to find which collection contains this entity
    // Returns formatted string like "persons/12345678-1234-1234-1234-123456789abc"
}
```

## Error Handling

All methods should return appropriate `StoreError` variants:

- **StoreError::Connection**: Database connection failures
- **StoreError::Query**: AQL execution errors
- **StoreError::EntityNotFound**: Get/update operations on non-existent entities
- **StoreError::UniqueViolation**: Constraint violations on canonical_name
- **StoreError::Arango**: ArangoDB-specific errors with code and message

Use the `From<ClientError> for StoreError` conversion implemented in Section 3.

## Tests

### Test Stubs (from TDD Plan)

Create test file: `/Users/marc/Projecten/iou-modern/crates/iou-core/tests/graphrag/entity_operations.rs`

```rust
use iou_core::graphrag::{Entity, EntityType, GraphStore, StoreError};
use uuid::Uuid;

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn create_entity_returns_entity_with_id() {
    // Test that created entity has generated/assigned ID
    let store = setup_test_store().await;
    let entity = Entity {
        id: Uuid::nil(), // Will be generated
        name: "Test Person".to_string(),
        entity_type: EntityType::Person,
        // ... other fields
        ..Default::default()
    };
    
    let created = store.create_entity(&entity).await.unwrap();
    assert_ne!(created.id, Uuid::nil());
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn create_entity_in_correct_collection() {
    // Test that Person entities go to `persons` collection
    // Test that Organization entities go to `organizations` collection
    // ... verify collection routing
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_found_returns_data() {
    // Test that existing entity returns populated Entity
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_entity_not_found_returns_none() {
    // Test that missing entity returns Ok(None)
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn update_entity_modifies_fields() {
    // Test that updated entity reflects changes
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn update_entity_not_found_returns_error() {
    // Test that updating non-existent entity returns StoreError::EntityNotFound
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn delete_entity_removes_document() {
    // Test that deleted entity no longer accessible
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn delete_entity_cascade_true_removes_edges() {
    // Test that cascade deletes all related edges
    // Create entity with relationships
    // Delete with cascade=true
    // Verify edges are removed
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn delete_entity_cascade_false_keeps_edges() {
    // Test that non-cascade leaves edges (orphaned)
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn list_entities_without_filters_returns_all() {
    // Test that no filters returns all entities
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn list_entities_with_type_filter_filters_correctly() {
    // Test that type filter works
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn list_entities_with_pagination_returns_page() {
    // Test that pagination limit/offset works
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_or_create_entity_new_creates() {
    // Test that non-existent entity is created
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn get_or_create_entity_existing_returns() {
    // Test that existing entity is returned (no duplicate)
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn upsert_entity_new_creates() {
    // Test that upsert creates new entity
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn upsert_entity_existing_updates() {
    // Test that upsert updates existing entity
}

#[tokio::test]
#[ignore = "Requires ArangoDB"]
async fn concurrent_entity_creation_resolves_to_single() {
    // Test that concurrent `get_or_create` returns same entity
    // Use tokio::spawn for concurrent calls
    // Verify only one entity created in database
}
```

## Module Exports

Update `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/mod.rs` to export the new types:

```rust
// Existing exports (preserve these)
pub use crate::graphrag::{
    Entity, EntityType, Relationship, RelationshipType,
    Community, DomainRelation, DomainRelationType,
    DiscoveryMethod, ContextVector, GraphAnalysisResult,
};

// New exports for this section
pub mod store;
pub use store::{
    GraphStore,
    EntityUpdate,
    EntityFilters,
    PaginatedEntities,
    PaginationOptions,
};
```

## Implementation Checklist

- [ ] Create `store.rs` file with GraphStore struct
- [ ] Implement `GraphStore::new()` constructor
- [ ] Implement `GraphStore::ensure_collections()` 
- [ ] Implement `GraphStore::ensure_indexes()`
- [ ] Implement `create_entity()` with collection routing
- [ ] Implement `get_entity()` with multi-collection query
- [ ] Implement `update_entity()` with partial updates
- [ ] Implement `delete_entity()` with cascade support
- [ ] Implement `list_entities()` with filters and pagination
- [ ] Implement `get_or_create_entity()` with idempotency
- [ ] Implement `upsert_entity()` with AQL UPSERT
- [ ] Add helper functions for collection name resolution
- [ ] Update `mod.rs` with public exports
- [ ] Write all test stubs in entity_operations.rs
- [ ] Run tests with `cargo test --package iou-core`
- [ ] Verify collection routing works for all entity types
- [ ] Verify cascade delete removes all edge types
- [ ] Verify pagination cursor logic works correctly

## Key Implementation Details

### ArangoDB Document Mapping

ArangoDB documents use `_id`, `_key`, and `_rev` fields. Map these to the Entity struct:

- Entity `id` ↔ ArangoDB `_key` (UUID as string)
- Store `_id` (collection/key) in metadata if needed for queries
- Ignore `_rev` for now (not exposed in Entity struct)

### AQL Query Safety

- Use bind parameters (`@param`) instead of string interpolation
- Sanitize all user input before using in queries
- Use collection name constants to prevent injection

### Performance Considerations

- Batch `ensure_collections()` calls to avoid round trips
- Use AQL UNION for multi-collection queries when possible
- Implement query result caching for frequently accessed entities
- Use `EXPLAIN` on queries to verify index usage

### Concurrency Handling

- `get_or_create_entity` must handle race conditions:
  - Option 1: Use ArangoDB UPSERT with INSERT only
  - Option 2: Use unique constraint + retry on conflict
  - Option 3: Use application-level mutex for critical sections

## Next Steps

After completing this section:

1. **Section 5**: Implement relationship operations (depends on entity operations)
2. **Section 6**: Implement graph traversals (depends on entity operations)
3. **Section 7**: Implement community operations (depends on entity operations)

The GraphStore will be extended with additional methods in those sections.

---

## Implementation Notes

**Implementation Date:** 2025-03-25

**Files Created:**
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/store.rs` (913 lines)
- `/Users/marc/Projecten/iou-modern/crates/iou-core/tests/graphrag/entity_operations.rs` (665 lines)

**Files Modified:**
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/mod.rs` - Added `store` module and exports

### Implemented Features

All planned CRUD operations were implemented:
- ✅ `GraphStore::new()` - Constructor from ArangoConfig
- ✅ `GraphStore::ensure_collections()` - Creates vertex and edge collections
- ✅ `GraphStore::ensure_indexes()` - Placeholder for index creation (TODO)
- ✅ `GraphStore::create_entity()` - Creates entity with UUID generation
- ✅ `GraphStore::get_entity()` - Retrieves entity by ID across all collections
- ✅ `GraphStore::update_entity()` - Partial updates via EntityUpdate struct
- ✅ `GraphStore::delete_entity()` - Deletion with optional cascade
- ✅ `GraphStore::list_entities()` - Filtering and pagination (basic implementation)
- ✅ `GraphStore::get_or_create_entity()` - Idempotent with race condition handling
- ✅ `GraphStore::upsert_entity()` - ArangoDB UPSERT operation

### Deviations from Plan

1. **AQL Query Approach:** Used arangors `aql_query()` method instead of direct collection API methods for most operations, providing more control over query construction.

2. **Race Condition Fix:** `get_or_create_entity()` uses try-first pattern (attempt insert, catch UniqueViolation, then lookup) instead of check-then-create pattern to handle concurrent requests safely.

3. **Removed Unused Code:** The `EntityDocument::apply_update()` method was not used and was removed to eliminate compiler warnings.

4. **Index Creation:** `ensure_indexes()` is currently a no-op as the arangors index API is complex. Can be implemented later using direct AQL CREATE INDEX queries.

### Test Coverage

Created 22 integration tests covering:
- Basic CRUD operations (create, read, update, delete)
- Collection routing verification
- Cascade delete behavior
- Filtering and pagination
- Idempotent get-or-create operations
- UPSERT create/update scenarios
- Concurrent entity creation

All tests marked with `#[ignore = "Requires ArangoDB"]` as they require a running ArangoDB instance.

### Code Review Notes

The implementation received a code review with the following outcomes:
- **Critical Issue Fixed:** Race condition in `get_or_create_entity()` resolved with try-first pattern
- **Auto-Applied Fix:** Removed unused `apply_update()` function
- **Deferred Items:** Pagination improvements, N+1 query optimization, tracing instrumentation deferred for future iterations

### Performance Considerations

- Sequential collection queries in `get_entity()` and `list_entities()` could be optimized with UNION queries in a future pass
- Pagination is currently in-memory; cursor-based pagination can be added when requirements are clearer
- Connection pooling via mobc provides good concurrency handling