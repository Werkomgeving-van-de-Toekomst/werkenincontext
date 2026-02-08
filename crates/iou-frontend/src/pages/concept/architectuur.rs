//! Architectuur - systeemcomponenten visualisatie via vis-network

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn ConceptArchitectuur() -> Element {
    // Initialize vis-network graph after DOM render
    use_effect(move || {
        let script = r#"
            (function() {
                var container = document.getElementById('concept-arch-graph');
                if (!container || container._visNetwork) return;
                container.innerHTML = '';
                container._visNetwork = true;

                var nodes = new vis.DataSet([
                    // Core platform (paars, box)
                    { id: 1, label: 'Context\nEngine', color: '#7C4DFF', font: { color: '#fff' }, shape: 'box' },
                    { id: 2, label: 'Knowledge\nGraph', color: '#7C4DFF', font: { color: '#fff' }, shape: 'box' },
                    { id: 3, label: 'Document\nManagement', color: '#7C4DFF', font: { color: '#fff' }, shape: 'box' },
                    // AI services (teal, ellipse)
                    { id: 4, label: 'Metadata\nExtractie', color: '#00BCD4', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 5, label: 'Classificatie\nAI', color: '#00BCD4', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 6, label: 'Compliance\nChecker', color: '#00BCD4', font: { color: '#fff' }, shape: 'ellipse' },
                    { id: 7, label: 'GraphRAG', color: '#00BCD4', font: { color: '#fff' }, shape: 'ellipse' },
                    // Externe systemen (groen, diamond)
                    { id: 8, label: 'Woo\nRegister', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    { id: 9, label: 'CBS\nOpen Data', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    { id: 10, label: 'PDOK\nGeo', color: '#4CAF50', font: { color: '#fff' }, shape: 'diamond', size: 25 },
                    // Frontend (blauwgrijs, box)
                    { id: 11, label: 'IOU\nFrontend', color: '#37474F', font: { color: '#fff' }, shape: 'box' },
                    { id: 12, label: 'REST\nAPI', color: '#37474F', font: { color: '#fff' }, shape: 'box' }
                ]);

                var edges = new vis.DataSet([
                    { from: 11, to: 12, label: 'HTTP', font: { size: 10 } },
                    { from: 12, to: 1, label: 'context', font: { size: 10 } },
                    { from: 12, to: 3, label: 'CRUD', font: { size: 10 } },
                    { from: 1, to: 2, label: 'query', font: { size: 10 } },
                    { from: 1, to: 5, label: 'classify', font: { size: 10 } },
                    { from: 3, to: 4, label: 'extract', font: { size: 10 } },
                    { from: 3, to: 6, label: 'check', font: { size: 10 } },
                    { from: 2, to: 7, label: 'traverse', font: { size: 10 } },
                    { from: 6, to: 8, label: 'validate', font: { size: 10 } },
                    { from: 4, to: 9, label: 'enrich', font: { size: 10 } },
                    { from: 1, to: 10, label: 'geo-context', font: { size: 10 } }
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
                        stabilization: { iterations: 100 }
                    },
                    edges: {
                        arrows: { to: { enabled: true, scaleFactor: 0.5 } },
                        color: { color: '#999', highlight: '#7C4DFF' },
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
        div { class: "concept",
            Header {}
            main { class: "container",
                div { class: "concept-intro",
                    h2 { "Systeemarchitectuur" }
                    p { "De IOU-Modern architectuur verbindt context, AI en compliance in \u{00e9}\u{00e9}n platform." }
                }

                Panel { title: "Architectuur Overzicht".to_string(),
                    p { style: "margin-bottom: 10px;",
                        "Interactieve weergave van de systeemcomponenten en hun relaties."
                    }
                    div {
                        id: "concept-arch-graph",
                        style: "height: 500px; border-radius: 8px;",
                    }
                }

                div { style: "height: 20px;" }

                div { style: "display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 20px;",
                    Panel { title: "Legenda".to_string(),
                        div { class: "graph-legend",
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #7C4DFF;" }
                                span { "Core Platform" }
                            }
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #00BCD4;" }
                                span { "AI Services" }
                            }
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #4CAF50;" }
                                span { "Externe Systemen" }
                            }
                            div { class: "legend-item",
                                span { class: "legend-color", style: "background: #37474F;" }
                                span { "Frontend & API" }
                            }
                        }
                    }

                    Panel { title: "Technologie Stack".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2699}" }
                            div { class: "label", "Backend" }
                            div { class: "value", "Rust / Axum" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F310}" }
                            div { class: "label", "Frontend" }
                            div { class: "value", "Dioxus WASM" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F916}" }
                            div { class: "label", "AI" }
                            div { class: "value", "LLM + RAG" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F517}" }
                            div { class: "label", "Graph" }
                            div { class: "value", "Neo4j / vis.js" }
                        }
                    }

                    Panel { title: "Statistieken".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F4CA}" }
                            div { class: "label", "Nodes" }
                            div { class: "value", "12" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F517}" }
                            div { class: "label", "Edges" }
                            div { class: "value", "11" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F465}" }
                            div { class: "label", "Lagen" }
                            div { class: "value", "4" }
                        }
                    }
                }
            }
        }
    }
}
