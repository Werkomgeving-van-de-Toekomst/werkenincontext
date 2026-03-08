---
status: testing
phase: 2-3d-buildings-enhancements
source: 2.2-view-toggle/2.2-01-SUMMARY.md, 2.3-density-analysis/2.3-01-SUMMARY.md, 2.4-polish/2.4-01-SUMMARY.md, 2.4-polish/2.4-02-SUMMARY.md, 2.1-building-filtering/2.1-PLAN.md
started: 2026-03-08T10:55:00Z
updated: 2026-03-08T10:55:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

number: 1
name: Filter Buildings by Year Range
expected: |
  Open the data explorer with 3D buildings. Use the year range slider to set a range (e.g., 1950-2000). Only buildings built within that year range should be visible on the map. The building count display should update to show the filtered count.
awaiting: user response

## Tests

### 1. Filter Buildings by Year Range
expected: Open the data explorer with 3D buildings. Use the year range slider to set a range (e.g., 1950-2000). Only buildings built within that year range should be visible on the map. The building count display should update to show the filtered count.
result: [pending]

### 2. Filter Buildings by Height Range
expected: Use the height range slider to filter by building height (e.g., 0-50 meters). Only buildings within that height range should be visible. The count display should update.
result: [pending]

### 3. Filter Buildings by Floor Count
expected: Use the floor count slider to filter by number of floors (e.g., 1-5 floors). Only buildings with matching floor counts should be visible. The count display should update.
result: [pending]

### 4. Clear All Filters
expected: Click the "Clear Filters" button. All buildings should become visible again, and sliders should reset to their default ranges. The count should show total buildings.
result: [pending]

### 5. Toggle 2D/3D View Mode
expected: Click the view toggle button (top-left of map). The map should smoothly animate between 2D (top-down, flat) and 3D (perspective, buildings extruded) views. The button label should change to reflect current mode.
result: [pending]

### 6. View Mode Persists After Refresh
expected: Set view mode to 2D or 3D, then refresh the browser page. The view mode should remain in the selected state (not reset to default).
result: [pending]

### 7. Enable Density Heatmap
expected: Click the density heatmap toggle button (top-right of map). A heatmap overlay should appear showing building density with color gradient (light blue = low density, dark purple = high density).
result: [pending]

### 8. Heatmap Updates on Map Pan/Zoom
expected: With heatmap enabled, pan or zoom the map. The heatmap should update to reflect the new viewport area after a brief delay (debounced).
result: [pending]

### 9. URL Updates with Filter Changes
expected: Change any filter slider. The browser URL should update with query parameters like `?year_min=1950&year_max=2024&height_min=0&height_max=100`. Parameters should be human-readable, not base64.
result: [pending]

### 10. URL Updates with View Toggle
expected: Click the 2D/3D view toggle. The URL should update with `?view=2d` or `?view=3d` parameter, preserving existing filter/heatmap params.
result: [pending]

### 11. URL Updates with Heatmap Toggle
expected: Click the density heatmap toggle. The URL should update with `?heatmap=true` or `?heatmap=false` parameter, preserving existing filter/view params.
result: [pending]

### 12. Restore State from URL
expected: Copy the URL with all parameters, open in a new browser or fresh tab. The map should load with the exact same filter ranges, view mode, and heatmap state as encoded in the URL.
result: [pending]

### 13. Browser Back/Forward Navigation
expected: Make several changes (filters, view, heatmap). Use browser back button - each step should restore the previous state. Use forward button to redo changes.
result: [pending]

### 14. Smooth View Toggle Animation
expected: Click 2D/3D toggle. The camera should animate smoothly (approx. 500ms) - no jarring cuts. UI elements should fade in/out smoothly during transition.
result: [pending]

### 15. Button Hover Transitions
expected: Hover over any control button (view toggle, heatmap, clear filters). Color change should be smooth (approx. 200ms transition), not instant. All buttons should have consistent hover behavior.
result: [pending]

## Summary

total: 15
passed: 0
issues: 0
pending: 15
skipped: 0

## Gaps

[none yet]
