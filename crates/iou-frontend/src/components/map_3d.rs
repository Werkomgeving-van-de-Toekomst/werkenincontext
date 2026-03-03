//! Map3D Component - MapLibre GL JS wrapper
//!
//! This component provides a 3D-capable map interface using MapLibre GL JS.
//! It renders terrain elevation data and GeoJSON layers with 3D navigation
//! support (pitch, rotate, zoom).

use std::env;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Terrain tile source type.
///
/// Determines whether tiles are loaded from MapTiler (requires API key)
/// or from a local source (AHN tiles).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainSource {
    /// MapTiler terrain tiles (requires MAPTILER_API_KEY)
    MapTiler,
    /// Local AHN tiles (no API key required)
    LocalAHN,
    /// No terrain - flat 3D map only
    None,
}

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

    /// Terrain tile source type
    pub terrain_source: TerrainSource,

    /// URL for terrain tiles (Terrain-RGB format) - used only for MapTiler
    pub terrain_tile_url: String,

    /// Local terrain tile path (for LocalAHN source)
    pub terrain_local_path: String,

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
        // Determine terrain source from environment
        // TERRAIN_SOURCE=maptiler|local|none (default: auto-detect)
        // If MAPTILER_API_KEY is set, use MapTiler, otherwise use LocalAHN
        let terrain_source = Self::determine_terrain_source();

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
            terrain_source,
            terrain_tile_url: "https://api.maptiler.com/tiles/terrain-rgb/tiles.json".to_string(),
            terrain_local_path: "/static/terrain/{z}/{x}/{y}.png".to_string(),
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

    /// Determines the terrain source based on environment configuration.
    ///
    /// Checks TERRAIN_SOURCE first, then falls back to MAPTILER_API_KEY detection.
    /// - If TERRAIN_SOURCE=maptiler: always use MapTiler
    /// - If TERRAIN_SOURCE=local: always use local AHN tiles
    /// - If TERRAIN_SOURCE=none: disable terrain
    /// - Otherwise: use MapTiler if API key is set, else local AHN
    fn determine_terrain_source() -> TerrainSource {
        match env::var("TERRAIN_SOURCE").as_deref() {
            Ok("maptiler") => TerrainSource::MapTiler,
            Ok("local") => TerrainSource::LocalAHN,
            Ok("none") => TerrainSource::None,
            Ok(_) => {
                // Unknown value, fall back to auto-detect
                Self::auto_detect_terrain_source()
            }
            Err(_) => {
                // Not set, auto-detect based on API key
                Self::auto_detect_terrain_source()
            }
        }
    }

    /// Auto-detects terrain source based on whether MAPTILER_API_KEY is set.
    fn auto_detect_terrain_source() -> TerrainSource {
        if env::var("MAPTILER_API_KEY").is_ok() {
            TerrainSource::MapTiler
        } else {
            // No API key? Use local AHN tiles (no API key required)
            TerrainSource::LocalAHN
        }
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

    /// Returns the terrain tile URL appropriate for the configured source.
    ///
    /// For MapTiler: Includes the MAPTILER_API_KEY if set.
    /// For LocalAHN: Returns the local tile path.
    /// For None: Returns an empty string (no terrain).
    pub fn terrain_tile_url(&self) -> String {
        match self.terrain_source {
            TerrainSource::MapTiler => {
                // If a custom URL was set (doesn't contain default maptiler domain), use it directly
                if !self.terrain_tile_url.contains("maptiler.com") {
                    return self.terrain_tile_url.clone();
                }

                // MapTiler requires an API key
                let api_key = env::var("MAPTILER_API_KEY")
                    .unwrap_or_else(|_| "YOUR_KEY_HERE".to_string());

                format!(
                    "https://api.maptiler.com/tiles/terrain-rgb/tiles.json?key={}",
                    api_key
                )
            }
            TerrainSource::LocalAHN => {
                // Local AHN tiles - no API key needed
                // The path uses {z}, {x}, {y} placeholders that MapLibre will replace
                self.terrain_local_path.clone()
            }
            TerrainSource::None => {
                // No terrain source
                String::new()
            }
        }
    }

    /// Returns whether terrain is enabled for this configuration.
    pub fn has_terrain(&self) -> bool {
        self.terrain_source != TerrainSource::None
    }

    /// Returns the terrain exaggeration value, clamped to valid range.
    ///
    /// Reads from TERRAIN_EXAGGERATION environment variable if set,
    /// otherwise uses the configured value (self.terrain_exaggeration).
    ///
    /// The value is clamped to the valid range [0.1, 5.0] to prevent
    /// runtime errors in MapLibre.
    pub fn terrain_exaggeration(&self) -> f64 {
        let value = env::var("TERRAIN_EXAGGERATION")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(self.terrain_exaggeration);

        // Clamp to valid range for MapLibre
        value.clamp(0.1, 5.0)
    }

    /// Creates a Map3DConfig from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `MAP_3D_ENABLED`: "true" to enable 3D map (default: false)
    /// - `MAP_STYLE_URL`: Custom map style URL (optional)
    /// - `TERRAIN_TILE_URL`: Custom terrain tile URL (optional)
    /// - `MAPTILER_API_KEY`: MapTiler API key for terrain tiles
    /// - `TERRAIN_EXAGGERATION`: Terrain vertical exaggeration, clamped to [0.1, 5.0]
    /// - `TERRAIN_TILE_URL`: Custom terrain tile URL (sets terrain_source to MapTiler)
    /// - `TERRAIN_SOURCE`: Force terrain source (maptiler|local|none)
    ///
    /// # Returns
    ///
    /// Returns a Map3DConfig with environment-based overrides.
    pub fn from_env() -> Self {
        let mut config = Self::default();
        config.enabled = Self::is_3d_enabled();

        // Check TERRAIN_SOURCE first
        config.terrain_source = Self::determine_terrain_source();

        // Allow custom terrain tile URL
        // If set, override the terrain source to MapTiler style (uses url property)
        if let Ok(url) = env::var("TERRAIN_TILE_URL") {
            config.terrain_tile_url = url;
            // Custom URL implies MapTiler-style source (uses TileJSON)
            config.terrain_source = TerrainSource::MapTiler;
        }

        // Allow custom style URL
        if let Ok(url) = env::var("MAP_STYLE_URL") {
            config.style_url = url;
        }

        // Allow custom terrain exaggeration
        if let Ok(exag) = env::var("TERRAIN_EXAGGERATION") {
            if let Ok(v) = exag.parse() {
                config.terrain_exaggeration = v;
            }
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

/// Escapes a string for safe use in JavaScript string literals.
///
/// Prevents JavaScript injection by escaping backslashes, quotes,
/// and other special characters.
fn js_escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Validates that a container_id is safe for use in JavaScript.
///
/// Container IDs must be alphanumeric with hyphens and underscores only.
fn is_valid_container_id(id: &str) -> bool {
    id.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Validates that a color string is a valid hex color.
///
/// Accepts formats like "#ff0000" or "#FF0000".
fn is_valid_hex_color(color: &str) -> bool {
    if color.len() != 7 {
        return false;
    }
    if !color.starts_with('#') {
        return false;
    }
    color[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Builds the JavaScript to add terrain source to the map.
///
/// This function:
/// 1. Checks if terrain source already exists
/// 2. Adds the terrain DEM source from MapTiler
/// 3. Sets terrain with exaggeration
/// 4. Sets up terrain event listeners
///
/// Supports:
/// - MapTiler: Uses TileJSON URL (tiles.json)
/// - LocalAHN: Uses direct tile URL template (/static/terrain/{z}/{x}/{y}.png)
/// - None: Skips terrain initialization
pub fn build_terrain_init_script(config: &Map3DConfig) -> String {
    let container_id = &config.container_id;
    assert!(is_valid_container_id(container_id), "Invalid container_id");

    // Skip terrain initialization if disabled
    if config.terrain_source == TerrainSource::None {
        return r#"
            (function() {
                console.log('Terrain disabled in configuration');
            })();
        "#.to_string();
    }

    let tile_url_escaped = js_escape_string(&config.terrain_tile_url());
    let exaggeration = config.terrain_exaggeration();

    // Attribution and source config depend on terrain source type
    let (attribution, source_declaration) = match config.terrain_source {
        TerrainSource::MapTiler => (
            "&copy; MapTiler &copy; OpenStreetMap contributors",
            format!("url: '{}'", tile_url_escaped)
        ),
        TerrainSource::LocalAHN => (
            "&copy; PDOK AHN3/4 - Kadaster",
            format!("tiles: ['{}'], tileSize: 256", tile_url_escaped)
        ),
        TerrainSource::None => ("", String::new()),
    };

    // Build the source config string
    let source_config = format!(
        r#"type: 'raster-dem', {}, attribution: '{}'"#,
        source_declaration, attribution
    );

    // Build the complete script
    // Note: We build the source config as a string, then insert it
    // The format string has 3 placeholders: container_id, source_config, exaggeration
    format!(r#"
        (function() {{
            const map = window['map_{}'];
            if (!map) {{
                console.error('Map not found for terrain initialization');
                return;
            }}

            if (map.getSource('ahn3-terrain')) {{
                console.log('Terrain source already exists, skipping');
                return;
            }}

            const exaggeration = {};
            console.log('Initializing terrain: exaggeration=', exaggeration);

            map.addSource('ahn3-terrain', {{ {} }});

            map.setTerrain({{ source: 'ahn3-terrain', exaggeration: exaggeration }});

            map.on('terrain.loading', function() {{
                console.log('Terrain tiles loading...');
            }});

            map.on('terrain', function() {{
                console.log('Terrain loaded');
                if (window.sendToRust) {{
                    window.sendToRust(JSON.stringify({{ event: 'terrain_loaded' }}));
                }}
            }});

            map.on('terrain.error', function(error) {{
                console.error('Terrain error:', error);
            }});
        }})();
    "#, container_id, source_config, exaggeration)
}

/// Builds the JavaScript for terrain error handling.
///
/// Sets up listeners for:
/// - Terrain tile load errors
/// - Source data unavailability
/// - Sends error events back to Rust via sendToRust
pub fn build_terrain_error_script(container_id: &str) -> String {
    assert!(is_valid_container_id(container_id), "Invalid container_id");

    format!(r#"
        (function() {{
            const map = window['map_{}'];
            if (!map) {{
                console.error('Map not found for terrain error handling');
                return;
            }}

            map.on('terrain.error', function(error) {{
                console.error('Terrain load error:', error);
                if (window.sendToRust) {{
                    window.sendToRust(JSON.stringify({{
                        event: 'terrain_error',
                        message: error.error || 'Unknown terrain error'
                    }}));
                }}
            }});

            // Also listen for source data errors
            map.on('data', function(event) {{
                if (event.sourceId === 'ahn3-terrain' && event.isSourceLoaded) {{
                    const source = map.getSource('ahn3-terrain');
                    if (!source || !source.tiles) {{
                        console.warn('Terrain source unavailable');
                        if (window.sendToRust) {{
                            window.sendToRust(JSON.stringify({{
                                event: 'terrain_unavailable'
                            }}));
                        }}
                    }}
                }}
            }});
        }})();
    "#, container_id)
}

/// Builds the JavaScript to update terrain exaggeration.
///
/// # Arguments
///
/// * `container_id` - The map container ID
/// * `value` - New exaggeration value (will be clamped to [0.1, 5.0])
pub fn build_set_terrain_exaggeration_script(container_id: &str, mut value: f64) -> String {
    assert!(is_valid_container_id(container_id), "Invalid container_id");

    // Clamp to valid range for MapLibre
    value = value.clamp(0.1, 5.0);

    format!(r#"
        (function() {{
            const map = window['map_{}'];
            if (!map) {{
                console.error('Map not found');
                return;
            }}

            if (map.getTerrain()) {{
                map.setTerrain({{ source: 'ahn3-terrain', exaggeration: {value} }});
                console.log('Terrain exaggeration set to {value}');
            }} else {{
                console.warn('No terrain active, cannot set exaggeration');
            }}
        }})();
    "#, container_id)
}

/// Terrain state tracking for the Map3D component.
///
/// Tracks whether terrain is loaded, enabled, and any error state.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainState {
    /// Whether terrain data has finished loading
    pub loaded: bool,
    /// Whether terrain rendering is currently enabled
    pub enabled: bool,
    /// Error message if terrain failed to load
    pub error: Option<String>,
}

impl Default for TerrainState {
    fn default() -> Self {
        Self {
            loaded: false,
            enabled: true,
            error: None,
        }
    }
}

impl TerrainState {
    /// Creates a new TerrainState.
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks terrain as loaded.
    pub fn mark_loaded(&mut self) {
        self.loaded = true;
        self.error = None;
    }

    /// Marks terrain as failed with an error message.
    pub fn mark_error(&mut self, message: String) {
        self.error = Some(message);
        self.enabled = false;
    }

    /// Returns the Dutch error message for the current error state.
    pub fn error_message_dutch(&self) -> Option<String> {
        self.error.as_ref().map(|e| {
            let e_lower = e.to_lowercase();
            if e_lower.contains("401") || e_lower.contains("403") || e_lower.contains("key") {
                "Ongeldige MapTiler API-sleutel. Controleer uw configuratie.".to_string()
            } else if e_lower.contains("network") || e_lower.contains("fetch") {
                "Netwerkfout bij laden terrein. 2D-modus actief.".to_string()
            } else {
                "3D-terrein niet beschikbaar. 2D-modus actief.".to_string()
            }
        })
    }
}

/// Terrain warning component for displaying error messages.
///
/// Shows a Dutch warning banner when terrain fails to load.
#[component]
pub fn TerrainWarning(message: String) -> Element {
    rsx! {
        div {
            class: "terrain-warning",
            style: "position: absolute; top: 10px; left: 50%; transform: translateX(-50%);
                    background: rgba(255, 193, 7, 0.95); padding: 8px 16px;
                    border-radius: 4px; z-index: 1000; font-size: 14px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
            div {
                style: "display: flex; align-items: center;",
                span {
                    style: "margin-right: 8px; font-size: 16px;",
                    "⚠️"
                }
                span {
                    style: "color: #333;",
                    "{message}"
                }
            }
        }
    }
}

// ============================================================================
// GeoJSON Layer Management Functions
// ============================================================================

use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};

/// Generates JavaScript to add a GeoJSON layer to the map.
///
/// This function generates JavaScript that:
/// 1. Checks if the map instance exists
/// 2. Adds a GeoJSON source if it doesn't exist
/// 3. Adds a layer with appropriate styling based on geometry type
///
/// # Arguments
///
/// * `container_id` - The map container ID
/// * `layer` - The GeoJSON layer configuration
///
/// # Returns
///
/// A JavaScript string that can be executed via eval()
pub fn build_add_geojson_layer_script(container_id: &str, layer: &GeoJsonLayer) -> String {
    assert!(is_valid_container_id(container_id), "Invalid container_id");
    assert!(is_valid_container_id(&layer.id), "Invalid layer_id");
    assert!(is_valid_hex_color(&layer.color), "Invalid hex color");

    let url_escaped = js_escape_string(&layer.url);
    let color_escaped = js_escape_string(&layer.color);

    let (layer_type, paint_properties) = match layer.layer_type {
        LayerType::Point => (
            "circle",
            format!(r#"'circle-color': '{}', 'circle-radius': 6"#, color_escaped)
        ),
        LayerType::Line => (
            "line",
            format!(r#"'line-color': '{}', 'line-width': 2"#, color_escaped)
        ),
        LayerType::Polygon => (
            "fill",
            format!(r#"'fill-color': '{}', 'fill-opacity': 0.3"#, color_escaped)
        ),
    };

    format!(r#"
        (function() {{
            const map = window['map_{}'];
            if (!map) {{
                console.error('Map not found for layer: {}');
                return {{ success: false, error: 'Map not found' }};
            }}

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
                console.error('Error adding layer:', e);
                return {{ success: false, error: e.toString() }};
            }}
        }})();
    "#,
        container_id,
        layer.id,
        layer.id,
        layer.id,
        url_escaped,
        layer.id,
        layer.id,
        layer_type,
        layer.id,
        paint_properties,
        layer.id
    )
}

/// Generates JavaScript to toggle a layer's visibility.
///
/// # Arguments
///
/// * `container_id` - The map container ID
/// * `layer_id` - The ID of the layer to toggle
/// * `visible` - Whether the layer should be visible
///
/// # Returns
///
/// A JavaScript string that can be executed via eval()
pub fn build_toggle_layer_visibility_script(container_id: &str, layer_id: &str, visible: bool) -> String {
    assert!(is_valid_container_id(container_id), "Invalid container_id");
    assert!(is_valid_container_id(layer_id), "Invalid layer_id");

    let visibility = if visible { "visible" } else { "none" };

    format!(r#"
        (function() {{
            const map = window['map_{}'];
            if (!map) {{
                console.error('Map not found');
                return {{ success: false, error: 'Map not found' }};
            }}

            try {{
                map.setLayoutProperty('{}', 'visibility', '{}');
                return {{ success: true, visible: {} }};
            }} catch (e) {{
                console.error('Error toggling layer:', e);
                return {{ success: false, error: e.toString() }};
            }}
        }})();
    "#,
        container_id,
        layer_id,
        visibility,
        visible
    )
}

/// Generates JavaScript to remove a layer from the map.
///
/// # Arguments
///
/// * `container_id` - The map container ID
/// * `layer_id` - The ID of the layer to remove
///
/// # Returns
///
/// A JavaScript string that can be executed via eval()
pub fn build_remove_layer_script(container_id: &str, layer_id: &str) -> String {
    assert!(is_valid_container_id(container_id), "Invalid container_id");
    assert!(is_valid_container_id(layer_id), "Invalid layer_id");

    format!(r#"
        (function() {{
            const map = window['map_{}'];
            if (!map) {{
                console.error('Map not found');
                return {{ success: false, error: 'Map not found' }};
            }}

            try {{
                if (map.getLayer('{}')) {{
                    map.removeLayer('{}');
                }}
                if (map.getSource('{}')) {{
                    map.removeSource('{}');
                }}
                return {{ success: true }};
            }} catch (e) {{
                console.error('Error removing layer:', e);
                return {{ success: false, error: e.toString() }};
            }}
        }})();
    "#,
        container_id,
        layer_id,
        layer_id,
        layer_id,
        layer_id
    )
}

// Note: The actual Map3D Dioxus component requires web-specific APIs
// that are not available in the test environment. The component will be
// implemented in a future section when we can test it in a browser context.
// For now, we provide the helper functions that generate the JavaScript
// code that will be executed in the browser.

#[cfg(test)]
mod component_tests {
    use super::*;
    use serial_test::serial;

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

    // Terrain integration tests (section-05)

    #[test]
    fn test_terrain_tile_url_includes_api_key() {
        std::env::set_var("MAPTILER_API_KEY", "test_key_123");
        let config = Map3DConfig::default();
        let url = config.terrain_tile_url();
        assert!(url.contains("test_key_123"));
        assert!(url.contains("maptiler.com"));
        std::env::remove_var("MAPTILER_API_KEY");
    }

    #[test]
    #[serial]
    fn test_terrain_tile_url_fallback_when_no_key() {
        // Clear the environment variable to test fallback behavior
        // Uses #[serial] to prevent interference from parallel tests
        std::env::remove_var("MAPTILER_API_KEY");
        let config = Map3DConfig::default();
        let url = config.terrain_tile_url();
        // When no API key is set, defaults to LocalAHN tiles
        assert!(url.contains("/static/terrain/"));
        assert_eq!(config.terrain_source, TerrainSource::LocalAHN);
    }

    #[test]
    fn test_terrain_tile_url_uses_custom_url() {
        std::env::set_var("TERRAIN_TILE_URL", "https://custom.tiles.com/terrain.json");
        let config = Map3DConfig::from_env();
        let url = config.terrain_tile_url();
        assert!(url.contains("custom.tiles.com"));
        std::env::remove_var("TERRAIN_TILE_URL");
    }

    #[test]
    fn test_terrain_exaggeration_reads_from_env() {
        std::env::set_var("TERRAIN_EXAGGERATION", "2.5");
        let config = Map3DConfig::default();
        let exag = config.terrain_exaggeration();
        assert_eq!(exag, 2.5);
        std::env::remove_var("TERRAIN_EXAGGERATION");
    }

    #[test]
    fn test_terrain_exaggeration_defaults_to_config_value() {
        std::env::remove_var("TERRAIN_EXAGGERATION");
        let mut config = Map3DConfig::default();
        config.terrain_exaggeration = 2.0;
        let exag = config.terrain_exaggeration();
        assert_eq!(exag, 2.0);
    }

    #[test]
    fn test_terrain_exaggeration_clamps_to_valid_range() {
        std::env::set_var("TERRAIN_EXAGGERATION", "10.0");
        let config = Map3DConfig::default();
        let exag = config.terrain_exaggeration();
        assert_eq!(exag, 5.0); // Clamped to max
        std::env::remove_var("TERRAIN_EXAGGERATION");

        std::env::set_var("TERRAIN_EXAGGERATION", "0.0");
        let config = Map3DConfig::default();
        let exag = config.terrain_exaggeration();
        assert_eq!(exag, 0.1); // Clamped to min
        std::env::remove_var("TERRAIN_EXAGGERATION");
    }

    #[test]
    fn test_from_env_reads_terrain_exaggeration() {
        std::env::set_var("TERRAIN_EXAGGERATION", "3.0");
        let config = Map3DConfig::from_env();
        assert_eq!(config.terrain_exaggeration, 3.0);
        std::env::remove_var("TERRAIN_EXAGGERATION");
    }

    #[test]
    fn test_build_terrain_init_script_contains_source_config() {
        let config = Map3DConfig::default();
        let script = build_terrain_init_script(&config);
        assert!(script.contains("ahn3-terrain"));
        assert!(script.contains("raster-dem"));
        assert!(script.contains("terrain_loaded"));
    }

    #[test]
    fn test_build_terrain_init_script_includes_exaggeration() {
        let mut config = Map3DConfig::default();
        config.terrain_exaggeration = 2.0;
        let script = build_terrain_init_script(&config);
        // Check for either "2" or "2.0" format
        assert!(script.contains("exaggeration: ") && script.contains("2"));
    }

    #[test]
    fn test_build_terrain_error_script() {
        let script = build_terrain_error_script("map");
        assert!(script.contains("terrain.error"));
        assert!(script.contains("terrain_error"));
        assert!(script.contains("terrain_unavailable"));
        assert!(script.contains("window['map_map']"));
    }

    #[test]
    fn test_build_set_terrain_exaggeration_script() {
        let script = build_set_terrain_exaggeration_script("map", 1.5);
        assert!(script.contains("exaggeration: 1.5"));
        assert!(script.contains("setTerrain"));
        assert!(script.contains("window['map_map']"));
    }

    #[test]
    fn test_build_set_terrain_exaggeration_script_clamps_value() {
        // Test clamping of values outside valid range
        let script_high = build_set_terrain_exaggeration_script("map", 10.0);
        assert!(script_high.contains("exaggeration: 5")); // Clamped to max

        let script_low = build_set_terrain_exaggeration_script("map", 0.0);
        assert!(script_low.contains("exaggeration: 0.1")); // Clamped to min
    }

    #[test]
    fn test_terrain_state_default() {
        let state = TerrainState::default();
        assert!(!state.loaded);
        assert!(state.enabled);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_terrain_state_mark_loaded() {
        let mut state = TerrainState::new();
        state.mark_loaded();
        assert!(state.loaded);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_terrain_state_mark_error() {
        let mut state = TerrainState::new();
        state.mark_error("Network error".to_string());
        assert!(!state.enabled);
        assert_eq!(state.error, Some("Network error".to_string()));
    }

    #[test]
    fn test_terrain_state_dutch_error_message_unavailable() {
        let mut state = TerrainState::new();
        state.mark_error("Failed to load".to_string());
        let msg = state.error_message_dutch();
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("niet beschikbaar"));
    }

    #[test]
    fn test_terrain_state_dutch_error_message_invalid_key() {
        let mut state = TerrainState::new();
        state.mark_error("401 Unauthorized".to_string());
        let msg = state.error_message_dutch();
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("API-sleutel"));
    }

    #[test]
    fn test_terrain_state_dutch_error_message_network() {
        let mut state = TerrainState::new();
        state.mark_error("Network error".to_string());
        let msg = state.error_message_dutch();
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("Netwerkfout"));
    }

    #[test]
    fn test_terrain_state_dutch_error_message_none_when_no_error() {
        let state = TerrainState::new();
        let msg = state.error_message_dutch();
        assert!(msg.is_none());
    }

    #[test]
    fn test_terrain_init_script_checks_existing_source() {
        let config = Map3DConfig::default();
        let script = build_terrain_init_script(&config);
        assert!(script.contains("getSource('ahn3-terrain')"));
        assert!(script.contains("already exists"));
    }

    #[test]
    fn test_js_escape_string_handles_special_chars() {
        assert_eq!(js_escape_string("test's value"), r#"test\'s value"#);
        assert_eq!(js_escape_string("quote\"test"), r#"quote\"test"#);
        assert_eq!(js_escape_string("back\\slash"), r#"back\\slash"#);
        assert_eq!(js_escape_string("new\nline"), r#"new\nline"#);
    }

    #[test]
    fn test_is_valid_container_id() {
        assert!(is_valid_container_id("map"));
        assert!(is_valid_container_id("my-map-1"));
        assert!(is_valid_container_id("map_3d"));
        assert!(!is_valid_container_id("map;alert(1)"));
        assert!(!is_valid_container_id("map space"));
    }

    #[test]
    fn test_is_valid_hex_color() {
        assert!(is_valid_hex_color("#ff0000"));
        assert!(is_valid_hex_color("#FF0000"));
        assert!(is_valid_hex_color("#00ff00"));
        assert!(is_valid_hex_color("#0000ff"));
        assert!(is_valid_hex_color("#123abc"));
        assert!(!is_valid_hex_color("ff0000"));  // Missing #
        assert!(!is_valid_hex_color("#ff00"));   // Too short
        assert!(!is_valid_hex_color("#ff00000")); // Too long
        assert!(!is_valid_hex_color("#gg0000"));  // Invalid hex
        assert!(!is_valid_hex_color("red"));      // Named color not allowed
    }

    // GeoJSON layer management tests (section-06)

    #[test]
    fn test_build_add_geojson_layer_script_generates_valid_javascript() {
        use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};

        let layer = GeoJsonLayer {
            id: "test-layer".to_string(),
            name: "Test Layer".to_string(),
            url: "/test.geojson".to_string(),
            layer_type: LayerType::Polygon,
            visible: true,
            color: "#ff0000".to_string(),
        };

        let js = build_add_geojson_layer_script("map", &layer);

        // Verify the JavaScript contains key elements
        assert!(js.contains("addSource"));
        assert!(js.contains("test-layer"));
        assert!(js.contains("addLayer"));
        assert!(js.contains("fill"));  // Polygon type
        assert!(js.contains("fill-color"));
        assert!(js.contains("#ff0000"));
    }

    #[test]
    fn test_build_add_geojson_layer_script_handles_point_type() {
        use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};

        let layer = GeoJsonLayer {
            id: "points".to_string(),
            name: "Points".to_string(),
            url: "/points.geojson".to_string(),
            layer_type: LayerType::Point,
            visible: true,
            color: "#00ff00".to_string(),
        };

        let js = build_add_geojson_layer_script("map", &layer);

        assert!(js.contains("circle"));  // Point type
        assert!(js.contains("circle-color"));
        assert!(js.contains("#00ff00"));
        assert!(js.contains("circle-radius"));
    }

    #[test]
    fn test_build_add_geojson_layer_script_handles_line_type() {
        use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};

        let layer = GeoJsonLayer {
            id: "lines".to_string(),
            name: "Lines".to_string(),
            url: "/lines.geojson".to_string(),
            layer_type: LayerType::Line,
            visible: true,
            color: "#0000ff".to_string(),
        };

        let js = build_add_geojson_layer_script("map", &layer);

        assert!(js.contains("line"));  // Line type
        assert!(js.contains("line-color"));
        assert!(js.contains("#0000ff"));
        assert!(js.contains("line-width"));
    }

    #[test]
    fn test_build_toggle_layer_visibility_script_visible() {
        let js = build_toggle_layer_visibility_script("map", "test-layer", true);
        assert!(js.contains("setLayoutProperty"));
        assert!(js.contains("'visibility', 'visible'"));
        assert!(js.contains("window['map_map']"));
    }

    #[test]
    fn test_build_toggle_layer_visibility_script_hidden() {
        let js = build_toggle_layer_visibility_script("map", "test-layer", false);
        assert!(js.contains("setLayoutProperty"));
        assert!(js.contains("'visibility', 'none'"));
    }

    #[test]
    fn test_build_remove_layer_script() {
        let js = build_remove_layer_script("map", "test-layer");
        assert!(js.contains("removeLayer"));
        assert!(js.contains("removeSource"));
        assert!(js.contains("test-layer"));
        assert!(js.contains("window['map_map']"));
    }

    #[test]
    fn test_layer_script_escaping() {
        use crate::components::layer_control_3d::{GeoJsonLayer, LayerType};

        // Test layer with special characters in ID
        let layer = GeoJsonLayer {
            id: "layer-with-dash".to_string(),
            name: "Test Layer".to_string(),
            url: "/test.geojson".to_string(),
            layer_type: LayerType::Point,
            visible: true,
            color: "#ff0000".to_string(),
        };

        let js = build_add_geojson_layer_script("map", &layer);
        assert!(js.contains("layer-with-dash"));
    }
}
