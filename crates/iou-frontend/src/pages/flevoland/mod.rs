//! Provincie Flevoland dashboard page
//!
//! Dit dashboard toont:
//! - PROVISA compliance (provinciale selectielijsten)
//! - PETRA procesclassificatie
//! - Hotspot monitoring
//! - Woo-documenten specifiek voor Flevoland

use dioxus::prelude::*;
use iou_regels::{
    PetraCategorie, ProvisaSelectielijst, HotspotRegister,
};

use crate::components::{AppCard, Header, Panel};
use crate::state::{AppState, UserInfo};
use crate::Route;

struct FlevolandDoc {
    titel: &'static str,
    samenvatting: &'static str,
    datum: &'static str,
    soort: &'static str,
    bron_id: &'static str,
    url: &'static str,
    categorie: PetraCategorie,
}

const FLEVOLAND_DOCS: &[FlevolandDoc] = &[
    FlevolandDoc {
        titel: "Provinciale Ruimtelijke Verordening Flevoland 2024-2028",
        samenvatting: "De PRV Flevoland 2024-2028 geeft het ruimtelijk beleid weer voor de gehele provincie. Naast kaders voor wonen, werken en recreatie bevat de verordening ook een zonering voor windenergie en zonne-energie.",
        datum: "15 jan 2024",
        soort: "Provinciale Verordening",
        bron_id: "prv-flevoland-2024",
        url: "https://www.flevoland.nl/onderwerpen/ruimte-en-wonen/provinciale-ruimtelijke-verordening",
        categorie: PetraCategorie::RuimtelijkePlanning,
    },
    FlevolandDoc {
        titel: "Vaststellingsbesluit Natuurgemeenschappen Flevoland",
        samenvatting: "Besluit van Gedeputeerde Staten tot vaststelling van de natuurgemeenschappen in Flevoland ten behoeve van de provinciale verordening natuur en landschap.",
        datum: "12 dec 2023",
        soort: "Besluit",
        bron_id: "gs-2023-456",
        url: "https://www.flevoland.nl",
        categorie: PetraCategorie::NatuurLandschap,
    },
    FlevolandDoc {
        titel: "Subsidiebeschikking Duurzaam Flevoland 2024",
        samenvatting: "Beschikking in het kader van de subsidieregeling Duurzaam Flevoland voor ondersteuning van energiebesparende maatregelen bij MKB-bedrijven.",
        datum: "8 feb 2024",
        soort: "Subsidie",
        bron_id: "subsidie-2024-123",
        url: "https://www.flevoland.nl/subsidies",
        categorie: PetraCategorie::EnergieKlimaat,
    },
    FlevolandDoc {
        titel: "Kennisgeving Lelystad Airport herstructurering",
        samenvatting: "Officiële kennisgeving over de herstructurering van Lelystad Airport en de provinciale inzet voor de toekomst van de luchthaven.",
        datum: "20 nov 2023",
        soort: "Brief",
        bron_id: "brief-2023-789",
        url: "https://www.flevoland.nl",
        categorie: PetraCategorie::VerkeerVervoer,
    },
];

#[derive(Clone)]
struct ProvisaStatus {
    categorie: PetraCategorie,
    totaal_documenten: u32,
    permanent: u32,
    tijdelijk: u32,
    vernietigbaar: u32,
    overdracht_pending: u32,
}

impl ProvisaStatus {
    fn compliance_percentage(&self) -> f32 {
        if self.totaal_documenten == 0 {
            return 100.0;
        }
        let goedgekeurd = self.permanent + self.tijdelijk;
        (goedgekeurd as f32 / self.totaal_documenten as f32) * 100.0
    }

    fn compliance_text(&self) -> String {
        format!("{:.0}%", self.compliance_percentage())
    }
}

#[component]
pub fn FlevolandDashboard() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut selected_doc = use_signal(|| None::<usize>);

    use_effect(move || {
        state.write().user = Some(UserInfo::flevoland());
    });

    // PROVISA selectielijst voor Flevoland
    let selectielijst = ProvisaSelectielijst::provinciaal_2020();

    // Hotspot register voor Flevoland
    let hotspot_register = use_hook(|| {
        let mut register = HotspotRegister::new("Flevoland");

        // Flevoland-specifieke hotspots
        use iou_regels::Hotspot as H;

        register.voeg_toe(
            H::new(
                "hs-lelystad-airport",
                "Lelystad Airport Herstructurering",
                "Herstructurering van Lelystad Airport en provinciale besluitvorming",
                chrono::NaiveDate::from_ymd_opt(2023, 11, 1).unwrap(),
            )
            .met_categorieen(vec![PetraCategorie::VerkeerVervoer, PetraCategorie::Economie])
            .met_publicatie(
                chrono::NaiveDate::from_ymd_opt(2023, 11, 15).unwrap(),
                Some("https://www.officielebekendmakingen.nl".to_string()),
            ),
        );

        register.voeg_toe(
            H::new(
                "hs-marker-wadden",
                "Marker Wadden Natuurontwikkeling",
                "Aanleg van natuurlijke eilanden in het Markermeer",
                chrono::NaiveDate::from_ymd_opt(2016, 1, 1).unwrap(),
            )
            .met_categorieen(vec![PetraCategorie::NatuurLandschap, PetraCategorie::Water])
        );

        register
    });

    // Simuleer PROVISA status per categorie
    let provisa_statussen: Vec<ProvisaStatus> = use_hook(|| {
        use PetraCategorie as PC;
        vec![
            ProvisaStatus {
                categorie: PC::Bestuur,
                totaal_documenten: 245,
                permanent: 180,
                tijdelijk: 60,
                vernietigbaar: 8,
                overdracht_pending: 12,
            },
            ProvisaStatus {
                categorie: PC::RuimtelijkePlanning,
                totaal_documenten: 189,
                permanent: 165,
                tijdelijk: 22,
                vernietigbaar: 2,
                overdracht_pending: 5,
            },
            ProvisaStatus {
                categorie: PC::NatuurLandschap,
                totaal_documenten: 134,
                permanent: 98,
                tijdelijk: 34,
                vernietigbaar: 2,
                overdracht_pending: 3,
            },
            ProvisaStatus {
                categorie: PC::VerkeerVervoer,
                totaal_documenten: 156,
                permanent: 89,
                tijdelijk: 62,
                vernietigbaar: 5,
                overdracht_pending: 8,
            },
        ]
    });

    let gemiddelde_compliance = use_hook(|| {
        if provisa_statussen.is_empty() {
            100.0
        } else {
            let total: f32 = provisa_statussen.iter()
                .map(|s| s.compliance_percentage())
                .sum();
            total / provisa_statussen.len() as f32
        }
    });

    let compliance_text = format!("{:.0}%", gemiddelde_compliance);

    rsx! {
        div { class: "flevoland",
            Header {}

            main { class: "container",
                // Context Bar
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Provincie Flevoland" }
                        span { " \u{203A} " }
                        span { class: "current", "Provinciaal Dashboard" }
                    }

                    div { class: "provincie-badge",
                        img {
                            src: "/assets/flevoland-logo.png",
                            alt: "Provincie Flevoland",
                            style: "height: 40px; vertical-align: middle;"
                        }
                        span { style: "margin-left: 10px; font-weight: 600;", "Provincie Flevoland" }
                    }
                }

                // Dashboard Grid
                div { class: "dashboard-grid",
                    // Left Column - PROVISA Apps & Status
                    div {
                        Panel { title: "Provinciale Apps".to_string(),
                            div { class: "app-grid",
                                AppCard {
                                    name: "PROVISA Beheer".to_string(),
                                    description: "Beheer selectielijsten en bewaartermijnen".to_string(),
                                    badge: "Compliance".to_string(),
                                }
                                AppCard {
                                    name: "Hotspot Register".to_string(),
                                    description: "Beheer maatschappelijke hotspots".to_string(),
                                }
                                AppCard {
                                    name: "PETRA Explorer".to_string(),
                                    description: "Verken provinciale processen".to_string(),
                                }
                                AppCard {
                                    name: "Archiefoverzicht".to_string(),
                                    description: "Monitoring vernietiging & overbrenging".to_string(),
                                    badge: "Alert".to_string(),
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "PROVISA Compliance".to_string(),
                            div { class: "compliance-summary",
                                div { class: "compliance-score",
                                    div { class: "score-value",
                                    "{compliance_text}"
                                    }
                                    div { class: "score-label", "Algemene compliance" }
                                }
                                div { class: "compliance-details",
                                    div { class: "detail-item ok",
                                        span { class: "label", "Permanent te bewaren" }
                                        span { class: "value", "532" }
                                    }
                                    div { class: "detail-item ok",
                                        span { class: "label", "Tijdelijke bewaring" }
                                        span { class: "value", "178" }
                                    }
                                    div { class: "detail-item warning",
                                        span { class: "label", "Vernietigbaar" }
                                        span { class: "value", "17" }
                                    }
                                    div { class: "detail-item alert",
                                        span { class: "label", "Actie vereist" }
                                        span { class: "value", "28" }
                                    }
                                }
                            }
                        }
                    }

                    // Center Column - Documents & Timeline
                    div {
                        Panel { title: "Provinciale Documenten".to_string(),
                            ul { class: "document-list",
                                for (i, doc) in FLEVOLAND_DOCS.iter().enumerate() {
                                    li {
                                        class: "document-item",
                                        style: if *selected_doc.read() == Some(i) { "background: #e8f4fd; cursor: pointer;" } else { "cursor: pointer;" },
                                        onclick: move |_| {
                                            if *selected_doc.read() == Some(i) {
                                                selected_doc.set(None);
                                            } else {
                                                selected_doc.set(Some(i));
                                            }
                                        },
                                        div { class: "document-icon", style: "background: #0066CC;", "\u{1F4C4}" }
                                        div { class: "document-info",
                                            h4 { "{doc.titel}" }
                                            div { class: "meta",
                                                "{doc.soort} \u{2022} {doc.datum} \u{2022} {doc.bron_id}"
                                            }
                                            div { class: "provisa-tag",
                                                "PROVISA: {doc.categorie}"
                                            }
                                        }
                                    }
                                    if *selected_doc.read() == Some(i) {
                                        li { style: "padding: 15px; background: #fff8f0; border-left: 3px solid #0066CC;",
                                            p { style: "font-size: 0.875rem; color: #444; line-height: 1.6; margin-bottom: 12px;",
                                                "{doc.samenvatting}"
                                            }
                                            div { style: "display: flex; gap: 10px; align-items: center;",
                                                a {
                                                    href: "{doc.url}",
                                                    target: "_blank",
                                                    class: "btn btn-primary",
                                                    style: "text-decoration: none; font-size: 0.8rem;",
                                                    "Bekijk op flevoland.nl \u{2197}"
                                                }
                                                // PROVISA beoordeling tonen
                                                div { class: "provisa-mini-status",
                                                    span { style: "font-size: 0.75rem; color: #666;", "Bewaartermijn: " }
                                                    span { style: "font-size: 0.75rem; font-weight: 600; color: #0066CC;",
                                                        match doc.categorie {
                                                            PetraCategorie::RuimtelijkePlanning => "Permanent (PRV)",
                                                            PetraCategorie::Bestuur => "Permanent",
                                                            _ => "Tijdelijk (10 jaar)",
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Actieve Hotspots".to_string(),
                            if hotspot_register.hotspots.is_empty() {
                                p { style: "color: #888; font-style: italic;", "Geen actieve hotspots" }
                            } else {
                                ul { class: "hotspot-list",
                                    for hotspot in &hotspot_register.hotspots {
                                        li { class: "hotspot-item",
                                            div { class: "hotspot-icon", "\u{1F525}" }
                                            div { class: "hotspot-info",
                                                h4 { "{hotspot.naam}" }
                                                p { style: "font-size: 0.85rem; color: #666;",
                                                    "{hotspot.beschrijving}"
                                                }
                                                div { style: "margin-top: 8px;",
                                                    for cat in &hotspot.categorieen {
                                                        span { class: "tag provincie",
                                                            "{cat}"
                                                        }
                                                    }
                                                }
                                            }
                                            div { class: "hotspot-badge", "Upgrade \u{2191}" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Right Column - PROVISA per Categorie & Actions
                    div {
                        Panel { title: "PROVISA per Categorie".to_string(),
                            table { class: "provisa-table",
                                thead {
                                    tr {
                                        th { "Categorie" }
                                        th { "Totaal" }
                                        th { "Compliance" }
                                    }
                                }
                                tbody {
                                    for status in &provisa_statussen {
                                        tr {
                                            td { "{status.categorie}" }
                                            td { "{status.totaal_documenten}" }
                                            td {
                                                div { class: "compliance-bar",
                                                    div {
                                                        class: if status.compliance_percentage() >= 95.0 {
                                                            "bar-fill good"
                                                        } else if status.compliance_percentage() >= 80.0 {
                                                            "bar-fill warning"
                                                        } else {
                                                            "bar-fill bad"
                                                        },
                                                        style: "width: {status.compliance_percentage()}%;"
                                                    }
                                                    span { style: "margin-left: 8px; font-size: 0.8rem;",
                                                        "{status.compliance_text()}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Actie Vereist".to_string(),
                            div { class: "action-list",
                                div { class: "action-item urgent",
                                    div { class: "action-icon", "\u{26A0}" }
                                    div { class: "action-content",
                                        h5 { "17 documenten vernietigbaar" }
                                        p { "Documenten zijn te vernietigen volgens PROVISA selectielijst" }
                                        button { class: "btn btn-small", "Start vernietiging" }
                                    }
                                }
                                div { class: "action-item warning",
                                    div { class: "action-icon", "\u{1F4C1}" }
                                    div { class: "action-content",
                                        h5 { "12 documenten naar archief" }
                                        p { "Permanent te bewaren documenten wachten op overbrenging" }
                                        button { class: "btn btn-small", "Plan overbrenging" }
                                    }
                                }
                                div { class: "action-item info",
                                    div { class: "action-icon", "\u{2139}" }
                                    div { class: "action-content",
                                        h5 { "PROVISA 2020 update" }
                                        p { "Nieuwe concordans beschikbaar voor categorie Ruimtelijke Planning" }
                                        button { class: "btn btn-small btn-outline", "Bekijk wijzigingen" }
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

// Voeg UserInfo extensie toe voor Flevoland
pub trait FlevolandUserInfo {
    fn flevoland() -> Self;
}

impl FlevolandUserInfo for UserInfo {
    fn flevoland() -> Self {
        use uuid::Uuid;
        Self {
            id: Uuid::new_v4(),
            name: "Archiefmedewerker Flevoland".to_string(),
            email: "archief@flevoland.nl".to_string(),
            initials: "AF".to_string(),
            organization: "Provincie Flevoland".to_string(),
            role: "Archiefmedewerker".to_string(),
        }
    }
}

/// PROVISA beheer pagina
#[component]
pub fn FlevolandProvisa() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    use_effect(move || {
        state.write().user = Some(UserInfo::flevoland());
    });

    rsx! {
        div { class: "flevoland",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Provincie Flevoland" }
                        span { " \u{203A} " }
                        span { class: "current", "PROVISA Beheer" }
                    }
                }
                Panel { title: "PROVISA Selectielijsten".to_string(),
                    p { "Beheer van provinciale selectielijsten en bewaartermijnen." }
                }
            }
        }
    }
}

/// Hotspots pagina
#[component]
pub fn FlevolandHotspots() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    use_effect(move || {
        state.write().user = Some(UserInfo::flevoland());
    });

    rsx! {
        div { class: "flevoland",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Provincie Flevoland" }
                        span { " \u{203A} " }
                        span { class: "current", "Hotspot Register" }
                    }
                }
                Panel { title: "Hotspot Register".to_string(),
                    p { "Beheer van maatschappelijke hotspots die archiefwaarde beïnvloeden." }
                }
            }
        }
    }
}

/// Archief pagina
#[component]
pub fn FlevolandArchief() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    use_effect(move || {
        state.write().user = Some(UserInfo::flevoland());
    });

    rsx! {
        div { class: "flevoland",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Provincie Flevoland" }
                        span { " \u{203A} " }
                        span { class: "current", "Archiefoverzicht" }
                    }
                }
                Panel { title: "Archiefoverzicht".to_string(),
                    p { "Monitoring van vernietiging en overbrenging naar het Nationaal Archief." }
                }
            }
        }
    }
}
