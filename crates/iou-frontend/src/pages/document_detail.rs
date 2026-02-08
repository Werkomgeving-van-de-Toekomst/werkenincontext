//! Document detail page

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn DocumentDetail(id: String) -> Element {
    rsx! {
        Header {}
        main { class: "container",
            Panel { title: "Document Details".to_string(),
                p { "Document ID: {id}" }
                p { "Implementatie in ontwikkeling..." }
            }
        }
    }
}
