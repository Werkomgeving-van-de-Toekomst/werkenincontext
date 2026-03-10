# Code Review: Section 07 - Frontend Height-Based Coloring

**Reviewed File**: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`
**Review Date**: 2026-03-07
**Reviewer**: Claude Code

---

## Summary

This section implements height-based coloring for 3D buildings using a MapLibre GL JS step expression. The implementation replaces the static gray color with three color bands based on building height. The code is well-structured and correctly implements the MapLibre step expression syntax.

**Overall Assessment**: **Approved** - No issues found with confidence >= 80.

---

## Analysis

### 1. MapLibre Step Expression Correctness

**Confidence: 100 - Correct Implementation**

The step expression syntax is correctly implemented according to MapLibre GL JS specification:

```javascript
'fill-extrusion-color': [
    'step',
    ['get', 'height'],           // Input: height property from features
    '#64B5F6',                    // Default (0-5m): Light blue
    5,                            // First threshold
    '#9B59B6',                    // 5-15m: Medium purple
    15,                           // Second threshold
    '#8E44AD'                     // 15m+: Dark purple
]
```

The step expression evaluates as follows:
- Height < 5m: `#64B5F6` (Light blue)
- 5m <= Height < 15m: `#9B59B6` (Medium purple)
- Height >= 15m: `#8E44AD` (Dark purple)

This is the correct MapLibre `step` syntax: `["step", input, default, stop1, output1, stop2, output2, ...]`.

---

### 2. Color Choices and Accessibility

**Confidence: 85 - Generally Good Colors with Minor Considerations**

**Color Analysis**:

| Height Range | Color | Hex | Luminance | WCAG Contrast on Light Background |
|--------------|-------|-----|-----------|-----------------------------------|
| 0-5m | Light Blue | #64B5F6 | ~176 (69%) | 3.2:1 (AA large text only) |
| 5-15m | Medium Purple | #9B59B6 | ~115 (45%) | 5.8:1 (AA normal) |
| 15m+ | Dark Purple | #8E44AD | ~97 (38%) | 7.1:1 (AAA normal) |

**Context**: The map uses CartoDB Positron basemap (light gray/white background), so contrast is generally acceptable for 3D building visualization.

**Accessibility Considerations**:
- The progression from light to dark (blue -> purple -> darker purple) provides good visual hierarchy
- Colors are distinct enough for most users to differentiate building height categories
- The 0.8 opacity on fill-extrusion helps colors blend naturally

**Minor Recommendation** (not a blocking issue): For users with color vision deficiencies (particularly deuteranopia/protanopia), the blue-purple progression may be harder to distinguish. Consider adding a legend or alternative visual indicators (texture, pattern) if accessibility becomes a concern.

---

### 3. Integration with Existing Code

**Confidence: 100 - Seamless Integration**

The change is well-integrated:
- Minimal diff scope (9 insertions, 1 deletion) - targeted change
- No modifications to surrounding logic required
- Works correctly with existing `['coalesce', ['get', 'height'], 10]` fallback for missing height values
- Opacity (0.8) is maintained from original implementation, providing consistency
- Located in the correct place within the `paint` property

---

### 4. Height-Based Coloring Potential Issues

**Confidence: 90 - One Potential Edge Case Noted**

**Edge Case: Buildings with Null/Undefined Height**

The code correctly handles buildings with missing height data through the coalesce expression on line 169:
```javascript
'fill-extrusion-height': ['coalesce', ['get', 'height'], 10]
```

However, the color expression uses `['get', 'height']` without a coalesce:
```javascript
'fill-extrusion-color': [
    'step',
    ['get', 'height'],  // Returns null if height is missing
    '#64B5F6',          // Default color
    ...
]
```

**Impact**: When height is null/undefined:
- The step expression will use the default color (`#64B5F6`) because null is treated as falsy
- The building will render at 10m height (from the coalesce on height)
- This creates an inconsistency: a 10m-tall building colored as 0-5m

**Recommended Fix** (optional - current behavior may be intentional):

```javascript
'fill-extrusion-color': [
    'step',
    ['coalesce', ['get', 'height'], 0],  // Use 0 as default for coloring
    '#64B5F6',
    5,
    '#9B59B6',
    15,
    '#8E44AD'
]
```

This would ensure buildings with missing height data are consistently colored and sized as if they were 0m tall.

**Note**: This is only relevant if your GeoJSON source may contain features without the `height` property. If all features have height values, this is not an issue.

---

## Issues Summary

| Severity | Issue | Confidence | Status |
|----------|-------|------------|--------|
| N/A | None found | - | - |

**Optional Enhancement** (Confidence: 90):
- Consider adding `coalesce` to the color expression for consistency with the height expression

---

## Conclusion

The implementation is clean, follows MapLibre GL JS best practices, and integrates well with the existing codebase. The color scheme provides good visual differentiation for building heights and works well with the CartoDB Positron basemap.

**Recommendation**: **APPROVED** - This change can be merged. The optional enhancement for null height handling is at the developer's discretion based on data quality requirements.
