//! Loading spinner component

use dioxus::prelude::*;

#[component]
pub fn Loading() -> Element {
    rsx! {
        div { class: "loading",
            div { class: "spinner" }
        }
    }
}
