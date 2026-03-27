Now I have all the context needed to generate the section content for `section-04-map3d-component`. Let me extract the relevant content and create a self-contained section.

# Section 04: Map3D Component

## Overview

This section implements the core `Map3D` Dioxus component that wraps MapLibre GL JS. The component is responsible for initializing the 3D map, managing its state, and providing a JavaScript bridge for communication between Rust and the browser.

## Dependencies

This section depends on:
- **section-01-project-setup**: The `map_3d.rs` file must be created and component exports must be added to `mod.rs`
- **section-02-config-structures**: The `Map3DConfig` struct must be defined with validation logic

## Tests

Write the following tests before implementation:

### Configuration Tests

**Test: Map3DConfig creates with valid default values**
```rust
// File: crates/iou-frontend/src/components/map_config_test.rs
#[test]
fn test_map3d_config_default_values() {
    let config = Map3DConfig::default();
    assert_eq!(config.container_id, "map");
    assert_eq!(config.center, (5.5, 52.4)); // (lon, lat) for Flevoland
    assert_eq!(config.zoom, 10.0);
    assert_eq!(config.pitch, 60.0);
    assert_eq!(config.bearing, 0.0);
    assert_eq!(config.min_zoom, 6.0);
    assert_eq!(config.max_zoom, 18.0);
    assert_eq!(config.terrain_exaggeration, 1.5);
}
```

**Test: Map3DConfig validates longitude (-180 to 180) and latitude (-90 to 90)**
```rust
#[test]
fn test_map3d_config_validates_coordinates() {
    // Valid coordinates
    let config = Map3DConfig::new("map".to_string(), (5.5, 52.4), 10.0);
    assert!(config.is_ok());
    
    // Invalid longitude (out of range)
    let config = Map3DConfig::new("map".to_string(), (200.0, 52.4), 10.0);
    assert!(config.is_err());
    
    // Invalid latitude (out of range)
    let config = Map3DConfig::new("map".to_string(), (5.5, 100.0), 10.0);
    assert!(config.is_err());
}
```

**Test: Map3DConfig validates pitch (0 to 60) and bearing (0 to 360)**
```rust
#[test]
fn test_map3d_config_validates_view_angles() {
    // Valid pitch and bearing
    let config = Map3DConfig::with_view("map".to_string(), 10.0, 45.0, 180.0);
    assert!(config.is_ok());
    
    // Invalid pitch (negative)
    let config = Map3DConfig::with_view("map".to_string(), 10.0, -10.0, 0.0);
    assert!(config.is_err());
    
    // Invalid pitch (> 60)
    let config = Map3DConfig::with_view("map".to_string(), 10.0, 70.0, 0.0);
    assert!(config.is_err());
    
    // Invalid bearing (negative)
    let config = Map3DConfig::with_view("map".to_string(), 10.0, 0.0, -10.0);
    assert!(config.is_err());
}
```

**Test: Map3DConfig validates zoom range (6 to 18)**
```rust
#[test]
fn test_map3d_config_validates_zoom() {
    // Valid zoom levels
    let config = Map3DConfig::new("map".to_string(), (5.5, 52.4), 10.0);
    assert!(config.is_ok());
    
    let config = Map3DConfig::new("map".to_string(), (5.5, 52.4), 6.0);
    assert!(config.is_ok());
    
    let config = Map3DConfig::new("map".to_string(), (5.5, 52.4), 18.0);
    assert!(config.is_ok());
    
    // Invalid zoom (too low)
    let config = Map3DConfig::new("map".to_string(), (5.5, 52.4), 5.0);
    assert!(config.is_err());
    
    // Invalid zoom (too high)
    let config = Map3DConfig::new("map".to_string(), (5.5, 52.4), 19.0);
    assert!(config.is_err());
}
```

### Component Rendering Tests

**Test: Map3D component renders container div with correct id**
```rust
// File: crates/iou-frontend/src/components/map_3d_test.rs
#[test]
fn test_map3d_renders_container_div() {
    // Verify that the rendered HTML includes a div with the correct id
    // This test will use Dioxus testing utilities to verify component output
}
```

**Test: Map3D component initializes only once (prevents duplicate map instances)**
```rust
#[test]
fn test_map3d_initializes_once() {
    // Verify that the map_loaded signal prevents re-initialization
    // Test should verify that subsequent renders don't call initialization code
}
```

### Dioxus Integration Pattern Tests

**Test: use_effect cleanup function executes on unmount**
```rust
#[test]
fn test_use_effect_cleanup_on_unmount() {
    // Verify that cleanup function removes event listeners
    // and cleans up map resources when component unmounts
}
```

**Test: map_loaded signal prevents re-initialization**
```rust
#[test]
fn test_map_loaded_signal_prevents_reinit() {
    // Verify that once map_loaded is set, re-renders don't trigger
    // another initialization
}
```

**Test: document::eval return value is parsed correctly**
```rust
#[test]
fn test_eval_return_value_parsing() {
    // Verify that JSON return values from JavaScript are parsed
    // correctly into Rust structs (MapEvent, LoadEvent, etc.)
}
```

## Implementation

### File Structure

Create or modify the following files:

1. **`/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/map_3d.rs`**
   - Main Map3D component implementation

2. **`/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/mod.rs`**
   - Export the Map3D component (should already be done in section-01)

### Component Structure

The `Map3D` component is a Dioxus component that:

1. **Accepts configuration props** - Takes a `Map3DConfig` struct
2. **Manages initialization state** - Uses signals to track map loaded state
3. **Renders a container div** - Provides the DOM element for MapLibre
4. **Initializes MapLibre via JavaScript bridge** - Uses `document::eval()` to execute JavaScript

### Core Component Signature

```rust
// File: crates/iou-frontend/src/components/map_3d.rs

use dioxus::prelude::*;

#[component]
pub fn Map3D(
    #[props(default = Map3DConfig::default())] config: Map3DConfig,
) -> Element {
    // Component implementation
}
```

### State Management

The component manages the following state:

```rust
use_signal(|| false)  // map_loaded: bool
```

### Initialization Logic

The component uses `use_effect` to initialize the map only once:

```rust
use_effect(move || {
    // Only initialize if not already loaded
    if map_loaded() {
        return;
    }
    
    // Execute JavaScript initialization
    let init_js = build_initialization_script(&config);
    let _ = document::eval(&init_js);
    
    // Cleanup function
    move || {
        // Remove event listeners and cleanup
        let cleanup_js = format!("if (window.map_{}) {{ window.map_{}.remove(); }}", 
                                 config.container_id, config.container_id);
        let _ = document::eval(&cleanup_js);
    }
});
```

### JavaScript Initialization Script

The initialization script builds JavaScript code that:

1. Checks for WebGL2 support
2. Creates the MapLibre map instance
3. Configures initial view state
4. Sets up event listeners

```rust
fn build_initialization_script(config: &Map3DConfig) -> String {
    format!(r#"
        (function() {{
            if (!window.WebGL2RenderingContext) {{
                console.error('WebGL2 not supported');
                return;
            }}
            
            const container = document.getElementById('{}');
            if (!container) {{
                console.error('Container not found');
                return;
            }}
            
            // Check if map already exists
            if (window['map_{}']) {{
                console.log('Map already initialized');
                return;
            }}
            
            // Create map instance
            window['map_{}'] = new maplibregl.Map({{
                container: '{}',
                style: 'https://api.maptiler.com/maps/streets/style.json?key=YOUR_KEY',
                center: [{}, {}],
                zoom: {},
                pitch: {},
                bearing: {},
                minZoom: {},
                maxZoom: {},
            }});
            
            // Add navigation controls
            window['map_{}'].addControl(new maplibregl.NavigationControl({{ visualizePitch: true }}));
            
            // Map loaded event
            window['map_{}'].on('load', function() {{
                console.log('Map loaded');
                // Signal back to Rust
            }});
            
            // Error handling
            window['map_{}'].on('error', function(e) {{
                console.error('Map error:', e);
            }});
        }})();
    "#,
        config.container_id,
        config.container_id,
        config.container_id,
        config.container_id,
        config.center.0, config.center.1,  // lon, lat order!
        config.zoom,
        config.pitch,
        config.bearing,
        config.min_zoom,
        config.max_zoom,
        config.container_id,
        config.container_id,
        config.container_id
    )
}
```

### Important: Coordinate Order

**CRITICAL:** MapLibre uses `[longitude, latitude]` ordering while Leaflet uses `[latitude, longitude]`. 

- Leaflet: `L.map('map').setView([52.45, 5.50], 10)` (lat, lon)
- MapLibre: `center: [5.50, 52.45]` (lon, lat)

The `Map3DConfig.center` field stores coordinates as `(longitude, latitude)` to match MapLibre's expected order.

### Rendered HTML

The component renders a simple container div:

```rust
rsx! {
    div { 
        class: "maplibre-container",
        id: "{config.container_id}",
        style: "width: 100%; height: 100%;"
    }
}
```

### Event Bridge Pattern

JavaScript events communicate back to Rust via return values from `document::eval()`. The return value is a JSON string that Rust parses.

**Example event handling:**

```rust
#[derive(Deserialize)]
struct MapEvent {
    event: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

// In JavaScript:
// map.on('load', () => {
//     return JSON.stringify({event: 'load', zoom: map.getZoom()});
// });
```

### Error Handling

The component must handle:

1. **Missing WebGL2 support** - Show error message: "Uw browser ondersteunt geen 3D-kaarten"
2. **Container not found** - Log error and prevent initialization
3. **Duplicate initialization** - Check `window['map_{id}']` before creating
4. **Map loading errors** - Log to console for debugging

## Acceptance Criteria

The implementation is complete when:

1. The Map3D component compiles without errors
2. The component renders a container div with the correct id
3. MapLibre initializes via JavaScript bridge without errors
4. The map centers on Flevoland at the configured coordinates
5. Navigation controls appear on the map
6. The `map_loaded` signal prevents duplicate initialization
7. Cleanup function executes on component unmount
8. All tests pass

## Next Steps

After completing this section:
- **section-05-terrain-integration**: Configure terrain source and enable 3D terrain rendering
- **section-06-geojson-support**: Add GeoJSON layer management functions