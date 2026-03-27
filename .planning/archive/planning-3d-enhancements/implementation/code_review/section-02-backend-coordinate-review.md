# Code Review: Section 02 - Backend Coordinate Conversion

## Review Scope
Reviewing changes to `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs` that enable proj crate for accurate RD to WGS84 coordinate conversion.

## Critical Issues

### 1. Performance Problem: Creating Proj Instance on Every Request - Confidence: 100

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`
**Lines:** 83, 152-154

**Issue:** The code creates a new `Proj` instance on every HTTP request (line 83) and potentially multiple times per request (lines 152-154 in `convert_geometry_proj`). The `Proj::new_known_crs()` constructor is expensive as it loads projection data and initializes the PROJ library.

**Impact:** This will cause significant performance degradation under load. Each request will:
- Check if proj is available (line 83) - creating one instance
- Process potentially dozens of building features
- Call `convert_geometry_proj()` for each building (lines 113-116)
- Create ANOTHER proj instance inside `convert_geometry_proj()` (line 152)

For a request with 50 buildings (the default limit), this could create 51+ Proj instances.

**Fix:** Use lazy_static or once_cell to create a single shared instance:

```rust
use once_cell::sync::Lazy;

static PROJ_INSTANCE: Lazy<Option<proj::Proj>> = Lazy::new(|| {
    proj::Proj::new_known_crs("EPSG:28992", "EPSG:4326", None).ok()
});

// Then in get_buildings_3d:
let use_proj = PROJ_INSTANCE.is_some();

// And in convert_geometry_proj:
fn convert_geometry_proj(...) -> serde_json::Value {
    let proj = match PROJ_INSTANCE.as_ref() {
        Some(p) => p,
        None => return convert_geometry_fallback(geometry, vertices, transform),
    };
    // ... rest of function
}
```

---

### 2. Inconsistent Error Handling Between Functions - Confidence: 90

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`
**Lines:** 15-24, 152-154, 186-189

**Issue:** There are three different error handling patterns for proj failures:

1. **`rd_to_wgs84()` function** (lines 15-24): Returns `Option<(f64, f64)>` - uses `?` operator to propagate errors and return `None`
2. **`convert_geometry_proj()` function** (lines 152-154): Creates proj instance, falls back to `convert_geometry_fallback()` on error
3. **Inside `convert_geometry_proj()`** (lines 186-189): When converting individual coordinates, uses `match` and returns `None` for failed conversions, which filters out that coordinate

**Impact:**
- The function `rd_to_wgs84()` is defined but never actually used in the hot path (lines 113-116 call `convert_geometry_proj` directly instead)
- Individual coordinate conversion failures (line 188) silently drop coordinates with `None`, potentially creating malformed polygons
- No logging when conversion fails, making debugging difficult

**Fix:**
1. Remove `rd_to_wgs84()` function or use it consistently
2. Add logging when proj fails or conversions fail
3. Consider returning a fallback point instead of filtering to maintain polygon integrity:

```rust
// Add logging
use tracing::warn;

match proj.convert((rd_x, rd_y)) {
    Ok((lon, lat)) => Some(json!([lon, lat])),
    Err(e) => {
        warn!("Failed to convert RD coordinates ({}, {}): {}", rd_x, rd_y, e);
        // Return fallback instead of None to maintain polygon structure
        let (fallback_lon, fallback_lat) = rd_to_wgs84_fallback(rd_x, rd_y);
        Some(json!([fallback_lon, fallback_lat]))
    }
}
```

---

## Important Issues

### 3. Test Quality: Weak Test Assertion - Confidence: 85

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`
**Lines:** 279-283

**Issue:** The test `test_rd_to_wgs84_returns_none_on_invalid` has a meaningless assertion:

```rust
assert!(result.is_none() || result.is_some()); // Should not panic
```

This assertion is always true (it's a tautology) - any result will pass this test. The comment says "Should not panic" but the test doesn't actually verify that no panic occurs.

**Impact:** This test provides no value and gives false confidence. It won't catch any actual bugs.

**Fix:**

```rust
#[test]
fn test_rd_to_wgs84_returns_none_on_invalid() {
    // Extreme coordinates that might fail conversion
    let result = rd_to_wgs84(f64::NAN, f64::NAN);
    assert!(result.is_none(), "NaN coordinates should return None");

    // Test infinity
    let result = rd_to_wgs84(f64::INFINITY, f64::INFINITY);
    assert!(result.is_none(), "Infinity coordinates should return None");

    // Test extremely large values (outside Netherlands)
    let result = rd_to_wgs84(1_000_000_000.0, 1_000_000_000.0);
    // proj may or may not handle this, but should not panic
    let _ = result;
}
```

---

### 4. Test Quality: Missing Integration Test - Confidence: 80

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`

**Issue:** All tests are unit tests testing individual functions in isolation. There's no integration test that verifies the full request flow:
- HTTP request → bbox parsing → 3DBAG API call → coordinate conversion → GeoJSON response

**Impact:** The tests don't verify that the actual endpoint works correctly with real data flow. Issues like the performance problem (issue #1) won't be caught by unit tests.

**Fix:** Add an integration test that mocks the 3DBAG API and tests the full endpoint.

---

### 5. Documentation: Missing API Documentation - Confidence: 85

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`

**Issue:** The coordinate conversion functions lack proper documentation explaining:
- The accuracy of proj vs fallback (how accurate is each?)
- When proj might fail (missing PROJ data? invalid coordinates?)
- The coordinate system details (RD vs WGS84)
- Performance characteristics

**Impact:** Future maintainers won't understand the trade-offs or when to use each method.

**Fix:** Add comprehensive documentation explaining accuracy, performance, and error handling.

---

## Positive Observations

1. **Good fallback strategy** - The code gracefully degrades to a fallback approximation if proj fails, ensuring the API continues to work
2. **Reasonable test coverage** - Tests cover the main scenarios including Amersfoort reference point
3. **Correct CRS codes** - Uses proper EPSG codes (28992 for RD, 4326 for WGS84)
4. **Defense in depth** - Multiple layers of error handling (proj check, conversion errors, fallback)
5. **Clean code structure** - Functions are well-organized and follow Rust conventions

## Summary

The code correctly implements coordinate conversion with good error handling, but has a **critical performance issue** that will cause problems under load. The test coverage needs improvement, and documentation would help future maintainers.

**Action Items (Priority Order):**
1. Fix performance issue by using lazy_static for shared Proj instance
2. Improve error handling consistency and add logging
3. Fix the meaningless test assertion
4. Add integration test for full request flow
5. Add comprehensive documentation

**Overall Assessment:** The code will function correctly for small loads but needs performance optimization before production use. The fallback strategy is well-implemented, and the tests provide basic coverage but need strengthening.
