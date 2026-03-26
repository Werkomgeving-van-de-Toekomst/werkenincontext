//! Financiële Controle app - Comptabiliteit & rechtmatigheid monitoring

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn MinFinFinancieleControle() -> Element {
    rsx! {
        div { class: "minfin",
            Header {}
            main { class: "container",
                div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-bottom: 20px;",
                    Panel { title: "Comptabiliteitswet".to_string(),
                        div { style: "text-align: center; padding: 20px;",
                            div { style: "font-size: 3rem; font-weight: bold; color: #4CAF50;", "97%" }
                            p { style: "color: #666;", "2 rapporten vereisen actie" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Verantwoord conform wet" }
                            div { class: "value", "284" }
                        }
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "!" }
                            div { class: "label", "Wacht op beoordeling" }
                            div { class: "value", "2" }
                        }
                    }

                    Panel { title: "Rechtmatigheid".to_string(),
                        div { style: "text-align: center; padding: 20px;",
                            div { style: "font-size: 3rem; font-weight: bold; color: #4CAF50;", "99%" }
                            p { style: "color: #666;", "1 onrechtmatige betaling" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Rechtmatig bevonden" }
                            div { class: "value", "12.847" }
                        }
                        div { class: "compliance-indicator error",
                            div { class: "icon", "\u{2717}" }
                            div { class: "label", "Onrechtmatig" }
                            div { class: "value", "1" }
                        }
                    }

                    Panel { title: "EU Begrotingsregels".to_string(),
                        div { style: "text-align: center; padding: 20px;",
                            div { style: "font-size: 3rem; font-weight: bold; color: #FF9800;", "94%" }
                            p { style: "color: #666;", "EMU-saldo grens nadert" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Binnen EMU-norm" }
                            div { class: "value", "Ja" }
                        }
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "!" }
                            div { class: "label", "Marge resterend" }
                            div { class: "value", "0,3% BBP" }
                        }
                    }
                }

                Panel { title: "Acties vereist".to_string(),
                    ul { class: "document-list",
                        li { class: "document-item",
                            div { class: "document-icon", style: "background: #FF9800;", "!" }
                            div { class: "document-info",
                                h4 { "Rapport financieel beheer indienen" }
                                div { class: "meta", "Comptabiliteitswet \u{2022} Deadline: 15 maart 2026" }
                            }
                            button { class: "btn btn-primary", "Opstellen" }
                        }
                        li { class: "document-item",
                            div { class: "document-icon", style: "background: #F44336;", "\u{2717}" }
                            div { class: "document-info",
                                h4 { "Onrechtmatige betaling correctie" }
                                div { class: "meta", "Rechtmatigheid \u{2022} Bedrag: \u{20AC}42.350" }
                            }
                            button { class: "btn btn-primary", "Corrigeren" }
                        }
                        li { class: "document-item",
                            div { class: "document-icon", style: "background: #FF9800;", "!" }
                            div { class: "document-info",
                                h4 { "EMU-saldo rapportage Q1 2026" }
                                div { class: "meta", "EU Begrotingsregels \u{2022} Deadline: 1 april 2026" }
                            }
                            button { class: "btn btn-secondary", "Voorbereiden" }
                        }
                    }
                }
            }
        }
    }
}
