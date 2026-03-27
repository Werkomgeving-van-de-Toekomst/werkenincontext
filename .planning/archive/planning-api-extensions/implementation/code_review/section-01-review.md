# Code Review: Section-01-Foundation

## Critical Issues (Confidence ≥ 80)

### 1. Secret Key Discarded - Critical Security Bug
**Confidence: 95**
**File:** `crates/iou-core/src/storage/s3.rs:61-74`

The secret key is read but immediately discarded, then replaced with `"***"` placeholder. This means S3 authentication will fail in Section 4.

```rust
let _secret_key = env::var("S3_SECRET_KEY")  // Discarded!
// ...
secret_key: "***".to_string(),  // Placeholder!
```

**Fix:** Store the actual secret key.

---

### 2. Unsafe Code Unnecessary - Rust Best Practice Violation
**Confidence: 90**
**File:** `crates/iou-core/src/storage/s3.rs:113-115`

Using `unsafe` blocks to call `std::env::remove_var` is completely unnecessary.

```rust
unsafe { std::env::remove_var("S3_ACCESS_KEY") };
```

**Fix:** Remove the `unsafe` blocks.

---

### 3. Secret Key Exposed in Logs - Security Vulnerability
**Confidence: 85**
**File:** `crates/iou-api/src/config.rs:44-46`

The S3Config doesn't redact the secret in Debug output. Also uses `unwrap_or_default()` which accepts empty credentials without validation.

**Fix:** Implement custom Debug to redact secret, add validation.

---

### 4. Duplicate S3Config Types - Design Issue
**Confidence: 95**

Two separate `S3Config` types exist in `iou-core` and `iou-api`. This causes confusion and conversion boilerplate.

**Fix:** Re-export `S3Config` from `iou-core`.

---

## Important Issues

### 5. WebSocket Handler Lacks TODO
**Confidence: 85**
**File:** `crates/iou-api/src/websockets/documents.rs:37-44`

The stub immediately closes connections but has no TODO marker indicating incomplete implementation.

---

### 6. ConnectionLimiter Memory Leak
**Confidence: 80**
**File:** `crates/iou-api/src/websockets/limiter.rs:52-63`

The `remove()` method exists but is never called automatically. Stale entries accumulate.

**Fix:** Implement automatic cleanup in `acquire()`.

---

## Summary

**Critical Issues:** 3 (secret key handling, unsafe blocks, log exposure)
**Important Issues:** 3 (duplicate types, missing TODO, memory leak)
**Overall Code Quality:** Good test coverage, clear module structure, proper async patterns
