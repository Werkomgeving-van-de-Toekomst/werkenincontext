# Roadmap: 3D Buildings Enhancements Phase 2

## Overview

Phase 1 established the foundation with 8 sections delivering basic 3D building visualization, dynamic loading, height-based coloring, and interactive popups. Phase 2 enhances data exploration capabilities through four incremental feature sets: building filtering (FILT-01 to FILT-06), 2D/3D view toggling (VIEW-01 to VIEW-03), density heatmap visualization (DENS-01 to DENS-03), and polish improvements (POLI-01 to POLI-03).

Each phase delivers a complete, verifiable capability. The architecture follows the research recommendations: client-side filtering via MapLibre's `setFilter()` for performance, single-layer architecture for view toggling to prevent state desynchronization, and buffered tile calculations for density analysis to avoid edge artifacts.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [ ] **Phase 2.1: Building Filtering** - Interactive filters for year, height, and floor count
- [ ] **Phase 2.2: View Toggle** - Switch between 2D footprint and 3D extrusion views
- [ ] **Phase 2.3: Density Analysis** - Heatmap overlay showing building density patterns
- [ ] **Phase 2.4: Polish** - URL state persistence, animations, design system styling

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
**Plans**: 3 plans

Plans:
- [ ] 2.1-01: Create BuildingFilter state struct and FilterPanel Dioxus component with slider controls
- [ ] 2.1-02: Implement MapLibre filter expression builder and setFilter() integration
- [ ] 2.1-03: Add visible building count display and clear filters button

### Phase 2.2: View Toggle
**Goal**: Users can switch between 2D footprint view and 3D extrusion view
**Depends on**: Phase 2.1
**Requirements**: VIEW-01, VIEW-02, VIEW-03
**Success Criteria** (what must be TRUE):
  1. User can click toggle button and map switches between 2D (footprints) and 3D (extruded) views
  2. Toggle state persists when user refreshes browser page
  3. Clicking building after toggle shows correct popup data (no state desync)
**Plans**: 2 plans

Plans:
- [ ] 2.2-01: Create ViewToggle component and implement single-layer fill-extrusion height switching
- [ ] 2.2-02: Add localStorage persistence for toggle state and verify event handler consistency

### Phase 2.3: Density Analysis
**Goal**: Users can visualize building density patterns through a heatmap overlay
**Depends on**: Phase 2.2
**Requirements**: DENS-01, DENS-02, DENS-03
**Success Criteria** (what must be TRUE):
  1. User can enable density heatmap overlay and see color gradient indicating density
  2. Heatmap colors transition from low (light) to high (dark/intense) density
  3. No visible seams or discontinuities at tile boundaries when panning map
**Plans**: 2 plans

Plans:
- [ ] 2.3-01: Implement density calculation with buffered tile regions and grid-based aggregation
- [ ] 2.3-02: Create DensityHeatmap component with MapLibre heatmap layer and enable toggle

### Phase 2.4: Polish
**Goal**: Complete production-ready UX with URL state sharing, smooth animations, and consistent styling
**Depends on**: Phase 2.3
**Requirements**: POLI-01, POLI-02, POLI-03
**Success Criteria** (what must be TRUE):
  1. User can share URL and recipient sees same filter/view state
  2. View transitions (2D/3D toggle) animate smoothly without jarring jumps
  3. All filter controls and buttons match design system styling
**Plans**: 2 plans

Plans:
- [ ] 2.4-01: Implement URL query parameter encoding/decoding for filter and view state
- [ ] 2.4-02: Add CSS transitions for view toggles and apply design system styles to all UI components

## Progress

**Execution Order:**
Phases execute in numeric order: 2.1 → 2.2 → 2.3 → 2.4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 2.1. Building Filtering | 0/3 | Not started | - |
| 2.2. View Toggle | 0/2 | Not started | - |
| 2.3. Density Analysis | 0/2 | Not started | - |
| 2.4. Polish | 0/2 | Not started | - |

## Complexity Indicators

| Phase | Complexity | Notes |
|-------|------------|-------|
| 2.1 | Medium | Filter expressions straightforward, but must avoid re-render cascade (PITFALL-01) |
| 2.2 | Low | Single-layer architecture simpler than dual-layer, state management is key |
| 2.3 | High | Buffered tile calculations require careful spatial logic (PITFALL-03) |
| 2.4 | Low | URL serialization and CSS animations are standard patterns |

## Integration Checkpoints

| After Phase | Verification |
|-------------|--------------|
| 2.1 | Profile filter changes verify <16ms frame time, no geometry reprocessing spikes |
| 2.2 | Test 20+ rapid toggles, verify popup data always matches clicked building |
| 2.3 | Visual inspection of tile boundaries, automated density continuity test |
| 2.4 | Cross-browser testing (Chrome, Firefox, Safari), URL sharing test |

## Dependencies

```
Phase 1 (Completed)
    ↓
Phase 2.1: Building Filtering
    ↓
Phase 2.2: View Toggle
    ↓
Phase 2.3: Density Analysis
    ↓
Phase 2.4: Polish
    ↓
Phase 2 Complete
```

**External Dependencies:**
- 3DBAG API (CityJSON format, rate limits)
- MapLibre GL JS 4.7.0 (filter expression support)
- Dioxus 0.7 (WASM frontend framework)

## Risk Mitigation

| Risk | Mitigation | Phase |
|------|------------|-------|
| Filter re-render cascade | Use `setFilter()` not layer recreation | 2.1 |
| View toggle state desync | Single-layer architecture, not dual layers | 2.2 |
| Density tile artifacts | Buffered calculations with overlap | 2.3 |
| Viewport race conditions | AbortController for pending requests | 2.1 |
| Filter expression complexity | Client-side simple expressions only | 2.1 |
