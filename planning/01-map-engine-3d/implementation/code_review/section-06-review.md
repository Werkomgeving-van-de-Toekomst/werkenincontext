# Code Review: Section 06 - GeoJSON Support

## Critical Issues

### 1. Escaped Variables Not Used in JavaScript Template (SECURITY)
**SEVERITY: CRITICAL**

The `js_escape_string()` function is called but the results are never used in the template:
- Lines 304-306 create: `layer_id_escaped`, `url_escaped`, `color_escaped`
- But the format string uses the unescaped values like `{layer.id}`, `{layer.url}`
- This creates an XSS vulnerability - malicious layer data could inject JavaScript

**Fix**: Use the escaped variables in the format string instead of raw values.

### 2. Missing Color Validation
The color field is escaped but never validated as a valid CSS hex color.
**Fix**: Add validation against regex `^#[0-9a-fA-F]{6}$`

### 3. Layer ID Not Validated
`container_id` is validated via `is_valid_container_id()` but `layer.id` is not.
**Fix**: Validate layer IDs the same way.

### 4. Inconsistent Map Instance Access
Implementation uses `window['map_{}']` while plan specified `window.map3dInstances['{}']`.
**Fix**: This is actually consistent with the rest of the codebase (section-05), so it's acceptable.

### 5. Returns Polygon Instead of Option for Empty GeoJSON
Empty FeatureCollections return Polygon as default, indistinguishable from valid polygons.
**Fix**: For MVP this is acceptable - could be improved later with Option<LayerType>.

### 6. Missing GeometryCollection Support
GeoJSON GeometryCollection is not handled.
**Fix**: Acceptable for MVP - rare in practice.

### 7. No Layer Z-Order Control
The `addLayer()` `before` parameter is not implemented.
**Fix**: Acceptable for MVP - layers render in addition order.

### 8. Source/Layer ID Collision
Source and layer share the same ID, could create duplicates.
**Fix**: Acceptable for MVP where each layer has unique data.

### 9. Missing Integration Test File
Plan specified creating `/Users/marc/Projecten/iou-modern/crates/iou-frontend/tests/geojson_integration_test.rs`.
**Fix**: Create this test file.

## Recommendations

1. **HIGHEST PRIORITY**: Fix the JavaScript template to use escaped variables (#1)
2. Add color hex validation (#2)
3. Validate layer IDs (#3)
4. Create the missing integration test (#9)
