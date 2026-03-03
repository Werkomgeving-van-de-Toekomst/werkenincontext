Now I have all the context needed to generate the section-08 content. Let me create a comprehensive, self-contained section for page integration.

# Section 08: Page Integration

## Overview

This section integrates the Map3D and LayerControl3D components into the Data Verkenner page. The integration uses a feature flag (`MAP_3D_ENABLED`) to allow conditional rendering between the legacy Leaflet map and the new 3D MapLibre implementation. This enables safe deployment, testing, and rollback without removing the existing Leaflet implementation.

## Dependencies

This section depends on:
- **section-04-map3d-component**: Provides the Map3D component with MapLibre initialization
- **section-05-terrain-integration**: Provides terrain rendering capability
- **section-06-geojson-support**: Provides GeoJSON layer management functions
- **section-07-layer-control**: Provides the LayerControl3D UI component

## File to Modify

**Primary file:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

## Tests

Write these tests before implementing the integration:

### Test: Data Verkenner page renders without crash

```rust
// File: crates/iou-frontend/tests/data_verkenner_test.rs
use dioxus::prelude::*;

#[test]
fn test_data_verkenner_renders_without_crash() {
    // Verify that the DataVerkenner component renders successfully
    // with both MAP_3D_ENABLED=true and MAP_3D_ENABLED=false
}
```

### Test: MAP_3D_ENABLED=false renders Leaflet map

```rust
#[test]
fn test_map_3d_disabled_renders_leaflet() {
    // When MAP_3D_ENABLED is false or unset:
    // 1. The map container div should have id="map"
    // 2. Leaflet initialization JavaScript should be present
    // 3. Map3D component should NOT be rendered
}
```

### Test: MAP_3D_ENABLED=true renders Map3D component

```rust
#[test]
fn test_map_3d_enabled_renders_map3d() {
    // When MAP_3D_ENABLED is true:
    // 1. Map3D component should be rendered
    // 2. Map container should use MapLibre initialization
    // 3. Leaflet initialization should NOT execute
}
```

## Implementation Details

### Current Data Verkenner Structure

The existing Data Verkenner page (`/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`) contains:

1. A dataset selector with mock data visualization
2. A Leaflet map initialized via `use_effect` with inline JavaScript
3. Six GeoJSON layers loaded via Leaflet
4. A two-column layout with dataset info and visualization panels

### Integration Approach

The integration maintains the existing page layout while conditionally rendering either the Leaflet map or the Map3D component based on the `MAP_3D_ENABLED` environment variable.

### Step 1: Add Imports

Add the necessary imports at the top of the file:

```rust
use dioxus::prelude::*;
use std::env;

use crate::components::{Header, Panel};
use crate::components::Map3D;
use crate::components::LayerControl3D;
use crate::components::{Map3DConfig, GeoJsonLayer, LayerType};
```

### Step 2: Define 3D Layer Configuration

Add a function to define the layers for the 3D map:

```rust
/// Returns the GeoJSON layers configured for 3D map display (MVP set).
/// 
/// This parallels the Leaflet layers but uses the MapLibre configuration format.
fn get_3d_map_layers() -> Vec<GeoJsonLayer> {
    vec![
        GeoJsonLayer::new(
            "provinciegrens",
            "Provinciegrens",
            "/geojson/provinciegrens.geojson",
            LayerType::Polygon,
            "#4488ff"
        ),
        GeoJsonLayer::new(
            "cultuurhistorie",
            "Cultuurhistorie",
            "/geojson/cultuurhistorie.geojson",
            LayerType::Point,
            "#ff8800"
        ),
    ]
}
```

### Step 3: Add Feature Flag Helper

Add a helper function to check the feature flag:

```rust
/// Checks if the 3D map is enabled via the MAP_3D_ENABLED environment variable.
/// 
/// Defaults to false for safety - the feature must be explicitly enabled.
fn is_3d_map_enabled() -> bool {
    env::var("MAP_3D_ENABLED")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
```

### Step 4: Modify the DataVerkenner Component

The key modification is to conditionally execute either the Leaflet initialization or render the Map3D component. The existing page structure (dataset selector, visualization panels) remains unchanged.

#### Existing Leaflet Initialization (Preserved)

The existing `use_effect` block with Leaflet initialization should be wrapped in a condition:

```rust
use_effect(move || {
    // Only initialize Leaflet if 3D map is NOT enabled
    if is_3d_map_enabled() {
        return;
    }
    
    let script = r#"
        // ... existing Leaflet initialization code ...
    "#;
    document::eval(script);
});
```

#### Map Container Rendering

Modify the map panel to include the 3D components when enabled:

```rust
Panel { title: "Kaart".to_string(),
    div {
        id: "map",
        style: "height: 550px; border-radius: 8px;",
        
        // Render 3D components when enabled
        if is_3d_map_enabled() {
            rsx! {
                Map3D {
                    config: Map3DConfig::default(),
                    layers: get_3d_map_layers(),
                }
                LayerControl3D {
                    layers: get_3d_map_layers(),
                    map_id: "map".to_string(),
                }
            }
        }
    }
}
```

### Step 5: Coordinate Order Considerations

**CRITICAL:** The existing Leaflet code uses `[latitude, longitude]` ordering:

```javascript
// Leaflet (existing)
L.map('map').setView([52.45, 5.50], 10);  // [lat, lon]
```

The Map3D configuration uses `[longitude, latitude]` ordering (as defined in section-02):

```rust
// Map3D configuration
center: (5.5, 52.4),  // (lon, lat)
```

Ensure the `Map3DConfig::default()` returns the correct Flevoland center coordinates.

### Step 6: Component Props

The Map3D component accepts the following props (from section-04):

```rust
#[component]
pub fn Map3D(
    #[props(default = Map3DConfig::default())] config: Map3DConfig,
) -> Element
```

The LayerControl3D component accepts (from section-07):

```rust
#[component]
pub fn LayerControl3D(
    layers: Vec<GeoJsonLayer>,
    map_id: String,
) -> Element
```

### Complete Modified Component Structure

```rust
#[component]
pub fn DataVerkenner() -> Element {
    let mut selected = use_signal(|| 0usize);
    let use_3d_map = is_3d_map_enabled();

    // Leaflet initialization (only when 3D is disabled)
    use_effect(move || {
        if use_3d_map {
            return;  // Skip Leaflet init when 3D is enabled
        }
        
        // ... existing Leaflet script ...
    });

    let idx = *selected.read();
    let dataset = &DATASETS[idx];

    rsx! {
        Header {}
        main { class: "container",
            // Dataset selector and visualization panels (unchanged)
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                Panel { title: "Datasets".to_string(),
                    // ... existing dataset selector content ...
                }
                Panel { title: "Visualisatie".to_string(),
                    // ... existing visualization content ...
                }
            }

            div { style: "height: 20px;" }

            // Map panel with conditional rendering
            Panel { title: "Kaart".to_string(),
                div {
                    id: "map",
                    style: "height: 550px; border-radius: 8px;",
                    
                    if use_3d_map {
                        rsx! {
                            Map3D {
                                config: Map3DConfig::default(),
                            }
                            LayerControl3D {
                                layers: get_3d_map_layers(),
                                map_id: "map".to_string(),
                            }
                        }
                    }
                }
            }
        }
    }
}
```

## Environment Configuration

### Development

To test the 3D map locally:

```bash
# Enable 3D map
export MAP_3D_ENABLED=true
cargo run

# Disable 3D map (use Leaflet)
unset MAP_3D_ENABLED
# or
export MAP_3D_ENABLED=false
cargo run
```

### Production Deployment

For initial deployment, deploy with `MAP_3D_ENABLED=false` (default). The feature flag can be enabled via environment configuration without code changes.

## Styling Considerations

### Map Container Height

Both Leaflet and MapLibre require explicit height for the map container:

```css
#map {
    height: 550px;
    border-radius: 8px;
}
```

### Layer Control Positioning

The LayerControl3D component should overlay the map similar to Leaflet's layer control. This is handled by the component implementation (section-07).

## Rollback Strategy

If issues are discovered with the 3D map implementation:

1. Set `MAP_3D_ENABLED=false` in the environment
2. The application immediately falls back to Leaflet
3. No code changes or redeployment required

This feature flag approach ensures safe, gradual rollout of the 3D functionality.

## Gradual Migration Path

### Phase 1: Deployment (This Section)

- Deploy with both rendering modes available
- Default to Leaflet (`MAP_3D_ENABLED=false`)
- Test 3D map in staging environment

### Phase 2: Beta Testing

- Enable for subset of users via A/B testing
- Monitor performance and error rates
- Gather user feedback

### Phase 3: Gradual Rollout

- Increase percentage of users with 3D enabled
- Monitor browser compatibility
- Address any Safari-specific issues

### Phase 4: Full Cutover (Future, Out of Scope)

- Set `MAP_3D_ENABLED=true` by default
- After successful production run, remove Leaflet code
- This removal is NOT part of this implementation plan

## Error Handling

### Map Initialization Failure

If MapLibre fails to initialize (e.g., WebGL2 not supported), the Map3D component should handle errors gracefully. The page should remain functional with an error message displayed in the map container.

### Feature Flag Reads

The `is_3d_map_enabled()` function safely handles missing environment variables by defaulting to `false` (Leaflet mode).

## File Paths Summary

| File | Action |
|------|--------|
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs` | Modify - Add conditional rendering |
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/tests/data_verkenner_test.rs` | Create - Integration tests |

## Browser Testing Checklist

After implementation, manually test both rendering modes:

**Leaflet Mode (MAP_3D_ENABLED=false):**
- [ ] Map loads centered on Flevoland
- [ ] All six layers load correctly
- [ ] Layer control toggles work
- [ ] Dataset selector functions normally
- [ ] Visualization panels display correctly

**3D Mode (MAP_3D_ENABLED=true):**
- [ ] Map loads centered on Flevoland
- [ ] Terrain is visible when tilted
- [ ] MVP layers (provinciegrens, cultuurhistorie) display
- [ ] Layer control toggles work
- [ ] Navigation controls (pitch, rotate, zoom) work
- [ ] Dataset selector and visualization panels unaffected

## Completion Criteria

This section is complete when:

1. The DataVerkenner page compiles without errors
2. `MAP_3D_ENABLED=false` renders the existing Leaflet map
3. `MAP_3D_ENABLED=true` renders the Map3D component
4. The dataset selector and visualization panels work in both modes
5. All integration tests pass
6. Both rendering modes have been manually tested in Chrome and Firefox
7. The feature flag can be toggled without code changes
8. No console errors appear in either mode

## Next Steps

After completing this section:
- **section-09-testing**: Complete testing suite, browser compatibility verification, and acceptance criteria validation