---
phase: 2-3d-buildings-enhancements
plan: GAP-05
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/iou-frontend/Dioxus.toml
  - crates/iou-frontend/src/components/density_heatmap.rs
autonomous: false
requirements: [DENS-01, DENS-02, DENS-03]
user_setup:
  - service: development
    why: "Dioxus dev server proxy needs to forward API requests to backend"
    env_vars: []
    dashboard_config: []
    notes: "Verify backend API is running on localhost:8000 when testing"

must_haves:
  truths:
    - "Density heatmap API requests return JSON, not HTML"
    - "No console errors about 'Unexpected token <!DOCTYPE'"
    - "Heatmap displays density data correctly"
  artifacts:
    - path: "crates/iou-frontend/Dioxus.toml"
      provides: "Fixed proxy configuration for /api/* requests"
      min_lines: 30
    - path: "crates/iou-frontend/src/components/density_heatmap.rs"
      provides: "Updated fetch URLs for API requests"
      min_lines: 500
  key_links:
    - from: "density_heatmap.js fetch(/api/buildings-3d)"
      to: "backend API at localhost:8000"
      via: "Dioxus dev server proxy configuration or absolute URL"
      pattern: "fetch.*api/buildings-3d|localhost:8000"
---

<objective>
Fix Dioxus proxy configuration so /api/* requests are forwarded to backend API server

Purpose: Density heatmap can fetch building data from API without receiving HTML fallback
Output: Working API proxy configuration or absolute URL fallback
</objective>

<execution_context>
@/Users/marc/.claude/get-shit-down/workflows/execute-plan.md
@/Users/marc/.claude/get-shit-down/templates/summary.md
</execution_context>

<context>
@.planning/phases/2-3d-buildings-enhancements/2-UAT.md

# The Problem

From UAT.md Gap 4:
"User reported: Dioxus dev server proxy not forwarding /api/* requests to backend API"

Console error: "Density update error: SyntaxError: Unexpected token '<', "<!DOCTYPE "... is not valid JSON - API returning HTML instead of JSON"

Root cause: Dioxus dev server proxy setting in Dioxus.toml is not forwarding /api/* requests to backend API. The proxy setting exists but doesn't work properly.

# Current Configuration

From Dioxus.toml:
```toml
[web.app]
title = "IOU-Modern - Informatie Ondersteunde Werkomgeving"
proxy = "http://localhost:8000"
```

The proxy setting exists but may not be correctly configured for path-based forwarding.

# Two-Approach Solution

1. First: Try to fix Dioxus.toml proxy configuration
2. Second: If proxy doesn't work, use absolute localhost:8000 URLs in fetch calls
</context>

<tasks>

<task type="auto">
  <name>Task 1: Attempt to fix Dioxus.toml proxy configuration</name>
  <files>crates/iou-frontend/Dioxus.toml</files>
  <action>
    Verify and update the proxy configuration in Dioxus.toml:

    1. Ensure the proxy setting is under [web.app] section
    2. Verify format: `proxy = "http://localhost:8000"`
    3. Try adding proxy_backend if available: `proxy_backend = "api"`

    If the configuration looks correct but still doesn't work after testing, proceed to Task 2 as fallback.

    Dioxus CLI proxy should forward requests to the backend. The setting should be:
    ```toml
    [web.app]
    title = "IOU-Modern - Informatie Ondersteunde Werkomgeving"
    proxy = "http://localhost:8000"
    ```

    Document the final configuration state in comments.
  </action>
  <verify>
    <automated>grep -A2 "\[web.app\]" crates/iou-frontend/Dioxus.toml | grep proxy || echo "No proxy found"</automated>
  </verify>
  <done>Dioxus.toml proxy configuration verified and documented (may need fallback to absolute URLs)</done>
</task>

<task type="auto">
  <name>Task 2: Update API fetch URLs to use absolute localhost URL</name>
  <files>crates/iou-frontend/src/components/density_heatmap.rs</files>
  <action>
    Update fetch URLs to use absolute backend URL for development:

    In build_density_calculation_script() and build_setup_density_update_script():
    Change: `fetch(\`/api/buildings-3d?bbox-wgs84=\${bboxWgs84}\`)`
    To: `fetch(\`http://localhost:8000/api/buildings-3d?bbox-wgs84=\${bboxWgs84}\`)`

    This ensures requests go directly to the backend API during development, bypassing any proxy issues.

    Add a comment noting this is development-only and production should use relative URLs with proper proxy.
  </action>
  <verify>
    <automated>grep -n "fetch.*api/buildings-3d" crates/iou-frontend/src/components/density_heatmap.rs | grep "localhost:8000" || echo "No absolute URL found"</automated>
  </verify>
  <done>API fetch URLs use absolute localhost:8000 URL for development</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <what-built>Fixed API request routing - density requests return JSON not HTML</what-built>
  <how-to-verify>
    1. Ensure backend API is running: cargo run --bin iou-api (should listen on localhost:8000)
    2. Verify API works: curl http://localhost:8000/api/buildings-3d?bbox-wgs84=test (should return JSON or proper error, not HTML)
    3. Start dev server: dx serve
    4. Open http://localhost:8080/apps/data-verkenner
    5. Click "Densiteitskaart" (Density Heatmap) button
    6. Verify: No console error "Unexpected token <!DOCTYPE"
    7. Verify: Heatmap overlay appears with density data
    8. Open browser DevTools Network tab
    9. Verify: /api/buildings-3d requests show 200 status (or 400 for bad bbox - that's OK)
    10. Verify: Response is JSON (starts with "{", not "<!DOCTYPE")
  </how-to-verify>
  <resume-signal>Type "approved" if API requests return JSON and heatmap works, or describe proxy issues</resume-signal>
</task>

</tasks>

<verification>
After checkpoint approval:
- API requests return JSON, not HTML
- No "Unexpected token" errors in console
- Heatmap displays density data correctly
- Density updates on pan/zoom work
</verification>

<success_criteria>
- [ ] Backend API running on localhost:8000
- [ ] Dioxus proxy forwards /api/* requests correctly OR absolute URLs are used
- [ ] density_heatmap fetch returns JSON (verified by checking response starts with "{")
- [ ] No HTML <!DOCTYPE responses in console errors
- [ ] Heatmap displays with correct color gradient
- [ ] Density data updates on viewport change
</success_criteria>

<output>
After completion, create `.planning/phases/2-3d-buildings-enhancements/2-GAP-05-SUMMARY.md`
</output>
