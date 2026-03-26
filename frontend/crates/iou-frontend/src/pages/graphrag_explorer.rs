//! GraphRAG Explorer page

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn GraphRAGExplorer() -> Element {
    // Initialize vis-network graph after DOM render
    use_effect(move || {
        let script = r#"
            (function() {
                var container = document.getElementById('network-graph');
                if (!container || container._visNetwork) return;
                container.innerHTML = '';
                container._visNetwork = true;

                var nodes = new vis.DataSet([
                    // Informatiedomeinen (blauw, box)
                    { id: 1, label: 'Duurzaamheid &\nEnergie', color: '#0066CC', font: { color: '#fff' }, shape: 'box' },
                    { id: 2, label: 'Windpark\nAlmere', color: '#0066CC', font: { color: '#fff' }, shape: 'box' },
                    { id: 3, label: 'Omgevingsvisie\n2030', color: '#0066CC', font: { color: '#fff' }, shape: 'box' },
                    { id: 4, label: 'Subsidie-\nregeling', color: '#0066CC', font: { color: '#fff' }, shape: 'box' },
                    { id: 5, label: 'Klimaat-\nadaptatie', color: '#0066CC', font: { color: '#fff' }, shape: 'box' },
                    // Organisaties (groen, ellipse)
                    { id: 6, label: 'Gemeente\nAlmere', color: '#7CB342', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 7, label: 'Omgevings-\ndienst', color: '#7CB342', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 8, label: 'Vattenfall', color: '#7CB342', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 9, label: 'Provincie\nFlevoland', color: '#7CB342', font: { color: '#fff' }, shape: 'ellipse' },
                    // Wetgeving (oranje, diamond)
                    { id: 10, label: 'Omgevingswet', color: '#FF9800', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    { id: 11, label: 'Wet\nmilieubeheer', color: '#FF9800', font: { color: '#fff' }, shape: 'diamond', size: 25 }
                ]);

                var edges = new vis.DataSet([
                    { from: 1, to: 2, label: 'gerelateerd', font: { size: 10 } },
                    { from: 1, to: 5, label: 'gerelateerd', font: { size: 10 } },
                    { from: 2, to: 8, label: 'aanvrager', font: { size: 10 } },
                    { from: 2, to: 6, label: 'stakeholder', font: { size: 10 } },
                    { from: 2, to: 10, label: 'valt onder', font: { size: 10 } },
                    { from: 2, to: 11, label: 'valt onder', font: { size: 10 } },
                    { from: 3, to: 9, label: 'stakeholder', font: { size: 10 } },
                    { from: 3, to: 1, label: 'gerelateerd', font: { size: 10 } },
                    { from: 4, to: 9, label: 'stakeholder', font: { size: 10 } },
                    { from: 4, to: 2, label: 'gerelateerd', font: { size: 10 } },
                    { from: 5, to: 3, label: 'gerelateerd', font: { size: 10 } },
                    { from: 7, to: 2, label: 'adviseert', font: { size: 10 } },
                    { from: 9, to: 10, label: 'valt onder', font: { size: 10 } }
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
                        color: { color: '#999', highlight: '#0066CC' },
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
        Header {}
        main { class: "container",
            Panel { title: "GraphRAG Explorer".to_string(),
                p { "Ontdek relaties tussen domeinen via de kennisgraaf." }
                div {
                    id: "network-graph",
                    style: "height: 500px; border-radius: 8px;",
                }
            }

            div { style: "height: 20px;" }

            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                Panel { title: "Legenda".to_string(),
                    div { class: "graph-legend",
                        div { class: "legend-item",
                            span { class: "legend-color", style: "background: #0066CC;" }
                            span { "Informatiedomeinen" }
                        }
                        div { class: "legend-item",
                            span { class: "legend-color", style: "background: #7CB342;" }
                            span { "Organisaties" }
                        }
                        div { class: "legend-item",
                            span { class: "legend-color", style: "background: #FF9800;" }
                            span { "Wetgeving" }
                        }
                    }
                }

                Panel { title: "Statistieken".to_string(),
                    div { class: "compliance-indicator ok",
                        div { class: "icon", "\u{1F4CA}" }
                        div { class: "label", "Nodes" }
                        div { class: "value", "11" }
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
