# Specification - Map Engine 3D Upgrade

## Overview

Upgrade the Flevoland Data Verkenner from Leaflet (2D) to MapLibre GL JS (3D) with AHN3 terrain integration. This is the foundation component for advanced map features including historical comparison and PDOK service integration.

## Context

The Flevoland Data Verkenner is a geographic data explorer built with:
- **Frontend:** Dioxus (Rust/WASM)
- **Current Map Engine:** Leaflet.js (2D only)
- **Target Region:** Flevoland province, Netherlands

## User Requirements

### Primary Goal

Enable 3D terrain visualization for Flevoland with AHN3 (Actueel Hoogtebestand) elevation data. This provides realistic terrain representation essential for:
- Understanding elevation changes in the polder landscape
- Visualizing infrastructure in geographic context
- Foundation for future historical comparison features

### Functional Requirements

1. **3D Map Rendering**
   - Replace Leaflet with MapLibre GL JS
   - Display AHN3 terrain elevation data
   - Enable 3D navigation (pitch, rotate, zoom)
   - Maintain smooth 60fps performance

2. **GeoJSON Layer Support**
   - Preserve existing GeoJSON layers (6 layers currently)
   - MVP: At least 2 layers working (can migrate others later)
   - Support point, line, and polygon geometries

3. **Layer Control**
   - Toggle layers on/off
   - Maintain layer control UI

4. **Browser Support**
   - Primary: Chrome, Chromium-based browsers (Edge, Brave)
   - Secondary: Firefox
   - Safari: Deferred to Phase 2

## Technical Decisions (from Interview)

### Q1: Terrain Data Strategy

**Question:** PDOK AHN3 doesn't provide XYZ tiles in Terrain-RGB format directly. Which approach should we use?

**Decision:** PDOK tile proxy

- Implement a tile proxy service that converts PDOK AHN3 WMS to Terrain-RGB XYZ tiles
- Proxy handles:
  - WMS → XYZ tile conversion
  - RGB encoding for elevation data
  - Tile caching
  - CORS headers

**Implications:**
- Extra backend service needed (or serverless function)
- Tile caching strategy important for performance

### Q2: Feature Migration Scope

**Question:** How many existing Leaflet features must be preserved?

**Decision:** 3D terrain first, Core only

**MVP Definition:**
1. MapLibre GL JS initialized with 3D terrain
2. Flevoland region visible with terrain elevation
3. 3D navigation (pitch, rotate, zoom) functional
4. At least 2 existing GeoJSON layers working

**Can defer to Phase 2:**
- Advanced pop-ups
- Custom point styling
- Layer reordering

### Q3: Browser Compatibility

**Decision:** Chromium + Firefox

- Primary support: Chrome, Chromium-based browsers
- Secondary: Firefox
- Safari: Phase 2

**Rationale:** Cover majority of users, reduce initial complexity

## Technical Requirements

### Map Library

- **MapLibre GL JS v5.x**
- Rationale: Open source (Apache 2.0), no API key required, compatible with Mapbox GL JS API

### Terrain Data Source

- **PDOK AHN3** (5m resolution)
- **WMS Endpoint:** `https://service.pdok.nl/cds/wms/ahn3_5m/wms/v1_0`
- **Tile Proxy Required:** WMS → XYZ Terrain-RGB conversion

### Initial View Configuration

- **Center:** [5.5, 52.4] (longitude, latitude for Flevoland)
- **Zoom:** 10
- **Pitch:** 60 degrees (3D perspective)
- **Bearing:** 0 (north-up)

### Terrain-RGB Encoding

Elevation encoding formula:
```
elevation = -10000 + ((R * 256 * 256 + G * 256 + B) * 0.1)
```

### Dioxus Integration Pattern

Use existing `document::eval()` pattern from Leaflet implementation:

```rust
use_effect(move || {
    let script = r#"
        (function() {
            // MapLibre GL JS initialization
        })();
    "#;
    document::eval(script);
});
```

## Architecture

### Frontend Components

```
crates/iou-frontend/src/
├── pages/
│   └── data_verkenner.rs      # Modify to use MapLibre
├── components/
│   ├── map_3d.rs              # New: MapLibre wrapper
│   └── layer_control_3d.rs    # New: Layer control for MapLibre
└── lib.rs
```

### Tile Proxy (Backend)

```
crates/iou-frontend/  (or separate service)
└── terrain_proxy/
    ├── main.rs              # Serverless function or small service
    ├── pdok_client.rs       # WMS fetch logic
    └── terrain_encoder.rs   # Elevation → RGB conversion
```

**Note:** Tile proxy could be:
- Separate Rust service
- Serverless function (Cloudflare Workers, AWS Lambda)
- Node.js service
- Decision to be made during implementation

## Existing GeoJSON Layers

All layers should eventually work. MVP requires 2:

| Layer | Type | Priority |
|-------|------|----------|
| provinciegrens.geojson | Polygon | High |
| cultuurhistorie.geojson | Point/Polygon | High |
| windturbines.geojson | Point | Medium |
| zonneparken.geojson | Point | Medium |
| fietsnetwerken.geojson | Line | Low |
| drinkwater.geojson | Point/Line | Low |

## Performance Requirements

- **Frame rate:** Smooth 60fps on modern hardware
- **Tile loading:** Progressive loading with loading indicators
- **Caching:** Tile caching to reduce PDOK requests
- **LOD:** Automatic Level of Detail management (MapLibre built-in)

## Constraints

### Dependencies

This component (`01-map-engine-3d`) is a foundation for:
- `02-historical-comparator` - needs 3D terrain
- `03-pdok-integration` - needs new map engine

### Phasing

**Phase 1 (MVP):**
- MapLibre GL JS integration
- 3D terrain with AHN3 (via proxy)
- 3D navigation controls
- 2 GeoJSON layers migrated
- Chrome/Firefox testing

**Phase 2 (Future):**
- Full GeoJSON layer migration
- Advanced pop-ups
- Safari support
- Performance optimization

## Acceptance Criteria

- [ ] 3D map loads with Flevoland terrain visible
- [ ] User can tilt, rotate, zoom in 3D mode
- [ ] At least 2 existing GeoJSON layers work
- [ ] Performance acceptable on modern hardware
- [ ] Map works in Chrome and Firefox

## Definition of Done

- Code review completed
- Unit tests for JS bridge functions
- Integration test with real GeoJSON data
- Browser compatibility test (Chrome, Firefox)
- Documentation of API
