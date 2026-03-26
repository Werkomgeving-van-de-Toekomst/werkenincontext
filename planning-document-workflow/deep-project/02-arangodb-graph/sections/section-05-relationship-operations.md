# Section 5: Relationship Operations

## Overview

This section implements CRUD operations for relationships in the graph database. Relationships are stored as edges in type-specific edge collections.

## Dependencies

- Section 2 (Connection Module)
- Section 3 (Error Types)
- Section 4 (Entity Operations)

## Tests

Write tests BEFORE implementation:
- `create_relationship_creates_in_correct_edge_collection`
- `create_relationship_prevents_duplicates`
- `get_relationship_found_returns_data`
- `get_relationship_not_found_returns_none`
- `delete_relationship_removes_document`
- `get_entity_relationships_returns_both_directions`
- `get_entity_relationships_filters_by_type`
- `get_entity_relationships_paginates_correctly`

## Implementation

File: `crates/iou-core/src/graphrag/store.rs`

### Edge Collection Routing

| RelationshipType | Edge Collection |
|-----------------|-----------------|
| WorksFor | edge_works_for |
| LocatedIn | edge_located_in |
| SubjectTo | edge_subject_to |
| RefersTo | edge_refers_to |
| RelatesTo | edge_relates_to |
| OwnerOf | edge_owner_of |
| ReportsTo | edge_reports_to |
| CollaboratesWith | edge_collaborates_with |
| Follows | edge_follows |
| PartOf | edge_part_of |

### Methods

```rust
impl GraphStore {
    pub async fn create_relationship(&self, rel: &Relationship) -> Result<Relationship, StoreError>;
    pub async fn get_relationship(&self, id: Uuid) -> Result<Option<Relationship>, StoreError>;
    pub async fn delete_relationship(&self, id: Uuid) -> Result<bool, StoreError>;
    pub async fn get_entity_relationships(&self, entity_id: Uuid, options: RelationshipQueryOptions) -> Result<Vec<Relationship>, StoreError>;
}
```

See full implementation plan for details.

---

## Implementation Notes

**Implementation Date:** 2025-03-25

**Files Created:**
- `crates/iou-core/tests/graphrag/relationship_operations.rs` - Integration tests (404 lines)

**Files Modified:**
- `crates/iou-core/src/graphrag/store.rs` - Added relationship CRUD operations (~400 new lines)
- `crates/iou-core/src/graphrag/mod.rs` - Added relationship type exports

### Implemented Features

All planned CRUD operations were implemented:
- ✅ `create_relationship()` - Creates relationship with UUID generation and edge collection routing
- ✅ `get_relationship()` - Retrieves relationship by ID across all edge collections
- ✅ `delete_relationship()` - Deletes relationship by ID
- ✅ `get_entity_relationships()` - Queries relationships with filtering and direction options
- ✅ `RelationshipQueryOptions` - Builder pattern for query configuration
- ✅ `RelationshipDirection` enum - Outgoing, Incoming, Both
- ✅ `collection_name_for_relationship_type()` - Helper for routing

### Deviations from Plan

1. **Edge Document Fields:** Added `_from` and `_to` fields to `RelationshipDocument` to support ArangoDB edge collection requirements. These are critical for graph traversals.

2. **Entity Collection References:** For initial implementation, `_from` and `_to` use a generic "entities/{uuid}" format. A full implementation would query to find the actual collections for each entity.

3. **Missing Operations:** The following operations were not implemented but can be added in future:
   - `update_relationship()` - Partial updates to relationships
   - `get_or_create_relationship()` - Idempotent edge creation
   - Batch relationship operations

### Test Coverage

Created 14 integration tests covering:
- Basic CRUD operations (create, read, delete)
- Edge collection routing verification
- Direction filtering (outgoing, incoming, both)
- Type filtering
- Confidence filtering
- Pagination limits
- Query options builder pattern

All tests marked with `#[ignore = "Requires ArangoDB"]` as they require a running ArangoDB instance.

### Code Review Notes

The implementation received a code review with the following outcomes:
- **Critical Issue Fixed:** Added `_from` and `_to` fields to `RelationshipDocument` for proper edge collection support
- **Deferred Items:** Sequential collection scanning, get_or_create pattern, update operations deferred for future iterations

### API Design

The relationship API follows the same patterns as entity operations:
- Async/await with Result types
- Bind parameters for AQL injection prevention
- Collection routing based on type enums
- Query options with builder pattern for fluent configuration

### Performance Considerations

- `get_relationship()` queries all edge collections sequentially (up to 11 queries worst case)
- `get_entity_relationships()` stops at first collection by default unless `include_all_collections` is true
- Future optimization could use UNION queries for cross-collection searches
