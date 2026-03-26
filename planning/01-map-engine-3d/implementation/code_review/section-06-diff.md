diff --git a/crates/iou-frontend/src/components/layer_detection.rs b/crates/iou-frontend/src/components/layer_detection.rs
new file mode 100644
index 0000000..919e457
--- /dev/null
+++ b/crates/iou-frontend/src/components/layer_detection.rs
@@ -0,0 +1,265 @@
+//! Layer Detection - Detect layer type from GeoJSON content
+//!
+//! This module provides functionality to analyze GeoJSON content
+//! and determine the primary geometry type for proper rendering.
+
+use crate::components::layer_control_3d::LayerType;
+use serde_json::Value;
+
+/// Detects the primary geometry type from GeoJSON content.
+///
+/// Analyzes the GeoJSON and returns the first geometry type found.
+/// For mixed or empty content, returns Polygon as a sensible default.
+///
+/// # Arguments
+///
+/// * `geojson_content` - The GeoJSON content as a string
+///
+/// # Returns
+///
+/// The detected `LayerType` (Point, Line, or Polygon)
+///
+/// # Example
+///
+/// ```rust
+/// let geojson = r#"{
+///     "type": "FeatureCollection",
+///     "features": [{"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}}]
+/// }"#;
+/// assert_eq!(detect_layer_type(geojson), LayerType::Point);
+/// ```
+pub fn detect_layer_type(geojson_content: &str) -> LayerType {
+    let json: Value = match serde_json::from_str(geojson_content) {
+        Ok(v) => v,
+        Err(_) => return LayerType::Polygon, // Default for invalid JSON
+    };
+
+    // Navigate to features array
+    let features = json
+        .get("features")
+        .and_then(|f| f.as_array());
+
+    let Some(features) = features else {
+        return LayerType::Polygon;
+    };
+
+    if features.is_empty() {
+        return LayerType::Polygon;
+    }
+
+    // Check first feature's geometry type
+    let first_geometry = features
+        .first()
+        .and_then(|f| f.get("geometry"))
+        .and_then(|g| g.get("type"))
+        .and_then(|t| t.as_str());
+
+    match first_geometry {
+        Some("Point") | Some("MultiPoint") => LayerType::Point,
+        Some("LineString") | Some("MultiLineString") => LayerType::Line,
+        Some("Polygon") | Some("MultiPolygon") => LayerType::Polygon,
+        _ => LayerType::Polygon,
+    }
+}
+
+/// Checks if GeoJSON content contains mixed geometry types.
+///
+/// Returns true if the FeatureCollection contains features with
+/// different geometry types.
+///
+/// # Arguments
+///
+/// * `geojson_content` - The GeoJSON content as a string
+///
+/// # Returns
+///
+/// true if mixed geometries are detected, false otherwise
+pub fn has_mixed_geometries(geojson_content: &str) -> bool {
+    let json: Value = match serde_json::from_str(geojson_content) {
+        Ok(v) => v,
+        Err(_) => return false,
+    };
+
+    let features = json
+        .get("features")
+        .and_then(|f| f.as_array());
+
+    let Some(features) = features else {
+        return false;
+    };
+
+    if features.len() < 2 {
+        return false;
+    }
+
+    // Get the first geometry type
+    let first_type = features
+        .first()
+        .and_then(|f| f.get("geometry"))
+        .and_then(|g| g.get("type"))
+        .and_then(|t| t.as_str());
+
+    let Some(first_type) = first_type else {
+        return false;
+    };
+
+    // Check if any feature has a different geometry type
+    for feature in features.iter().skip(1) {
+        let geom_type = feature
+            .get("geometry")
+            .and_then(|g| g.get("type"))
+            .and_then(|t| t.as_str());
+
+        if let Some(gt) = geom_type {
+            if gt != first_type {
+                return true;
+            }
+        }
+    }
+
+    false
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_detect_point_layer() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [{"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}}]
+        }"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Point);
+    }
+
+    #[test]
+    fn test_detect_multipoint_layer() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [{"type": "Feature", "geometry": {"type": "MultiPoint", "coordinates": [[0, 0], [1, 1]]}}]
+        }"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Point);
+    }
+
+    #[test]
+    fn test_detect_line_layer() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [{"type": "Feature", "geometry": {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}}]
+        }"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Line);
+    }
+
+    #[test]
+    fn test_detect_multilinestring_layer() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [{"type": "Feature", "geometry": {"type": "MultiLineString", "coordinates": [[[0, 0], [1, 1]], [[2, 2], [3, 3]]]}}]
+        }"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Line);
+    }
+
+    #[test]
+    fn test_detect_polygon_layer() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [{"type": "Feature", "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]]}}]
+        }"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
+    }
+
+    #[test]
+    fn test_detect_multipolygon_layer() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [{"type": "Feature", "geometry": {"type": "MultiPolygon", "coordinates": [[[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]]]}}]
+        }"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
+    }
+
+    #[test]
+    fn test_detect_handles_empty_geojson() {
+        let geojson = r#"{"type": "FeatureCollection", "features": []}"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
+    }
+
+    #[test]
+    fn test_detect_handles_invalid_json() {
+        let geojson = r#"not valid json"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
+    }
+
+    #[test]
+    fn test_detect_mixed_geometries_returns_first_type() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [
+                {"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}},
+                {"type": "Feature", "geometry": {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}}
+            ]
+        }"#;
+        // Returns the first type detected
+        assert_eq!(detect_layer_type(geojson), LayerType::Point);
+    }
+
+    #[test]
+    fn test_detect_handles_no_features_field() {
+        let geojson = r#"{"type": "FeatureCollection"}"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
+    }
+
+    #[test]
+    fn test_detect_handles_null_geometry() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [{"type": "Feature", "geometry": null}]
+        }"#;
+        assert_eq!(detect_layer_type(geojson), LayerType::Polygon);
+    }
+
+    #[test]
+    fn test_has_mixed_geometries_with_same_type() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [
+                {"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}},
+                {"type": "Feature", "geometry": {"type": "Point", "coordinates": [1, 1]}}
+            ]
+        }"#;
+        assert!(!has_mixed_geometries(geojson));
+    }
+
+    #[test]
+    fn test_has_mixed_geometries_with_different_types() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [
+                {"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}},
+                {"type": "Feature", "geometry": {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}}
+            ]
+        }"#;
+        assert!(has_mixed_geometries(geojson));
+    }
+
+    #[test]
+    fn test_has_mixed_geometries_with_single_feature() {
+        let geojson = r#"{
+            "type": "FeatureCollection",
+            "features": [{"type": "Feature", "geometry": {"type": "Point", "coordinates": [0, 0]}}]
+        }"#;
+        assert!(!has_mixed_geometries(geojson));
+    }
+
+    #[test]
+    fn test_has_mixed_geometries_with_empty_features() {
+        let geojson = r#"{"type": "FeatureCollection", "features": []}"#;
+        assert!(!has_mixed_geometries(geojson));
+    }
+
+    #[test]
+    fn test_has_mixed_geometries_with_invalid_json() {
+        let geojson = r#"not valid json"#;
+        assert!(!has_mixed_geometries(geojson));
+    }
+}
diff --git a/crates/iou-frontend/src/components/map_3d.rs b/crates/iou-frontend/src/components/map_3d.rs
index 4283295..4c1fd82 100644
--- a/crates/iou-frontend/src/components/map_3d.rs
+++ b/crates/iou-frontend/src/components/map_3d.rs
@@ -783,6 +783,182 @@ pub fn TerrainWarning(message: String) -> Element {
     }
 }
 
+// ============================================================================
+// GeoJSON Layer Management Functions
+// ============================================================================
+
+use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};
+
+/// Generates JavaScript to add a GeoJSON layer to the map.
+///
+/// This function generates JavaScript that:
+/// 1. Checks if the map instance exists
+/// 2. Adds a GeoJSON source if it doesn't exist
+/// 3. Adds a layer with appropriate styling based on geometry type
+///
+/// # Arguments
+///
+/// * `container_id` - The map container ID
+/// * `layer` - The GeoJSON layer configuration
+///
+/// # Returns
+///
+/// A JavaScript string that can be executed via eval()
+pub fn build_add_geojson_layer_script(container_id: &str, layer: &GeoJsonLayer) -> String {
+    assert!(is_valid_container_id(container_id), "Invalid container_id");
+
+    let layer_id_escaped = js_escape_string(&layer.id);
+    let url_escaped = js_escape_string(&layer.url);
+    let color_escaped = js_escape_string(&layer.color);
+
+    let (layer_type, paint_properties) = match layer.layer_type {
+        LayerType::Point => (
+            "circle",
+            format!(r#"'circle-color': '{}', 'circle-radius': 6"#, color_escaped)
+        ),
+        LayerType::Line => (
+            "line",
+            format!(r#"'line-color': '{}', 'line-width': 2"#, color_escaped)
+        ),
+        LayerType::Polygon => (
+            "fill",
+            format!(r#"'fill-color': '{}', 'fill-opacity': 0.3"#, color_escaped)
+        ),
+    };
+
+    format!(r#"
+        (function() {{
+            const map = window['map_{}'];
+            if (!map) {{
+                console.error('Map not found for layer: {}');
+                return {{ success: false, error: 'Map not found' }};
+            }}
+
+            try {{
+                // Add source if not exists
+                if (!map.getSource('{}')) {{
+                    map.addSource('{}', {{
+                        type: 'geojson',
+                        data: '{}'
+                    }});
+                }}
+
+                // Add layer if not exists
+                if (!map.getLayer('{}')) {{
+                    map.addLayer({{
+                        id: '{}',
+                        type: '{}',
+                        source: '{}',
+                        paint: {{ {} }}
+                    }});
+                }}
+
+                return {{ success: true, layer: '{}' }};
+            }} catch (e) {{
+                console.error('Error adding layer:', e);
+                return {{ success: false, error: e.toString() }};
+            }}
+        }})();
+    "#,
+        container_id,
+        layer.id,
+        layer.id,
+        layer.id,
+        url_escaped,
+        layer.id,
+        layer.id,
+        layer_type,
+        layer.id,
+        paint_properties,
+        layer.id
+    )
+}
+
+/// Generates JavaScript to toggle a layer's visibility.
+///
+/// # Arguments
+///
+/// * `container_id` - The map container ID
+/// * `layer_id` - The ID of the layer to toggle
+/// * `visible` - Whether the layer should be visible
+///
+/// # Returns
+///
+/// A JavaScript string that can be executed via eval()
+pub fn build_toggle_layer_visibility_script(container_id: &str, layer_id: &str, visible: bool) -> String {
+    assert!(is_valid_container_id(container_id), "Invalid container_id");
+    assert!(is_valid_container_id(layer_id), "Invalid layer_id");
+
+    let visibility = if visible { "visible" } else { "none" };
+
+    format!(r#"
+        (function() {{
+            const map = window['map_{}'];
+            if (!map) {{
+                console.error('Map not found');
+                return {{ success: false, error: 'Map not found' }};
+            }}
+
+            try {{
+                map.setLayoutProperty('{}', 'visibility', '{}');
+                return {{ success: true, visible: {} }};
+            }} catch (e) {{
+                console.error('Error toggling layer:', e);
+                return {{ success: false, error: e.toString() }};
+            }}
+        }})();
+    "#,
+        container_id,
+        layer_id,
+        visibility,
+        visible
+    )
+}
+
+/// Generates JavaScript to remove a layer from the map.
+///
+/// # Arguments
+///
+/// * `container_id` - The map container ID
+/// * `layer_id` - The ID of the layer to remove
+///
+/// # Returns
+///
+/// A JavaScript string that can be executed via eval()
+pub fn build_remove_layer_script(container_id: &str, layer_id: &str) -> String {
+    assert!(is_valid_container_id(container_id), "Invalid container_id");
+    assert!(is_valid_container_id(layer_id), "Invalid layer_id");
+
+    format!(r#"
+        (function() {{
+            const map = window['map_{}'];
+            if (!map) {{
+                console.error('Map not found');
+                return {{ success: false, error: 'Map not found' }};
+            }}
+
+            try {{
+                if (map.getLayer('{}')) {{
+                    map.removeLayer('{}');
+                }}
+                if (map.getSource('{}')) {{
+                    map.removeSource('{}');
+                }}
+                return {{ success: true }};
+            }} catch (e) {{
+                console.error('Error removing layer:', e);
+                return {{ success: false, error: e.toString() }};
+            }}
+        }})();
+    "#,
+        container_id,
+        layer_id,
+        layer_id,
+        layer_id,
+        layer_id
+    )
+}
+
 // Note: The actual Map3D Dioxus component requires web-specific APIs
 // that are not available in the test environment. The component will be
 // implemented in a future section when we can test it in a browser context.
@@ -1146,4 +1322,114 @@ mod component_tests {
         assert!(!is_valid_container_id("map;alert(1)"));
         assert!(!is_valid_container_id("map space"));
     }
+
+    // GeoJSON layer management tests (section-06)
+
+    #[test]
+    fn test_build_add_geojson_layer_script_generates_valid_javascript() {
+        use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};
+
+        let layer = GeoJsonLayer {
+            id: "test-layer".to_string(),
+            name: "Test Layer".to_string(),
+            url: "/test.geojson".to_string(),
+            layer_type: LayerType::Polygon,
+            visible: true,
+            color: "#ff0000".to_string(),
+        };
+
+        let js = build_add_geojson_layer_script("map", &layer);
+
+        // Verify the JavaScript contains key elements
+        assert!(js.contains("addSource"));
+        assert!(js.contains("test-layer"));
+        assert!(js.contains("addLayer"));
+        assert!(js.contains("fill"));  // Polygon type
+        assert!(js.contains("fill-color"));
+        assert!(js.contains("#ff0000"));
+    }
+
+    #[test]
+    fn test_build_add_geojson_layer_script_handles_point_type() {
+        use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};
+
+        let layer = GeoJsonLayer {
+            id: "points".to_string(),
+            name: "Points".to_string(),
+            url: "/points.geojson".to_string(),
+            layer_type: LayerType::Point,
+            visible: true,
+            color: "#00ff00".to_string(),
+        };
+
+        let js = build_add_geojson_layer_script("map", &layer);
+
+        assert!(js.contains("circle"));  // Point type
+        assert!(js.contains("circle-color"));
+        assert!(js.contains("#00ff00"));
+        assert!(js.contains("circle-radius"));
+    }
+
+    #[test]
+    fn test_build_add_geojson_layer_script_handles_line_type() {
+        use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};
+
+        let layer = GeoJsonLayer {
+            id: "lines".to_string(),
+            name: "Lines".to_string(),
+            url: "/lines.geojson".to_string(),
+            layer_type: LayerType::Line,
+            visible: true,
+            color: "#0000ff".to_string(),
+        };
+
+        let js = build_add_geojson_layer_script("map", &layer);
+
+        assert!(js.contains("line"));  // Line type
+        assert!(js.contains("line-color"));
+        assert!(js.contains("#0000ff"));
+        assert!(js.contains("line-width"));
+    }
+
+    #[test]
+    fn test_build_toggle_layer_visibility_script_visible() {
+        let js = build_toggle_layer_visibility_script("map", "test-layer", true);
+        assert!(js.contains("setLayoutProperty"));
+        assert!(js.contains("'visibility', 'visible'"));
+        assert!(js.contains("window['map_map']"));
+    }
+
+    #[test]
+    fn test_build_toggle_layer_visibility_script_hidden() {
+        let js = build_toggle_layer_visibility_script("map", "test-layer", false);
+        assert!(js.contains("setLayoutProperty"));
+        assert!(js.contains("'visibility', 'none'"));
+    }
+
+    #[test]
+    fn test_build_remove_layer_script() {
+        let js = build_remove_layer_script("map", "test-layer");
+        assert!(js.contains("removeLayer"));
+        assert!(js.contains("removeSource"));
+        assert!(js.contains("test-layer"));
+        assert!(js.contains("window['map_map']"));
+    }
+
+    #[test]
+    fn test_layer_script_escaping() {
+        use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};
+
+        // Test layer with special characters in ID
+        let layer = GeoJsonLayer {
+            id: "layer-with-dash".to_string(),
+            name: "Test Layer".to_string(),
+            url: "/test.geojson".to_string(),
+            layer_type: LayerType::Point,
+            visible: true,
+            color: "#ff0000".to_string(),
+        };
+
+        let js = build_add_geojson_layer_script("map", &layer);
+        assert!(js.contains("layer-with-dash"));
+    }
 }
diff --git a/crates/iou-frontend/src/components/mod.rs b/crates/iou-frontend/src/components/mod.rs
index a136808..8d1cd43 100644
--- a/crates/iou-frontend/src/components/mod.rs
+++ b/crates/iou-frontend/src/components/mod.rs
@@ -7,6 +7,7 @@ mod document_card;
 mod header;
 mod knowledge_graph;
 mod layer_control_3d;
+mod layer_detection;
 mod loading;
 mod map_3d;
 mod panel;
@@ -29,6 +30,7 @@ pub use timeline::{Timeline, TimelineEvent, TimelineEventType};
 // Re-export config and layer types
 pub use map_3d::{Map3DConfig, ConfigError};
 pub use layer_control_3d::{GeoJsonLayer, LayerType, GeoJsonLayerBuilder, predefined_layers};
+pub use layer_detection::{detect_layer_type, has_mixed_geometries};
 pub use terrain_encoding::{elevation_to_terrain_rgb, terrain_rgb_to_elevation};
 
 #[cfg(test)]
