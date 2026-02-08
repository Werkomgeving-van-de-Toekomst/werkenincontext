//! Main dashboard page

use dioxus::prelude::*;

use crate::components::{AppCard, Header, Panel};
use crate::state::{AppState, UserInfo};
use crate::Route;

struct WooDoc {
    titel: &'static str,
    samenvatting: &'static str,
    datum: &'static str,
    soort: &'static str,
    bron_id: &'static str,
    url: &'static str,
}

const WOO_DOCS: &[WooDoc] = &[
    WooDoc {
        titel: "Kennisgeving Projectbesluit en MER Rondweg Lelystad-Zuid",
        samenvatting: "Kennisgeving van het projectbesluit en milieueffectrapport voor de Rondweg Lelystad-Zuid (Laan van Nieuw Land \u{2013} Verlengde Westerdreef). Betreft de aanleg van een nieuwe provinciale weg ter verbetering van de bereikbaarheid.",
        datum: "30 jan 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1767",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1767.html",
    },
    WooDoc {
        titel: "Besluit omgevingsvergunning Natura 2000 zandwinning IJsselmeer",
        samenvatting: "Besluit (positieve) weigering omgevingsvergunning voor een Natura 2000-activiteit zandwinning vaargeul Amsterdam-Lemmer (VAL5) in het IJsselmeer. De vergunning is geweigerd vanwege mogelijke impact op beschermde natuur.",
        datum: "28 jan 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1405",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1405.html",
    },
    WooDoc {
        titel: "Ontheffing helikopterlanding provincie Flevoland 2026",
        samenvatting: "Wet Luchtvaart generieke ontheffing Tijdelijk en Uitzonderlijk Gebruik kalenderjaar 2026 in de provincie Flevoland voor het landen en stijgen met een helikopter.",
        datum: "29 jan 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1457",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1457.html",
    },
    WooDoc {
        titel: "Ondermandaat Bedrijfsvoering Omgevingsdienst Flevoland",
        samenvatting: "Gewijzigd ondermandaat voor de bedrijfsvoering van de Omgevingsdienst Flevoland & Gooi en Vechtstreek. Regelt de bevoegdheidsverdeling voor operationele beslissingen.",
        datum: "4 feb 2026",
        soort: "Blad gemeenschappelijke regeling",
        bron_id: "bgr-2026-301",
        url: "https://zoek.officielebekendmakingen.nl/bgr-2026-301.html",
    },
    WooDoc {
        titel: "Last onder bestuursdwang vaartuigen Hoge Vaart",
        samenvatting: "Handhavingsbesluit last onder bestuursdwang voor vaartuigen in de berm langs de Hoge Vaart. Eigenaren worden gesommeerd de vaartuigen te verwijderen.",
        datum: "5 feb 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1953",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1953.html",
    },
];

#[component]
pub fn Dashboard() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut selected_doc = use_signal(|| None::<usize>);

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
                    Panel { title: "Recente Woo-documenten (open.overheid.nl)".to_string(),
                        ul { class: "document-list",
                            for (i, doc) in WOO_DOCS.iter().enumerate() {
                                li {
                                    class: "document-item",
                                    style: if *selected_doc.read() == Some(i) { "background: #f0ebff; cursor: pointer;" } else { "cursor: pointer;" },
                                    onclick: move |_| {
                                        if *selected_doc.read() == Some(i) {
                                            selected_doc.set(None);
                                        } else {
                                            selected_doc.set(Some(i));
                                        }
                                    },
                                    div { class: "document-icon", "\u{1F4C4}" }
                                    div { class: "document-info",
                                        h4 { "{doc.titel}" }
                                        div { class: "meta", "{doc.soort} \u{2022} {doc.datum} \u{2022} {doc.bron_id}" }
                                    }
                                    span { class: "tag woo", "Woo" }
                                }
                                if *selected_doc.read() == Some(i) {
                                    li { style: "padding: 15px; background: #f8f6ff; border-left: 3px solid #5B3CC4;",
                                        p { style: "font-size: 0.875rem; color: #444; line-height: 1.6; margin-bottom: 12px;",
                                            "{doc.samenvatting}"
                                        }
                                        div { style: "display: flex; gap: 10px; align-items: center;",
                                            a {
                                                href: "{doc.url}",
                                                target: "_blank",
                                                class: "btn btn-primary",
                                                style: "text-decoration: none; font-size: 0.8rem;",
                                                "Bekijk op open.overheid.nl \u{2197}"
                                            }
                                            span { style: "font-size: 0.75rem; color: #888;", "{doc.bron_id}" }
                                        }
                                    }
                                }
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
