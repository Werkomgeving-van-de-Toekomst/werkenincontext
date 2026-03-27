Now I have all the information needed to generate the section content. Let me create the self-contained markdown section for section-07-frontend-coloring.

# Section 07: Frontend Height-Based Coloring

## Overview

This section implements height-based coloring for 3D buildings on the map. Currently all buildings render with a single gray color (`#8899aa`). After this section, buildings will be colored using a blue-to-purple gradient based on their height, making it easier to visually distinguish low-rise from high-rise structures.

**Dependencies:** This section requires `section-06-frontend-loading` to be complete, as it relies on the building data source and GeoJSON structure being properly loaded.

## Context

### Current Paint Properties

The current implementation in `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs` (lines 168-173) uses a single static color:

```javascript
paint: {
    'fill-extrusion-color': '#8899aa',
    'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
    'fill-extrusion-base': 0,
    'fill-extrusion-opacity': 0.8
}
```

### Building Height Property

The `height` property in the GeoJSON features comes from the 3DBAG API's `b3_h_dak_max` (roof height) attribute, processed by the backend in `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs`.

### Color Scheme Specification

| Height Range | Color | Hex Code |
|--------------|-------|----------|
| 0 - 5 meters | Light blue | `#64B5F6` |
| 5 - 15 meters | Medium purple | `#9B59B6` |
| 15+ meters | Dark purple | `#8E44AD` |

## Tests

### Frontend: Paint Properties

These are manual browser tests since MapLibre GL integration cannot be unit tested in Rust.

- Test: fill-extrusion-color uses step expression
- Test: Color thresholds match specification (5m, 15m)
- Test: Buildings with height 0-5m show light blue
- Test: Buildings with height 5-15m show medium purple
- Test: Buildings with height 15m+ show dark purple
- Test: Missing height defaults to reasonable color (first step)
- Test: Opacity remains set to 0.8

### Frontend: Legend (Optional Enhancement)

If implementing the legend overlay:

- Test: Legend HTML is generated
- Test: Legend shows correct color scale
- Test: Legend is positioned on map without overlapping controls

## Implementation

### File to Modify

**Primary file:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

### Changes Required

In the `get_map3d_init_script()` function, update the `paint` property for the `building-3d` layer (around line 168-173):

```rust
fn get_map3d_init_script() -> String {
    r#"
    // ... existing initialization code ...
    
    map.addLayer({
        id: 'building-3d',
        type: 'fill-extrusion',
        source: 'buildings',
        paint: {
            'fill-extrusion-color': [
                'step',
                ['get', 'height'],
                '#64B5F6',  // 0-5m: Light blue
                5,
                '#9B59B6',  // 5-15m: Medium purple
                15,
                '#8E44AD'   // 15m+: Dark purple
            ],
            'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
            'fill-extrusion-base': 0,
            'fill-extrusion-opacity': 0.8
        }
    });
    
    // ... rest of code ...
    "#
}
```

### Understanding the Step Expression

The MapLibre step expression works as follows:

```javascript
['step',
    input,            // The input value (height)
    default_output,   // Output when input < first threshold
    threshold_1,      // First threshold value
    output_1,         // Output when threshold_1 <= input < threshold_2
    threshold_2,      // Second threshold value
    output_2          // Output when input >= threshold_2
]
```

So for our coloring:
- Height < 5m → `#64B5F6` (light blue)
- 5m <= Height < 15m → `#9B59B6` (medium purple)
- Height >= 15m → `#8E44AD` (dark purple)

### Optional: Legend Overlay

To help users understand the color scale, add a legend overlay. This can be implemented as additional HTML/CSS in the map container.

**HTML structure** (add inside the map container):

```html
<div class="building-legend">
    <div class="legend-title">Gebouwhoogte</div>
    <div class="legend-scale">
        <div class="legend-item" style="background: #64B5F6;"><span>0-5m</span></div>
        <div class="legend-item" style="background: #9B59B6;"><span>5-15m</span></div>
        <div class="legend-item" style="background: #8E44AD;"><span>15m+</span></div>
    </div>
</div>
```

**CSS** (add to the page styles):

```css
.building-legend {
    position: absolute;
    bottom: 30px;
    left: 10px;
    background: white;
    padding: 10px;
    border-radius: 4px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.2);
    z-index: 1000;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    font-size: 12px;
}

.legend-title {
    font-weight: bold;
    margin-bottom: 6px;
    color: #333;
}

.legend-scale {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.legend-item {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    border-radius: 3px;
    color: white;
    font-weight: 500;
}
```

## Testing Checklist

### Functional Tests

- [ ] Buildings display with height-based colors
- [ ] Low buildings (0-5m) show light blue
- [ ] Medium buildings (5-15m) show medium purple
- [ ] Tall buildings (15m+) show dark purple
- [ ] Buildings with missing height data still render (default color)
- [ ] Opacity remains at 0.8
- [ ] Colors update correctly when panning to new areas

### Visual Tests

- [ ] Color distinction is visually clear
- [ ] Colors work well with the basemap style (CartoDB Positron)
- [ ] Legend (if implemented) is readable and positioned correctly
- [ ] Legend colors match actual building colors

### Edge Cases

- [ ] Buildings with height = 0 render correctly
- [ ] Buildings exactly at threshold values (5m, 15m) show correct color
- [ ] Very tall buildings (>100m) still render correctly

## Implementation Notes

1. **No backend changes required** - This section modifies only frontend rendering logic
2. **Height property must exist** - The `height` property comes from the GeoJSON data loaded in section-06
3. **MapLibre step expression** - The `['step', ...]` expression is the most efficient way to do discrete color bands in MapLibre
4. **Color accessibility** - The chosen colors provide good contrast both with each other and against the light basemap
5. **Legend is optional** - The legend overlay is a nice-to-have enhancement that improves user experience but is not required for core functionality

## Related Files

| File | Purpose |
|------|---------|
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs` | Contains the map initialization script to modify |
| `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/buildings_3d.rs` | Backend that provides the `height` property in GeoJSON |

## Next Steps

After completing this section, proceed to `section-08-frontend-popups` to implement click handlers for showing building information in popups.

## Actual Implementation (2025-03-07)

**File Modified:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

**Changes from Original Plan:**
- Added `coalesce` to the color expression for consistency with the height expression
- Changed from `['get', 'height']` to `['coalesce', ['get', 'height'], 0]` to handle null height values explicitly

**Code Review Enhancement Applied:**
- Used `coalesce` in the step expression input to ensure buildings with missing height data are consistently colored and sized. This provides defensive programming and makes the fallback behavior explicit.

**Final Paint Properties:**
```javascript
paint: {
    'fill-extrusion-color': [
        'step',
        ['coalesce', ['get', 'height'], 0],
        '#64B5F6',  // 0-5m: Light blue
        5,
        '#9B59B6',  // 5-15m: Medium purple
        15,
        '#8E44AD'   // 15m+: Dark purple
    ],
    'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
    'fill-extrusion-base': 0,
    'fill-extrusion-opacity': 0.8
}
```