# Code Review: Section 00 - Feasibility Spike

**Agent:** feature-dev:code-reviewer
**Date:** 2026-03-16
**Files Changed:** 9 files, 856 insertions

---

## Summary

Well-structured feasibility spike that validates external dependencies for stakeholder extraction. The code demonstrates good Rust practices overall, but there are several areas needing improvement before production use.

---

## CRITICAL Issues (Must Fix)

### 1. Inconsistent Error Handling Pattern - Missing Custom Error Type
**Confidence: 95/100**

**Location:** All modules in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/`

**Issue:** The codebase uses `thiserror` for error handling (see `LlmError` and `PipelineError`), but the stakeholder modules use `Result<String, String>` and `Option<&str>` instead of proper error types.

**Evidence:**
- `document_probe.rs`: `pub fn verify_document_structure(...) -> Result<String, String>`
- `fallback_dict.rs`: `pub fn get_fallback_canonical_name(...) -> Option<&'static str>`
- `rijksoverheid_api_probe.rs`: Returns `ApiProbeResult` but no error type

**Fix:** Create a stakeholder error type using `thiserror`:

```rust
// Create stakeholder/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StakeholderError {
    #[error("Document content is empty")]
    EmptyDocument,

    #[error("API request failed: {0}")]
    ApiError(#[from] reqwest::Error),

    #[error("Organization not found: {0}")]
    OrganizationNotFound(String),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, StakeholderError>;
```

---

### 2. Missing Timeout and Retry Logic for API Calls
**Confidence: 90/100**

**Location:** `rijksoverheid_api_probe.rs`

**Issue:** The API probe has a 10-second timeout, but no retry logic for transient failures. The existing codebase has error severity classification which is ignored here.

**Fix:** Add retry logic with exponential backoff for transient failures.

---

### 3. Duplicate Key in HashMap - Data Integrity Bug
**Confidence: 100/100**

**Location:** `fallback_dict.rs`

**Issue:** "BZK" is inserted twice with different values. The second insert silently overwrites the first, and there's a misleading comment about "Ministry for Housing and Spatial Planning."

**Fix:** Remove duplicate entry or fix the mapping.

---

## IMPORTANT Issues (Should Fix)

### 4. Unused Dependency - `lazy_static` Not Used
**Confidence: 100/100**

The diff adds `once_cell` but the existing `lazy_static` dependency in Cargo.toml is unused in the new code. Consider removing or migrating.

---

### 5. Inconsistent Public API Design
**Confidence: 85/100**

`FALLBACK_DICT` is exported but shouldn't be public API. Users should use `get_fallback_canonical_name` instead.

---

### 6. Missing Documentation for Cost Estimator Constants
**Confidence: 85/100**

Cost model has hardcoded pricing that will become outdated. No clear guidance on when to update them.

---

### 7. Floating Point Comparison in Tests
**Confidence: 90/100**

Tests use direct floating point comparison which can be flaky. Use consistent relative tolerance.

---

### 8. Inadequate Test Coverage for Edge Cases
**Confidence: 85/100**

Missing test cases for empty strings, whitespace-only, special characters, and Unicode normalization.

---

### 9. Missing Clone Implementation Considerations
**Confidence: 85/100**

`ApiProbeResult` derives `Clone` but contains `Option<serde_json::Value>` which is expensive to clone.

---

### 10. Inefficient String Allocations in Fallback Dictionary
**Confidence: 85/100**

Every dictionary entry creates temporary string allocations. Consider using a macro for cleaner initialization.

---

## SUGGESTIONS (Minor Polish)

### 11. Inconsistent Naming Convention
Mix of English and Dutch in documentation. Choose one language consistently.

### 12. Missing Top-Level Module Documentation
Could benefit from architecture diagram, usage examples, performance characteristics.

### 13. Test Naming Convention Inconsistency
Follow a consistent test naming convention.

### 14. No Integration Tests
Add integration tests for the full workflow.

### 15. Hardcoded Dutch Organization List Maintenance Burden
Consider tracking fallback dictionary misses to identify missing organizations.

---

## Summary

**Critical Issues (Must Fix):** 3
**Important Issues (Should Fix):** 7
**Suggestions (Nice to Have):** 5

The code demonstrates good Rust practices with proper use of `static Lazy`, `&'static str` for static data, and comprehensive unit tests. The main issues are around consistency with existing project patterns (error handling) and production readiness (retry logic, edge case handling).
