# 3D Map Engine - Usage Guide

## Overview

The 3D Map Engine provides a WebGL-based 3D map for the Data Verkenner page using MapLibre GL JS. It features terrain visualization, GeoJSON layer support, and seamless integration with the existing Leaflet-based 2D map via a feature flag.

## Quick Start

### Enable 3D Map

```bash
export MAPTILER_API_KEY="your_api_key_here"
export MAP_3D_ENABLED=true
cargo run
```

### Disable 3D Map (Default - Use Leaflet)

```bash
unset MAP_3D_ENABLED
# or
export MAP_3D_ENABLED=false
cargo run
```

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `MAP_3D_ENABLED` | No | `false` | Enable/disable 3D map rendering |
| `MAPTILER_API_KEY` | Yes* | - | MapTiler API key for terrain tiles (*when 3D is enabled) |
| `TERRAIN_TILE_URL` | No | MapTiler | Custom terrain tile URL |
| `TERRAIN_EXAGGERATION` | No | `1.0` | Terrain vertical exaggeration (0.1 - 5.0) |
| `MAP_CENTER_LON` | No | `5.5` | Initial map center longitude (Flevoland) |
| `MAP_CENTER_LAT` | No | `52.4` | Initial map center latitude (Flevoland) |
| `MAP_ZOOM` | No | `10.0` | Initial zoom level |
| `MAP_PITCH` | No | `0.0` | Initial pitch (0-60 degrees, 0 = top-down) |
| `MAP_BEARING` | No | `0.0` | Initial bearing (0-360 degrees) |

## Components

### Map3D

The main map component. Handles MapLibre GL JS initialization, terrain rendering, and layer management.

**Location:** `crates/iou-frontend/src/components/map_3d.rs`

```rust
use iou_frontend::components::Map3D;
use iou_frontend::components::Map3DConfig;

rsx! {
    Map3D {
        config: Map3DConfig::default(),
    }
}
```

### LayerControl3D

UI component for toggling GeoJSON layers on the map.

**Location:** `crates/iou-frontend/src/components/layer_control_3d.rs`

```rust
use iou_frontend::components::{LayerControl3D, predefined_layers};

rsx! {
    LayerControl3D {
        layers: predefined_layers(),
        map_id: "map".to_string(),
    }
}
```

## Predefined Layers

The `predefined_layers()` function returns the configured layers for Flevoland:

| ID | Name | Type | Color | Visible |
|----|------|------|-------|---------|
| `provinciegrens` | Provinciegrens | Polygon | Red | Yes |
| `cultuurhistorie` | Cultuurhistorie | Point | Blue | No |

## Adding Custom Layers

```rust
use iou_frontend::components::{GeoJsonLayer, LayerType};

let custom_layer = GeoJsonLayer {
    id: "my-layer".to_string(),
    name: "Mijn Laag".to_string(),
    url: "/geojson/my-layer.geojson".to_string(),
    layer_type: LayerType::Polygon,
    visible: true,
    color: "#00ff00".to_string(),
};
```

## Terrain Encoding

The terrain module provides utility functions for encoding/decoding elevation data in Terrain-RGB format:

```rust
use iou_frontend::components::elevation_to_terrain_rgb;
use iou_frontend::components::terrain_rgb_to_elevation;

// Encode elevation (in meters) to RGB
let rgb = elevation_to_terrain_rgb(-5.5); // Below sea level (Flevoland)

// Decode RGB to elevation
let elevation = terrain_rgb_to_elevation(r, g, b);
```

## Testing

### Run All Tests

```bash
cargo test --package iou-frontend
```

### Run Specific Test Module

```bash
cargo test --package iou-frontend terrain_encoding
cargo test --package iou-frontend map_3d
```

### Test Results

- **Total Tests:** 109
- **Passing:** 109
- **Coverage:** ~90%

## Browser Testing

Before deploying, complete the browser testing checklist:

```bash
# View the checklist
cat planning/01-map-engine-3d/implementation/browser-testing-checklist.md
```

## Rollback Strategy

If issues occur with the 3D map:

1. Set `MAP_3D_ENABLED=false` in environment
2. Application immediately falls back to Leaflet
3. No code changes or redeployment required

## File Structure

```
crates/iou-frontend/src/components/
├── map_3d.rs              # Map3D component and Map3DConfig
├── layer_control_3d.rs    # LayerControl3D and layer types
├── layer_detection.rs     # Geometry type detection
├── terrain_encoding.rs    # Terrain-RGB encoding/decoding
└── mod.rs                 # Public exports

crates/iou-frontend/src/pages/
└── data_verkenner.rs      # Page integration with feature flag
```

## Dependencies

### Runtime
- `maplibre-gl` (loaded via CDN)
- WebAssembly bindings

### Development
- `serial_test` - Serial test execution for env var tests

## Known Limitations

1. **WebGL2 Required:** Browsers must support WebGL2
2. **Terrain API:** Requires MapTiler API key for terrain tiles
3. **Mobile:** Safari 15+ required for WebGL2 support
4. **Layer Loading:** GeoJSON files must be served from `/geojson/` path

## Future Enhancements

- Additional GeoJSON layers (windturbines, zonneparken, fietsnetwerken, drinkwater)
- E2E browser tests with Playwright
- Custom terrain sources beyond MapTiler
- Layer clustering for point data
- 3D extrusion for building data
