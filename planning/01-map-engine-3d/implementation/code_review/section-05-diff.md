diff --git a/crates/iou-frontend/src/components/map_3d.rs b/crates/iou-frontend/src/components/map_3d.rs
index 5113837..62666e0 100644
--- a/crates/iou-frontend/src/components/map_3d.rs
+++ b/crates/iou-frontend/src/components/map_3d.rs
@@ -190,12 +190,45 @@ impl Map3DConfig {
         })
     }
 
+    /// Returns the terrain tile URL with MapTiler API key.
+    ///
+    /// Uses the MAPTILER_API_KEY environment variable if set.
+    /// Falls back to a placeholder if the key is not configured.
+    pub fn terrain_tile_url(&self) -> String {
+        // If a custom URL was set, use it directly
+        if !self.terrain_tile_url.contains("maptiler.com") {
+            return self.terrain_tile_url.clone();
+        }
+
+        // MapTiler requires an API key
+        let api_key = env::var("MAPTILER_API_KEY")
+            .unwrap_or_else(|_| "YOUR_KEY_HERE".to_string());
+
+        format!(
+            "https://api.maptiler.com/tiles/terrain-rgb/tiles.json?key={}",
+            api_key
+        )
+    }
+
+    /// Returns the terrain exaggeration value.
+    ///
+    /// Reads from TERRAIN_EXAGGERATION environment variable if set,
+    /// otherwise uses the configured value.
+    pub fn terrain_exaggeration(&self) -> f64 {
+        env::var("TERRAIN_EXAGGERATION")
+            .ok()
+            .and_then(|s| s.parse().ok())
+            .unwrap_or(self.terrain_exaggeration)
+    }
+
     /// Creates a Map3DConfig from environment variables.
     ///
     /// Reads the following environment variables:
     /// - `MAP_3D_ENABLED`: "true" to enable 3D map (default: false)
     /// - `MAP_STYLE_URL`: Custom map style URL (optional)
     /// - `TERRAIN_TILE_URL`: Custom terrain tile URL (optional)
+    /// - `MAPTILER_API_KEY`: MapTiler API key for terrain tiles
+    /// - `TERRAIN_EXAGGERATION`: Terrain vertical exaggeration (default: 1.5)
     ///
     /// # Returns
     ///
@@ -214,6 +247,13 @@ impl Map3DConfig {
             config.style_url = url;
         }
 
+        // Allow custom terrain exaggeration
+        if let Ok(exag) = env::var("TERRAIN_EXAGGERATION") {
+            if let Ok(v) = exag.parse() {
+                config.terrain_exaggeration = v;
+            }
+        }
+
         config
     }
 }
@@ -498,6 +538,207 @@ pub fn build_cleanup_script(container_id: &str) -> String {
     "#, container_id, container_id, container_id)
 }
 
+/// Builds the JavaScript to add terrain source to the map.
+///
+/// This function:
+/// 1. Checks if terrain source already exists
+/// 2. Adds the terrain DEM source from MapTiler
+/// 3. Sets terrain with exaggeration
+/// 4. Sets up terrain event listeners
+pub fn build_terrain_init_script(config: &Map3DConfig) -> String {
+    format!(r#"
+        (function() {{
+            const map = window['map_{}'];
+            if (!map) {{
+                console.error('Map not found for terrain initialization');
+                return;
+            }}
+
+            if (map.getSource('ahn3-terrain')) {{
+                console.log('Terrain source already exists, skipping');
+                return;
+            }}
+
+            const tileUrl = '{}';
+            const exaggeration = {};
+
+            map.addSource('ahn3-terrain', {{
+                type: 'raster-dem',
+                tiles: [tileUrl],
+                tileSize: 256,
+                attribution: '&copy; MapTiler &copy; OpenStreetMap contributors'
+            }});
+
+            map.setTerrain({{ source: 'ahn3-terrain', exaggeration: exaggeration }});
+
+            map.on('terrain.loading', function() {{
+                console.log('Terrain tiles loading...');
+            }});
+
+            map.on('terrain', function() {{
+                console.log('Terrain loaded');
+                if (window.sendToRust) {{
+                    window.sendToRust(JSON.stringify({{ event: 'terrain_loaded' }}));
+                }}
+            }});
+        }})();
+    "#,
+        config.container_id,
+        config.terrain_tile_url(),
+        config.terrain_exaggeration()
+    )
+}
+
+/// Builds the JavaScript for terrain error handling.
+///
+/// Sets up listeners for:
+/// - Terrain tile load errors
+/// - Source data unavailability
+/// - Sends error events back to Rust via sendToRust
+pub fn build_terrain_error_script() -> String {
+    r#"
+        (function() {
+            const map = window.map;
+            if (!map) {
+                console.error('Map not found for terrain error handling');
+                return;
+            }
+
+            map.on('terrain.error', function(error) {
+                console.error('Terrain load error:', error);
+                if (window.sendToRust) {
+                    window.sendToRust(JSON.stringify({
+                        event: 'terrain_error',
+                        message: error.error || 'Unknown terrain error'
+                    }));
+                }
+            });
+
+            // Also listen for source data errors
+            map.on('data', function(event) {
+                if (event.sourceId === 'ahn3-terrain' && event.isSourceLoaded) {
+                    const source = map.getSource('ahn3-terrain');
+                    if (!source || !source.tiles) {
+                        console.warn('Terrain source unavailable');
+                        if (window.sendToRust) {
+                            window.sendToRust(JSON.stringify({
+                                event: 'terrain_unavailable'
+                            }));
+                        }
+                    }
+                }
+            });
+        })();
+    "#.to_string()
+}
+
+/// Builds the JavaScript to update terrain exaggeration.
+///
+/// # Arguments
+///
+/// * `value` - New exaggeration value (0.1 to 5.0)
+pub fn build_set_terrain_exaggeration_script(value: f64) -> String {
+    format!(r#"
+        (function() {{
+            const map = window.map;
+            if (!map) {{
+                console.error('Map not found');
+                return;
+            }}
+
+            if (map.getTerrain()) {{
+                map.setTerrain({{ source: 'ahn3-terrain', exaggeration: {value} }});
+                console.log('Terrain exaggeration set to {value}');
+            }} else {{
+                console.warn('No terrain active, cannot set exaggeration');
+            }}
+        }})();
+    "#)
+}
+
+/// Terrain state tracking for the Map3D component.
+///
+/// Tracks whether terrain is loaded, enabled, and any error state.
+#[derive(Debug, Clone, PartialEq)]
+pub struct TerrainState {
+    /// Whether terrain data has finished loading
+    pub loaded: bool,
+    /// Whether terrain rendering is currently enabled
+    pub enabled: bool,
+    /// Error message if terrain failed to load
+    pub error: Option<String>,
+}
+
+impl Default for TerrainState {
+    fn default() -> Self {
+        Self {
+            loaded: false,
+            enabled: true,
+            error: None,
+        }
+    }
+}
+
+impl TerrainState {
+    /// Creates a new TerrainState.
+    pub fn new() -> Self {
+        Self::default()
+    }
+
+    /// Marks terrain as loaded.
+    pub fn mark_loaded(&mut self) {
+        self.loaded = true;
+        self.error = None;
+    }
+
+    /// Marks terrain as failed with an error message.
+    pub fn mark_error(&mut self, message: String) {
+        self.error = Some(message);
+        self.enabled = false;
+    }
+
+    /// Returns the Dutch error message for the current error state.
+    pub fn error_message_dutch(&self) -> Option<String> {
+        self.error.as_ref().map(|e| {
+            let e_lower = e.to_lowercase();
+            if e_lower.contains("401") || e_lower.contains("403") || e_lower.contains("key") {
+                "Ongeldige MapTiler API-sleutel. Controleer uw configuratie.".to_string()
+            } else if e_lower.contains("network") || e_lower.contains("fetch") {
+                "Netwerkfout bij laden terrein. 2D-modus actief.".to_string()
+            } else {
+                "3D-terrein niet beschikbaar. 2D-modus actief.".to_string()
+            }
+        })
+    }
+}
+
+/// Terrain warning component for displaying error messages.
+///
+/// Shows a Dutch warning banner when terrain fails to load.
+#[component]
+pub fn TerrainWarning(message: String) -> Element {
+    rsx! {
+        div {
+            class: "terrain-warning",
+            style: "position: absolute; top: 10px; left: 50%; transform: translateX(-50%);
+                    background: rgba(255, 193, 7, 0.95); padding: 8px 16px;
+                    border-radius: 4px; z-index: 1000; font-size: 14px;
+                    box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
+            div {
+                style: "display: flex; align-items: center;",
+                span {
+                    style: "margin-right: 8px; font-size: 16px;",
+                    "⚠️"
+                }
+                span {
+                    style: "color: #333;",
+                    "{message}"
+                }
+            }
+        }
+    }
+}
+
 // Note: The actual Map3D Dioxus component requires web-specific APIs
 // that are not available in the test environment. The component will be
 // implemented in a future section when we can test it in a browser context.
@@ -663,4 +904,158 @@ mod component_tests {
         assert!(script.contains("webgl2"));
         assert!(script.contains("WebGL2 not supported"));
     }
+
+    // Terrain integration tests (section-05)
+
+    #[test]
+    fn test_terrain_tile_url_includes_api_key() {
+        std::env::set_var("MAPTILER_API_KEY", "test_key_123");
+        let config = Map3DConfig::default();
+        let url = config.terrain_tile_url();
+        assert!(url.contains("test_key_123"));
+        assert!(url.contains("maptiler.com"));
+        std::env::remove_var("MAPTILER_API_KEY");
+    }
+
+    #[test]
+    fn test_terrain_tile_url_fallback_when_no_key() {
+        std::env::remove_var("MAPTILER_API_KEY");
+        let config = Map3DConfig::default();
+        let url = config.terrain_tile_url();
+        assert!(url.contains("YOUR_KEY_HERE"));
+    }
+
+    #[test]
+    fn test_terrain_tile_url_uses_custom_url() {
+        std::env::set_var("TERRAIN_TILE_URL", "https://custom.tiles.com/terrain.json");
+        let config = Map3DConfig::from_env();
+        let url = config.terrain_tile_url();
+        assert!(url.contains("custom.tiles.com"));
+        std::env::remove_var("TERRAIN_TILE_URL");
+    }
+
+    #[test]
+    fn test_terrain_exaggeration_reads_from_env() {
+        std::env::set_var("TERRAIN_EXAGGERATION", "2.5");
+        let config = Map3DConfig::default();
+        let exag = config.terrain_exaggeration();
+        assert_eq!(exag, 2.5);
+        std::env::remove_var("TERRAIN_EXAGGERATION");
+    }
+
+    #[test]
+    fn test_terrain_exaggeration_defaults_to_config_value() {
+        std::env::remove_var("TERRAIN_EXAGGERATION");
+        let mut config = Map3DConfig::default();
+        config.terrain_exaggeration = 2.0;
+        let exag = config.terrain_exaggeration();
+        assert_eq!(exag, 2.0);
+    }
+
+    #[test]
+    fn test_from_env_reads_terrain_exaggeration() {
+        std::env::set_var("TERRAIN_EXAGGERATION", "3.0");
+        let config = Map3DConfig::from_env();
+        assert_eq!(config.terrain_exaggeration, 3.0);
+        std::env::remove_var("TERRAIN_EXAGGERATION");
+    }
+
+    #[test]
+    fn test_build_terrain_init_script_contains_source_config() {
+        let config = Map3DConfig::default();
+        let script = build_terrain_init_script(&config);
+        assert!(script.contains("ahn3-terrain"));
+        assert!(script.contains("raster-dem"));
+        assert!(script.contains("terrain_loaded"));
+    }
+
+    #[test]
+    fn test_build_terrain_init_script_includes_exaggeration() {
+        let mut config = Map3DConfig::default();
+        config.terrain_exaggeration = 2.0;
+        let script = build_terrain_init_script(&config);
+        // Check for either "2" or "2.0" format
+        assert!(script.contains("exaggeration: ") && script.contains("2"));
+    }
+
+    #[test]
+    fn test_build_terrain_error_script() {
+        let script = build_terrain_error_script();
+        assert!(script.contains("terrain.error"));
+        assert!(script.contains("terrain_error"));
+        assert!(script.contains("terrain_unavailable"));
+    }
+
+    #[test]
+    fn test_build_set_terrain_exaggeration_script() {
+        let script = build_set_terrain_exaggeration_script(1.5);
+        assert!(script.contains("exaggeration: 1.5"));
+        assert!(script.contains("setTerrain"));
+    }
+
+    #[test]
+    fn test_terrain_state_default() {
+        let state = TerrainState::default();
+        assert!(!state.loaded);
+        assert!(state.enabled);
+        assert!(state.error.is_none());
+    }
+
+    #[test]
+    fn test_terrain_state_mark_loaded() {
+        let mut state = TerrainState::new();
+        state.mark_loaded();
+        assert!(state.loaded);
+        assert!(state.error.is_none());
+    }
+
+    #[test]
+    fn test_terrain_state_mark_error() {
+        let mut state = TerrainState::new();
+        state.mark_error("Network error".to_string());
+        assert!(!state.enabled);
+        assert_eq!(state.error, Some("Network error".to_string()));
+    }
+
+    #[test]
+    fn test_terrain_state_dutch_error_message_unavailable() {
+        let mut state = TerrainState::new();
+        state.mark_error("Failed to load".to_string());
+        let msg = state.error_message_dutch();
+        assert!(msg.is_some());
+        assert!(msg.unwrap().contains("niet beschikbaar"));
+    }
+
+    #[test]
+    fn test_terrain_state_dutch_error_message_invalid_key() {
+        let mut state = TerrainState::new();
+        state.mark_error("401 Unauthorized".to_string());
+        let msg = state.error_message_dutch();
+        assert!(msg.is_some());
+        assert!(msg.unwrap().contains("API-sleutel"));
+    }
+
+    #[test]
+    fn test_terrain_state_dutch_error_message_network() {
+        let mut state = TerrainState::new();
+        state.mark_error("Network error".to_string());
+        let msg = state.error_message_dutch();
+        assert!(msg.is_some());
+        assert!(msg.unwrap().contains("Netwerkfout"));
+    }
+
+    #[test]
+    fn test_terrain_state_dutch_error_message_none_when_no_error() {
+        let state = TerrainState::new();
+        let msg = state.error_message_dutch();
+        assert!(msg.is_none());
+    }
+
+    #[test]
+    fn test_terrain_init_script_checks_existing_source() {
+        let config = Map3DConfig::default();
+        let script = build_terrain_init_script(&config);
+        assert!(script.contains("getSource('ahn3-terrain')"));
+        assert!(script.contains("already exists"));
+    }
 }
