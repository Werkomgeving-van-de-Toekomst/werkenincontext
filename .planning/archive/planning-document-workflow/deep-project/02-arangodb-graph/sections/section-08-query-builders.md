# Section 8: AQL Query Builders

## Overview

This section implements safe AQL query construction functions using bind parameters. Provides centralized, reusable query builders for all graph operations.

## Dependencies

- Section 2 (Connection)
- Section 3 (Errors)

## Tests

All tests implemented in `crates/iou-core/src/graphrag/queries.rs`:
- `build_neighbors_aql_uses_bind_parameters` ✅
- `build_neighbors_aql_includes_filters` ✅
- `build_shortest_path_aql_uses_named_graph` ✅
- `build_traversal_aql_includes_depth_range` ✅
- `build_traversal_aql_includes_options` ✅
- `build_community_members_aql_inbound_direction` ✅

## Implementation

File: `crates/iou-core/src/graphrag/queries.rs`

### Functions Implemented

```rust
pub fn build_neighbors_aql(entity_id: Uuid, filters: &NeighborFilters) -> String;
pub fn build_shortest_path_aql(from: Uuid, to: Uuid) -> String;
pub fn build_traversal_aql(request: &TraversalRequest) -> String;
pub fn build_community_members_aql(community_id: Uuid) -> String;
```

All queries use bind parameters (`@param`) not string interpolation.

### Implementation Notes

1. **build_neighbors_aql**: Returns AQL that fetches edges and connected entities using subquery
2. **build_shortest_path_aql**: Template using SHORTEST_PATH with 'knowledge_graph' (requires named graph setup)
3. **build_traversal_aql**: Template using graph traversal with depth range and BFS options
4. **build_community_members_aql**: Joins edge_member_of with entity collections

These are utility functions that could be used to refactor store.rs methods in the future for better separation of concerns.
