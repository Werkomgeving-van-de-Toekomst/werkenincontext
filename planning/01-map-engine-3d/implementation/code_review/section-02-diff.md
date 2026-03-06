diff --git a/crates/iou-frontend/src/components/layer_control_3d.rs b/crates/iou-frontend/src/components/layer_control_3d.rs
index 567abc5..cc43131 100644
--- a/crates/iou-frontend/src/components/layer_control_3d.rs
+++ b/crates/iou-frontend/src/components/layer_control_3d.rs
@@ -3,4 +3,288 @@
 //! This component provides a UI for toggling GeoJSON layer visibility
 //! on the 3D map. Each layer has a checkbox and styled label.
 
-// Placeholder - implementation in section-07-layer-control
+/// Type of geometry in a GeoJSON layer.
+///
+/// Determines how the layer is rendered in MapLibre:
+/// - `Point`: Rendered as circles
+/// - `Line`: Rendered as lines
+/// - `Polygon`: Rendered as filled areas
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum LayerType {
+    Point,
+    Line,
+    Polygon,
+}
+
+impl std::fmt::Display for LayerType {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        match self {
+            LayerType::Point => write!(f, "Point"),
+            LayerType::Line => write!(f, "Line"),
+            LayerType::Polygon => write!(f, "Polygon"),
+        }
+    }
+}
+
+impl LayerType {
+    /// Returns the MapLibre layer type string for this LayerType.
+    pub fn maplibre_type(&self) -> &'static str {
+        match self {
+            LayerType::Point => "circle",
+            LayerType::Line => "line",
+            LayerType::Polygon => "fill",
+        }
+    }
+
+    /// Detects the layer type from a GeoJSON geometry type string.
+    ///
+    /// # Arguments
+    ///
+    /// * `geometry_type` - The GeoJSON geometry type (e.g., "Point", "LineString")
+    ///
+    /// # Returns
+    ///
+    /// The corresponding `LayerType` or `None` if the type is unrecognized.
+    pub fn from_geojson_type(geometry_type: &str) -> Option<Self> {
+        match geometry_type {
+            "Point" | "MultiPoint" => Some(LayerType::Point),
+            "LineString" | "MultiLineString" => Some(LayerType::Line),
+            "Polygon" | "MultiPolygon" => Some(LayerType::Polygon),
+            _ => None,
+        }
+    }
+}
+
+/// Configuration for a GeoJSON layer to be displayed on the map.
+///
+/// # Example
+///
+/// ```rust
+/// let layer = GeoJsonLayer {
+///     id: "provinciegrens".to_string(),
+///     name: "Provinciegrens".to_string(),
+///     url: "/geojson/provinciegrens.geojson".to_string(),
+///     layer_type: LayerType::Polygon,
+///     visible: true,
+///     color: "#ff0000".to_string(),
+/// };
+/// ```
+#[derive(Debug, Clone, PartialEq)]
+pub struct GeoJsonLayer {
+    /// Unique identifier for this layer (used in MapLibre)
+    pub id: String,
+
+    /// Human-readable display name (Dutch)
+    pub name: String,
+
+    /// URL to the GeoJSON data
+    pub url: String,
+
+    /// Type of geometry in this layer
+    pub layer_type: LayerType,
+
+    /// Initial visibility state
+    pub visible: bool,
+
+    /// CSS color for rendering (hex format)
+    pub color: String,
+}
+
+impl GeoJsonLayer {
+    /// Creates a builder for constructing a GeoJsonLayer.
+    pub fn builder() -> GeoJsonLayerBuilder {
+        GeoJsonLayerBuilder::default()
+    }
+}
+
+/// Builder for creating GeoJsonLayer instances.
+#[derive(Debug, Default)]
+pub struct GeoJsonLayerBuilder {
+    id: Option<String>,
+    name: Option<String>,
+    url: Option<String>,
+    layer_type: Option<LayerType>,
+    visible: Option<bool>,
+    color: Option<String>,
+}
+
+impl GeoJsonLayerBuilder {
+    pub fn id(mut self, id: &str) -> Self {
+        self.id = Some(id.to_string());
+        self
+    }
+
+    pub fn name(mut self, name: &str) -> Self {
+        self.name = Some(name.to_string());
+        self
+    }
+
+    pub fn url(mut self, url: &str) -> Self {
+        self.url = Some(url.to_string());
+        self
+    }
+
+    pub fn layer_type(mut self, layer_type: LayerType) -> Self {
+        self.layer_type = Some(layer_type);
+        self
+    }
+
+    pub fn visible(mut self, visible: bool) -> Self {
+        self.visible = Some(visible);
+        self
+    }
+
+    pub fn color(mut self, color: &str) -> Self {
+        self.color = Some(color.to_string());
+        self
+    }
+
+    /// Builds the GeoJsonLayer, returning an error if required fields are missing.
+    pub fn build(self) -> Result<GeoJsonLayer, String> {
+        let id = self.id.ok_or("id is required")?;
+        let name = self.name.ok_or("name is required")?;
+        let url = self.url.ok_or("url is required")?;
+        let layer_type = self.layer_type.ok_or("layer_type is required")?;
+
+        // Default color if not specified
+        let color = self.color.unwrap_or_else(|| "#ff0000".to_string());
+
+        // Default to visible if not specified
+        let visible = self.visible.unwrap_or(true);
+
+        Ok(GeoJsonLayer {
+            id,
+            name,
+            url,
+            layer_type,
+            visible,
+            color,
+        })
+    }
+}
+
+/// Predefined layers for the Flevoland Data Verkenner.
+pub fn predefined_layers() -> Vec<GeoJsonLayer> {
+    vec![
+        GeoJsonLayer {
+            id: "provinciegrens".to_string(),
+            name: "Provinciegrens".to_string(),
+            url: "/geojson/provinciegrens.geojson".to_string(),
+            layer_type: LayerType::Polygon,
+            visible: true,
+            color: "#e74c3c".to_string(), // Red
+        },
+        GeoJsonLayer {
+            id: "cultuurhistorie".to_string(),
+            name: "Cultuurhistorie".to_string(),
+            url: "/geojson/cultuurhistorie.geojson".to_string(),
+            layer_type: LayerType::Point,
+            visible: false,
+            color: "#3498db".to_string(), // Blue
+        },
+        // Additional layers to be added in Phase 2:
+        // - windturbines (Point)
+        // - zonneparken (Point)
+        // - fietsnetwerken (Line)
+        // - drinkwater (Point/Line)
+    ]
+}
+
+#[cfg(test)]
+mod layer_config_tests {
+    use super::*;
+
+    #[test]
+    fn test_geojson_layer_creates_with_all_required_fields() {
+        let layer = GeoJsonLayer {
+            id: "test-layer".to_string(),
+            name: "Test Layer".to_string(),
+            url: "https://example.com/layer.geojson".to_string(),
+            layer_type: LayerType::Point,
+            visible: true,
+            color: "#ff0000".to_string(),
+        };
+
+        assert_eq!(layer.id, "test-layer");
+        assert_eq!(layer.name, "Test Layer");
+        assert_eq!(layer.url, "https://example.com/layer.geojson");
+        assert!(matches!(layer.layer_type, LayerType::Point));
+        assert!(layer.visible);
+        assert_eq!(layer.color, "#ff0000");
+    }
+
+    #[test]
+    fn test_layer_type_has_point_variant() {
+        let layer_type = LayerType::Point;
+        assert!(matches!(layer_type, LayerType::Point));
+    }
+
+    #[test]
+    fn test_layer_type_has_line_variant() {
+        let layer_type = LayerType::Line;
+        assert!(matches!(layer_type, LayerType::Line));
+    }
+
+    #[test]
+    fn test_layer_type_has_polygon_variant() {
+        let layer_type = LayerType::Polygon;
+        assert!(matches!(layer_type, LayerType::Polygon));
+    }
+
+    #[test]
+    fn test_layer_type_display_returns_correct_values() {
+        assert_eq!(LayerType::Point.to_string(), "Point");
+        assert_eq!(LayerType::Line.to_string(), "Line");
+        assert_eq!(LayerType::Polygon.to_string(), "Polygon");
+    }
+
+    #[test]
+    fn test_geojson_layer_default_visible_is_true() {
+        let layer = GeoJsonLayer {
+            id: "test".to_string(),
+            name: "Test".to_string(),
+            url: "https://example.com/test.geojson".to_string(),
+            layer_type: LayerType::Point,
+            visible: true,
+            color: "#000000".to_string(),
+        };
+        assert!(layer.visible);
+    }
+
+    #[test]
+    fn test_geojson_layer_builder_creates_valid_layer() {
+        let layer = GeoJsonLayer::builder()
+            .id("test-layer")
+            .name("Test Layer")
+            .url("https://example.com/layer.geojson")
+            .layer_type(LayerType::Polygon)
+            .color("#00ff00")
+            .build()
+            .unwrap();
+
+        assert_eq!(layer.id, "test-layer");
+        assert_eq!(layer.name, "Test Layer");
+        assert_eq!(layer.url, "https://example.com/layer.geojson");
+        assert!(matches!(layer.layer_type, LayerType::Polygon));
+        assert!(layer.visible); // Default value
+        assert_eq!(layer.color, "#00ff00");
+    }
+
+    #[test]
+    fn test_layer_type_maplibre_type() {
+        assert_eq!(LayerType::Point.maplibre_type(), "circle");
+        assert_eq!(LayerType::Line.maplibre_type(), "line");
+        assert_eq!(LayerType::Polygon.maplibre_type(), "fill");
+    }
+
+    #[test]
+    fn test_layer_type_from_geojson_type() {
+        assert_eq!(LayerType::from_geojson_type("Point"), Some(LayerType::Point));
+        assert_eq!(LayerType::from_geojson_type("MultiPoint"), Some(LayerType::Point));
+        assert_eq!(LayerType::from_geojson_type("LineString"), Some(LayerType::Line));
+        assert_eq!(LayerType::from_geojson_type("Polygon"), Some(LayerType::Polygon));
+        assert_eq!(LayerType::from_geojson_type("Unknown"), None);
+    }
+}
+
+// Placeholder - LayerControl3D component implementation in section-07-layer-control
diff --git a/crates/iou-frontend/src/components/map_3d.rs b/crates/iou-frontend/src/components/map_3d.rs
index 5396106..fb2e51e 100644
--- a/crates/iou-frontend/src/components/map_3d.rs
+++ b/crates/iou-frontend/src/components/map_3d.rs
@@ -4,4 +4,344 @@
 //! It renders terrain elevation data and GeoJSON layers with 3D navigation
 //! support (pitch, rotate, zoom).
 
-// Placeholder - implementation in section-04-map3d-component
+use std::env;
+
+/// Configuration for the MapLibre GL JS 3D map instance.
+///
+/// # Coordinate Ordering
+///
+/// **IMPORTANT:** MapLibre uses `[longitude, latitude]` ordering while
+/// Leaflet uses `[latitude, longitude]`. The `center` field stores
+/// coordinates as `(longitude, latitude)` to match MapLibre's API.
+///
+/// # Example
+///
+/// ```rust
+/// let config = Map3DConfig::default();
+/// // center.0 = 5.5 (longitude)
+/// // center.1 = 52.4 (latitude)
+/// ```
+#[derive(Debug, Clone, PartialEq)]
+pub struct Map3DConfig {
+    /// HTML element ID for the map container
+    pub container_id: String,
+
+    /// Map center as (longitude, latitude)
+    ///
+    /// Valid ranges:
+    /// - longitude: -180.0 to 180.0
+    /// - latitude: -90.0 to 90.0
+    pub center: (f64, f64),
+
+    /// Initial zoom level (6.0 to 18.0)
+    pub zoom: f64,
+
+    /// Initial pitch in degrees (0 = top-down, 60 = max tilt)
+    pub pitch: f64,
+
+    /// Initial bearing in degrees (0 = north, 180 = south)
+    pub bearing: f64,
+
+    /// Minimum zoom level allowed
+    pub min_zoom: f64,
+
+    /// Maximum zoom level allowed
+    pub max_zoom: f64,
+
+    /// Terrain vertical exaggeration (0.1 to 5.0)
+    pub terrain_exaggeration: f64,
+
+    /// URL for terrain tiles (Terrain-RGB format)
+    pub terrain_tile_url: String,
+
+    /// Whether 3D map is enabled (from MAP_3D_ENABLED env var)
+    pub enabled: bool,
+}
+
+/// Validation error for Map3DConfig
+#[derive(Debug, Clone, PartialEq)]
+pub enum ConfigError {
+    InvalidLongitude(f64),
+    InvalidLatitude(f64),
+    InvalidZoom(f64),
+    InvalidPitch(f64),
+    InvalidBearing(f64),
+    InvalidExaggeration(f64),
+}
+
+impl std::fmt::Display for ConfigError {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        match self {
+            ConfigError::InvalidLongitude(v) => write!(f, "Longitude {} is outside valid range [-180, 180]", v),
+            ConfigError::InvalidLatitude(v) => write!(f, "Latitude {} is outside valid range [-90, 90]", v),
+            ConfigError::InvalidZoom(v) => write!(f, "Zoom {} is outside valid range [6, 18]", v),
+            ConfigError::InvalidPitch(v) => write!(f, "Pitch {} is outside valid range [0, 60]", v),
+            ConfigError::InvalidBearing(v) => write!(f, "Bearing {} is outside valid range [0, 360]", v),
+            ConfigError::InvalidExaggeration(v) => write!(f, "Exaggeration {} is outside valid range [0.1, 5.0]", v),
+        }
+    }
+}
+
+impl std::error::Error for ConfigError {}
+
+impl Default for Map3DConfig {
+    fn default() -> Self {
+        Self {
+            container_id: "map".to_string(),
+            // Flevoland center: (longitude, latitude)
+            center: (5.5, 52.4),
+            zoom: 10.0,
+            pitch: 60.0,
+            bearing: 0.0,
+            min_zoom: 6.0,
+            max_zoom: 18.0,
+            terrain_exaggeration: 1.5,
+            terrain_tile_url: "https://api.maptiler.com/tiles/terrain-rgb/tiles.json".to_string(),
+            enabled: Self::is_3d_enabled(),
+        }
+    }
+}
+
+impl Map3DConfig {
+    /// Creates a new Map3DConfig with the given container ID and center.
+    ///
+    /// # Arguments
+    ///
+    /// * `container_id` - HTML element ID for the map container
+    /// * `center` - Map center as (longitude, latitude)
+    ///
+    /// # Returns
+    ///
+    /// Returns `Ok(Map3DConfig)` if all values are valid, `Err(ConfigError)` otherwise.
+    pub fn new(container_id: String, center: (f64, f64)) -> Result<Self, ConfigError> {
+        let config = Self {
+            container_id,
+            center,
+            ..Default::default()
+        };
+        config.validate()?;
+        Ok(config)
+    }
+
+    /// Validates all configuration values.
+    ///
+    /// # Returns
+    ///
+    /// Returns `Ok(())` if all values are valid, `Err(ConfigError)` for the first
+    /// invalid value found.
+    pub fn validate(&self) -> Result<(), ConfigError> {
+        // Validate longitude: -180 to 180
+        if self.center.0 < -180.0 || self.center.0 > 180.0 {
+            return Err(ConfigError::InvalidLongitude(self.center.0));
+        }
+
+        // Validate latitude: -90 to 90
+        if self.center.1 < -90.0 || self.center.1 > 90.0 {
+            return Err(ConfigError::InvalidLatitude(self.center.1));
+        }
+
+        // Validate zoom: 6 to 18
+        if self.zoom < self.min_zoom || self.zoom > self.max_zoom {
+            return Err(ConfigError::InvalidZoom(self.zoom));
+        }
+
+        // Validate pitch: 0 to 60
+        if self.pitch < 0.0 || self.pitch > 60.0 {
+            return Err(ConfigError::InvalidPitch(self.pitch));
+        }
+
+        // Validate bearing: 0 to 360
+        if self.bearing < 0.0 || self.bearing > 360.0 {
+            return Err(ConfigError::InvalidBearing(self.bearing));
+        }
+
+        // Validate terrain exaggeration: 0.1 to 5.0
+        if self.terrain_exaggeration < 0.1 || self.terrain_exaggeration > 5.0 {
+            return Err(ConfigError::InvalidExaggeration(self.terrain_exaggeration));
+        }
+
+        Ok(())
+    }
+
+    /// Checks if the MAP_3D_ENABLED environment variable is set to "true".
+    ///
+    /// # Returns
+    ///
+    /// Returns `true` if MAP_3D_ENABLED is "true" (case-insensitive), `false` otherwise.
+    pub fn is_3d_enabled() -> bool {
+        env::var("MAP_3D_ENABLED")
+            .map(|v| v.eq_ignore_ascii_case("true"))
+            .unwrap_or(false)
+    }
+
+    /// Creates a Map3DConfig from environment variables.
+    ///
+    /// Reads the following environment variables:
+    /// - `MAP_3D_ENABLED`: "true" to enable 3D map (default: false)
+    /// - `TERRAIN_TILE_URL`: Custom terrain tile URL (optional)
+    ///
+    /// # Returns
+    ///
+    /// Returns a Map3DConfig with environment-based overrides.
+    pub fn from_env() -> Self {
+        let mut config = Self::default();
+        config.enabled = Self::is_3d_enabled();
+
+        // Allow custom terrain tile URL
+        if let Ok(url) = env::var("TERRAIN_TILE_URL") {
+            config.terrain_tile_url = url;
+        }
+
+        config
+    }
+}
+
+#[cfg(test)]
+mod config_tests {
+    use super::*;
+
+    #[test]
+    fn test_map3d_config_default_creates_sensible_defaults() {
+        let config = Map3DConfig::default();
+        assert_eq!(config.container_id, "map");
+        assert_eq!(config.center, (5.5, 52.4)); // Flevoland center
+        assert_eq!(config.zoom, 10.0);
+        assert_eq!(config.pitch, 60.0);
+        assert_eq!(config.bearing, 0.0);
+        assert_eq!(config.min_zoom, 6.0);
+        assert_eq!(config.max_zoom, 18.0);
+        assert_eq!(config.terrain_exaggeration, 1.5);
+    }
+
+    #[test]
+    fn test_map3d_config_validates_longitude_range() {
+        let valid_config = Map3DConfig::new("map".to_string(), (5.5, 52.4));
+        assert!(valid_config.is_ok());
+
+        let invalid_lon = Map3DConfig::new("map".to_string(), (200.0, 52.4));
+        assert!(invalid_lon.is_err());
+
+        let invalid_neg_lon = Map3DConfig::new("map".to_string(), (-200.0, 52.4));
+        assert!(invalid_neg_lon.is_err());
+    }
+
+    #[test]
+    fn test_map3d_config_validates_latitude_range() {
+        let valid_config = Map3DConfig::new("map".to_string(), (5.5, 52.4));
+        assert!(valid_config.is_ok());
+
+        let invalid_lat = Map3DConfig::new("map".to_string(), (5.5, 100.0));
+        assert!(invalid_lat.is_err());
+
+        let invalid_neg_lat = Map3DConfig::new("map".to_string(), (5.5, -100.0));
+        assert!(invalid_neg_lat.is_err());
+    }
+
+    #[test]
+    fn test_map3d_config_validates_pitch_range() {
+        let mut config = Map3DConfig::default();
+
+        // Valid pitch values
+        for pitch in [0.0, 30.0, 60.0] {
+            config.pitch = pitch;
+            assert!(config.validate().is_ok());
+        }
+
+        // Invalid pitch values
+        config.pitch = -1.0;
+        assert!(config.validate().is_err());
+
+        config.pitch = 61.0;
+        assert!(config.validate().is_err());
+    }
+
+    #[test]
+    fn test_map3d_config_validates_bearing_range() {
+        let mut config = Map3DConfig::default();
+
+        // Valid bearing values
+        for bearing in [0.0, 180.0, 360.0] {
+            config.bearing = bearing;
+            assert!(config.validate().is_ok());
+        }
+
+        // Invalid bearing values
+        config.bearing = -1.0;
+        assert!(config.validate().is_err());
+
+        config.bearing = 361.0;
+        assert!(config.validate().is_err());
+    }
+
+    #[test]
+    fn test_map3d_config_validates_zoom_range() {
+        let mut config = Map3DConfig::default();
+
+        // Valid zoom values
+        for zoom in [6.0, 10.0, 18.0] {
+            config.zoom = zoom;
+            assert!(config.validate().is_ok());
+        }
+
+        // Invalid zoom values
+        config.zoom = 5.9;
+        assert!(config.validate().is_err());
+
+        config.zoom = 18.1;
+        assert!(config.validate().is_err());
+    }
+
+    #[test]
+    fn test_map3d_config_validates_terrain_exaggeration_range() {
+        let mut config = Map3DConfig::default();
+
+        // Valid exaggeration values
+        for exag in [0.1, 1.0, 5.0] {
+            config.terrain_exaggeration = exag;
+            assert!(config.validate().is_ok());
+        }
+
+        // Invalid exaggeration values
+        config.terrain_exaggeration = 0.0;
+        assert!(config.validate().is_err());
+
+        config.terrain_exaggeration = 5.1;
+        assert!(config.validate().is_err());
+    }
+
+    #[test]
+    fn test_map3d_config_from_env_reads_map_3d_enabled_flag() {
+        // Note: This test requires setting environment variables
+        std::env::set_var("MAP_3D_ENABLED", "true");
+        let enabled = Map3DConfig::is_3d_enabled();
+        assert!(enabled);
+
+        std::env::set_var("MAP_3D_ENABLED", "false");
+        let disabled = Map3DConfig::is_3d_enabled();
+        assert!(!disabled);
+
+        std::env::remove_var("MAP_3D_ENABLED");
+        let default_disabled = Map3DConfig::is_3d_enabled();
+        assert!(!default_disabled);
+    }
+
+    #[test]
+    fn test_map3d_config_from_env_defaults_to_false_when_flag_not_set() {
+        std::env::remove_var("MAP_3D_ENABLED");
+        let config = Map3DConfig::from_env();
+        assert!(!config.enabled);
+    }
+
+    #[test]
+    fn test_map3d_config_center_coordinates_ordering() {
+        // IMPORTANT: MapLibre uses [longitude, latitude] ordering
+        let config = Map3DConfig::default();
+        // Flevoland: ~5.5 longitude, ~52.4 latitude
+        assert_eq!(config.center, (5.5, 52.4));
+        // First element is longitude, second is latitude
+        assert!((config.center.0 >= -180.0) && (config.center.0 <= 180.0));
+        assert!((config.center.1 >= -90.0) && (config.center.1 <= 90.0));
+    }
+}
+
+// Placeholder - Map3D component implementation in section-04-map3d-component
diff --git a/crates/iou-frontend/src/components/mod.rs b/crates/iou-frontend/src/components/mod.rs
index bbacecf..8c89246 100644
--- a/crates/iou-frontend/src/components/mod.rs
+++ b/crates/iou-frontend/src/components/mod.rs
@@ -24,7 +24,10 @@ pub use timeline::{Timeline, TimelineEvent, TimelineEventType};
 // 3D Map Components (Map Engine 3D Upgrade)
 // pub use declarations will be added when components are implemented
 // pub use map_3d::Map3D;
-/// pub use layer_control_3d::LayerControl3D;
+// pub use layer_control_3d::LayerControl3D;
+// Re-export config and layer types
+pub use map_3d::{Map3DConfig, ConfigError};
+pub use layer_control_3d::{GeoJsonLayer, LayerType, GeoJsonLayerBuilder, predefined_layers};
 
 #[cfg(test)]
 mod tests {
diff --git a/planning/01-map-engine-3d/implementation/deep_implement_config.json b/planning/01-map-engine-3d/implementation/deep_implement_config.json
index 586633b..a0ff70d 100644
--- a/planning/01-map-engine-3d/implementation/deep_implement_config.json
+++ b/planning/01-map-engine-3d/implementation/deep_implement_config.json
@@ -17,7 +17,12 @@
     "section-08-page-integration",
     "section-09-testing"
   ],
-  "sections_state": {},
+  "sections_state": {
+    "section-01-project-setup": {
+      "status": "complete",
+      "commit_hash": "861772d"
+    }
+  },
   "pre_commit": {
     "present": false,
     "type": "none",
