//! Mobiliteitsverkenner - Provinciale mobiliteitsdata explorer

use dioxus::prelude::*;

use crate::components::{Header, Panel};

struct MobiliteitsDataset {
    name: &'static str,
    records: &'static str,
    last_update: &'static str,
    source: &'static str,
    bars: &'static [(&'static str, u32)],
}

const DATASETS: &[MobiliteitsDataset] = &[
    MobiliteitsDataset {
        name: "Verkeersintensiteit hoofdwegennet ZH",
        records: "2.340.000",
        last_update: "2026-02-01",
        source: "RWS / NDW",
        bars: &[
            ("A4", 92),
            ("A13", 88),
            ("A16", 76),
            ("A15", 71),
            ("A20", 65),
        ],
    },
    MobiliteitsDataset {
        name: "OV-reizigers per corridor",
        records: "890.000",
        last_update: "2026-01-28",
        source: "HTM / RET / NS",
        bars: &[
            ("R'dam-DH", 95),
            ("Leiden-DH", 72),
            ("R'dam-Dor", 58),
            ("Zoetermeer", 45),
            ("Hoekse Lijn", 32),
        ],
    },
    MobiliteitsDataset {
        name: "Fietsgebruik provinciale routes",
        records: "456.000",
        last_update: "2026-01-20",
        source: "Provincie ZH / CROW",
        bars: &[
            ("Westland", 85),
            ("Midden-DL", 78),
            ("Bollenstreek", 65),
            ("Drechtsteden", 52),
            ("Voorne-Putten", 38),
        ],
    },
    MobiliteitsDataset {
        name: "Scheepvaartbewegingen Nieuwe Waterweg",
        records: "124.500",
        last_update: "2026-02-05",
        source: "Havenbedrijf / RWS",
        bars: &[
            ("Containers", 90),
            ("Bulk droog", 72),
            ("Bulk nat", 68),
            ("RoRo", 45),
            ("Overig", 25),
        ],
    },
    MobiliteitsDataset {
        name: "Luchtkwaliteit langs snelwegen",
        records: "78.200",
        last_update: "2026-01-15",
        source: "RIVM / GGD ZH",
        bars: &[
            ("NO\u{2082} A13", 78),
            ("NO\u{2082} A4", 65),
            ("PM10 A15", 55),
            ("PM2.5 R'dam", 48),
            ("CO\u{2082} ZH-gem", 42),
        ],
    },
];

#[component]
pub fn ZHMobiliteitsverkenner() -> Element {
    let mut selected = use_signal(|| 0usize);

    let idx = *selected.read();
    let dataset = &DATASETS[idx];

    rsx! {
        div { class: "zuidholland",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Zuid-Holland" }
                        span { " \u{203A} " }
                        span { class: "current", "Mobiliteitsverkenner" }
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                    Panel { title: "Mobiliteitsdata".to_string(),
                        select {
                            style: "width: 100%; padding: 10px; margin-bottom: 15px; border: 1px solid #e0e0e0; border-radius: 4px;",
                            onchange: move |evt: Event<FormData>| {
                                if let Ok(i) = evt.value().parse::<usize>() {
                                    selected.set(i);
                                }
                            },
                            for (i, ds) in DATASETS.iter().enumerate() {
                                option {
                                    value: "{i}",
                                    selected: i == idx,
                                    "{ds.name}"
                                }
                            }
                        }

                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F4CA}" }
                            div { class: "label", "Totaal meetpunten" }
                            div { class: "value", "{dataset.records}" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F4C5}" }
                            div { class: "label", "Laatste update" }
                            div { class: "value", "{dataset.last_update}" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{1F3E2}" }
                            div { class: "label", "Bron" }
                            div { class: "value", "{dataset.source}" }
                        }
                    }

                    Panel { title: "Top-5 Verdeling".to_string(),
                        div { class: "chart-container", style: "display: flex; flex-direction: column; justify-content: center; gap: 8px;",
                            for &(label, value) in dataset.bars {
                                div { class: "bar-row",
                                    span { class: "bar-label", "{label}" }
                                    div { class: "bar-track",
                                        div {
                                            class: "bar-fill",
                                            style: "width: {value}%;",
                                        }
                                    }
                                    span { class: "bar-value", "{value}%" }
                                }
                            }
                        }
                    }
                }

                div { style: "height: 20px;" }

                div { style: "display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 20px;",
                    Panel { title: "Knelpunten".to_string(),
                        div { class: "compliance-indicator error",
                            div { class: "icon", "!" }
                            div { class: "label", "A16 Rotterdam - structureel" }
                        }
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "!" }
                            div { class: "label", "A4 Leidschendam - spits" }
                        }
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "!" }
                            div { class: "label", "A13/A16 knooppunt" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "A15 Maasvlakte - verbeterd" }
                        }
                    }

                    Panel { title: "Trends".to_string(),
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2191}" }
                            div { class: "label", "OV-gebruik" }
                            div { class: "value", "+8%" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2191}" }
                            div { class: "label", "Fietsgebruik" }
                            div { class: "value", "+12%" }
                        }
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "\u{2191}" }
                            div { class: "label", "Autoverkeer" }
                            div { class: "value", "+3%" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2193}" }
                            div { class: "label", "CO\u{2082} transport" }
                            div { class: "value", "-5%" }
                        }
                    }

                    Panel { title: "Databronnen".to_string(),
                        ul { class: "document-list",
                            li { class: "document-item",
                                div { class: "document-icon", "\u{1F4CA}" }
                                div { class: "document-info",
                                    h4 { "NDW Open Data" }
                                    div { class: "meta", "Realtime verkeersdata" }
                                }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", "\u{1F683}" }
                                div { class: "document-info",
                                    h4 { "OV-chipkaart data" }
                                    div { class: "meta", "Geanonimiseerd" }
                                }
                            }
                            li { class: "document-item",
                                div { class: "document-icon", "\u{1F6A2}" }
                                div { class: "document-info",
                                    h4 { "AIS Scheepvaart" }
                                    div { class: "meta", "Havenbedrijf Rotterdam" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
