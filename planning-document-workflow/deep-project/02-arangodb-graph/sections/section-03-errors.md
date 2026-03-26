Now I have all the context. Let me generate the section content for section-03-errors (Error Types).</think># Section 3: Error Types

## Overview

This section implements the centralized error handling system for the ArangoDB graph persistence layer. The `StoreError` enum provides type-safe, ergonomic error handling with proper conversion from ArangoDB client errors.

**Dependencies:** None (can be implemented in parallel with section-01-dependencies)

**Blocks:** section-04-entity-operations (entity operations need error types)

---

## Background

### Why Centralized Error Types

The graph store operations can fail in multiple ways:
- **Connection failures:** Database unavailable, authentication failed, network issues
- **Query failures:** Invalid AQL syntax, constraint violations, permission errors
- **Not found errors:** Entity/relationship/community doesn't exist
- **ArangoDB-specific errors:** Unique constraint violations, document update conflicts

A centralized error enum with `thiserror` provides:
1. **Ergonomic error handling** - `?` operator works seamlessly
2. **Descriptive error messages** - User-readable error descriptions
3. **Type safety** - Callers can match on specific error variants
4. **Automatic conversion** - `From<ClientError>` impl for seamless integration

---

## File Structure

**File to create:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/error.rs`

---

## Implementation

### StoreError Enum Definition

Define the error enum with variants for all failure modes:

```rust
use thiserror::Error;
use uuid::Uuid;

/// Centralized error type for graph store operations
#[derive(Debug, Error)]
pub enum StoreError {
    /// Connection or authentication error
    #[error("Connection error: {0}")]
    Connection(String),

    /// AQL query execution error
    #[error("Query error: {0}")]
    Query(String),

    /// Entity not found in database
    #[error("Entity not found: {0}")]
    EntityNotFound(Uuid),

    /// Relationship not found in database
    #[error("Relationship not found: {0}")]
    RelationshipNotFound(Uuid),

    /// Community not found in database
    #[error("Community not found: {0}")]
    CommunityNotFound(Uuid),

    /// Unique constraint violation (e.g., duplicate canonical_name)
    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    /// ArangoDB-specific error with error code and message
    #[error("ArangoDB error [{code}]: {message}")]
    Arango { code: u16, message: String },
}
```

### Conversion from arangors ClientError

Implement automatic conversion from `arangors::ClientError`:

```rust
use arangors::ClientError;

impl From<ClientError> for StoreError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::Connection(msg) => StoreError::Connection(msg),
            ClientError::Query(msg) => StoreError::Query(msg),
            ClientError::Arango(code, message) => StoreError::Arango { code, message },
            // Other variants are converted to Query errors as a catch-all
            _ => StoreError::Query(err.to_string()),
        }
    }
}
```

**Note:** The actual `ClientError` enum variants may differ. Adjust the match arms based on the actual `arangors` crate error definition.

---

## Tests

### Test Stubs (from TDD Plan)

Write these tests in `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/error.rs` (in a `#[cfg(test)]` module):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use arangors::ClientError;

    #[test]
    fn error_from_client_error_connection() {
        let client_error = ClientError::Connection("connection refused".to_string());
        let store_error: StoreError = client_error.into();
        
        assert!(matches!(store_error, StoreError::Connection(_)));
        assert_eq!(store_error.to_string(), "Connection error: connection refused");
    }

    #[test]
    fn error_from_client_error_query() {
        let client_error = ClientError::Query("syntax error".to_string());
        let store_error: StoreError = client_error.into();
        
        assert!(matches!(store_error, StoreError::Query(_)));
        assert_eq!(store_error.to_string(), "Query error: syntax error");
    }

    #[test]
    fn error_from_client_error_arango() {
        let client_error = ClientError::Arango(1200, "duplicate key".to_string());
        let store_error: StoreError = client_error.into();
        
        assert!(matches!(store_error, StoreError::Arango { code: 1200, .. }));
        assert_eq!(store_error.to_string(), "ArangoDB error [1200]: duplicate key");
    }

    #[test]
    fn error_display_formats_correctly() {
        let err = StoreError::EntityNotFound(Uuid::new_v4());
        let display = format!("{}", err);
        
        assert!(display.contains("Entity not found"));
    }
}
```

---

## Module Integration

After implementing this section, the error module will be re-exported in section-09-module-exports:

```rust
// In graphrag/mod.rs (implemented in section-09)
pub mod error;

pub use error::StoreError;
```

---

## Implementation Checklist

- [ ] Create `crates/iou-core/src/graphrag/error.rs`
- [ ] Define `StoreError` enum with all variants
- [ ] Derive `Debug` and `thiserror::Error` traits
- [ ] Implement `From<ClientError> for StoreError`
- [ ] Write all test stubs
- [ ] Run `cargo test --package iou-core` to verify
- [ ] Verify error messages are descriptive and user-readable

---

## Usage Example

After implementation, error handling in store operations will look like:

```rust
use crate::graphrag::{StoreError, GraphStore};

impl GraphStore {
    pub async fn get_entity(&self, id: Uuid) -> Result<Option<Entity>, StoreError> {
        // ... query logic ...
        
        // If document not found:
        Ok(None)  // Returns Ok(None) without error
        
        // On query failure:
        Err(ClientError::Query("...".into()))?  // Automatically converts to StoreError::Query
    }
}
```

---

## Notes

- **No external dependencies beyond `thiserror`** (already in project dependencies)
- **Zero-cost abstractions** - Error enum has no runtime overhead
- **Thread-safe** - All error types are `Send + Sync`
- **Future-proof** - Easy to add new error variants without breaking existing code
---

## Implementation Status

**COMPLETED** - 2026-03-25

### Files Created/Modified

1. **Created:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/error.rs`
   - `StoreError` enum with 10 variants: `Connection`, `Query`, `EntityNotFound`, `RelationshipNotFound`, `CommunityNotFound`, `UniqueViolation`, `Arango`, `HttpClient`, `PermissionDenied`, `Serialization`, `InvalidServer`
   - `#[non_exhaustive]` attribute for future-proofing
   - `From<arangors::ClientError>` implementation

2. **Created:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/mod.rs`
   - Module declaration and re-exports

3. **Moved:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag.rs` → `/Users/marc/Projecten/iou-modern/crates/iou-core/src/graphrag/types.rs`
   - Existing type definitions moved to subdirectory

### Test Results

All 17 tests passing:
- Display format tests for all error variants
- Pattern matching tests
- Send + Sync bounds verification
- Conversion tests for HttpClient and Serde variants

### Code Review Findings

- Added `#[non_exhaustive]` attribute based on code review feedback
- Helper methods (`is_retryable`, `is_not_found`) deferred for now
- Permission debug formatting noted as aesthetic improvement only

### Verification

```bash
cargo test --package iou-core --lib graphrag::error
# test result: ok. 17 passed; 0 failed
```
