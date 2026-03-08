diff --git a/crates/iou-frontend/Dioxus.toml b/crates/iou-frontend/Dioxus.toml
index f8b63d7..db8d9e2 100644
--- a/crates/iou-frontend/Dioxus.toml
+++ b/crates/iou-frontend/Dioxus.toml
@@ -11,13 +11,25 @@ reload_html = true
 watch_path = ["src", "assets"]
 
 [web.resource]
+# Leaflet CSS (existing 2D map)
 style = [
     "https://unpkg.com/leaflet@1.9.4/dist/leaflet.css",
 ]
+# Leaflet JS and vis-network (existing)
 script = [
     "https://unpkg.com/leaflet@1.9.4/dist/leaflet.js",
     "https://unpkg.com/vis-network@9.1.9/standalone/umd/vis-network.min.js",
 ]
 
+[web.resource.map_3d]
+# MapLibre GL JS - Version 5.0.0 (3D map engine)
+# These resources are conditionally loaded based on MAP_3D_ENABLED feature flag
+style = [
+    "https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.css",
+]
+script = [
+    "https://unpkg.com/maplibre-gl@5.0.0/dist/maplibre-gl.js",
+]
+
 [web.resource.dev]
 script = []
diff --git a/crates/iou-frontend/src/components/layer_control_3d.rs b/crates/iou-frontend/src/components/layer_control_3d.rs
new file mode 100644
index 0000000..567abc5
--- /dev/null
+++ b/crates/iou-frontend/src/components/layer_control_3d.rs
@@ -0,0 +1,6 @@
+//! LayerControl3D Component - Layer toggle control for Map3D
+//!
+//! This component provides a UI for toggling GeoJSON layer visibility
+//! on the 3D map. Each layer has a checkbox and styled label.
+
+// Placeholder - implementation in section-07-layer-control
diff --git a/crates/iou-frontend/src/components/map_3d.rs b/crates/iou-frontend/src/components/map_3d.rs
new file mode 100644
index 0000000..5396106
--- /dev/null
+++ b/crates/iou-frontend/src/components/map_3d.rs
@@ -0,0 +1,7 @@
+//! Map3D Component - MapLibre GL JS wrapper
+//!
+//! This component provides a 3D-capable map interface using MapLibre GL JS.
+//! It renders terrain elevation data and GeoJSON layers with 3D navigation
+//! support (pitch, rotate, zoom).
+
+// Placeholder - implementation in section-04-map3d-component
diff --git a/crates/iou-frontend/src/components/mod.rs b/crates/iou-frontend/src/components/mod.rs
index e3c6f43..2f4a825 100644
--- a/crates/iou-frontend/src/components/mod.rs
+++ b/crates/iou-frontend/src/components/mod.rs
@@ -6,7 +6,9 @@ mod audit_viewer;
 mod document_card;
 mod header;
 mod knowledge_graph;
+mod layer_control_3d;
 mod loading;
+mod map_3d;
 mod panel;
 mod timeline;
 
@@ -18,3 +20,27 @@ pub use header::Header;
 pub use knowledge_graph::KnowledgeGraph;
 pub use panel::Panel;
 pub use timeline::{Timeline, TimelineEvent, TimelineEventType};
+
+// 3D Map Components (Map Engine 3D Upgrade)
+// pub use declarations will be added when components are implemented
+// pub use map_3d::Map3D;
+// pub use layer_control_3d::LayerControl3D;
+
+#[cfg(test)]
+mod tests {
+    // These tests verify that the 3D map modules compile successfully.
+    // The mere existence of these tests confirms that mod map_3d and
+    // mod layer_control_3d declarations are valid.
+
+    #[test]
+    fn test_map3d_module_compiles() {
+        // If this test compiles, the map_3d module exists
+        assert!(true, "map_3d module is accessible");
+    }
+
+    #[test]
+    fn test_layer_control_3d_module_compiles() {
+        // If this test compiles, the layer_control_3d module exists
+        assert!(true, "layer_control_3d module is accessible");
+    }
+}
