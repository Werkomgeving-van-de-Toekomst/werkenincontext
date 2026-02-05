//! GraphRAG Explorer page

use leptos::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn GraphRAGExplorer() -> impl IntoView {
    view! {
        <Header/>
        <main class="container">
            <Panel title="GraphRAG Explorer">
                <p>"Ontdek relaties tussen domeinen via de kennisgraaf."</p>
                <div id="network-graph" style="height: 500px; background: #f5f7fa; border-radius: 8px; display: flex; align-items: center; justify-content: center;">
                    <p style="color: #666;">"Network visualisatie wordt geladen..."</p>
                </div>
            </Panel>
        </main>
    }
}
