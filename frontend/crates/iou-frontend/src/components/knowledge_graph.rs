//! Knowledge Graph component for visualizing PROVISA relationships
//!
//! Simple CSS-based flow visualization without external JavaScript dependencies

use dioxus::prelude::*;

/// Knowledge Graph component showing PROVISA relationships
#[component]
pub fn KnowledgeGraph(
    #[props(default)]
    title: String,
    #[props(default)]
    mermaid_code: String,
) -> Element {
    rsx! {
        div { class: "knowledge-graph",
            if !title.is_empty() {
                h3 { style: "margin-bottom: 15px; font-size: 1.1rem; font-weight: 600;",
                    "{title}"
                }
            }
            div { class: "knowledge-graph-canvas",
                style: "background: #f8f9fa; padding: 20px; border-radius: 8px; min-height: 400px; overflow-x: auto;",

                // Simple CSS-based PROVISA workflow visualization
                div { style: "display: flex; flex-direction: column; align-items: center; gap: 20px; min-width: 600px;",

                    // Step 1: Document Upload
                    div { class: "workflow-node",
                        style: r#"background: #4fc3f7; color: white; padding: 15px 25px; border-radius: 8px;
                                   font-weight: 600; text-align: center; min-width: 180px; box-shadow: 0 2px 8px rgba(79, 195, 247, 0.3);"#,
                        div { style: "font-size: 1.1rem;", "📄 Document Upload" }
                        div { style: "font-size: 0.8rem; margin-top: 5px; opacity: 0.9;", "Woo document" }
                    }

                    // Arrow
                    div { style: "font-size: 24px; color: #7C4DFF;", "⬇" }

                    // Step 2: PETRA Classificatie
                    div { class: "workflow-node",
                        style: r#"background: #7C4DFF; color: white; padding: 15px 25px; border-radius: 8px;
                                   font-weight: 600; text-align: center; min-width: 180px; box-shadow: 0 2px 8px rgba(124, 77, 255, 0.3);"#,
                        div { style: "font-size: 1.1rem;", "🏷️ PETRA Analyse" }
                        div { style: "font-size: 0.8rem; margin-top: 5px; opacity: 0.9;", "Classificatie" }
                    }

                    // Arrow
                    div { style: "font-size: 24px; color: #7C4DFF;", "⬇" }

                    // Step 3: DMN Regelaar
                    div { class: "workflow-node",
                        style: r#"background: #ffca28; color: #333; padding: 15px 25px; border-radius: 8px;
                                   font-weight: 600; text-align: center; min-width: 180px; box-shadow: 0 2px 8px rgba(255, 202, 40, 0.3);"#,
                        div { style: "font-size: 1.1rem;", "⚖️ DMN Regelaar" }
                        div { style: "font-size: 0.8rem; margin-top: 5px; opacity: 0.8;", "PROVISA Rules" }
                    }

                    // Arrow
                    div { style: "font-size: 24px; color: #7C4DFF;", "⬇" }

                    // Step 4 & 5: Resultaten (side by side)
                    div { style: "display: flex; gap: 30px;",
                        div { class: "workflow-node",
                            style: r#"background: #66bb6a; color: white; padding: 15px 25px; border-radius: 8px;
                                       font-weight: 600; text-align: center; min-width: 150px; box-shadow: 0 2px 8px rgba(102, 187, 106, 0.3);"#,
                            div { style: "font-size: 1rem;", "⏱️ Bewaartermijn" }
                            div { style: "font-size: 0.75rem; margin-top: 5px; opacity: 0.9;", "Permanent / Tijdelijk" }
                        }

                        div { class: "workflow-node",
                            style: r#"background: #7c4dff; color: white; padding: 15px 25px; border-radius: 8px;
                                       font-weight: 600; text-align: center; min-width: 150px; box-shadow: 0 2px 8px rgba(124, 77, 255, 0.3);"#,
                            div { style: "font-size: 1rem;", "🗄️ TriplyDB" }
                            div { style: "font-size: 0.75rem; margin-top: 5px; opacity: 0.9;", "Kennisgraaf" }
                        }
                    }
                }

                // Legend
                div { style: "margin-top: 30px; padding-top: 20px; border-top: 1px solid #ddd;",
                    h5 { style: "margin-bottom: 10px; font-size: 0.9rem; color: #666;", "IOU Componenten" }
                    div { style: "display: flex; flex-wrap: wrap; gap: 15px;",
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            div { style: "width: 12px; height: 12px; background: #4fc3f7; border-radius: 3px;" }
                            span { style: "font-size: 0.85rem; color: #555;", "Document Invoer" }
                        }
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            div { style: "width: 12px; height: 12px; background: #7C4DFF; border-radius: 3px;" }
                            span { style: "font-size: 0.85rem; color: #555;", "Classificatie & Opslag" }
                        }
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            div { style: "width: 12px; height: 12px; background: #ffca28; border-radius: 3px;" }
                            span { style: "font-size: 0.85rem; color: #555;", "DMN Regelaar" }
                        }
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            div { style: "width: 12px; height: 12px; background: #66bb6a; border-radius: 3px;" }
                            span { style: "font-size: 0.85rem; color: #555;", "Resultaat" }
                        }
                    }
                }
            }
        }
    }
}
