//! Beleidsdocument Generator app - Kamerbrieven & begrotingsnota's

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn MinFinBeleidsdocumentGenerator() -> Element {
    let mut step = use_signal(|| 1i32);

    rsx! {
        div { class: "minfin",
            Header {}
            main { class: "container",
                Panel { title: "Beleidsdocument Generator".to_string(),
                    // Wizard steps
                    div { style: "display: flex; gap: 10px; margin-bottom: 20px;",
                        div { class: if step() >= 1 { "tag woo" } else { "tag" }, "1. Type" }
                        div { class: if step() >= 2 { "tag woo" } else { "tag" }, "2. Content" }
                        div { class: if step() >= 3 { "tag woo" } else { "tag" }, "3. Metadata" }
                        div { class: if step() >= 4 { "tag woo" } else { "tag" }, "4. Compliance" }
                        div { class: if step() >= 5 { "tag woo" } else { "tag" }, "5. Preview" }
                    }

                    {match step() {
                        1 => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Selecteer documenttype" }
                                div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px;",
                                    div { class: "app-card", onclick: move |_| step.set(2),
                                        h3 { "Kamerbrief" }
                                        p { "Brief aan de Tweede Kamer" }
                                    }
                                    div { class: "app-card", onclick: move |_| step.set(2),
                                        h3 { "Begrotingsnota" }
                                        p { "Miljoenennota / Voorjaarsnota" }
                                    }
                                    div { class: "app-card", onclick: move |_| step.set(2),
                                        h3 { "Beleidsdoorlichting" }
                                        p { "Evaluatie van begrotingsartikel" }
                                    }
                                }
                            }
                        },
                        2 => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Document inhoud" }
                                textarea {
                                    style: "width: 100%; height: 200px; padding: 10px; border: 1px solid #ddd; border-radius: 8px;",
                                    placeholder: "Voer de documentinhoud in...",
                                }
                                div { style: "display: flex; justify-content: space-between; margin-top: 15px;",
                                    button { class: "btn btn-secondary", onclick: move |_| step.set(1), "Vorige" }
                                    button { class: "btn btn-primary", onclick: move |_| step.set(3), "Volgende" }
                                }
                            }
                        },
                        3 => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Metadata (AI suggesties)" }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Onderwerp: Belastingplan" }
                                    div { class: "value", "96%" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{1F916}" }
                                    div { class: "label", "Tags: fiscaal, begroting, beleid" }
                                    div { class: "value", "91%" }
                                }
                                div { style: "display: flex; justify-content: space-between; margin-top: 15px;",
                                    button { class: "btn btn-secondary", onclick: move |_| step.set(2), "Vorige" }
                                    button { class: "btn btn-primary", onclick: move |_| step.set(4), "Volgende" }
                                }
                            }
                        },
                        4 => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Compliance check" }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{2713}" }
                                    div { class: "label", "Comptabiliteitswet" }
                                    div { class: "value", "Conform" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{2713}" }
                                    div { class: "label", "Begrotingsregels" }
                                    div { class: "value", "Conform" }
                                }
                                div { class: "compliance-indicator ok",
                                    div { class: "icon", "\u{2713}" }
                                    div { class: "label", "Geheimhouding" }
                                    div { class: "value", "Geen restrictie" }
                                }
                                div { style: "display: flex; justify-content: space-between; margin-top: 15px;",
                                    button { class: "btn btn-secondary", onclick: move |_| step.set(3), "Vorige" }
                                    button { class: "btn btn-primary", onclick: move |_| step.set(5), "Volgende" }
                                }
                            }
                        },
                        _ => rsx! {
                            div {
                                h3 { style: "margin-bottom: 15px;", "Preview & Opslaan" }
                                div { style: "background: #f5f7fa; padding: 20px; border-radius: 8px; margin-bottom: 15px;",
                                    p { "Document preview wordt hier getoond..." }
                                }
                                div { style: "display: flex; justify-content: space-between;",
                                    button { class: "btn btn-secondary", onclick: move |_| step.set(4), "Vorige" }
                                    button { class: "btn btn-primary", "Opslaan" }
                                }
                            }
                        },
                    }}
                }
            }
        }
    }
}
