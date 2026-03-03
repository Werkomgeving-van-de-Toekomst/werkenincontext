Now I have all the context. Let me find the specific content for section-05-terrain-integration. From the manifest, this section is about:
- "Configure MapTiler terrain source, implement terrain rendering with exaggeration support, and add fallback error handling."

Let me extract the relevant test and implementation content:

**From the index.md manifest:**
- section-05-terrain-integration depends on: section-03-terrain-encoding, section-04-map3d-component
- section-05-terrain-integration blocks: section-06-geojson-support

**Relevant tests from claude-plan-tdd.md for Step 3: Terrain Integration:**
- Manual verification checklist for terrain

**Relevant implementation details from claude-plan.md for Step 3:**
- Section 5.3: Tile Proxy Decision - using MapTiler
- Section 4.2: Terrain Source Configuration
- Section 8.4: Error handling for terrain

Now I'll generate the section content:

---

# Section 5: Terrain Integration

## Overview

This section implements terrain rendering in the Map3D component using MapTiler as the terrain tile source. Terrain provides the 3D elevation effect that distinguishes the 3D map from the 2D Leaflet implementation.

## Dependencies

This section depends on:
- **section-03-terrain-encoding**: Terrain-RGB encoding functions (for future AHN3 integration)
- **section-04-map3d-component**: Core Map3D component with MapLibre initialization

## Context

### Why MapTiler?

For the MVP, we use MapTiler terrain tiles instead of building a custom PDOK AHN3 tile proxy. This decision:

- Eliminates tile proxy development overhead
- Provides free tier (100,000 tile requests/month)
- Offers global coverage including Flevoland
- Allows immediate focus on MapLibre integration

Phase 2 may implement a Rust service for official AHN3 data if MapTiler resolution is insufficient.

### Terrain Source Format

MapLibre requires terrain data as raster-dem tiles with Terrain-RGB encoding. The encoding formula:
```
elevation_meters = -10000 + ((R * 65536 + G * 256 + B) * 0.1)
```

This is implemented in `section-03-terrain-encoding` for future use.

## Tests

### Manual Verification Checklist

Before considering this section complete, verify:

- [ ] Terrain source loads from MapTiler without console errors
- [ ] Terrain is visible when map pitch > 30 degrees
- [ ] Elevation exaggeration affects terrain appearance (test 1.0, 1.5, 2.0)
- [ ] Flevoland region shows recognizable elevation patterns
- [ ] Map remains interactive during terrain tile loading
- [ ] Terrain tiles include proper attribution

### Error Handling Tests

Verify these fallback behaviors:

- [ ] Terrain tile load failure triggers graceful degradation to 2D mode
- [ ] User sees Dutch warning: "3D-terrein niet beschikbaar. 2D-modus actief."
- [ ] Map remains functional without terrain
- [ ] Invalid API key shows user-friendly error message

## Implementation

### 5.1 MapTiler Configuration

**File:** `crates/iou-frontend/src/components/map_3d.rs`

Add terrain configuration to the `Map3DConfig` structure (from section-04):

```rust
impl Map3DConfig {
    pub fn terrain_tile_url(&self) -> String {
        // MapTiler requires an API key
        let api_key = std::env::var("MAPTILER_API_KEY")
            .unwrap_or_else(|_| "YOUR_KEY_HERE".to_string());
        format!(
            "https://api.maptiler.com/tiles/terrain-rgb/tiles.json?key={}",
            api_key
        )
    }

    pub fn terrain_exaggeration(&self) -> f64 {
        std::env::var("TERRAIN_EXAGGERATION")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.5)
    }
}
```

### 5.2 Terrain Source Addition

**File:** `crates/iou-frontend/src/components/map_3d.rs`

Extend the MapLibre initialization JavaScript to include terrain source. Add this after map creation:

```rust
fn terrain_init_js(config: &Map3DConfig) -> String {
    format!(r#"
if (map.getSource('ahn3-terrain')) {{
    console.log('Terrain source already exists, skipping');
    return;
}}

map.addSource('ahn3-terrain', {{
    type: 'raster-dem',
    tiles: ['{tile_url}'],
    tileSize: 256,
    attribution: '&copy; MapTiler &copy; OpenStreetMap contributors'
}});

map.setTerrain({{ source: 'ahn3-terrain', exaggeration: {exaggeration} }});

map.on('terrain.loading', () => {{
    console.log('Terrain tiles loading...');
}});

map.on('terrain', () => {{
    console.log('Terrain loaded');
    sendToRust(JSON.stringify({{ event: 'terrain_loaded' }}));
}});
"#,
        tile_url = config.terrain_tile_url(),
        exaggeration = config.terrain_exaggeration()
    )
}
```

### 5.3 Terrain State Management

**File:** `crates/iou-frontend/src/components/map_3d.rs`

Add terrain state tracking to Map3DState:

```rust
pub struct Map3DState {
    // ... existing fields
    terrain_loaded: bool,
    terrain_enabled: bool,
    terrain_error: Option<String>,
}
```

### 5.4 Error Handling

**File:** `crates/iou-frontend/src/components/map_3d.rs`

Implement graceful fallback when terrain fails:

```rust
fn terrain_fallback_js() -> String {
    r#"
map.on('terrain.error', (error) => {
    console.error('Terrain load error:', error);
    sendToRust(JSON.stringify({
        event: 'terrain_error',
        message: error.error || 'Unknown terrain error'
    }));
});

// Also listen for source data errors
map.on('data', (event) => {
    if (event.sourceId === 'ahn3-terrain' && event.isSourceLoaded) {
        const source = map.getSource('ahn3-terrain');
        if (!source || !source.tiles) {
            console.warn('Terrain source unavailable');
            sendToRust(JSON.stringify({
                event: 'terrain_unavailable'
            }));
        }
    }
});
"#.to_string()
}
```

### 5.5 UI Error Display

**File:** `crates/iou-frontend/src/components/map_3d.rs`

Add error display component:

```rust
#[component]
fn TerrainWarning(message: String) -> Element {
    rsx! {
        div {
            class: "terrain-warning",
            style: "position: absolute; top: 10px; left: 50%; transform: translateX(-50%); 
                    background: rgba(255, 193, 7, 0.95); padding: 8px 16px; 
                    border-radius: 4px; z-index: 1000; font-size: 14px;",
            icon {
                class: "warning-icon",
                style: "margin-right: 8px;",
                lucide::AlertTriangle
            }
            {message}
        }
    }
}
```

Dutch error messages:
- `"3D-terrein niet beschikbaar. 2D-modus actief."` - Terrain unavailable
- `"Kaart kon niet worden geladen. Vernieuw de pagina of probeer het later opnieuw."` - General load failure

### 5.6 Exaggeration Control (Optional Enhancement)

For interactive terrain exaggeration adjustment:

```rust
fn set_terrain_exaggeration_js(value: f64) -> String {
    format!(r#"
if (map.getTerrain()) {{
    map.setTerrain({{ source: 'ahn3-terrain', exaggeration: {value} }});
    console.log('Terrain exaggeration set to {value}');
}}
"#)
}
```

## Environment Variables

Add to deployment configuration:

```bash
# MapTiler API key (get free key from maptiler.com)
MAPTILER_API_KEY=your_key_here

# Terrain exaggeration multiplier (0.1 to 5.0, default 1.5)
TERRAIN_EXAGGERATION=1.5
```

## Success Criteria

This section is complete when:

1. Terrain loads from MapTiler without console errors
2. Elevation is visible when map pitch > 30 degrees
3. Flevoland shows recognizable terrain features
4. Terrain load failures trigger user-friendly Dutch message
5. Map remains functional when terrain is unavailable
6. Exaggeration setting affects terrain appearance
7. Proper attribution appears on map

## Next Steps

After completing this section:
- **section-06-geojson-support**: Add GeoJSON layer support to display data over terrain