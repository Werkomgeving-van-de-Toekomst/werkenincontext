//! Ministerie van Financiën dashboard page

use dioxus::prelude::*;

use crate::components::{AppCard, Header, Panel};
use crate::state::{AppState, UserInfo};
use crate::Route;

#[component]
pub fn MinFinDashboard() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    use_effect(move || {
        state.write().user = Some(UserInfo::minfin());
    });

    rsx! {
        div { class: "minfin",
            Header {}

            main { class: "container",
                // Context Bar
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Ministerie van Financi\u{00EB}n" }
                        span { " \u{203A} " }
                        span { class: "current", "Rijksbegroting 2026" }
                    }

                    select {
                        option { "Rijksbegroting 2026" }
                        option { "Belastingplan 2026" }
                        option { "Voorjaarsnota" }
                        option { "Najaarsnota" }
                    }

                    div { class: "search-input",
                        input {
                            r#type: "text",
                            placeholder: "Zoeken in begrotingscontext...",
                        }
                        button { class: "btn btn-primary", "Zoeken" }
                    }
                }

                // Dashboard Grid
                div { class: "dashboard-grid",
                    // Left Column - Apps & Compliance
                    div {
                        Panel { title: "Context Apps".to_string(),
                            div { class: "app-grid",
                                Link { to: Route::MinFinBegrotingsverkenner,
                                    AppCard {
                                        name: "Begrotingsverkenner".to_string(),
                                        description: "Verken rijksbegrotingsdata".to_string(),
                                        badge: "Populair".to_string(),
                                    }
                                }
                                Link { to: Route::MinFinBeleidsdocumentGenerator,
                                    AppCard {
                                        name: "Beleidsdocument Generator".to_string(),
                                        description: "Genereer Kamerbrieven & nota's".to_string(),
                                        badge: "Nieuw".to_string(),
                                    }
                                }
                                Link { to: Route::MinFinFinancieleControle,
                                    AppCard {
                                        name: "Financi\u{00EB}le Controle".to_string(),
                                        description: "Monitor comptabiliteit & rechtmatigheid".to_string(),
                                    }
                                }
                                Link { to: Route::MinFinKennisnetwerk,
                                    AppCard {
                                        name: "Kennisnetwerk".to_string(),
                                        description: "Ontdek relaties via financieel netwerk".to_string(),
                                        badge: "AI".to_string(),
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Compliance Status".to_string(),
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "Comptabiliteitswet" }
                                div { class: "value", "97%" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "Rechtmatigheid" }
                                div { class: "value", "99%" }
                            }
                            div { class: "compliance-indicator warning",
                                div { class: "icon", "!" }
                                div { class: "label", "EU Begrotingsregels" }
                                div { class: "value", "94%" }
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
                                        h4 { "Miljoenennota 2026" }
                                        div { class: "meta", "Begrotingsnota \u{2022} 1 week geleden" }
                                    }
                                    span { class: "tag woo", "Openbaar" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4C4}" }
                                    div { class: "document-info",
                                        h4 { "Voorjaarsnota" }
                                        div { class: "meta", "Nota \u{2022} 2 weken geleden" }
                                    }
                                    span { class: "tag woo", "Openbaar" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4E7}" }
                                    div { class: "document-info",
                                        h4 { "Kamerbrief belastingplan" }
                                        div { class: "meta", "Brief \u{2022} 3 weken geleden" }
                                    }
                                    span { class: "tag", "Kamer" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CA}" }
                                    div { class: "document-info",
                                        h4 { "Rapport Algemene Rekenkamer" }
                                        div { class: "meta", "Rapport \u{2022} 1 maand geleden" }
                                    }
                                    span { class: "tag", "ARK" }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Gerelateerde Domeinen".to_string(),
                            div { style: "display: flex; flex-wrap: wrap; gap: 10px;",
                                div { class: "tag", "Rijksbegroting" }
                                div { class: "tag", "Belastingen" }
                                div { class: "tag", "Staatsschuld" }
                                div { class: "tag", "Toeslagen" }
                                div { class: "tag", "Douane" }
                            }
                        }
                    }

                    // Right Column - Stakeholders & AI
                    div {
                        Panel { title: "Stakeholders".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E17000;", "\u{1F3E2}" }
                                    div { class: "document-info",
                                        h4 { "Algemene Rekenkamer" }
                                        div { class: "meta", "Controle & verantwoording" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E17000;", "\u{1F4CA}" }
                                    div { class: "document-info",
                                        h4 { "CPB" }
                                        div { class: "meta", "Economische doorrekeningen" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E17000;", "\u{1F3E6}" }
                                    div { class: "document-info",
                                        h4 { "DNB" }
                                        div { class: "meta", "Monetair beleid & stabiliteit" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E17000;", "\u{1F4B0}" }
                                    div { class: "document-info",
                                        h4 { "Belastingdienst" }
                                        div { class: "meta", "Uitvoering fiscaal beleid" }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "AI Suggesties".to_string(),
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F916}" }
                                div { class: "label", "5 begrotingsclassificatie suggesties" }
                            }
                            p { style: "font-size: 0.875rem; color: #666; margin-top: 10px;",
                                "AI heeft automatisch classificaties voorgesteld voor begrotingsposten op basis van COFOG-indeling."
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
}
