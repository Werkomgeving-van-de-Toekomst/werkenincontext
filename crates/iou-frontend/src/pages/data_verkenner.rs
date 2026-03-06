//! Data Verkenner app

use dioxus::prelude::*;
use std::env;

use crate::components::{Header, Panel};
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

/// Returns the Map3D initialization script.
fn get_map3d_init_script() -> String {
    r#"
    (function() {
        console.log('Map3D script loaded');

        function waitForElement(id, callback) {
            var element = document.getElementById(id);
            if (element) {
                callback(element);
            } else {
                setTimeout(function() { waitForElement(id, callback); }, 100);
            }
        }

        function initMap3D() {
            if (typeof window.maplibregl === 'undefined') {
                console.log('MapLibre GL not loaded yet, waiting...');
                setTimeout(initMap3D, 200);
                return;
            }

            if (window.map_map) {
                console.log('Map already initialized');
                return;
            }

            console.log('Initializing Map3D...');

            try {
                var map = new maplibregl.Map({
                    container: 'map',
                    style: 'https://basemaps.cartocdn.com/gl/positron-gl-style/style.json',
                    center: [5.6, 52.4],
                    zoom: 10,
                    pitch: 60,
                    bearing: 0,
                    antialias: true
                });

                map.addControl(new maplibregl.NavigationControl({ visualizePitch: true }));
                window.map_map = map;

                map.on('load', function() {
                    console.log('Map loaded, fetching buildings...');

                    fetch('/api/buildings-3d?bbox=150000,470000,170000,490000&limit=100')
                        .then(r => {
                            console.log('Buildings fetch status:', r.status, r.statusText);
                            if (!r.ok) throw new Error('Failed to fetch: ' + r.status + ' ' + r.statusText);
                            return r.json();
                        })
                        .then(data => {
                            console.log('Buildings response:', data);

                            if (data.features && data.features.length > 0) {
                                // Log first building coordinates for debugging
                                console.log('First building coordinates:', data.features[0].geometry.coordinates);

                                map.addSource('buildings', {
                                    type: 'geojson',
                                    data: data
                                });

                                map.addLayer({
                                    id: 'building-3d',
                                    type: 'fill-extrusion',
                                    source: 'buildings',
                                    paint: {
                                        'fill-extrusion-color': '#8899aa',
                                        'fill-extrusion-height': ['coalesce', ['get', 'height'], 10],
                                        'fill-extrusion-base': 0,
                                        'fill-extrusion-opacity': 0.8
                                    }
                                });

                                // Fit map to buildings bounds
                                var coordinates = [];
                                data.features.forEach(function(f) {
                                    if (f.geometry && f.geometry.coordinates && f.geometry.coordinates[0]) {
                                        var coords = f.geometry.coordinates[0];
                                        coords.forEach(function(c) {
                                            coordinates.push(c);
                                        });
                                    }
                                });
                                var bounds = coordinates.reduce(function(bounds, coord) {
                                    return bounds.extend(coord);
                                }, new maplibregl.LngLatBounds(coordinates[0], coordinates[0]));

                                map.fitBounds(bounds, {
                                    padding: 50,
                                    pitch: 60
                                });

                                console.log('Loaded ' + data.features.length + ' buildings from 3DBAG');
                                console.log('Map bounds fitted to buildings');
                            } else {
                                console.warn('No buildings found');
                            }
                        })
                        .catch(err => console.error('Failed to load buildings:', err));
                });

                map.on('error', function(e) {
                    console.error('Map error:', e);
                });

            } catch (e) {
                console.error('Error creating map:', e);
            }
        }

        waitForElement('map', function() {
            console.log('Map container found, initializing...');
            initMap3D();
        });
    })();
    "#.to_string()
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
                        LayerControl3D {
                            layers: predefined_layers(),
                            map_id: "map".to_string(),
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
