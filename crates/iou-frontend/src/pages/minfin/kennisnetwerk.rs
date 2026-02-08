//! Kennisnetwerk - Financieel netwerk explorer via vis-network

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn MinFinKennisnetwerk() -> Element {
    // Initialize vis-network graph after DOM render
    use_effect(move || {
        let script = r#"
            (function() {
                var container = document.getElementById('minfin-network-graph');
                if (!container || container._visNetwork) return;
                container.innerHTML = '';
                container._visNetwork = true;

                var nodes = new vis.DataSet([
                    // Beleidsterreinen (donkerblauw, box)
                    { id: 1, label: 'Rijks-\nbegroting', color: '#154273', font: { color: '#fff' }, shape: 'box' },
                    { id: 2, label: 'Belastingen', color: '#154273', font: { color: '#fff' }, shape: 'box' },
                    { id: 3, label: 'Staatsschuld', color: '#154273', font: { color: '#fff' }, shape: 'box' },
                    { id: 4, label: 'Toeslagen', color: '#154273', font: { color: '#fff' }, shape: 'box' },
                    { id: 5, label: 'Douane', color: '#154273', font: { color: '#fff' }, shape: 'box' },
                    // Organisaties (oranje, ellipse)
                    { id: 6, label: 'Belasting-\ndienst', color: '#E17000', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 7, label: 'CPB', color: '#E17000', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 8, label: 'DNB', color: '#E17000', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 9, label: 'Algemene\nRekenkamer', color: '#E17000', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 10, label: 'CBS', color: '#E17000', font: { color: '#fff' }, shape: 'ellipse' },
                    // Wetgeving (groen, diamond)
                    { id: 11, label: 'Comptabi-\nliteitswet', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    { id: 12, label: 'AWR', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    { id: 13, label: 'EU-Stabili-\nteitspact', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 }
                ]);

                var edges = new vis.DataSet([
                    { from: 1, to: 11, label: 'valt onder', font: { size: 10 } },
                    { from: 1, to: 13, label: 'valt onder', font: { size: 10 } },
                    { from: 2, to: 12, label: 'valt onder', font: { size: 10 } },
                    { from: 2, to: 6, label: 'uitvoerder', font: { size: 10 } },
                    { from: 3, to: 8, label: 'adviseert', font: { size: 10 } },
                    { from: 3, to: 13, label: 'valt onder', font: { size: 10 } },
                    { from: 4, to: 6, label: 'uitvoerder', font: { size: 10 } },
                    { from: 5, to: 6, label: 'uitvoerder', font: { size: 10 } },
                    { from: 7, to: 1, label: 'adviseert', font: { size: 10 } },
                    { from: 9, to: 1, label: 'controleert', font: { size: 10 } },
                    { from: 10, to: 7, label: 'levert data', font: { size: 10 } },
                    { from: 6, to: 12, label: 'valt onder', font: { size: 10 } },
                    { from: 8, to: 3, label: 'beheert', font: { size: 10 } }
                ]);

                var options = {
                    physics: {
                        forceAtlas2Based: {
                            gravitationalConstant: -30,
                            centralGravity: 0.005,
                            springLength: 150,
                            springConstant: 0.04
                        },
                        solver: 'forceAtlas2Based',
                        stabilization: { iterations: 100 }
                    },
                    edges: {
                        arrows: { to: { enabled: true, scaleFactor: 0.5 } },
                        color: { color: '#999', highlight: '#154273' },
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
        div { class: "minfin",
            Header {}
            main { class: "container",
                Panel { title: "Kennisnetwerk Financi\u{00eb}n".to_string(),
                    p { "Ontdek relaties tussen beleidsterreinen, organisaties en wetgeving." }
                    div {
                        id: "minfin-network-graph",
                        style: "height: 500px; border-radius: 8px;",
                    }
                }

                div { style: "height: 20px;" }

                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                    Panel { title: "Legenda".to_string(),
                        div { class: "graph-legend",
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #154273;" }
                                span { "Beleidsterreinen" }
                            }
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #E17000;" }
                                span { "Organisaties" }
                            }
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #4CAF50;" }
                                span { "Wetgeving" }
                            }
                        }
                    }

                    Panel { title: "Statistieken".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F4CA}" }
                            div { class: "label", "Nodes" }
                            div { class: "value", "13" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F517}" }
                            div { class: "label", "Edges" }
                            div { class: "value", "13" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F465}" }
                            div { class: "label", "Communities" }
                            div { class: "value", "3" }
                        }
                    }
                }
            }
        }
    }
}
