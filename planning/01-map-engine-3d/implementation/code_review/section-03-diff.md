# Section 03: Terrain Encoding - Diff

```diff
diff --git a/crates/iou-frontend/Cargo.toml b/crates/iou-frontend/Cargo.toml
index 571fbac..f43a3d3 100644
--- a/crates/iou-frontend/Cargo.toml
+++ b/crates/iou-frontend/Cargo.toml
@@ -31,6 +31,9 @@ reqwest = { version = "0.12", default-features = false, features = ["json"], opt
 # Error handling
 thiserror.workspace = true

+[dev-dependencies]
+approx = "0.5"
+
 [features]
 default = ["web"]
 web = ["dioxus/web"]
diff --git a/crates/iou-frontend/src/components/mod.rs b/crates/iou-frontend/src/components/mod.rs
index 8c89246..a136808 100644
--- a/crates/iou-frontend/src/components/mod.rs
+++ b/crates/iou-frontend/src/components/mod.rs
@@ -10,6 +10,7 @@ mod layer_control_3d;
 mod loading;
 mod map_3d;
 mod panel;
+mod terrain_encoding;
 mod timeline;

 pub use approval_actions::ApprovalActions;
@@ -28,6 +29,7 @@ pub use timeline::{Timeline, TimelineEvent, TimelineEventType};
 // Re-export config and layer types
 pub use map_3d::{Map3DConfig, ConfigError};
 pub use layer_control_3d::{GeoJsonLayer, LayerType, GeoJsonLayerBuilder, predefined_layers};
+pub use terrain_encoding::{elevation_to_terrain_rgb, terrain_rgb_to_elevation};

 #[cfg(test)]
 mod tests {
diff --git a/crates/iou-frontend/src/components/terrain_encoding.rs b/crates/iou-frontend/src/components/terrain_encoding.rs
new file mode 100644
index 0000000..e4765d9
--- /dev/null
+++ b/crates/iou-frontend/src/components/terrain_encoding.rs
@@ -0,0 +1,220 @@
+//! Terrain-RGB encoding for MapLibre GL JS terrain tiles.
+//!
+//! This module implements the Mapbox Terrain-RGB specification for encoding
+//! elevation values into RGB pixel data. MapLibre uses this format to render
+//! 3D terrain from raster elevation tiles.
+//!
+//! # Encoding Formula
+//!
+//! The standard encoding formula is:
+//! ```text
+//! elevation_meters = -10000 + ((R * 65536 + G * 256 + B) * 0.1)
+//! ```
+//!
+//! This provides:
+//! - Range: -10,000m to approximately +6,553m
+//! - Precision: 0.1m (10cm)
+
+/// Encodes an elevation value in meters to Terrain-RGB color components.
+///
+/// # Arguments
+/// * `elevation_meters` - Elevation in meters (can be negative)
+///
+/// # Returns
+/// A tuple of (R, G, B) values, each in the range 0-255
+///
+/// # Encoding
+/// The elevation is normalized to a 24-bit unsigned integer:
+/// - Add 10,000 to shift minimum to 0
+/// - Multiply by 10 for 0.1m precision
+/// - Extract R, G, B from high, mid, low bytes
+///
+/// # Example
+/// ```rust
+/// use iou_frontend::components::terrain_encoding::elevation_to_terrain_rgb;
+///
+/// // Sea level encodes to a specific RGB value
+/// let (r, g, b) = elevation_to_terrain_rgb(0.0);
+/// ```
+pub fn elevation_to_terrain_rgb(elevation_meters: f64) -> (u8, u8, u8) {
+    // Shift and scale: -10000m becomes 0
+    let normalized = (elevation_meters + 10000.0) * 10.0;
+
+    // Clamp to valid range [0, 2^24 - 1]
+    let clamped = normalized.clamp(0.0, 16777215.0);
+
+    // Convert to integer
+    let encoded = clamped as u32;
+
+    // Extract RGB components
+    let r = ((encoded >> 16) & 0xFF) as u8;
+    let g = ((encoded >> 8) & 0xFF) as u8;
+    let b = (encoded & 0xFF) as u8;
+
+    (r, g, b)
+}
+
+/// Decodes Terrain-RGB color components back to elevation in meters.
+///
+/// # Arguments
+/// * `r` - Red channel value (0-255)
+/// * `g` - Green channel value (0-255)
+/// * `b` - Blue channel value (0-255)
+///
+/// # Returns
+/// Elevation in meters (can be negative)
+///
+/// # Example
+/// ```rust
+/// use iou_frontend::components::terrain_encoding::terrain_rgb_to_elevation;
+///
+/// let elevation = terrain_rgb_to_elevation(129, 48, 160);
+/// ```
+pub fn terrain_rgb_to_elevation(r: u8, g: u8, b: u8) -> f64 {
+    // Combine RGB into 24-bit value
+    let encoded = (r as u32) * 65536 + (g as u32) * 256 + (b as u32);
+
+    // Reverse the encoding: scale and shift back
+    let elevation = (encoded as f64 * 0.1) - 10000.0;
+
+    elevation
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use approx::assert_relative_eq;
+
+    #[test]
+    fn test_elevation_to_terrain_rgb_minimum() {
+        // -10000m should encode to (0, 0, 0)
+        let (r, g, b) = elevation_to_terrain_rgb(-10000.0);
+        assert_eq!((r, g, b), (0, 0, 0));
+    }
+
+    #[test]
+    fn test_elevation_to_terrain_rgb_sea_level() {
+        // 0m (sea level) encoding
+        let (r, g, b) = elevation_to_terrain_rgb(0.0);
+        // 0 = -10000 + (encoded * 0.1)
+        // encoded = 100000
+        // 100000 = 65536 * 1 + 34464
+        // 34464 = 256 * 134 + 160
+        let encoded: u32 = 100000;
+        let expected = ((encoded >> 16) & 0xFF, (encoded >> 8) & 0xFF, encoded & 0xFF);
+        assert_eq!((r, g, b), (expected.0 as u8, expected.1 as u8, expected.2 as u8));
+    }
+
+    #[test]
+    fn test_elevation_to_terrain_rgb_flevoland_negative() {
+        // Test typical Flevoland negative elevation
+        let (r, g, b) = elevation_to_terrain_rgb(-5.5);
+        let decoded = terrain_rgb_to_elevation(r, g, b);
+        assert_relative_eq!(decoded, -5.5, epsilon = 0.1);
+    }
+
+    #[test]
+    fn test_elevation_to_terrain_rgb_flevoland_positive() {
+        // Test typical Flevoland positive elevation
+        let (r, g, b) = elevation_to_terrain_rgb(3.2);
+        let decoded = terrain_rgb_to_elevation(r, g, b);
+        assert_relative_eq!(decoded, 3.2, epsilon = 0.1);
+    }
+
+    #[test]
+    fn test_encoding_round_trip() {
+        // Round-trip: encode then decode should return original
+        let original = -10000.0;
+        let (r, g, b) = elevation_to_terrain_rgb(original);
+        let decoded = terrain_rgb_to_elevation(r, g, b);
+        assert_relative_eq!(original, decoded, epsilon = 0.1);
+    }
+
+    #[test]
+    fn test_encoding_round_trip_sea_level() {
+        let original = 0.0;
+        let (r, g, b) = elevation_to_terrain_rgb(original);
+        let decoded = terrain_rgb_to_elevation(r, g, b);
+        assert_relative_eq!(original, decoded, epsilon = 0.1);
+    }
+
+    #[test]
+    fn test_encoding_round_trip_positive() {
+        let original = 1234.5;
+        let (r, g, b) = elevation_to_terrain_rgb(original);
+        let decoded = terrain_rgb_to_elevation(r, g, b);
+        assert_relative_eq!(original, decoded, epsilon = 0.1);
+    }
+
+    #[test]
+    fn test_fractional_elevation_rounding() {
+        // Test that fractional values are handled correctly
+        // The encoding has 0.1m precision, so some rounding is expected
+        let original = 1.23;
+        let (r, g, b) = elevation_to_terrain_rgb(original);
+        let decoded = terrain_rgb_to_elevation(r, g, b);
+        assert_relative_eq!(original, decoded, epsilon = 0.1);
+    }
+
+    #[test]
+    fn test_terrain_rgb_to_elevation_max() {
+        // Maximum encodable value: (255, 255, 255)
+        // elevation = -10000 + ((255*65536 + 255*256 + 255) * 0.1)
+        let expected = -10000.0 + ((255 * 65536 + 255 * 256 + 255) as f64 * 0.1);
+        let actual = terrain_rgb_to_elevation(255, 255, 255);
+        assert_relative_eq!(expected, actual);
+    }
+
+    #[test]
+    fn test_elevation_clamping_negative() {
+        // Values below -10000 should be clamped to 0,0,0
+        let (r, g, b) = elevation_to_terrain_rgb(-15000.0);
+        assert_eq!((r, g, b), (0, 0, 0));
+    }
+
+    #[test]
+    fn test_elevation_clamping_positive() {
+        // Values above maximum should be clamped to 255,255,255
+        // Maximum encodable: -10000 + (16777215 * 0.1) ≈ 1,667,721m
+        let (r, g, b) = elevation_to_terrain_rgb(2000000.0);
+        assert_eq!((r, g, b), (255, 255, 255));
+    }
+
+    #[test]
+    fn test_flevoland_range_negative() {
+        // Flevoland minimum elevation: approximately -6.7m
+        let test_elevations = [-6.7, -6.0, -5.0, -3.5];
+        for elevation in test_elevations {
+            let (r, g, b) = elevation_to_terrain_rgb(elevation);
+            let decoded = terrain_rgb_to_elevation(r, g, b);
+            assert_relative_eq!(elevation, decoded, epsilon = 0.0);
+        }
+    }
+
+    #[test]
+    fn test_flevoland_range_positive() {
+        // Flevoland maximum elevation: approximately +5m
+        let test_elevations = [0.0, 1.0, 2.5, 4.0, 5.0];
+        for elevation in test_elevations {
+            let (r, g, b) = elevation_to_terrain_rgb(elevation);
+            let decoded = terrain_rgb_to_elevation(r, g, b);
+            assert_relative_eq!(elevation, decoded, epsilon = 0.1);
+        }
+    }
+
+    #[test]
+    fn test_sea_level_exact_value() {
+        // 0m should encode to exactly (129, 134, 160)
+        // Because: 0 = -10000 + (100000 * 0.1)
+        // 100000 = 1*65536 + 134*256 + 160
+        let (r, g, b) = elevation_to_terrain_rgb(0.0);
+        assert_eq!((r, g, b), (1, 134, 160));
+    }
+
+    #[test]
+    fn test_decode_known_rgb_values() {
+        // Test some known RGB -> elevation conversions
+        assert_eq!(terrain_rgb_to_elevation(0, 0, 0), -10000.0);
+        assert_relative_eq!(terrain_rgb_to_elevation(1, 134, 160), 0.0, epsilon = 0.1);
+    }
+}
```
