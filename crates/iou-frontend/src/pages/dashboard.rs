//! Main dashboard page

use dioxus::prelude::*;

use crate::components::{AppCard, Header, Panel};
use crate::state::{AppState, UserInfo};
use crate::Route;

#[component]
pub fn Dashboard() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    use_effect(move || {
        state.write().user = Some(UserInfo::flevoland());
    });

    rsx! {
        Header {}

        main { class: "container",
            // Context Bar
            div { class: "context-bar",
                div { class: "breadcrumb",
                    span { "Provincie Flevoland" }
                    span { " \u{203A} " }
                    span { class: "current", "Duurzaamheid & Energie" }
                }

                select {
                    option { "Duurzaamheid & Energie" }
                    option { "Windpark Almere" }
                    option { "Omgevingsvergunning Bouw" }
                    option { "Omgevingsvisie 2030" }
                }

                div { class: "search-input",
                    input {
                        r#type: "text",
                        placeholder: "Zoeken in context...",
                        oninput: move |evt| {
                            state.write().search_query = evt.value();
                        },
                    }
                    button { class: "btn btn-primary", "Zoeken" }
                }
            }

            // Dashboard Grid
            div { class: "dashboard-grid",
                // Left Column - Apps
                div {
                    Panel { title: "Context Apps".to_string(),
                        div { class: "app-grid",
                            Link { to: Route::DataVerkenner,
                                AppCard {
                                    name: "Data Verkenner".to_string(),
                                    description: "Verken provinciale datasets".to_string(),
                                    badge: "Populair".to_string(),
                                }
                            }
                            Link { to: Route::DocumentGenerator,
                                AppCard {
                                    name: "Document Generator".to_string(),
                                    description: "Genereer compliant documenten".to_string(),
                                    badge: "Nieuw".to_string(),
                                }
                            }
                            Link { to: Route::Nalevingscontrole,
                                AppCard {
                                    name: "Nalevingscontrole".to_string(),
                                    description: "Monitor Woo/AVG compliance".to_string(),
                                }
                            }
                            AppCard {
                                name: "Tijdlijn Weergave".to_string(),
                                description: "Bekijk activiteiten tijdlijn".to_string(),
                            }
                            Link { to: Route::GraphRAGExplorer,
                                AppCard {
                                    name: "GraphRAG Explorer".to_string(),
                                    description: "Ontdek relaties via kennisgraaf".to_string(),
                                    badge: "AI".to_string(),
                                }
                            }
                            AppCard {
                                name: "Samenwerkingscentrum".to_string(),
                                description: "Werk samen met anderen".to_string(),
                            }
                        }
                    }

                    div { style: "height: 20px;" }

                    Panel { title: "Compliance Status".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Woo Compliance" }
                            div { class: "value", "98%" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "AVG Compliance" }
                            div { class: "value", "100%" }
                        }
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "!" }
                            div { class: "label", "Bewaartermijnen" }
                            div { class: "value", "3 acties" }
                        }
                    }
                }

                // Center Column - Documents
                div {
                    Panel { title: "Recente Documenten".to_string(),
                        ul { class: "document-list",
                            li { class: "document-item",
                                div { class: "document-icon", "\u{1F4C4}" }
                                div { class: "document-info",
                                    h4 { "Besluit subsidieverlening windpark" }
                                    div { class: "meta", "Besluit \u{2022} 2 dagen geleden" }
                                }
                                span { class: "tag woo", "Woo" }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", "\u{1F4E7}" }
                                div { class: "document-info",
                                    h4 { "Re: Voortgang projectplan duurzaamheid" }
                                    div { class: "meta", "Email \u{2022} 3 dagen geleden" }
                                }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", "\u{1F4C4}" }
                                div { class: "document-info",
                                    h4 { "Advies Omgevingsdienst Flevoland" }
                                    div { class: "meta", "Document \u{2022} 1 week geleden" }
                                }
                                span { class: "tag", "advies" }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", "\u{1F4CA}" }
                                div { class: "document-info",
                                    h4 { "Dataset energieverbruik gemeenten" }
                                    div { class: "meta", "Data \u{2022} 2 weken geleden" }
                                }
                                span { class: "tag", "CBS" }
                            }
                        }
                    }

                    div { style: "height: 20px;" }

                    Panel { title: "Gerelateerde Domeinen".to_string(),
                        div { style: "display: flex; flex-wrap: wrap; gap: 10px;",
                            div { class: "tag", "Windpark Almere" }
                            div { class: "tag", "Omgevingsvisie 2030" }
                            div { class: "tag", "Subsidieregeling Energie" }
                            div { class: "tag", "Klimaatadaptatie" }
                        }
                    }
                }

                // Right Column - Stakeholders & AI
                div {
                    Panel { title: "Stakeholders".to_string(),
                        ul { class: "document-list",
                            li { class: "document-item",
                                div { class: "document-icon", style: "background: #7CB342;", "\u{1F464}" }
                                div { class: "document-info",
                                    h4 { "Gemeente Almere" }
                                    div { class: "meta", "Mede-initiatiefnemer" }
                                }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", style: "background: #7CB342;", "\u{1F464}" }
                                div { class: "document-info",
                                    h4 { "Omgevingsdienst Flevoland" }
                                    div { class: "meta", "Adviseur" }
                                }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", style: "background: #7CB342;", "\u{1F464}" }
                                div { class: "document-info",
                                    h4 { "Vattenfall NL" }
                                    div { class: "meta", "Aanvrager" }
                                }
                            }
                        }
                    }

                    div { style: "height: 20px;" }

                    Panel { title: "AI Suggesties".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F916}" }
                            div { class: "label", "3 nieuwe metadata suggesties" }
                        }
                        p { style: "font-size: 0.875rem; color: #666; margin-top: 10px;",
                            "AI heeft automatisch tags en classificaties voorgesteld voor 3 nieuwe documenten."
                        }
                        button { class: "btn btn-secondary", style: "margin-top: 10px; width: 100%;",
                            "Bekijk suggesties"
                        }
                    }
                }
            }
        }
    }
}
