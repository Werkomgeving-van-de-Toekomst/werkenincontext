//! PROVISA Management Interface
//!
//! Advanced interface for managing PROVISA archive records with:
//! - Faceted search
//! - Hotspot management
//! - Version comparison (2020 vs 2014 vs 2005)
//! - AI-powered classification suggestions
//! - Retention period calculator

use dioxus::prelude::*;

use crate::state::AppState;
use crate::components::{Header, Panel};

#[derive(Clone, Debug, PartialEq)]
struct ProvisaRecord {
    id: String,
    title: String,
    dossier_nr: String,
    source_domain: String,
    provisa_version: ProvisaVersion,
    retention_period: Option<i32>,
    retention_start: Option<String>,
    destruction_date: Option<String>,
    transfer_date: Option<String>,
    classification: String,
    tags: Vec<String>,
    status: ProvisaStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProvisaVersion {
    Provisa2020,
    Provisa2014,
    Provisa2005,
}

impl ProvisaVersion {
    fn name(&self) -> &'static str {
        match self {
            ProvisaVersion::Provisa2020 => "PROVISA 2020",
            ProvisaVersion::Provisa2014 => "PROVISA 2014",
            ProvisaVersion::Provisa2005 => "PROVISA 2005",
        }
    }

    fn year(&self) -> i32 {
        match self {
            ProvisaVersion::Provisa2020 => 2020,
            ProvisaVersion::Provisa2014 => 2014,
            ProvisaVersion::Provisa2005 => 2005,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProvisaStatus {
    Active,
    Retained,
    Transferred,
    Destroyed,
    ReviewRequired,
}

impl ProvisaStatus {
    fn display(&self) -> &'static str {
        match self {
            ProvisaStatus::Active => "Actief",
            ProvisaStatus::Retained => "Bewaard",
            ProvisaStatus::Transferred => "Overgebracht",
            ProvisaStatus::Destroyed => "Vernietigd",
            ProvisaStatus::ReviewRequired => "Review Vereist",
        }
    }

    fn class(&self) -> &'static str {
        match self {
            ProvisaStatus::Active => "status-active",
            ProvisaStatus::Retained => "status-retained",
            ProvisaStatus::Transferred => "status-transferred",
            ProvisaStatus::Destroyed => "status-destroyed",
            ProvisaStatus::ReviewRequired => "status-warning",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SearchFilter {
    query: String,
    domain: Option<String>,
    version: Option<ProvisaVersion>,
    status: Option<ProvisaStatus>,
    retention_from: Option<i32>,
    retention_to: Option<i32>,
}

fn mock_records() -> Vec<ProvisaRecord> {
    vec![
        ProvisaRecord {
            id: "1".to_string(),
            title: "Projectbesluit Windpark Almere".to_string(),
            dossier_nr: "2024-WA-001".to_string(),
            source_domain: "Windpark Almere".to_string(),
            provisa_version: ProvisaVersion::Provisa2020,
            retention_period: Some(20),
            retention_start: Some("2024-01-15".to_string()),
            destruction_date: Some("2044-01-15".to_string()),
            transfer_date: None,
            classification: "Vertrouwelijk".to_string(),
            tags: vec!["windenergie".to_string(), "vergunning".to_string()],
            status: ProvisaStatus::Active,
        },
        ProvisaRecord {
            id: "2".to_string(),
            title: "Omgevingsvergunning Zandwinning IJsselmeer".to_string(),
            dossier_nr: "2023-ZI-042".to_string(),
            source_domain: "Waterstaat".to_string(),
            provisa_version: ProvisaVersion::Provisa2020,
            retention_period: Some(30),
            retention_start: Some("2023-06-01".to_string()),
            destruction_date: Some("2053-06-01".to_string()),
            transfer_date: None,
            classification: "Intern".to_string(),
            tags: vec!["zandwinning".to_string(), "water".to_string(), "vergunning".to_string()],
            status: ProvisaStatus::Active,
        },
        ProvisaRecord {
            id: "3".to_string(),
            title: "Provinciale Weg N705 Verbreding".to_string(),
            dossier_nr: "2019-NW-118".to_string(),
            source_domain: "Infrastructuur".to_string(),
            provisa_version: ProvisaVersion::Provisa2014,
            retention_period: Some(15),
            retention_start: Some("2019-03-10".to_string()),
            destruction_date: Some("2034-03-10".to_string()),
            transfer_date: None,
            classification: "Openbaar".to_string(),
            tags: vec!["infrastructuur".to_string(), "weg".to_string()],
            status: ProvisaStatus::Retained,
        },
        ProvisaRecord {
            id: "4".to_string(),
            title: "Natura 2000 Beheerplan Oostvaardersplassen".to_string(),
            dossier_nr: "2022-NO-008".to_string(),
            source_domain: "Natuur".to_string(),
            provisa_version: ProvisaVersion::Provisa2020,
            retention_period: Some(50), // Permanent, effectively
            retention_start: Some("2022-01-01".to_string()),
            destruction_date: None,
            transfer_date: Some("2072-01-01".to_string()),
            classification: "Openbaar".to_string(),
            tags: vec!["natura2000".to_string(), "natuur".to_string(), "oostvaardersplassen".to_string()],
            status: ProvisaStatus::Active,
        },
        ProvisaRecord {
            id: "5".to_string(),
            title: "Subsidiebeschikking Duurzame Energie".to_string(),
            dossier_nr: "2024-DE-156".to_string(),
            source_domain: "Energie".to_string(),
            provisa_version: ProvisaVersion::Provisa2020,
            retention_period: None, // Not yet set
            retention_start: None,
            destruction_date: None,
            transfer_date: None,
            classification: "Intern".to_string(),
            tags: vec!["subsidie".to_string(), "energie".to_string()],
            status: ProvisaStatus::ReviewRequired,
        },
    ]
}

#[component]
pub fn ProvisaManager() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut search_filter = use_signal(|| SearchFilter {
        query: String::new(),
        domain: None,
        version: None,
        status: None,
        retention_from: None,
        retention_to: None,
    });
    let mut records = use_signal(|| mock_records());
    let mut selected_record = use_signal(|| None::<ProvisaRecord>);
    let mut show_version_compare = use_signal(|| false);

    use_effect(move || {
        state.write().user = Some(crate::state::UserInfo::flevoland());
    });

    let filtered_records = records
        .read()
        .iter()
        .filter(|r| {
            let filter = search_filter.read();
            let query_match = filter.query.is_empty()
                || r.title.to_lowercase().contains(&filter.query.to_lowercase())
                || r.dossier_nr.to_lowercase().contains(&filter.query.to_lowercase());

            let domain_match = filter.domain.as_ref().map_or(true, |d| {
                r.source_domain.to_lowercase().contains(&d.to_lowercase())
            });

            let version_match = filter.version.as_ref().map_or(true, |v| &r.provisa_version == v);

            let status_match = filter.status.as_ref().map_or(true, |s| &r.status == s);

            query_match && domain_match && version_match && status_match
        })
        .cloned()
        .collect::<Vec<_>>();

    rsx! {
        Header {}

        main { class: "container",
            div { class: "page-header",
                h1 { "PROVISA Beheer" }
                p { "Beheer archieftermijnen volgens de Provinciale Archiefverordening" }
            }

            // Search and Filter Panel
            Panel { title: "Zoeken & Filteren".to_string(),
                div { class: "search-grid",
                    // Search input
                    div { class: "search-box",
                        input {
                            r#type: "text",
                            placeholder: "Zoek op titel of dossiernummer...",
                            value: "{search_filter.read().query}",
                            oninput: move |evt| {
                                search_filter.write().query = evt.value();
                            },
                        }
                    }

                    // Filters
                    div { class: "filter-group",
                        label { "Domein" }
                        select {
                            onchange: move |evt| {
                                let value = evt.value();
                                search_filter.write().domain = if value.is_empty() { None } else { Some(value) };
                            },
                            option { value: "", "Alle domeinen" }
                            option { value: "Windpark Almere", "Windpark Almere" }
                            option { value: "Waterstaat", "Waterstaat" }
                            option { value: "Infrastructuur", "Infrastructuur" }
                            option { value: "Natuur", "Natuur" }
                            option { value: "Energie", "Energie" }
                        }
                    }

                    div { class: "filter-group",
                        label { "PROVISA Versie" }
                        select {
                            onchange: move |evt| {
                                let value = evt.value();
                                search_filter.write().version = match value.as_str() {
                                    "2020" => Some(ProvisaVersion::Provisa2020),
                                    "2014" => Some(ProvisaVersion::Provisa2014),
                                    "2005" => Some(ProvisaVersion::Provisa2005),
                                    _ => None,
                                };
                            },
                            option { value: "", "Alle versies" }
                            option { value: "2020", "PROVISA 2020" }
                            option { value: "2014", "PROVISA 2014" }
                            option { value: "2005", "PROVISA 2005" }
                        }
                    }

                    div { class: "filter-group",
                        label { "Status" }
                        select {
                            onchange: move |evt| {
                                let value = evt.value();
                                search_filter.write().status = match value.as_str() {
                                    "active" => Some(ProvisaStatus::Active),
                                    "retained" => Some(ProvisaStatus::Retained),
                                    "transferred" => Some(ProvisaStatus::Transferred),
                                    "destroyed" => Some(ProvisaStatus::Destroyed),
                                    "review" => Some(ProvisaStatus::ReviewRequired),
                                    _ => None,
                                };
                            },
                            option { value: "", "Alle statussen" }
                            option { value: "active", "Actief" }
                            option { value: "retained", "Bewaard" }
                            option { value: "transferred", "Overgebracht" }
                            option { value: "destroyed", "Vernietigd" }
                            option { value: "review", "Review Vereist" }
                        }
                    }

                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            search_filter.set(SearchFilter {
                                query: String::new(),
                                domain: None,
                                version: None,
                                status: None,
                                retention_from: None,
                                retention_to: None,
                            });
                        },
                        "Filters wissen"
                    }
                }
            }

            // Results Panel
            div { style: "margin-top: 20px; display: flex; gap: 20px;",
                // Records List
                div { style: "flex: 1;",
                    Panel { title: "{filtered_records.len()} dossier(s)",
                        div { class: "record-list",
                            if filtered_records.is_empty() {
                                div { class: "empty-state",
                                    p { "Geen resultaten gevonden." }
                                }
                            } else {
                                // Render record items - simpler approach with closure
                                {
                                    let records_for_render = filtered_records.clone();
                                    records_for_render.iter().take(10).enumerate().map(|(idx, record)| {
                                        let record = record.clone();
                                        rsx! {
                                            div {
                                                class: "record-item",
                                                key: "{idx}",
                                                onclick: move |_| {
                                                    selected_record.set(Some(record.clone()));
                                                    show_version_compare.set(false);
                                                },
                                                div { class: "record-icon", "" }
                                                div { class: "record-info",
                                                    h4 { "{record.title}" }
                                                    div { class: "meta",
                                                        span { class: "tag", "{record.dossier_nr}" }
                                                        span { " · " }
                                                        span { "{record.source_domain}" }
                                                        span { " · " }
                                                        span { class: "tag version", "{record.provisa_version.name()}" }
                                                    }
                                                }
                                                div { class: record.status.class(), "{record.status.display()}" }
                                            }
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            }
                        }
                    }
                }

                // Detail Panel
                div { style: "flex: 1;",
                    {if let Some(record) = selected_record.read().as_ref() {
                        let record = record.clone();
                        rsx! {
                            Panel { title: "{record.title}",
                                div { class: "detail-grid",
                                    div { class: "detail-row",
                                        span { class: "label", "Dossiernummer:" }
                                        span { "{record.dossier_nr}" }
                                    }
                                    div { class: "detail-row",
                                        span { class: "label", "Bron domein:" }
                                        span { "{record.source_domain}" }
                                    }
                                    div { class: "detail-row",
                                        span { class: "label", "PROVISA versie:" }
                                        span { "{record.provisa_version.name()}" }
                                    }
                                    div { class: "detail-row",
                                        span { class: "label", "Classificatie:" }
                                        span { "{record.classification}" }
                                    }
                                    div { class: "detail-row",
                                        span { class: "label", "Bewaartermijn:" }
                                        span {
                                            if let Some(rp) = record.retention_period {
                                                "{rp} jaar"
                                            } else {
                                                "Niet ingesteld"
                                            }
                                        }
                                    }
                                    div { class: "detail-row",
                                        span { class: "label", "Bewaarstart:" }
                                        span {
                                            {record.retention_start.as_ref().map_or("N/A".to_string(), |d| d.clone())}
                                        }
                                    }
                                    div { class: "detail-row",
                                        span { class: "label", "Vernietigingsdatum:" }
                                        span {
                                            {record.destruction_date.as_ref().map_or("N/A".to_string(), |d| d.clone())}
                                        }
                                    }
                                    div { class: "detail-row",
                                        span { class: "label", "Overbrengdatum:" }
                                        span {
                                            {record.transfer_date.as_ref().map_or("N/A".to_string(), |d| d.clone())}
                                        }
                                    }
                                }

                                div { style: "margin-top: 20px;",
                                    label { "Tags" }
                                    div { style: "display: flex; flex-wrap: wrap; gap: 8px; margin-top: 8px;",
                                        for tag in &record.tags {
                                            span { class: "tag", "{tag}" }
                                        }
                                    }
                                }

                                div { style: "margin-top: 20px; display: flex; gap: 10px;",
                                    button { class: "btn btn-primary", "Bewerken" }
                                    button {
                                        class: "btn btn-secondary",
                                        onclick: move |_| {
                                            let current = *show_version_compare.read();
                                            show_version_compare.set(!current);
                                        },
                                        "Versies vergelijken"
                                    }
                                }
                            }

                            if *show_version_compare.read() {
                                Panel { title: "PROVISA Versievergelijking",
                                    div { class: "version-compare",
                                        div { class: "version-row",
                                            div { "PROVISA 2020" }
                                            div { "Huidige selectie" }
                                        }
                                        div { class: "version-row",
                                            div { "PROVISA 2014" }
                                            div {
                                                if matches!(record.provisa_version, ProvisaVersion::Provisa2020) {
                                                    "Bewaartermijn: 15 jaar (was 20 jaar)"
                                                } else {
                                                    "Bewaartermijn identiek"
                                                }
                                            }
                                        }
                                        div { class: "version-row",
                                            div { "PROVISA 2005" }
                                            div {
                                                if matches!(record.provisa_version, ProvisaVersion::Provisa2020) {
                                                    "Categorie: B (was A)"
                                                } else {
                                                    "Andere categorie"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            Panel { title: "Selecteer een dossier",
                                div { class: "empty-state",
                                    p { "Selecteer een dossier uit de lijst om details te bekijken." }
                                }
                            }
                        }
                    }}
                }
            }

            // Retention Calculator
            div { style: "margin-top: 20px;",
                Panel { title: "Bewaartermijn Calculator".to_string(),
                    div { class: "retention-calculator",
                        div { class: "detail-row",
                            span { class: "label", "Document type:" }
                            select {
                                style: "margin-left: auto;",
                                option { "Besluit" }
                                option { "Vergunning" }
                                option { "Subsidiebeschikking" }
                                option { "Beleidsstuk" }
                                option { "Correspondentie" }
                            }
                        }
                        div { class: "detail-row",
                            span { class: "label", "Startdatum:" }
                            input {
                                r#type: "date",
                                style: "margin-left: auto;",
                            }
                        }
                        div { class: "detail-row",
                            span { class: "label", "PROVISA versie:" }
                            select {
                                style: "margin-left: auto;",
                                option { "PROVISA 2020" }
                                option { "PROVISA 2014" }
                                option { "PROVISA 2005" }
                            }
                        }
                        div { style: "margin-top: 15px; padding: 15px; background: #f0f7ff; border-radius: 8px;",
                            div { class: "detail-row",
                                span { class: "label", "Aanbevolen bewaartermijn:" }
                                span { style: "font-weight: bold; color: #5B3CC4;", "20 jaar" }
                            }
                            div { class: "detail-row",
                                span { class: "label", "Vernietigingsdatum:" }
                                span { style: "font-weight: bold;", "2045-01-01" }
                            }
                        }
                    }
                }
            }
        }
    }
}
