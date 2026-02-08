//! Nalevingscontrole (Compliance monitoring) app

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn Nalevingscontrole() -> Element {
    rsx! {
        Header {}
        main { class: "container",
            div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-bottom: 20px;",
                Panel { title: "Woo Compliance".to_string(),
                    div { style: "text-align: center; padding: 20px;",
                        div { style: "font-size: 3rem; font-weight: bold; color: #4CAF50;", "98%" }
                        p { style: "color: #666;", "2 documenten vereisen actie" }
                    }
                    div { class: "compliance-indicator ok",
                        div { class: "icon", "\u{2713}" }
                        div { class: "label", "Openbaar geclassificeerd" }
                        div { class: "value", "156" }
                    }
                    div { class: "compliance-indicator warning",
                        div { class: "icon", "!" }
                        div { class: "label", "Wacht op beoordeling" }
                        div { class: "value", "2" }
                    }
                }

                Panel { title: "AVG Compliance".to_string(),
                    div { style: "text-align: center; padding: 20px;",
                        div { style: "font-size: 3rem; font-weight: bold; color: #4CAF50;", "100%" }
                        p { style: "color: #666;", "Alle documenten conform" }
                    }
                    div { class: "compliance-indicator ok",
                        div { class: "icon", "\u{2713}" }
                        div { class: "label", "Geen persoonsgegevens" }
                        div { class: "value", "142" }
                    }
                    div { class: "compliance-indicator ok",
                        div { class: "icon", "\u{2713}" }
                        div { class: "label", "Geanonimiseerd" }
                        div { class: "value", "16" }
                    }
                }

                Panel { title: "Archiefwet".to_string(),
                    div { style: "text-align: center; padding: 20px;",
                        div { style: "font-size: 3rem; font-weight: bold; color: #FF9800;", "94%" }
                        p { style: "color: #666;", "3 termijnen overschreden" }
                    }
                    div { class: "compliance-indicator ok",
                        div { class: "icon", "\u{2713}" }
                        div { class: "label", "Bewaartermijn ingesteld" }
                        div { class: "value", "151" }
                    }
                    div { class: "compliance-indicator error",
                        div { class: "icon", "\u{2717}" }
                        div { class: "label", "Vernietigen vereist" }
                        div { class: "value", "3" }
                    }
                }
            }

            Panel { title: "Documenten met acties vereist".to_string(),
                ul { class: "document-list",
                    li { class: "document-item",
                        div { class: "document-icon", style: "background: #FF9800;", "!" }
                        div { class: "document-info",
                            h4 { "Concept advies windpark" }
                            div { class: "meta", "Woo classificatie vereist" }
                        }
                        button { class: "btn btn-primary", "Beoordelen" }
                    }
                    li { class: "document-item",
                        div { class: "document-icon", style: "background: #FF9800;", "!" }
                        div { class: "document-info",
                            h4 { "Email correspondentie project" }
                            div { class: "meta", "Woo classificatie vereist" }
                        }
                        button { class: "btn btn-primary", "Beoordelen" }
                    }
                    li { class: "document-item",
                        div { class: "document-icon", style: "background: #F44336;", "\u{2717}" }
                        div { class: "document-info",
                            h4 { "Oude projectdocumentatie 2017" }
                            div { class: "meta", "Bewaartermijn verstreken - vernietigen" }
                        }
                        button { class: "btn btn-secondary", "Vernietigen" }
                    }
                }
            }
        }
    }
}
