# Architecture Research

**Domain:** 3D Building Data Filtering and Visualization
**Researched:** 2026-03-08
**Confidence:** MEDIUM

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Frontend Layer (WASM)                        │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌──────────┐  │
│  │ FilterPanel │  │ ViewToggle  │  │ DensityHeatmap│ │ LayerMgr │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └────┬─────┘  │
│         │                │                │             │          │
│         └────────────────┼────────────────┴─────────────┘          │
│                          ▼                                          │
│                  ┌───────────────┐                                  │
│                  │ BuildingFilter│ (aggregates filters)             │
│                  └───────┬───────┘                                  │
│                          ▼                                          │
│                  ┌───────────────┐                                  │
│                  │ Map3D Wrapper │ (MapLibre bridge)                │
│                  └───────┬───────┘                                  │
└──────────────────────────┼──────────────────────────────────────────┘
                           │ HTTP (fetch)
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Backend Layer (Rust)                         │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│  │ Buildings3D │  │ FilterParam │  │ DensityCalc  │                │
│  │   Route     │  │   Extractor │  │   (future)   │                │
│  └──────┬──────┘  └──────┬──────┘  └─────────────┘                │
│         │                │                                          │
│         └────────────────┼──────────────────┐                       │
│                          ▼                  ▼                       │
│                  ┌───────────────┐  ┌────────────┐                 │
│                  │ 3DBAG Proxy   │  │ DuckDB     │                 │
│                  │ (CityJSON→GEO)│  │ (Analytics)│                 │
│                  └───────┬───────┘  └────────────┘                 │
└──────────────────────────┼──────────────────────────────────────────┘
                           │ HTTP
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      External Services                               │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐                                  │
│  │  3DBAG API  │  │ MapTiler    │ (optional terrain)               │
│  │ (CityJSON)  │  │ Terrain     │                                  │
│  └─────────────┘  └─────────────┘                                  │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| **FilterPanel** | UI controls for year/height/floor range sliders | Dioxus component with reactive state |
| **ViewToggle** | 2D/3D mode switching button | Dioxus component updating layer visibility |
| **DensityHeatmap** | Heatmap layer visualization | MapLibre heatmap layer with density data |
| **BuildingFilter** | Centralized filter state and validation | Rust struct with serde serialization |
| **Map3D Wrapper** | MapLibre JS bridge and layer management | JavaScript interop functions |
| **Buildings3D Route** | Backend filtering endpoint | Axum handler with query params |
| **FilterParam Extractor** | Parse and validate filter parameters | Axum extractor with validation |
| **DensityCalc** | Spatial density calculation (future) | DuckDB spatial queries or post-processing |

## Recommended Project Structure

```
crates/iou-frontend/src/
├── components/
│   ├── map_3d.rs              # Existing MapLibre wrapper
│   ├── filter_panel_3d.rs     # NEW: Filter UI controls
│   ├── view_toggle_3d.rs      # NEW: 2D/3D toggle button
│   ├── density_heatmap.rs     # NEW: Heatmap layer component
│   └── layer_control_3d.rs    # Existing: Layer management
├── state/
│   ├── building_filter.rs     # NEW: Filter state management
│   └── mod.rs                 # Export filter state
└── lib.rs                     # Register new components

crates/iou-api/src/
├── routes/
│   ├── buildings_3d.rs        # Existing: Basic buildings endpoint
│   └── buildings_filter.rs    # NEW: Filter-specific endpoint (optional)
├── models/
│   └── building_filter.rs     # NEW: Filter parameter models
└── main.rs                    # Register new routes
```

### Structure Rationale

- **`components/`**: UI components separate from business logic
  - `filter_panel_3d.rs`: Isolated filter controls (testable, reusable)
  - `view_toggle_3d.rs`: Simple state toggle (minimal dependencies)
  - `density_heatmap.rs`: Complex visualization (isolated complexity)

- **`state/`**: Centralized state management
  - `building_filter.rs`: Single source of truth for filter values
  - Reactive updates trigger map layer re-filtering

- **Backend**:
  - Filter validation on both frontend (UX) and backend (security)
  - Optional dedicated filter endpoint for performance optimization

## Architectural Patterns

### Pattern 1: MapLibre Filter Expressions (Client-Side)

**What:** MapLibre's declarative filtering using expressions in layer paint properties

**When to use:** Small datasets (< 1000 features), simple property comparisons, real-time filtering

**Trade-offs:**
- ✅ Fast response (no network round-trip)
- ✅ Progressive enhancement (works offline)
- ❌ Limited to loaded features (viewport only)
- ❌ Complex expressions reduce readability

**Example:**
```javascript
// MapLibre filter expression for buildings
map.setFilter('buildings-3d', [
  'all',
  ['>=', ['get', 'construction_year'], min_year],
  ['<=', ['get', 'construction_year'], max_year],
  ['>=', ['get', 'height'], min_height],
  ['<=', ['get', 'height'], max_height],
  ['>=', ['get', 'floor_count'], min_floors]
]);

// Data-driven color based on height
map.setPaintProperty('buildings-3d', 'fill-extrusion-color', [
  'case',
  ['>', ['get', 'height'], 50], '#4a148c',  // Tall: dark purple
  ['>', ['get', 'height'], 20], '#7b1fa2',  // Medium: medium purple
  '#e1bee7'                                   // Short: light purple
]);
```

### Pattern 2: Server-Side Filtering with Query Params

**What:** Backend filters dataset before returning GeoJSON

**When to use:** Large datasets (> 1000 features), complex spatial queries, bandwidth constraints

**Trade-offs:**
- ✅ Reduces bandwidth (only filtered features transferred)
- ✅ Can leverage database indexes
- ❌ Network latency on filter change
- ❌ Server load increases

**Example:**
```rust
// Backend route with filter parameters
#[derive(Deserialize)]
pub struct BuildingFilterParams {
    bbox_wgs84: String,
    min_year: Option<u32>,
    max_year: Option<u32>,
    min_height: Option<f64>,
    max_height: Option<f64>,
    min_floors: Option<u32>,
    max_floors: Option<u32>,
}

pub async fn get_buildings_filtered(
    Query(params): Query<BuildingFilterParams>
) -> Result<Json<GeoJSON>, ApiError> {
    // Query 3DBAG with filters
    let url = format!(
        "https://api.3dbag.nl/collections/pand/items?bbox={}&limit={}&property=>=,construction_year,{}&property=<=,construction_year,{}",
        params.bbox_wgs84, 150, params.min_year.unwrap_or(1900), params.max_year.unwrap_or(2024)
    );
    // ... fetch and return
}
```

### Pattern 3: Hybrid Filtering (Recommended)

**What:** Client-side for viewport-only, server-side for full-dataset export

**When to use:** Interactive exploration + export functionality

**Trade-offs:**
- ✅ Best UX (instant viewport filtering)
- ✅ Export capability (full dataset)
- ❌ Higher complexity (two code paths)

**Implementation:**
```rust
// Frontend: Determine filter strategy
pub fn should_use_server_filter(building_count: usize) -> bool {
    building_count > 1000 || is_export_mode()
}

// Client-side path
fn apply_filters_client_side(filter: &BuildingFilter) {
    let map = get_map_instance();
    map.setFilter('buildings-3d', build_filter_expression(filter));
}

// Server-side path
async fn apply_filters_server_side(filter: &BuildingFilter) -> Result<GeoJSON> {
    fetch_with_params("/api/buildings-3d/filter", filter).await
}
```

## Data Flow

### Request Flow

```
[User adjusts filter slider]
    ↓ (Dioxus reactive signal)
[FilterPanel component]
    ↓ (emits FilterChanged event)
[BuildingFilter state]
    ↓ (validates, serializes)
[Map3D Wrapper]
    ↓ (generates JS filter expression)
[MapLibre GL JS]
    ↓ (applies filter to layer)
[GPU re-render]
    ↓
[Visible buildings update]
```

### State Management

```
[BuildingFilter Signal] ← (global state via use_signal)
    ↓ (subscribe)
[FilterPanel] ←→ [Slider Widgets]
    ↓               ↑
    │               └─ (user input updates signal)
    │
[ViewToggle] ← (2D/3D mode state)
    ↓
[Map3D Wrapper]
    ↓ (signal change triggers)
[apply_layer_updates()]
    ↓
[MapLibre setFilter / setPaintProperty / setLayoutProperty]
```

### Key Data Flows

1. **Filter application:** User input → Validation → Filter expression → Map update
2. **View toggle:** 2D/3D button → Layer visibility switch → Map repaint
3. **Density calculation:** Viewport bounds → Aggregation query → Heatmap layer update
4. **Layer loading:** Viewport change → Debounced fetch → GeoJSON source add → Layer render

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1K users | Client-side filtering sufficient, single backend instance |
| 1K-100K users | Add server-side filter endpoint, caching layer (Redis) |
| 100K+ users | CDN for static tiles, spatial database (PostGIS), load balancing |

### Scaling Priorities

1. **First bottleneck:** MapLibre rendering performance with >1000 buildings
   - Fix: Implement LOD (Level of Detail), simplify geometry at distance, use tile caching

2. **Second bottleneck:** Network bandwidth for large GeoJSON responses
   - Fix: Server-side filtering, gzip compression, vector tiles instead of GeoJSON

## Anti-Patterns

### Anti-Pattern 1: Tight Coupling Between Filter UI and Map

**What people do:** Put filter logic directly in the Map3D component

**Why it's wrong:** Makes filter UI non-reusable, hard to test, violates single responsibility

**Do this instead:** Separate `FilterPanel` component that emits events, `Map3D` component that consumes filter state

**Example:**
```rust
// BAD: Filter logic in map component
#[component]
fn Map3D() -> Element {
    let mut min_year = use_signal(|| 1900);
    // Map component shouldn't own filter state!

    rsx! {
        div {
            input {
                type: "range",
                min: 1900,
                max: 2024,
                oninput: move |e| min_year.set(e.value.parse().unwrap())
            }
        }
    }
}

// GOOD: Separated concerns
#[component]
fn FilterPanel(on_filter_change: Callback<BuildingFilter>) -> Element {
    let mut min_year = use_signal(|| 1900);
    // Filter panel owns filter logic

    rsx! {
        div {
            input {
                type: "range",
                oninput: move |e| {
                    min_year.set(e.value.parse().unwrap());
                    on_filter_change.call(BuildingFilter { min_year: min_year(), .. });
                }
            }
        }
    }
}
```

### Anti-Pattern 2: Synchronous Heavy Filtering on Main Thread

**What people do:** Apply filters that trigger complex calculations on every slider move

**Why it's wrong:** UI freezes, poor UX, especially with large datasets

**Do this instead:** Debounce filter changes, use web workers for heavy computation

**Example:**
```javascript
// BAD: Immediate filter on every input event
slider.addEventListener('input', (e) => {
    applyFilters(e.target.value); // Blocks UI
});

// GOOD: Debounced filter application
const debouncedFilter = debounce(applyFilters, 300);
slider.addEventListener('input', (e) => {
    debouncedFilter(e.target.value); // Waits 300ms after last input
});
```

### Anti-Pattern 3: Hardcoded Filter Values in JavaScript

**What people do:** Build filter expressions with hardcoded values

**Why it's wrong:** Not maintainable, error-prone, can't validate on backend

**Do this instead:** Serialize filter state from Rust, generate expressions dynamically

**Example:**
```javascript
// BAD: Hardcoded filter values
map.setFilter('buildings', ['>=', ['get', 'construction_year'], 1950]);

// GOOD: Dynamic filter from serialized state
const filter = JSON.parse(window.filterState);
map.setFilter('buildings', [
  '>=', ['get', 'construction_year'], filter.min_year
]);
```

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| **3DBAG API** | HTTP proxy with bbox + filter params | CityJSON format, coordinate conversion required (RD↔WGS84) |
| **MapLibre GL JS** | JavaScript interop via window.global | Map instance stored in `window['map_{container_id}']` |
| **MapTiler Terrain** | Optional raster-dem source | Requires API key, fallback to local AHN tiles |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| **FilterPanel ↔ Map3D** | Dioxus signals/events | FilterPanel owns state, Map3D consumes |
| **Frontend ↔ Backend** | HTTP GET with query params | Bbox required, optional filter params |
| **Map3D ↔ MapLibre** | JavaScript interop | Escape container IDs for security |
| **Backend ↔ 3DBAG** | HTTP client with reqwest | Handle failures gracefully (return empty FeatureCollection) |

## Build Order Implications

Based on component dependencies:

1. **Foundation** (must exist first):
   - `BuildingFilter` state struct
   - `Map3D` wrapper functions for filter application
   - `build_filter_expression()` helper

2. **Phase 1 - Basic Filtering**:
   - `FilterPanel` UI component
   - Wire filter state to MapLibre `setFilter()`
   - Add validation (year range, height range)

3. **Phase 2 - View Toggles**:
   - `ViewToggle` component (2D/3D switch)
   - Layer visibility management
   - State persistence (remember last view mode)

4. **Phase 3 - Advanced Features** (parallel):
   - `DensityHeatmap` component
   - Server-side filter endpoint (if needed)
   - Texture mapping (requires custom shaders)

## Sources

- [MapLibre GL JS Style Specification](https://maplibre.org/maplibre-gl-js-docs/style-spec/) - HIGH confidence (official docs)
- [MapLibre GL JS GitHub Repository](https://github.com/maplibre/maplibre-gl-js) - HIGH confidence (official source)
- Existing codebase: `/crates/iou-frontend/src/components/map_3d.rs` - HIGH confidence (actual implementation)
- Existing codebase: `/crates/iou-api/src/routes/buildings_3d.rs` - HIGH confidence (actual implementation)
- Web search attempts for MapLibre filtering patterns - LOW confidence (no results returned, used training data instead)

---
*Architecture research for: 3D Building Data Filtering and Visualization*
*Researched: 2026-03-08*
