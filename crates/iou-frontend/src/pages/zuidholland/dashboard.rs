//! Zuid-Holland dashboard page

use dioxus::prelude::*;

use crate::components::{AppCard, Header, Panel};
use crate::state::{AppState, UserInfo};
use crate::Route;

#[component]
pub fn ZuidHolland() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    use_effect(move || {
        state.write().user = Some(UserInfo::zuidholland());
    });

    rsx! {
        div { class: "zuidholland",
            Header {}

            main { class: "container",
                // Context Bar
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Provincie Zuid-Holland" }
                        span { " \u{203A} " }
                        span { class: "current", "Mobiliteit & Economie" }
                    }

                    select {
                        option { "Mobiliteit & Economie" }
                        option { "Rotterdams Haven & Industrie" }
                        option { "Groene Hart & Landbouw" }
                        option { "Tuinbouw Westland" }
                        option { "Wonen & Leefomgeving" }
                        option { "Energietransitie Noordzee" }
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
                                Link { to: Route::ZHMobiliteitsverkenner,
                                    AppCard {
                                        name: "Mobiliteitsverkenner".to_string(),
                                        description: "Verken provinciale mobiliteitsdata".to_string(),
                                        badge: "Populair".to_string(),
                                    }
                                }
                                Link { to: Route::ZHHavenmonitor,
                                    AppCard {
                                        name: "Havenmonitor".to_string(),
                                        description: "Rotterdam Haven dashboard".to_string(),
                                        badge: "Nieuw".to_string(),
                                    }
                                }
                                Link { to: Route::ZHProjectPortfolio,
                                    AppCard {
                                        name: "Project Portfolio".to_string(),
                                        description: "Beheer grote infra-projecten".to_string(),
                                    }
                                }
                                Link { to: Route::ZHKennisnetwerk,
                                    AppCard {
                                        name: "Kennisnetwerk".to_string(),
                                        description: "Ontdek relaties via kennisgraaf".to_string(),
                                        badge: "AI".to_string(),
                                    }
                                }
                                Link { to: Route::ZHStakeholderDossier,
                                    AppCard {
                                        name: "Stakeholder Dossier".to_string(),
                                        description: "Overzicht partners & relaties".to_string(),
                                    }
                                }
                                Link { to: Route::Nalevingscontrole,
                                    AppCard {
                                        name: "Nalevingscontrole".to_string(),
                                        description: "Monitor Woo/AVG compliance".to_string(),
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Compliance Status".to_string(),
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "Woo Compliance" }
                                div { class: "value", "96%" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "AVG Compliance" }
                                div { class: "value", "100%" }
                            }
                            div { class: "compliance-indicator warning",
                                div { class: "icon", "!" }
                                div { class: "label", "Bewaartermijnen" }
                                div { class: "value", "5 acties" }
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
                                        h4 { "Besluit MIRT-verlening A16 Rotterdam" }
                                        div { class: "meta", "Besluit \u{2022} 1 dag geleden" }
                                    }
                                    span { class: "tag woo", "Woo" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CA}" }
                                    div { class: "document-info",
                                        h4 { "Monitor Havenbedrijf Rotterdam 2025" }
                                        div { class: "meta", "Rapportage \u{2022} 3 dagen geleden" }
                                    }
                                    span { class: "tag", "Economie" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4E7}" }
                                    div { class: "document-info",
                                        h4 { "Re: Voortgang programmeers Wmo" }
                                        div { class: "meta", "Email \u{2022} 4 dagen geleden" }
                                    }
                                    span { class: "tag", "Wmo" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F3A2}" }
                                    div { class: "document-info",
                                        h4 { "Kwetsbaarheid Hoofdwegennet" }
                                        div { class: "meta", "Analyse \u{2022} 1 week geleden" }
                                    }
                                    span { class: "tag", "Mobiliteit" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CA}" }
                                    div { class: "document-info",
                                        h4 { "Dataset verkeersintensiteit ZH" }
                                    div { class: "meta", "Data \u{2022} 2 weken geleden" }
                                    }
                                    span { class: "tag", "CBS" }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Gerelateerde Domeinen".to_string(),
                            div { style: "display: flex; flex-wrap: wrap; gap: 10px;",
                                div { class: "tag", "Rotterdams Haven & Industrie" }
                                div { class: "tag", "Hoofdwegennet" }
                                div { class: "tag", "Brede School Regio Rotterdam" }
                                div { class: "tag", "Tuinbouw Westland" }
                                div { class: "tag", "Energietransitie Noordzee" }
                            }
                        }
                    }

                    // Right Column - Stakeholders & AI
                    div {
                        Panel { title: "Stakeholders".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E31837;", "\u{1F464}" }
                                    div { class: "document-info",
                                        h4 { "Gemeente Rotterdam" }
                                        div { class: "meta", "Hoofdstad & havenstad" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E31837;", "\u{1F464}" }
                                    div { class: "document-info",
                                        h4 { "Gemeente Den Haag" }
                                        div { class: "meta", "Residentieel & Internationaal" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E31837;", "\u{1F464}" }
                                    div { class: "document-info",
                                        h4 { "Havenbedrijf Rotterdam" }
                                        div { class: "meta", "Haven & industriecomplex" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E31837;", "\u{1F464}" }
                                    div { class: "document-info",
                                        h4 { "Rijkswaterstaat" }
                                        div { class: "meta", "Infrastructuur & water" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E31837;", "\u{1F464}" }
                                    div { class: "document-info",
                                        h4 { "TU Delft" }
                                        div { class: "meta", "Kennispartner mobiliteit" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E31837;", "\u{1F464}" }
                                    div { class: "document-info",
                                        h4 { "Erasmus MC" }
                                        div { class: "meta", "Gezondheidsregio ZH" }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "AI Suggesties".to_string(),
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F916}" }
                                div { class: "label", "7 nieuwe metadata suggesties" }
                            }
                            p { style: "font-size: 0.875rem; color: #666; margin-top: 10px;",
                                "AI heeft automatisch tags en classificaties voorgesteld voor nieuwe documenten over mobiliteitsprojecten."
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
