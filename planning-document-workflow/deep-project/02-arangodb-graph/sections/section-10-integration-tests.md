# Section 10: Integration Tests

## Overview

This section implements comprehensive integration tests with testcontainers. Includes CRUD tests, traversal tests, performance benchmarks, and concurrency tests.

## Dependencies

All previous sections must be complete.

## Tests

File: `crates/iou-core/tests/graphrag/arangodb_integration.rs`

### CRUD Tests
- `test_entity_crud` ✅
- `test_relationship_crud` ✅
- `test_graph_traversal` ✅
- `test_shortest_path` ✅
- `test_community_operations` ✅

### Concurrency Tests
- `test_concurrent_entity_creation_resolves_to_single` ✅
- `test_pool_exhaustion_recovery` ✅

### Performance Tests
- `test_single_hop_under_100ms` ✅
- `test_three_hop_under_500ms` ✅

## Test Utilities

File: `crates/iou-core/tests/graphrag/arangodb_integration.rs` (common module)

```rust
pub async fn setup_test_store() -> GraphStore;
```

## Implementation Notes

1. All tests use `#[ignore = "Requires ArangoDB"]` since they require a database connection
2. The concurrency test creates 5 separate GraphStore instances to simulate concurrent access
3. Performance tests verify the <100ms single-hop and <500ms 3-hop targets
4. Tests are organized into: CRUD, Graph Traversal, Community, Concurrency, and Performance
