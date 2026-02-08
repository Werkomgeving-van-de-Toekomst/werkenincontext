//! Stakeholder Dossier - Overzicht partners & relaties Provincie Zuid-Holland

use dioxus::prelude::*;

use crate::components::{Header, Panel};

struct Stakeholder {
    name: &'static str,
    category: &'static str,
    role: &'static str,
    domains: &'static [&'static str],
    contact_freq: &'static str,
    icon: &'static str,
}

const STAKEHOLDERS: &[Stakeholder] = &[
    Stakeholder {
        name: "Gemeente Rotterdam",
        category: "Gemeente",
        role: "Grootste gemeente, havenstad",
        domains: &["Mobiliteit", "Haven", "Economie", "Wonen"],
        contact_freq: "Wekelijks",
        icon: "\u{1F3DB}",
    },
    Stakeholder {
        name: "Gemeente Den Haag",
        category: "Gemeente",
        role: "Residentie, internationale stad",
        domains: &["Mobiliteit", "Internationaal", "Veiligheid"],
        contact_freq: "Wekelijks",
        icon: "\u{1F3DB}",
    },
    Stakeholder {
        name: "Havenbedrijf Rotterdam",
        category: "Uitvoeringsorganisatie",
        role: "Haven & industriecomplex",
        domains: &["Haven", "Economie", "Energietransitie"],
        contact_freq: "Wekelijks",
        icon: "\u{1F6A2}",
    },
    Stakeholder {
        name: "Rijkswaterstaat",
        category: "Rijksoverheid",
        role: "Infrastructuur & waterbeheer",
        domains: &["Mobiliteit", "Water", "Onderhoud"],
        contact_freq: "Wekelijks",
        icon: "\u{1F6E4}",
    },
    Stakeholder {
        name: "TU Delft",
        category: "Kennisinstelling",
        role: "Kennispartner mobiliteit & technologie",
        domains: &["Innovatie", "Mobiliteit", "Energie"],
        contact_freq: "Maandelijks",
        icon: "\u{1F393}",
    },
    Stakeholder {
        name: "Erasmus MC",
        category: "Kennisinstelling",
        role: "Gezondheidsregio Zuid-Holland",
        domains: &["Gezondheid", "Leefomgeving"],
        contact_freq: "Kwartaal",
        icon: "\u{1FA7A}",
    },
    Stakeholder {
        name: "Metropoolregio Rotterdam Den Haag",
        category: "Samenwerkingsverband",
        role: "Regionale samenwerking 23 gemeenten",
        domains: &["Mobiliteit", "Economie", "Wonen"],
        contact_freq: "Wekelijks",
        icon: "\u{1F91D}",
    },
    Stakeholder {
        name: "Hoogheemraadschap Delfland",
        category: "Waterschap",
        role: "Waterbeheer & waterkwaliteit",
        domains: &["Water", "Klimaat", "Leefomgeving"],
        contact_freq: "Maandelijks",
        icon: "\u{1F30A}",
    },
    Stakeholder {
        name: "DCMR Milieudienst Rijnmond",
        category: "Uitvoeringsorganisatie",
        role: "Milieuvergunnningen Rijnmondgebied",
        domains: &["Milieu", "Haven", "Handhaving"],
        contact_freq: "Wekelijks",
        icon: "\u{1F33F}",
    },
    Stakeholder {
        name: "ProRail",
        category: "Uitvoeringsorganisatie",
        role: "Spoorinfrastructuur",
        domains: &["Mobiliteit", "OV", "Goederenvervoer"],
        contact_freq: "Maandelijks",
        icon: "\u{1F682}",
    },
];

#[component]
pub fn ZHStakeholderDossier() -> Element {
    let mut selected_category = use_signal(|| "Alle".to_string());
    let cat = selected_category.read().clone();

    let categories = ["Alle", "Gemeente", "Rijksoverheid", "Uitvoeringsorganisatie", "Kennisinstelling", "Samenwerkingsverband", "Waterschap"];

    let filtered: Vec<&Stakeholder> = if cat == "Alle" {
        STAKEHOLDERS.iter().collect()
    } else {
        STAKEHOLDERS.iter().filter(|s| s.category == cat.as_str()).collect()
    };

    rsx! {
        div { class: "zuidholland",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Zuid-Holland" }
                        span { " \u{203A} " }
                        span { class: "current", "Stakeholder Dossier" }
                    }

                    select {
                        onchange: move |evt: Event<FormData>| {
                            selected_category.set(evt.value());
                        },
                        for cat_name in categories {
                            option { value: "{cat_name}", "{cat_name}" }
                        }
                    }
                }

                // Stats balk
                div { style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 20px;",
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 16px;",
                            div { style: "font-size: 1.75rem; font-weight: 700; color: #E31837;", "{STAKEHOLDERS.len()}" }
                            div { style: "font-size: 0.875rem; color: #666;", "Stakeholders" }
                        }
                    }
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 16px;",
                            div { style: "font-size: 1.75rem; font-weight: 700; color: #E31837;", "6" }
                            div { style: "font-size: 0.875rem; color: #666;", "Categorie\u{00EB}n" }
                        }
                    }
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 16px;",
                            div { style: "font-size: 1.75rem; font-weight: 700; color: #E31837;", "23" }
                            div { style: "font-size: 0.875rem; color: #666;", "Actieve dossiers" }
                        }
                    }
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 16px;",
                            div { style: "font-size: 1.75rem; font-weight: 700; color: #E31837;", "7" }
                            div { style: "font-size: 0.875rem; color: #666;", "Wekelijks contact" }
                        }
                    }
                }

                div { style: "display: grid; grid-template-columns: 2fr 1fr; gap: 20px;",
                    Panel { title: format!("Stakeholders ({} resultaten)", filtered.len()),
                        ul { class: "document-list",
                            for stakeholder in filtered.iter() {
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #E31837;", "{stakeholder.icon}" }
                                    div { class: "document-info",
                                        h4 { "{stakeholder.name}" }
                                        div { class: "meta", "{stakeholder.category} \u{2022} {stakeholder.role}" }
                                        div { style: "margin-top: 4px; display: flex; flex-wrap: wrap; gap: 4px;",
                                            for domain in stakeholder.domains.iter() {
                                                span { class: "tag", "{domain}" }
                                            }
                                        }
                                    }
                                    span { class: "tag woo", "{stakeholder.contact_freq}" }
                                }
                            }
                        }
                    }

                    div {
                        Panel { title: "Samenwerkingsgebieden".to_string(),
                            div { class: "chart-container", style: "display: flex; flex-direction: column; justify-content: center; gap: 8px;",
                                div { class: "bar-row",
                                    span { class: "bar-label", "Mobiliteit" }
                                    div { class: "bar-track",
                                        div { class: "bar-fill", style: "width: 80%;" }
                                    }
                                    span { class: "bar-value", "8" }
                                }
                                div { class: "bar-row",
                                    span { class: "bar-label", "Economie" }
                                    div { class: "bar-track",
                                        div { class: "bar-fill", style: "width: 50%;" }
                                    }
                                    span { class: "bar-value", "5" }
                                }
                                div { class: "bar-row",
                                    span { class: "bar-label", "Haven" }
                                    div { class: "bar-track",
                                        div { class: "bar-fill", style: "width: 40%;" }
                                    }
                                    span { class: "bar-value", "4" }
                                }
                                div { class: "bar-row",
                                    span { class: "bar-label", "Water" }
                                    div { class: "bar-track",
                                        div { class: "bar-fill", style: "width: 30%;" }
                                    }
                                    span { class: "bar-value", "3" }
                                }
                                div { class: "bar-row",
                                    span { class: "bar-label", "Energie" }
                                    div { class: "bar-track",
                                        div { class: "bar-fill", style: "width: 30%;" }
                                    }
                                    span { class: "bar-value", "3" }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Recente interacties".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4E7}" }
                                    div { class: "document-info",
                                        h4 { "Bestuurlijk overleg MRDH" }
                                        div { class: "meta", "Vandaag" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CB}" }
                                    div { class: "document-info",
                                        h4 { "Havenoverleg Q1 2026" }
                                        div { class: "meta", "Gisteren" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4E7}" }
                                    div { class: "document-info",
                                        h4 { "Werkbezoek TU Delft" }
                                        div { class: "meta", "3 dagen geleden" }
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
