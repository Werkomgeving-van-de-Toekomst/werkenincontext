//! Meerjarenplan Conclusies — 7 beleidsconclusies en hoe IOU-Modern deze adresseert
//!
//! Gebaseerd op de Meerjarenplannen Digitale Informatiehuishouding
//! en Openbaarheid 2026-2030 (Rijksoverheid, VNG, IPO, UvW).

use dioxus::prelude::*;

use crate::components::{Header, Panel};

struct Conclusie {
    nummer: usize,
    titel: &'static str,
    icoon: &'static str,
    bron: &'static str,
    conclusie: &'static str,
    iou_antwoord: &'static str,
    status: &'static str,
    status_pct: u8,
}

const CONCLUSIES: &[Conclusie] = &[
    Conclusie {
        nummer: 1,
        titel: "Informatiehuishouding op orde",
        icoon: "\u{1F4C1}",
        bron: "Woo basisverplichting \u{2022} Rijks-MJP \u{2022} IPO",
        conclusie: "De informatiehuishouding moet op orde zijn als basisvoorwaarde voor openbaarheid. Alle bestuurslagen erkennen dat dit de fundatie is: zonder geordende informatie geen effectieve openbaarmaking.",
        iou_antwoord: "IOU-Modern structureert informatie in informatiedomeinen \u{2014} contextgebonden eenheden waarin documenten, e-mails en data samen betekenis krijgen. Niet de technologie is leidend, maar de manier waarop mensen met informatie werken (IPO).",
        status: "Operationeel",
        status_pct: 90,
    },
    Conclusie {
        nummer: 2,
        titel: "Duurzame toegankelijkheid",
        icoon: "\u{1F4DA}",
        bron: "Archiefwet 2027 \u{2022} Rijks-MJP \u{2022} VNG",
        conclusie: "Informatie moet duurzaam toegankelijk blijven: vindbaar, bruikbaar en authentiek over de tijd. De nieuwe Archiefwet (2027) verkort de overbrengingstermijn en eist archivering by design.",
        iou_antwoord: "Automatische bewaartermijnen en classificatie zijn ingebouwd in het contextmodel. Bij opname van een document stelt AI de juiste archiefcategorie en bewaartermijn voor. Wanneer de Archiefwet 2027 in werking treedt, zijn informatiedomeinen al compliant.",
        status: "Actief",
        status_pct: 75,
    },
    Conclusie {
        nummer: 3,
        titel: "Actieve openbaarmaking",
        icoon: "\u{1F4E2}",
        bron: "GWV \u{2022} Zoek & Vind 2.0 \u{2022} VNG-pilaar 1",
        conclusie: "Transparantie wordt de standaard. Overheden moeten informatie proactief openbaar maken via de Generieke Woo-voorziening (GWV) en Zoek & Vind 2.0, niet alleen wachten op verzoeken.",
        iou_antwoord: "Een publicatieworkflow koppelt informatiedomeinen aan GWV: AI identificeert publicatiewaardige informatie, de medewerker accordeert, en het systeem publiceert automatisch naar open.overheid.nl. Context zorgt dat gerelateerde stukken mee worden ontsloten.",
        status: "In ontwikkeling",
        status_pct: 60,
    },
    Conclusie {
        nummer: 4,
        titel: "E-mail en chatarchivering",
        icoon: "\u{1F4E7}",
        bron: "Rijksvoorzieningen 2026 \u{2022} Rijks-MJP",
        conclusie: "E-mail- en chatarchivering worden rijksbreed beschikbaar (zomer 2026). Alle bestuurslagen moeten deze berichten duurzaam bewaren als onderdeel van de informatiehuishouding.",
        iou_antwoord: "Ingest-koppelingen voor e-mail (zomer \u{2019}26) en chat (2026) plaatsen berichten automatisch in het juiste informatiedomein. AI herkent context, deelnemers en onderwerp en koppelt berichten aan lopende dossiers.",
        status: "Gepland",
        status_pct: 35,
    },
    Conclusie {
        nummer: 5,
        titel: "Governance en prestatie-indicatoren",
        icoon: "\u{1F4CA}",
        bron: "VNG KPI\u{2019}s \u{2022} Rijks-MJP \u{2022} IPO",
        conclusie: "Structurele governance met meetbare prestatie-indicatoren is nodig om voortgang te borgen. VNG definieert KPI\u{2019}s voor Woo-doorlooptijden, publicatievolume en informatiehuishouding.",
        iou_antwoord: "Compliance dashboards monitoren KPI\u{2019}s in real-time: Woo-doorlooptijden, percentage actief gepubliceerd, bewaartermijn-naleving en archiveringsgraad. Elke organisatie ziet haar eigen scores en kan benchmarken.",
        status: "In ontwikkeling",
        status_pct: 50,
    },
    Conclusie {
        nummer: 6,
        titel: "Nieuwe Archiefwet 2027",
        icoon: "\u{1F4C5}",
        bron: "Archiefwet 2027 \u{2022} Rijks-MJP \u{2022} IPO \u{2022} VNG",
        conclusie: "De nieuwe Archiefwet (inwerkingtreding 1 januari 2027) vereist archivering by design: classificatie en metadata moeten vanaf het moment van creatie meelopen, niet achteraf worden toegevoegd.",
        iou_antwoord: "Het contextmodel is archivering by design: bij creatie of import wordt elk informatieobject automatisch geclassificeerd (AI-suggestie + menselijke review) en voorzien van metadata conform MDTO. De overbrengingstermijn van 10 jaar is ingebouwd.",
        status: "Gereed voor 2027",
        status_pct: 80,
    },
    Conclusie {
        nummer: 7,
        titel: "AI innovatie-agenda",
        icoon: "\u{1F916}",
        bron: "BZK \u{2022} VNG \u{2022} IPO \u{2022} UvW",
        conclusie: "Rijksoverheid, gemeenten, provincies en waterschappen werken samen aan AI-oplossingen voor informatiehuishouding, Woo-afhandeling en datakwaliteit. De mens beslist altijd.",
        iou_antwoord: "De AI-pipeline in IOU-Modern biedt metadata-suggesties, automatische classificatie, relatie-detectie (GraphRAG) en Woo-beoordeling. Alle suggesties worden ter review aangeboden \u{2014} de mens beslist. Het systeem leert van correcties.",
        status: "Operationeel",
        status_pct: 85,
    },
];

#[component]
pub fn ConceptMeerjarenplanConclusies() -> Element {
    let mut active_thema = use_signal(|| 0usize);
    let idx = *active_thema.read();
    let c = &CONCLUSIES[idx];

    rsx! {
        div { class: "concept",
            Header {}
            main { class: "container",
                div { class: "concept-intro",
                    h2 { "Meerjarenplan Conclusies 2026\u{2013}2030" }
                    p {
                        "Zeven beleidsconclusies uit de "
                        strong { "Meerjarenplannen Digitale Informatiehuishouding en Openbaarheid" }
                        " en hoe IOU-Modern deze concreet adresseert."
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 2fr; gap: 20px;",
                    // Left: thema selector
                    Panel { title: "Beleidsconclusies".to_string(),
                        ul { class: "document-list",
                            for (i, conclusie) in CONCLUSIES.iter().enumerate() {
                                li {
                                    class: "document-item",
                                    style: if i == idx { "background: #f0ebff; cursor: pointer;" } else { "cursor: pointer;" },
                                    onclick: move |_| active_thema.set(i),
                                    div { class: "document-icon concept-step", "{conclusie.nummer}" }
                                    div { class: "document-info",
                                        h4 { "{conclusie.titel}" }
                                        div { class: "meta", "{conclusie.bron}" }
                                    }
                                }
                            }
                        }
                    }

                    // Right: detail panel
                    div {
                        Panel { title: format!("{}  {}", c.icoon, c.titel),
                            div { style: "padding: 10px 0;",
                                // Conclusie
                                h4 { style: "margin-bottom: 8px; color: #5B3CC4;", "Conclusie uit Meerjarenplannen" }
                                p { style: "margin-bottom: 20px; color: #444; line-height: 1.6;",
                                    "{c.conclusie}"
                                }

                                // IOU-antwoord
                                h4 { style: "margin-bottom: 8px; color: #2E7D32;", "IOU-Modern Antwoord" }
                                p { style: "margin-bottom: 20px; color: #444; line-height: 1.6;",
                                    "{c.iou_antwoord}"
                                }

                                // Status
                                h4 { style: "margin-bottom: 8px;", "Status" }
                                div { style: "display: flex; align-items: center; gap: 12px; margin-bottom: 8px;",
                                    span { class: "tag woo", "{c.status}" }
                                    span { style: "font-size: 0.875rem; color: #666;", "{c.status_pct}% gerealiseerd" }
                                }
                                // Progress bar
                                div { style: "background: #e0e0e0; border-radius: 6px; height: 10px; width: 100%;",
                                    div {
                                        style: format!("background: linear-gradient(90deg, #5B3CC4, #7E57C2); border-radius: 6px; height: 10px; width: {}%;", c.status_pct),
                                    }
                                }

                                // Bron
                                div { style: "margin-top: 15px;",
                                    span { style: "font-size: 0.8rem; color: #888;", "Bron: " }
                                    span { style: "font-size: 0.8rem; color: #666;", "{c.bron}" }
                                }
                            }
                        }
                    }
                }

                // Totaaloverzicht met bar charts
                div { style: "margin-top: 30px;",
                    Panel { title: "Totaaloverzicht realisatie".to_string(),
                        div { style: "display: grid; grid-template-columns: repeat(7, 1fr); gap: 12px; align-items: end; padding: 20px 0 10px;",
                            for conclusie in CONCLUSIES.iter() {
                                div { style: "display: flex; flex-direction: column; align-items: center; gap: 6px;",
                                    span { style: "font-size: 0.75rem; font-weight: 600; color: #333;", "{conclusie.status_pct}%" }
                                    // Bar
                                    div { style: "width: 100%; max-width: 40px; background: #e0e0e0; border-radius: 4px; height: 120px; position: relative;",
                                        div {
                                            style: format!(
                                                "position: absolute; bottom: 0; width: 100%; background: linear-gradient(180deg, #5B3CC4, #7E57C2); border-radius: 4px; height: {}%;",
                                                conclusie.status_pct
                                            ),
                                        }
                                    }
                                    span { style: "font-size: 0.65rem; color: #666; text-align: center; max-width: 80px;", "{conclusie.titel}" }
                                }
                            }
                        }
                        div { style: "text-align: center; margin-top: 10px; font-size: 0.8rem; color: #888;",
                            {
                                let avg: u8 = (CONCLUSIES.iter().map(|c| c.status_pct as u16).sum::<u16>() / CONCLUSIES.len() as u16) as u8;
                                format!("Gemiddelde realisatiegraad: {}%", avg)
                            }
                        }
                    }
                }
            }
        }
    }
}
