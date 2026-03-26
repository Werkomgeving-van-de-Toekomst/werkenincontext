---
phase: 2-3d-buildings-enhancements
plan: GAP-03
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/iou-frontend/src/components/filter_panel_3d.rs
autonomous: false
requirements: [POLI-01]
user_setup: []

must_haves:
  truths:
    - "Browser URL updates immediately when filter sliders change"
    - "URL contains filter params: year_min, year_max, height_min, height_max, floors_min, floors_max"
    - "Filter values are preserved when sharing URL with others"
  artifacts:
    - path: "crates/iou-frontend/src/components/filter_panel_3d.rs"
      provides: "Filter panel with reactive URL state persistence"
      min_lines: 380
  key_links:
    - from: "filter_panel_3d.rs slider oninput handlers"
      to: "browser URL"
      via: "Inline JavaScript calling history.replaceState() on each slider change"
      pattern: "history.replaceState|url.search"
---

<objective>
Add URL state persistence to FilterPanel3D so filter changes update the browser URL immediately

Purpose: Users can share their filtered view with others via URL
Output: Working URL sync for all filter parameters
</objective>

<execution_context>
@/Users/marc/.claude/get-shit-down/workflows/execute-plan.md
@/Users/marc/.claude/get-shit-down/templates/summary.md
</execution_context>

<context>
@.planning/phases/2-3d-buildings-enhancements/2-UAT.md
@.planning/phases/2.4-polish/2.4-01-SUMMARY.md

# The Problem

From UAT.md Gap 3:
"User reported: Filter sliders work but don't update URL - only view toggle and heatmap toggle update URL"

Root cause: FilterPanel3D's use_effect only runs on mount, not when sliders change. View toggle and heatmap components have inline URL update JavaScript in their click handlers.

# Working Pattern (from ViewToggle)

ViewToggle onclick handler (lines 154-192) shows correct pattern:
```javascript
// Read heatmap state from localStorage
const heatmapEnabled = localStorage.getItem('densityHeatmapEnabled') === 'true';

// Build URL params with new view mode
const params = new URLSearchParams();
params.set('view', '3d');

// Preserve existing filter params from current URL
const currentParams = new URLSearchParams(window.location.search);
if (currentParams.has('year_min')) params.set('year_min', currentParams.get('year_min'));
// ... (preserve all existing params)

// Update URL
const url = new URL(window.location.href);
url.search = params.toString();
window.history.replaceState({state: 'urlStateUpdated'}, '', url);
```

# The Fix Approach

The use_effect in filter_panel_3d.rs only runs on mount. To update URL on every slider change, use use_reactive_effect which runs whenever the tracked signals change, OR add inline URL update JavaScript to each slider's oninput handler.

Since use_reactive_effect in Dioxus has limitations with JavaScript execution, the recommended approach is to add inline URL update to each slider's oninput handler (similar to ViewToggle pattern).
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add inline URL update to slider oninput handlers</name>
  <files>crates/iou-frontend/src/components/filter_panel_3d.rs</files>
  <action>
    Add inline URL update JavaScript to each slider's oninput handler:

    For each slider input (year min/max, height min/max, floors min/max):
    1. After the existing oninput handler code that updates the Rust signal
    2. Add inline JavaScript that:
       - Reads current filter values from the signals (via the JavaScript binding)
       - Reads view mode and heatmap state from localStorage
       - Builds URLSearchParams with all current values
       - Calls history.replaceState() to update URL without page reload

    Pattern to follow (from ViewToggle):
    ```javascript
    const params = new URLSearchParams();
    params.set('year_min', currentValue);
    params.set('year_max', currentValue);
    // ... set all filter params
    params.set('view', localStorage.getItem('viewMode') || '3d');
    if (localStorage.getItem('densityHeatmapEnabled') === 'true') {
        params.set('heatmap', 'true');
    }
    const url = new URL(window.location.href);
    url.search = params.toString();
    window.history.replaceState({state: 'urlStateUpdated'}, '', url);
    ```

    IMPORTANT: Do NOT rely on use_effect for URL updates - it only runs on mount. The URL update MUST be in the oninput handler to trigger on every slider change.

    Create a helper function build_update_url_script() that returns the JavaScript to update URL, then call it inline in each oninput handler after the signal update.
  </action>
  <verify>
    <automated>cargo test --package iou-frontend --lib filter_panel_3d 2>&1 | head -20</automated>
  </verify>
  <done>All slider oninput handlers include inline JavaScript to update URL immediately on change</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <what-built>Added inline URL state persistence to filter controls - URL updates immediately when sliders change</what-built>
  <how-to-verify>
    1. Start dev server: dx serve
    2. Open http://localhost:8080/apps/data-verkenner
    3. Adjust year range slider (e.g., 1950-2000)
    4. Verify: Browser URL updates IMMEDIATELY with year_min=1950&year_max=2000
    5. Adjust height range slider (e.g., 10-50)
    6. Verify: URL updates IMMEDIATELY with height_min=10&height_max=50
    7. Adjust floor count slider (e.g., 2-8)
    8. Verify: URL updates IMMEDIATELY with floors_min=2&floors_max=8
    9. Verify: All params are preserved together (not replaced when changing a different slider)
    10. Copy the full URL with all params
    11. Open in new browser tab or incognito window
    12. Verify: Filter values are restored from URL (this will be verified in GAP-04)
  </how-to-verify>
  <resume-signal>Type "approved" if URL updates immediately with filter changes, or describe issues</resume-signal>
</task>

</tasks>

<verification>
After checkpoint approval:
- URL updates immediately when any filter slider changes (no delay)
- URL contains all filter parameters
- URL params are human-readable (not base64)
- Existing view and heatmap params are preserved during filter updates
</verification>

<success_criteria>
- [ ] Year slider changes update URL immediately with year_min/year_max
- [ ] Height slider changes update URL immediately with height_min/height_max
- [ ] Floor slider changes update URL immediately with floors_min/floors_max
- [ ] All filter params visible in URL simultaneously
- [ ] View and heatmap params preserved during filter updates
- [ ] URL update happens on slider drag (not just on release)
</success_criteria>

<output>
After completion, create `.planning/phases/2-3d-buildings-enhancements/2-GAP-03-SUMMARY.md`
</output>
