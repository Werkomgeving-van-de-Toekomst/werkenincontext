//! Home page - kies tussen organisatiemodules

use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Home() -> Element {
    rsx! {
        header { class: "header",
            div { style: "display: flex; justify-content: space-between; align-items: center;",
                div {
                    h1 { "IOU-Modern" }
                    p { class: "subtitle", "Informatie Ondersteunde Werkomgeving" }
                }
            }
        }

        main { class: "container",
            div { class: "home-hero",
                h2 { "Kies uw werkomgeving" }
                p { "Selecteer de organisatie waarvoor u wilt werken." }
            }

            div { class: "home-grid home-grid-4",
                Link { to: Route::ConceptDashboard,
                    div { class: "home-card concept",
                        div { class: "home-card-accent" }
                        div { class: "home-card-body",
                            div { class: "home-card-icon", "\u{1F4A1}" }
                            h3 { "Het Concept" }
                            p { "Werken in context met AI-tooling" }
                            div { class: "home-card-stats",
                                span { "4 modules" }
                                span { "\u{2022}" }
                                span { "Context & AI" }
                                span { "\u{2022}" }
                                span { "Architectuur" }
                            }
                        }
                    }
                }

                Link { to: Route::FlevolandDashboard,
                    div { class: "home-card flevoland",
                        div { class: "home-card-accent" }
                        div { class: "home-card-body",
                            div { class: "home-card-icon", "\u{1F3DB}" }
                            h3 { "Provincie Flevoland" }
                            p { "Duurzaamheid, omgevingsbeleid & provinciale datasets" }
                            div { class: "home-card-stats",
                                span { "5 apps" }
                                span { "\u{2022}" }
                                span { "158 documenten" }
                                span { "\u{2022}" }
                                span { "98% compliant" }
                            }
                        }
                    }
                }

                Link { to: Route::MinFinDashboard,
                    div { class: "home-card minfin",
                        div { class: "home-card-accent" }
                        div { class: "home-card-body",
                            div { class: "home-card-icon", "\u{1F3E6}" }
                            h3 { "Ministerie van Financi\u{00EB}n" }
                            p { "Rijksbegroting, fiscaal beleid & financieel beheer" }
                            div { class: "home-card-stats",
                                span { "4 apps" }
                                span { "\u{2022}" }
                                span { "Rijksbegroting 2026" }
                                span { "\u{2022}" }
                                span { "97% compliant" }
                            }
                        }
                    }
                }

                Link { to: Route::ZuidHolland,
                    div { class: "home-card zuidholland",
                        div { class: "home-card-accent" }
                        div { class: "home-card-body",
                            div { class: "home-card-icon", "\u{1F3A2}" }
                            h3 { "Provincie Zuid-Holland" }
                            p { "Mobiliteit, haven & economie" }
                            div { class: "home-card-stats",
                                span { "6 apps" }
                                span { "\u{2022}" }
                                span { "412 documenten" }
                                span { "\u{2022}" }
                                span { "96% compliant" }
                            }
                        }
                    }
                }
            }
        }
    }
}
