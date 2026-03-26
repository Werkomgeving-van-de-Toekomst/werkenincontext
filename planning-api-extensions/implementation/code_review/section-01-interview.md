# Code Review Interview: Section-01-Foundation

## Date
2026-03-10

## Review Summary
The code review identified 6 issues across critical and important categories.

---

## Issue 1: Secret Key Placeholder (CRITICAL)
**Decision: DEFER to Section 4**

The review noted that the secret key is replaced with `"***"` placeholder in the stub.
This is intentional for the foundation section - the actual S3 implementation with
proper credential handling will be completed in Section 4.

**Rationale:** This is stub code. The test validates environment variable parsing,
not actual authentication. Section 4 will implement the real S3 client.

---

## Issue 2: Unsafe Blocks in Tests (CRITICAL)
**Decision: KEEP (Required by Rust 2024)**

The review suggested removing `unsafe` blocks from test code. However, in Rust 2024
edition, `std::env::remove_var` is actually unsafe, so the blocks are required.

**Action:** Keep unsafe blocks with comment explaining Rust 2024 requirement.

---

## Issue 3: Secret in Debug Logs (IMPORTANT)
**Decision: DEFER to Section 4**

The S3Config doesn't redact secrets in Debug output. This will be addressed when
real S3 operations are implemented in Section 4.

**Rationale:** Current config uses placeholder values, not real secrets.

---

## Issue 4: Duplicate S3Config Types (IMPORTANT)
**Decision: CONSOLIDATE (user approved)**

User chose to consolidate S3Config types by re-exporting from `iou-core` instead
of maintaining duplicate definitions in `iou-api`.

**Action:** Modify `iou-api/src/config.rs` to use `iou_core::storage::S3Config`.

---

## Issue 5: Missing TODO Comment (IMPORTANT)
**Decision: AUTO-FIX**

Add a prominent TODO comment to the WebSocket handler stub to indicate incomplete
implementation.

**Action:** Add TODO comment explaining Section 5 implementation.

---

## Issue 6: ConnectionLimiter Memory Leak (IMPORTANT)
**Decision: DEFER to Section 5**

The ConnectionLimiter doesn't auto-cleanup entries. Since full WebSocket
implementation happens in Section 5, cleanup logic can be added then.

**Rationale:** Premature optimization for stub code.

---

## Actions to Apply

1. **Remove unsafe blocks** from `crates/iou-core/src/storage/s3.rs` test
2. **Add TODO comment** to `crates/iou-api/src/websockets/documents.rs`
3. **Consolidate S3Config** - update `iou-api/src/config.rs` to re-export from core
