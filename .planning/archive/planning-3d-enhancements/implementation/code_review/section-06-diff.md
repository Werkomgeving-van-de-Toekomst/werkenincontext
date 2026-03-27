# Section 06 Diff: Frontend Dynamic Loading

## Files Modified

- `crates/iou-frontend/src/pages/data_verkenner.rs` (197 insertions, 84 deletions)

## Summary of Changes

This section implements dynamic building loading based on map viewport. The changes replace the hardcoded RD bbox fetch with a dynamic WGS84-based loading system.

### Key Changes:

1. **New function `build_buildings_fetch_script()`**: Generates JavaScript for dynamic loading
   - State tracking: `lastFetchedBbox`, `fetchTimeout`
   - `shouldFetch()`: 10% threshold check
   - `updateBuildingsSource()`: Updates GeoJSON source, clears old popups
   - `showNoBuildingsMessage()`: Empty state handler
   - `showErrorMessage()`: Error logging
   - `fetchBuildings()`: Async fetch from `/api/buildings-3d?bbox-wgs84=...`
   - `debouncedFetch()`: 300ms debounce

2. **Modified `get_map3d_init_script()`**: Now incorporates the fetch script
   - Uses `format!` to inject the fetch script
   - Attaches `debouncedFetch` to `moveend` and `zoomend` events
   - Calls `debouncedFetch()` on initial map load

3. **Added tests**: 5 unit tests verifying the fetch script
   - `test_fetch_script_includes_wgs84_endpoint`
   - `test_fetch_script_includes_debounce`
   - `test_fetch_script_includes_state_variables`
   - `test_fetch_script_includes_threshold_check`
   - `test_fetch_script_includes_error_handling`

## Full Diff

diff --git a/crates/iou-frontend/src/pages/data_verkenner.rs b/crates/iou-frontend/src/pages/data_verkenner.rs
index e19877d..3d7c0f4 100644
--- a/crates/iou-frontend/src/pages/data_verkenner.rs
+++ b/crates/iou-frontend/src/pages/data_verkenner.rs
@@ -97,37 +97,159 @@ fn is_3d_map_enabled() -> bool {
     }
 }
 
+/// Builds the JavaScript fetch script for dynamic building loading.
+fn build_buildings_fetch_script() -> String {
+    r#"
+        // State tracking for dynamic loading
+        let lastFetchedBbox = null;
+        let fetchTimeout = null;
+
+        // Check if bounds changed by at least 10% threshold
+        function shouldFetch(newBounds) {
+            if (!lastFetchedBbox) return true;
+
+            const width = newBounds[2] - newBounds[0];
+            const lastWidth = lastFetchedBbox[2] - lastFetchedBbox[0];
+            const height = newBounds[3] - newBounds[1];
+            const lastHeight = lastFetchedBbox[3] - lastFetchedBbox[1];
+
+            // Handle zero dimensions
+            if (lastWidth === 0 || lastHeight === 0) return true;
+
+            const widthChange = Math.abs(width - lastWidth) / lastWidth;
+            const heightChange = Math.abs(height - lastHeight) / lastHeight;
+
+            return widthChange > 0.1 || heightChange > 0.1;
+        }
+
+        // Update buildings source with new GeoJSON data
+        function updateBuildingsSource(geojson) {
+            // Clear any existing popups
+            document.querySelectorAll('.maplibregl-popup').forEach(p => p.remove());
+
+            const source = map.getSource('buildings');
+            if (source) {
+                source.setData(geojson);
+            } else {
+                // Source doesn't exist yet, create it
+                map.addSource('buildings', {
+                    type: 'geojson',
+                    data: geojson
+                });
+
+                map.addLayer({
+                    id: 'building-3d',
+                    type: 'fill-extrusion',
+                    source: 'buildings',
+                    paint: {
+                        'fill-extrusion-color': '#8899aa',
+                        'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
+                        'fill-extrusion-base': 0,
+                        'fill-extrusion-opacity': 0.8
+                    }
+                });
+            }
+        }
+
+        // Show empty state when no buildings found
+        function showNoBuildingsMessage() {
+            const source = map.getSource('buildings');
+            if (source) {
+                source.setData({
+                    type: 'FeatureCollection',
+                    features: []
+                });
+            }
+        }
+
+        // Show error message
+        function showErrorMessage(message) {
+            console.warn(message);
+        }
+
+        // Fetch buildings from API with WGS84 bbox
+        async function fetchBuildings(bbox) {
+            try {
+                const response = await fetch(
+                    `/api/buildings-3d?bbox-wgs84=${bbox.join(',')}&limit=150`
+                );
+
+                if (!response.ok) {
+                    if (response.status === 404) {
+                        showNoBuildingsMessage();
+                        return;
+                    }
+                    throw new Error(`API error: ${response.status}`);
+                }
+
+                const data = await response.json();
+
+                if (data.features.length === 0) {
+                    showNoBuildingsMessage();
+                    return;
+                }
+
+                updateBuildingsSource(data);
+                console.log('Loaded ' + data.features.length + ' buildings from 3DBAG');
+
+            } catch (error) {
+                console.error('Failed to fetch buildings:', error);
+                showErrorMessage('Kon gebouwen niet laden. Probeer het opnieuw.');
+            }
+        }
+
+        // Debounced fetch function
+        function debouncedFetch() {
+            clearTimeout(fetchTimeout);
+            fetchTimeout = setTimeout(() => {
+                const bounds = map.getBounds();
+                const bbox = [bounds.getWest(), bounds.getSouth(),
+                              bounds.getEast(), bounds.getNorth()];
+
+                if (shouldFetch(bbox)) {
+                    fetchBuildings(bbox);
+                    lastFetchedBbox = bbox;
+                }
+            }, 300);
+        }
+    "#.to_string()
+}
+
 /// Returns the Map3D initialization script.
 fn get_map3d_init_script() -> String {
-    r#"
-    (function() {
+    let fetch_script = build_buildings_fetch_script();
+    format!(
+        r#"
+    (function() {{
         console.log('Map3D script loaded');
 
-        function waitForElement(id, callback) {
+        {fetch_script}
+
+        function waitForElement(id, callback) {{
             var element = document.getElementById(id);
-            if (element) {
+            if (element) {{
                 callback(element);
-            } else {
-                setTimeout(function() { waitForElement(id, callback); }, 100);
-            }
-        }
+            }} else {{
+                setTimeout(function() {{ waitForElement(id, callback); }}, 100);
+            }}
+        }}
 
-        function initMap3D() {
-            if (typeof window.maplibregl === 'undefined') {
+        function initMap3D() {{
+            if (typeof window.maplibregl === 'undefined') {{
                 console.log('MapLibre GL not loaded yet, waiting...');
                 setTimeout(initMap3D, 200);
                 return;
-            }
+            }}
 
-            if (window.map_map) {
+            if (window.map_map) {{
                 console.log('Map already initialized');
                 return;
-            }
+            }}
 
             console.log('Initializing Map3D...');
 
-            try {
-                var map = new maplibregl.Map({
+            try {{
+                var map = new maplibregl.Map({{
                     container: 'map',
                     style: 'https://basemaps.cartocdn.com/gl/positron-gl-style/style.json',
                     center: [5.6, 52.4],
@@ -135,87 +257,38 @@ fn get_map3d_init_script() -> String {
                     pitch: 60,
                     bearing: 0,
                     antialias: true
-                });
+                }});
 
-                map.addControl(new maplibregl.NavigationControl({ visualizePitch: true }));
+                map.addControl(new maplibregl.NavigationControl({{ visualizePitch: true }}));
                 window.map_map = map;
 
-                map.on('load', function() {
-                    console.log('Map loaded, fetching buildings...');
-
-                    fetch('/api/buildings-3d?bbox=150000,470000,170000,490000&limit=100')
-                        .then(r => {
-                            console.log('Buildings fetch status:', r.status, r.statusText);
-                            if (!r.ok) throw new Error('Failed to fetch: ' + r.status + ' ' + r.statusText);
-                            return r.json();
-                        })
-                        .then(data => {
-                            console.log('Buildings response:', data);
-
-                            if (data.features && data.features.length > 0) {
-                                // Log first building coordinates for debugging
-                                console.log('First building coordinates:', data.features[0].geometry.coordinates);
-
-                                map.addSource('buildings', {
-                                    type: 'geojson',
-                                    data: data
-                                });
+                map.on('load', function() {{
+                    console.log('Map loaded, setting up dynamic building loading...');
 
-                                map.addLayer({
-                                    id: 'building-3d',
-                                    type: 'fill-extrusion',
-                                    source: 'buildings',
-                                    paint: {
-                                        'fill-extrusion-color': '#8899aa',
-                                        'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
-                                        'fill-extrusion-base': 0,
-                                        'fill-extrusion-opacity': 0.8
-                                    }
-                                });
+                    // Add event listeners for dynamic loading
+                    map.on('moveend', debouncedFetch);
+                    map.on('zoomend', debouncedFetch);
 
-                                // Fit map to buildings bounds
-                                var coordinates = [];
-                                data.features.forEach(function(f) {
-                                    if (f.geometry && f.geometry.coordinates && f.geometry.coordinates[0]) {
-                                        var coords = f.geometry.coordinates[0];
-                                        coords.forEach(function(c) {
-                                            coordinates.push(c);
-                                        });
-                                    }
-                                });
-                                var bounds = coordinates.reduce(function(bounds, coord) {
-                                    return bounds.extend(coord);
-                                }, new maplibregl.LngLatBounds(coordinates[0], coordinates[0]));
-
-                                map.fitBounds(bounds, {
-                                    padding: 50,
-                                    pitch: 60
-                                });
+                    // Initial fetch for starting viewport
+                    debouncedFetch();
+                }});
 
-                                console.log('Loaded ' + data.features.length + ' buildings from 3DBAG');
-                                console.log('Map bounds fitted to buildings');
-                            } else {
-                                console.warn('No buildings found');
-                            }
-                        })
-                        .catch(err => console.error('Failed to load buildings:', err));
-                });
-
-                map.on('error', function(e) {
+                map.on('error', function(e) {{
                     console.error('Map error:', e);
-                });
+                }});
 
-            } catch (e) {
+            }} catch (e) {{
                 console.error('Error creating map:', e);
-            }
-        }
+            }}
+        }}
 
-        waitForElement('map', function() {
+        waitForElement('map', function() {{
             console.log('Map container found, initializing...');
             initMap3D();
-        });
-    })();
-    "#.to_string()
+        }});
+    }})();
+    "#
+    )
 }
 
 #[component]
@@ -400,3 +473,43 @@ pub fn DataVerkenner() -> Element {
         }
     }
 }
+
+#[cfg(test)]
+mod test_frontend_loading {
+    use super::*;
+
+    #[test]
+    fn test_fetch_script_includes_wgs84_endpoint() {
+        let script = build_buildings_fetch_script();
+        assert!(script.contains("bbox-wgs84"),
+                "Fetch script should use WGS84 bbox parameter");
+    }
+
+    #[test]
+    fn test_fetch_script_includes_debounce() {
+        let script = build_buildings_fetch_script();
+        assert!(script.contains("setTimeout") && script.contains("300"),
+                "Fetch script should include 300ms debounce");
+    }
+
+    #[test]
+    fn test_fetch_script_includes_state_variables() {
+        let script = build_buildings_fetch_script();
+        assert!(script.contains("lastFetchedBbox") && script.contains("fetchTimeout"),
+                "Fetch script should initialize state variables");
+    }
+
+    #[test]
+    fn test_fetch_script_includes_threshold_check() {
+        let script = build_buildings_fetch_script();
+        assert!(script.contains("0.1") || script.contains("10%"),
+                "Fetch script should include 10% threshold check");
+    }
+
+    #[test]
+    fn test_fetch_script_includes_error_handling() {
+        let script = build_buildings_fetch_script();
+        assert!(script.contains("catch") && script.contains("error"),
+                "Fetch script should include error handling");
+    }
+}
