# Stack Research

**Domain:** 3D Building Visualization Enhancements (Filtering, Texturing, Density Analysis, 2D/3D Toggle)
**Researched:** 2025-03-08
**Confidence:** HIGH

## Executive Summary

For implementing building filters, texture mapping, density visualization, and 2D/3D view toggling in a MapLibre GL JS + Dioxus WASM application, the recommended stack leverages **native MapLibre GL JS v4.7.0 capabilities** for most features, avoiding external dependencies. The existing JavaScript interop pattern should be extended rather than introducing new abstraction layers.

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **MapLibre GL JS** | 4.7.0 | 3D rendering, filtering, heatmaps | Already integrated; native support for filters, heatmaps, fill-extrusion with patterns, and camera controls (pitch/bearing). Verified current version in production. |
| **Dioxus** | 0.7 (web feature) | Rust WASM frontend framework | Existing stack; proven JavaScript interop pattern via `eval()` and `window` globals. No need for additional React-style bindings. |
| **wasm-bindgen** | 0.2 | Rust↔JavaScript bridge | Already in use; sufficient for bidirectional communication between Rust and MapLibre. |
| **Axum + DuckDB** | Existing | Backend analytics | Existing backend can precompute density statistics; no changes needed for this phase. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **MapLibre Expressions** | Built-in | Dynamic filtering and styling | Use for all filter logic (year, height, floor range). No external library needed—expressions are part of core MapLibre. |
| **MapLibre Heatmap Layer** | Built-in | Density visualization | Native `heatmap` layer type with `heatmap-color`, `heatmap-weight`, `heatmap-radius`. Use for building density heatmaps. |
| **MapLibre Camera Controls** | Built-in | 2D/3D toggle | Use `map.easeTo({ pitch: 0 })` for 2D, `map.easeTo({ pitch: 60 })` for 3D. No additional library required. |
| **MapLibre fill-extrusion-pattern** | Built-in | Building textures | Pattern support via sprites. Limited but sufficient for basic texturing (roof materials, facades). |
| **Turf.js** | (Optional) | Geospatial calculations | Only needed for advanced density calculations (e.g., buildings-per-hectare). Skip if DuckDB precomputes statistics. |
| **deck.gl** | ❌ NOT recommended | Advanced 3D visualizations | Overkill for this use case. MapLibre's native fill-extrusion + heatmap is sufficient. Deck.gl adds 500KB+ and complexity. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| **MapLibre Style Specification** | API reference | Use official docs: https://maplibre.org/maplibre-gl-js-docs/style-spec/ for filter expressions and paint properties. |
| **MapLibre Examples** | Code patterns | Reference: https://maplibre.org/maplibre-gl-js-docs/example/ — filter symbols, create heatmap, display buildings 3D, set pitch/bearing. |
| **Browser DevTools** | Debugging | MapLibre logs to console; existing `window['map_{}']` global pattern allows manual testing. |
| **wasm-pack** | Build | Existing build toolchain; no changes needed. |

## Installation

```bash
# No additional npm packages required — all features are built into MapLibre GL JS 4.7.0

# Current installation (already in index.html):
# <script src="https://unpkg.com/maplibre-gl@4.7.0/dist/maplibre-gl.js"></script>
# <link href="https://unpkg.com/maplibre-gl@4.7.0/dist/maplibre-gl.css" rel="stylesheet" />

# If Turf.js is needed for advanced density calculations:
# npm install @turf/turf
# Then add to index.html:
# <script src="https://unpkg.com/@turf/turf@6/turf.min.js"></script>
```

## Implementation Approach by Feature

### 1. Building Filters
**Native MapLibre Expressions** — No additional library needed.

```javascript
// Example: Filter by construction year range
map.setFilter('buildings-3d', [
    'all',
    ['>=', ['get', 'construction_year'], 1950],
    ['<=', ['get', 'construction_year'], 2000]
]);

// Example: Filter by height and floor count
map.setFilter('buildings-3d', [
    'all',
    ['>=', ['get', 'height'], 10],
    ['<=', ['get', 'floors'], 5]
]);
```

**Rust Integration**: Extend existing `build_*_script()` pattern in `map_3d.rs` to generate filter update scripts.

### 2. Density Visualization
**Native MapLibre Heatmap Layer** — Convert building centroids to point features.

```javascript
// Add heatmap layer for building density
map.addLayer({
    id: 'building-density',
    type: 'heatmap',
    source: 'building-centroids',
    paint: {
        'heatmap-weight': 1,  // Each building = weight of 1
        'heatmap-intensity': ['interpolate', ['linear'], ['zoom'], 10, 1, 15, 3],
        'heatmap-radius': ['interpolate', ['linear'], ['zoom'], 10, 10, 15, 30],
        'heatmap-color': [
            'interpolate', ['linear'], ['heatmap-density'],
            0, 'rgba(0,0,255,0)',
            0.3, 'blue',
            0.5, 'cyan',
            0.7, 'yellow',
            1, 'red'
        ]
    }
});
```

**Data Preparation**: Backend (Axum + DuckDB) should precompute building centroids from 3DBAG polygon data.

### 3. Texture Mapping
**Native MapLibre fill-extrusion-pattern** — Limited but functional.

```javascript
// Add sprite with texture patterns
map.addImage('roof-pattern', '/static/textures/roof.png');

// Apply pattern to fill-extrusion layer
map.addLayer({
    id: 'buildings-textured',
    type: 'fill-extrusion',
    source: 'buildings',
    paint: {
        'fill-extrusion-pattern': 'roof-pattern',
        'fill-extrusion-height': ['get', 'height'],
        'fill-extrusion-base': 0
    }
});
```

**Limitations**: MapLibre does **NOT** support UV-mapped textures on 3D extrusions. Patterns repeat across surfaces. For realistic textures, use Three.js (not recommended — breaks integration).

**Recommendation**: Use pattern-based texturing for roof materials only. Skip facade texturing due to MapLibre limitations.

### 4. 2D/3D Toggle
**Native MapLibre Camera Controls** — Use `map.easeTo()` to animate pitch.

```javascript
// Toggle 2D/3D view
function toggle2D3D() {
    const pitch = map.getPitch();
    if (pitch > 0) {
        // Switch to 2D
        map.easeTo({ pitch: 0, bearing: 0, duration: 1000 });
    } else {
        // Switch to 3D
        map.easeTo({ pitch: 60, bearing: -17.6, duration: 1000 });
    }
}
```

**Rust Integration**: Add `build_toggle_view_script()` to `map_3d.rs` that generates the toggle function and binds it to a button click.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Native MapLibre filters | deck.gl layers | Only if visualizing >100K buildings with complex 3D shaders. Unnecessary for <10K buildings. |
| Native MapLibre heatmap | Turf.js density | Only if needing custom spatial aggregation (e.g., hexbin, quadtree). DuckDB can precompute this server-side. |
| Native MapLibre patterns | Three.js textures | Only if realistic UV-mapped facades are required. Adds significant complexity and breaks MapLibre integration. |
| Existing Dioxus + eval() | react-map-gl + hooks | Only if rewriting frontend in React. Not worth migration cost. Current pattern works. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **deck.gl** | Adds 500KB+, unnecessary for simple heatmaps and filters. MapLibre's native layers are sufficient. | MapLibre heatmap + fill-extrusion layers |
| **Three.js** | Requires separate WebGL context, breaks MapLibre integration, complex event coordination. | MapLibre fill-extrusion-pattern (limited) or skip texturing |
| **mapbox-gl-js** | Proprietary license, requires API key, MapLibre is the open-source fork. | MapLibre GL JS (already in use) |
| **Leaflet** | No native 3D support, would require plugins. | MapLibre (already has 3D) |
| **Custom WebGL shaders** | Extremely complex, maintenance burden. MapLibre's expression language is sufficient for 99% of use cases. | MapLibre expressions for dynamic styling |
| **React wrapper libraries** | Dioxus is the framework. Adding React wrappers creates dual framework overhead. | Extend existing Dioxus + eval() pattern |

## Stack Patterns by Variant

**If density requires complex spatial aggregation:**
- Use **Turf.js** for client-side hexbin/quadtree clustering
- OR precompute in **DuckDB** server-side (recommended for >10K buildings)
- Because: DuckDB is faster for large datasets, reduces client-side load

**If texturing requires UV mapping:**
- Accept that **MapLibre cannot do UV-mapped textures** on extrusions
- Use **fill-extrusion-pattern** for repeating roof patterns
- OR skip texturing entirely, focus on color-based styling
- Because: Three.js integration breaks MapLibre event handling and adds significant complexity

**If filters need to combine AND/OR logic:**
- Use **MapLibre expression syntax**: `['all', ...]` for AND, `['any', ...]` for OR
- Build expressions dynamically in Rust, serialize to JSON for JavaScript
- Because: Native expressions are faster than client-side JavaScript filtering

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| maplibre-gl@4.7.0 | Dioxus 0.7 | Works via existing `eval()` interop pattern. No direct Dioxus bindings needed. |
| maplibre-gl@4.7.0 | wasm-bindgen 0.2 | Compatible. MapLibre accessed via `window` globals, not direct WASM bindings. |
| maplibre-gl@4.7.0 | Turf.js 6.x | Compatible if needed for advanced geospatial calculations. Optional. |

## Data Flow Architecture

```
┌─────────────┐
│  Dioxus UI  │ (Rust WASM)
└──────┬──────┘
       │ eval() scripts
       ▼
┌─────────────────────────────┐
│  window['map_container']    │ (MapLibre instance)
│  - addLayer()               │
│  - setFilter()              │
│  - easeTo()                 │
│  - addSource()              │
└──────────┬──────────────────┘
           │ HTTP requests
           ▼
┌─────────────────────────────┐
│  Axum API + DuckDB          │ (Backend)
│  - /api/buildings           │
│  - /api/buildings/centroid  │
│  - /api/stats/density       │
└─────────────────────────────┘
```

**Key Patterns**:
1. **Rust generates JavaScript** via `build_*_script()` functions
2. **JavaScript executes via `eval()`** in browser
3. **Events flow back via `window.sendToRust()`** (existing pattern)
4. **No direct WASM bindings** to MapLibre — uses JavaScript bridge

## Performance Considerations

| Feature | < 1K buildings | 1K-10K buildings | > 10K buildings |
|---------|----------------|------------------|-----------------|
| Filters (client) | Fast (native) | Fast (native) | May lag — consider server-side filtering |
| Heatmap | Smooth | Smooth | Use tile-based aggregation (DuckDB) |
| Textures (patterns) | Fast | Moderate overhead | Avoid — patterns cause render lag |
| 2D/3D Toggle | Instant animation | Smooth animation | Precompute tiles for faster transitions |

**Recommendation**: For >10K buildings, precompute density grids in DuckDB and serve as vector tiles. Use MapLibre's `vector` source instead of raw GeoJSON.

## Sources

- **MapLibre GL JS API Reference** — https://maplibre.org/maplibre-gl-js-docs/api/ (Current version: v4.5.0, production uses v4.7.0) — **HIGH confidence**
- **MapLibre Style Specification** — https://maplibre.org/maplibre-gl-js-docs/style-spec/ — Verified filter expressions, heatmap properties, fill-extrusion-pattern support — **HIGH confidence**
- **MapLibre Examples** — https://maplibre.org/maplibre-gl-js-docs/example/ — Verified examples for "Filter symbols by toggling a list", "Create a heatmap layer", "Display buildings in 3D", "Set pitch and bearing" — **HIGH confidence**
- **Existing codebase** — `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/map_3d.rs` — Current MapLibre v4.7.0 integration via `eval()` pattern — **HIGH confidence**
- **Existing codebase** — `/Users/marc/Projecten/iou-modern/crates/iou-frontend/index.html` — MapLibre v4.7.0 CDN links — **HIGH confidence**

---
*Stack research for: 3D Building Visualization Enhancements (Filtering, Texturing, Density Analysis, 2D/3D Toggle)*
*Researched: 2025-03-08*
