//! Panel component - reusable card with header

use dioxus::prelude::*;

#[component]
pub fn Panel(title: String, children: Element) -> Element {
    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                h2 { "{title}" }
            }
            div { class: "panel-content",
                {children}
            }
        }
    }
}
