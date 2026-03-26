# Section 6: Graph Traversals

## Overview

This section implements graph traversal operations for multi-hop queries, shortest path finding, and neighbor discovery.

**Note on Implementation:** This section uses client-side graph algorithms (BFS) and edge collection queries. For production use with large graphs, ArangoDB's native graph traversal features with a named graph should be used instead.

## Performance Targets

- Single-hop: <100ms
- 3-hop: <500ms

## Dependencies

- Section 2 (Connection)
- Section 3 (Errors)
- Section 4 (Entity Operations)

## Tests

All tests implemented in `crates/iou-core/tests/graphrag/graph_traversals.rs`:
- `find_shortest_path_returns_path` ✅
- `find_shortest_path_no_path_returns_none` ✅
- `find_shortest_path_performance_under_100ms` ✅
- `traverse_one_hop_returns_immediate_neighbors` ✅
- `traverse_three_hops_returns_connected_entities` ✅
- `traverse_with_filters_filters_correctly` ✅
- `traverse_performance_under_500ms` ✅
- `get_neighbors_returns_connected_entities` ✅
- `get_neighbors_filters_by_type` ✅
- `get_neighbors_incoming_only` ✅
- `get_neighbors_filters_by_confidence` ✅

## Implementation

File: `crates/iou-core/src/graphrag/store.rs`

### Methods Implemented

```rust
impl GraphStore {
    /// Find shortest path using BFS algorithm
    /// Max depth: 6 hops to prevent infinite loops
    pub async fn find_shortest_path(&self, from: Uuid, to: Uuid) -> Result<Option<GraphPath>, StoreError>;

    /// Multi-hop traversal with depth tracking
    /// Respects min_depth and max_depth parameters
    /// Uses subqueries to fetch vertices with edges (reduces N+1 queries)
    pub async fn traverse(&self, request: TraversalRequest) -> Result<TraversalResult, StoreError>;

    /// Get immediate neighbors with single query
    /// Uses AQL subquery to fetch connected entities
    pub async fn get_neighbors(&self, entity_id: Uuid, filters: NeighborFilters) -> Result<Vec<Neighbor>, StoreError>;
}
```

### Types Added

- `GraphPath` - Result of shortest path query
- `TraversalRequest` - Request with start_id, min_depth, max_depth, direction, limit
- `TraversalResult` - Result with vertices and edges
- `Neighbor` - Combined entity and relationship data
- `NeighborFilters` - Filter options (direction, types, confidence, limit)
- `TraversalDirection` - Outgoing, Incoming, Any

### Implementation Notes

1. **find_shortest_path**: Uses client-side BFS because ArangoDB's SHORTEST_PATH requires a named graph which hasn't been set up. Max depth limited to 6 hops.

2. **traverse()**: Iterative depth-by-depth traversal that tracks visited entities to prevent cycles. Uses subquery to fetch entity data with edge data.

3. **get_neighbors()**: Single query with AQL subquery to fetch connected entities, avoiding N+1 query problem.

4. **TraversalDirection::Any**: Correctly queries both source_entity_id and target_entity_id.

### Future Improvements

1. Create named graph 'knowledge_graph' in ArangoDB
2. Use native SHORTEST_PATH AQL for shortest path
3. Use proper graph traversal syntax with named graphs
4. Move filters from Rust code to AQL WHERE clauses
5. Add performance tests with larger datasets
