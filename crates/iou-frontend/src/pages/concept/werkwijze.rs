//! Werkwijze - de drie Woo-verplichtingen in de praktijk
//!
//! De meerjarenplannen vertalen de drie Woo-verplichtingen naar een integrale
//! werkwijze: actieve openbaarmaking, openbaarmaking op verzoek, en
//! informatiehuishouding op orde. VNG: "Procesherinrichting vanaf de start."

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn ConceptWerkwijze() -> Element {
    let mut active_fase = use_signal(|| 0usize);
    let idx = *active_fase.read();

    rsx! {
        div { class: "concept",
            Header {}
            main { class: "container",
                div { class: "concept-intro",
                    h2 { "Werkwijze: Van verplichting naar praktijk" }
                    p {
                        "De drie Woo-verplichtingen vormen de kern: "
                        strong { "actieve openbaarmaking, openbaarmaking op verzoek, informatiehuishouding op orde" }
                        ". Vijf fasen maken dit concreet \u{2014} met AI-ondersteuning."
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 2fr; gap: 20px;",
                    // Left: fase selector
                    Panel { title: "Fasen".to_string(),
                        ul { class: "document-list",
                            li {
                                class: "document-item",
                                style: if idx == 0 { "background: #f0ebff;" } else { "" },
                                onclick: move |_| active_fase.set(0),
                                div { class: "document-icon concept-step", "1" }
                                div { class: "document-info",
                                    h4 { "Context Defini\u{00eb}ren" }
                                    div { class: "meta", "Basis: informatiehuishouding op orde" }
                                }
                            }
                            li {
                                class: "document-item",
                                style: if idx == 1 { "background: #f0ebff;" } else { "" },
                                onclick: move |_| active_fase.set(1),
                                div { class: "document-icon concept-step", "2" }
                                div { class: "document-info",
                                    h4 { "Informatie Verzamelen" }
                                    div { class: "meta", "Documenten, e-mail, chat in context" }
                                }
                            }
                            li {
                                class: "document-item",
                                style: if idx == 2 { "background: #f0ebff;" } else { "" },
                                onclick: move |_| active_fase.set(2),
                                div { class: "document-icon concept-step", "3" }
                                div { class: "document-info",
                                    h4 { "AI Verrijking" }
                                    div { class: "meta", "Innovatie-agenda: automatisering" }
                                }
                            }
                            li {
                                class: "document-item",
                                style: if idx == 3 { "background: #f0ebff;" } else { "" },
                                onclick: move |_| active_fase.set(3),
                                div { class: "document-icon concept-step", "4" }
                                div { class: "document-info",
                                    h4 { "Actief Openbaar Maken" }
                                    div { class: "meta", "Woo-verplichting 1: GWV integratie" }
                                }
                            }
                            li {
                                class: "document-item",
                                style: if idx == 4 { "background: #f0ebff;" } else { "" },
                                onclick: move |_| active_fase.set(4),
                                div { class: "document-icon concept-step", "5" }
                                div { class: "document-info",
                                    h4 { "Verzoek & Verantwoording" }
                                    div { class: "meta", "Woo-verplichting 2: op verzoek" }
                                }
                            }
                        }
                    }

                    // Right: fase detail
                    div {
                        {match idx {
                            0 => rsx! {
                                Panel { title: "Fase 1: Context Defini\u{00eb}ren (Informatiehuishouding op orde)".to_string(),
                                    div { style: "padding: 10px 0;",
                                        p { style: "margin-bottom: 15px; color: #666;",
                                            "IPO: \"Niet technologie is leidend, maar de manier waarop mensen met informatie omgaan.\" Begin met een informatiedomein \u{2014} de basis voor geordende informatiehuishouding (Woo-verplichting 3)."
                                        }
                                        h4 { style: "margin-bottom: 10px;", "Activiteiten" }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{2713}" }
                                            div { class: "label", "Kies informatiedomein (bijv. beleidsthema, project)" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{2713}" }
                                            div { class: "label", "Definieer scope en afbakening (VNG: procesherinrichting)" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{2713}" }
                                            div { class: "label", "Identificeer relevante wet- en regelgeving (Woo, AVG, Archiefwet)" }
                                        }
                                        h4 { style: "margin: 15px 0 10px;", "MJP-koppeling" }
                                        p { style: "font-size: 0.875rem; color: #666;",
                                            span { class: "tag woo", "Verplichting 3" }
                                            " Informatiehuishouding op orde \u{2014} de basis waarop andere verplichtingen voortbouwen."
                                        }
                                    }
                                }
                            },
                            1 => rsx! {
                                Panel { title: "Fase 2: Informatie Verzamelen (Duurzaam toegankelijk)".to_string(),
                                    div { style: "padding: 10px 0;",
                                        p { style: "margin-bottom: 15px; color: #666;",
                                            "Voeg documenten, e-mail, chat en datasets toe aan het domein. Het meerjarenplan verplicht e-mail- en chatarchivering (beschikbaar zomer 2026)."
                                        }
                                        h4 { style: "margin-bottom: 10px;", "Activiteiten" }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{2713}" }
                                            div { class: "label", "Upload documenten (besluiten, brieven, rapporten)" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F4E7}" }
                                            div { class: "label", "Koppel e-mailarchief (Rijksvoorziening, zomer 2026)" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F4AC}" }
                                            div { class: "label", "Koppel chatarchief (Rijksbrede voorziening 2026)" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{2713}" }
                                            div { class: "label", "Registreer stakeholders en hun rollen" }
                                        }
                                        h4 { style: "margin: 15px 0 10px;", "MJP-koppeling" }
                                        p { style: "font-size: 0.875rem; color: #666;",
                                            "Duurzaam toegankelijk \u{2014} informatie blijft vindbaar en bruikbaar over de tijd (Archiefwet 2027)."
                                        }
                                    }
                                }
                            },
                            2 => rsx! {
                                Panel { title: "Fase 3: AI Verrijking (Innovatie-agenda)".to_string(),
                                    div { style: "padding: 10px 0;",
                                        p { style: "margin-bottom: 15px; color: #666;",
                                            "De Innovatie-agenda ontwikkelt AI-oplossingen voor automatisering van informatiehuishouding. AI suggereert; de mens beslist."
                                        }
                                        h4 { style: "margin-bottom: 10px;", "AI-taken uit Innovatie-agenda" }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F916}" }
                                            div { class: "label", "Automatisering informatiehuishouding" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F916}" }
                                            div { class: "label", "Versnelling Woo-verzoekafhandeling" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F916}" }
                                            div { class: "label", "Datakwaliteit en -beheer" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F916}" }
                                            div { class: "label", "Metadata, classificatie, relatie-suggesties" }
                                        }
                                        h4 { style: "margin: 15px 0 10px;", "MJP-koppeling" }
                                        p { style: "font-size: 0.875rem; color: #666;",
                                            "Samenwerking tussen Rijksoverheid, VNG, IPO en UvW aan gedeelde AI-oplossingen."
                                        }
                                    }
                                }
                            },
                            3 => rsx! {
                                Panel { title: "Fase 4: Actief Openbaar Maken (Woo-verplichting 1)".to_string(),
                                    div { style: "padding: 10px 0;",
                                        p { style: "margin-bottom: 15px; color: #666;",
                                            "Actieve openbaarmaking wordt de standaard (VNG-pilaar 1). Via de Generieke Woo-voorziening (GWV) en Zoek & Vind 2.0 wordt informatie proactief ontsloten."
                                        }
                                        h4 { style: "margin-bottom: 10px;", "Tools & Voorzieningen" }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F310}" }
                                            div { class: "label", "Generieke Woo-voorziening (GWV)" }
                                            div { class: "value", "Rijksbreed" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F50D}" }
                                            div { class: "label", "Zoek & Vind 2.0" }
                                            div { class: "value", "Eind 2026" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F517}" }
                                            div { class: "label", "GraphRAG voor contextueel zoeken" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F916}" }
                                            div { class: "label", "AI suggereert openbaar te maken info" }
                                        }
                                        h4 { style: "margin: 15px 0 10px;", "MJP-koppeling" }
                                        p { style: "font-size: 0.875rem; color: #666;",
                                            span { class: "tag woo", "Verplichting 1" }
                                            " Actieve openbaarmaking \u{2014} transparantie als standaard."
                                        }
                                    }
                                }
                            },
                            _ => rsx! {
                                Panel { title: "Fase 5: Verzoek & Verantwoording (Woo-verplichting 2)".to_string(),
                                    div { style: "padding: 10px 0;",
                                        p { style: "margin-bottom: 15px; color: #666;",
                                            "Openbaarmaking op verzoek met verbeterde doorlooptijden. Genereer compliant documenten en verantwoord je handelen \u{2014} alles binnen de context."
                                        }
                                        h4 { style: "margin-bottom: 10px;", "Activiteiten" }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F4C4}" }
                                            div { class: "label", "Genereer compliant documenten via wizard" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{2713}" }
                                            div { class: "label", "Automatische Woo/AVG/Archiefwet check" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F4CA}" }
                                            div { class: "label", "Prestatie-indicatoren (VNG) gemonitord" }
                                        }
                                        div { class: "compliance-indicator ok",
                                            div { class: "icon", "\u{1F4C1}" }
                                            div { class: "label", "Audit trail voor verantwoording" }
                                        }
                                        h4 { style: "margin: 15px 0 10px;", "MJP-koppeling" }
                                        p { style: "font-size: 0.875rem; color: #666;",
                                            span { class: "tag woo", "Verplichting 2" }
                                            " Openbaarmaking op verzoek \u{2014} verbeterde verwerkingstijden."
                                        }
                                    }
                                }
                            },
                        }}
                    }
                }
            }
        }
    }
}
