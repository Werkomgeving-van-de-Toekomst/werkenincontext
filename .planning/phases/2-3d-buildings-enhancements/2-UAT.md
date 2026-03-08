---
status: diagnosed
phase: 2-3d-buildings-enhancements
source: 2.2-view-toggle/2.2-01-SUMMARY.md, 2.3-density-analysis/2.3-01-SUMMARY.md, 2.4-polish/2.4-01-SUMMARY.md, 2.4-polish/2.4-02-SUMMARY.md, 2.1-building-filtering/2.1-PLAN.md
started: 2026-03-08T10:55:00Z
updated: 2026-03-08T11:58:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Filter Buildings by Year Range
expected: Open the data explorer with 3D buildings. Use the year range slider to set a range (e.g., 1950-2000). Only buildings built within that year range should be visible on the map. The building count display should update to show the filtered count.
result: issue
reported: "Filter update error: Error: Style is not done loading. at de._checkLoaded (style.ts:556:19) at de.setFilter (style.ts:1112:14)"
severity: blocker

### 2. Filter Buildings by Height Range
expected: Use the height range slider to filter by building height (e.g., 0-50 meters). Only buildings within that height range should be visible. The count display should update.
result: issue
reported: "Filter update error: TypeError: Converting circular structure to JSON at JSON.stringify (<anonymous>) at console.error (patch_console.js:1:681)"
severity: blocker

### 3. Filter Buildings by Floor Count
expected: Use the floor count slider to filter by number of floors (e.g., 1-5 floors). Only buildings with matching floor counts should be visible. The count display should update.
result: issue
reported: "Same filter update error as test 2 - circular structure to JSON when calling setFilter"
severity: blocker

### 4. Clear All Filters
expected: Click the "Clear Filters" button. All buildings should become visible again, and sliders should reset to their default ranges. The count should show total buildings.
result: issue
reported: "Same filter update error - setFilter fails with circular structure JSON error"
severity: blocker

### 5. Toggle 2D/3D View Mode
expected: Click the view toggle button (top-left of map). The map should smoothly animate between 2D (top-down, flat) and 3D (perspective, buildings extruded) views. The button label should change to reflect current mode.
result: issue
reported: "Sometimes see purple buildings in 3D near Almere and then near Lelystad - appears to be density heatmap incorrectly rendering on building geometries"
severity: major

### 6. View Mode Persists After Refresh
expected: Set view mode to 2D or 3D, then refresh the browser page. The view mode should remain in the selected state (not reset to default).
result: pass

### 7. Enable Density Heatmap
expected: Click the density heatmap toggle button (top-right of map). A heatmap overlay should appear showing building density with color gradient (light blue = low density, dark purple = high density).
result: issue
reported: "Can see purple buildings in Lelystad - heatmap appears to render on building geometries instead of as overlay"
severity: major

### 8. Heatmap Updates on Map Pan/Zoom
expected: With heatmap enabled, pan or zoom the map. The heatmap should update to reflect the new viewport area after a brief delay (debounced).
result: pass

### 9. URL Updates with Filter Changes
expected: Change any filter slider. The browser URL should update with query parameters like `?year_min=1950&year_max=2024&height_min=0&height_max=100`. Parameters should be human-readable, not base64.
result: issue
reported: "Browser URL does not update with filter changes - filters work but URL state persistence is broken"
severity: major

### 10. URL Updates with View Toggle
expected: Click the 2D/3D view toggle. The URL should update with `?view=2d` or `?view=3d` parameter, preserving existing filter/heatmap params.
result: pass

### 11. URL Updates with Heatmap Toggle
expected: Click the density heatmap toggle. The URL should update with `?heatmap=true` or `?heatmap=false` parameter, preserving existing filter/view params.
result: pass
note: "Console error: Density update error: SyntaxError: Unexpected token '<', \"<!DOCTYPE \"... is not valid JSON - API returning HTML instead of JSON"

### 12. Restore State from URL
expected: Copy the URL with all parameters, open in a new browser or fresh tab. The map should load with the exact same filter ranges, view mode, and heatmap state as encoded in the URL.
result: issue
reported: "Buildings popping up during load and URL doesn't preserve parameters - shows base URL http://localhost:8080/apps/data-verkenner without query params"
severity: major

### 13. Browser Back/Forward Navigation
expected: Make several changes (filters, view, heatmap). Use browser back button - each step should restore the previous state. Use forward button to redo changes.
result: pass

### 14. Smooth View Toggle Animation
expected: Click 2D/3D toggle. The camera should animate smoothly (approx. 500ms) - no jarring cuts. UI elements should fade in/out smoothly during transition.
result: issue
reported: "Buildings loading lags during view toggle - animation not smooth"
severity: minor

### 15. Button Hover Transitions
expected: Hover over any control button (view toggle, heatmap, clear filters). Color change should be smooth (approx. 200ms transition), not instant. All buttons should have consistent hover behavior.
result: pass

## Summary

total: 15
passed: 6
issues: 9
pending: 0
skipped: 0

## Gaps

- truth: "Filter buildings by year, height, floor count should work without errors"
  status: failed
  reason: "User reported: Race condition - setFilter called before MapLibre style loads, plus console.log circular reference error"
  severity: blocker
  test: "1, 2, 3, 4"
  root_cause: "FilterPanel3D's use_effect runs immediately on mount, calling map.setFilter() before MapLibre style has finished loading. console.log with filter object causes circular reference JSON.stringify error."
  artifacts:
    - path: "crates/iou-frontend/src/components/filter_panel_3d.rs"
      issue: "use_effect at line 116 runs immediately on mount, before map style loads. build_set_filter_script (line 35-69) calls map.setFilter() without checking map.isStyleLoaded()"
  missing:
    - "map.isStyleLoaded() check before calling map.setFilter()"
    - "map.on('styledata') or map.on('load') event-based filter initialization"
    - "Filter initialization should occur AFTER map.on('load') completes"
    - "Use console.log with JSON.stringify(filter) instead of raw object"

- truth: "Density heatmap should render as overlay, not color buildings purple"
  status: failed
  reason: "User reported: Purple buildings appear instead of heatmap overlay - MapLibre fill-extrusion layers always render on top of heatmap layers"
  severity: major
  test: "5, 7"
  root_cause: "MapLibre GL's fill-extrusion layers always render on top of heatmap layers regardless of layer ordering. The heatmap layer type is incompatible with 3D extruded buildings as an overlay."
  artifacts:
    - path: "crates/iou-frontend/src/components/density_heatmap.rs"
      issue: "Line 69 inserts heatmap layer before 'building-3d', but fill-extrusion layers always render on top of heatmap layers. Duplicate event listeners registered on each toggle (lines 376-377)."
    - path: "crates/iou-frontend/src/pages/data_verkenner.rs"
      issue: "Lines 166-174 define building color scheme using purple (#9B59B6, #8E44AD) for 5m+ buildings - users see these when heatmap fails to render"
  missing:
    - "Use a separate 2D circle/fill layer for heatmap visualization or render buildings as 2D layer when heatmap enabled"
    - "Prevent duplicate event listener registration"
    - "Trigger initial density calculation immediately after enabling heatmap"

- truth: "Browser URL should update when filter sliders change"
  status: failed
  reason: "User reported: Filter sliders work but don't update URL - only view toggle and heatmap toggle update URL"
  severity: major
  test: 9
  root_cause: "FilterPanel3D's use_effect only calls build_set_filter_script() but does NOT call build_update_url_script(). View toggle and heatmap components have inline URL update JavaScript in their click handlers."
  artifacts:
    - path: "crates/iou-frontend/src/components/filter_panel_3d.rs"
      issue: "Lines 116-129: use_effect only calls build_set_filter_script(), missing URL sync. Lines 164-249: Slider oninput handlers only update signal state, no URL update"
    - path: "crates/iou-frontend/src/components/view_toggle.rs"
      working: "Lines 154-192: Shows correct pattern - inline JavaScript updates URL after state change"
    - path: "crates/iou-frontend/src/components/density_heatmap.rs"
      working: "Lines 455-491: Shows correct pattern - inline JavaScript updates URL after state change"
  missing:
    - "URL update call in FilterPanel3D's use_effect hook OR in each slider's oninput handler"
    - "Follow ViewToggle/DensityHeatmap pattern: read current URL, preserve existing params, update changed param, call history.replaceState()"

- truth: "Density heatmap API should return JSON, not HTML"
  status: failed
  reason: "User reported: Dioxus dev server proxy not forwarding /api/* requests to backend API"
  severity: major
  test: 11
  root_cause: "Dioxus dev server proxy setting in Dioxus.toml is not forwarding /api/* requests to backend API. The proxy setting exists but doesn't work properly."
  artifacts:
    - path: "crates/iou-frontend/Dioxus.toml"
      issue: "proxy = \"http://localhost:8000\" setting is not forwarding /api/* requests"
    - path: "crates/iou-frontend/src/components/density_heatmap.rs"
      issue: "Frontend makes requests to /api/buildings-3d expecting JSON, but receives HTML from Dioxus dev server's fallback"
  missing:
    - "Fix Dioxus proxy configuration OR serve frontend from API server's static file handler"
    - "Alternative: use reverse proxy (nginx/traefik) for development"

- truth: "URL state should be restorable from query parameters"
  status: failed
  reason: "User reported: Buildings popping up during load and URL doesn't preserve parameters - shows base URL without query params"
  severity: major
  test: 12
  root_cause: "Related to filter URL state not being updated (gap 3) - since filters don't write to URL, URL restoration can't work. Also buildings 'pop up' due to async loading without initial state."
  artifacts:
    - path: "crates/iou-frontend/src/pages/data_verkenner.rs"
      issue: "Map initialization doesn't restore filter state from URL params on load"
  missing:
    - "Parse URL params on page load and apply to filter state"
    - "Depends on fixing gap 3 (filter URL state writing)"

- truth: "View toggle animation should be smooth without loading lag"
  status: failed
  reason: "User reported: Buildings loading lags during view toggle - animation not smooth"
  severity: minor
  test: 14
  root_cause: "Buildings are fetched and rendered asynchronously during view toggle, causing visual lag. The 3D building data loading is not pre-buffered or synchronized with the camera animation."
  artifacts:
    - path: "crates/iou-frontend/src/components/view_toggle.rs"
      issue: "View toggle triggers async building loads without pre-buffering"
  missing:
    - "Pre-buffer buildings before animation or use fade transitions"
    - "Lower priority cosmetic issue"
