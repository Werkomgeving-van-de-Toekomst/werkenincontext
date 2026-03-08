//! Data Verkenner app

use dioxus::prelude::*;
use std::env;

use crate::components::{Header, Panel, FilterPanel3D, ViewToggle, DensityHeatmap};
use crate::components::{LayerControl3D, predefined_layers};

/// Mock dataset metadata
struct DatasetInfo {
    name: &'static str,
    records: &'static str,
    last_update: &'static str,
    source: &'static str,
    bars: &'static [(&'static str, u32)],
}

const DATASETS: &[DatasetInfo] = &[
    DatasetInfo {
        name: "Verkeersintensiteit Flevoland",
        records: "12.847",
        last_update: "2025-01-15",
        source: "CBS",
        bars: &[
            ("Almere", 85),
            ("Lelystad", 62),
            ("Dronten", 45),
            ("Zeewolde", 28),
            ("Urk", 38),
        ],
    },
    DatasetInfo {
        name: "Archeologische vindplaatsen",
        records: "3.241",
        last_update: "2024-11-20",
        source: "RCE",
        bars: &[
            ("Almere", 42),
            ("Lelystad", 78),
            ("Dronten", 55),
            ("Zeewolde", 65),
            ("Urk", 30),
        ],
    },
    DatasetInfo {
        name: "CBS Bevolkingsstatistiek",
        records: "45.920",
        last_update: "2025-02-01",
        source: "CBS",
        bars: &[
            ("Almere", 95),
            ("Lelystad", 40),
            ("Dronten", 22),
            ("Zeewolde", 12),
            ("Urk", 10),
        ],
    },
    DatasetInfo {
        name: "Energieverbruik gemeenten",
        records: "8.156",
        last_update: "2024-12-10",
        source: "RVO",
        bars: &[
            ("Almere", 72),
            ("Lelystad", 58),
            ("Dronten", 68),
            ("Zeewolde", 35),
            ("Urk", 48),
        ],
    },
    DatasetInfo {
        name: "Natuurnetwerk Nederland",
        records: "1.893",
        last_update: "2025-01-05",
        source: "BIJ12",
        bars: &[
            ("Almere", 25),
            ("Lelystad", 50),
            ("Dronten", 70),
            ("Zeewolde", 90),
            ("Urk", 15),
        ],
    },
];

/// Checks if the 3D map is enabled.
fn is_3d_map_enabled() -> bool {
    #[cfg(feature = "web")]
    {
        true
    }
    #[cfg(not(feature = "web"))]
    {
        std::env::var("MAP_3D_ENABLED")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }
}

/// Builds the JavaScript fetch script for dynamic building loading.
fn build_buildings_fetch_script() -> String {
    r#"
        // Constants
        const BUILDINGS_FETCH_LIMIT = 150;

        // State tracking for dynamic loading
        let lastFetchedBbox = null;
        let fetchTimeout = null;
        let abortController = null;

        // Check if bounds changed by at least 10% threshold
        // Also checks if center position moved significantly (pan detection)
        function shouldFetch(newBounds) {
            if (!lastFetchedBbox) return true;

            const width = newBounds[2] - newBounds[0];
            const lastWidth = lastFetchedBbox[2] - lastFetchedBbox[0];
            const height = newBounds[3] - newBounds[1];
            const lastHeight = lastFetchedBbox[3] - lastFetchedBbox[1];

            // Handle zero dimensions
            if (lastWidth === 0 || lastHeight === 0) return true;

            // Calculate centers to detect panning
            const newCenterX = (newBounds[0] + newBounds[2]) / 2;
            const newCenterY = (newBounds[1] + newBounds[3]) / 2;
            const lastCenterX = (lastFetchedBbox[0] + lastFetchedBbox[2]) / 2;
            const lastCenterY = (lastFetchedBbox[1] + lastFetchedBbox[3]) / 2;

            // Check if center moved more than 10% of viewport size
            const centerMoveX = Math.abs(newCenterX - lastCenterX) / lastWidth;
            const centerMoveY = Math.abs(newCenterY - lastCenterY) / lastHeight;

            const widthChange = Math.abs(width - lastWidth) / lastWidth;
            const heightChange = Math.abs(height - lastHeight) / lastHeight;

            return widthChange > 0.1 || heightChange > 0.1 || centerMoveX > 0.1 || centerMoveY > 0.1;
        }

        // Update buildings source with new GeoJSON data
        function updateBuildingsSource(geojson) {
            // Clear any existing popups
            document.querySelectorAll('.maplibregl-popup').forEach(p => p.remove());

            const source = map.getSource('buildings');
            if (source) {
                source.setData(geojson);
            } else {
                // Source doesn't exist yet, create it
                map.addSource('buildings', {
                    type: 'geojson',
                    data: geojson
                });

                map.addLayer({
                    id: 'building-3d',
                    type: 'fill-extrusion',
                    source: 'buildings',
                    paint: {
                        'fill-extrusion-color': [
                            'step',
                            ['coalesce', ['get', 'height'], 0],
                            '#64B5F6',  // 0-5m: Light blue
                            5,
                            '#9B59B6',  // 5-15m: Medium purple
                            15,
                            '#8E44AD'   // 15m+: Dark purple
                        ],
                        'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
                        'fill-extrusion-base': 0,
                        'fill-extrusion-opacity': 0.8
                    }
                });
            }
        }

        // Show empty state when no buildings found
        function showNoBuildingsMessage() {
            const source = map.getSource('buildings');
            if (source) {
                source.setData({
                    type: 'FeatureCollection',
                    features: []
                });
            }
        }

        // Show error message
        function showErrorMessage(message) {
            console.warn(message);
        }

        // Fetch buildings from API with WGS84 bbox
        async function fetchBuildings(bbox) {
            // Cancel any pending fetch
            if (abortController) {
                abortController.abort();
            }

            // Create new abort controller for this fetch
            abortController = new AbortController();

            try {
                const response = await fetch(
                    `/api/buildings-3d?bbox-wgs84=${bbox.join(',')}&limit=${BUILDINGS_FETCH_LIMIT}`,
                    { signal: abortController.signal }
                );

                if (!response.ok) {
                    if (response.status === 404) {
                        showNoBuildingsMessage();
                        return;
                    }
                    throw new Error(`API error: ${response.status}`);
                }

                const data = await response.json();

                if (data.features.length === 0) {
                    showNoBuildingsMessage();
                    return;
                }

                updateBuildingsSource(data);
                // Update state AFTER successful fetch (fixes race condition)
                lastFetchedBbox = bbox;
                console.log('Loaded ' + data.features.length + ' buildings from 3DBAG');

            } catch (error) {
                if (error.name === 'AbortError') {
                    console.log('Fetch aborted due to new request');
                    return;
                }
                console.error('Failed to fetch buildings:', error);
                showErrorMessage('Kon gebouwen niet laden. Probeer het opnieuw.');
            } finally {
                abortController = null;
            }
        }

        // Debounced fetch function
        function debouncedFetch() {
            clearTimeout(fetchTimeout);
            fetchTimeout = setTimeout(() => {
                const bounds = map.getBounds();
                const bbox = [bounds.getWest(), bounds.getSouth(),
                              bounds.getEast(), bounds.getNorth()];

                if (shouldFetch(bbox)) {
                    fetchBuildings(bbox);
                    // Note: lastFetchedBbox is now updated inside fetchBuildings
                    // after successful fetch to prevent race conditions
                }
            }, 300);
        }
    "#.to_string()
}

/// Builds the JavaScript popup handler script for building click interactions.
fn build_popup_handler_script() -> String {
    r#"
        // Create popup content using XSS-safe DOM methods
        function createBuildingPopup(props) {
            var container = document.createElement('div');
            container.className = 'building-popup';

            // Title
            var title = document.createElement('h3');
            title.textContent = 'Gebouw Info';
            container.appendChild(title);

            // Helper function for adding rows (XSS-safe)
            function addRow(label, value) {
                if (value === null || value === undefined) return;
                var row = document.createElement('p');
                var labelEl = document.createElement('strong');
                labelEl.textContent = label + ': ';
                row.appendChild(labelEl);
                row.appendChild(document.createTextNode(String(value)));
                container.appendChild(row);
            }

            // Add property rows
            addRow('ID', props.bag_id);
            addRow('Hoogte', props.height !== undefined ? props.height.toFixed(1) + 'm' : null);
            addRow('Verdiepingen', props.floors);
            addRow('Bouwjaar', props.construction_year);

            return new maplibregl.Popup({ offset: 15 }).setDOMContent(container);
        }

        // Register click handler for buildings
        map.on('click', 'building-3d', function(e) {
            var features = map.queryRenderedFeatures(e.point, {
                layers: ['building-3d']
            });

            if (features.length > 0 && features[0].properties) {
                var props = features[0].properties;
                var popup = createBuildingPopup(props);
                popup.setLngLat(e.lngLat).addTo(map);
            }
        });

        // Close popup when map moves
        map.on('move', function() {
            var popups = document.querySelectorAll('.maplibregl-popup');
            popups.forEach(function(p) { p.remove(); });
        });
    "#.to_string()
}

/// Returns the Map3D initialization script.
fn get_map3d_init_script() -> String {
    let fetch_script = build_buildings_fetch_script();
    let popup_handler_script = build_popup_handler_script();
    format!(
        r#"
    (function() {{
        console.log('Map3D script loaded');

        {fetch_script}

        function waitForElement(id, callback) {{
            var element = document.getElementById(id);
            if (element) {{
                callback(element);
            }} else {{
                setTimeout(function() {{ waitForElement(id, callback); }}, 100);
            }}
        }}

        function initMap3D() {{
            if (typeof window.maplibregl === 'undefined') {{
                console.log('MapLibre GL not loaded yet, waiting...');
                setTimeout(initMap3D, 200);
                return;
            }}

            if (window.map_map) {{
                console.log('Map already initialized');
                return;
            }}

            console.log('Initializing Map3D...');

            try {{
                var map = new maplibregl.Map({{
                    container: 'map',
                    style: 'https://basemaps.cartocdn.com/gl/positron-gl-style/style.json',
                    center: [5.6, 52.4],
                    zoom: 10,
                    pitch: 60,
                    bearing: 0,
                    antialias: true
                }});

                map.addControl(new maplibregl.NavigationControl({{ visualizePitch: true }}));
                window.map_map = map;

                map.on('load', function() {{
                    console.log('Map loaded, setting up dynamic building loading...');

                    // Add event listeners for dynamic loading
                    map.on('moveend', debouncedFetch);
                    map.on('zoomend', debouncedFetch);

                    // Initial fetch for starting viewport
                    debouncedFetch();

                    // Setup popup handler after buildings are loaded
                    {popup_handler_script}
                }});

                map.on('error', function(e) {{
                    console.error('Map error:', e);
                }});

            }} catch (e) {{
                console.error('Error creating map:', e);
            }}
        }}

        waitForElement('map', function() {{
            console.log('Map container found, initializing...');
            initMap3D();
        }});
    }})();
    "#
    )
}

#[component]
pub fn DataVerkenner() -> Element {
    let mut selected = use_signal(|| 0usize);
    let use_3d_map = is_3d_map_enabled();

    // Initialize Leaflet map when 3D is NOT enabled
    use_effect(move || {
        if use_3d_map {
            return;
        }

        let script = r#"
            (function() {
                var el = document.getElementById('map');
                if (!el || el._leaflet_id) return;
                el.innerHTML = '';

                var map = L.map('map').setView([52.45, 5.50], 10);

                var osm = L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                    attribution: '&copy; OpenStreetMap'
                }).addTo(map);

                var grensLayer = L.layerGroup().addTo(map);
                var cultuurLayer = L.layerGroup().addTo(map);
                var windLayer = L.layerGroup();
                var zonLayer = L.layerGroup();
                var fietsLayer = L.layerGroup();
                var waterLayer = L.layerGroup();

                L.control.layers(
                    { 'OpenStreetMap': osm },
                    {
                        'Provinciegrens': grensLayer,
                        'Cultuurhistorische waarden': cultuurLayer,
                        'Windturbines': windLayer,
                        'Zonneparken': zonLayer,
                        'Fietsnetwerken': fietsLayer,
                        'Drinkwatergebieden': waterLayer
                    },
                    { collapsed: false }
                ).addTo(map);

                function load(file, group, style, popup, pointStyle) {
                    fetch('/assets/geodata/' + file)
                        .then(function(r) { return r.json(); })
                        .then(function(data) {
                            var opts = { style: function() { return style; } };
                            if (pointStyle) {
                                opts.pointToLayer = function(f, ll) {
                                    return L.circleMarker(ll, pointStyle);
                                };
                            }
                            var lyr = L.geoJSON(data, opts);
                            if (popup) {
                                lyr.bindPopup(function(l) {
                                    var p = l.feature.properties;
                                    var name = p.NAAM || p.Naam || p.naam || p.Name || p.STATUS || '';
                                    return '<strong>' + popup + '</strong>' + (name ? '<br>' + name : '');
                                });
                            }
                            group.addLayer(lyr);
                        });
                }

                load('provinciegrens.geojson', grensLayer,
                    { color: '#0066CC', weight: 2, fillOpacity: 0.03 }, null);

                load('cultuurhistorie.geojson', cultuurLayer,
                    {}, 'Cultuurhistorie',
                    { radius: 7, color: '#8B4513', fillColor: '#D2691E', fillOpacity: 0.8, weight: 1 });

                load('windturbines.geojson', windLayer,
                    {}, 'Windturbine',
                    { radius: 6, color: '#1565C0', fillColor: '#42A5F5', fillOpacity: 0.8, weight: 1 });

                load('zonneparken.geojson', zonLayer,
                    { color: '#F57F17', weight: 1, fillColor: '#FFD600', fillOpacity: 0.5 },
                    'Zonnepark');

                load('fietsnetwerken.geojson', fietsLayer,
                    { color: '#7CB342', weight: 3, opacity: 0.7 }, null);

                load('drinkwater.geojson', waterLayer,
                    { color: '#0288D1', weight: 1, fillColor: '#4FC3F7', fillOpacity: 0.35 },
                    'Drinkwatergebied');
            })();
        "#;
        document::eval(script);
    });

    // Initialize Map3D when 3D is enabled
    use_effect(move || {
        if !use_3d_map {
            return;
        }

        let map_script = get_map3d_init_script();
        document::eval(&map_script);
    });

    let idx = *selected.read();
    let dataset = &DATASETS[idx];

    rsx! {
        Header {}
        main { class: "container",
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                Panel { title: "Datasets".to_string(),
                    select {
                        style: "width: 100%; padding: 10px; margin-bottom: 15px;",
                        onchange: move |evt: Event<FormData>| {
                            if let Ok(i) = evt.value().parse::<usize>() {
                                selected.set(i);
                            }
                        },
                        for (i, ds) in DATASETS.iter().enumerate() {
                            option {
                                value: "{i}",
                                selected: i == idx,
                                "{ds.name}"
                            }
                        }
                    }

                    div { class: "compliance-indicator ok",
                        div { class: "icon", "\u{1F4CA}" }
                        div { class: "label", "Totaal records" }
                        div { class: "value", "{dataset.records}" }
                    }
                    div { class: "compliance-indicator ok",
                        div { class: "icon", "\u{1F4C5}" }
                        div { class: "label", "Laatste update" }
                        div { class: "value", "{dataset.last_update}" }
                    }
                    div { class: "compliance-indicator ok",
                        div { class: "icon", "\u{1F3E2}" }
                        div { class: "label", "Bron" }
                        div { class: "value", "{dataset.source}" }
                    }
                }

                Panel { title: "Visualisatie".to_string(),
                    div { class: "chart-container", style: "display: flex; flex-direction: column; justify-content: center; gap: 8px;",
                        for &(label, value) in dataset.bars {
                            div { class: "bar-row",
                                span { class: "bar-label", "{label}" }
                                div { class: "bar-track",
                                    div {
                                        class: "bar-fill",
                                        style: "width: {value}%;",
                                    }
                                }
                                span { class: "bar-value", "{value}%" }
                            }
                        }
                    }
                }
            }

            div { style: "height: 20px;" }

            Panel { title: "Kaart".to_string(),
                div {
                    style: "position: relative;",

                    if use_3d_map {
                        // View toggle button (top-left)
                        ViewToggle {}

                        // Layer controls (top-right)
                        LayerControl3D {
                            layers: predefined_layers(),
                            map_id: "map".to_string(),
                        }

                        // Filter panel for 3D buildings (right, below layer controls)
                        div {
                            style: "position: absolute; top: 60px; right: 10px; z-index: 1000;",
                            FilterPanel3D {}
                        }

                        // Density heatmap toggle (top-right, positioned below layer controls)
                        div {
                            style: "position: absolute; top: 10px; right: 10px; z-index: 1000;",
                            DensityHeatmap {}
                        }
                    }

                    div {
                        id: "map",
                        style: "height: 550px; border-radius: 8px;",
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test_frontend_loading {
    use super::*;

    #[test]
    fn test_fetch_script_includes_wgs84_endpoint() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("bbox-wgs84"),
                "Fetch script should use WGS84 bbox parameter");
    }

    #[test]
    fn test_fetch_script_includes_debounce() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("setTimeout") && script.contains("300"),
                "Fetch script should include 300ms debounce");
    }

    #[test]
    fn test_fetch_script_includes_state_variables() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("lastFetchedBbox") && script.contains("fetchTimeout"),
                "Fetch script should initialize state variables");
    }

    #[test]
    fn test_fetch_script_includes_threshold_check() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("0.1") || script.contains("10%"),
                "Fetch script should include 10% threshold check");
    }

    #[test]
    fn test_fetch_script_includes_error_handling() {
        let script = build_buildings_fetch_script();
        assert!(script.contains("catch") && script.contains("error"),
                "Fetch script should include error handling");
    }
}

#[cfg(test)]
mod test_popup_handler {
    use super::*;

    #[test]
    fn test_popup_script_uses_dom_methods() {
        let script = build_popup_handler_script();

        // Verify XSS-safe methods are used
        assert!(script.contains("textContent"),
                "Popup must use textContent for XSS safety");
    }

    #[test]
    fn test_popup_does_not_use_innerhtml() {
        let script = build_popup_handler_script();

        // Verify innerHTML is NOT used
        assert!(!script.contains("innerHTML"),
                "Popup must NOT use innerHTML for XSS safety");
    }

    #[test]
    fn test_popup_includes_all_properties() {
        let script = build_popup_handler_script();

        // Verify all expected properties are referenced
        assert!(script.contains("bag_id"), "Popup should show BAG ID");
        assert!(script.contains("height"), "Popup should show height");
        assert!(script.contains("floors"), "Popup should show floors");
        assert!(script.contains("construction_year"), "Popup should show construction year");
    }

    #[test]
    fn test_popup_has_correct_css_class() {
        let script = build_popup_handler_script();

        // Verify the CSS class from section-05 is used
        assert!(script.contains("building-popup"),
                "Popup should use the building-popup CSS class");
    }

    #[test]
    fn test_popup_has_click_handler() {
        let script = build_popup_handler_script();

        // Verify click event listener is registered
        assert!(script.contains("'click', 'building-3d'") ||
                script.contains("\"click\", \"building-3d\""),
                "Popup should register click handler on building-3d layer");
    }
}
