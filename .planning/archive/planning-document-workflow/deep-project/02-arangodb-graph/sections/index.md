<!-- PROJECT_CONFIG
runtime: rust-cargo
test_command: cargo test --package iou-core
END_PROJECT_CONFIG -->

<!-- SECTION_MANIFEST
section-01-dependencies
section-02-connection
section-03-errors
section-04-entity-operations
section-05-relationship-operations
section-06-graph-traversals
section-07-communities
section-08-query-builders
section-09-module-exports
section-10-integration-tests
section-11-transactions
section-12-bulk-operations
section-13-migration
END_MANIFEST -->

# Implementation Sections Index

## Dependency Graph

| Section | Depends On | Blocks | Parallelizable |
|---------|------------|--------|----------------|
| section-01-dependencies | - | 02 | Yes |
| section-02-connection | 01 | 03, 04 | No |
| section-03-errors | - | 04 | Yes |
| section-04-entity-operations | 02, 03 | 05 | No |
| section-05-relationship-operations | 02, 03, 04 | 06 | No |
| section-06-graph-traversals | 02, 03, 04 | 07 | No |
| section-07-communities | 02, 03, 04 | - | No |
| section-08-query-builders | 02, 03 | 06 | No |
| section-09-module-exports | all | 10 | No |
| section-10-integration-tests | all | - | No |
| section-11-transactions | 04, 05, 07 | - | No |
| section-12-bulk-operations | 04, 05 | - | No |
| section-13-migration | all | - | No |

## Execution Order

**Sequential execution** (each section depends on the previous):

1. **Batch 1 (Parallel):**
   - section-01-dependencies (no dependencies)
   - section-03-errors (no dependencies)

2. **After 01 & 03:**
   - section-02-connection (requires 01)

3. **After 02:**
   - section-04-entity-operations (requires 02, 03)
   - section-05-relationship-operations (requires 02, 03, 04)
   - section-06-graph-traversals (requires 02, 03, 04)
   - section-07-communities (requires 02, 03, 04)
   - section-08-query-builders (requires 02, 03)

4. **After 04, 05, 06, 07, 08:**
   - section-09-module-exports (requires all previous)

5. **After 09:**
   - section-10-integration-tests (requires all modules)
   - section-11-transactions (requires 04, 05, 07)
   - section-12-bulk-operations (requires 04, 05)

6. **Final:**
   - section-13-migration (requires all)

## Section Summaries

### section-01-dependencies
Adds arangors, mobc-arangors, and testcontainers to Cargo.toml. Verifies version compatibility.

### section-02-connection
Implements ArangorsConnectionManager and connection pooling with mobc. Handles JWT authentication and connection lifecycle.

### section-03-errors
Defines StoreError enum with thiserror for all database operations. Implements conversion from arangors ClientError.

### section-04-entity-operations
Implements GraphStore struct with entity CRUD operations. Includes create, read, update, delete (with cascade), list (with pagination), and upsert patterns. Routes entities to type-specific collections.

### section-05-relationship-operations
Implements relationship CRUD operations. Routes relationships to type-specific edge collections. Handles duplicate prevention and pagination.

### section-06-graph-traversals
Implements graph traversal methods: shortest_path, traverse, get_neighbors. Uses AQL GRAPH syntax with performance optimizations (BFS, uniqueVertices).

### section-07-communities
Implements community management with vertex + edges approach. Supports hierarchical communities and dynamic membership.

### section-08-query-builders
Implements AQL query builder functions using bind parameters (not string interpolation). Provides safe, reusable query construction for all graph operations.

### section-09-module-exports
Updates graphrag/mod.rs to export new modules and types. Preserves existing Entity, Relationship, Community types for backward compatibility.

### section-10-integration-tests
Comprehensive integration tests with testcontainers. Includes CRUD tests, traversal tests, performance benchmarks, and concurrency tests.

### section-11-transactions
Implements transaction support using ArangoDB stream transactions. Provides atomic multi-step operations for complex workflows.

### section-12-bulk-operations
Implements bulk create/delete operations for entities and relationships. Targets 1000 entities/sec throughput for large-scale data import.

### section-13-migration
Documents migration strategy from PostgreSQL to ArangoDB. Includes dual-write phase, read migration, cutover plan, and validation queries.
