//! Context detail page

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn ContextDetail(id: String) -> Element {
    rsx! {
        Header {}
        main { class: "container",
            Panel { title: "Context Details".to_string(),
                p { "Context ID: {id}" }
                p { "Implementatie in ontwikkeling..." }
            }
        }
    }
}
