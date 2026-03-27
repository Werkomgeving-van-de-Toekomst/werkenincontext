# Implementation Interview: ArangoDB Graph Persistence

**Date:** 2026-03-25
**Project:** iou-modern ArangoDB integration

---

## Q1: Entity Collection Structure

**Question:** How should entities be stored in ArangoDB - unified or separate collections?

**Answer:** Separate per type

**Rationale:** Better scalability as the graph grows. Each entity type can have:
- Type-specific indexes
- Optimized shard keys in cluster deployments
- Independent growth patterns

**Collections to create:**
- `persons` - Person entities
- `organizations` - Organization entities
- `locations` - Location entities
- `laws` - Law/legislation entities

---

## Q2: Relationship Collection Structure

**Question:** How should relationships be modeled - single or multiple edge collections?

**Answer:** Separate per type

**Rationale:** Better query performance and organization:
- Each relationship type gets optimized indexes
- Traversals can target specific edge collections
- Easier to add type-specific attributes

**Edge collections to create:**
- `edge_works_for` - Person → Organization
- `edge_located_in` - Organization/Person → Location
- `edge_subject_to` - Entity → Law
- `edge_refers_to` - Entity → Entity (general reference)
- `edge_relates_to` - Entity → Entity (general relation)
- `edge_owner_of` - Person/Org → Entity
- `edge_reports_to` - Person → Person
- `edge_collaborates_with` - Entity ↔ Entity
- `edge_follows` - Entity → Entity
- `edge_part_of` - Entity → Entity

---

## Q3: Community Structure

**Question:** How should communities be structured?

**Answer:** Vertex + edges

**Rationale:** More flexible for:
- Hierarchical communities (parent → child edges)
- Dynamic membership (add/remove entities without updating community document)
- Future extensions (community metadata, scores)

**Structure:**
- `communities` vertex collection
- `edge_member_of` edge collection (entity → community)
- `edge_subcommunity` edge collection for hierarchy (community → community)

---

## Q4: Testing Approach

**Question:** How should we test the ArangoDB integration?

**Answer:** Testcontainers

**Implementation:**
- Use `testcontainers` crate with ArangoDB image
- Spin up real ArangoDB instance for integration tests
- Use `#[tokio::test]` and `#[ignore]` attributes
- Clean up collections between tests

**Benefits:**
- Real database behavior
- No mocking complexity
- Validates AQL queries work correctly

---

## Q5: Connection Pooling

**Question:** Should we use connection pooling for ArangoDB?

**Answer:** External pool

**Implementation:**
- Use `mobc-arangors` for connection pooling
- Configure pool size based on application needs
- Reuse connections across requests

**Benefits:**
- Better performance under load
- Efficient resource usage
- Production-ready

---

## Additional Technical Decisions

### Graph Definition

Create named graph `knowledge_graph` grouping all edge collections:

```javascript
{
  "edgeDefinitions": [
    {"collection": "edge_works_for", "from": ["persons"], "to": ["organizations"]},
    {"collection": "edge_located_in", "from": ["persons", "organizations"], "to": ["locations"]},
    // ... other edge collections
  ]
}
```

### Document Schema

Each document follows the existing struct:
- `_key` = `id` (Uuid)
- Preserve all fields from existing types
- Add ArangoDB metadata fields automatically

### Migration Strategy

- No database migrations needed (ArangoDB schemaless)
- Use application code to ensure collection/index existence
- Create collections on first use if they don't exist

### Error Handling

- Convert `arangors::ClientError` to domain-specific `StoreError`
- Preserve existing error handling patterns with `thiserror`
- Handle ArangoDB-specific errors (constraint violations, not found)

---

## Performance Considerations

### Indexes

**Vertex collections:**
- Persistent index on `name` (for search)
- Persistent index on `source_domain_id`
- Hash index on `canonical_name` (unique)

**Edge collections:**
- Edge indexes automatic on `_from`, `_to`
- Persistent index on `relationship_type`
- Persistent index on `source_domain_id`

### Query Optimization

- Use named graph in AQL for cleaner syntax
- Use `PRUNE` for early exit in traversals
- Use `uniqueVertices: 'global'` for large traversals
- Batch queries for large result sets

---

## Summary

| Aspect | Decision |
|--------|----------|
| Vertex collections | Separate per type (4 collections) |
| Edge collections | Separate per type (10+ collections) |
| Communities | Vertex + membership edges |
| Testing | Testcontainers with real ArangoDB |
| Connection pool | mobc-arangors |
| Graph definition | Named graph `knowledge_graph` |
| Schema | Schemaless with application validation |
