//! Search results page

use dioxus::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn SearchResults() -> Element {
    rsx! {
        Header {}
        main { class: "container",
            Panel { title: "Zoekresultaten".to_string(),
                p { "Implementatie in ontwikkeling..." }
            }
        }
    }
}
