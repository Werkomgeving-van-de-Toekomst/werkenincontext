//! AI Tooling - Innovatie-agenda in actie
//!
//! De Innovatie-agenda ontwikkelt AI-oplossingen voor:
//! - Automatisering informatiehuishouding
//! - Versnelling Woo-verzoekafhandeling
//! - Actieve openbaarmaking
//! - Datakwaliteit en -beheer

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn ConceptAiTooling() -> Element {
    let mut step = use_signal(|| 1i32);

    rsx! {
        div { class: "concept",
            Header {}
            main { class: "container",
                div { class: "concept-intro",
                    h2 { "AI Tooling: Innovatie-agenda" }
                    p {
                        "De meerjarenplannen ontwikkelen samen AI-oplossingen voor "
                        strong { "automatisering, Woo-verzoeken, actieve openbaarmaking en datakwaliteit" }
                        ". Doorloop de pipeline om te zien hoe."
                    }
                }

                Panel { title: "AI Pipeline: Innovatie-agenda".to_string(),
                    // Pipeline steps
                    div { style: "display: flex; gap: 10px; margin-bottom: 20px;",
                        div { class: if step() >= 1 { "tag woo" } else { "tag" }, "1. Ingest" }
                        div { class: if step() >= 2 { "tag woo" } else { "tag" }, "2. Extractie" }
                        div { class: if step() >= 3 { "tag woo" } else { "tag" }, "3. Classificatie" }
                        div { class: if step() >= 4 { "tag woo" } else { "tag" }, "4. Woo Check" }
                        div { class: if step() >= 5 { "tag woo" } else { "tag" }, "5. Actie" }
                    }

                    {match step() {
                        1 => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Stap 1: Ingest (Informatiehuishouding op orde)" }
                                p { style: "margin-bottom: 15px; color: #666;",
                                    "Nieuwe informatie komt binnen in het informatiedomein \u{2014} inclusief e-mailarchief en chatberichten (Rijksvoorzieningen 2026)."
                                }
                                div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px;",
                                    div { class: "app-card", onclick: move |_| step.set(2),
                                        h3 { "E-mail" }
                                        p { "Archivering (zomer 2026)" }
                                    }
                                    div { class: "app-card", onclick: move |_| step.set(2),
                                        h3 { "Chat" }
                                        p { "Archivering (2026)" }
                                    }
                                    div { class: "app-card", onclick: move |_| step.set(2),
                                        h3 { "Document" }
                                        p { "Besluit, rapport, brief" }
                                    }
                                }
                                p { style: "margin-top: 15px; font-size: 0.85rem; color: #666;",
                                    span { class: "tag woo", "Verplichting 3" }
                                    " Structured information collection \u{2014} basis voor verdere verwerking."
                                }
                            }
                        },
                        2 => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Stap 2: Metadata Extractie (Datakwaliteit)" }
                                p { style: "margin-bottom: 15px; color: #666;",
                                    "AI extraheert automatisch metadata \u{2014} onderdeel van de Innovatie-agenda voor datakwaliteit en -beheer."
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Onderwerp gedetecteerd" }
                                    div { class: "value", "97%" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Auteur herkend" }
                                    div { class: "value", "99%" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Datum & type" }
                                    div { class: "value", "100%" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Taal: NL/EN" }
                                    div { class: "value", "95%" }
                                }
                                p { style: "margin-top: 15px; font-size: 0.85rem; color: #666;",
                                    "Innovatie-agenda: AI voor automatisering van informatiehuishouding."
                                }
                                div { style: "display: flex; justify-content: space-between; margin-top: 15px;",
                                    button { class: "btn btn-secondary", onclick: move |_| step.set(1), "Vorige" }
                                    button { class: "btn btn-primary", onclick: move |_| step.set(3), "Volgende" }
                                }
                            }
                        },
                        3 => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Stap 3: Automatische Classificatie (Automatisering)" }
                                p { style: "margin-bottom: 15px; color: #666;",
                                    "Het document wordt geclassificeerd volgens domein, Woo-status en bewaartermijn \u{2014} klaar voor GWV publicatie."
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Informatiedomein" }
                                    div { class: "value", "94%" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Documenttype" }
                                    div { class: "value", "89%" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Tags & trefwoorden" }
                                    div { class: "value", "86%" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Bewaartermijn (Archiefwet)" }
                                    div { class: "value", "10/20/jr" }
                                }
                                p { style: "margin-top: 15px; font-size: 0.85rem; color: #666;",
                                    "Klaar voor actieve openbaarmaking via Generieke Woo-voorziening."
                                }
                                div { style: "display: flex; justify-content: space-between; margin-top: 15px;",
                                    button { class: "btn btn-secondary", onclick: move |_| step.set(2), "Vorige" }
                                    button { class: "btn btn-primary", onclick: move |_| step.set(4), "Volgende" }
                                }
                            }
                        },
                        4 => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Stap 4: Woo Check (Actieve openbaarmaking)" }
                                p { style: "margin-bottom: 15px; color: #666;",
                                    "AI bepaalt of het document actief openbaar moet worden (Woo-verplichting 1) en bereidt GWV-publicatie voor."
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F4E2}" }
                                    div { class: "label", "Woo-classificatie" }
                                    div { class: "value", "Openbaar" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{2713}" }
                                    div { class: "label", "AVG-check (geen PII)" }
                                    div { class: "value", "Schoon" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F4C5}" }
                                    div { class: "label", "Archiefwet 2027" }
                                    div { class: "value", "Conform" }
                                }
                                div { class: "compliance-indicator warning",
                                    div { class: "icon", "!" }
                                    div { class: "label", "Openbaarmakingstermijn" }
                                    div { class: "value", "GWV" }
                                }
                                p { style: "margin-top: 15px; font-size: 0.85rem; color: #666;",
                                    span { class: "tag woo", "Verplichting 1" }
                                    " Transparantie als standaard (VNG-pilaar 1)."
                                }
                                div { style: "display: flex; justify-content: space-between; margin-top: 15px;",
                                    button { class: "btn btn-secondary", onclick: move |_| step.set(3), "Vorige" }
                                    button { class: "btn btn-primary", onclick: move |_| step.set(5), "Volgende" }
                                }
                            }
                        },
                        _ => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Stap 5: Actie & Suggesties (Versnelling Woo-verzoeken)" }
                                p { style: "margin-bottom: 15px; color: #666;",
                                    "AI doet concrete suggesties die de medewerker kan accepteren \u{2014} voor snellere Woo-verzoekafhandeling en betere datakwaliteit."
                                }
                                ul { class: "document-list",
                                    li { class: "document-item",
                                        div { class: "document-icon", style: "background: #7C4DFF;", "\u{1F4E2}" }
                                        div { class: "document-info",
                                            h4 { "Publiceer naar GWV" }
                                            div { class: "meta", "Actieve openbaarmaking \u{2022} 95% relevantie" }
                                        }
                                        button { class: "btn btn-primary", "Publiceer" }
                                    }
                                    li { class: "document-item",
                                        div { class: "document-icon", style: "background: #7C4DFF;", "\u{1F517}" }
                                        div { class: "document-info",
                                            h4 { "Koppel aan domein & stakeholders" }
                                            div { class: "meta", "Relatie-suggestie \u{2022} 91% match" }
                                        }
                                        button { class: "btn btn-primary", "Koppel" }
                                    }
                                    li { class: "document-item",
                                        div { class: "document-icon", style: "background: #7C4DFF;", "\u{1F916}" }
                                        div { class: "document-info",
                                            h4 { "Datakwaliteit verbeteren" }
                                            div { class: "meta", "Ontbrekende metadata aanvullen \u{2022} 87% zekerheid" }
                                        }
                                        button { class: "btn btn-primary", "Accepteer" }
                                    }
                                }
                                p { style: "margin-top: 15px; font-size: 0.85rem; color: #666;",
                                    "De mens beslist altijd \u{2014} AI ondersteunt, niet vervangt."
                                }
                                div { style: "display: flex; justify-content: flex-start; margin-top: 15px;",
                                    button { class: "btn btn-secondary", onclick: move |_| step.set(4), "Vorige" }
                                }
                            }
                        },
                    }}
                }
            }
        }
    }
}
