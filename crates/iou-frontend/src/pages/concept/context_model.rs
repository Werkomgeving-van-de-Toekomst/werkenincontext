//! Context Model - informatiedomeinen als basis voor de Woo-verplichtingen
//!
//! Het IPO stelt: "Niet technologie of systemen zijn leidend, maar de manier
//! waarop mensen met informatie omgaan." Dit contextmodel maakt dat concreet.

use dioxus::prelude::*;

use crate::components::{Header, Panel};

/// Contextlaag met beschrijving en MJP-koppelingen
struct ContextLaag {
    naam: &'static str,
    beschrijving: &'static str,
    voorbeelden: &'static [&'static str],
    impact: u32,
    /// Koppelingen naar Meerjarenplan-conclusies
    mjp_koppelingen: &'static [MjpKoppeling],
}

struct MjpKoppeling {
    conclusie_nr: u8,
    conclusie_titel: &'static str,
    relatie: &'static str,
}

const LAGEN: &[ContextLaag] = &[
    ContextLaag {
        naam: "Informatiedomein",
        beschrijving: "Het overkoepelende beleidsthema \u{2014} basis voor informatiehuishouding op orde (MJP-verplichting 3)",
        voorbeelden: &["Duurzaamheid & Energie", "Rijksbegroting 2026", "Omgevingsvisie"],
        impact: 95,
        mjp_koppelingen: &[
            MjpKoppeling {
                conclusie_nr: 1,
                conclusie_titel: "Informatiehuishouding op orde",
                relatie: "Informatiedomeinen zijn de structurerende eenheid waarmee informatiehuishouding \"op orde\" komt",
            },
            MjpKoppeling {
                conclusie_nr: 6,
                conclusie_titel: "Nieuwe Archiefwet 2027",
                relatie: "Archivering by design: classificatie en metadata worden bij domein-creatie ingericht",
            },
        ],
    },
    ContextLaag {
        naam: "Documenten",
        beschrijving: "Besluiten, brieven, e-mails en chats \u{2014} duurzaam toegankelijk conform e-mail- en chatarchivering (MJP 2026)",
        voorbeelden: &["Besluit subsidie", "Kamerbrief", "E-mailarchief", "Chatberichten"],
        impact: 88,
        mjp_koppelingen: &[
            MjpKoppeling {
                conclusie_nr: 2,
                conclusie_titel: "Duurzame toegankelijkheid",
                relatie: "Documenten blijven vindbaar en bruikbaar over de tijd binnen hun informatiedomein",
            },
            MjpKoppeling {
                conclusie_nr: 4,
                conclusie_titel: "E-mail en chatarchivering",
                relatie: "Ingest-koppelingen plaatsen e-mail en chat automatisch in het juiste domein",
            },
        ],
    },
    ContextLaag {
        naam: "Stakeholders",
        beschrijving: "Organisaties en personen \u{2014} alle bestuurslagen werken samen (Rijksoverheid, VNG, IPO, UvW)",
        voorbeelden: &["Gemeente Almere", "Belastingdienst", "CPB", "Waterschap"],
        impact: 82,
        mjp_koppelingen: &[
            MjpKoppeling {
                conclusie_nr: 5,
                conclusie_titel: "Governance en prestatie-indicatoren",
                relatie: "Stakeholders en hun rollen zijn zichtbaar \u{2014} basis voor governance en verantwoording",
            },
        ],
    },
    ContextLaag {
        naam: "Openbaarheid",
        beschrijving: "Actieve openbaarmaking (MJP-verplichting 1) en openbaarmaking op verzoek (verplichting 2) via GWV",
        voorbeelden: &["Woo", "AVG", "Archiefwet 2027", "GWV", "Zoek & Vind 2.0"],
        impact: 90,
        mjp_koppelingen: &[
            MjpKoppeling {
                conclusie_nr: 3,
                conclusie_titel: "Actieve openbaarmaking",
                relatie: "Publicatieworkflow koppelt domeinen aan GWV en Zoek & Vind 2.0",
            },
            MjpKoppeling {
                conclusie_nr: 6,
                conclusie_titel: "Nieuwe Archiefwet 2027",
                relatie: "Bewaartermijnen en openbaarheidsstatus worden per document bijgehouden",
            },
        ],
    },
    ContextLaag {
        naam: "AI-verrijking",
        beschrijving: "Innovatie-agenda: AI voor automatisering informatiehuishouding, Woo-verzoeken en datakwaliteit",
        voorbeelden: &["Metadata-extractie", "Classificatie", "Woo-suggesties", "Relatie-ontdekking"],
        impact: 75,
        mjp_koppelingen: &[
            MjpKoppeling {
                conclusie_nr: 7,
                conclusie_titel: "AI innovatie-agenda",
                relatie: "AI-pipeline biedt metadata, classificatie en relatie-detectie \u{2014} de mens beslist",
            },
            MjpKoppeling {
                conclusie_nr: 5,
                conclusie_titel: "Governance en prestatie-indicatoren",
                relatie: "AI monitort KPI\u{2019}s en genereert compliance-rapportages automatisch",
            },
        ],
    },
];

#[component]
pub fn ConceptContextModel() -> Element {
    let mut selected = use_signal(|| 0usize);

    let idx = *selected.read();
    let laag = &LAGEN[idx];

    rsx! {
        div { class: "concept",
            Header {}
            main { class: "container",
                div { class: "concept-intro",
                    h2 { "Het Context Model" }
                    p {
                        "Het meerjarenplan stelt: niet technologie is leidend, maar "
                        strong { "de manier waarop mensen met informatie omgaan" }
                        ". Contextlagen structureren informatiehuishouding \u{2014} de basis voor \"op orde\" (Woo-verplichting 3)."
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                    Panel { title: "Contextlagen".to_string(),
                        select {
                            style: "width: 100%; padding: 10px; margin-bottom: 15px;",
                            onchange: move |evt: Event<FormData>| {
                                if let Ok(i) = evt.value().parse::<usize>() {
                                    selected.set(i);
                                }
                            },
                            for (i, l) in LAGEN.iter().enumerate() {
                                option {
                                    value: "{i}",
                                    selected: i == idx,
                                    "{l.naam}"
                                }
                            }
                        }

                        div { style: "padding: 15px; background: #f5f7fa; border-radius: 8px; margin-bottom: 15px;",
                            h4 { style: "margin-bottom: 8px;", "{laag.naam}" }
                            p { style: "font-size: 0.875rem; color: #666; margin-bottom: 12px;",
                                "{laag.beschrijving}"
                            }
                            div { style: "display: flex; flex-wrap: wrap; gap: 6px;",
                                for voorbeeld in laag.voorbeelden {
                                    span { class: "tag", "{voorbeeld}" }
                                }
                            }
                        }

                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F3AF}" }
                            div { class: "label", "Relevantie-impact" }
                            div { class: "value", "{laag.impact}%" }
                        }

                        if !laag.mjp_koppelingen.is_empty() {
                            h4 { style: "margin: 15px 0 8px; font-size: 0.85rem; color: #5B3CC4;",
                                "Meerjarenplan-koppelingen"
                            }
                            for koppeling in laag.mjp_koppelingen {
                                div { style: "padding: 8px 12px; background: #f0ebff; border-radius: 6px; margin-bottom: 6px; font-size: 0.8rem;",
                                    div { style: "display: flex; align-items: center; gap: 8px; margin-bottom: 4px;",
                                        span { class: "tag woo", "MJP {koppeling.conclusie_nr}" }
                                        strong { "{koppeling.conclusie_titel}" }
                                    }
                                    span { style: "color: #666;", "{koppeling.relatie}" }
                                }
                            }
                        }
                    }

                    Panel { title: "Contextlagen Overzicht".to_string(),
                        div { class: "chart-container", style: "display: flex; flex-direction: column; justify-content: center; gap: 8px;",
                            for l in LAGEN {
                                div { class: "bar-row",
                                    span { class: "bar-label", "{l.naam}" }
                                    div { class: "bar-track",
                                        div {
                                            class: "bar-fill",
                                            style: "width: {l.impact}%;",
                                        }
                                    }
                                    span { class: "bar-value", "{l.impact}%" }
                                }
                            }
                        }
                    }
                }

                div { style: "height: 20px;" }

                Panel { title: "Meerjarenplan 2026\u{2013}2030: Zonder vs. Met Context".to_string(),
                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                        div { style: "padding: 20px; background: #fff3e0; border-radius: 8px;",
                            h4 { style: "color: #E65100; margin-bottom: 10px;", "Huidige situatie (versnipperd)" }
                            ul { style: "font-size: 0.875rem; color: #666; padding-left: 20px;",
                                li { "Informatiehuishouding versnipperd, niet goed geordend (IPO)" }
                                li { "10.000+ documenten zonder context" }
                                li { "Woo-verzoeken handmatig verwerken" }
                                li { "E-mail en chat niet structureel gearchiveerd" }
                                li { "Geen gezamenlijke indicatoren (VNG)" }
                            }
                        }
                        div { style: "padding: 20px; background: #e8f5e9; border-radius: 8px;",
                            h4 { style: "color: #2E7D32; margin-bottom: 10px;", "Met IOU-context (MJP-ambitie)" }
                            ul { style: "font-size: 0.875rem; color: #666; padding-left: 20px;",
                                li { "Informatiehuishouding op orde (verplichting 3)" }
                                li { "Context toont alleen relevante ~150 documenten" }
                                li { "AI ondersteunt bij Woo-verzoeken" }
                                li { "E-mailarchivering (zomer 2026), chatarchivering ge\u{00ef}ntegreerd" }
                                li { "Gedeelde prestatie-indicatoren over alle bestuurslagen" }
                            }
                        }
                    }
                }

                div { style: "height: 20px;" }

                Panel { title: "Contextlagen \u{2194} Meerjarenplan Conclusies".to_string(),
                    p { style: "font-size: 0.8rem; color: #666; margin-bottom: 15px;",
                        "Elke contextlaag adresseert concrete conclusies uit de Meerjarenplannen 2026\u{2013}2030. De matrix toont welke lagen bijdragen aan welke MJP-conclusies."
                    }
                    // Matrix header
                    div { style: "display: grid; grid-template-columns: 140px repeat(7, 1fr); gap: 2px; font-size: 0.75rem;",
                        // Header row
                        div { style: "padding: 8px 4px; font-weight: 600; color: #333;", "" }
                        div { style: "padding: 8px 4px; text-align: center; font-weight: 600; color: #5B3CC4; writing-mode: vertical-lr; transform: rotate(180deg); min-height: 80px;",
                            "1. Info\u{00ad}huishouding"
                        }
                        div { style: "padding: 8px 4px; text-align: center; font-weight: 600; color: #5B3CC4; writing-mode: vertical-lr; transform: rotate(180deg); min-height: 80px;",
                            "2. Duurzame toegank."
                        }
                        div { style: "padding: 8px 4px; text-align: center; font-weight: 600; color: #5B3CC4; writing-mode: vertical-lr; transform: rotate(180deg); min-height: 80px;",
                            "3. Actieve openb."
                        }
                        div { style: "padding: 8px 4px; text-align: center; font-weight: 600; color: #5B3CC4; writing-mode: vertical-lr; transform: rotate(180deg); min-height: 80px;",
                            "4. E-mail & chat"
                        }
                        div { style: "padding: 8px 4px; text-align: center; font-weight: 600; color: #5B3CC4; writing-mode: vertical-lr; transform: rotate(180deg); min-height: 80px;",
                            "5. Governance"
                        }
                        div { style: "padding: 8px 4px; text-align: center; font-weight: 600; color: #5B3CC4; writing-mode: vertical-lr; transform: rotate(180deg); min-height: 80px;",
                            "6. Archiefwet '27"
                        }
                        div { style: "padding: 8px 4px; text-align: center; font-weight: 600; color: #5B3CC4; writing-mode: vertical-lr; transform: rotate(180deg); min-height: 80px;",
                            "7. AI innovatie"
                        }

                        // Data rows
                        for laag in LAGEN {
                            div { style: "padding: 10px 4px; font-weight: 600; color: #333; display: flex; align-items: center;",
                                "{laag.naam}"
                            }
                            for mjp_nr in 1u8..=7 {
                                {
                                    let is_linked = laag.mjp_koppelingen.iter().any(|k| k.conclusie_nr == mjp_nr);
                                    rsx! {
                                        div {
                                            style: if is_linked {
                                                "padding: 10px 4px; text-align: center; background: #e8f5e9; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 1.1rem;"
                                            } else {
                                                "padding: 10px 4px; text-align: center; background: #f5f5f5; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #ccc;"
                                            },
                                            if is_linked { "\u{2713}" } else { "\u{2022}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { style: "margin-top: 12px; display: flex; gap: 20px; font-size: 0.75rem; color: #888;",
                        div { style: "display: flex; align-items: center; gap: 4px;",
                            span { style: "display: inline-block; width: 14px; height: 14px; background: #e8f5e9; border-radius: 3px; text-align: center; line-height: 14px; font-size: 0.7rem;", "\u{2713}" }
                            " Directe koppeling"
                        }
                        div { style: "display: flex; align-items: center; gap: 4px;",
                            span { style: "display: inline-block; width: 14px; height: 14px; background: #f5f5f5; border-radius: 3px; text-align: center; line-height: 14px; color: #ccc;", "\u{2022}" }
                            " Geen directe koppeling"
                        }
                    }
                }
            }
        }
    }
}
