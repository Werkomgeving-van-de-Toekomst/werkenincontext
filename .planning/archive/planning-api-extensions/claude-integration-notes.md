# External Review Integration Notes

## Review Source
- **Model:** Claude Opus 4.6
- **File:** `reviews/iteration-1-opus.md`
- **Date:** 2026-03-10

## Summary
The Opus review identified several critical architectural issues, particularly around duplicate state machine implementation, security concerns, and missing edge cases. Below is the integration decision log.

---

## Integrated Suggestions

### 1. CRITICAL: Use Existing `iou-orchestrator` Instead of Creating Duplicate Executor

**Recommendation:** Integrate the existing `iou-orchestrator` crate instead of creating a new `workflows/executor.rs`.

**Integration:** **ACCEPTED** - This is the most significant architectural finding. The plan will be updated to:
- Add `iou-orchestrator` as a dependency to `iou-api`
- Create an `orchestrator` module in `iou-api` that wraps the existing `WorkflowStateMachine`
- Remove the duplicate executor creation from Section 2
- Update Section 2 to focus on orchestrator integration instead

### 2. Fix Role Matching Syntax Bug (Section 3)

**Recommendation:** The pattern `matches!(r, Role::ObjectCreator | Role::DomainEditor)` doesn't work correctly in the `any()` context.

**Integration:** **ACCEPTED** - Will fix to:
```rust
let has_permission = auth.roles.iter()
    .any(|r| *r == Role::ObjectCreator || *r == Role::DomainEditor);
```

### 3. Add S3 Credential Validation at Startup

**Recommendation:** Validate S3 connectivity during application startup rather than failing on first request.

**Integration:** **ACCEPTED** - Add to Section 1:
- S3 connection check during app initialization
- Fail fast if credentials are invalid
- Use structured config instead of direct env var access

### 4. Add Organization-Based Access Control

**Recommendation:** Use `organization_id` from `AuthContext` for data access scoping.

**Integration:** **ACCEPTED** - Add to Section 3:
- Verify user can access documents in their organization only
- Add organization scoping to document queries

### 5. Add Idempotency for Document Creation

**Recommendation:** Use document_id as idempotency key to handle duplicate requests.

**Integration:** **ACCEPTED** - Add to Section 2:
- Check for existing document_id before creating
- Return existing document if already in progress

### 6. WebSocket Connection Limits and Cleanup

**Recommendation:** Add connection limit per document and proper cleanup on abnormal closure.

**Integration:** **ACCEPTED** - Add to Section 5:
- Max 10 connections per document_id
- Drop guards for cleanup
- Panic recovery in send/receive tasks

### 7. Streaming Download Instead of Buffering

**Recommendation:** Stream S3 responses directly to HTTP client instead of loading entire document into memory.

**Integration:** **ACCEPTED** - Modify Section 4:
- Use `tokio::io::copy` or similar for streaming
- Avoid buffering large documents in memory

### 8. Add Bounded Channel for WebSocket Broadcast

**Recommendation:** Use bounded broadcast channel to prevent memory growth.

**Integration:** **ACCEPTED** - Change to `tokio::sync::broadcast::channel(100)`

### 9. Remove Synchronous Mode Ambiguity

**Recommendation:** Clarify or remove synchronous execution mode.

**Integration:** **ACCEPTED** - Remove synchronous mode. The API will be async-only with WebSocket status updates. This simplifies the implementation and avoids ambiguity.

### 10. Fix Timeout Implementation

**Recommendation:** The current timeout only applies per-agent, not the overall pipeline.

**Integration:** **ACCEPTED** - Track cumulative execution time and apply overall timeout.

---

## Not Integrated Suggestions

### 1. S3 Client Library Choice

**Recommendation:** Consider using `aws-sdk-s3` instead of `rust-s3`.

**Decision:** **DEFER** - Keep `rust-s3` for now as it's lighter weight and the project may already have it. Can swap later if issues arise through the storage abstraction trait.

### 2. Multipart Upload for Large Files

**Recommendation:** Add multipart upload for files >5MB.

**Decision:** **DEFER** - The 10MB limit is small enough that multipart upload adds unnecessary complexity. Add later if limit is increased.

### 3. WebSocket Sequence Numbers

**Recommendation:** Add sequence numbers to allow resume from disconnect.

**Decision:** **DEFER** - Nice to have but not critical for v1. Messages can be replayed by requesting current status on reconnection.

### 4. Testcontainers for Integration Tests

**Recommendation:** Use testcontainers for DuckDB/S3 setup.

**Decision:** **DEFER** - Will note in testing section but keep tests simple for initial implementation. Can enhance in follow-up work.

### 5. OpenAPI Documentation Updates

**Recommendation:** Document all new endpoints with proper schemas.

**Decision:** **DEFER** - Important but out of scope for this implementation plan. Can be added as documentation task.

---

## Modified Plan Sections

The following sections of `claude-plan.md` will be updated based on the integrated feedback:

1. **Section 1** - Add S3 validation at startup
2. **Section 2** - Replace new executor with orchestrator integration, add idempotency
3. **Section 3** - Fix role check bug, add organization scoping
4. **Section 4** - Implement streaming download
5. **Section 5** - Add connection limits, bounded channels, proper cleanup
6. **Section 6** - Add testing infrastructure notes

The synchronous mode feature is removed entirely to simplify the implementation.
