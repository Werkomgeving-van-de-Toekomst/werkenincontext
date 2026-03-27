# Code Review: Section 08 - Frontend Click Popups

**Reviewed File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

**Review Date:** 2026-03-07

**Review Scope:** XSS safety, popup behavior/lifecycle, MapLibre GL JS integration, property display, error handling, security vulnerabilities

---

## Summary

The section implements interactive popups for the 3D buildings layer using MapLibre GL JS. The implementation demonstrates strong security awareness with proper XSS-safe DOM methods throughout. The popup lifecycle is well-managed with cleanup on map movement. All 5 test cases pass with good coverage of security-critical behaviors.

**Overall Assessment:** APPROVED with minor suggestions

---

## Critical Issues (90-100)

### None Found

No critical security vulnerabilities or bugs were identified. The implementation follows security best practices for XSS prevention.

---

## Important Issues (80-89)

### 1. Potential Memory Leak with Multiple Popups

**Confidence: 85/100**

**Location:** `build_popup_handler_script()`, lines 293-303

**Issue:** When clicking on buildings, a new popup is created each time without explicitly removing the previous popup. While MapLibre may handle this automatically, the code relies on implicit behavior rather than explicit cleanup.

**Current Code:**
```javascript
map.on('click', 'building-3d', function(e) {
    var features = map.queryRenderedFeatures(e.point, {
        layers: ['building-3d']
    });

    if (features.length > 0) {
        var props = features[0].properties;
        var popup = createBuildingPopup(props);
        popup.setLngLat(e.lngLat).addTo(map);
    }
});
```

**Recommendation:**
```javascript
// Store reference to current popup
var currentPopup = null;

map.on('click', 'building-3d', function(e) {
    var features = map.queryRenderedFeatures(e.point, {
        layers: ['building-3d']
    });

    if (features.length > 0) {
        // Remove previous popup if exists
        if (currentPopup) {
            currentPopup.remove();
        }
        var props = features[0].properties;
        currentPopup = createBuildingPopup(props);
        currentPopup.setLngLat(e.lngLat).addTo(map);
    }
});
```

**Impact:** Low - MapLibre's Popup class likely handles this, but explicit cleanup is clearer and prevents potential memory accumulation.

---

### 2. Missing Error Handling for Missing Properties

**Confidence: 82/100**

**Location:** `build_popup_handler_script()`, line 299

**Issue:** The code accesses `features[0].properties` without validation. If a feature exists but has no properties object, this will cause a JavaScript error.

**Current Code:**
```javascript
if (features.length > 0) {
    var props = features[0].properties;
    var popup = createBuildingPopup(props);
    popup.setLngLat(e.lngLat).addTo(map);
}
```

**Recommendation:**
```javascript
if (features.length > 0 && features[0].properties) {
    var props = features[0].properties;
    var popup = createBuildingPopup(props);
    popup.setLngLat(e.lngLat).addTo(map);
}
```

**Impact:** Low - In practice, GeoJSON features from the 3DBAG API should always have properties. However, defensive coding would prevent crashes from malformed data.

---

## Security Analysis

### XSS Safety: EXCELLENT

The implementation demonstrates strong security practices:

1. **No `innerHTML` usage** - All content is added via DOM methods
2. **Proper use of `textContent`** - All user-controlled data uses `textContent`
3. **`document.createElement`** - DOM is built programmatically
4. **`createTextNode`** - Even string values are properly escaped via DOM API

**Test coverage confirms:**
- `test_popup_script_uses_dom_methods` - Verifies `textContent` usage
- `test_popup_does_not_use_innerhtml` - Confirms no `innerHTML` present

### Property Display Safety

**Confidence: 95/100**

The `addRow` helper function properly handles null/undefined values:
```javascript
function addRow(label, value) {
    if (value === null || value === undefined) return;
    // ... safe DOM construction
}
```

**Height formatting is safe:**
```javascript
addRow('Hoogte', props.height !== undefined ? props.height.toFixed(1) + 'm' : null);
```

The ternary check prevents calling `.toFixed()` on undefined values.

---

## Integration with MapLibre GL JS

### Correct Usage Patterns

1. **`queryRenderedFeatures`** - Properly filters by layer name
2. **`Popup.setDOMContent()`** - Correct method for custom content
3. **Event listener placement** - Handler is registered inside `map.on('load')` callback

### Popup Offset

**Line 289:** `new maplibregl.Popup({ offset: 15 })`

The 15px offset is appropriate for 3D building clicks to prevent the popup from being obscured by the building geometry.

---

## Test Coverage Analysis

All 5 tests are meaningful and well-targeted:

| Test | Purpose | Status |
|------|---------|--------|
| `test_popup_script_uses_dom_methods` | Verify `textContent` usage | PASS |
| `test_popup_does_not_use_innerhtml` | Verify no unsafe HTML insertion | PASS |
| `test_popup_includes_all_properties` | Verify all expected properties displayed | PASS |
| `test_popup_has_correct_css_class` | Verify CSS class integration | PASS |
| `test_popup_has_click_handler` | Verify event listener registration | PASS |

### Suggested Additional Tests

1. **Test for null safety in `addRow` logic**
2. **Test for move event handler presence**
3. **Test for popup offset configuration**

---

## Popup Lifecycle

### Creation

- Triggered by click on `building-3d` layer
- Creates DOM elements via `createBuildingPopup()`
- Positioned at click coordinates via `setLngLat(e.lngLat)`

### Cleanup

- Popups are removed on map movement via `move` event listener
- Uses `document.querySelectorAll('.maplibregl-popup')` for cleanup
- This is a bit broad but safe - it only removes MapLibre popup elements

### Potential Improvement

The move event handler could use a reference-based approach instead of `querySelectorAll`:

```javascript
// Store popup reference for cleaner removal
var activePopup = null;

map.on('move', function() {
    if (activePopup) {
        activePopup.remove();
        activePopup = null;
    }
});
```

---

## CSS Integration

The popup correctly uses the `building-popup` CSS class defined in section-05:

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/styles/building_popup.css`

```css
.building-popup { ... }
.building-popup h3 { ... }
.building-popup p { ... }
.building-popup strong { ... }
```

The integration is verified by `test_popup_has_correct_css_class`.

---

## Recommendations Summary

### High Priority
None - the implementation is secure and functional.

### Medium Priority
1. Add explicit popup cleanup on new click to prevent potential accumulation
2. Add null check for `features[0].properties` before access

### Low Priority
1. Consider storing popup reference for more efficient cleanup on move
2. Add test coverage for popup offset configuration

---

## Conclusion

This is a well-implemented feature with strong security practices. The XSS-safe approach using `textContent` and DOM methods is exemplary. The integration with MapLibre GL JS follows correct patterns, and the popup lifecycle is properly managed.

The code is ready for merge. The suggested improvements are minor and do not represent defects that would prevent deployment.

**Status:** APPROVED
