# TDD Plan: ArangoDB Graph Persistence

This document defines test stubs for each implementation section. Tests should be written BEFORE implementing the corresponding functionality.

---

## Section 1: Dependencies and Setup

### Test Stubs
- **Test:** `dependencies_compile` - Verify all dependencies compile without errors
- **Test:** `arangors_version_check` - Verify compatible arangors version
- **Test:** `mobc_compatibility` - Verify mobc-arangors works with chosen arangors version

---

## Section 2: Connection Module

### Test Stubs
- **Test:** `connection_establish_jwt` - JWT authentication creates connection
- **Test:** `connection_establish_basic_auth` - Basic auth creates connection
- **Test:** `connection_failure_invalid_credentials` - Invalid credentials return error
- **Test:** `connection_failure_unreachable_host` - Unreachable host returns error
- **Test:** `pool_create` - Pool is created with specified size
- **Test:** `pool_connection_reuse` - Pool reuses connections
- **Test:** `pool_connection_exhaustion` - Pool waits when max connections reached

---

## Section 3: Error Types

### Test Stubs
- **Test:** `error_from_client_error_connection` - ClientError::Connection converts to StoreError::Connection
- **Test:** `error_from_client_error_query` - ClientError::Query converts to StoreError::Query
- **Test:** `error_from_client_error_arango` - ClientError::Arango converts to StoreError::Arango with code/message
- **Test:** `error_display_formats_correctly` - Error Display impl shows readable message

---

## Section 4: GraphStore - Entity Operations

### Test Stubs
- **Test:** `create_entity_returns_entity_with_id` - Created entity has generated/assigned ID
- **Test:** `create_entity_in_correct_collection` - Person entities go to `persons` collection
- **Test:** `get_entity_found_returns_data` - Existing entity returns populated Entity
- **Test:** `get_entity_not_found_returns_none` - Missing entity returns Ok(None)
- **Test:** `update_entity_modifies_fields` - Updated entity reflects changes
- **Test:** `update_entity_not_found_returns_error` - Updating non-existent entity returns error
- **Test:** `delete_entity_removes_document` - Deleted entity no longer accessible
- **Test:** `delete_entity_cascade_true_removes_edges` - Cascade deletes all related edges
- **Test:** `delete_entity_cascade_false_keeps_edges` - Non-cascade leaves edges (orphaned)
- **Test:** `list_entities_without_filters_returns_all` - No filters returns all entities
- **Test:** `list_entities_with_type_filter_filters_correctly` - Type filter works
- **Test:** `list_entities_with_pagination_returns_page` - Pagination limit/offset works
- **Test:** `get_or_create_entity_new_creates` - Non-existent entity is created
- **Test:** `get_or_create_entity_existing_returns` - Existing entity is returned (no duplicate)
- **Test:** `upsert_entity_new_creates` - Upsert creates new entity
- **Test:** `upsert_entity_existing_updates` - Upsert updates existing entity
- **Test:** `concurrent_entity_creation_resolves_to_single` - Concurrent `get_or_create` returns same entity

---

## Section 5: GraphStore - Relationship Operations

### Test Stubs
- **Test:** `create_relationship_creates_in_correct_edge_collection` - WorksFor goes to `edge_works_for`
- **Test:** `create_relationship_prevents_duplicates` - Same source/target/type returns error
- **Test:** `get_relationship_found_returns_data` - Existing relationship returned
- **Test:** `get_relationship_not_found_returns_none` - Missing relationship returns None
- **Test:** `delete_relationship_removes_document` - Deleted relationship no longer accessible
- **Test:** `get_entity_relationships_returns_both_directions` - Returns incoming and outgoing
- **Test:** `get_entity_relationships_filters_by_type` - Type filter works
- **Test:** `get_entity_relationships_paginates_correctly` - Pagination works

---

## Section 6: GraphStore - Graph Traversals

### Test Stubs
- **Test:** `find_shortest_path_returns_path` - Path exists between connected entities
- **Test:** `find_shortest_path_no_path_returns_none` - Disconnected entities return None
- **Test:** `find_shortest_path_performance_under_100ms` - Single-hop <100ms
- **Test:** `traverse_one_hop_returns_immediate_neighbors` - 1-hop returns direct connections
- **Test:** `traverse_three_hops_returns_connected_entities` - 3-hop returns network
- **Test:** `traverse_with_filters_filters_correctly` - Filters apply during traversal
- **Test:** `traverse_performance_under_500ms` - 3-hop traversal <500ms
- **Test:** `get_neighbors_returns_connected_entities` - Neighbors query works

---

## Section 7: GraphStore - Community Operations

### Test Stubs
- **Test:** `create_community_creates_community_vertex` - Community document created
- **Test:** `create_community_creates_membership_edges` - Member edges created
- **Test:** `get_community_found_returns_with_members` - Community includes member list
- **Test:** `get_community_not_found_returns_none` - Missing community returns None
- **Test:** `add_community_member_creates_edge` - Membership edge created
- **Test:** `add_community_member_prevents_duplicates` - Duplicate membership returns false
- **Test:** `remove_community_member_removes_edge` - Membership edge removed
- **Test:** `remove_community_member_non_member_returns_error` - Removing non-member errors
- **Test:** `get_community_members_returns_all_members` - All members returned

---

## Section 8: AQL Query Builders

### Test Stubs
- **Test:** `build_neighbors_aql_uses_bind_parameters` - No string interpolation, uses @params
- **Test:** `build_neighbors_aql_includes_filters` - Filters correctly included in AQL
- **Test:** `build_shortest_path_aql_uses_named_graph` - GRAPH clause present
- **Test:** `build_traversal_aql_includes_depth_range` - Min/max depth in query
- **Test:** `build_traversal_aql_includes_options` - BFS and uniqueVertices options set
- **Test:** `build_community_members_aql_inbound_direction` - INBOUND for members

---

## Section 9: Module Exports

### Test Stubs
- **Test:** `module_exports_graphstore` - GraphStore is publicly accessible
- **Test:** `module_exports_storeerror` - StoreError is publicly accessible
- **Test:** `module_exports_arangoconfig` - ArangoConfig is publicly accessible
- **Test:** `module_exports_preserves_existing_types` - Entity, Relationship, etc. still accessible
- **Test:** `module_does_not_export_internal_types` - Internal types not leaked

---

## Section 10: Integration Tests

### Test Stubs
- **Test:** `test_entity_crud` - Full CRUD cycle for entity
- **Test:** `test_relationship_crud` - Full CRUD cycle for relationship
- **Test:** `test_graph_traversal` - End-to-end traversal query
- **Test:** `test_shortest_path` - End-to-end shortest path query
- **Test:** `test_community_operations` - Full community lifecycle
- **Test:** `test_concurrent_entity_creation` - Concurrent creation resolves correctly
- **Test:** `test_pool_exhaustion_recovery` - Pool recovers from exhaustion
- **Test:** `test_single_hop_under_100ms` - Performance benchmark
- **Test:** `test_three_hop_under_500ms` - Performance benchmark

---

## Section 11: Transaction Support

### Test Stubs
- **Test:** `transaction_commit_persists_changes` - Committed transaction persists
- **Test:** `transaction_rollback_discards_changes` - Rolled back transaction discards
- **Test:** `transaction_failure_returns_error` - Transaction error returns StoreError
- **Test:** `transaction_nested_not_supported` - Nested transactions return error (if applicable)

---

## Section 12: Bulk Operations

### Test Stubs
- **Test:** `bulk_create_entities_creates_all` - All entities created
- **Test:** `bulk_create_entities_returns_ids` - Returns IDs of created entities
- **Test:** `bulk_create_entities_performance_1000_per_sec` - Performance target met
- **Test:** `bulk_create_relationships_creates_all` - All relationships created
- **Test:** `bulk_delete_entities_removes_all` - All specified entities deleted
- **Test:** `bulk_delete_entities_cascades_to_edges` - Related edges also deleted

---

## Section 13: Migration Strategy

### Test Stubs
- **Test:** `migration_entity_counts_match` - Compare PostgreSQL vs ArangoDB counts
- **Test:** `migration_relationship_counts_match` - Compare relationship counts
- **Test:** `migration_sample_data_matches` - Random sample matches between systems

---

## Performance Benchmarks

### Target Metrics
- Single-hop traversal: <100ms
- 3-hop traversal: <500ms
- Bulk insert: 1000 entities/sec

### Test Stubs
- **Test:** `benchmark_single_hop_p99_under_100ms` - P99 latency <100ms
- **Test:** `benchmark_three_hop_p99_under_500ms` - P99 latency <500ms
- **Test:** `benchmark_bulk_insert_throughput` - 1000 entities/sec minimum
