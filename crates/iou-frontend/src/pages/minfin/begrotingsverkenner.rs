//! Begrotingsverkenner app - Rijksbegroting data explorer

use dioxus::prelude::*;

use crate::components::{Header, Panel};

/// Mock budget dataset metadata
struct BegrotingsDataset {
    name: &'static str,
    records: &'static str,
    last_update: &'static str,
    source: &'static str,
    bars: &'static [(&'static str, u32)],
}

const DATASETS: &[BegrotingsDataset] = &[
    BegrotingsDataset {
        name: "Rijksbegroting per ministerie (uitgaven)",
        records: "142.300",
        last_update: "2026-01-15",
        source: "Ministerie van Financi\u{00eb}n",
        bars: &[
            ("SZW", 95),
            ("VWS", 82),
            ("OCW", 45),
            ("Defensie", 38),
            ("I&W", 32),
        ],
    },
    BegrotingsDataset {
        name: "Belastingopbrengsten per soort",
        records: "89.450",
        last_update: "2026-01-20",
        source: "Belastingdienst",
        bars: &[
            ("Loon/IB", 90),
            ("BTW", 75),
            ("Vpb", 42),
            ("Accijnzen", 28),
            ("Divid.bel.", 15),
        ],
    },
    BegrotingsDataset {
        name: "Staatsschuld ontwikkeling",
        records: "24.100",
        last_update: "2025-12-31",
        source: "DSTA",
        bars: &[
            ("2022", 48),
            ("2023", 50),
            ("2024", 52),
            ("2025", 49),
            ("2026 (r)", 47),
        ],
    },
    BegrotingsDataset {
        name: "Uitgaven Sociale Zekerheid",
        records: "67.800",
        last_update: "2026-01-10",
        source: "SZW / UWV",
        bars: &[
            ("AOW", 90),
            ("WW", 35),
            ("Bijstand", 42),
            ("WIA/WAO", 55),
            ("Kinderbijsl.", 25),
        ],
    },
    BegrotingsDataset {
        name: "Zorguitgaven per sector",
        records: "53.200",
        last_update: "2026-01-08",
        source: "VWS / NZa",
        bars: &[
            ("Zvw", 85),
            ("Wlz", 72),
            ("Jeugdwet", 30),
            ("Wmo", 28),
            ("Prev.", 12),
        ],
    },
];

#[component]
pub fn MinFinBegrotingsverkenner() -> Element {
    let mut selected = use_signal(|| 0usize);

    let idx = *selected.read();
    let dataset = &DATASETS[idx];

    rsx! {
        div { class: "minfin",
            Header {}
            main { class: "container",
                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                    Panel { title: "Datasets".to_string(),
                        select {
                            style: "width: 100%; padding: 10px; margin-bottom: 15px;",
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
                            div { class: "label", "Totaal records" }
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
                                            class: "bar-fill minfin-bar",
                                            style: "width: {value}%;",
                                        }
                                    }
                                    span { class: "bar-value", "{value}%" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
