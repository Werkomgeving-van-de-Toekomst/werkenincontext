# IOU Modern: 3D Buildings Enhancements Phase 2

## What This Is

A data visualization platform for Dutch government organizations, featuring an interactive 3D map of buildings with height-based coloring, dynamic loading, and click-to-view property details. This phase adds advanced filtering, visual toggles, texturing, and density analysis to enhance data exploration capabilities.

## Core Value

**Users can explore and analyze 3D building data interactively** — filtering by properties, toggling between 2D/3D views, and visualizing patterns through density heatmaps.

## Requirements

### Validated

<!-- Completed from Phase 1 (2025-03-07) -->

- ✓ Accurate coordinate conversion (RD ↔ WGS84 via proj crate) — existing
- ✓ WGS84 bbox API endpoint for frontend queries — existing
- ✓ Building properties: bag_id, height, floors, construction_year — existing
- ✓ Dynamic viewport-based loading with debouncing — existing
- ✓ Height-based coloring (light blue → medium purple → dark purple) — existing
- ✓ Click popups with building information — existing
- ✓ Popup CSS styling with design system consistency — existing

### Active

<!-- Phase 2 enhancements to implement -- -->

- [ ] **Filter controls** — Filter buildings by year, height, floor count
- [ ] **2D/3D toggle** — Switch between 2D footprint and 3D extrusion views
- [ ] **Texture mapping** — Apply satellite/aerial imagery textures to buildings
- [ ] **Density analysis** — Heatmap or clustering visualization of building density

### Out of Scope

- **Building interior modeling** — 3DBAG provides exterior shells only
- **Real-time data streaming** — Static dataset with periodic refresh is sufficient
- **User accounts/permissions** — Public access without authentication
- **Mobile apps** — Web-first approach, responsive design only

## Context

**Existing codebase:**
- Rust backend with Axum API server
- DuckDB for embedded analytics
- Dioxus WASM frontend with MapLibre GL JS
- 3DBAG API integration for Dutch building data

**Phase 1 completed (2025-03-07):**
- 8 sections implemented with code review fixes applied
- All tests passing
- Manual browser testing completed

**Technical constraints:**
- MapLibre GL JS for 3D rendering (limited texturing support)
- 3DBAG API provides CityJSON format only
- Coordinate conversion required (Netherlands RD system)
- Frontend is compiled to WebAssembly

## Constraints

- **MapLibre GL JS limits:** Texturing support is limited; may require custom shaders
- **Browser performance:** Large building counts (1000+) can impact rendering
- **3DBAG API:** External dependency, rate limits may apply
- **WASM compilation:** Dioxus frontend compiles to WebAssembly
- **Time zone:** Netherlands (CET/CEST) for any scheduling

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Filters first implementation | Enables data exploration, most requested feature | — Pending |
| Use MapLibre built-in filtering | Leverages existing platform capabilities | — Pending |
| Heatmap over clustering | Simpler implementation, clearer visualization | — Pending |
| Progressive enhancement | Each feature standalone, can ship independently | — Pending |

---
*Last updated: 2026-03-08 after project initialization*
