---
phase: 2-3d-buildings-enhancements
plan: GAP-04
type: execute
wave: 2
depends_on: [GAP-03]
files_modified:
  - crates/iou-frontend/src/components/filter_panel_3d.rs
autonomous: false
requirements: [POLI-01]
user_setup: []

must_haves:
  truths:
    - "Opening a URL with filter params restores the filter state on page load"
    - "Filter sliders reflect values from URL params"
    - "Map shows filtered buildings based on URL params"
  artifacts:
    - path: "crates/iou-frontend/src/components/filter_panel_3d.rs"
      provides: "Filter panel that reads initial state from URL in Rust"
      min_lines: 420
  key_links:
    - from: "filter_panel_3d.rs use_memo/use_effect"
      to: "URL params"
      via: "web_sys::UrlSearchParams parsed in Rust and set as initial signal values"
      pattern: "UrlSearchParams|window.location.search"
---

<objective>
Add URL state restoration on page load so opening a shared URL restores filter, view, and heatmap state

Purpose: Users can share URLs that restore exact same view when opened
Output: Working URL state restoration
</objective>

<execution_context>
@/Users/marc/.claude/get-shit-down/workflows/execute-plan.md
@/Users/marc/.claude/get-shit-down/templates/summary.md
</execution_context>

<context>
@.planning/phases/2-3d-buildings-enhancements/2-UAT.md
@.planning/phases/2.4-polish/2.4-01-SUMMARY.md

# The Problem

From UAT.md Gap 5:
"User reported: Buildings popping up during load and URL doesn't preserve parameters - shows base URL without query params"

Root cause: Map initialization doesn't restore filter state from URL params on load.

# Why DOM manipulation doesn't work

Setting input.value via JavaScript won't trigger Rust oninput handlers because the event is synthetic and may not propagate through Dioxus's event system correctly.

# The Solution: Parse in Rust, Set Signals Directly

Instead of using JavaScript to manipulate DOM elements, parse URL params in Rust using web_sys::UrlSearchParams and set the signal values directly during component initialization.

This approach:
1. Uses Rust to read window.location.search on mount
2. Parses URL parameters using web_sys::UrlSearchParams
3. Sets signal values directly (min_year.set(), max_year.set(), etc.)
4. Triggers the filter update use_effect which already applies filters
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add URL param restoration using Rust signals</name>
  <files>crates/iou-frontend/src/components/filter_panel_3d.rs</files>
  <action>
    Add URL state restoration using Rust's web_sys::UrlSearchParams:

    1. Add web_sys::UrlSearchParams to the use_effect hook
    2. On mount, read window.location.search
    3. Parse URL parameters using UrlSearchParams::new(&search)
    4. For each filter param (year_min, year_max, height_min, height_max, floors_min, floors_max):
       - Parse the value as i32
       - Set the corresponding signal directly: min_year.set(parsed_value)
    5. After setting all signals, call build_set_filter_script() to apply the filter

    Example pattern:
    ```rust
    let search = window.location().search().unwrap();
    let params = UrlSearchParams::new(&search).unwrap();
    if let Some(year_min_str) = params.get("year_min") {
        if let Ok(val) = year_min_str.parse::<i32>() {
            min_year.set(val);
        }
    }
    // Repeat for all 6 filter params
    ```

    DO NOT use JavaScript DOM manipulation - set signals directly in Rust.
  </action>
  <verify>
    <automated>cargo test --package iou-frontend --lib filter_panel_3d 2>&1 | head -20</automated>
  </verify>
  <done>FilterPanel3D has use_effect that parses URL params in Rust and sets signal values directly</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <what-built>Added URL state restoration using Rust signals - opening URL with params restores filter state</what-built>
  <how-to-verify>
    1. Start dev server: dx serve
    2. Open http://localhost:8080/apps/data-verkenner
    3. Set filters: year=1950-2000, height=10-50, floors=2-5
    4. Copy the URL (should contain all filter params)
    5. Close the tab and open a NEW browser tab (or incognito window)
    6. Paste the URL and navigate
    7. Verify: Page loads with filter sliders set to 1950-2000, 10-50, 2-5
    8. Verify: Map shows only buildings matching those filter values (no buildings appearing/disappearing)
    9. Verify: Building count shows correct filtered count
    10. Verify: No "buildings popping up" during load
  </how-to-verify>
  <resume-signal>Type "approved" if URL state restores correctly, or describe issues</resume-signal>
</task>

</tasks>

<verification>
After checkpoint approval:
- Opening URL with filter params restores filter slider positions via Rust signals
- Map shows filtered buildings based on URL params from initial load
- Building count reflects restored filter state
- No "buildings popping up" during load (filters applied before buildings rendered)
</verification>

<success_criteria>
- [ ] URL params year_min/year_max restore year slider via Rust signals
- [ ] URL params height_min/height_max restore height slider via Rust signals
- [ ] URL params floors_min/floors_max restore floors slider via Rust signals
- [ ] All restored values apply after map loads
- [ ] Building count updates to reflect restored filters
- [ ] No visual artifacts during load
</success_criteria>

<output>
After completion, create `.planning/phases/2-3d-buildings-enhancements/2-GAP-04-SUMMARY.md`
</output>
