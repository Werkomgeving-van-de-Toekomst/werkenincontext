# Section 1: Dependencies and Setup

## Overview

This section establishes the foundation for ArangoDB graph persistence by adding the necessary Rust dependencies. The primary dependencies are:

- **arangors**: ArangoDB client library for database operations
- **mobc-arangors**: Connection pooling for efficient database connections
- **testcontainers**: Integration testing with real ArangoDB instances

## Dependencies to Add

### File: `/Users/marc/Projecten/iou-modern/crates/iou-core/Cargo.toml`

Add the following dependencies to the `[dependencies]` section:

```toml
# ArangoDB client
arangors = { version = "0.6", features = ["reqwest_async", "rocksdb"] }

# Connection pooling for ArangoDB
mobc-arangors = "0.2"
```

Add the following to the `[dev-dependencies]` section:

```toml
# Testcontainers for integration testing
testcontainers = "0.15"
```

### Dependency Rationale

| Dependency | Purpose | Key Features |
|------------|---------|--------------|
| `arangors` | Official ArangoDB Rust client | Async support, AQL queries, connection management |
| `mobc-arangors` | Connection pool adapter | Manages connection lifecycle, prevents exhaustion |
| `testcontainers` | Docker-based test environments | Spins up real ArangoDB for integration tests |

## Verification Tests

### Test: Dependencies Compile

After adding dependencies, verify the project compiles:

```bash
cargo check --package iou-core
```

Expected result: No errors, dependencies resolve successfully.

### Test: Arangors Version Check

Verify arangors 0.6 provides required functionality:

- AQL query execution
- Document CRUD operations
- Graph traversal support
- Async API with reqwest client

### Test: Mobc Compatibility

Verify mobc-arangors 0.2 works with arangors 0.6:

- Connection manager implementation
- Pool configuration options
- Connection reuse behavior

## Compatibility Notes

### Tokio Runtime

The project already uses `tokio` with `full` features via workspace dependencies. Arangors requires an async runtime, which is satisfied by the existing configuration.

### Reqwest Client

The `arangors` dependency uses `reqwest_async` feature, which aligns with the project's existing `reqwest` workspace dependency (version 0.11).

### Existing Database Dependencies

The project already uses `sqlx` for PostgreSQL. The ArangoDB additions are complementary and do not conflict:

- PostgreSQL continues to handle audit logging and relational data
- ArangoDB will handle graph operations via the new `graphrag` module

## Next Steps

Once dependencies are verified:

1. Proceed to **Section 2: Connection Module** (`section-02-connection.md`) - implements the connection pooling infrastructure
2. Parallel work can begin on **Section 3: Error Types** (`section-03-errors.md`) - defines error handling

## Section Dependencies

This section has no dependencies on other implementation sections. It can be implemented immediately.

## Implementation Status

**COMPLETED** - 2025-03-25

Added dependencies to `crates/iou-core/Cargo.toml`:
- `arangors = { version = "0.6", features = ["reqwest_async", "rocksdb"] }`
- `mobc-arangors = "0.2"`
- `testcontainers = "0.15"` (dev-dependency)

Verification: `cargo check --package iou-core` completed successfully with no errors.

## Blocks

This section blocks:
- `section-02-connection` (requires arangors and mobc-arangors)
- All subsequent sections that use ArangoDB functionality