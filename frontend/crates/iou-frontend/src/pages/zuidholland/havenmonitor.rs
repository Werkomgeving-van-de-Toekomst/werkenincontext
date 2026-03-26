//! Havenmonitor - Rotterdam Haven dashboard

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn ZHHavenmonitor() -> Element {
    rsx! {
        div { class: "zuidholland",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Zuid-Holland" }
                        span { " \u{203A} " }
                        span { class: "current", "Havenmonitor Rotterdam" }
                    }
                }

                // KPI rij bovenaan
                div { style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 20px;",
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 20px;",
                            div { style: "font-size: 2rem; font-weight: 700; color: #E31837;", "438,8" }
                            div { style: "font-size: 0.875rem; color: #666;", "Mton overslag 2025" }
                            div { style: "font-size: 0.75rem; color: #4CAF50; margin-top: 4px;", "\u{2191} +2,1% t.o.v. 2024" }
                        }
                    }
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 20px;",
                            div { style: "font-size: 2rem; font-weight: 700; color: #E31837;", "28.450" }
                            div { style: "font-size: 0.875rem; color: #666;", "Zeeschepen 2025" }
                            div { style: "font-size: 0.75rem; color: #4CAF50; margin-top: 4px;", "\u{2191} +1,5% t.o.v. 2024" }
                        }
                    }
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 20px;",
                            div { style: "font-size: 2rem; font-weight: 700; color: #E31837;", "14,8M" }
                            div { style: "font-size: 0.875rem; color: #666;", "TEU containers" }
                            div { style: "font-size: 0.75rem; color: #FF9800; margin-top: 4px;", "\u{2192} -0,3% t.o.v. 2024" }
                        }
                    }
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 20px;",
                            div { style: "font-size: 2rem; font-weight: 700; color: #E31837;", "385K" }
                            div { style: "font-size: 0.875rem; color: #666;", "Directe banen" }
                            div { style: "font-size: 0.75rem; color: #4CAF50; margin-top: 4px;", "\u{2191} +800 t.o.v. 2024" }
                        }
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                    Panel { title: "Overslag per goederensoort".to_string(),
                        div { class: "chart-container", style: "display: flex; flex-direction: column; justify-content: center; gap: 8px;",
                            div { class: "bar-row",
                                span { class: "bar-label", "Droog bulk" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 75%;" }
                                }
                                span { class: "bar-value", "75,2 Mt" }
                            }
                            div { class: "bar-row",
                                span { class: "bar-label", "Nat bulk" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 92%;" }
                                }
                                span { class: "bar-value", "195,3 Mt" }
                            }
                            div { class: "bar-row",
                                span { class: "bar-label", "Containers" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 65%;" }
                                }
                                span { class: "bar-value", "130,1 Mt" }
                            }
                            div { class: "bar-row",
                                span { class: "bar-label", "RoRo" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 25%;" }
                                }
                                span { class: "bar-value", "28,4 Mt" }
                            }
                            div { class: "bar-row",
                                span { class: "bar-label", "Overig" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 8%;" }
                                }
                                span { class: "bar-value", "9,8 Mt" }
                            }
                        }
                    }

                    Panel { title: "Energietransitie Haven".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Walstroomaansluitingen" }
                            div { class: "value", "12/15" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Waterstof-backbone" }
                            div { class: "value", "Fase 2" }
                        }
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "!" }
                            div { class: "label", "CO\u{2082}-reductie doelstelling" }
                            div { class: "value", "62%" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Wind op Maasvlakte II" }
                            div { class: "value", "Operationeel" }
                        }

                        div { style: "margin-top: 16px;",
                            p { style: "font-size: 0.875rem; color: #666;",
                                "De haven streeft naar klimaatneutraal in 2050. Huidige CO\u{2082}-uitstoot is 22,3 Mton (doel 2030: 18,5 Mton)."
                            }
                        }
                    }
                }

                div { style: "height: 20px;" }

                div { style: "display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 20px;",
                    Panel { title: "Milieu-indicatoren".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Luchtkwaliteit Europoort" }
                            div { class: "value", "Goed" }
                        }
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "!" }
                            div { class: "label", "Geluidsnorm Maasvlakte" }
                            div { class: "value", "Grens" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Waterkwaliteit Maas" }
                            div { class: "value", "Voldoende" }
                        }
                    }

                    Panel { title: "Recente meldingen".to_string(),
                        ul { class: "document-list",
                            li { class: "document-item",
                                div { class: "document-icon", "\u{26A0}" }
                                div { class: "document-info",
                                    h4 { "Stremming Calandkanaal" }
                                    div { class: "meta", "Onderhoud \u{2022} 12 feb - 14 feb" }
                                }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", "\u{1F4CB}" }
                                div { class: "document-info",
                                    h4 { "Inspectie Botlekbrug" }
                                    div { class: "meta", "Gepland \u{2022} 15 feb" }
                                }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", "\u{2705}" }
                                div { class: "document-info",
                                    h4 { "Nieuwe walstroom Europoort" }
                                    div { class: "meta", "Opgeleverd \u{2022} 3 feb" }
                                }
                            }
                        }
                    }

                    Panel { title: "Belangrijke partners".to_string(),
                        ul { class: "document-list",
                            li { class: "document-item",
                                div { class: "document-icon", style: "background: #E31837;", "\u{1F3E2}" }
                                div { class: "document-info",
                                    h4 { "Havenbedrijf Rotterdam" }
                                    div { class: "meta", "Havenexploitant" }
                                }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", style: "background: #E31837;", "\u{1F3E2}" }
                                div { class: "document-info",
                                    h4 { "Deltalinqs" }
                                    div { class: "meta", "Ondernemersvereniging" }
                                }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", style: "background: #E31837;", "\u{1F3E2}" }
                                div { class: "document-info",
                                    h4 { "DCMR Milieudienst" }
                                    div { class: "meta", "Vergunningen & handhaving" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
