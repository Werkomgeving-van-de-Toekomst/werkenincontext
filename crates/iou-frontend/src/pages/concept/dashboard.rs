//! IOU-Concept dashboard - Werken in Context met AI-tooling
//!
//! Gebaseerd op de Meerjarenplannen Digitale Informatiehuishouding
//! en Openbaarheid 2026-2030 (Rijksoverheid, VNG, IPO, UvW).

use dioxus::prelude::*;

use crate::components::{AppCard, Header, Panel, TimelineEvent, TimelineEventType, Timeline};
use crate::state::{AppState, UserInfo};
use crate::Route;

#[component]
pub fn ConceptDashboard() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    use_effect(move || {
        state.write().user = Some(UserInfo::concept());
    });

    rsx! {
        div { class: "concept",
            Header {}

            main { class: "container",
                // Context Bar
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "IOU-Modern" }
                        span { " \u{203A} " }
                        span { class: "current", "Werken in Context" }
                    }
                    span { class: "tag woo", "Meerjarenplan 2026\u{2013}2030" }
                }

                // Intro
                div { class: "concept-intro",
                    h2 { "Werken in Context met AI-tooling" }
                    p {
                        "IOU-Modern maakt de ambities van de "
                        strong { "Meerjarenplannen Digitale Informatiehuishouding en Openbaarheid 2026\u{2013}2030" }
                        " concreet: informatie duurzaam toegankelijk, actief openbaar, en compliant \u{2014} door te werken in context met AI-ondersteuning."
                    }
                }

                // Dashboard Grid
                div { class: "dashboard-grid",
                    // Left Column - Apps & Woo-verplichtingen
                    div {
                        Panel { title: "Conceptmodules".to_string(),
                            div { class: "app-grid",
                                Link { to: Route::ConceptContextModel,
                                    AppCard {
                                        name: "Context Model".to_string(),
                                        description: "Informatiedomeinen & contextlagen".to_string(),
                                        badge: "Kern".to_string(),
                                    }
                                }
                                Link { to: Route::ConceptAiTooling,
                                    AppCard {
                                        name: "AI Tooling".to_string(),
                                        description: "Metadata, classificatie & suggesties".to_string(),
                                        badge: "AI".to_string(),
                                    }
                                }
                                Link { to: Route::ConceptArchitectuur,
                                    AppCard {
                                        name: "Architectuur".to_string(),
                                        description: "Systeemcomponenten & integraties".to_string(),
                                    }
                                }
                                Link { to: Route::ConceptWerkwijze,
                                    AppCard {
                                        name: "Werkwijze".to_string(),
                                        description: "Stapsgewijze methode voor contextwerk".to_string(),
                                    }
                                }
                                Link { to: Route::ConceptMeerjarenplanConclusies,
                                    AppCard {
                                        name: "Meerjarenplan Conclusies".to_string(),
                                        description: "7 beleidsconclusies en IOU-antwoorden".to_string(),
                                        badge: "MJP".to_string(),
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Drie Woo-verplichtingen".to_string(),
                            p { style: "font-size: 0.8rem; color: #666; margin-bottom: 10px;",
                                "De Wet open overheid kent drie kernverplichtingen waarop de meerjarenplannen zijn gebouwd:"
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4E2}" }
                                div { class: "label", "Actieve openbaarmaking" }
                                div { class: "value", "GWV" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4E8}" }
                                div { class: "label", "Openbaarmaking op verzoek" }
                                div { class: "value", "Woo-verzoek" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4C1}" }
                                div { class: "label", "Informatiehuishouding op orde" }
                                div { class: "value", "Basis" }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "VNG-pilaren".to_string(),
                            p { style: "font-size: 0.8rem; color: #666; margin-bottom: 10px;",
                                "\"Grip op Informatie\" \u{2014} strategische pilaren voor gemeenten:"
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4A1}" }
                                div { class: "label", "Transparantie als standaard" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2699}" }
                                div { class: "label", "Procesherinrichting vanaf de start" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F464}" }
                                div { class: "label", "Bewust omgaan met informatie" }
                            }
                        }
                    }

                    // Center Column - Hoe IOU de ambities realiseert
                    div {
                        Panel { title: "Hoe IOU de ambities realiseert".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon concept-step", "1" }
                                    div { class: "document-info",
                                        h4 { "Informatiehuishouding op orde" }
                                        div { class: "meta", "Informatiedomeinen structureren informatie \u{2014} niet technologie maar werkwijze is leidend (IPO)" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon concept-step", "2" }
                                    div { class: "document-info",
                                        h4 { "Duurzaam toegankelijk" }
                                        div { class: "meta", "Context zorgt dat documenten, e-mails en chats vindbaar en gearchiveerd blijven" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon concept-step", "3" }
                                    div { class: "document-info",
                                        h4 { "AI automatiseert verrijking" }
                                        div { class: "meta", "Innovatie-agenda: AI voor metadata, classificatie en Woo-afhandeling" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon concept-step", "4" }
                                    div { class: "document-info",
                                        h4 { "Actief openbaar maken" }
                                        div { class: "meta", "Generieke Woo-voorziening (GWV) en Zoek & Vind 2.0 ge\u{00ef}ntegreerd" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon concept-step", "5" }
                                    div { class: "document-info",
                                        h4 { "Compliant by design" }
                                        div { class: "meta", "Woo, AVG, Archiefwet (nieuw per 2027) \u{2014} ingebouwd, geen extra stap" }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Meerjarenplannen per bestuurslaag".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F3DB}" }
                                    div { class: "document-info",
                                        h4 { "Rijksoverheid 2026\u{2013}2030" }
                                        div { class: "meta", "Openbaarheid en Informatiehuishouding \u{2022} BZK" }
                                    }
                                    span { class: "tag woo", "MJP" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F3E2}" }
                                    div { class: "document-info",
                                        h4 { "Gemeenten (VNG) 2026\u{2013}2030" }
                                        div { class: "meta", "\"Grip op Informatie\" \u{2022} Strategisch Meerjarenplan" }
                                    }
                                    span { class: "tag woo", "MJP" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F3DB}" }
                                    div { class: "document-info",
                                        h4 { "Provincies (IPO) 2026\u{2013}2030" }
                                        div { class: "meta", "Digitale Informatiehuishouding \u{2022} Mens centraal" }
                                    }
                                    span { class: "tag woo", "MJP" }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F30A}" }
                                    div { class: "document-info",
                                        h4 { "Waterschappen (UvW) 2026\u{2013}2030" }
                                        div { class: "meta", "Digitale Informatiehuishouding \u{2022} UvW" }
                                    }
                                    span { class: "tag woo", "MJP" }
                                }
                            }
                        }
                    }

                    // Right Column - Rijksvoorzieningen & Innovatie
                    div {
                        Panel { title: "Rijksvoorzieningen 2026".to_string(),
                            p { style: "font-size: 0.8rem; color: #666; margin-bottom: 10px;",
                                "Concrete voorzieningen uit het meerjarenplan:"
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F310}" }
                                div { class: "label", "Generieke Woo-voorziening" }
                                div { class: "value", "GWV" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4E7}" }
                                div { class: "label", "E-mailarchivering" }
                                div { class: "value", "Zomer '26" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4AC}" }
                                div { class: "label", "Chatarchivering" }
                                div { class: "value", "2026" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F50D}" }
                                div { class: "label", "Zoek & Vind 2.0" }
                                div { class: "value", "Eind '26" }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Innovatie-agenda (AI)".to_string(),
                            p { style: "font-size: 0.8rem; color: #666; margin-bottom: 10px;",
                                "Samen met medeoverheden werkt BZK aan AI-oplossingen voor:"
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F916}" }
                                div { class: "label", "Automatisering informatiehuishouding" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F916}" }
                                div { class: "label", "Versnelling Woo-verzoeken" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F916}" }
                                div { class: "label", "Actieve openbaarmaking" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F916}" }
                                div { class: "label", "Datakwaliteit & -beheer" }
                            }
                            p { style: "font-size: 0.8rem; color: #666; margin-top: 10px;",
                                "Alle AI-suggesties worden ter review aangeboden \u{2014} de mens beslist altijd."
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Tijdlijn: IOU-Modern".to_string(),
                            Timeline {
                                title: String::new(),
                                events: vec![
                                    TimelineEvent {
                                        id: "1".to_string(),
                                        title: "Nieuwe Archiefwet van kracht".to_string(),
                                        date: "2027-01-01".to_string(),
                                        date_display: "1 jan 2027".to_string(),
                                        description: "De nieuwe Archiefwet vervangt de Archiefwet 1995. Digitale duurzame bewaring wordt verplicht.".to_string(),
                                        event_type: TimelineEventType::ProjectMilestone,
                                        url: None,
                                    },
                                    TimelineEvent {
                                        id: "2".to_string(),
                                        title: "E-mailarchivering rijksbreed".to_string(),
                                        date: "2026-07-01".to_string(),
                                        date_display: "Zomer 2026".to_string(),
                                        description: "Rijksbrede voorziening voor automatische e-mailarchivering operationeel.".to_string(),
                                        event_type: TimelineEventType::Email,
                                        url: None,
                                    },
                                    TimelineEvent {
                                        id: "3".to_string(),
                                        title: "Generieke Woo-voorziening (GWV) live".to_string(),
                                        date: "2026-06-01".to_string(),
                                        date_display: "Juni 2026".to_string(),
                                        description: "De Generieke Woo-voorziening maakt actieve openbaarmaking mogelijk voor alle overheidsorganisaties.".to_string(),
                                        event_type: TimelineEventType::ProjectMilestone,
                                        url: None,
                                    },
                                    TimelineEvent {
                                        id: "4".to_string(),
                                        title: "Meerjarenplan Openbaarheid 2026-2030".to_string(),
                                        date: "2026-01-01".to_string(),
                                        date_display: "Januari 2026".to_string(),
                                        description: "Publicatie van de meerjarenplannen voor alle bestuurslagen (Rijk, VNG, IPO, UvW).".to_string(),
                                        event_type: TimelineEventType::Document,
                                        url: Some("https://www.rijksoverheid.nl".to_string()),
                                    },
                                    TimelineEvent {
                                        id: "5".to_string(),
                                        title: "Wet open overheid volledig in werking".to_string(),
                                        date: "2022-05-01".to_string(),
                                        date_display: "Mei 2022".to_string(),
                                        description: "De Woo volledig in werking getreden voor bestuurslagen.".to_string(),
                                        event_type: TimelineEventType::Besluit,
                                        url: None,
                                    },
                                ],
                                max_items: 5,
                                context_label: Some("Meerjarenplan 2026-2030".to_string()),
                            }
                        }
                    }
                }
            }
        }
    }
}
