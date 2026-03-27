# Code Review: Section 04 - Map3D Component

## Summary

The implementation provides helper functions (`MapEvent` enum, `build_map_init_script`, `build_cleanup_script`) and their tests. The actual Dioxus `Map3D` component is deferred, which is noted as acceptable. The code quality is generally good with comprehensive test coverage for the implemented functionality.

## Critical Issues

### 1. XSS/Cross-Site Scripting Vulnerability (High Severity)

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/map_3d.rs`

**Lines:** 385, 404-449

**Issue:** The `MapEvent::Error::to_json()` and `build_map_init_script()` functions directly interpolate user-provided strings into JavaScript without sanitization:

```rust
MapEvent::Error { message } => format!(r#"{{"Error":"{}"}}"#, message),
```

If `message` contains quotes or backslashes, this breaks JSON syntax and can lead to script injection.

**Fix:** Use proper JSON serialization or escape special characters.

### 2. container_id is Not Validated for Safe HTML IDs (Medium Severity)

**Lines:** 407-449 in `build_map_init_script()`

**Issue:** The `container_id` is directly interpolated into `document.getElementById('{}')`. If `container_id` contains malicious characters, this could be exploited.

**Mitigation:** The config validation should enforce safe HTML ID characters (alphanumeric, hyphens, underscores only).

### 3. Missing WebGL2 Support Check in Initialization Script

**Plan Requirement:** Section 04 plan explicitly states: "Checks for WebGL2 support"

**Implementation:** The generated JavaScript does NOT check for `WebGL2RenderingContext` before creating the map.

**Impact:** Map initialization will fail with cryptic errors on browsers without WebGL2 support.

## Design Issues

### 4. Inconsistent JSON Serialization Pattern

**Lines:** 380-394

The `MapEvent::to_json()` method hand-crafts JSON strings instead of using `serde`. This is error-prone and produces inconsistent JSON structure.

**Recommendation:** Use serde Serialize derive for consistency and safety.

### 5. MapLibre Style URL Contains Placeholder

**Line:** `style: 'https://api.maptiler.com/maps/streets/style.json?key=YOUR_KEY'`

This is a hardcoded placeholder that will cause the map to fail loading in production.

**Recommendation:** The style URL should come from `Map3DConfig` or environment variable.

## Test Coverage

### Good Coverage

- All MapEvent variants have serialization tests
- Script generation is tested for key features
- Cleanup script is tested

### Gaps

- No test validates what happens when `container_id` contains special characters
- No test for WebGL2 check (acknowledged as not implemented)

## Missing from Plan (Acknowledged)

**Dioxus Component Not Implemented** - Lines 466-470: Explicitly deferred with comment explaining web-specific APIs. This is acceptable as documented, but means these acceptance criteria are NOT met:
- "The component renders a container div with the correct id"
- "The map_loaded signal prevents duplicate initialization"
- "Cleanup function executes on component unmount"

## Positive Notes

- Coordinate order (lon, lat) is correctly handled throughout
- Configuration validation tests from section-02 remain intact
- Code is well-documented with rustdoc comments
- Test names are descriptive and follow good conventions
- The `build_cleanup_script()` properly checks for map existence before removal

## Recommendations

1. **High Priority:** Fix XSS vulnerability in `MapEvent::Error::to_json()` and `build_map_init_script()`
2. **High Priority:** Add WebGL2 support check to `build_map_init_script()`
3. **Medium Priority:** Consider using serde for JSON serialization
4. **Medium Priority:** Validate `container_id` format to ensure safe HTML IDs
5. **Low Priority:** Make style URL configurable instead of hardcoded placeholder
