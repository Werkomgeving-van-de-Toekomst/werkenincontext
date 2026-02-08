//! Kennisnetwerk - Zuid-Holland relatienetwerk via vis-network

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn ZHKennisnetwerk() -> Element {
    // Initialize vis-network graph after DOM render
    // Note: eval is used here intentionally to bootstrap the vis.js library
    // which requires direct DOM access not available through Dioxus WASM bindings.
    use_effect(move || {
        let script = r#"
            (function() {
                var container = document.getElementById('zh-network-graph');
                if (!container || container._visNetwork) return;
                container.innerHTML = '';
                container._visNetwork = true;

                var nodes = new vis.DataSet([
                    // Beleidsdomeinen (rood, box)
                    { id: 1, label: 'Mobiliteit &\nInfrastructuur', color: '#E31837', font: { color: '#fff' }, shape: 'box', size: 30 },
                    { id: 2, label: 'Haven &\nIndustrie', color: '#E31837', font: { color: '#fff' }, shape: 'box', size: 30 },
                    { id: 3, label: 'Economie &\nInnovatie', color: '#E31837', font: { color: '#fff' }, shape: 'box' },
                    { id: 4, label: 'Energie-\ntransitie', color: '#E31837', font: { color: '#fff' }, shape: 'box' },
                    { id: 5, label: 'Water &\nKustbeheer', color: '#E31837', font: { color: '#fff' }, shape: 'box' },
                    { id: 6, label: 'Wonen &\nLeefomgeving', color: '#E31837', font: { color: '#fff' }, shape: 'box' },

                    // Organisaties (goud/amber, ellipse)
                    { id: 10, label: 'Gemeente\nRotterdam', color: '#DAA520', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 11, label: 'Gemeente\nDen Haag', color: '#DAA520', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 12, label: 'Havenbedrijf\nRotterdam', color: '#DAA520', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 13, label: 'Rijkswater-\nstaat', color: '#DAA520', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 14, label: 'TU Delft', color: '#DAA520', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 15, label: 'MRDH', color: '#DAA520', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 16, label: 'ProRail', color: '#DAA520', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 17, label: 'DCMR', color: '#DAA520', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 18, label: 'Deltalinqs', color: '#DAA520', font: { color: '#fff' }, shape: 'ellipse' },

                    // Projecten (groen, diamond)
                    { id: 20, label: 'A16\nRotterdam', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    { id: 21, label: 'Rijnland-\nroute', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    { id: 22, label: 'Blanken-\nburg', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    { id: 23, label: 'Warmtelinq', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    { id: 24, label: 'Kust-\nversterking', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 }
                ]);

                var edges = new vis.DataSet([
                    // Domeinen <-> Organisaties
                    { from: 1, to: 13, label: 'beheer', font: { size: 9 } },
                    { from: 1, to: 15, label: 'samenwerking', font: { size: 9 } },
                    { from: 1, to: 16, label: 'spoor', font: { size: 9 } },
                    { from: 2, to: 12, label: 'exploitatie', font: { size: 9 } },
                    { from: 2, to: 18, label: 'belangen', font: { size: 9 } },
                    { from: 2, to: 17, label: 'vergunningen', font: { size: 9 } },
                    { from: 3, to: 10, label: 'economisch', font: { size: 9 } },
                    { from: 3, to: 14, label: 'kennis', font: { size: 9 } },
                    { from: 4, to: 12, label: 'H2-backbone', font: { size: 9 } },
                    { from: 4, to: 14, label: 'onderzoek', font: { size: 9 } },
                    { from: 5, to: 13, label: 'waterbeheer', font: { size: 9 } },
                    { from: 6, to: 10, label: 'woningbouw', font: { size: 9 } },
                    { from: 6, to: 11, label: 'stedelijk', font: { size: 9 } },

                    // Domeinen <-> Projecten
                    { from: 1, to: 20, label: 'project', font: { size: 9 }, dashes: true },
                    { from: 1, to: 21, label: 'project', font: { size: 9 }, dashes: true },
                    { from: 1, to: 22, label: 'project', font: { size: 9 }, dashes: true },
                    { from: 4, to: 23, label: 'project', font: { size: 9 }, dashes: true },
                    { from: 5, to: 24, label: 'project', font: { size: 9 }, dashes: true },

                    // Organisaties <-> Projecten
                    { from: 13, to: 20, font: { size: 9 } },
                    { from: 10, to: 20, font: { size: 9 } },
                    { from: 13, to: 22, font: { size: 9 } }
                ]);

                var options = {
                    physics: {
                        forceAtlas2Based: {
                            gravitationalConstant: -35,
                            centralGravity: 0.005,
                            springLength: 160,
                            springConstant: 0.04
                        },
                        solver: 'forceAtlas2Based',
                        stabilization: { iterations: 120 }
                    },
                    edges: {
                        arrows: { to: { enabled: true, scaleFactor: 0.5 } },
                        color: { color: '#999', highlight: '#E31837' },
                        smooth: { type: 'continuous' }
                    },
                    interaction: {
                        hover: true,
                        tooltipDelay: 200
                    }
                };

                new vis.Network(container, { nodes: nodes, edges: edges }, options);
            })();
        "#;
        document::eval(script);
    });

    rsx! {
        div { class: "zuidholland",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Zuid-Holland" }
                        span { " \u{203A} " }
                        span { class: "current", "Kennisnetwerk" }
                    }
                }

                Panel { title: "Kennisnetwerk Zuid-Holland".to_string(),
                    p { style: "margin-bottom: 16px;",
                        "Ontdek relaties tussen beleidsdomeinen, organisaties en infrastructuurprojecten in Zuid-Holland."
                    }
                    div {
                        id: "zh-network-graph",
                        style: "width: 100%; height: 500px; border: 1px solid #e0e0e0; border-radius: 8px;",
                    }
                }

                div { style: "height: 20px;" }

                div { style: "display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 20px;",
                    Panel { title: "Legenda".to_string(),
                        div { class: "graph-legend",
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #E31837;" }
                                span { "Beleidsdomeinen" }
                            }
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #DAA520;" }
                                span { "Organisaties" }
                            }
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #4CAF50;" }
                                span { "Infrastructuurprojecten" }
                            }
                        }
                    }

                    Panel { title: "Statistieken".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F4CA}" }
                            div { class: "label", "Nodes" }
                            div { class: "value", "20" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F517}" }
                            div { class: "label", "Relaties" }
                            div { class: "value", "23" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F465}" }
                            div { class: "label", "Communities" }
                            div { class: "value", "3" }
                        }
                    }

                    Panel { title: "Meest verbonden".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "1" }
                            div { class: "label", "Mobiliteit & Infrastructuur" }
                            div { class: "value", "7" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "2" }
                            div { class: "label", "Haven & Industrie" }
                            div { class: "value", "5" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "3" }
                            div { class: "label", "Rijkswaterstaat" }
                            div { class: "value", "4" }
                        }
                    }
                }
            }
        }
    }
}
