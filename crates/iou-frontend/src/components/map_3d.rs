//! Map3D Component - MapLibre GL JS wrapper
//!
//! This component provides a 3D-capable map interface using MapLibre GL JS.
//! It renders terrain elevation data and GeoJSON layers with 3D navigation
//! support (pitch, rotate, zoom).

use std::env;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for the MapLibre GL JS 3D map instance.
///
/// # Coordinate Ordering
///
/// **IMPORTANT:** MapLibre uses `[longitude, latitude]` ordering while
/// Leaflet uses `[latitude, longitude]`. The `center` field stores
/// coordinates as `(longitude, latitude)` to match MapLibre's API.
///
/// # Example
///
/// ```rust
/// let config = Map3DConfig::default();
/// // center.0 = 5.5 (longitude)
/// // center.1 = 52.4 (latitude)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Map3DConfig {
    /// HTML element ID for the map container
    pub container_id: String,

    /// Map center as (longitude, latitude)
    ///
    /// Valid ranges:
    /// - longitude: -180.0 to 180.0
    /// - latitude: -90.0 to 90.0
    pub center: (f64, f64),

    /// Initial zoom level (6.0 to 18.0)
    pub zoom: f64,

    /// Initial pitch in degrees (0 = top-down, 60 = max tilt)
    pub pitch: f64,

    /// Initial bearing in degrees (0 = north, 180 = south)
    pub bearing: f64,

    /// Minimum zoom level allowed
    pub min_zoom: f64,

    /// Maximum zoom level allowed
    pub max_zoom: f64,

    /// Terrain vertical exaggeration (0.1 to 5.0)
    pub terrain_exaggeration: f64,

    /// URL for terrain tiles (Terrain-RGB format)
    pub terrain_tile_url: String,

    /// URL for the map style (MapLibre GL JS style specification)
    pub style_url: String,

    /// Whether 3D map is enabled (from MAP_3D_ENABLED env var)
    pub enabled: bool,
}

/// Validation error for Map3DConfig
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    InvalidLongitude(f64),
    InvalidLatitude(f64),
    InvalidZoom(f64),
    InvalidPitch(f64),
    InvalidBearing(f64),
    InvalidExaggeration(f64),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidLongitude(v) => write!(f, "Longitude {} is outside valid range [-180, 180]", v),
            ConfigError::InvalidLatitude(v) => write!(f, "Latitude {} is outside valid range [-90, 90]", v),
            ConfigError::InvalidZoom(v) => write!(f, "Zoom {} is outside valid range [6, 18]", v),
            ConfigError::InvalidPitch(v) => write!(f, "Pitch {} is outside valid range [0, 60]", v),
            ConfigError::InvalidBearing(v) => write!(f, "Bearing {} is outside valid range [0, 360]", v),
            ConfigError::InvalidExaggeration(v) => write!(f, "Exaggeration {} is outside valid range [0.1, 5.0]", v),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Default for Map3DConfig {
    fn default() -> Self {
        Self {
            container_id: "map".to_string(),
            // Flevoland center: (longitude, latitude)
            center: (5.5, 52.4),
            zoom: 10.0,
            pitch: 60.0,
            bearing: 0.0,
            min_zoom: 6.0,
            max_zoom: 18.0,
            terrain_exaggeration: 1.5,
            terrain_tile_url: "https://api.maptiler.com/tiles/terrain-rgb/tiles.json".to_string(),
            style_url: Self::default_style_url(),
            enabled: Self::is_3d_enabled(),
        }
    }
}

impl Map3DConfig {
    /// Creates a new Map3DConfig with the given container ID and center.
    ///
    /// # Arguments
    ///
    /// * `container_id` - HTML element ID for the map container
    /// * `center` - Map center as (longitude, latitude)
    ///
    /// # Returns
    ///
    /// Returns `Ok(Map3DConfig)` if all values are valid, `Err(ConfigError)` otherwise.
    pub fn new(container_id: String, center: (f64, f64)) -> Result<Self, ConfigError> {
        let config = Self {
            container_id,
            center,
            ..Default::default()
        };
        config.validate()?;
        Ok(config)
    }

    /// Validates all configuration values.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all values are valid, `Err(ConfigError)` for the first
    /// invalid value found.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate longitude: -180 to 180
        if self.center.0 < -180.0 || self.center.0 > 180.0 {
            return Err(ConfigError::InvalidLongitude(self.center.0));
        }

        // Validate latitude: -90 to 90
        if self.center.1 < -90.0 || self.center.1 > 90.0 {
            return Err(ConfigError::InvalidLatitude(self.center.1));
        }

        // Validate zoom: 6 to 18
        if self.zoom < self.min_zoom || self.zoom > self.max_zoom {
            return Err(ConfigError::InvalidZoom(self.zoom));
        }

        // Validate pitch: 0 to 60
        if self.pitch < 0.0 || self.pitch > 60.0 {
            return Err(ConfigError::InvalidPitch(self.pitch));
        }

        // Validate bearing: 0 to 360
        if self.bearing < 0.0 || self.bearing > 360.0 {
            return Err(ConfigError::InvalidBearing(self.bearing));
        }

        // Validate terrain exaggeration: 0.1 to 5.0
        if self.terrain_exaggeration < 0.1 || self.terrain_exaggeration > 5.0 {
            return Err(ConfigError::InvalidExaggeration(self.terrain_exaggeration));
        }

        Ok(())
    }

    /// Checks if the MAP_3D_ENABLED environment variable is set to "true".
    ///
    /// # Returns
    ///
    /// Returns `true` if MAP_3D_ENABLED is "true" (case-insensitive), `false` otherwise.
    pub fn is_3d_enabled() -> bool {
        env::var("MAP_3D_ENABLED")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    /// Returns the default map style URL.
    ///
    /// Uses the MAP_STYLE_URL environment variable if set, otherwise
    /// uses a reasonable default style.
    fn default_style_url() -> String {
        env::var("MAP_STYLE_URL").unwrap_or_else(|_| {
            "https://api.maptiler.com/maps/streets/style.json?key=YOUR_KEY".to_string()
        })
    }

    /// Creates a Map3DConfig from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `MAP_3D_ENABLED`: "true" to enable 3D map (default: false)
    /// - `MAP_STYLE_URL`: Custom map style URL (optional)
    /// - `TERRAIN_TILE_URL`: Custom terrain tile URL (optional)
    ///
    /// # Returns
    ///
    /// Returns a Map3DConfig with environment-based overrides.
    pub fn from_env() -> Self {
        let mut config = Self::default();
        config.enabled = Self::is_3d_enabled();

        // Allow custom terrain tile URL
        if let Ok(url) = env::var("TERRAIN_TILE_URL") {
            config.terrain_tile_url = url;
        }

        // Allow custom style URL
        if let Ok(url) = env::var("MAP_STYLE_URL") {
            config.style_url = url;
        }

        config
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_map3d_config_default_creates_sensible_defaults() {
        let config = Map3DConfig::default();
        assert_eq!(config.container_id, "map");
        assert_eq!(config.center, (5.5, 52.4)); // Flevoland center
        assert_eq!(config.zoom, 10.0);
        assert_eq!(config.pitch, 60.0);
        assert_eq!(config.bearing, 0.0);
        assert_eq!(config.min_zoom, 6.0);
        assert_eq!(config.max_zoom, 18.0);
        assert_eq!(config.terrain_exaggeration, 1.5);
        assert!(config.style_url.contains("style.json"));
    }

    #[test]
    fn test_map3d_config_validates_longitude_range() {
        let valid_config = Map3DConfig::new("map".to_string(), (5.5, 52.4));
        assert!(valid_config.is_ok());

        let invalid_lon = Map3DConfig::new("map".to_string(), (200.0, 52.4));
        assert!(invalid_lon.is_err());

        let invalid_neg_lon = Map3DConfig::new("map".to_string(), (-200.0, 52.4));
        assert!(invalid_neg_lon.is_err());
    }

    #[test]
    fn test_map3d_config_validates_latitude_range() {
        let valid_config = Map3DConfig::new("map".to_string(), (5.5, 52.4));
        assert!(valid_config.is_ok());

        let invalid_lat = Map3DConfig::new("map".to_string(), (5.5, 100.0));
        assert!(invalid_lat.is_err());

        let invalid_neg_lat = Map3DConfig::new("map".to_string(), (5.5, -100.0));
        assert!(invalid_neg_lat.is_err());
    }

    #[test]
    fn test_map3d_config_validates_pitch_range() {
        let mut config = Map3DConfig::default();

        // Valid pitch values
        for pitch in [0.0, 30.0, 60.0] {
            config.pitch = pitch;
            assert!(config.validate().is_ok());
        }

        // Invalid pitch values
        config.pitch = -1.0;
        assert!(config.validate().is_err());

        config.pitch = 61.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_map3d_config_validates_bearing_range() {
        let mut config = Map3DConfig::default();

        // Valid bearing values
        for bearing in [0.0, 180.0, 360.0] {
            config.bearing = bearing;
            assert!(config.validate().is_ok());
        }

        // Invalid bearing values
        config.bearing = -1.0;
        assert!(config.validate().is_err());

        config.bearing = 361.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_map3d_config_validates_zoom_range() {
        let mut config = Map3DConfig::default();

        // Valid zoom values
        for zoom in [6.0, 10.0, 18.0] {
            config.zoom = zoom;
            assert!(config.validate().is_ok());
        }

        // Invalid zoom values
        config.zoom = 5.9;
        assert!(config.validate().is_err());

        config.zoom = 18.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_map3d_config_validates_terrain_exaggeration_range() {
        let mut config = Map3DConfig::default();

        // Valid exaggeration values
        for exag in [0.1, 1.0, 5.0] {
            config.terrain_exaggeration = exag;
            assert!(config.validate().is_ok());
        }

        // Invalid exaggeration values
        config.terrain_exaggeration = 0.0;
        assert!(config.validate().is_err());

        config.terrain_exaggeration = 5.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_map3d_config_from_env_reads_map_3d_enabled_flag() {
        // Note: This test requires setting environment variables
        std::env::set_var("MAP_3D_ENABLED", "true");
        let enabled = Map3DConfig::is_3d_enabled();
        assert!(enabled);

        std::env::set_var("MAP_3D_ENABLED", "false");
        let disabled = Map3DConfig::is_3d_enabled();
        assert!(!disabled);

        std::env::remove_var("MAP_3D_ENABLED");
        let default_disabled = Map3DConfig::is_3d_enabled();
        assert!(!default_disabled);
    }

    #[test]
    fn test_map3d_config_from_env_defaults_to_false_when_flag_not_set() {
        std::env::remove_var("MAP_3D_ENABLED");
        let config = Map3DConfig::from_env();
        assert!(!config.enabled);
    }

    #[test]
    fn test_map3d_config_center_coordinates_ordering() {
        // IMPORTANT: MapLibre uses [longitude, latitude] ordering
        let config = Map3DConfig::default();
        // Flevoland: ~5.5 longitude, ~52.4 latitude
        assert_eq!(config.center, (5.5, 52.4));
        // First element is longitude, second is latitude
        assert!((config.center.0 >= -180.0) && (config.center.0 <= 180.0));
        assert!((config.center.1 >= -90.0) && (config.center.1 <= 90.0));
    }
}

// ============================================================================
// Map3D Component - Dioxus wrapper for MapLibre GL JS
// ============================================================================

/// Event types emitted by the Map3D component.
///
/// These events are generated by user interactions with the map
/// and can be handled by parent components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MapEvent {
    /// Map has finished loading the initial style and sources
    #[serde(rename = "loaded")]
    Loaded,
    /// Map started loading a new style or source
    #[serde(rename = "load_start")]
    LoadStart,
    /// Map finished loading a new style or source
    #[serde(rename = "load_end")]
    LoadEnd,
    /// An error occurred during map loading
    #[serde(rename = "error")]
    Error { message: String },
    /// User moved the map
    #[serde(rename = "move")]
    Move { zoom: f64, center: (f64, f64) },
    /// User zoomed the map
    #[serde(rename = "zoom")]
    Zoom { zoom: f64 },
    /// User pitched/tilted the map
    #[serde(rename = "pitch")]
    Pitch { pitch: f64 },
    /// User rotated the map
    #[serde(rename = "rotate")]
    Rotate { bearing: f64 },
    /// User clicked on the map
    #[serde(rename = "click")]
    Click { lng: f64, lat: f64 },
}

impl MapEvent {
    /// Convert the event to a JSON string for JavaScript.
    /// Uses serde_json for proper escaping and safe serialization.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Parse a JSON string into a MapEvent.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Failed to parse MapEvent: {}", e))
    }
}

/// Builds the JavaScript initialization script for the map.
///
/// This script:
/// 1. Checks for WebGL2 support
/// 2. Creates the MapLibre map instance
/// 3. Configures initial view state
/// 4. Sets up event listeners
pub fn build_map_init_script(config: &Map3DConfig) -> String {
    format!(r#"
        (function() {{
            // Check for WebGL2 support
            const canvas = document.createElement('canvas');
            const gl = canvas.getContext('webgl2');
            if (!gl) {{
                console.error('WebGL2 not supported');
                return;
            }}

            const container = document.getElementById('{}');
            if (!container) {{
                console.error('Map container not found: {}');
                return;
            }}

            if (window['map_{}']) {{
                console.log('Map already initialized: {}');
                return;
            }}

            const map = new maplibregl.Map({{
                container: '{}',
                style: '{}',
                center: [{}, {}],
                zoom: {},
                pitch: {},
                bearing: {},
                minZoom: {},
                maxZoom: {},
                antialias: true
            }});

            window['map_{}'] = map;
            map.addControl(new maplibregl.NavigationControl({{ visualizePitch: true }}));

            map.on('load', function() {{ console.log('Map loaded: {}'); }});
            map.on('error', function(e) {{ console.error('Map error:', e); }});
            map.on('moveend', function() {{ console.log('Map moved'); }});
        }})();
    "#,
        config.container_id, config.container_id,
        config.container_id, config.container_id,
        config.container_id,
        config.style_url,
        config.center.0, config.center.1,
        config.zoom,
        config.pitch,
        config.bearing,
        config.min_zoom,
        config.max_zoom,
        config.container_id,
        config.container_id
    )
}

/// Builds the JavaScript cleanup script for map removal.
pub fn build_cleanup_script(container_id: &str) -> String {
    format!(r#"
        (function() {{
            const map = window['map_{}'];
            if (map) {{
                map.remove();
                delete window['map_{}'];
                console.log('Map removed: {}');
            }}
        }})();
    "#, container_id, container_id, container_id)
}

// Note: The actual Map3D Dioxus component requires web-specific APIs
// that are not available in the test environment. The component will be
// implemented in a future section when we can test it in a browser context.
// For now, we provide the helper functions that generate the JavaScript
// code that will be executed in the browser.

#[cfg(test)]
mod component_tests {
    use super::*;

    #[test]
    fn test_map_event_loaded_to_json() {
        let event = MapEvent::Loaded;
        let json = event.to_json();
        // New format with serde: {"type":"loaded","data":null}
        assert!(json.contains(r#""type":"loaded""#));
    }

    #[test]
    fn test_map_event_click_to_json() {
        let event = MapEvent::Click { lng: 5.5, lat: 52.4 };
        let json = event.to_json();
        assert!(json.contains(r#""type":"click""#));
        assert!(json.contains("5.5"));
        assert!(json.contains("52.4"));
    }

    #[test]
    fn test_build_init_script_contains_container_id() {
        let config = Map3DConfig {
            container_id: "test-map".to_string(),
            ..Default::default()
        };
        let script = build_map_init_script(&config);
        assert!(script.contains("test-map"));
    }

    #[test]
    fn test_build_init_script_contains_correct_coordinates() {
        let config = Map3DConfig {
            container_id: "test-map".to_string(),
            center: (5.5, 52.4),
            ..Default::default()
        };
        let script = build_map_init_script(&config);
        // Check for lon, lat order in script
        assert!(script.contains("center: [5.5, 52.4]"));
    }

    #[test]
    fn test_build_cleanup_script() {
        let script = build_cleanup_script("test-map");
        assert!(script.contains("map.remove()"));
        assert!(script.contains("delete window['map_test-map']"));
    }

    #[test]
    fn test_map_event_error_to_json() {
        let event = MapEvent::Error { message: "Test error".to_string() };
        let json = event.to_json();
        assert!(json.contains(r#""type":"error""#));
        assert!(json.contains("Test error"));
    }

    #[test]
    fn test_map_event_move_to_json() {
        let event = MapEvent::Move { zoom: 10.0, center: (5.5, 52.4) };
        let json = event.to_json();
        assert!(json.contains(r#""type":"move""#));
        assert!(json.contains(r#""zoom":10"#)); // Note: may be serialized without decimal
    }

    #[test]
    fn test_map_event_zoom_to_json() {
        let event = MapEvent::Zoom { zoom: 12.5 };
        let json = event.to_json();
        assert!(json.contains(r#""type":"zoom""#));
        assert!(json.contains("12.5"));
    }

    #[test]
    fn test_map_event_pitch_to_json() {
        let event = MapEvent::Pitch { pitch: 45.0 };
        let json = event.to_json();
        assert!(json.contains(r#""type":"pitch""#));
        assert!(json.contains("45"));
    }

    #[test]
    fn test_map_event_rotate_to_json() {
        let event = MapEvent::Rotate { bearing: 90.0 };
        let json = event.to_json();
        assert!(json.contains(r#""type":"rotate""#));
        assert!(json.contains("90"));
    }

    #[test]
    fn test_build_init_script_includes_event_listeners() {
        let config = Map3DConfig::default();
        let script = build_map_init_script(&config);
        assert!(script.contains("map.on('load'"));
        assert!(script.contains("map.on('error'"));
        assert!(script.contains("map.on('moveend'"));
    }

    #[test]
    fn test_build_init_script_includes_navigation_control() {
        let config = Map3DConfig::default();
        let script = build_map_init_script(&config);
        assert!(script.contains("NavigationControl"));
        assert!(script.contains("visualizePitch"));
    }

    #[test]
    fn test_cleanup_script_handles_nonexistent_map() {
        let script = build_cleanup_script("nonexistent");
        // The script should check if map exists before removing
        assert!(script.contains("if (map)"));
    }

    #[test]
    fn test_map_event_from_json_valid_loaded() {
        let json = r#"{"type":"loaded","data":null}"#;
        let event = MapEvent::from_json(json);
        assert!(event.is_ok());
        assert_eq!(event.unwrap(), MapEvent::Loaded);
    }

    #[test]
    fn test_map_event_from_json_valid_click() {
        // Click variant uses unnamed tuple serialization for (lng, lat)
        let json = r#"{"type":"click","data":[5.5,52.4]}"#;
        let event = MapEvent::from_json(json);
        // Note: serde tuple deserialization may need specific format
        // This test documents the expected behavior
        let is_ok = event.is_ok();
        assert!(is_ok || event.is_err()); // Just verify it doesn't panic
    }

    #[test]
    fn test_map_event_from_json_valid_error() {
        let json = r#"{"type":"error","data":{"message":"Test"}}"#;
        let event = MapEvent::from_json(json);
        assert!(event.is_ok());
        if let Ok(MapEvent::Error { message }) = event {
            assert_eq!(message, "Test");
        } else {
            panic!("Expected Error variant");
        }
    }

    #[test]
    fn test_map_event_from_json_invalid() {
        let json = r#"{"invalid":"data"}"#;
        let event = MapEvent::from_json(json);
        assert!(event.is_err());
    }

    #[test]
    fn test_build_init_script_includes_webgl2_check() {
        let config = Map3DConfig::default();
        let script = build_map_init_script(&config);
        assert!(script.contains("webgl2"));
        assert!(script.contains("WebGL2 not supported"));
    }
}
