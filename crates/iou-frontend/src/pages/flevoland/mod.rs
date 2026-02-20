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

use crate::components::{AppCard, Header, Panel, TimelineEvent, TimelineEventType, Timeline};
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
        titel: "Kennisgeving Projectbesluit en MER Rondweg Lelystad-Zuid",
        samenvatting: "Kennisgeving van het projectbesluit en milieueffectrapport voor de Rondweg Lelystad-Zuid (Laan van Nieuw Land - Verlengde Westerdreef). Betreft de aanleg van een nieuwe provinciale weg ter verbetering van de bereikbaarheid.",
        datum: "30 jan 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1767",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1767.html",
        categorie: PetraCategorie::VerkeerVervoer,
    },
    FlevolandDoc {
        titel: "Besluit omgevingsvergunning Natura 2000 zandwinning IJsselmeer",
        samenvatting: "Besluit (positieve) weigering omgevingsvergunning voor een Natura 2000-activiteit zandwinning vaargeul Amsterdam-Lemmer (VAL5) in het IJsselmeer. De vergunning is geweigerd vanwege mogelijke impact op beschermde natuur.",
        datum: "28 jan 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1405",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1405.html",
        categorie: PetraCategorie::NatuurLandschap,
    },
    FlevolandDoc {
        titel: "Ontheffing helikopterlanding provincie Flevoland 2026",
        samenvatting: "Wet Luchtvaart generieke ontheffing Tijdelijk en Uitzonderlijk Gebruik kalenderjaar 2026 in de provincie Flevoland voor het landen en stijgen met een helikopter.",
        datum: "29 jan 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1457",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1457.html",
        categorie: PetraCategorie::VerkeerVervoer,
    },
    FlevolandDoc {
        titel: "Ondermandaat Bedrijfsvoering ODFL",
        samenvatting: "Gewijzigd ondermandaat voor de bedrijfsvoering van de Omgevingsdienst Flevoland & Gooi en Vechtstreek. Regelt de bevoegdheidsverdeling voor operationele beslissingen.",
        datum: "4 feb 2026",
        soort: "Blad gemeenschappelijke regeling",
        bron_id: "bgr-2026-301",
        url: "https://zoek.officielebekendmakingen.nl/bgr-2026-301.html",
        categorie: PetraCategorie::Bestuur,
    },
    FlevolandDoc {
        titel: "Last onder bestuursdwang vaartuigen Hoge Vaart",
        samenvatting: "Handhavingsbesluit last onder bestuursdwang voor vaartuigen in de berm langs de Hoge Vaart. Eigenaren worden gesommeerd de vaartuigen te verwijderen.",
        datum: "5 feb 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1953",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1953.html",
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
                    // Left Column - Context Apps
                    div {
                        Panel { title: "Context Apps".to_string(),
                            div { class: "app-grid",
                                Link { to: Route::DataVerkenner,
                                    AppCard {
                                        name: "Data Verkenner".to_string(),
                                        description: "Verken provinciale datasets".to_string(),
                                        badge: "Populair".to_string(),
                                    }
                                }
                                Link { to: Route::DocumentGenerator,
                                    AppCard {
                                        name: "Document Generator".to_string(),
                                        description: "Genereer compliant documenten".to_string(),
                                        badge: "Nieuw".to_string(),
                                    }
                                }
                                Link { to: Route::Nalevingscontrole,
                                    AppCard {
                                        name: "Nalevingscontrole".to_string(),
                                        description: "Monitor Woo/AVG compliance".to_string(),
                                    }
                                }
                                Link { to: Route::GraphRAGExplorer,
                                    AppCard {
                                        name: "GraphRAG Explorer".to_string(),
                                        description: "Ontdek relaties via kennisgraaf".to_string(),
                                        badge: "AI".to_string(),
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Provinciale Apps".to_string(),
                            div { class: "app-grid",
                                Link { to: Route::FlevolandProvisa,
                                    AppCard {
                                        name: "PROVISA Beheer".to_string(),
                                        description: "Beheer selectielijsten en bewaartermijnen".to_string(),
                                        badge: "Compliance".to_string(),
                                    }
                                }
                                Link { to: Route::FlevolandHotspots,
                                    AppCard {
                                        name: "Hotspot Register".to_string(),
                                        description: "Beheer maatschappelijke hotspots".to_string(),
                                    }
                                }
                                Link { to: Route::FlevolandArchief,
                                    AppCard {
                                        name: "Archiefoverzicht".to_string(),
                                        description: "Monitoring vernietiging & overbrenging".to_string(),
                                        badge: "Alert".to_string(),
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Compliance Status".to_string(),
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "PROVISA Compliance" }
                                div { class: "value", "{compliance_text}" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "Archiefwet 1995" }
                                div { class: "value", "100%" }
                            }
                            div { class: "compliance-indicator warning",
                                div { class: "icon", "!" }
                                div { class: "label", "Vernietiging acties" }
                                div { class: "value", "17" }
                            }
                        }
                    }

                    // Center Column - Timeline & Documents
                    div {
                        Panel { title: "Tijdlijn: Provinciaal Beleid".to_string(),
                            Timeline {
                                title: String::new(),
                                events: vec![
                                    TimelineEvent {
                                        id: "1".to_string(),
                                        title: "Subsidiebeschikking Duurzaam Flevoland 2024".to_string(),
                                        date: "2024-02-08".to_string(),
                                        date_display: "8 feb 2024".to_string(),
                                        description: "Beschikking in het kader van de subsidieregeling Duurzaam Flevoland voor ondersteuning van energiebesparende maatregelen bij MKB-bedrijven.".to_string(),
                                        event_type: TimelineEventType::Besluit,
                                        url: Some("https://www.flevoland.nl".to_string()),
                                    },
                                    TimelineEvent {
                                        id: "2".to_string(),
                                        title: "Provinciale Ruimtelijke Verordening 2024-2028".to_string(),
                                        date: "2024-01-15".to_string(),
                                        date_display: "15 jan 2024".to_string(),
                                        description: "De PRV Flevoland 2024-2028 geeft het ruimtelijk beleid weer voor de gehele provincie.".to_string(),
                                        event_type: TimelineEventType::ProjectMilestone,
                                        url: Some("https://www.flevoland.nl".to_string()),
                                    },
                                    TimelineEvent {
                                        id: "3".to_string(),
                                        title: "Kennisgeving Lelystad Airport herstructurering".to_string(),
                                        date: "2023-11-20".to_string(),
                                        date_display: "20 nov 2023".to_string(),
                                        description: "Officiële kennisgeving over de herstructurering van Lelystad Airport en de provinciale inzet voor de toekomst.".to_string(),
                                        event_type: TimelineEventType::Document,
                                        url: Some("https://www.flevoland.nl".to_string()),
                                    },
                                ],
                                max_items: 5,
                                context_label: Some("Flevoland".to_string()),
                            }
                        }

                        div { style: "height: 20px;" }

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
                                                    "Bekijk op open.overheid.nl \u{2197}"
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

                    // Right Column - Stakeholders & AI
                    div {
                        Panel { title: "Stakeholders".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #0066CC;", "\u{1F3E2}" }
                                    div { class: "document-info",
                                        h4 { "Gemeente Almere" }
                                        div { class: "meta", "Samenwerkingspartner" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #0066CC;", "\u{1F3E2}" }
                                    div { class: "document-info",
                                        h4 { "Gemeente Lelystad" }
                                        div { class: "meta", "Samenwerkingspartner" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #0066CC;", "\u{1F3DE}" }
                                    div { class: "document-info",
                                        h4 { "Omgevingsdienst Flevoland" }
                                        div { class: "meta", "Adviseur" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", style: "background: #0066CC;", "\u{1F393}" }
                                    div { class: "document-info",
                                        h4 { "HZ University of Applied Sciences" }
                                        div { class: "meta", "Kennispartner" }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Gerelateerde Domeinen".to_string(),
                            div { style: "display: flex; flex-wrap: wrap; gap: 10px;",
                                div { class: "tag", "Ruimtelijke Planning" }
                                div { class: "tag", "Energie & Klimaat" }
                                div { class: "tag", "Natuur & Landschap" }
                                div { class: "tag", "Verkeer & Vervoer" }
                                div { class: "tag", "Economie" }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "AI Suggesties".to_string(),
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F916}" }
                                div { class: "label", "3 nieuwe metadata suggesties" }
                            }
                            p { style: "font-size: 0.875rem; color: #666; margin-top: 10px;",
                                "AI heeft automatisch PROVISA-classificaties voorgesteld voor 3 nieuwe provinciale documenten."
                            }
                            button { class: "btn btn-secondary", style: "margin-top: 10px; width: 100%;",
                                "Bekijk suggesties"
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
                        Link { to: Route::FlevolandDashboard, span { "Provincie Flevoland" } }
                        span { " \u{203A} " }
                        span { class: "current", "PROVISA Beheer" }
                    }
                }
                div { class: "dashboard-grid",
                    div {
                        Panel { title: "PROVISA Selectielijst 2020".to_string(),
                            p { style: "margin-bottom: 15px;",
                                "De Provinciale Selectielijst Archieven (PROVISA) 2020 is de leidraad voor het beheren van overheidsarchieven van de provincie Flevoland."
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4C1}" }
                                div { class: "label", "Versie" }
                                div { class: "value", "PROVISA 2020" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4C5}" }
                                div { class: "label", "Publicatiedatum" }
                                div { class: "value", "1 jan 2020" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4CA}" }
                                div { class: "label", "Aantal categorieën" }
                                div { class: "value", "26" }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "PETRA Categorieën".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CB}" }
                                    div { class: "document-info",
                                        h4 { "Bestuur" }
                                        div { class: "meta", "Besluiten, vergaderingen, raadsvragen" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CB}" }
                                    div { class: "document-info",
                                        h4 { "Ruimtelijke Planning" }
                                        div { class: "meta", "PRV, bestemmingsplannen, vergunningen" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CB}" }
                                    div { class: "document-info",
                                        h4 { "Natuur & Landschap" }
                                        div { class: "meta", "Beheerplannen, vergunningen" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CB}" }
                                    div { class: "document-info",
                                        h4 { "Verkeer & Vervoer" }
                                        div { class: "meta", "Infrastructuur, openbaar vervoer" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CB}" }
                                    div { class: "document-info",
                                        h4 { "Energie & Klimaat" }
                                        div { class: "meta", "Duurzaamheid, energietransitie" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CB}" }
                                    div { class: "document-info",
                                        h4 { "Economie" }
                                        div { class: "meta", "Subsidies, ondernemen, werkgelegenheid" }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        Panel { title: "Bewaartermijnen per Categorie".to_string(),
                            table { class: "provisa-table",
                                thead {
                                    tr {
                                        th { "Categorie" }
                                        th { "Permanent" }
                                        th { "Tijdelijk" }
                                        th { "Bewaartermijn" }
                                    }
                                }
                                tbody {
                                    tr {
                                        td { "Bestuur" }
                                        td { class: "status-ok", "\u{2713}" }
                                        td { "" }
                                        td { "Permanent" }
                                    }
                                    tr {
                                        td { "Ruimtelijke Planning" }
                                        td { class: "status-ok", "\u{2713}" }
                                        td { "" }
                                        td { "Permanent" }
                                    }
                                    tr {
                                        td { "Natuur & Landschap" }
                                        td { class: "status-ok", "\u{2713}" }
                                        td { "" }
                                        td { "Permanent" }
                                    }
                                    tr {
                                        td { "Verkeer & Vervoer" }
                                        td { "" }
                                        td { class: "status-partial", "\u{2713}" }
                                        td { "10 jaar" }
                                    }
                                    tr {
                                        td { "Energie & Klimaat" }
                                        td { "" }
                                        td { class: "status-partial", "\u{2713}" }
                                        td { "10 jaar" }
                                    }
                                    tr {
                                        td { "Economie" }
                                        td { "" }
                                        td { class: "status-partial", "\u{2713}" }
                                        td { "5 jaar" }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Acties Vereist".to_string(),
                            div { class: "compliance-indicator warning",
                                div { class: "icon", "!" }
                                div { class: "label", "17 documenten vernietigbaar" }
                                div { class: "value", "Nu" }
                            }
                            div { class: "compliance-indicator warning",
                                div { class: "icon", "!" }
                                div { class: "label", "12 documenten naar archief" }
                                div { class: "value", "Binnen 3 maanden" }
                            }
                        }
                    }
                    div {
                        Panel { title: "Concordans".to_string(),
                            p { style: "font-size: 0.875rem; color: #666;",
                                "De concordans koppelt provinciale procescategorieën (PETRA) aan de juiste bewaartermijnen uit de selectielijst."
                            }
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4DD}" }
                                    div { class: "document-info",
                                        h4 { "Collegebesluiten" }
                                        div { class: "meta", "PETRA: Bestuur \u{2192} Permanent" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4DD}" }
                                    div { class: "document-info",
                                        h4 { "Vergunningaanvragen" }
                                        div { class: "meta", "PETRA: Ruimtelijke Planning \u{2192} Permanent" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4DD}" }
                                    div { class: "document-info",
                                        h4 { "Subsidiebeschikkingen" }
                                        div { class: "meta", "PETRA: Economie \u{2192} 5 jaar" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4DD}" }
                                    div { class: "document-info",
                                        h4 { "Beleidsnotities" }
                                        div { class: "meta", "PETRA: Algemeen \u{2192} 10 jaar" }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Links".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F517}" }
                                    div { class: "document-info",
                                        a {
                                            href: "https://www.bij12.nl",
                                            target: "_blank",
                                            h4 { "BIJ12 - PROVISA" }
                                        }
                                        div { class: "meta", "Officiële PROVISA informatie" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F517}" }
                                    div { class: "document-info",
                                        a {
                                            href: "https://www.nationaalarchief.nl",
                                            target: "_blank",
                                            h4 { "Nationaal Archief" }
                                        }
                                        div { class: "meta", "Archiefwet 1995" }
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

/// Hotspots pagina
#[component]
pub fn FlevolandHotspots() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    use_effect(move || {
        state.write().user = Some(UserInfo::flevoland());
    });

    let hotspots = use_hook(|| {
        let mut register = HotspotRegister::new("Flevoland");
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
        register.voeg_toe(
            H::new(
                "hs-almere-stad",
                "Almere Stad Uitbreiding",
                "Grootschalige stadsuitbreiding en woningbouw",
                chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            )
            .met_categorieen(vec![PetraCategorie::RuimtelijkePlanning, PetraCategorie::Wonen])
        );
        register
    });

    rsx! {
        div { class: "flevoland",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        Link { to: Route::FlevolandDashboard, span { "Provincie Flevoland" } }
                        span { " \u{203A} " }
                        span { class: "current", "Hotspot Register" }
                    }
                }
                div { class: "dashboard-grid",
                    div {
                        Panel { title: "Wat is een Hotspot?".to_string(),
                            p { style: "margin-bottom: 10px;",
                                "Een maatschappelijke hotspot is een gebeurtenis of ontwikkeling die van nationaal of provinciaal belang is en die kan leiden tot upgrade van archiefwaarde van documenten."
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F525}" }
                                div { class: "label", "Aantal hotspots" }
                                div { class: "value", "{hotspots.hotspots.len()}" }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Hotspot Effecten".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{2191}" }
                                    div { class: "document-info",
                                        h4 { "Upgrade naar Permanent" }
                                        div { class: "meta", "Tijdelijke documenten worden permanent bij hotspot relevantie" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F50D}" }
                                    div { class: "document-info",
                                        h4 { "Verhoogde vindbaarheid" }
                                        div { class: "meta", "Hotspot-documenten worden gemarkeerd voor snelle toegang" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CA}" }
                                    div { class: "document-info",
                                        h4 { "Extra metadata" }
                                        div { class: "meta", "Automatisch koppelen aan relevante procescategorieën" }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        Panel { title: "Actieve Hotspots".to_string(),
                            ul { class: "hotspot-list",
                                for hotspot in &hotspots.hotspots {
                                    li { class: "hotspot-item",
                                        div { class: "hotspot-icon", "\u{1F525}" }
                                        div { class: "hotspot-info",
                                            h4 { "{hotspot.naam}" }
                                            p { style: "font-size: 0.85rem; color: #666;",
                                                "{hotspot.beschrijving}"
                                            }
                                            div { style: "margin-top: 8px;",
                                                for cat in &hotspot.categorieen {
                                                    span { class: "tag provincie", "{cat}" }
                                                }
                                            }
                                            div { style: "margin-top: 8px; font-size: 0.8rem; color: #888;",
                                                "Sinds {hotspot.start_datum}"
                                            }
                                        }
                                        div { class: "hotspot-badge", "Upgrade \u{2191}" }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Nieuwe Hotspot Toevoegen".to_string(),
                            p { style: "font-size: 0.875rem; color: #666;",
                                "Voeg een nieuwe maatschappelijke hotspot toe aan het register."
                            }
                            div { style: "display: flex; flex-direction: column; gap: 10px;",
                                input {
                                    r#type: "text",
                                    placeholder: "Naam van de hotspot",
                                }
                                textarea {
                                    placeholder: "Beschrijving van de hotspot",
                                    rows: 3,
                                }
                                input {
                                    r#type: "date",
                                }
                                button { class: "btn btn-primary", "Hotspot toevoegen" }
                            }
                        }
                    }
                    div {
                        Panel { title: "Documenten bij Hotspots".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4C4}" }
                                    div { class: "document-info",
                                        h4 { "Kennisgeving Lelystad Airport" }
                                        div { class: "meta", "Gelinkt aan: Lelystad Airport Herstructurering" }
                                        span { class: "tag hotspot", "Hotspot" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4C4}" }
                                    div { class: "document-info",
                                        h4 { "Marker Wadden Projectplan" }
                                        div { class: "meta", "Gelinkt aan: Marker Wadden Natuurontwikkeling" }
                                        span { class: "tag hotspot", "Hotspot" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4C4}" }
                                    div { class: "document-info",
                                        h4 { "Structuurvisie Almere 2.0" }
                                        div { class: "meta", "Gelinkt aan: Almere Stad Uitbreiding" }
                                        span { class: "tag hotspot", "Hotspot" }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Hotspot Kalender".to_string(),
                            p { style: "font-size: 0.875rem; color: #666;",
                                "Timeline van belangrijke hotspot-momenten."
                            }
                            Timeline {
                                title: String::new(),
                                events: vec![
                                    TimelineEvent {
                                        id: "1".to_string(),
                                        title: "Start Lelystad Airport Herstructurering".to_string(),
                                        date: "2023-11-01".to_string(),
                                        date_display: "1 nov 2023".to_string(),
                                        description: "Besluitvorming over toekomst Lelystad Airport.".to_string(),
                                        event_type: TimelineEventType::ProjectMilestone,
                                        url: None,
                                    },
                                    TimelineEvent {
                                        id: "2".to_string(),
                                        title: "Marker Wadden fase 2".to_string(),
                                        date: "2024-04-01".to_string(),
                                        date_display: "1 apr 2024".to_string(),
                                        description: "Start tweede fase eilandaanleg.".to_string(),
                                        event_type: TimelineEventType::ProjectMilestone,
                                        url: None,
                                    },
                                ],
                                max_items: 3,
                                context_label: Some("Hotspots".to_string()),
                            }
                        }
                    }
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
                        Link { to: Route::FlevolandDashboard, span { "Provincie Flevoland" } }
                        span { " \u{203A} " }
                        span { class: "current", "Archiefoverzicht" }
                    }
                }
                div { class: "dashboard-grid",
                    div {
                        Panel { title: "Archiefstatus".to_string(),
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4C1}" }
                                div { class: "label", "Totaal archief" }
                                div { class: "value", "724 documenten" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "Permanent bewaard" }
                                div { class: "value", "532" }
                            }
                            div { class: "compliance-indicator warning",
                                div { class: "icon", "!" }
                                div { class: "label", "Vernietigbaar" }
                                div { class: "value", "17" }
                            }
                            div { class: "compliance-indicator warning",
                                div { class: "icon", "!" }
                                div { class: "label", "Actie vereist" }
                                div { class: "value", "28" }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Vernietigingen".to_string(),
                            h4 { style: "margin-bottom: 10px;", "Documenten klaar voor vernietiging" }
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F5D1}" }
                                    div { class: "document-info",
                                        h4 { "Subsidieaanvragen 2015-2016" }
                                        div { class: "meta", "Bewaartermijn verstreken op 1 jan 2026" }
                                        span { class: "tag alert", "Vernietigbaar" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F5D1}" }
                                    div { class: "document-info",
                                        h4 { "Beleidsnotities mobiliteit 2014" }
                                        div { class: "meta", "Bewaartermijn verstreken op 1 jan 2025" }
                                        span { class: "tag alert", "Vernietigbaar" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F5D1}" }
                                    div { class: "document-info",
                                        h4 { "Klachtenregistratie 2015" }
                                        div { class: "meta", "Bewaartermijn 5 jaar verstreken" }
                                        span { class: "tag alert", "Vernietigbaar" }
                                    }
                                }
                            }
                            button { class: "btn btn-warning", style: "margin-top: 10px; width: 100%;",
                                "Start vernietigingsronde"
                            }
                        }
                    }
                    div {
                        Panel { title: "Overbrenging naar Nationaal Archief".to_string(),
                            h4 { style: "margin-bottom: 10px;", "Documenten wachten op overbrenging" }
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CE}" }
                                    div { class: "document-info",
                                        h4 { "Provinciale Verordeningen 2010-2015" }
                                        div { class: "meta", "Permanent te bewaren" }
                                        span { class: "tag warning", "Wachten op overbrenging" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CE}" }
                                    div { class: "document-info",
                                        h4 { "Collegebesluiten 2010-2015" }
                                        div { class: "meta", "Permanent te bewaren" }
                                        span { class: "tag warning", "Wachten op overbrenging" }
                                    }
                                }
                            }
                            button { class: "btn btn-primary", style: "margin-top: 10px; width: 100%;",
                                "Plan overbrenging"
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Archiefwet 2027".to_string(),
                            p { style: "font-size: 0.875rem; color: #666;",
                                "Per 1 januari 2027 treedt de nieuwe Archiefwet in werking. Belangrijke wijzigingen:"
                            }
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{2139}" }
                                    div { class: "document-info",
                                        h4 { "Digitale duurzame bewaring verplicht" }
                                        div { class: "meta", "Analoge archieven moeten gedigitaliseerd worden" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{2139}" }
                                    div { class: "document-info",
                                        h4 { "Metadata-standaarden" }
                                        div { class: "meta", "Verplichting voor consistente metadata" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{2139}" }
                                    div { class: "document-info",
                                        h4 { "Selectie op creatie" }
                                        div { class: "meta", "Directe selectie bij documentcreatie" }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        Panel { title: "Rapportages".to_string(),
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CA}" }
                                    div { class: "document-info",
                                        h4 { "Jaarrapport 2025" }
                                        div { class: "meta", "Archiefstatus eind 2025" }
                                        button { class: "btn btn-small btn-outline", "Download" }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CA}" }
                                    div { class: "document-info",
                                        h4 { "PROVISA Compliance Rapport" }
                                        div { class: "meta", "Kwartaal 1 2026" }
                                        button { class: "btn btn-small btn-outline", "Download" }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Actiepunten".to_string(),
                            div { class: "compliance-indicator urgent",
                                div { class: "icon", "!" }
                                div { class: "label", "17 vernietigingen uitvoeren" }
                                div { class: "value", "Hoog" }
                            }
                            div { class: "compliance-indicator urgent",
                                div { class: "icon", "!" }
                                div { class: "label", "12 overbrengingen plannen" }
                                div { class: "value", "Midden" }
                            }
                            div { class: "compliance-indicator warning",
                                div { class: "icon", "!" }
                                div { class: "label", "Voorbereiden Archiefwet 2027" }
                                div { class: "value", "Langer termijn" }
                            }
                        }
                    }
                }
            }
        }
    }
}
