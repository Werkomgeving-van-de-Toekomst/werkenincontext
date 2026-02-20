//! Ministerie van Financiën dashboard page

use dioxus::prelude::*;

use crate::components::{AppCard, Header, Panel, TimelineEvent, TimelineEventType, Timeline};
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
        titel: "Wijziging Besluit inrichtingseisen bpm en mrb",
        samenvatting: "Besluit van de Staatssecretaris van Financi\u{00eb}n van 28 januari 2026 (nr. 2026-599) tot wijziging van het Besluit inrichtingseisen belasting van personenauto\u{2019}s en motorrijwielen (bpm) en motorrijtuigenbelasting (mrb).",
        datum: "3 feb 2026",
        soort: "Staatscourant",
        bron_id: "stcrt-2026-3026",
        url: "https://zoek.officielebekendmakingen.nl/stcrt-2026-3026.html",
    },
    WooDoc {
        titel: "Instelbesluit Co\u{00f6}rdinatiegroep Verrekenprijzen 2026",
        samenvatting: "Besluit van de Staatssecretaris van Financi\u{00eb}n over de verantwoordelijkheid, taakomschrijving, werkterrein en werkwijze van de Co\u{00f6}rdinatiegroep verrekenprijzen (CGVP). Regelt de interne organisatie voor internationaal fiscaal beleid.",
        datum: "2 feb 2026",
        soort: "Staatscourant",
        bron_id: "stcrt-2026-2199",
        url: "https://zoek.officielebekendmakingen.nl/stcrt-2026-2199.html",
    },
    WooDoc {
        titel: "Advies Raad van State \u{2014} Wijziging Uitvoeringsbesluit AWI",
        samenvatting: "Advies van de Raad van State inzake het ontwerp van een algemene maatregel van bestuur, houdende wijziging van het Uitvoeringsbesluit Algemene wet inkomensafhankelijke regelingen en het Besluit belasting- en invorderingsrente.",
        datum: "30 jan 2026",
        soort: "Staatscourant",
        bron_id: "stcrt-2026-2590",
        url: "https://zoek.officielebekendmakingen.nl/stcrt-2026-2590.html",
    },
    WooDoc {
        titel: "Besluit ROW 2026 \u{2014} resultaat uit overige werkzaamheden",
        samenvatting: "Besluit van de Staatssecretaris van Financi\u{00eb}n van 15 januari 2026 (nr. 2026-436) over resultaat uit overige werkzaamheden. Bevat beleidsregels voor de fiscale behandeling van inkomsten uit diverse werkzaamheden.",
        datum: "28 jan 2026",
        soort: "Staatscourant",
        bron_id: "stcrt-2026-1724",
        url: "https://zoek.officielebekendmakingen.nl/stcrt-2026-1724.html",
    },
    WooDoc {
        titel: "Wijzigingen Bezwaarschriftenadviescommissie hersteloperatie toeslagen",
        samenvatting: "Besluit van de Staatssecretaris van Financi\u{00eb}n \u{2013} Herstel en Toeslagen, houdende verschillende wijzigingen voor de Bezwaarschriftenadviescommissie hersteloperatie toeslagen. Betreft de samenstelling en werkwijze van de commissie.",
        datum: "21 jan 2026",
        soort: "Staatscourant",
        bron_id: "stcrt-2026-1374",
        url: "https://zoek.officielebekendmakingen.nl/stcrt-2026-1374.html",
    },
];

#[component]
pub fn MinFinDashboard() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut selected_doc = use_signal(|| None::<usize>);

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

                    // Center Column - Timeline & Documents
                    div {
                        Panel { title: "Tijdlijn: Rijksbegroting 2026".to_string(),
                            Timeline {
                                title: String::new(),
                                events: vec![
                                    TimelineEvent {
                                        id: "1".to_string(),
                                        title: "Besluit van 28 januari 2026".to_string(),
                                        date: "2026-01-28".to_string(),
                                        date_display: "28 jan 2026".to_string(),
                                        description: "Staatscourant besluit over de rijksbegroting van 28 januari (nr. 6).".to_string(),
                                        event_type: TimelineEventType::Besluit,
                                        url: Some("https://www.rijksoverheid.nl".to_string()),
                                    },
                                    TimelineEvent {
                                        id: "2".to_string(),
                                        title: "Wijzigingen Besluit inrichtingsbesluit".to_string(),
                                        date: "2026-01-21".to_string(),
                                        date_display: "21 jan 2026".to_string(),
                                        description: "Publicatie van de wijzigingen besluit inrichtingsbesluit voor het jaar 2026.".to_string(),
                                        event_type: TimelineEventType::Document,
                                        url: Some("https://www.rijksoverheid.nl".to_string()),
                                    },
                                    TimelineEvent {
                                        id: "3".to_string(),
                                        title: "Najaarsnota 2025".to_string(),
                                        date: "2025-09-18".to_string(),
                                        date_display: "18 sep 2025".to_string(),
                                        description: "Miljoennota 2025 over de uitvoering van het begrotingsbeleid en fiscale hervormingen.".to_string(),
                                        event_type: TimelineEventType::Document,
                                        url: Some("https://www.rijksoverheid.nl".to_string()),
                                    },
                                    TimelineEvent {
                                        id: "4".to_string(),
                                        title: "Voorjaarsnota 2026".to_string(),
                                        date: "2025-09-17".to_string(),
                                        date_display: "17 sep 2025".to_string(),
                                        description: "Begroting op hoofdlijnen voor het begrotingsjaar 2026.".to_string(),
                                        event_type: TimelineEventType::ProjectMilestone,
                                        url: Some("https://www.rijksoverheid.nl".to_string()),
                                    },
                                    TimelineEvent {
                                        id: "5".to_string(),
                                        title: "Aanname Wet open overheid van kracht".to_string(),
                                        date: "2022-05-11".to_string(),
                                        date_display: "11 mei 2022".to_string(),
                                        description: "De Wet open overheid tradt op 11 mei 2022 in werking en vervangt de Wet openbaarheid van bestuur.".to_string(),
                                        event_type: TimelineEventType::Besluit,
                                        url: Some("https://www.rijksoverheid.nl".to_string()),
                                    },
                                ],
                                max_items: 5,
                                context_label: Some("Fiscaal Beleid".to_string()),
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Recente Woo-documenten (open.overheid.nl)".to_string(),
                            ul { class: "document-list",
                                for (i, doc) in WOO_DOCS.iter().enumerate() {
                                    li {
                                        class: "document-item",
                                        style: if *selected_doc.read() == Some(i) { "background: #fff3e0; cursor: pointer;" } else { "cursor: pointer;" },
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
                                        li { style: "padding: 15px; background: #fff8f0; border-left: 3px solid #E17000;",
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
