//! Provincie Flevoland dashboard page
//!
//! Dit dashboard toont:
//! - PROVISA compliance (provinciale selectielijsten)
//! - PETRA procesclassificatie
//! - Hotspot monitoring
//! - Woo-documenten specifiek voor Flevoland

use dioxus::prelude::*;
use iou_regels::{
    PetraCategorie, ProvisaSelectielijst, HotspotRegister, Archiefwaarde,
};

use crate::components::{AppCard, Header, Panel, TimelineEvent, TimelineEventType, Timeline};
use crate::state::{AppState, UserInfo};
use crate::Route;

/// Document met PROVISA-beoordeling
struct ProvisaDocument {
    titel: &'static str,
    samenvatting: &'static str,
    datum: &'static str,
    soort: &'static str,
    bron_id: &'static str,
    url: &'static str,
    /// PETRA categorie
    categorie: PetraCategorie,
    /// Bewaartermijn in jaren (None = permanent)
    bewaartermijn: Option<u32>,
    /// Archiefwaarde volgens PROVISA
    archiefwaarde: Archiefwaarde,
    /// Vernietigingsdatum (indien van toepassing)
    vernietigingsdatum: Option<&'static str>,
    /// Overbrengingsdatum (indien permanent)
    overbrengingsdatum: Option<&'static str>,
}

impl ProvisaDocument {
    /// Formatted bewaartermijn tekst
    fn bewaartermijn_tekst(&self) -> String {
        match self.bewaartermijn {
            None => "Permanent".to_string(),
            Some(jaren) => format!("{} jaar", jaren),
        }
    }

    /// CSS class voor archiefwaarde badge
    fn archief_class(&self) -> String {
        match self.archiefwaarde {
            Archiefwaarde::Permanent => "badge-success".to_string(),
            Archiefwaarde::Tijdelijk => "badge-info".to_string(),
        }
    }

    /// Of het document actie vereist
    fn actie_vereist(&self) -> bool {
        self.vernietigingsdatum.is_some() || self.overbrengingsdatum.is_some()
    }

    /// Achtergrondkleur voor icon
    fn icon_kleur(&self) -> String {
        match self.archiefwaarde {
            Archiefwaarde::Permanent => "#0066CC".to_string(),
            Archiefwaarde::Tijdelijk => "#F59E0B".to_string(),
        }
    }

    /// CSS class voor archiefwaarde tag
    fn archief_tag_class(&self) -> String {
        match self.archiefwaarde {
            Archiefwaarde::Permanent => "tag success".to_string(),
            Archiefwaarde::Tijdelijk => "tag warning".to_string(),
        }
    }
}

/// Concrete Woo-documenten van Flevoland met PROVISA-beoordeling
const PROVISA_DOCUMENTEN: &[ProvisaDocument] = &[
    ProvisaDocument {
        titel: "Kennisgeving Projectbesluit en MER Rondweg Lelystad-Zuid",
        samenvatting: "Kennisgeving van het projectbesluit en milieueffectrapport voor de Rondweg Lelystad-Zuid. Een grote infrastructuurproject met significante ruimtelijke impact.",
        datum: "30 jan 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1767",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1767.html",
        categorie: PetraCategorie::RuimtelijkePlanning,
        bewaartermijn: None,  // Permanent
        archiefwaarde: Archiefwaarde::Permanent,
        vernietigingsdatum: None,
        overbrengingsdatum: Some("2036"),  // 10 jaar na creatie
    },
    ProvisaDocument {
        titel: "Provinciale Ruimtelijke Verordening Flevoland 2024-2028",
        samenvatting: "De PRV geeft het ruimtelijk beleid weer voor de gehele provincie, inclusief zonering voor windenergie en zonne-energie.",
        datum: "15 jan 2024",
        soort: "Provinciale Verordening",
        bron_id: "prv-2024",
        url: "https://zoek.officielebekendmakingen.nl/prv-2024.html",
        categorie: PetraCategorie::RuimtelijkePlanning,
        bewaartermijn: None,  // Permanent
        archiefwaarde: Archiefwaarde::Permanent,
        vernietigingsdatum: None,
        overbrengingsdatum: Some("2034"),  // 10 jaar na creatie
    },
    ProvisaDocument {
        titel: "Besluit omgevingsvergunning Natura 2000 zandwinning IJsselmeer",
        samenvatting: "Besluit tot weigering van omgevingsvergunning voor zandwinning in Natura 2000 gebied.",
        datum: "28 jan 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1405",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1405.html",
        categorie: PetraCategorie::NatuurLandschap,
        bewaartermijn: None,  // Permanent - natuurwetgeving is altijd permanent
        archiefwaarde: Archiefwaarde::Permanent,
        vernietigingsdatum: None,
        overbrengingsdatum: Some("2036"),
    },
    ProvisaDocument {
        titel: "Collegebesluit 15 april 2024 - Duurzaamheidsagenda",
        samenvatting: "Besluit van Gedeputeerde Staten vaststelling van de duurzaamheidsagenda 2024-2030.",
        datum: "15 apr 2024",
        soort: "Collegebesluit",
        bron_id: "gs-2024-089",
        url: "https://zoek.officielebekendmakingen.nl/gs-2024-089.html",
        categorie: PetraCategorie::EnergieKlimaat,
        bewaartermijn: None,  // Permanent
        archiefwaarde: Archiefwaarde::Permanent,
        vernietigingsdatum: None,
        overbrengingsdatum: Some("2034"),
    },
    ProvisaDocument {
        titel: "Subsidiebeschikking Duurzaam Flevoland 2024-123",
        samenvatting: "Beschikking in het kader van de subsidieregeling Duurzaam Flevoland voor energiebesparende maatregelen bij MKB-bedrijf.",
        datum: "8 feb 2024",
        soort: "Subsidiebeschikking",
        bron_id: "subsidie-2024-123",
        url: "https://zoek.officielebekendmakingen.nl/subsidie-2024-123.html",
        categorie: PetraCategorie::Economie,
        bewaartermijn: Some(5),  // 5 jaar voor subsidiebeschikkingen
        archiefwaarde: Archiefwaarde::Tijdelijk,
        vernietigingsdatum: Some("2029-02-08"),
        overbrengingsdatum: None,
    },
    ProvisaDocument {
        titel: "Ontheffing helikopterlanding provincie Flevoland 2026",
        samenvatting: "Wet Luchtvaart generieke ontheffing voor helikopterlandingen in Flevoland.",
        datum: "29 jan 2026",
        soort: "Provinciaal blad",
        bron_id: "prb-2026-1457",
        url: "https://zoek.officielebekendmakingen.nl/prb-2026-1457.html",
        categorie: PetraCategorie::VerkeerVervoer,
        bewaartermijn: Some(10),  // 10 jaar
        archiefwaarde: Archiefwaarde::Tijdelijk,
        vernietigingsdatum: Some("2036-01-29"),
        overbrengingsdatum: None,
    },
    ProvisaDocument {
        titel: "Ondermandaat Bedrijfsvoering ODFL",
        samenvatting: "Gewijzigd ondermandaat voor de bedrijfsvoering van de Omgevingsdienst Flevoland & Gooi en Vechtstreek.",
        datum: "4 feb 2026",
        soort: "Blad gemeenschappelijke regeling",
        bron_id: "bgr-2026-301",
        url: "https://zoek.officielebekendmakingen.nl/bgr-2026-301.html",
        categorie: PetraCategorie::Bestuur,
        bewaartermijn: None,  // Permanent
        archiefwaarde: Archiefwaarde::Permanent,
        vernietigingsdatum: None,
        overbrengingsdatum: Some("2036"),
    },
    ProvisaDocument {
        titel: "Klachtenregistratie Q1 2020",
        samenvatting: "Overzicht van ontvangen klachten in het eerste kwartaal van 2020.",
        datum: "1 apr 2020",
        soort: "Interne registratie",
        bron_id: "klacht-2020-q1",
        url: "#",
        categorie: PetraCategorie::Communicatie,
        bewaartermijn: Some(5),  // 5 jaar voor klachtenregistratie
        archiefwaarde: Archiefwaarde::Tijdelijk,
        vernietigingsdatum: Some("2025-04-01"),  // Reeds vernietigbaar!
        overbrengingsdatum: None,
    },
    ProvisaDocument {
        titel: "Beleidsnotities mobiliteit 2014",
        samenvatting: "Verzameling van beleidsnotities over provinciaal mobiliteitsbeleid uit 2014.",
        datum: "31 dec 2014",
        soort: "Beleidsnotities",
        bron_id: "beleid-2014-mob",
        url: "#",
        categorie: PetraCategorie::Strategie,
        bewaartermijn: Some(10),  // 10 jaar voor beleidsnotities
        archiefwaarde: Archiefwaarde::Tijdelijk,
        vernietigingsdatum: Some("2025-01-01"),  // Reeds vernietigbaar!
        overbrengingsdatum: None,
    },
    ProvisaDocument {
        titel: "Subsidieaanvragen 2015-2016",
        samenvatting: "Geweigerde en goedgekeurde subsidieaanvragen uit de periode 2015-2016.",
        datum: "31 dec 2016",
        soort: "Interne registratie",
        bron_id: "subsidie-2015-2016",
        url: "#",
        categorie: PetraCategorie::Economie,
        bewaartermijn: Some(5),  // 5 jaar voor subsidieadministratie
        archiefwaarde: Archiefwaarde::Tijdelijk,
        vernietigingsdatum: Some("2022-01-01"),  // Reeds vernietigbaar!
        overbrengingsdatum: None,
    },
];

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
                                Link { to: Route::FlevolandArchitectuur,
                                    AppCard {
                                        name: "IOU Architectuur".to_string(),
                                        description: "DMN regels en CPSV integratie".to_string(),
                                        badge: "Nieuw".to_string(),
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

/// IOU Architectuur integratie voor PROVISA
#[component]
pub fn FlevolandArchitectuur() -> Element {
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
                        span { class: "current", "IOU Architectuur" }
                    }
                }

                div { class: "concept-intro",
                    h2 { "IOU Architectuur voor PROVISA" }
                    p {
                        "Hoe de IOU architectuur PROVISA compliance automatiseert via "
                        strong { "DMN regels, CPSV Editor en Linked Data" }
                        "."
                    }
                }

                // Architectuur diagram
                Panel { title: "PROVISA in IOU Architectuur".to_string(),
                    div { style: "background: #f8f6ff; padding: 20px; border-radius: 8px; margin-bottom: 20px;",
                        h3 { style: "margin-top: 0;", "\u{1F4C1} Architectuur Overzicht" }
                        pre { style: "background: #2d2d2d; color: #f8f8f2; padding: 15px; border-radius: 4px; overflow-x: auto; font-size: 0.8rem;",
r#"graph TB
    subgraph "Flevoland PROVISA Workflow"
        A[Ambtenaar] -->|CPSV Editor| B[CPSV-AP Dienst]
        B -->|TTL/RDF| C[TriplyDB KG]
        D[PROVISA DMN Rules] -->|Deploy| E[Operaton Engine]
        F[Business API] -->|REST| E
        E -->|Evaluate| G[Archiefwaarde]
        H[Linked Data Explorer] -->|SPARQL| C
        A -->|Document Upload| F
    end

    subgraph "IOU Platform"
        I[Keycloak IAM] -->|OIDC Token| F
        J[Orchestration Service] -->|DMN Deploy| E
    end

    style B fill:#7C4DFF
    style C fill:#00BCD4
    style D fill:#4CAF50
    style E fill:#FF9800
    style G fill:#4CAF50"#
                        }
                    }

                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                        // Left column - Components
                        div {
                            h4 { "Componenten" }
                            ul { class: "document-list",
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F4CB}" }
                                    div { class: "document-info",
                                        h4 { "CPSV Editor" }
                                        div { class: "meta", "Maakt CPSV-AP 3.2.0 bestanden met PROVISA metadata" }
                                        a {
                                            href: "https://cpsv-editor.open-regels.nl",
                                            target: "_blank",
                                            class: "tag woo",
                                            "cpsv-editor.open-regels.nl \u{2197}"
                                        }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{2696}" }
                                    div { class: "document-info",
                                        h4 { "PROVISA DMN Rules" }
                                        div { class: "meta", "Decision tables voor bewaartermijn bepaling" }
                                        a {
                                            href: "https://operaton.open-regels.nl",
                                            target: "_blank",
                                            class: "tag woo",
                                            "operaton.open-regels.nl \u{2197}"
                                        }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F517}" }
                                    div { class: "document-info",
                                        h4 { "Linked Data Explorer" }
                                        div { class: "meta", "SPARQL queries op PROVISA kennisgraaf" }
                                        a {
                                            href: "https://linkeddata.open-regels.nl",
                                            target: "_blank",
                                            class: "tag woo",
                                            "linkeddata.open-regels.nl \u{2197}"
                                        }
                                    }
                                }
                                li { class: "document-item",
                                    div { class: "document-icon", "\u{1F3E2}" }
                                    div { class: "document-info",
                                        h4 { "Business API" }
                                        div { class: "meta", "Secure gateway met Keycloak authenticatie" }
                                        a {
                                            href: "https://backend.linkeddata.open-regels.nl",
                                            target: "_blank",
                                            class: "tag woo",
                                            "backend.linkeddata.open-regels.nl \u{2197}"
                                        }
                                    }
                                }
                            }
                        }

                        // Right column - Flow
                        div {
                            h4 { "PROVISA Workflow" }
                            div { class: "workflow-steps",
                                div { class: "workflow-step",
                                    div { class: "step-number", "1" }
                                    div { class: "step-content",
                                        h5 { "Document Aanbieden" }
                                        p { "Ambtenaar upload document via CPSV Editor" }
                                    }
                                }
                                div { class: "workflow-step",
                                    div { class: "step-number", "2" }
                                    div { class: "step-content",
                                        h5 { "Classificatie" }
                                        p { "Systeem classificeert volgens PETRA categorie" }
                                    }
                                }
                                div { class: "workflow-step",
                                    div { class: "step-number", "3" }
                                    div { class: "step-content",
                                        h5 { "PROVISA DMN Evaluate" }
                                        p { "DMN engine bepaalt bewaartermijn en archiefwaarde" }
                                    }
                                }
                                div { class: "workflow-step",
                                    div { class: "step-number", "4" }
                                    div { class: "step-content",
                                        h5 { "Resultaat Opslaan" }
                                        p { "Bewaartermijn opslaan in TriplyDB kennisgraaf" }
                                    }
                                }
                                div { class: "workflow-step",
                                    div { class: "step-number", "5" }
                                    div { class: "step-content",
                                        h5 { "Actie (Vernietigen/Overbrengen)" }
                                        p { "Automatische melding voor actie vereist" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { style: "height: 20px;" }

                // Standaarden
                Panel { title: "Standaarden Mapping".to_string(),
                    div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px;",
                        div { class: "standard-card",
                            div { class: "standard-header", style: "background: #7C4DFF;",
                                        h4 { "PROVISA" }
                                        span { "Provinciaal" }
                                    }
                                    ul { style: "margin: 10px 0; padding-left: 20px; font-size: 0.9rem;",
                                        li { "PETRA categorieën" }
                                        li { "Bewaartermijnen" }
                                        li { "Concordans" }
                                        li { "Hotspots" }
                                    }
                                }
                                div { class: "standard-card",
                                    div { class: "standard-header", style: "background: #00BCD4;",
                                        h4 { "CPSV-AP 3.2.0" }
                                        span { "Europees" }
                                    }
                                    ul { style: "margin: 10px 0; padding-left: 20px; font-size: 0.9rem;",
                                        li { "Public Service Vocabulary" }
                                        li { "Dienst metadata" }
                                        li { "Regel koppeling" }
                                    }
                                }
                                div { class: "standard-card",
                                    div { class: "standard-header", style: "background: #4CAF50;",
                                        h4 { "DMN 1.4" }
                                        span { "OMG" }
                                    }
                                    ul { style: "margin: 10px 0; padding-left: 20px; font-size: 0.9rem;",
                                        li { "Decision Tables" }
                                        li { "FEEL expressies" }
                                        li { "Hit Policy" }
                                    }
                                }
                            }
                }

                div { style: "height: 20px;" }

                // Voorbeelden
                Panel { title: "PROVISA DMN Voorbeelden".to_string(),
                    p { style: "margin-bottom: 15px; color: #666;",
                        "Concrete DMN decision tables voor PROVISA implementatie."
                    }
                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 15px;",
                        // DMN Example 1
                        div { class: "dmn-example",
                            h4 { "Bewaartermijn Ruimtelijke Planning" }
                            div { style: "background: #f5f5f5; padding: 10px; border-radius: 4px; font-family: monospace; font-size: 0.8rem;",
                                table { style: "width: 100%; border-collapse: collapse;",
                                    thead {
                                        tr { style: "background: #ddd;",
                                            th { style: "border: 1px solid #999; padding: 8px; text-align: left;", "Input" }
                                            th { style: "border: 1px solid #999; padding: 8px; text-align: left;", "PRV?" }
                                            th { style: "border: 1px solid #999; padding: 8px; text-align: left;", "Besluit?" }
                                            th { style: "border: 1px solid #999; padding: 8px; text-align: left;", "Bewaartermijn" }
                                        }
                                    }
                                    tbody {
                                        tr { style: "border-bottom: 1px solid #ddd;",
                                            td { style: "border: 1px solid #ddd; padding: 8px;", "Structuurplan" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; text-align: center;", "\u{2705}" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; text-align: center;", "\u{2705}" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; color: #0066CC;", "Permanent" }
                                        }
                                        tr { style: "border-bottom: 1px solid #ddd;",
                                            td { style: "border: 1px solid #ddd; padding: 8px;", "Omgevingsvergunning" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; text-align: center;", "\u{2705}" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; text-align: center;", "\u{2705}" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; color: #0066CC;", "Permanent" }
                                        }
                                        tr { style: "border-bottom: 1px solid #ddd;",
                                            td { style: "border: 1px solid #ddd; padding: 8px;", "Beleidsnotitie" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; text-align: center;", "\u{274C}" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; text-align: center;", "\u{2705}" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; color: #F59E0B;", "10 jaar" }
                                        }
                                    }
                                }
                            }
                        }

                        // DMN Example 2
                        div { class: "dmn-example",
                            h4 { "Hotspot Upgrade Check" }
                            div { style: "background: #f5f5f5; padding: 10px; border-radius: 4px; font-family: monospace; font-size: 0.8rem;",
                                table { style: "width: 100%; border-collapse: collapse;",
                                    thead {
                                        tr { style: "background: #ddd;",
                                            th { style: "border: 1px solid #999; padding: 8px; text-align: left;", "Document" }
                                            th { style: "border: 1px solid #999; padding: 8px; text-align: left;", "Hotspot?" }
                                            th { style: "border: 1px solid #999; padding: 8px; text-align: left;", "Upgrade" }
                                        }
                                    }
                                    tbody {
                                        tr { style: "border-bottom: 1px solid #ddd;",
                                            td { style: "border: 1px solid #ddd; padding: 8px;", "Lelystad Airport" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; text-align: center; color: #4CAF50;", "Actief" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; color: #0066CC;", "Ja" }
                                        }
                                        tr { style: "border-bottom: 1px solid #ddd;",
                                            td { style: "border: 1px solid #ddd; padding: 8px;", "Almere Stad" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; text-align: center; color: #4CAF50;", "Actief" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; color: #0066CC;", "Ja" }
                                        }
                                        tr { style: "border-bottom: 1px solid #ddd;",
                                            td { style: "border: 1px solid #ddd; padding: 8px;", "Routine vergunning" }
                                            td { style: "border: 1px solid #ddd; padding: 8px; text-align: center; color: #999;", "Nee" }
                                            td { style: "border: 1px solid #ddd; padding: 8px;", "Nee" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { style: "height: 20px;" }

                // Interactief Workflow Demo met echt Woo-document
                Panel { title: "Workflow Demo: PRV Rondweg Lelystad-Zuid".to_string(),
                    p { style: "margin-bottom: 15px; color: #666;",
                        "Volg een concreet Woo-document door de PROVISA workflow."
                    }

                    // Document header
                    div { style: "background: #f8f6ff; padding: 15px; border-radius: 8px; border-left: 4px solid #7C4DFF; margin-bottom: 20px;",
                        div { style: "display: flex; justify-content: space-between; align-items: start;",
                            div {
                                h4 { style: "margin: 0 0 5px 0;", "\u{1F4C4} Kennisgeving Projectbesluit en MER Rondweg Lelystad-Zuid" }
                                div { style: "font-size: 0.85rem; color: #666;",
                                    "Provinciaal blad | 30 jan 2026 | prb-2026-1767"
                                }
                                a {
                                    href: "https://zoek.officielebekendmakingen.nl/prb-2026-1767.html",
                                    target: "_blank",
                                    style: "color: #0066CC; text-decoration: none; font-size: 0.85rem;",
                                    "Bekijk op open.overheid.nl \u{2197}"
                                }
                            }
                            span { class: "tag woo", "Woo Document" }
                        }
                    }

                    // Workflow interactief
                    div { style: "display: grid; grid-template-columns: repeat(2, 1fr); gap: 20px;",
                        // Links - Workflow stappen
                        div {
                            h4 { "PROVISA Workflow Stappen" }
                            div { class: "workflow-demo",
                                // Stap 1
                                div { class: "workflow-demo-step completed",
                                    div { class: "workflow-demo-header",
                                        div { class: "step-number-small", "1" }
                                        div { class: "step-title", "CPSV Editor: Dienst Aanmaken" }
                                        div { class: "step-status", "\u{2705}" }
                                    }
                                    div { class: "workflow-demo-content",
                                        p { style: "margin: 5px 0; font-size: 0.85rem;",
                                            "Dienst: projectbesluit-ruimtelijke-planning"
                                        }
                                        div { style: "margin-top: 8px;",
                                            span { class: "tag provincie", "PETRA: Ruimtelijke Planning" }
                                            span { class: "tag woo", "CPSV-AP 3.2.0" }
                                        }
                                        div { style: "margin-top: 5px; font-size: 0.8rem; color: #888;",
                                            "a href=\"cpsv-editor.open-regels.nl\" \u{2192}"
                                        }
                                    }
                                }

                                // Stap 2
                                div { class: "workflow-demo-step completed",
                                    div { class: "workflow-demo-header",
                                        div { class: "step-number-small", "2" }
                                        div { class: "step-title", "Classificatie: PETRA" }
                                        div { class: "step-status", "\u{2705}" }
                                    }
                                    div { class: "workflow-demo-content",
                                        p { style: "margin: 5px 0; font-size: 0.85rem;",
                                            "Systeem detecteert: Projectbesluit MER"
                                        }
                                        div { style: "margin-top: 8px;",
                                            span { class: "tag success", "PRV relevant" }
                                            span { class: "tag info", "Besluit formaliteit" }
                                        }
                                    }
                                }

                                // Stap 3
                                div { class: "workflow-demo-step completed",
                                    div { class: "workflow-demo-header",
                                        div { class: "step-number-small", "3" }
                                        div { class: "step-title", "PROVISA DMN: Evaluate" }
                                        div { class: "step-status", "\u{2705}" }
                                    }
                                    div { class: "workflow-demo-content",
                                        div { style: "background: #f5f5f5; padding: 10px; border-radius: 4px; font-family: monospace; font-size: 0.75rem; margin-top: 8px;",
                                            table { style: "width: 100%; border-collapse: collapse;",
                                                thead {
                                                    tr { style: "background: #ddd;",
                                                        th { style: "border: 1px solid #999; padding: 4px; font-size: 0.7rem;", "Input" }
                                                        th { style: "border: 1px solid #999; padding: 4px; font-size: 0.7rem;", "Type" }
                                                        th { style: "border: 1px solid #999; padding: 4px; font-size: 0.7rem;", "Resultaat" }
                                                    }
                                                }
                                                tbody {
                                                    tr {
                                                        td { style: "border: 1px solid #ddd; padding: 4px;", "PRV document" }
                                                        td { style: "border: 1px solid #ddd; padding: 4px;", "Projectbesluit" }
                                                        td { style: "border: 1px solid #ddd; padding: 4px; color: #0066CC; font-weight: 600;", "\u{2192} Permanent" }
                                                    }
                                                    tr {
                                                        td { style: "border: 1px solid #ddd; padding: 4px;", "MER aanwezig" }
                                                        td { style: "border: 1px solid #ddd; padding: 4px;", "Ja" }
                                                        td { style: "border: 1px solid #ddd; padding: 4px; color: #0066CC; font-weight: 600;", "\u{2192} Permanent" }
                                                    }
                                                    tr {
                                                        td { style: "border: 1px solid #ddd; padding: 4px;", "Hotspot check" }
                                                        td { style: "border: 1px solid #ddd; padding: 4px;", "Nee" }
                                                        td { style: "border: 1px solid #ddd; padding: 4px; color: #666;", "\u{2192} Geen upgrade" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Stap 4
                                div { class: "workflow-demo-step completed",
                                    div { class: "workflow-demo-header",
                                        div { class: "step-number-small", "4" }
                                        div { class: "step-title", "TriplyDB: Opslaan" }
                                        div { class: "step-status", "\u{2705}" }
                                    }
                                    div { class: "workflow-demo-content",
                                        pre { style: "background: #2d2d2d; color: #f8f8f2; padding: 10px; border-radius: 4px; font-size: 0.7rem; overflow-x: auto;",
                                            "@prefix prov: <https://provincie.flevoland.nl/provisa/>
prov:prb-2026-1767 a prov:Archiefwaarde ;
    prov:bewaartermijn \"Permanent\" ;
    prov:petraCategorie prov:RuimtelijkePlanning ;
    prov:hotspotUpgrade false ."
                                        }
                                    }
                                }

                                // Stap 5
                                div { class: "workflow-demo-step warning",
                                    div { class: "workflow-demo-header",
                                        div { class: "step-number-small", "5" }
                                        div { class: "step-title", "Actie: Overbrenging NA (2046)" }
                                        div { class: "step-status", "!" }
                                    }
                                    div { class: "workflow-demo-content",
                                        p { style: "margin: 5px 0; font-size: 0.85rem;",
                                            "Document is permanent bewaren. Overbrenging naar Nationaal Archief gepland over 20 jaar."
                                        }
                                        div { style: "margin-top: 8px;",
                                            span { class: "tag alert", "Overbrenging: 2046-01-30" }
                                        }
                                        div { style: "margin-top: 10px; padding: 10px; background: #fff3e0; border-radius: 4px;",
                                            div { style: "display: flex; gap: 8px;",
                                                span { "\u{23F0}" }
                                                span { style: "font-size: 0.85rem;",
                                                    strong { "Automatische reminder: " }
                                                    "20 jaar na publicatie"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Rechts - Resultaat
                        div {
                            h4 { "PROVISA Beoordeling Resultaat" }
                            div { style: "background: #f8f9fa; padding: 20px; border-radius: 8px; margin-top: 15px;",
                                div { style: "text-align: center; margin-bottom: 20px;",
                                    div { class: "compliance-indicator ok",
                                        div { class: "icon", "\u{2713}" }
                                        div { class: "label", "Archiefwaarde" }
                                        div { class: "value", "PERMANENT" }
                                    }
                                }

                                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 15px;",
                                    div {
                                        h5 { "Bewaartermijn" }
                                        p { style: "font-size: 1.5rem; font-weight: 700; color: #0066CC; margin: 10px 0;",
                                            "Permanent"
                                        }
                                        p { style: "font-size: 0.85rem; color: #666;",
                                            "Omdat het een Projectbesluit MER betreft in het kader van de PRV."
                                        }
                                    }
                                    div {
                                        h5 { "Overbrenging" }
                                        p { style: "font-size: 1.2rem; font-weight: 600; color: #F59E0B; margin: 10px 0;",
                                            "2046-01-30"
                                        }
                                        p { style: "font-size: 0.85rem; color: #666;",
                                            "20 jaar na publicatie (Archiefwet 1995)"
                                        }
                                    }
                                }

                                div { style: "margin-top: 20px; padding-top: 15px; border-top: 1px solid #ddd;",
                                    h5 { "Concordans Referentie" }
                                    ul { style: "margin: 10px 0; padding-left: 20px; font-size: 0.85rem;",
                                        li { "PROVISA 2020, categorie 2.1: Ruimtelijke planning" }
                                        li { "Toelichting: Projectbesluiten en MER zijn permanent" }
                                        li { "Besluittype: Provinciaal Verordening / Projectbesluit" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { style: "height: 20px;" }

                // Call to action
                Panel { title: "Aan de Slag".to_string(),
                    div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px;",
                        a {
                            href: "https://cpsv-editor.open-regels.nl",
                            target: "_blank",
                            class: "app-card",
                            style: "text-decoration: none; display: block;",
                            div { class: "document-icon", style: "background: #7C4DFF;", "\u{270F}" }
                            h4 { "Start CPSV Editor" }
                            p { "Maak PROVISA diensten" }
                        }
                        a {
                            href: "https://linkeddata.open-regels.nl",
                            target: "_blank",
                            class: "app-card",
                            style: "text-decoration: none; display: block;",
                            div { class: "document-icon", style: "background: #00BCD4;", "\u{1F50D}" }
                            h4 { "Linked Data Explorer" }
                            p { "Query PROVISA graaf" }
                        }
                        a {
                            href: "https://iou-architectuur.open-regels.nl",
                            target: "_blank",
                            class: "app-card",
                            style: "text-decoration: none; display: block;",
                            div { class: "document-icon", style: "background: #4CAF50;", "\u{1F4DD}" }
                            h4 { "IOU Documentatie" }
                            p { "Architectuur details" }
                        }
                    }
                }
            }
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

                        Panel { title: "PROVISA Statistieken".to_string(),
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F4C1}" }
                                div { class: "label", "Totaal beoordeeld" }
                                div { class: "value", "{PROVISA_DOCUMENTEN.len()}" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "Permanent" }
                                div { class: "value",
                                    "{PROVISA_DOCUMENTEN.iter().filter(|d| d.archiefwaarde == Archiefwaarde::Permanent).count()}"
                                }
                            }
                            div { class: "compliance-indicator warning",
                                div { class: "icon", "\u{23F1}" }
                                div { class: "label", "Tijdelijk" }
                                div { class: "value",
                                    "{PROVISA_DOCUMENTEN.iter().filter(|d| d.archiefwaarde == Archiefwaarde::Tijdelijk).count()}"
                                }
                            }
                            div { class: "compliance-indicator alert",
                                div { class: "icon", "!" }
                                div { class: "label", "Actie vereist" }
                                div { class: "value",
                                    "{PROVISA_DOCUMENTEN.iter().filter(|d| d.actie_vereist()).count()}"
                                }
                            }
                        }

                        div { style: "height: 20px;" }

                        Panel { title: "Acties Vereist".to_string(),
                            p { style: "font-size: 0.875rem; color: #666; margin-bottom: 10px;",
                                "Documenten die onmiddellijke actie vereisten."
                            }
                            for doc in PROVISA_DOCUMENTEN.iter().filter(|d| d.actie_vereist()) {
                                div { class: "compliance-indicator alert",
                                    div { class: "icon", "!" }
                                    div { class: "label", "{doc.titel}" }
                                    div { class: "value",
                                        if doc.vernietigingsdatum.is_some() {
                                            "Vernietigen"
                                        } else {
                                            "Overbrengen"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        Panel { title: "Documenten met PROVISA Beoordeling".to_string(),
                            p { style: "font-size: 0.875rem; color: #666; margin-bottom: 15px;",
                                "Concrete Woo-documenten met toegepaste PROVISA selectielijst beoordeling."
                            }
                            ul { class: "document-list",
                                for doc in PROVISA_DOCUMENTEN {
                                    li {
                                        class: if doc.actie_vereist() { "document-item urgent" } else { "document-item" },
                                        div { class: "document-icon",
                                            style: "background: {doc.icon_kleur()};",
                                        }
                                        div { class: "document-info",
                                            h4 { "{doc.titel}" }
                                            div { class: "meta",
                                                "{doc.soort} \u{2022} {doc.datum} \u{2022} {doc.bron_id}"
                                            }
                                            div { style: "margin-top: 6px; display: flex; gap: 8px; flex-wrap: wrap;",
                                                span {
                                                    class: "tag provincie",
                                                    "{doc.categorie}"
                                                }
                                                span {
                                                    class: "{doc.archief_tag_class()}",
                                                    "{doc.archiefwaarde}"
                                                }
                                                span { class: "tag info", "Bewaartermijn: {doc.bewaartermijn_tekst()}" }
                                                if let Some(vernietiging) = doc.vernietigingsdatum {
                                                    span { class: "tag alert", "Vernietigen: {vernietiging}" }
                                                }
                                                if let Some(overbrenging) = doc.overbrengingsdatum {
                                                    span { class: "tag success", "Overbrengen: {overbrenging}" }
                                                }
                                            }
                                            p { style: "font-size: 0.85rem; color: #666; margin-top: 6px;",
                                                "{doc.samenvatting}"
                                            }
                                        }
                                        a {
                                            href: "{doc.url}",
                                            target: "_blank",
                                            class: "btn btn-small btn-outline",
                                            style: "text-decoration: none;",
                                            "Bekijk \u{2197}"
                                        }
                                    }
                                }
                            }
                        }

                        div { style: "height: 20px;" }

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
