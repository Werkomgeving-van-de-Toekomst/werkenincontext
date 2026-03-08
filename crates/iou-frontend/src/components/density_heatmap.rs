//! Density Heatmap Component
//!
//! Toggle button for enabling/disabling building density visualization.
//! Uses client-side grid aggregation with buffered bbox to avoid tile seams.

use dioxus::prelude::*;
use crate::state::DensityHeatmap as DensityHeatmapState;

/// LocalStorage key for persisting heatmap enabled state
const STORAGE_KEY: &'static str = "densityHeatmapEnabled";

/// Grid cell size in meters for density aggregation
const CELL_SIZE_METERS: f64 = 100.0;

/// Buffer size in meters to prevent tile boundary artifacts
const BUFFER_METERS: f64 = 50.0;

/// Builds JavaScript to add or update the heatmap layer on the map
///
/// # Arguments
/// * `enabled` - Whether the heatmap should be visible
///
/// # Returns
/// JavaScript code that creates/updates the MapLibre heatmap layer
pub fn build_add_heatmap_layer_script(enabled: bool) -> String {
    format!(
        r#"
        (function() {{
            const map = window['map_map'];
            if (!map) {{
                console.error('Map not found for heatmap layer');
                return;
            }}

            try {{
                const sourceId = 'density-points';
                const layerId = 'density-heatmap';

                if ({enabled}) {{
                    // Create source if it doesn't exist
                    if (!map.getSource(sourceId)) {{
                        map.addSource(sourceId, {{
                            type: 'geojson',
                            data: {{ type: 'FeatureCollection', features: [] }}
                        }});
                    }}

                    // Add heatmap layer if it doesn't exist
                    if (!map.getLayer(layerId)) {{
                        map.addLayer({{
                            id: layerId,
                            type: 'heatmap',
                            source: sourceId,
                            paint: {{
                                'heatmap-color': [
                                    'interpolate',
                                    ['linear'],
                                    ['heatmap-density'],
                                    0, 'rgba(230, 240, 255, 0)',
                                    0.2, 'rgba(100, 181, 246, 0.5)',
                                    0.4, 'rgba(33, 150, 243, 0.7)',
                                    0.6, 'rgba(156, 39, 176, 0.8)',
                                    0.8, 'rgba(74, 20, 140, 0.9)'
                                ],
                                'heatmap-radius': 25,
                                'heatmap-weight': ['get', 'weight'],
                                'heatmap-opacity': 0.7
                            }}
                        }}, 'building-3d');  // Insert before building layer
                    }}

                    // Make layer visible
                    map.setLayoutProperty(layerId, 'visibility', 'visible');

                    // Persist to localStorage
                    localStorage.setItem('{storage_key}', 'true');

                    console.log('Density heatmap layer enabled');

                }} else {{
                    // Hide the layer
                    if (map.getLayer(layerId)) {{
                        map.setLayoutProperty(layerId, 'visibility', 'none');
                    }}

                    // Persist disabled state
                    localStorage.setItem('{storage_key}', 'false');

                    console.log('Density heatmap layer disabled');
                }}
            }} catch (e) {{
                console.error('Heatmap layer error:', e);
            }}
        }})();
        "#,
        enabled = enabled,
        storage_key = STORAGE_KEY
    )
}

/// Builds JavaScript to calculate density from buildings in the viewport
///
/// # Arguments
/// * `bbox_str` - Bounding box as "minLon,minLat,maxLon,maxLat"
///
/// # Returns
/// JavaScript code that fetches buildings and aggregates to density grid
pub fn build_density_calculation_script(bbox_str: String) -> String {
    format!(
        r#"
        (function() {{
            const map = window['map_map'];
            if (!map) {{
                console.error('Map not found for density calculation');
                return;
            }}

            try {{
                // Parse bbox
                const bbox = {bbox_str}.split(',').map(Number);
                const [minLon, minLat, maxLon, maxLat] = bbox;

                // Calculate buffered bbox to prevent tile seams
                // Buffer: ~50 meters at Dutch latitudes (~0.00045 degrees)
                const buffer = {buffer_degrees};
                const bufferedBbox = [
                    minLon - buffer,
                    minLat - buffer,
                    maxLon + buffer,
                    maxLat + buffer
                ].join(',');

                // Fetch buildings from API
                fetch(`/api/buildings-3d?bbox-wgs84=${{bufferedBbox}}&limit=500`)
                    .then(response => {{
                        if (!response.ok) {{
                            throw new Error(`API error: ${{response.status}}`);
                        }}
                        return response.json();
                    }})
                    .then(data => {{
                        // Grid aggregation
                        const cellSize = {cell_size_degrees};
                        const grid = {{}};

                        data.features.forEach(feature => {{
                            let point = null;

                            // Extract centroid from building geometry
                            if (feature.geometry?.type === 'Polygon') {{
                                const coords = feature.geometry.coordinates[0];
                                const lon = coords.reduce((sum, c) => sum + c[0], 0) / coords.length;
                                const lat = coords.reduce((sum, c) => sum + c[1], 0) / coords.length;
                                point = {{ lon, lat }};
                            }} else if (feature.geometry?.type === 'MultiPolygon') {{
                                // For multipolygons, use the first polygon's centroid
                                const coords = feature.geometry.coordinates[0][0];
                                const lon = coords.reduce((sum, c) => sum + c[0], 0) / coords.length;
                                const lat = coords.reduce((sum, c) => sum + c[1], 0) / coords.length;
                                point = {{ lon, lat }};
                            }}

                            if (point) {{
                                // Calculate grid cell
                                const cellX = Math.floor((point.lon - minLon) / cellSize);
                                const cellY = Math.floor((point.lat - minLat) / cellSize);
                                const key = `${{cellX}},${{cellY}}`;

                                // Increment building count for this cell
                                grid[key] = (grid[key] || 0) + 1;
                            }}
                        }});

                        // Convert grid to GeoJSON points
                        const densityPoints = Object.entries(grid)
                            .filter(([key]) => {{
                                // Only include cells whose center is within original bbox (not buffered)
                                const [cx, cy] = key.split(',').map(Number);
                                const centerLon = minLon + (cx + 0.5) * cellSize;
                                const centerLat = minLat + (cy + 0.5) * cellSize;
                                return centerLon >= minLon && centerLon <= maxLon &&
                                       centerLat >= minLat && centerLat <= maxLat;
                            }})
                            .map(([key, count]) => {{
                                const [cx, cy] = key.split(',').map(Number);
                                return {{
                                    type: 'Feature',
                                    geometry: {{
                                        type: 'Point',
                                        coordinates: [
                                            minLon + (cx + 0.5) * cellSize,
                                            minLat + (cy + 0.5) * cellSize
                                        ]
                                    }},
                                    properties: {{ weight: count }}
                                }};
                            }});

                        // Update heatmap source
                        const source = map.getSource('density-points');
                        if (source) {{
                            source.setData({{
                                type: 'FeatureCollection',
                                features: densityPoints
                            }});
                            console.log(`Density calculated: ${{densityPoints.length}} grid cells from ${{data.features.length}} buildings`);
                        }}
                    }})
                    .catch(error => {{
                        console.error('Failed to fetch density data:', error);
                    }});
            }} catch (e) {{
                console.error('Density calculation error:', e);
            }}
        }})();
        "#,
        bbox_str = bbox_str,
        buffer_degrees = 0.00045,  // ~50 meters at Dutch latitudes
        cell_size_degrees = 0.0009   // ~100 meters at Dutch latitudes
    )
}

/// Builds JavaScript to setup viewport event listeners for density updates
///
/// # Returns
/// JavaScript code that registers moveend/zoomend event listeners with debouncing
pub fn build_setup_density_update_script() -> String {
    format!(
        r#"
        (function() {{
            const map = window['map_map'];
            if (!map) {{
                console.error('Map not found for density event setup');
                return;
            }}

            try {{
                // State for debounced density updates
                let densityLastFetchedBbox = null;
                let densityFetchTimeout = null;
                let densityAbortController = null;

                // Check if bounds changed significantly (10% threshold)
                function shouldFetchDensity(newBounds) {{
                    if (!densityLastFetchedBbox) return true;

                    const width = newBounds[2] - newBounds[0];
                    const lastWidth = densityLastFetchedBbox[2] - densityLastFetchedBbox[0];
                    const height = newBounds[3] - newBounds[1];
                    const lastHeight = densityLastFetchedBbox[3] - densityLastFetchedBbox[1];

                    if (lastWidth === 0 || lastHeight === 0) return true;

                    const widthChange = Math.abs(width - lastWidth) / lastWidth;
                    const heightChange = Math.abs(height - lastHeight) / lastHeight;

                    return widthChange > 0.1 || heightChange > 0.1;
                }}

                // Debounced density update function
                function debouncedDensityUpdate() {{
                    clearTimeout(densityFetchTimeout);
                    densityFetchTimeout = setTimeout(() => {{
                        // Only update if heatmap is enabled
                        const isEnabled = localStorage.getItem('{storage_key}') === 'true';
                        if (!isEnabled) {{
                            return;
                        }}

                        const bounds = map.getBounds();
                        const bbox = [
                            bounds.getWest(),
                            bounds.getSouth(),
                            bounds.getEast(),
                            bounds.getNorth()
                        ];

                        if (shouldFetchDensity(bbox)) {{
                            // Calculate density for new viewport
                            const bboxStr = bbox.join(',');

                            // Cancel any pending density fetch
                            if (densityAbortController) {{
                                densityAbortController.abort();
                            }}

                            densityAbortController = new AbortController();

                            fetch(`/api/buildings-3d?bbox-wgs84=${{bboxStr}}&limit=500`, {{
                                signal: densityAbortController.signal
                            }})
                            .then(response => response.json())
                            .then(data => {{
                                // Update last fetched bbox after successful fetch
                                densityLastFetchedBbox = bbox;

                                // Process data and update heatmap...
                                const minLon = bbox[0];
                                const minLat = bbox[1];
                                const cellSize = 0.0009;  // ~100m
                                const buffer = 0.00045;   // ~50m
                                const bufferedBbox = [
                                    minLon - buffer,
                                    minLat - buffer,
                                    bbox[2] + buffer,
                                    bbox[3] + buffer
                                ].join(',');

                                // Re-fetch with buffer for seamless density
                                return fetch(`/api/buildings-3d?bbox-wgs84=${{bufferedBbox}}&limit=500`);
                            }})
                            .then(response => response.json())
                            .then(data => {{
                                // Grid aggregation (same as calculation script)
                                const grid = {{}};
                                const cellSize = 0.0009;

                                data.features.forEach(feature => {{
                                    let point = null;
                                    if (feature.geometry?.type === 'Polygon') {{
                                        const coords = feature.geometry.coordinates[0];
                                        const lon = coords.reduce((sum, c) => sum + c[0], 0) / coords.length;
                                        const lat = coords.reduce((sum, c) => sum + c[1], 0) / coords.length;
                                        point = {{ lon, lat }};
                                    }}

                                    if (point) {{
                                        const cellX = Math.floor((point.lon - minLon) / cellSize);
                                        const cellY = Math.floor((point.lat - minLat) / cellSize);
                                        const key = `${{cellX}},${{cellY}}`;
                                        grid[key] = (grid[key] || 0) + 1;
                                    }}
                                }});

                                const densityPoints = Object.entries(grid)
                                    .filter(([key]) => {{
                                        const [cx, cy] = key.split(',').map(Number);
                                        const centerLon = minLon + (cx + 0.5) * cellSize;
                                        const centerLat = minLat + (cy + 0.5) * cellSize;
                                        return centerLon >= bbox[0] && centerLon <= bbox[2] &&
                                               centerLat >= bbox[1] && centerLat <= bbox[3];
                                    }})
                                    .map(([key, count]) => {{
                                        const [cx, cy] = key.split(',').map(Number);
                                        return {{
                                            type: 'Feature',
                                            geometry: {{
                                                type: 'Point',
                                                coordinates: [
                                                    minLon + (cx + 0.5) * cellSize,
                                                    minLat + (cy + 0.5) * cellSize
                                                ]
                                            }},
                                            properties: {{ weight: count }}
                                        }};
                                    }});

                                const source = map.getSource('density-points');
                                if (source) {{
                                    source.setData({{
                                        type: 'FeatureCollection',
                                        features: densityPoints
                                    }});
                                }}
                            }})
                            .catch(error => {{
                                if (error.name !== 'AbortError') {{
                                    console.error('Density update error:', error);
                                }}
                            }});
                        }}
                    }}, 300);  // 300ms debounce (same as buildings fetch)
                }}

                // Register event listeners for viewport changes
                map.on('moveend', debouncedDensityUpdate);
                map.on('zoomend', debouncedDensityUpdate);

                // Initial density calculation if enabled
                const isEnabled = localStorage.getItem('{storage_key}') === 'true';
                if (isEnabled) {{
                    debouncedDensityUpdate();
                }}

                console.log('Density heatmap event listeners registered');
            }} catch (e) {{
                console.error('Density event setup error:', e);
            }}
        }})();
        "#,
        storage_key = STORAGE_KEY
    )
}

/// Density Heatmap Toggle Button Component
///
/// Renders a button that enables/disables the building density heatmap overlay.
/// The heatmap shows building density as a color gradient from light blue (low)
/// to dark purple (high).
#[component]
pub fn DensityHeatmap() -> Element {
    let mut heatmap = use_signal(|| DensityHeatmapState::default());

    // Initialize from localStorage on mount
    use_effect(move || {
        let sync_script = format!(
            r#"
            (function() {{
                try {{
                    const stored = localStorage.getItem('{}');
                    if (stored === 'true') {{
                        // Set enabled state
                        const map = window['map_map'];
                        if (map) {{
                            // Trigger heatmap enable
                        }}
                    }}
                }} catch (e) {{
                    console.error('Error restoring heatmap state:', e);
                }}
            }})();
            "#,
            STORAGE_KEY
        );
        document::eval(&sync_script);
    });

    let current_state = *heatmap.read();

    rsx! {
        div {
            class: "density-heatmap-container",
            style: "position: absolute; top: 10px; right: 10px; z-index: 1000;",

            button {
                class: "btn-density-heatmap",
                style: "background: white; padding: 8px 12px; border: 1px solid #ccc;
                        border-radius: 4px; cursor: pointer; font-size: 14px;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.1);",
                onclick: move |_| {
                    heatmap.write().toggle();
                    let new_state = *heatmap.read();

                    // Toggle heatmap layer
                    let layer_script = build_add_heatmap_layer_script(new_state.is_enabled());
                    document::eval(&layer_script);

                    // If enabling, setup viewport event listeners and trigger initial calculation
                    if new_state.is_enabled() {
                        let setup_script = build_setup_density_update_script();
                        document::eval(&setup_script);
                    }
                },
                "Densiteitskaart"
            }

            // Current state indicator
            div {
                style: "margin-top: 4px; font-size: 11px; color: #666; text-align: center;",
                "{current_state.label()}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_add_heatmap_layer_script_contains_heatmap_type() {
        let script = build_add_heatmap_layer_script(true);
        assert!(script.contains("type: 'heatmap'"),
                "Script should create a heatmap layer");
    }

    #[test]
    fn test_build_add_heatmap_layer_script_includes_interpolate_color_expression() {
        let script = build_add_heatmap_layer_script(true);
        assert!(script.contains("'heatmap-color'"),
                "Script should set heatmap-color property");
        assert!(script.contains("'interpolate'"),
                "Script should use interpolate expression for color gradient");
    }

    #[test]
    fn test_build_add_heatmap_layer_script_uses_visibility_none_when_disabled() {
        let script = build_add_heatmap_layer_script(false);
        assert!(script.contains("'visibility', 'none'"),
                "Disabled heatmap should use visibility: none");
    }

    #[test]
    fn test_build_add_heatmap_layer_script_uses_visibility_visible_when_enabled() {
        let script = build_add_heatmap_layer_script(true);
        assert!(script.contains("'visibility', 'visible'"),
                "Enabled heatmap should use visibility: visible");
    }

    #[test]
    fn test_build_density_calculation_script_includes_buffered_bbox() {
        let script = build_density_calculation_script("5.0,52.0,5.5,52.5".to_string());
        assert!(script.contains("buffer"),
                "Script should calculate buffered bbox to prevent tile seams");
        assert!(script.contains("0.00045") || script.contains("bufferedBbox"),
                "Script should use ~50m buffer (~0.00045 degrees)");
    }

    #[test]
    fn test_build_density_calculation_script_fetches_from_buildings_3d_endpoint() {
        let script = build_density_calculation_script("5.0,52.0,5.5,52.5".to_string());
        assert!(script.contains("/api/buildings-3d"),
                "Script should fetch from buildings-3d endpoint");
        assert!(script.contains("bbox-wgs84"),
                "Script should use bbox-wgs84 parameter");
    }

    #[test]
    fn test_build_setup_density_update_script_registers_moveend_listener() {
        let script = build_setup_density_update_script();
        assert!(script.contains("'moveend'") || script.contains("\"moveend\""),
                "Script should register moveend event listener");
    }

    #[test]
    fn test_build_setup_density_update_script_registers_zoomend_listener() {
        let script = build_setup_density_update_script();
        assert!(script.contains("'zoomend'") || script.contains("\"zoomend\""),
                "Script should register zoomend event listener");
    }

    #[test]
    fn test_build_setup_density_update_script_has_debounced_function() {
        let script = build_setup_density_update_script();
        assert!(script.contains("setTimeout"),
                "Script should use setTimeout for debouncing");
        assert!(script.contains("300"),
                "Script should use 300ms debounce delay");
    }

    #[test]
    fn test_build_add_heatmap_layer_script_includes_paint_properties() {
        let script = build_add_heatmap_layer_script(true);
        assert!(script.contains("'heatmap-radius'"),
                "Script should set heatmap-radius property");
        assert!(script.contains("'heatmap-weight'"),
                "Script should set heatmap-weight property");
        assert!(script.contains("'heatmap-opacity'"),
                "Script should set heatmap-opacity property");
    }

    #[test]
    fn test_heatmap_color_gradient_includes_expected_colors() {
        let script = build_add_heatmap_layer_script(true);
        // Light blue for low density
        assert!(script.contains("100, 181, 246") || script.contains("230, 240, 255"),
                "Script should include light blue colors");
        // Dark purple for high density
        assert!(script.contains("156, 39, 176") || script.contains("74, 20, 140"),
                "Script should include dark purple colors");
    }
}
