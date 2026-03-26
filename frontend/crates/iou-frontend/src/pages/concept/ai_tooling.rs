//! AI Tooling - Innovatie-agenda in actie
//!
//! De Innovatie-agenda ontwikkelt AI-oplossingen voor:
//! - Automatisering informatiehuishouding
//! - Versnelling Woo-verzoekafhandeling
//! - Actieve openbaarmaking
//! - Datakwaliteit en -beheer
//! - DMN/FLINT regelengine integratie

use dioxus::prelude::*;

use crate::components::{Header, Panel};

/// DMN Decision definitie voorbeeld
struct DmnVoorbeeld {
    key: &'static str,
    naam: &'static str,
    organisatie: &'static str,
    beschrijving: &'static str,
    inputs: &'static [&'static str],
    outputs: &'static [&'static str],
    endpoint: &'static str,
}

const DMN_VOORBEELDEN: &[DmnVoorbeeld] = &[
    DmnVoorbeeld {
        key: "SVB_LeeftijdsInformatie",
        naam: "SVB Leeftijds Informatie",
        organisatie: "SVB",
        beschrijving: "Bepaalt leeftijdscategorieën (jonger dan 18, 18-20, 21+, AOW) voor aanvrager en partner, inclusief AOW-datum berekening.",
        inputs: &["geboortedatumAanvrager", "geboortedatumPartner", "dagVanAanvraag"],
        outputs: &["aanvragerIsJongerDan18", "aanvragerIs181920", "aanvragerIsTenminste21", "aanvragerHeeftAOWLeeftijd", "leeftijdAanvrager", "AOWDatumAanvrager"],
        endpoint: "https://operaton.open-regels.nl/engine-rest/decision-definition/key/SVB_LeeftijdsInformatie",
    },
    DmnVoorbeeld {
        key: "Rijk_Bijstand",
        naam: "Rijk Bijstandsuitkering",
        organisatie: "Rijk",
        beschrijving: "Berekent recht op bijstand op basis van inkomen, vermogen en leefsituatie volgens Participatiewet.",
        inputs: &["inkomen", "vermogen", "leeftijd", "gezinsSituatie", "woonsituatie"],
        outputs: &["heeftRechtOpBijstand", "bijstandsbedrag", "algemeneKorting"],
        endpoint: "https://operaton.open-regels.nl/engine-rest/decision-definition/key/Rijk_Bijstand",
    },
    DmnVoorbeeld {
        key: "DUO_Studiefinanciering",
        naam: "DUO Studiefinanciering",
        organisatie: "DUO",
        beschrijving: "Bepaalt recht en hoogte van studiefinanciering op basis van opleiding, voortgang en inkomen ouders.",
        inputs: &["opleidingstype", "studievoortgang", "inkomenOuders", "woonSituatie"],
        outputs: &["heeftRechtOpOV", "ovBedrag", "aanvullendeBeurs", "tegemoetkoming"],
        endpoint: "https://operaton.open-regels.nl/engine-rest/decision-definition/key/DUO_Studiefinanciering",
    },
];

const API_EXAMPLE: &str = r#"POST https://operaton.open-regels.nl/engine-rest/decision-definition/key/SVB_LeeftijdsInformatie/evaluate
Content-Type: application/json

{
  "variables": {
    "geboortedatumAanvrager": "1990-05-15",
    "geboortedatumPartner": null,
    "dagVanAanvraag": "2026-02-20"
  }
}

// Response:
{
  "leeftijdAanvrager": 35,
  "aanvragerIsTenminste21": true,
  "aanvragerIsJongerDan18": false,
  "aanvragerHeeftAOWLeeftijd": false,
  "AOWDatumAanvrager": "2057-05-15"
}"#;

#[component]
pub fn ConceptAiTooling() -> Element {
    let mut tab = use_signal(|| 0i32);  // 0 = Pipeline, 1 = DMN Regels
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
                        ". Doorloop de pipeline of bekijk de DMN regelengine."
                    }
                }

                // Tab buttons
                div { style: "display: flex; gap: 10px; margin-bottom: 20px;",
                    button {
                        class: if tab() == 0 { "btn btn-primary" } else { "btn btn-secondary" },
                        onclick: move |_| tab.set(0),
                        "AI Pipeline"
                    }
                    button {
                        class: if tab() == 1 { "btn btn-primary" } else { "btn btn-secondary" },
                        onclick: move |_| tab.set(1),
                        "DMN/FLINT Regels"
                    }
                }

                if tab() == 0 {
                    // AI Pipeline Tab
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
                } else {
                    // DMN/FLINT Tab
                    Panel { title: "DMN/FLINT: Regelengine voor Overheidsbesluiten".to_string(),
                            p { style: "margin-bottom: 20px; color: #666;",
                                "DMN (Decision Model and Notation) en FLINT (Formalised Legislation Integration Tool) "
                                "maken het mogelijk om wetgeving en regels uitvoerbaar te maken als besluitregels. "
                                "De Open Regels infrastructuur biedt een centrale regelengine."
                            }

                            // Introductie DMN/FLINT
                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 30px;",
                                div { style: "background: #f8f6ff; padding: 20px; border-radius: 8px; border-left: 4px solid #5B3CC4;",
                                    h4 { style: "margin-top: 0;", "\u{1F916} DMN - Decision Model and Notation" }
                                    p { style: "font-size: 0.9rem; color: #666; margin-bottom: 10px;",
                                        "OMG standaard voor het modelleren en uitvoeren van besluiten. Beslissingstabellen maken regels expliciet en reproduceerbaar."
                                    }
                                    ul { style: "font-size: 0.85rem; color: #666; padding-left: 20px;",
                                        li { "Decision Tables" }
                                        li { "Input/Output variabelen" }
                                        li { "Hit Policy (ALL, ANY, FIRST)" }
                                        li { "FEEL expressies" }
                                    }
                                }
                                div { style: "background: #fff8f0; padding: 20px; border-radius: 8px; border-left: 4px solid #E17000;",
                                    h4 { style: "margin-top: 0;", "\u{2696} FLINT - Formalised Legislation" }
                                    p { style: "font-size: 0.9rem; color: #666; margin-bottom: 10px;",
                                        "Instrument om wetgeving te formaliseren en uitvoerbaar te maken. Vertaalt wettekst naar executable regels."
                                    }
                                    ul { style: "font-size: 0.85rem; color: #666; padding-left: 20px;",
                                        li { "Wettekst parsing" }
                                        li { "Regel extractie" }
                                        li { "Validatie gegenereerde DMN" }
                                        li { "Versiebeheer wetgeving" }
                                    }
                                }
                            }

                            // DMN Voorbeelden
                            h3 { style: "margin-bottom: 15px;", "Beschikbare DMN Regels (Open Regels)" }
                            div { style: "display: flex; flex-direction: column; gap: 15px; margin-bottom: 30px;",
                                for dmn in DMN_VOORBEELDEN {
                                    div { style: "background: #f8f9fa; padding: 20px; border-radius: 8px; border: 1px solid #e0e0e0;",
                                        div { style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 10px;",
                                            div {
                                                h4 { style: "margin: 0; color: #333;",
                                                    span { class: "tag woo", "{dmn.organisatie}" }
                                                    " {dmn.naam}"
                                                }
                                                p { style: "margin: 8px 0 0 0; font-size: 0.9rem; color: #666;",
                                                    "{dmn.beschrijving}"
                                                }
                                            }
                                            a {
                                                href: "{dmn.endpoint}",
                                                target: "_blank",
                                                class: "btn btn-small btn-outline",
                                                style: "text-decoration: none;",
                                                "Bekijk DMN \u{2197}"
                                            }
                                        }
                                        div { style: "margin-top: 15px; display: flex; gap: 20px; flex-wrap: wrap;",
                                            div { style: "flex: 1; min-width: 250px;",
                                                div { style: "font-size: 0.8rem; font-weight: 600; color: #888; margin-bottom: 5px;", "INPUTS" }
                                                div { style: "display: flex; flex-wrap: wrap; gap: 5px;",
                                                    for input in &dmn.inputs {
                                                        span { class: "tag info", "{input}" }
                                                    }
                                                }
                                            }
                                            div { style: "flex: 1; min-width: 250px;",
                                                div { style: "font-size: 0.8rem; font-weight: 600; color: #888; margin-bottom: 5px;", "OUTPUTS" }
                                                div { style: "display: flex; flex-wrap: wrap; gap: 5px;",
                                                    for output in &dmn.outputs {
                                                        span { class: "tag success", "{output}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Voordelen
                            h3 { style: "margin-bottom: 15px;", "Voordelen van DMN/FLINT voor IOU" }
                            div { style: "display: grid; grid-template-columns: repeat(2, 1fr); gap: 15px;",
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{270D}" }
                                    div { class: "label", "Reproduceerbaar" }
                                    div { class: "value", "Zelfde besluit" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F504}" }
                                    div { class: "label", "Versiebeheer" }
                                    div { class: "value", "Tracerbaar" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F4A1}" }
                                    div { class: "label", "Transparant" }
                                    div { class: "value", "Uitlegbaar" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{26A1}" }
                                    div { class: "label", "Snel" }
                                    div { class: "value", "Real-time" }
                                }
                            }

                            // API Integratie voorbeeld
                            div { style: "margin-top: 30px; background: #f0f8ff; padding: 20px; border-radius: 8px;",
                                h4 { style: "margin-top: 0;", "API Integratie Voorbeeld" }
                                p { style: "font-size: 0.9rem; color: #666; margin-bottom: 15px;",
                                    "De Open Regels engine kan direct aangeroepen worden vanuit applicaties:"
                                }
                                pre { style: "background: #2d2d2d; color: #f8f8f2; padding: 15px; border-radius: 4px; overflow-x: auto; font-size: 0.85rem;",
                                    "{API_EXAMPLE}"
                                }
                            }

                            // Links
                            div { style: "margin-top: 30px;",
                                h4 { "Meer informatie" }
                                ul { class: "document-list",
                                    li { class: "document-item",
                                        div { class: "document-icon", "\u{1F517}" }
                                        div { class: "document-info",
                                            a {
                                                href: "https://open-regels.nl",
                                                target: "_blank",
                                                style: "text-decoration: none; color: inherit;",
                                                h4 { "Open Regels Platform" }
                                            }
                                            div { class: "meta", "Centrale repository voor DMN regels" }
                                        }
                                    }
                                    li { class: "document-item",
                                        div { class: "document-icon", "\u{1F517}" }
                                        div { class: "document-info",
                                            a {
                                                href: "https://github.com/Open-Regels",
                                                target: "_blank",
                                                style: "text-decoration: none; color: inherit;",
                                                h4 { "Open Regels GitHub" }
                                            }
                                            div { class: "meta", "Open source code en documentatie" }
                                        }
                                    }
                                    li { class: "document-item",
                                        div { class: "document-icon", "\u{1F517}" }
                                        div { class: "document-info",
                                            a {
                                                href: "https://www.omg.org/spec/DMN/",
                                                target: "_blank",
                                                style: "text-decoration: none; color: inherit;",
                                                h4 { "DMN Specificatie (OMG)" }
                                            }
                                            div { class: "meta", "Officiële DMN standaard" }
                                        }
                                    }
                                }
                            }
                    }
                }
            }
        }
    }
}
