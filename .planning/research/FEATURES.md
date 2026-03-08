# Feature Research

**Domain:** 3D Building Data Visualization
**Researched:** 2026-03-08
**Confidence:** MEDIUM

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Property-based filtering** | Users exploring building data expect to filter by height, year, floors | MEDIUM | MapLibre supports `filter` property; requires UI controls |
| **Height range slider** | Standard for exploring building height distributions | LOW | Dual-handle slider implementation |
| **Year range filter** | Construction year analysis is core use case for urban planning | LOW | Similar to height slider; year data available |
| **Floor count filter** | Building scale filtering (low-rise vs high-rise) | LOW | Integer-based filtering |
| **Clear filters button** | Users need quick way to reset to full dataset | LOW | Simple UI control |
| **Visible count indicator** | Users expect to know how many buildings match current filters | LOW | Display count after filter application |
| **2D/3D view toggle** | Standard in 3D map tools; users need to switch views | MEDIUM | Toggle between `fill-extrusion` and `fill` layers |
| **Click-to-inspect** | Already implemented; expected for any interactive map | LOW | Popup with building properties |
| **Responsive filter UI** | Filters must work on mobile/tablet | MEDIUM | Collapsible panel or drawer |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Dynamic density heatmap** | Visualizes building density patterns without user configuration | HIGH | Grid-based aggregation, color gradient, shader work |
| **Texture mapping with satellite imagery** | Photorealistic building surfaces for context awareness | HIGH | MapLibre has limited support; may need custom shaders |
| **Filter combinations persistence** | Shareable filter states via URL parameters | MEDIUM | URL state sync, bookmarkable views |
| **Animated filter transitions** | Smooth visual feedback when filters change | MEDIUM | CSS transitions or tweening for color/height |
| **Comparison mode** | Side-by-side filtered views (e.g., pre-2000 vs post-2000) | HIGH | Dual viewport or split-screen |
| **Export filtered data** | Download building subset as GeoJSON/CSV | MEDIUM | Client-side export from filtered features |
| **Building clustering at scale** | Automatic grouping for dense urban areas | HIGH | Custom clustering algorithm for 3D geometries |
| **Shadow analysis** | Time-based shadow visualization for urban planning | VERY HIGH | Requires sun position calculation, custom shaders |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Real-time data streaming** | "Live" building updates seems valuable | 3DBAG is static; updates are periodic; adds complexity | Periodic data refresh (daily/weekly) |
| **Building interior modeling** | Complete 3D experience | 3DBAG provides exterior shells only; interior data unavailable | Focus on exterior analysis |
| **VR/AR support** | Immersive building exploration | Requires different rendering pipeline; limited browser support | Web-first 3D with good camera controls |
| **Social sharing of views** | "Share this building" feature | Requires backend infrastructure; authentication complexity | URL-based state sharing (no accounts) |
| **Custom color palettes per user** | Personalization appeal | Requires user accounts; state management complexity | Well-designed default color schemes |
| **Measurement tools (distance/area)** | CAD-like functionality | Complex UI interaction; conflicts with map navigation | Export data to external GIS tools |
| **Historical timeline slider** | "See buildings over time" | Requires multiple historical datasets; data availability unlikely | Static current-state visualization with year filter |

## Feature Dependencies

```
[Property-based filtering]
    ├──requires──> [Filter UI controls]
    └──requires──> [Building properties in dataset]

[2D/3D view toggle]
    └──requires──> [Dual layer configuration (fill + fill-extrusion)]

[Texture mapping]
    ├──requires──> [Satellite imagery tile source]
    └──enhances──> [2D/3D view toggle]

[Density heatmap]
    ├──requires──> [Grid aggregation backend]
    └──conflicts──> [Individual building highlighting]
         (density visualization vs individual features)

[Filter persistence via URL]
    └──enhances──> [Property-based filtering]
```

### Dependency Notes

- **Property-based filtering requires filter UI controls:** Users need sliders/inputs to set filter ranges. Without UI, filtering is inaccessible.
- **Property-based filtering requires building properties in dataset:** Confirmed available from 3DBAG API (height, construction_year, floor_count).
- **2D/3D view toggle requires dual layer configuration:** MapLibre needs separate `fill` (2D) and `fill-extrusion` (3D) layers. Toggle switches visibility between them.
- **Texture mapping requires satellite imagery tile source:** Need aerial/satellite tile provider (e.g., Esri World Imagery, Mapbox Satellite). MapLibre's texturing support for extruded layers is limited—may need custom shaders.
- **Texture mapping enhances 2D/3D view toggle:** Textures apply to both 2D and 3D, making the toggle more visually impactful.
- **Density heatmap conflicts with individual building highlighting:** When showing aggregated density, individual buildings are obscured. These are mutually exclusive visualization modes.

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [ ] **Property-based filtering (height, year, floors)** — Core data exploration feature; enables "find all buildings built after 2000 over 50m tall"
- [ ] **Filter UI controls** — Sliders for ranges; essential for usability
- [ ] **Visible count indicator** — Users need feedback on filter results
- [ ] **Clear filters button** — Standard UX pattern
- [ ] **2D/3D view toggle** — Basic view switching; users expect it
- [ ] **Responsive filter panel** — Works on desktop and mobile

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] **Filter combinations persistence (URL)** — Trigger: Users request sharing links; enables bookmarking
- [ ] **Animated filter transitions** — Trigger: Polish phase; improves perceived responsiveness
- [ ] **Export filtered data** — Trigger: Users need external analysis; simple GeoJSON export
- [ ] **Dynamic density heatmap** — Trigger: Users ask for "big picture" patterns; requires backend aggregation

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Texture mapping** — Defer: High complexity, limited MapLibre support, may not add core value
- [ ] **Building clustering at scale** — Defer: Only needed if performance degrades with >10K buildings
- [ ] **Comparison mode** — Defer: Advanced use case; validate single-view first
- [ ] **Shadow analysis** — Defer: Very high complexity; niche use case

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Property-based filtering | HIGH | MEDIUM | P1 |
| Filter UI controls | HIGH | LOW | P1 |
| 2D/3D view toggle | HIGH | MEDIUM | P1 |
| Visible count indicator | MEDIUM | LOW | P1 |
| Clear filters button | MEDIUM | LOW | P1 |
| Responsive filter panel | HIGH | MEDIUM | P1 |
| Dynamic density heatmap | MEDIUM | HIGH | P2 |
| Filter persistence (URL) | MEDIUM | MEDIUM | P2 |
| Animated filter transitions | LOW | MEDIUM | P2 |
| Export filtered data | MEDIUM | LOW | P2 |
| Texture mapping | LOW | HIGH | P3 |
| Building clustering | LOW | HIGH | P3 |
| Comparison mode | LOW | HIGH | P3 |
| Shadow analysis | LOW | VERY HIGH | P3 |

**Priority key:**
- P1: Must have for launch (core filtering + view toggle)
- P2: Should have, add when possible (enhanced UX, density)
- P3: Nice to have, future consideration (advanced visualizations)

## Competitor Feature Analysis

| Feature | Mapbox GL JS | CesiumJS | Our Approach |
|---------|--------------|----------|--------------|
| 3D building filtering | Supported via `filter` property | Supported via `EntityCollection` | Use MapLibre `filter` with expression syntax |
| 2D/3D toggle | Common pattern | Common pattern | Switch between `fill` and `fill-extrusion` layers |
| Texture mapping | Limited for extrusions | Supported via `Material` | Defer to v2; MapLibre limitation |
| Density heatmap | Via `heatmap` layer (2D only) | Via `HeatmapImageryProvider` | Custom grid aggregation + gradient fill |
| View persistence | URL hash/state | Camera state in URL | Filter state in URL query params |

**Key differentiator:** Our focus on Dutch building data (3DBAG) with specific properties like construction year and floor count, combined with a clean, filter-centric interface optimized for urban planning analysis.

## Sources

- Mapbox GL JS documentation (filter expressions, fill-extrusion layers) — MEDIUM confidence (based on official docs patterns)
- CesiumJS documentation (3D building examples) — LOW confidence (searches failed, general knowledge)
- MapLibre GL JS documentation (filtering API) — HIGH confidence (existing codebase uses MapLibre)
- General 3D GIS visualization patterns — MEDIUM confidence (industry standard practices)
- Existing codebase analysis — HIGH confidence (verified current implementation)

**Research limitations:** Web searches had limited success due to rate limiting. Recommendations based on:
1. Official MapLibre GL JS documentation patterns (verified in existing code)
2. Standard GIS visualization practices
3. Existing feature set analysis
4. Known MapLibre capabilities from Phase 1 implementation

**Validation needed:** Texture mapping feasibility with MapLibre requires deeper technical research before implementation.

---
*Feature research for: 3D Building Data Visualization*
*Researched: 2026-03-08*
