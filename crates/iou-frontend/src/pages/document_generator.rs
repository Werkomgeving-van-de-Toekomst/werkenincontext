//! Document Generator app

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn DocumentGenerator() -> Element {
    let mut step = use_signal(|| 1i32);

    rsx! {
        Header {}
        main { class: "container",
            Panel { title: "Document Generator".to_string(),
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
                                    h3 { "Adviesbrief" }
                                    p { "Formeel advies aan college of raad" }
                                }
                                div { class: "app-card", onclick: move |_| step.set(2),
                                    h3 { "Besluit" }
                                    p { "Officieel besluit op aanvraag" }
                                }
                                div { class: "app-card", onclick: move |_| step.set(2),
                                    h3 { "Raadsvoorstel" }
                                    p { "Voorstel voor provinciale staten" }
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
                                div { class: "label", "Onderwerp: Windenergie" }
                                div { class: "value", "95%" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{1F916}" }
                                div { class: "label", "Tags: subsidie, duurzaam, energie" }
                                div { class: "value", "88%" }
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
                                div { class: "label", "Woo classificatie" }
                                div { class: "value", "Openbaar" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "Bewaartermijn" }
                                div { class: "value", "20 jaar" }
                            }
                            div { class: "compliance-indicator ok",
                                div { class: "icon", "\u{2713}" }
                                div { class: "label", "AVG check" }
                                div { class: "value", "Geen PII" }
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
