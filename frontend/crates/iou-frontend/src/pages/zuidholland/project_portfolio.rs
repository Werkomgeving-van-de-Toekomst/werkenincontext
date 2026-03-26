//! Project Portfolio - Infra-projecten beheer Provincie Zuid-Holland

use dioxus::prelude::*;

use crate::components::{Header, Panel};

struct InfraProject {
    name: &'static str,
    status: &'static str,
    status_class: &'static str,
    budget: &'static str,
    progress: u32,
    deadline: &'static str,
    category: &'static str,
}

const PROJECTS: &[InfraProject] = &[
    InfraProject {
        name: "A16 Rotterdam",
        status: "In uitvoering",
        status_class: "ok",
        budget: "\u{20AC} 1,24 mrd",
        progress: 72,
        deadline: "2028-Q4",
        category: "Wegen",
    },
    InfraProject {
        name: "Rijnlandroute (N434)",
        status: "In uitvoering",
        status_class: "ok",
        budget: "\u{20AC} 920 mln",
        progress: 88,
        deadline: "2027-Q2",
        category: "Wegen",
    },
    InfraProject {
        name: "Blankenburgverbinding (A24)",
        status: "In uitvoering",
        status_class: "ok",
        budget: "\u{20AC} 1,08 mrd",
        progress: 65,
        deadline: "2029-Q1",
        category: "Wegen",
    },
    InfraProject {
        name: "HOV Zuidplaspolder",
        status: "Voorbereiding",
        status_class: "warning",
        budget: "\u{20AC} 185 mln",
        progress: 25,
        deadline: "2030-Q3",
        category: "OV",
    },
    InfraProject {
        name: "Warmtelinq (Havenwarmte)",
        status: "In uitvoering",
        status_class: "ok",
        budget: "\u{20AC} 350 mln",
        progress: 55,
        deadline: "2027-Q4",
        category: "Energie",
    },
    InfraProject {
        name: "Kustversterking Delfland",
        status: "In uitvoering",
        status_class: "ok",
        budget: "\u{20AC} 210 mln",
        progress: 42,
        deadline: "2028-Q2",
        category: "Water",
    },
    InfraProject {
        name: "Programma Groenblauwe Scheggen",
        status: "Planfase",
        status_class: "warning",
        budget: "\u{20AC} 78 mln",
        progress: 15,
        deadline: "2031-Q4",
        category: "Groen",
    },
    InfraProject {
        name: "Smart Mobility Corridor A13",
        status: "Pilot",
        status_class: "ok",
        budget: "\u{20AC} 42 mln",
        progress: 35,
        deadline: "2027-Q1",
        category: "Innovatie",
    },
];

#[component]
pub fn ZHProjectPortfolio() -> Element {
    let total_budget: f64 = 4_105.0; // mln
    let avg_progress: u32 = PROJECTS.iter().map(|p| p.progress).sum::<u32>() / PROJECTS.len() as u32;
    let in_progress = PROJECTS.iter().filter(|p| p.status == "In uitvoering").count();

    rsx! {
        div { class: "zuidholland",
            Header {}
            main { class: "container",
                div { class: "context-bar",
                    div { class: "breadcrumb",
                        span { "Zuid-Holland" }
                        span { " \u{203A} " }
                        span { class: "current", "Projecten Portfoliomanagement" }
                    }
                }

                // KPI rij
                div { style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 20px;",
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 16px;",
                            div { style: "font-size: 1.75rem; font-weight: 700; color: #E31837;", "{PROJECTS.len()}" }
                            div { style: "font-size: 0.875rem; color: #666;", "Projecten" }
                        }
                    }
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 16px;",
                            div { style: "font-size: 1.75rem; font-weight: 700; color: #E31837;", "\u{20AC} {total_budget:.0}M" }
                            div { style: "font-size: 0.875rem; color: #666;", "Totaal budget" }
                        }
                    }
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 16px;",
                            div { style: "font-size: 1.75rem; font-weight: 700; color: #E31837;", "{in_progress}" }
                            div { style: "font-size: 0.875rem; color: #666;", "In uitvoering" }
                        }
                    }
                    div { class: "panel",
                        div { class: "panel-content", style: "text-align: center; padding: 16px;",
                            div { style: "font-size: 1.75rem; font-weight: 700; color: #E31837;", "{avg_progress}%" }
                            div { style: "font-size: 0.875rem; color: #666;", "Gem. voortgang" }
                        }
                    }
                }

                Panel { title: "Projectenportfolio".to_string(),
                    // Tabel-header
                    div { style: "display: grid; grid-template-columns: 2fr 1fr 1fr 2fr 1fr 1fr; gap: 8px; padding: 12px 16px; background: #f5f7fa; font-size: 0.75rem; font-weight: 600; color: #666; text-transform: uppercase; letter-spacing: 0.5px;",
                        span { "Project" }
                        span { "Categorie" }
                        span { "Status" }
                        span { "Voortgang" }
                        span { "Budget" }
                        span { "Deadline" }
                    }
                    for project in PROJECTS.iter() {
                        div { style: "display: grid; grid-template-columns: 2fr 1fr 1fr 2fr 1fr 1fr; gap: 8px; padding: 12px 16px; border-bottom: 1px solid #e0e0e0; align-items: center; font-size: 0.875rem;",
                            span { style: "font-weight: 500;", "{project.name}" }
                            span { class: "tag", "{project.category}" }
                            div { class: "compliance-indicator {project.status_class}", style: "margin: 0; padding: 4px 8px;",
                                div { class: "label", style: "font-size: 0.75rem;", "{project.status}" }
                            }
                            div { class: "bar-row", style: "margin: 0;",
                                div { class: "bar-track", style: "height: 14px;",
                                    div {
                                        class: "bar-fill",
                                        style: "width: {project.progress}%; height: 14px;",
                                    }
                                }
                                span { class: "bar-value", style: "font-size: 0.75rem;", "{project.progress}%" }
                            }
                            span { style: "color: #666;", "{project.budget}" }
                            span { style: "color: #666;", "{project.deadline}" }
                        }
                    }
                }

                div { style: "height: 20px;" }

                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px;",
                    Panel { title: "Budget per categorie".to_string(),
                        div { class: "chart-container", style: "display: flex; flex-direction: column; justify-content: center; gap: 8px;",
                            div { class: "bar-row",
                                span { class: "bar-label", "Wegen" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 95%;" }
                                }
                                span { class: "bar-value", "\u{20AC}3,2mrd" }
                            }
                            div { class: "bar-row",
                                span { class: "bar-label", "Energie" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 25%;" }
                                }
                                span { class: "bar-value", "\u{20AC}350M" }
                            }
                            div { class: "bar-row",
                                span { class: "bar-label", "Water" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 18%;" }
                                }
                                span { class: "bar-value", "\u{20AC}210M" }
                            }
                            div { class: "bar-row",
                                span { class: "bar-label", "OV" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 13%;" }
                                }
                                span { class: "bar-value", "\u{20AC}185M" }
                            }
                            div { class: "bar-row",
                                span { class: "bar-label", "Overig" }
                                div { class: "bar-track",
                                    div { class: "bar-fill", style: "width: 10%;" }
                                }
                                span { class: "bar-value", "\u{20AC}120M" }
                            }
                        }
                    }

                    Panel { title: "Risico's & aandachtspunten".to_string(),
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "!" }
                            div { class: "label", "A16: Stikstof-vergunning vertraging" }
                        }
                        div { class: "compliance-indicator warning",
                            div { class: "icon", "!" }
                            div { class: "label", "HOV Zuidplaspolder: grondverwerving" }
                        }
                        div { class: "compliance-indicator error",
                            div { class: "icon", "!" }
                            div { class: "label", "Warmtelinq: leveringsketen vertraging" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Rijnlandroute: op schema" }
                        }
                        div { class: "compliance-indicator ok",
                            div { class: "icon", "\u{2713}" }
                            div { class: "label", "Kustversterking: binnen budget" }
                        }
                    }
                }
            }
        }
    }
}
