---
phase: 2-3d-buildings-enhancements
plan: GAP-01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/iou-frontend/src/components/filter_panel_3d.rs
autonomous: false
requirements: [FILT-01, FILT-02, FILT-03, FILT-04, FILT-05, FILT-06]
user_setup: []

must_haves:
  truths:
    - "Filter buildings by year, height, floor count works without 'Style is not done loading' error"
    - "All filter sliders update visible building count correctly"
    - "Clear filters button resets all sliders and shows all buildings"
  artifacts:
    - path: "crates/iou-frontend/src/components/filter_panel_3d.rs"
      provides: "Filter panel with style-loaded check"
      min_lines: 350
  key_links:
    - from: "filter_panel_3d.rs use_effect"
      to: "map.on('load') event"
      via: "map.isStyleLoaded() check before setFilter()"
      pattern: "isStyleLoaded.*setFilter|on\\('load'"
---

<objective>
Fix MapLibre style load race condition causing "Style is not done loading" errors when filter sliders are adjusted

Purpose: Users can adjust filter sliders immediately without encountering MapLibre errors
Output: Working filter controls with proper style load detection
</objective>

<execution_context>
@/Users/marc/.claude/get-shit-down/workflows/execute-plan.md
@/Users/marc/.claude/get-shit-down/templates/summary.md
</execution_context>

<context>
@.planning/phases/2-3d-buildings-enhancements/2-UAT.md
@.planning/phases/2.4-polish/2.4-01-SUMMARY.md

# Existing Filter Panel Pattern

From filter_panel_3d.rs (lines 116-129):
The use_effect runs immediately on component mount, before MapLibre's style has finished loading.

From build_set_filter_script (lines 35-69):
Currently calls map.setFilter() directly without checking if style is loaded.

# The Problem

The use_effect runs immediately on component mount, before MapLibre's style has finished loading. This causes:
- "Style is not done loading" error from MapLibre
- "Converting circular structure to JSON" from console.log with filter object

# The Fix

Add map.isStyleLoaded() check OR use map.on('load') event to defer filter initialization until style is ready.
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add style load check to build_set_filter_script</name>
  <files>crates/iou-frontend/src/components/filter_panel_3d.rs</files>
  <action>
    Modify build_set_filter_script() to check map.isStyleLoaded() before calling setFilter():

    1. Add map.isStyleLoaded() check after the map exists check
    2. If style is not loaded, use map.once('load', ...) to defer the filter application
    3. Fix console.log to NOT log the raw filter object (causes circular reference)

    Modified build_set_filter_script should:
    - Check if (map && map.isStyleLoaded())
    - If true: apply filter immediately
    - If false: register map.once('load', ...) callback to apply filter when ready
    - Log only the filter expression string, not the parsed object
  </action>
  <verify>
    <automated>cargo test --package iou-frontend --lib build_set_filter_script 2>&1 | head -20</automated>
  </verify>
  <done>build_set_filter_script includes isStyleLoaded() check and uses map.once('load') callback</done>
</task>

<task type="auto">
  <name>Task 2: Add style load check to build_clear_filter_script</name>
  <files>crates/iou-frontend/src/components/filter_panel_3d.rs</files>
  <action>
    Apply the same isStyleLoaded() check pattern to build_clear_filter_script():
    1. Add map.isStyleLoaded() check
    2. Use map.once('load', ...) for deferred execution if style not loaded
    3. Ensure filter clearing works even when map is still initializing
  </action>
  <verify>
    <automated>cargo test --package iou-frontend --lib build_clear_filter_script 2>&1 | head -20</automated>
  </verify>
  <done>build_clear_filter_script includes isStyleLoaded() check</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <what-built>Fixed filter controls with style load detection - no more "Style is not done loading" errors</what-built>
  <how-to-verify>
    1. Start dev server: dx serve
    2. Open http://localhost:8080/apps/data-verkenner
    3. Wait for map to fully load (buildings visible)
    4. Adjust year range slider (e.g., 1950-2000)
    5. Verify: No console error "Style is not done loading"
    6. Verify: Visible building count updates correctly
    7. Adjust height range slider
    8. Verify: No console error, filter works
    9. Click "Filters wissen" (Clear Filters) button
    10. Verify: All buildings become visible, sliders reset to defaults
  </how-to-verify>
  <resume-signal>Type "approved" if filters work without errors, or describe any remaining issues</resume-signal>
</task>

</tasks>

<verification>
After checkpoint approval:
- All filter controls (year, height, floors) work without MapLibre errors
- Building count updates correctly
- Clear filters button works
- No "Style is not done loading" or "circular structure" errors in console
</verification>

<success_criteria>
- [ ] build_set_filter_script includes map.isStyleLoaded() check
- [ ] build_clear_filter_script includes map.isStyleLoaded() check
- [ ] No "Style is not done loading" errors when using filter sliders
- [ ] No "circular structure to JSON" errors
- [ ] Filter changes update visible building count
- [ ] Clear filters button resets all sliders
</success_criteria>

<output>
After completion, create `.planning/phases/2-3d-buildings-enhancements/2-GAP-01-SUMMARY.md`
</output>
