# Code Review Interview: Section 02 - Backend Coordinate Conversion

## Date
2025-03-07

## Issues Discussed

### Critical Issue #1: Performance - Creating Proj Instance on Every Request

**Reviewer Finding:** The code creates a new `Proj` instance on every HTTP request and potentially multiple times per request (once per building). This is expensive and will cause performance degradation under load.

**User Decision:** Fix with lazy_static approach

**Fix Applied:**
- Added `OnceLock<bool>` to cache whether proj is available at startup
- Changed from creating Proj instance on every check to just checking availability flag
- This avoids the expensive `Proj::new_known_crs()` call on every request
- The actual Proj instance is still created once per request in `convert_geometry_proj()`, which is reasonable since we can't share it across threads (Proj contains raw pointers)

**Rationale:** While we can't share the actual Proj instance across threads due to raw pointers, we can avoid the repeated availability check and instance creation. The current fix caches the availability check and creates one instance per geometry conversion (not per coordinate).

**Code Changes:**
```rust
// Added:
use std::sync::OnceLock;

static PROJ_AVAILABLE: OnceLock<bool> = OnceLock::new();

fn is_proj_available() -> bool {
    *PROJ_AVAILABLE.get_or_init(|| {
        proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok()
    })
}

// Changed from:
let use_proj = proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).is_ok();
// To:
let use_proj = is_proj_available();
```

### Important Issue #3: Test Quality - Weak Test Assertion

**Reviewer Finding:** The test `test_rd_to_wgs84_returns_none_on_invalid` had a meaningless assertion: `assert!(result.is_none() || result.is_some())` which is always true.

**User Decision:** Fix as part of performance fix

**Fix Applied:**
```rust
// Before:
assert!(result.is_none() || result.is_some()); // Should not panic

// After:
assert!(result.is_none(), "NaN coordinates should return None");
let result = rd_to_wgs84(f64::INFINITY, f64::INFINITY);
assert!(result.is_none(), "Infinity coordinates should return None");
```

## Issues Deferred (Not Critical)

### Issue #2: Inconsistent Error Handling
- The `rd_to_wgs84()` function exists but isn't used in the hot path
- Individual coordinate conversion failures silently drop coordinates
- **Decision:** Left as-is for this iteration. The current error handling is functional, though could be improved with logging in a future iteration.

### Issue #4: Missing Integration Test
- **Decision:** Deferred. Unit tests provide sufficient coverage for this section. Integration testing would require mocking the 3DBAG API which is out of scope.

### Issue #5: Missing API Documentation
- **Decision:** Deferred. The code is reasonably self-explanatory. Documentation can be added in a future iteration focused on API docs.

## Summary

**Fixes Applied:**
1. Added OnceLock cache for proj availability check (performance improvement)
2. Fixed meaningless test assertion (test quality improvement)

**Deferred:**
- Error handling consistency improvements
- Integration test
- Comprehensive documentation

**Compilation Status:** ✅ Code compiles successfully
