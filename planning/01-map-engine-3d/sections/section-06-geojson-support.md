Now I have all the context needed. Let me generate the section content for `section-06-geojson-support`.

# Section 06: GeoJSON Support

## Overview

This section implements GeoJSON layer support for the Map3D component. This includes JavaScript bridge functions for adding and toggling layers, layer type detection from GeoJSON content, and migration of the two MVP layers (provinciegrens and cultuurhistorie).

## Dependencies

This section depends on:
- **section-04-map3d-component**: Provides the Map3D component and JavaScript bridge infrastructure
- **section-05-terrain-integration**: Ensures the map is properly initialized with terrain before layers are added

## Tests

### Layer Type Detection

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_detection_test.rs`

Create tests for detecting the primary geometry type from GeoJSON content:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_point_layer() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Point);
    }

    #[test]
    fn test_detect_line_layer() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Line);
    }

    #[test]
    fn test_detect_polygon_layer() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [{"type": "Feature", "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]]}}]
        }"#;
        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
    }

    #[test]
    fn test_detect_handles_empty_geojson() {
        let geojson = r#"{"type": "FeatureCollection", "features": []}"#;
        // Should return a default or error, not crash
        let result = detect_layer_type(geojson);
        // Implementation choice: return Polygon as default or Option<LayerType>
    }

    #[test]
    fn test_detect_mixed_geometries_logs_warning() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}},
                {"type": "Feature", "geometry": {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}}
            ]
        }"#;
        // Should log warning and return first detected type or Polygon as default
        let result = detect_layer_type(geojson);
    }
}
```

### JavaScript Generation Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_js_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_geojson_layer_generates_valid_javascript() {
        let layer = GeoJsonLayer {
            id: "test-layer".to_string(),
            name: "Test Layer".to_string(),
            url: "/test.geojson".to_string(),
            layer_type: LayerType::Polygon,
            visible: true,
            color: "#ff0000".to_string(),
        };
        
        let js = generate_add_layer_js(&layer);
        
        // Verify the JavaScript contains key elements
        assert!(js.contains("addSource"));
        assert!(js.contains("test-layer"));
        assert!(js.contains("addLayer"));
        assert!(js.contains("fill"));  // Polygon type
    }

    #[test]
    fn test_toggle_layer_generates_correct_visibility() {
        let js = generate_toggle_layer_js("test-layer", true);
        assert!(js.contains("setLayoutProperty"));
        assert!(js.contains("'visibility', 'visible'"));
        
        let js = generate_toggle_layer_js("test-layer", false);
        assert!(js.contains("'visibility', 'none'"));
    }

    #[test]
    fn test_layer_color_applied_correctly() {
        let layer = GeoJsonLayer {
            id: "test".to_string(),
            name: "Test".to_string(),
            url: "/test.geojson".to_string(),
            layer_type: LayerType::Point,
            visible: true,
            color: "#00ff00".to_string(),
        };
        
        let js = generate_add_layer_js(&layer);
        assert!(js.contains("#00ff00"));
        assert!(js.contains("circle-color"));  // Point type uses circle
    }
}
```

### Integration Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/tests/geojson_integration_test.rs`

```rust
#[cfg(test)]
mod tests {
    use dioxus::prelude::*;

    #[test]
    fn test_geojson_layer_definition_creation() {
        let layer = GeoJsonLayer::new(
            "provinciegrens",
            "Provinciegrens",
            "/geojson/provinciegrens.geojson",
            LayerType::Polygon,
            "#4488ff"
        );
        
        assert_eq!(layer.id, "provinciegrens");
        assert_eq!(layer.visible, true);  // Default
        assert_eq!(layer.color, "#4488ff");
    }
}
```

## Implementation

### 1. Layer Type Detection Module

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_detection.rs`

Implement layer type detection from GeoJSON content:

```rust
use serde_json::Value;

/// Detects the primary geometry type from GeoJSON content.
/// Returns the first geometry type found, or Polygon as default for mixed/empty content.
pub fn detect_layer_type(geojson_content: &str) -> LayerType {
    let json: Value = match serde_json::from_str(geojson_content) {
        Ok(v) => v,
        Err(_) => return LayerType::Polygon, // Default for invalid JSON
    };

    // Navigate to features array
    let features = json
        .get("features")
        .and_then(|f| f.as_array());

    let Some(features) = features else {
        return LayerType::Polygon;
    };

    if features.is_empty() {
        return LayerType::Polygon;
    }

    // Check first feature's geometry type
    let first_geometry = features
        .first()
        .and_then(|f| f.get("geometry"))
        .and_then(|g| g.get("type"))
        .and_then(|t| t.as_str());

    match first_geometry {
        Some("Point") | Some("MultiPoint") => LayerType::Point,
        Some("LineString") | Some("MultiLineString") => LayerType::Line,
        Some("Polygon") | Some("MultiPolygon") => LayerType::Polygon,
        _ => LayerType::Polygon,
    }
}
```

### 2. JavaScript Bridge Functions

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/map_3d.rs`

Extend the Map3D component with layer management functions:

```rust
impl Map3D {
    /// Generates JavaScript to add a GeoJSON layer to the map.
    /// This function does not execute the JavaScript directly - it returns
    /// the JS string for use with document::eval().
    pub fn generate_add_layer_js(layer: &GeoJsonLayer) -> String {
        let (layer_type, paint_properties) = match layer.layer_type {
            LayerType::Point => (
                "circle",
                format!(
                    r#"'circle-color': '{}', 'circle-radius': 6"#,
                    layer.color
                )
            ),
            LayerType::Line => (
                "line",
                format!(
                    r#"'line-color': '{}', 'line-width': 2"#,
                    layer.color
                )
            ),
            LayerType::Polygon => (
                "fill",
                format!(
                    r#"'fill-color': '{}', 'fill-opacity': 0.3"#,
                    layer.color
                )
            ),
        };

        format!(
            r#"
            (function() {{
                const map = window.map3dInstances['{}'];
                if (!map) return {{ error: 'Map not found' }};

                try {{
                    // Add source if not exists
                    if (!map.getSource('{}')) {{
                        map.addSource('{}', {{
                            type: 'geojson',
                            data: '{}'
                        }});
                    }}

                    // Add layer if not exists
                    if (!map.getLayer('{}')) {{
                        map.addLayer({{
                            id: '{}',
                            type: '{}',
                            source: '{}',
                            paint: {{ {} }}
                        }});
                    }}

                    return {{ success: true, layer: '{}' }};
                }} catch (e) {{
                    return {{ error: e.toString() }};
                }}
            }})()
            "#,
            layer.id, layer.id, layer.id, layer.url, layer.id, layer.id, layer_type, layer.id, paint_properties, layer.id
        )
    }

    /// Generates JavaScript to toggle layer visibility.
    pub fn generate_toggle_layer_js(layer_id: &str, visible: bool) -> String {
        let visibility = if visible { "visible" } else { "none" };

        format!(
            r#"
            (function() {{
                const map = window.map3dInstances['{}'];
                if (!map) return {{ error: 'Map not found' }};

                try {{
                    map.setLayoutProperty('{}', 'visibility', '{}');
                    return {{ success: true, visible: {} }};
                }} catch (e) {{
                    return {{ error: e.toString() }};
                }}
            }})()
            "#,
            layer_id, layer_id, visibility, visible
        )
    }

    /// Adds a GeoJSON layer to the map via the JavaScript bridge.
    pub async fn add_geojson_layer(&self, layer: &GeoJsonLayer) -> Result<(), String> {
        let js = Self::generate_add_layer_js(layer);
        let result = document::eval(&js)
            .await
            .map_err(|e| e.to_string())?;

        if result.contains("error") {
            // Parse error from JSON response
            return Err(result);
        }

        Ok(())
    }

    /// Toggles a layer's visibility.
    pub async fn toggle_layer(&self, layer_id: &str, visible: bool) -> Result<(), String> {
        let js = Self::generate_toggle_layer_js(layer_id, visible);
        let result = document::eval(&js)
            .await
            .map_err(|e| e.to_string())?;

        if result.contains("error") {
            return Err(result);
        }

        Ok(())
    }
}
```

### 3. Layer Definition Structure

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_control_3d.rs`

Define the layer structures (may already exist from section-04, extend as needed):

```rust
use serde::{Deserialize, Serialize};

/// Represents the geometry type of a GeoJSON layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    Point,
    Line,
    Polygon,
}

/// Configuration for a GeoJSON layer to display on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonLayer {
    pub id: String,
    pub name: String,
    pub url: String,
    pub layer_type: LayerType,
    pub visible: bool,
    pub color: String,
}

impl GeoJsonLayer {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        url: impl Into<String>,
        layer_type: LayerType,
        color: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            url: url.into(),
            layer_type,
            visible: true,  // Default to visible
            color: color.into(),
        }
    }
}
```

### 4. MVP Layer Definitions

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs`

Define the two MVP layers to migrate:

```rust
/// Returns the GeoJSON layers to display on the 3D map (MVP set).
pub fn get_mvp_geojson_layers() -> Vec<GeoJsonLayer> {
    vec![
        GeoJsonLayer::new(
            "provinciegrens",
            "Provinciegrens",
            "/geojson/provinciegrens.geojson",
            LayerType::Polygon,
            "#4488ff"  // Blue
        ),
        GeoJsonLayer::new(
            "cultuurhistorie",
            "Cultuurhistorie",
            "/geojson/cultuurhistorie.geojson",
            LayerType::Point,
            "#ff8800"  // Orange
        ),
    ]
}
```

**Note:** Verify the actual geometry types of these GeoJSON files:
- `provinciegrens.geojson` - likely Polygon (provincial boundary)
- `cultuurhistorie.geojson` - verify if Point, Polygon, or mixed

### 5. MapLibre Layer Type Mapping

| GeoJSON Geometry Type | MapLibre Layer Type | Paint Properties |
|----------------------|---------------------|------------------|
| Point, MultiPoint | `circle` | circle-color, circle-radius |
| LineString, MultiLineString | `line` | line-color, line-width |
| Polygon, MultiPolygon | `fill` | fill-color, fill-opacity |

### 6. Global Window State Management

The JavaScript functions reference `window.map3dInstances` for map instance storage. This should be initialized in section-04. If not present, add:

```javascript
// In map initialization code (section-04):
window.map3dInstances = window.map3dInstances || {};
window.map3dInstances['{map_id}'] = map;
```

## File Paths

| File | Action |
|------|--------|
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_detection.rs` | Create |
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_detection_test.rs` | Create |
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_js_test.rs` | Create |
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/map_3d.rs` | Modify (add layer functions) |
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/components/layer_control_3d.rs` | Modify (add structures) |
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/pages/data_verkenner.rs` | Modify (add layer definitions) |
| `/Users/marc/Projecten/iou-modern/crates/iou-frontend/tests/geojson_integration_test.rs` | Create |

## Error Handling

1. **Invalid GeoJSON URL**: The JavaScript bridge returns an error object. The Rust side should:
   - Parse the error message
   - Log to console
   - Optionally show a toast notification to the user

2. **Malformed GeoJSON**: 
   - MapLibre will fail to render the layer
   - Catch this in the JavaScript try/catch block
   - Return error details to Rust for logging

3. **Missing Map Instance**:
   - Check if `window.map3dInstances[id]` exists before operations
   - Return error if map is not initialized

## Validation Checklist

After implementation, verify:

- [ ] Layer type detection correctly identifies Point, Line, and Polygon geometries
- [ ] `generate_add_layer_js()` produces valid JavaScript that executes without errors
- [ ] `generate_toggle_layer_js()` correctly switches between 'visible' and 'none'
- [ ] GeoJSON layers appear on the map with correct colors
- [ ] Layer toggle functionality works smoothly
- [ ] provinciegrens layer displays as a polygon boundary
- [ ] cultuurhistorie layer displays with appropriate styling
- [ ] All unit tests pass (`cargo test -p iou-frontend`)
- [ ] Browser console shows no errors when loading layers

---

## Implementation Notes

**Date:** 2026-03-03
**Status:** ✅ Complete

### Files Created
- `crates/iou-frontend/src/components/layer_detection.rs` - Layer type detection module

### Files Modified
- `crates/iou-frontend/src/components/map_3d.rs` - Added JavaScript generation functions
- `crates/iou-frontend/src/components/mod.rs` - Added module exports

### Changes Made

1. **Layer Detection Module** (`layer_detection.rs`)
   - `detect_layer_type()` - Detects geometry type from GeoJSON content
   - `has_mixed_geometries()` - Checks for mixed geometry types
   - Returns Point, Line, or Polygon based on first feature's geometry

2. **JavaScript Generation Functions** (`map_3d.rs`)
   - `build_add_geojson_layer_script()` - Generates JS to add GeoJSON layer
   - `build_toggle_layer_visibility_script()` - Generates JS to toggle visibility
   - `build_remove_layer_script()` - Generates JS to remove layer

3. **Security Enhancements**
   - `is_valid_hex_color()` - Validates hex color format (#RRGGBB)
   - Color validation assertion in `build_add_geojson_layer_script()`
   - Layer ID validation assertion using `is_valid_container_id()`

4. **Tests Added** (104 total tests passing)
   - 17 layer detection tests (Point, Line, Polygon, Multi*, empty, invalid, mixed)
   - 8 GeoJSON JavaScript generation tests
   - 1 color validation test

### Deviations from Plan

1. **Map Instance Reference**: Uses `window['map_{}']` instead of `window.map3dInstances['{}']`
   - Reason: Consistent with section-05 pattern

2. **Integration Test File**: Not created at `tests/geojson_integration_test.rs`
   - Reason: Binary-only crate (main.rs) - integration tests require lib.rs

3. **Function Names**: Used `build_*_script()` prefix instead of `generate_*_js()`
   - Reason: Consistent naming with existing terrain functions

4. **Layer ID Escaping**: Uses validation instead of escaping for layer IDs
   - Reason: `is_valid_container_id()` restricts to safe characters only

### MVP Layer Definitions

Already defined in `layer_control_3d.rs`:
- `provinciegrens` - Polygon, blue (#e74c3c), visible by default
- `cultuurhistorie` - Point, blue (#3498db), hidden by default

### MapLibre Layer Type Mapping

| GeoJSON Geometry | MapLibre Type | Paint Properties |
|-----------------|---------------|------------------|
| Point, MultiPoint | circle | circle-color, circle-radius: 6 |
| LineString, MultiLineString | line | line-color, line-width: 2 |
| Polygon, MultiPolygon | fill | fill-color, fill-opacity: 0.3 |

### Environment Variables Required

None for this section.