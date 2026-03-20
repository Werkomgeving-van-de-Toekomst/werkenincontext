# Code Review Interview: Section 00 - Feasibility Spike

**Date:** 2026-03-16
**Section:** section-00-feasibility-spike

---

## Decision Items (User Input Required)

### 1. Error Handling Pattern Refactor

**Finding:** The stakeholder module uses `Result<String, String>` and `Option<&str>` instead of proper error types, inconsistent with the project's `thiserror` pattern (`LlmError`, `PipelineError`).

**Options:**
1. **Refactor now** - Create `StakeholderError` using thiserror (recommended for consistency)
2. **Defer** - Keep simple types for this feasibility spike, refactor in section-01-foundation-types

**Decision:** Refactor in section-01-foundation-types when we create the full stakeholder types. For this feasibility spike, simple error types are acceptable.

**Rationale:** This is a spike to validate external dependencies. Full error handling will be established in section-01 with proper domain types.

---

### 2. API Retry Logic

**Finding:** The Rijksoverheid API probe has no retry logic for transient failures.

**Options:**
1. **Add retry logic now** - Use exponential backoff (adds complexity)
2. **Skip for spike** - The fallback dictionary is the primary solution anyway

**Decision:** Skip for this spike. The local fallback dictionary is the primary solution, not the API.

**Rationale:** The API probe is diagnostic only. Production code will rely on the fallback dictionary.

---

## Auto-Fixes (Applied)

### 3. Duplicate HashMap Key (CRITICAL - Data Bug)

**Finding:** "BZK" inserted twice in FALLBACK_DICT with misleading comment.

**Action:** Remove duplicate entry on line 483-484 of `fallback_dict.rs`.

```rust
// REMOVE these lines (they duplicate the BZK entry on line 447):
// Ministry for Housing and Spatial Planning
// m.insert("BZK".to_lowercase(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties");
```

**Status:** Apply fix

---

### 4. Public API - Hide FALLBACK_DICT

**Finding:** `FALLBACK_DICT` is exported but shouldn't be public API.

**Action:** Remove from public exports in `mod.rs`:

```rust
// BEFORE:
pub use fallback_dict::{get_fallback_canonical_name, FALLBACK_DICT};

// AFTER:
pub use fallback_dict::get_fallback_canonical_name;
```

**Status:** Apply fix

---

### 5. Missing Edge Case Tests

**Finding:** No tests for empty string, whitespace-only input.

**Action:** Add tests to `fallback_dict.rs`:

```rust
#[test]
fn test_empty_string() {
    assert_eq!(get_fallback_canonical_name(""), None);
}

#[test]
fn test_whitespace_only() {
    assert_eq!(get_fallback_canonical_name("   "), None);
}
```

**Status:** Apply fix

---

## Let Go (Not Applying)

### 6. Floating Point Comparison in Tests
**Reasoning:** Tests pass and tolerances are reasonable for cost estimation. Relative error would be better but not critical for a spike.

### 7. Inefficient String Allocations
**Reasoning:** Happens once at startup via `Lazy`. Negligible impact.

### 8. Clone of serde_json::Value
**Reasoning:** `ApiProbeResult` not cloned in hot path. Not worth optimizing.

### 9. Test Naming Inconsistency
**Reasoning:** Nitpicky. Test names are descriptive enough.

### 10. Language Mixing (EN/NL)
**Reasoning:** Docs are clear. Consistency would be nice but not a blocker.

### 11. Missing Integration Tests
**Reasoning:** Unit tests provide good coverage for a spike. Integration tests come in section-10.

---

## Summary

- **User decisions:** 2 items (both deferred to later sections)
- **Auto-fixes applied:** 3 items
- **Let go:** 6 items (nitpicks, low priority)

The core functionality is solid. Main fixes are removing a duplicate key and cleaning up the public API.
