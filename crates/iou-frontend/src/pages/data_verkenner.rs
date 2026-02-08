//! Data Verkenner app

use dioxus::prelude::*;

use crate::components::{Header, Panel};

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

#[component]
pub fn DataVerkenner() -> Element {
    let mut selected = use_signal(|| 0usize);

    // Initialize Leaflet map with local GeoJSON from Kaartportaal Flevoland
    use_effect(move || {
        let script = r#"
            (function() {
                var el = document.getElementById('map');
                if (!el || el._leaflet_id) return;
                el.innerHTML = '';

                var map = L.map('map').setView([52.45, 5.50], 10);

                var osm = L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                    attribution: '&copy; OpenStreetMap'
                }).addTo(map);

                // Layer groups for the control
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
                    id: "map",
                    style: "height: 550px; border-radius: 8px;",
                }
            }
        }
    }
}
