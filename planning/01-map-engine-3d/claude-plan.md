# Implementation Plan - Map Engine 3D Upgrade

## 1. Overview

This plan describes upgrading the Flevoland Data Verkenner from a 2D Leaflet-based map to a 3D-capable MapLibre GL JS implementation with AHN3 terrain data integration.

### 1.1 What We're Building

A WebGL2-based 3D map component that:
- Renders Flevoland terrain using PDOK AHN3 elevation data (5m resolution)
- Supports 3D navigation (pitch, rotate, zoom)
- Displays existing GeoJSON layers
- Integrates with the existing Dioxus WASM frontend

### 1.2 Why This Approach

**MapLibre GL JS was chosen because:**
- Open source (Apache 2.0) with no API key requirements
- Native 3D terrain support via WebGL2
- Compatible with Mapbox GL JS API (well-documented)
- Active community and maintenance

**Tile proxy architecture is necessary because:**
- PDOK AHN3 provides WMS, not XYZ tiles
- MapLibre requires Terrain-RGB encoded XYZ tiles
- Conversion layer enables using official Dutch elevation data

### 1.3 Scope Phasing

**Phase 1 (MVP - this plan):**
- Core 3D map with terrain
- 2 GeoJSON layers migrated
- Chrome/Firefox support

**Phase 2 (Future, out of scope):**
- Remaining GeoJSON layers
- Safari support
- Advanced pop-ups and styling

---

## 2. Architecture

### 2.1 System Context

The system consists of:
- Dioxus frontend (Rust/WASM)
- JavaScript bridge via document::eval()
- MapLibre GL JS running in browser
- Optional tile proxy for terrain data

### 2.2 Frontend Structure

```
crates/iou-frontend/src/
├── pages/
│   └── data_verkenner.rs          # Modified to use Map3D
├── components/
│   ├── map_3d.rs                  # NEW: MapLibre wrapper component
│   ├── layer_control_3d.rs        # NEW: Layer toggle control
│   └── mod.rs                     # Updated exports
└── lib.rs
```

### 2.3 Tile Proxy Structure (Optional)

If implementing as a separate service:

```
crates/terrain-proxy/              # NEW (optional crate)
├── src/
│   ├── main.rs                    # Server entry point
│   ├── pdok.rs                    # PDOK WMS client
│   ├── encoder.rs                 # Elevation to Terrain-RGB conversion
│   └── cache.rs                   # Tile caching
└── Cargo.toml
```

---

## 3. Components

### 3.1 Map3D Component

**File:** `crates/iou-frontend/src/components/map_3d.rs`

**Purpose:** Dioxus component that wraps MapLibre GL JS

**Responsibilities:**
- Initialize MapLibre GL JS instance via JavaScript bridge
- Configure terrain source
- Handle map events (zoom, pan, rotate)
- Manage GeoJSON layer sources
- Provide methods for layer manipulation

**State structure:**
```rust
pub struct Map3DState {
    map_id: String,
    center: (f64, f64),    // (longitude, latitude)
    zoom: f64,
    pitch: f64,            // 0-60 degrees
    bearing: f64,          // 0-360 degrees
    terrain_enabled: bool,
}
```

### 3.2 LayerControl3D Component

**File:** `crates/iou-frontend/src/components/layer_control_3d.rs`

**Purpose:** UI for toggling map layers on/off

**Layer definition:**
```rust
pub struct GeoJsonLayer {
    pub id: String,
    pub name: String,
    pub url: String,
    pub layer_type: LayerType,     // Point, Line, Polygon
    pub visible: bool,
    pub color: String,             // CSS color
}

pub enum LayerType {
    Point,
    Line,
    Polygon,
}
```

### 3.3 Data Verkenner Page Integration

**File:** `crates/iou-frontend/src/pages/data_verkenner.rs`

**Changes:**
- Replace Leaflet initialization with Map3D component
- Update layer list to use new layer definitions
- Preserve existing page layout and UI

---

## 4. JavaScript Bridge

### 4.1 MapLibre Initialization

The JavaScript executed via document::eval() will:

1. Load MapLibre GL JS library from CDN
2. Create map instance with configuration
3. Add terrain source
4. Enable terrain rendering
5. Attach event listeners

**Initial configuration:**
- Container: 'map'
- Center: [5.5, 52.4] (lon, lat for Flevoland)
- Zoom: 10
- Pitch: 60 degrees
- Bearing: 0

### 4.2 Terrain Source Configuration

```javascript
map.addSource('ahn3-terrain', {
    type: 'raster-dem',
    tiles: [terrain_tile_url],
    tileSize: 256,
    attribution: 'PDOK AHN3'
});

map.setTerrain({ source: 'ahn3-terrain', exaggeration: 1.5 });
```

### 4.3 GeoJSON Layer Management

Functions to expose:
- addGeoJsonLayer(id, url, type, color)
- toggleLayer(id, visible)
- setTerrain(enabled, exaggeration)

### 4.4 Event Handling

**Event Bridge Mechanism:**

JavaScript events return values to Rust via `document::eval()` return value. The JavaScript code returns a JSON string that Rust parses.

For discrete events (load, layer toggle):
```javascript
map.on('load', () => {
    sendToRust(JSON.stringify({event: 'load', zoom: map.getZoom()}));
});
```

For continuous events (drag, zoom):
- Keep state JavaScript-side only
- Avoid flooding the Rust-JavaScript bridge
- Sync to Rust on specific trigger (e.g., button click)

**Events to expose to Rust:**
- load: Map fully loaded (one-time, returns config state)
- layer_toggle: Layer visibility changed
- error: Map or terrain loading failed

**Events kept JavaScript-side:**
- zoom: Continuous during zoom gesture
- pitch: Continuous during tilt
- rotate: Continuous during rotation
- move: Continuous during pan

---

## 5. Terrain Data Strategy

### 5.1 The Challenge

PDOK AHN3 provides WMS 1.3.0 service. MapLibre requires XYZ tiles in Terrain-RGB format.

### 5.2 Terrain-RGB Encoding

Elevation encoding formula:
```
elevation_meters = -10000 + ((R * 65536 + G * 256 + B) * 0.1)
```

For encoding:
```rust
fn elevation_to_terrain_rgb(elevation_meters: f64) -> (u8, u8, u8) {
    let normalized = (elevation_meters + 10000.0) * 10.0;
    let encoded = normalized as u32;
    let r = ((encoded >> 16) & 0xFF) as u8;
    let g = ((encoded >> 8) & 0xFF) as u8;
    let b = (encoded & 0xFF) as u8;
    (r, g, b)
}
```

### 5.3 Tile Proxy Decision

**For MVP, we will use MapTiler terrain tiles.**

**Rationale:**
- Zero development overhead - no proxy service to build/maintain
- Free tier provides sufficient tiles for development/testing
- Global coverage includes Flevoland with acceptable resolution
- Enables immediate focus on MapLibre integration

**Decision Matrix:**

| Option | Dev Time | Maintenance | Data Accuracy | Performance |
|--------|----------|-------------|---------------|-------------|
| MapTiler | 0 days | Low | Good | Excellent |
| Serverless | 2-3 days | Medium | Best (AHN3) | Good |
| Rust Service | 3-5 days | High | Best (AHN3) | Excellent |
| Pre-generated | 1-2 days | Low | Best (AHN3) | Excellent |

**Phase 2 Consideration:**
If MapTiler resolution is insufficient, implement the Rust Service option for AHN3 data.

**MapTiler Configuration:**
- URL: `https://api.maptiler.com/tiles/terrain-rgb/tiles.json?key=<YOUR_KEY>`
- Free tier: 100,000 tile requests/month
- Attribution required on map

---

## 6. GeoJSON Layer Migration

### 6.1 Existing Layers

Migrate in priority order:

| Layer | Type | Phase |
|-------|------|-------|
| provinciegrens.geojson | Polygon | MVP |
| cultuurhistorie.geojson | Point/Polygon | MVP |
| windturbines.geojson | Point | Phase 2 |
| zonneparken.geojson | Point | Phase 2 |
| fietsnetwerken.geojson | Line | Phase 2 |
| drinkwater.geojson | Point/Line | Phase 2 |

### 6.2 MapLibre Layer Types

- Circle: For point features
- Line: For line features
- Fill: For polygon features

---

## 7. Configuration

### 7.1 Map Configuration Structure

```rust
pub struct Map3DConfig {
    pub container_id: String,
    pub center: (f64, f64),     // (lon, lat) - NOTE: MapLibre uses [longitude, latitude]
    pub zoom: f64,
    pub pitch: f64,             // Default: 60
    pub bearing: f64,           // Default: 0
    pub min_zoom: f64,          // Default: 6
    pub max_zoom: f64,          // Default: 18
    pub terrain_exaggeration: f64, // Default: 1.5
    pub terrain_tile_url: String,
}
```

**IMPORTANT:** MapLibre uses `[longitude, latitude]` ordering while Leaflet uses `[latitude, longitude]`. This is a common source of bugs. Leaflet code: `L.map('map').setView([52.45, 5.50], 10)` (lat, lon). MapLibre equivalent: `center: [5.50, 52.45]` (lon, lat).

### 7.2 Environment Variables

Optional configuration:
- MAPLIBRE_CSS_URL: Default to unpkg CDN
- MAPLIBRE_JS_URL: Default to unpkg CDN
- TERRAIN_TILE_URL: Tile proxy URL

---

## 8. Testing Strategy

### 8.1 Unit Tests

- Terrain-RGB encoding/decoding
- Layer configuration object generation
- Layer type detection from GeoJSON

### 8.2 Integration Tests

- MapLibre instance creation
- Terrain source addition
- GeoJSON layer loading

### 8.3 Browser Tests

Manual testing:
- Chrome: Map loads, terrain visible, navigation works
- Firefox: Map loads, terrain visible, navigation works
- Performance: Smooth panning/zooming at 60fps

### 8.4 Error Handling

**Load Failures:**
- MapLibre CDN unavailable: Show error message with retry button
- WebGL2 not supported: Display message "Uw browser ondersteunt geen 3D-kaarten"
- Terrain tiles fail to load: Gracefully fall back to 2D mode with warning

**GeoJSON Errors:**
- File not found (404): Show layer as disabled in control panel
- Malformed GeoJSON: Log error, skip that layer, show toast notification
- Mixed geometry types: Handle by creating multiple layer types

**User-Facing Error Messages (Dutch):**
- "Kaart kon niet worden geladen. Vernieuw de pagina of probeer het later opnieuw."
- "3D-terrein niet beschikbaar. 2D-modus actief."
- "Laag {naam} kon niet worden geladen."

**Retry Logic:**
- Tile loading: Built-in MapLibre retry (3 attempts)
- CDN scripts: Browser cache with fallback CDN option

---

## 9. Implementation Steps

### Step 1: Project Setup
1. Create map_3d.rs component file
2. Create layer_control_3d.rs component file
3. Update components/mod.rs exports
4. Add CSS for MapLibre to index.html

### Step 2: MapLibre Integration
1. Implement Map3D component structure
2. Create JavaScript initialization string
3. Load MapLibre GL JS via CDN
4. Initialize map with base configuration
5. Add navigation controls

### Step 3: Terrain Integration
1. Implement tile proxy or configure MapTiler fallback
2. Add terrain source to map
3. Enable terrain rendering
4. Verify terrain displays for Flevoland

### Step 4: GeoJSON Support
1. Define layer structure in Rust
2. Implement JavaScript layer functions
3. Migrate 2 priority layers
4. Test layer display

### Step 5: Layer Control UI
1. Implement LayerControl3D component
2. Add checkboxes for each layer
3. Connect toggle to map visibility
4. Style to match existing UI

### Step 6: Integration
1. Add MAP_3D_ENABLED feature flag
2. Modify data_verkenner.rs to conditionally render Map3D or Leaflet
3. Add Map3D and LayerControl3D components alongside Leaflet
4. Test both rendering modes
5. Default to Leaflet (MAP_3D_ENABLED=false)

Note: Leaflet code removal happens after successful production deployment, not in this step.

### Step 7: Testing
1. Unit tests for encoding/logic
2. Integration tests for map functions
3. Browser compatibility tests
4. Performance testing

---

## 10. Dependencies

### External Dependencies

**MapLibre GL JS (version pinned to 5.0.0):**
- JavaScript: https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.js
- CSS: https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.css

**Dioxus.toml configuration:**
```toml
[web.resource]
style = [
    "https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.css",
]
script = [
    "https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.js",
]
```

**MapTiler Terrain:**
- Tiles: https://api.maptiler.com/tiles/terrain-rgb/tiles.json
- Requires free API key from maptiler.com

**PDOK AHN3 (Phase 2):**
- WMS: https://service.pdok.nl/cds/wms/ahn3_5m/wms/v1_0

### Rust Dependencies

No new Rust dependencies expected. Dioxus framework already includes dioxus-web with document::eval() capability.

---

## 11. Risks and Mitigations

### Risk 1: Tile Proxy Complexity

**Concern:** WMS to Terrain-RGB conversion may be complex.

**Mitigation:** MapTiler available as fallback for MVP.

### Risk 2: Performance Issues

**Concern:** 3D rendering may not meet 60fps target.

**Mitigation:** Progressive tile loading, terrain LOD (built-in), configurable exaggeration.

### Risk 3: Browser Incompatibilities

**Concern:** WebGL2 implementation differences.

**Mitigation:** Focus on Chromium + Firefox first, Safari in Phase 2.

### 11.1 Rollback Strategy

**Feature Flag Approach:**

Add environment variable to control map engine selection:
```rust
let use_3d_map = std::env::var("MAP_3D_ENABLED")
    .unwrap_or("false".to_string())
    .parse::<bool>()
    .unwrap_or(false);
```

**Implementation Phases:**
1. Keep Leaflet code initially, conditionally render Map3D or Leaflet
2. Deploy with `MAP_3D_ENABLED=false` (default)
3. Test with `MAP_3D_ENABLED=true` in staging
4. Gradually enable for beta users
5. Only remove Leaflet code after successful production deployment

**Graceful Degradation:**
If MapLibre fails to initialize, automatically fall back to Leaflet with user notification.

---

## 12. Success Criteria

The implementation is complete when:

1. MapLibre GL JS initializes without errors
2. Flevoland region is centered and visible
3. Terrain elevation is clearly visible
4. User can tilt, rotate, and zoom smoothly
5. At least 2 GeoJSON layers load and display correctly
6. Layer toggle works (on/off)
7. Works in Chrome and Firefox
8. Unit and integration tests pass
