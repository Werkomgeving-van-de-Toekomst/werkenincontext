# Transaction Support - Implementation Note

## Status: Deferred

Full transaction support using ArangoDB stream transactions is not implemented in this iteration.

## Why Deferred

ArangoDB stream transactions require:
1. Complex async API with begin/commit/rollback flow
2. Stream transaction cursor management
3. Proper error propagation and cleanup
4. The closure-based API pattern shown in the spec is complex

## Current Workaround

The current implementation uses:
- Individual operations with immediate commit
- `get_or_create_entity` handles race conditions via UniqueViolation handling
- Operations are atomic at the single-document level

## Future Implementation

To add full transaction support:
1. Add transaction method that accepts a closure
2. Begin stream transaction, execute operations, commit/rollback
3. Handle connection pool transaction boundaries
4. Add tests for commit/rollback/failure scenarios

For now, use operations that have built-in idempotency:
- `get_or_create_entity` for entities
- `upsert_entity` for updates
- Manual cleanup for partial failures
