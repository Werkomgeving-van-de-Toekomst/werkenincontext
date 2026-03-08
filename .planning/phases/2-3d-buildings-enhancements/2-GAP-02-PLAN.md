---
phase: 2-3d-buildings-enhancements
plan: GAP-02
type: execute
wave: 2
depends_on: [GAP-01]
files_modified:
  - crates/iou-frontend/src/components/density_heatmap.rs
  - crates/iou-frontend/src/components/view_toggle.rs
autonomous: false
requirements: [DENS-01, DENS-02, DENS-03]
user_setup: []

must_haves:
  truths:
    - "Density heatmap renders as overlay with color gradient, not as purple buildings"
    - "Heatmap colors show light blue (low density) to dark purple (high density) gradient"
    - "When heatmap is enabled, buildings render normally (not colored purple)"
  artifacts:
    - path: "crates/iou-frontend/src/components/density_heatmap.rs"
      provides: "Heatmap component that switches to 2D when enabled"
      min_lines: 520
  key_links:
    - from: "density_heatmap toggle"
      to: "view_mode state"
      via: "Auto-switch to 2D view when heatmap enabled"
      pattern: "setPaintProperty.*fill-extrusion-height.*0"
---

<objective>
Fix heatmap rendering incorrectly on building geometries by switching to 2D view when heatmap is enabled

Purpose: Heatmap displays as proper color gradient overlay, not as purple-colored 3D buildings
Output: Working density heatmap visualization
</objective>

<execution_context>
@/Users/marc/.claude/get-shit-down/workflows/execute-plan.md
@/Users/marc/.claude/get-shit-down/templates/summary.md
</execution_context>

<context>
@.planning/phases/2-3d-buildings-enhancements/2-UAT.md
@.planning/phases/2.3-density-analysis/2.3-01-SUMMARY.md

# The Problem

From UAT.md Gap 2:
"User reported: Purple buildings appear instead of heatmap overlay - MapLibre fill-extrusion layers always render on top of heatmap layers regardless of layer ordering."

Root cause: MapLibre GL's fill-extrusion layers always render on top of heatmap layers. The heatmap layer type is incompatible with 3D extruded buildings as an overlay.

# Current Implementation

From density_heatmap.rs line 69:
```javascript
map.addLayer({
    id: layerId,
    type: 'heatmap',
    ...
}, 'building-3d');  // Insert before building layer
```

This doesn't work because fill-extrusion layers render on top of heatmap layers regardless of insertion order.

# The Fix

When heatmap is enabled, automatically switch buildings to 2D view (fill-extrusion-height: 0) so the heatmap can render on top of the 2D footprints. When heatmap is disabled, restore the previous 3D/2D view state.

# Exact setPaintProperty Syntax

From view_toggle.rs build_set_view_mode_script(), the exact syntax is:
- 2D mode: `map.setPaintProperty('building-3d', 'fill-extrusion-height', 0);`
- 3D mode: `map.setPaintProperty('building-3d', 'fill-extrusion-height', ["coalesce", ["get", "height"], 10]);`

Copy these EXACTLY including the array expression for 3D mode with coalesce and property get.
</context>

<tasks>

<task type="auto">
  <name>Task 1: Store previous view mode when enabling heatmap</name>
  <files>crates/iou-frontend/src/components/density_heatmap.rs</files>
  <action>
    Modify build_add_heatmap_layer_script() to:

    1. When enabling heatmap (enabled=true):
       - Store current view mode to localStorage as 'savedViewMode'
       - Force buildings to 2D using EXACT syntax:
         `map.setPaintProperty('building-3d', 'fill-extrusion-height', 0);`
       - Set pitch to 0 for top-down view: `map.setPitch(0);`
       - This allows heatmap to render properly on top of 2D footprints

    2. When disabling heatmap (enabled=false):
       - Restore saved view mode from localStorage 'savedViewMode'
       - If saved mode is '3d', restore using EXACT syntax:
         `map.setPaintProperty('building-3d', 'fill-extrusion-height', ["coalesce", ["get", "height"], 10]);`
         `map.setPitch(60);`
       - If no saved mode or '2d', keep 2D: `map.setPaintProperty('building-3d', 'fill-extrusion-height', 0);`
       - Clear localStorage 'savedViewMode'

    CRITICAL: Use EXACT setPaintProperty syntax from view_toggle.rs - the 3D expression MUST be `["coalesce", ["get", "height"], 10]` not just a number.
  </action>
  <verify>
    <automated>cargo test --package iou-frontend --lib build_add_heatmap_layer_script 2>&1 | head -20</automated>
  </verify>
  <done>build_add_heatmap_layer_script stores/restores view mode with exact setPaintProperty syntax</done>
</task>

<task type="auto">
  <name>Task 2: Prevent duplicate event listener registration</name>
  <files>crates/iou-frontend/src/components/density_heatmap.rs</files>
  <action>
    Fix duplicate event listener registration in build_setup_density_update_script():

    Current issue: Lines 376-377 register event listeners every time heatmap toggles on
    Fix: Add a flag to track if listeners are already registered

    1. Add window['densityListenersRegistered'] flag check
    2. Only register moveend/zoomend listeners if flag is false
    3. Set flag to true after registration
  </action>
  <verify>
    <automated>cargo test --package iou-frontend --lib build_setup_density_update_script 2>&1 | head -20</automated>
  </verify>
  <done>build_setup_density_update_script checks for existing listeners before registering</done>
</task>

<task type="auto">
  <name>Task 3: Trigger initial density calculation when enabling heatmap</name>
  <files>crates/iou-frontend/src/components/density_heatmap.rs</files>
  <action>
    Add immediate density calculation trigger when heatmap is enabled:

    In build_add_heatmap_layer_script() when enabled=true:
    - After adding layer and switching to 2D, trigger an immediate density update
    - Call the same density calculation logic used in viewport updates
    - This ensures heatmap shows data immediately, not after next pan/zoom
  </action>
  <verify>
    <automated>cargo test --package iou-frontend --lib build_add_heatmap_layer_script 2>&1 | head -20</automated>
  </verify>
  <done>build_add_heatmap_layer_script triggers immediate density calculation on enable</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <what-built>Fixed density heatmap to render as overlay with 2D buildings - no more purple buildings</what-built>
  <how-to-verify>
    1. Start dev server: dx serve
    2. Open http://localhost:8080/apps/data-verkenner
    3. Wait for map to fully load
    4. Click "Densiteitskaart" (Density Heatmap) button
    5. Verify: Buildings switch to 2D (flat, top-down view)
    6. Verify: Heatmap overlay appears with color gradient (light blue to dark purple)
    7. Verify: No buildings are colored purple - heatmap is separate overlay
    8. Pan/zoom the map
    9. Verify: Heatmap updates to reflect new viewport
    10. Click "Densiteitskaart" button again to disable
    11. Verify: View returns to previous 3D state with proper extrusion height
  </how-to-verify>
  <resume-signal>Type "approved" if heatmap renders correctly as overlay, or describe issues</resume-signal>
</task>

</tasks>

<verification>
After checkpoint approval:
- Heatmap renders as color gradient overlay, not purple buildings
- Buildings automatically switch to 2D when heatmap enabled
- View restores to previous state with proper 3D extrusion when heatmap disabled
- No duplicate event listeners registered
- Heatmap data shows immediately on enable
</verification>

<success_criteria>
- [ ] Heatmap layer renders as color gradient overlay
- [ ] Buildings switch to 2D (flat) when heatmap enabled
- [ ] No purple-colored buildings visible
- [ ] View mode restored with proper 3D extrusion when heatmap disabled
- [ ] No duplicate event listeners
- [ ] Heatmap data visible immediately on enable
</success_criteria>

<output>
After completion, create `.planning/phases/2-3d-buildings-enhancements/2-GAP-02-SUMMARY.md`
</output>
