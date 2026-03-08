# Requirements: 3D Buildings Enhancements Phase 2

**Defined:** 2026-03-08
**Core Value:** Users can explore and analyze 3D building data interactively — filtering by properties, toggling between 2D/3D views, and visualizing patterns through density heatmaps.

## v1 Requirements

Requirements for Phase 2 release. Each maps to roadmap phases.

### Filtering

- [x] **FILT-01**: User can filter buildings by construction year range (min/max sliders)
- [x] **FILT-02**: User can filter buildings by height range (min/max sliders)
- [x] **FILT-03**: User can filter buildings by floor count range (min/max sliders)
- [x] **FILT-04**: User can clear all filters with single button
- [x] **FILT-05**: Filter results display visible building count
- [x] **FILT-06**: Filters use MapLibre `setFilter()` for performance (no re-render cascade)

### View Toggle

- [x] **VIEW-01**: User can toggle between 2D footprint and 3D extrusion views
- [x] **VIEW-02**: Toggle state persists across browser session
- [x] **VIEW-03**: Single-layer architecture prevents state desync

### Density Analysis

- [ ] **DENS-01**: User can enable density heatmap overlay
- [ ] **DENS-02**: Heatmap colors indicate building density (low → high)
- [ ] **DENS-03**: Heatmap uses buffered tile calculations to avoid edge artifacts

### Polish

- [ ] **POLI-01**: Filter state persists in URL query parameters
- [ ] **POLI-02**: View transitions have smooth animations
- [ ] **POLI-03**: UI matches design system styling

## v2 Requirements

Deferred to future release.

### Texture Mapping

- **TEX-01**: Buildings display satellite/aerial imagery textures
- **TEX-02**: Texture manager handles memory disposal

### Export

- **EXPT-01**: User can export filtered building data as CSV
- **EXPT-02**: User can export current view as image

## Out of Scope

| Feature | Reason |
|---------|--------|
| Building interior modeling | 3DBAG provides exterior shells only |
| Real-time data streaming | Static dataset with periodic refresh is sufficient |
| User accounts/permissions | Public access without authentication |
| Mobile apps | Web-first approach, responsive design only |
| VR/AR support | Out of scope for data visualization platform |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FILT-01 | Phase 2.1 | Complete (2026-03-08) |
| FILT-02 | Phase 2.1 | Complete (2026-03-08) |
| FILT-03 | Phase 2.1 | Complete (2026-03-08) |
| FILT-04 | Phase 2.1 | Complete (2026-03-08) |
| FILT-05 | Phase 2.1 | Complete (2026-03-08) |
| FILT-06 | Phase 2.1 | Complete (2026-03-08) |
| VIEW-01 | Phase 2.2 | Complete |
| VIEW-02 | Phase 2.2 | Complete |
| VIEW-03 | Phase 2.2 | Complete |
| DENS-01 | Phase 2.3 | Pending |
| DENS-02 | Phase 2.3 | Pending |
| DENS-03 | Phase 2.3 | Pending |
| POLI-01 | Phase 2.4 | Pending |
| POLI-02 | Phase 2.4 | Pending |
| POLI-03 | Phase 2.4 | Pending |

**Coverage:**
- v1 requirements: 15 total
- Mapped to phases: 15
- Unmapped: 0 ✓
- Complete: 6 (40%)

---
*Requirements defined: 2026-03-08*
*Last updated: 2026-03-08 after Phase 2.1-01 completion*
