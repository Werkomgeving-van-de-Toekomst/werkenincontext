# Roadmap: 3D Buildings Enhancements Phase 2

## Overview

Phase 1 established the foundation with 8 sections delivering basic 3D building visualization, dynamic loading, height-based coloring, and interactive popups. Phase 2 enhances data exploration capabilities through four incremental feature sets: building filtering (FILT-01 to FILT-06), 2D/3D view toggling (VIEW-01 to VIEW-03), density heatmap visualization (DENS-01 to DENS-03), and polish improvements (POLI-01 to POLI-03).

Each phase delivers a complete, verifiable capability. The architecture follows the research recommendations: client-side filtering via MapLibre's `setFilter()` for performance, single-layer architecture for view toggling to prevent state desynchronization, and buffered tile calculations for density analysis to avoid edge artifacts.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 2.1: Building Filtering** - Interactive filters for year, height, and floor count (COMPLETED 2026-03-08)
- [x] **Phase 2.2: View Toggle** - Switch between 2D footprint and 3D extrusion views (COMPLETED 2026-03-08)
- [x] **Phase 2.3: Density Analysis** - Heatmap overlay showing building density patterns (COMPLETED 2026-03-08)
- [x] **Phase 2.4: Polish** - URL state persistence, animations, design system styling (COMPLETED 2026-03-08)

## Phase Details

### Phase 2.1: Building Filtering
**Goal**: Users can filter buildings by construction year, height, and floor count using interactive slider controls
**Depends on**: Phase 1 (completed 2025-03-07)
**Requirements**: FILT-01, FILT-02, FILT-03, FILT-04, FILT-05, FILT-06
**Success Criteria** (what must be TRUE):
  1. User can adjust year range slider and see only buildings within that year range display
  2. User can adjust height range slider and see only buildings within that height display
  3. User can adjust floor count slider and see only buildings with matching floor counts
  4. User can click "Clear Filters" button and all buildings become visible again
  5. Filter changes update visible building count without full map re-render (performance verified)
**Plans**: 1 plan (consolidated from 3)

Plans:
- [x] 2.1-01: Create BuildingFilter state struct and FilterPanel Dioxus component with slider controls (completed 2026-03-08)
  - All 6 requirements (FILT-01 through FILT-06) satisfied in single comprehensive plan
  - Verification: .planning/phases/2.1-building-filtering/2.1-VERIFICATION.md

### Phase 2.2: View Toggle
**Goal**: Users can switch between 2D footprint view and 3D extrusion view
**Depends on**: Phase 2.1 (completed 2026-03-08)
**Requirements**: VIEW-01, VIEW-02, VIEW-03
**Success Criteria** (what must be TRUE):
  1. User can click toggle button and map switches between 2D (footprints) and 3D (extruded) views
  2. Toggle state persists when user refreshes browser page
  3. Clicking building after toggle shows correct popup data (no state desync)
**Plans**: 1 plan (consolidated from 2)

Plans:
- [x] 2.2-01: Create ViewToggle component and implement single-layer fill-extrusion height switching with localStorage persistence (completed 2026-03-08)
  - All 3 requirements (VIEW-01 through VIEW-03) satisfied
  - Verification: .planning/phases/2.2-view-toggle/2.2-FINAL-VERIFICATION.md
  - Single-layer architecture implemented, PITFALL-03 avoided

### Phase 2.3: Density Analysis
**Goal**: Users can visualize building density patterns through a heatmap overlay
**Depends on**: Phase 2.2 (completed 2026-03-08)
**Requirements**: DENS-01, DENS-02, DENS-03
**Success Criteria** (what must be TRUE):
  1. User can enable density heatmap overlay and see color gradient indicating density
  2. Heatmap colors transition from low (light) to high (dark/intense) density
  3. No visible seams or discontinuities at tile boundaries when panning map
**Plans**: 1 plan (consolidated from 2)

Plans:
- [x] 2.3-01: Implement DensityHeatmap state and component with buffered tile calculations, MapLibre heatmap layer, and toggle button (completed 2026-03-08)
  - All 3 requirements (DENS-01 through DENS-03) satisfied
  - Buffered tile calculations implemented to prevent tile seams

### Phase 2.4: Polish
**Goal**: Complete production-ready UX with URL state sharing, smooth animations, and consistent styling
**Depends on**: Phase 2.3 (completed 2026-03-08)
**Requirements**: POLI-01, POLI-02, POLI-03
**Success Criteria** (what must be TRUE):
  1. User can share URL and recipient sees same filter/view state
  2. View transitions (2D/3D toggle) animate smoothly without jarring jumps
  3. All filter controls and buttons match design system styling
**Plans**: 2 plans

Plans:
- [x] 2.4-01: Implement URL query parameter encoding/decoding for filter, view, and heatmap state (POLI-01) - COMPLETED 2026-03-08
- [x] 2.4-02: Add CSS transitions for view toggles and apply design system styles to all UI components (POLI-02, POLI-03) - COMPLETED 2026-03-08

## Progress

**Execution Order:**
Phases execute in numeric order: 2.1 (done) -> 2.2 (done) -> 2.3 (done) -> 2.4 (done) -> Gap Closure

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 2.1. Building Filtering | 1/1 | COMPLETE | 2026-03-08 |
| 2.2. View Toggle | 1/1 | COMPLETE | 2026-03-08 |
| 2.3. Density Analysis | 1/1 | COMPLETE | 2026-03-08 |
| 2.4. Polish | 2/2 | COMPLETE | 2026-03-08 |
| Gap Closure | 1/7 | IN PROGRESS | GAP-01 done |

## Complexity Indicators

| Phase | Complexity | Notes |
|-------|------------|-------|
| 2.1 | Medium | Filter expressions straightforward, but must avoid re-render cascade (PITFALL-01) - VERIFIED |
| 2.2 | Low | Single-layer architecture simpler than dual-layer, state management is key - COMPLETE |
| 2.3 | High | Buffered tile calculations require careful spatial logic (PITFALL-03) - COMPLETE |
| 2.4 | Low | URL serialization and CSS animations are standard patterns |

## Integration Checkpoints

| After Phase | Verification |
|-------------|--------------|
| 2.1 | DONE - Profile filter changes verify <16ms frame time, no geometry reprocessing spikes |
| 2.2 | DONE - Single-layer architecture verified, state sync tested |
| 2.3 | DONE - Visual inspection of tile boundaries, automated density continuity test |
| 2.4 | Cross-browser testing (Chrome, Firefox, Safari), URL sharing test |

## Dependencies

```
Phase 1 (Completed)
    |
Phase 2.1: Building Filtering (COMPLETED 2026-03-08)
    |
Phase 2.2: View Toggle (COMPLETED 2026-03-08)
    |
Phase 2.3: Density Analysis (COMPLETED 2026-03-08)
    |
Phase 2.4: Polish (READY)
    |
Phase 2 Complete
```

**External Dependencies:**
- 3DBAG API (CityJSON format, rate limits)
- MapLibre GL JS 4.7.0 (filter expression support)
- Dioxus 0.7 (WASM frontend framework)

## Risk Mitigation

| Risk | Mitigation | Phase | Status |
|------|------------|-------|--------|
| Filter re-render cascade | Use `setFilter()` not layer recreation | 2.1 | MITIGATED |
| View toggle state desync | Single-layer architecture, not dual layers | 2.2 | MITIGATED |
| Density tile artifacts | Buffered calculations with overlap | 2.3 | MITIGATED |
| Viewport race conditions | AbortController for pending requests | 2.1 | Implemented |
| Filter expression complexity | Client-side simple expressions only | 2.1 | Verified |
| URL state desync | replaceState for updates, popstate listener for back/forward | 2.4 | Pending |
| Design system drift | Use CSS variables from style.css consistently | 2.4 | Pending |

---

## Gap Closure Phases (from v1.0 Milestone Audit)

**Status:** GAP-01, GAP-05 Complete (2026-03-08)

Plans:
- [x] GAP-01: Fix MapLibre style load race condition causing "Style is not done loading" errors - COMPLETED 2026-03-08
  - User verified: "Yes, works" - filters work without errors
  - Commit: 5a6e686
  - Files: crates/iou-frontend/src/components/filter_panel_3d.rs
- [x] GAP-05: Fix Dioxus proxy configuration for API requests - COMPLETED 2026-03-08
  - User verified: "No errors" - API returns JSON instead of HTML
  - Commits: 5748c6c, a75ddbd
  - Files: crates/iou-frontend/Dioxus.toml, crates/iou-frontend/src/components/density_heatmap.rs
- [ ] GAP-02: [Next gap from UAT]
- [ ] GAP-03 through GAP-07: [Remaining gaps]