# Research Findings - Map Engine 3D Upgrade

## Codebase Analysis

### Existing Architecture

**Current Implementation:** `crates/iou-frontend/src/pages/data_verkenner.rs`

The Flevoland Data Verkenner currently uses:
- **Leaflet.js** for 2D mapping
- **Dioxus** WASM framework for the frontend
- **JavaScript bridge pattern** via `document::eval()` for map initialization

**Key Pattern:**
```rust
use_effect(move || {
    let script = r#"
        (function() {
            var el = document.getElementById('map');
            if (!el || el._leaflet_id) return;
            el.innerHTML = '';
            var map = L.map('map').setView([52.45, 5.50], 10);
            // Leaflet initialization continues...
        })();
    "#;
    document::eval(script);
});
```

This pattern should be preserved for MapLibre GL JS integration.

### Frontend Structure

```
crates/iou-frontend/
├── src/
│   ├── pages/
│   │   └── data_verkenner.rs    # Main map page (to be modified)
│   ├── components/              # New components will go here
│   └── lib.rs                   # Module exports
└── Cargo.toml                   # Rust dependencies
```

### Existing GeoJSON Layers

The following GeoJSON layers are currently loaded and need to be preserved:
- `provinciegrens.geojson` - Province boundaries
- `cultuurhistorie.geojson` - Cultural heritage sites
- `windturbines.geojson` - Wind turbine locations
- `zonneparken.geojson` - Solar park locations
- `fietsnetwerken.geojson` - Cycling networks
- `drinkwater.geojson` - Drinking water infrastructure

## Web Research: MapLibre GL JS

### Library Overview

**MapLibre GL JS v5.x** is an open-source fork of Mapbox GL JS:
- Apache 2.0 licensed (no API key required)
- WebGL2-based rendering for hardware acceleration
- Native 3D terrain support
- Compatible with Mapbox GL JS API

### 3D Terrain API

**Terrain Source Configuration:**
```javascript
map.addSource('terrain', {
    type: 'raster-dem',
    tiles: [
        'https://example.com/terrain/{z}/{x}/{y}.png'
    ],
    tileSize: 256,
    attribution: 'Terrain data'
});

map.setTerrain({ source: 'terrain', exaggeration: 1.5 });
```

**Terrain-RGB Encoding:**
Elevation is encoded in RGB channels:
```
elevation = -10000 + ((R * 256 * 256 + G * 256 + B) * 0.1)
```

### Map Initialization Pattern

```javascript
const map = new maplibregl.Map({
    container: 'map',
    style: 'https://demotiles.maplibre.org/style.json', // or custom style
    center: [5.5, 52.4],  // [lon, lat] for Flevoland
    zoom: 10,
    pitch: 60,            // 3D tilt
    bearing: 0
});
```

### GeoJSON Layer Pattern

```javascript
map.addSource('geojson-layer', {
    type: 'geojson',
    data: '/path/to/layer.geojson'
});

map.addLayer({
    id: 'layer-fill',
    type: 'fill',
    source: 'geojson-layer',
    paint: {
        'fill-color': '#088',
        'fill-opacity': 0.5
    }
});
```

### Navigation Controls

MapLibre provides built-in controls:
```javascript
map.addControl(new maplibregl.NavigationControl());
map.addControl(new maplibregl.FullscreenControl());
```

## Web Research: PDOK AHN3

### PDOK AHN3 Service

**AHN3** (Actueel Hoogtebestand Nederland, 3rd version):
- **Resolution:** 5m for entire Netherlands
- **WMS Endpoint:** `https://service.pdok.nl/cds/wms/ahn3_5m/wms/v1_0`
- **Format:** WMS 1.3.0 (not XYZ tiles)

### The Tile Proxy Problem

PDOK provides WMS, but MapLibre requires:
- **XYZ tiles** (/{z}/{x}/{y} URLs) OR
- **Terrain-RGB encoded raster DEM tiles**

**Solution:** Implement a tile proxy that:
1. Accepts XYZ tile requests from MapLibre
2. Fetches WMS tiles from PDOK
3. Converts to Terrain-RGB encoding
4. Serves with proper CORS headers

### Terrain-RGB Conversion

For elevation in meters (range: -10m to +50m for Flevoland):

```python
def meters_to_rgb(elevation):
    """Convert elevation in meters to Terrain-RGB color."""
    elevation += 10000  # Offset
    encoded = int(elevation * 10)
    r = (encoded >> 16) & 0xFF
    g = (encoded >> 8) & 0xFF
    b = encoded & 0xFF
    return (r, g, b)
```

### Alternative: MapTiler

MapTiler provides Terrain-RGB tiles globally:
- Free tier available
- May not have PDOK's 5m resolution
- Commercial option if proxy implementation is too complex

## Browser Compatibility

**WebGL2 Requirements:**
- Chrome/Chromium: Full support (since 2017)
- Firefox: Full support (since 2017)
- Safari: Supported but may have different performance characteristics
- Edge (Chromium-based): Full support

**Detection:**
```javascript
const canvas = document.createElement('canvas');
const hasWebGL2 = !!canvas.getContext('webgl2');
```

## Performance Considerations

### Terrain Tile Caching

- Cache tiles locally to reduce PDOK requests
- Implement tile pre-fetching for viewport
- Consider Service Worker for offline caching

### Level of Detail (LOD)

MapLibre automatically manages LOD based on:
- Distance from camera
- Current zoom level
- Available tile resolutions

### Target Performance

- **60fps rendering** on modern hardware
- Progressive tile loading
- Smooth 3D navigation

## Key Takeaways

1. **Use existing Dioxus patterns** - The `document::eval()` pattern works for any JS library
2. **Tile proxy is essential** - PDOK WMS must be converted to XYZ Terrain-RGB
3. **MVP scope** - Start with terrain + 2 GeoJSON layers
4. **Chromium+Firefox first** - Safari support can be added later
5. **MapLibre GL JS v5.x** - Open source, no API key needed
