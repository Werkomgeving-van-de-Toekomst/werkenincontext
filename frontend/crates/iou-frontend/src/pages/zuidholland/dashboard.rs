//! Zuid-Holland dashboard page

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
        titel: "Omgevingsvergunning stremmen Delftse Schie \u{2014} Warmtenet Delft",
        samenvatting: "Aanvraag omgevingsvergunning voor het stremmen van de Delftse Schie ten behoeve van het uitvoeren van werkzaamheden aan Warmtenet Delft. De stremming is nodig voor de aanleg van warmteleidingen onder de vaarweg.",
        datum: "6 feb 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-2027",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-2027.html",
    },
    WooDoc {
        titel: "Beschikking Wet Natuurbescherming Zuid-Holland",
        samenvatting: "Beschikking wet Natuurbescherming voor de gehele provincie Zuid-Holland, met uitzondering van natuurgebieden in beheer bij Natuurmonumenten, Staatsbosbeheer en Zuid-Hollands Landschap, Natura 2000-gebieden en ganzen-rustgebieden.",
        datum: "6 feb 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-2033",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-2033.html",
    },
    WooDoc {
        titel: "Omgevingsvergunning stremmen Rijn-Schiekanaal 14 mei 2026",
        samenvatting: "Aanvraag omgevingsvergunning voor het stremmen en belemmeren van de scheepvaart op het Rijn-Schiekanaal op 14 mei 2026. Betreft werkzaamheden aan de provinciale vaarweg.",
        datum: "6 feb 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-2031",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-2031.html",
    },
    WooDoc {
        titel: "Herstel telecommunicatiekabel N209 Bleiswijk",
        samenvatting: "Aanvraag omgevingsvergunning voor het herstellen van een telecommunicatiekabel langs de N209 Overbuurtseweg te Bleiswijk. Betreft kabelherstel in de provinciale wegberm.",
        datum: "6 feb 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1972",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1972.html",
    },
    WooDoc {
        titel: "Kennisgeving beschikking omgevingsvergunning Wassenaar",
        samenvatting: "Kennisgeving van de beschikking op een aanvraag omgevingsvergunning, Meijendelseweg 28 te Wassenaar. Het besluit is genomen in het kader van de Omgevingswet.",
        datum: "6 feb 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-2002",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-2002.html",
    },
];

#[component]
pub fn ZuidHolland() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut selected_doc = use_signal(|| None::<usize>);

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
                        Panel { title: "Recente Woo-documenten (open.overheid.nl)".to_string(),
                            ul { class: "document-list",
                                for (i, doc) in WOO_DOCS.iter().enumerate() {
                                    li {
                                        class: "document-item",
                                        style: if *selected_doc.read() == Some(i) { "background: #fce4ec; cursor: pointer;" } else { "cursor: pointer;" },
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
                                        li { style: "padding: 15px; background: #fef6f8; border-left: 3px solid #E31837;",
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
