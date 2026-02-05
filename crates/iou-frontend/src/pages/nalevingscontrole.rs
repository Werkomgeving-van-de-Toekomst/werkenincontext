//! Nalevingscontrole (Compliance monitoring) app

use leptos::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn Nalevingscontrole() -> impl IntoView {
    view! {
        <Header/>
        <main class="container">
            <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-bottom: 20px;">
                <Panel title="Woo Compliance">
                    <div style="text-align: center; padding: 20px;">
                        <div style="font-size: 3rem; font-weight: bold; color: #4CAF50;">"98%"</div>
                        <p style="color: #666;">"2 documenten vereisen actie"</p>
                    </div>
                    <div class="compliance-indicator ok">
                        <div class="icon">"✓"</div>
                        <div class="label">"Openbaar geclassificeerd"</div>
                        <div class="value">"156"</div>
                    </div>
                    <div class="compliance-indicator warning">
                        <div class="icon">"!"</div>
                        <div class="label">"Wacht op beoordeling"</div>
                        <div class="value">"2"</div>
                    </div>
                </Panel>

                <Panel title="AVG Compliance">
                    <div style="text-align: center; padding: 20px;">
                        <div style="font-size: 3rem; font-weight: bold; color: #4CAF50;">"100%"</div>
                        <p style="color: #666;">"Alle documenten conform"</p>
                    </div>
                    <div class="compliance-indicator ok">
                        <div class="icon">"✓"</div>
                        <div class="label">"Geen persoonsgegevens"</div>
                        <div class="value">"142"</div>
                    </div>
                    <div class="compliance-indicator ok">
                        <div class="icon">"✓"</div>
                        <div class="label">"Geanonimiseerd"</div>
                        <div class="value">"16"</div>
                    </div>
                </Panel>

                <Panel title="Archiefwet">
                    <div style="text-align: center; padding: 20px;">
                        <div style="font-size: 3rem; font-weight: bold; color: #FF9800;">"94%"</div>
                        <p style="color: #666;">"3 termijnen overschreden"</p>
                    </div>
                    <div class="compliance-indicator ok">
                        <div class="icon">"✓"</div>
                        <div class="label">"Bewaartermijn ingesteld"</div>
                        <div class="value">"151"</div>
                    </div>
                    <div class="compliance-indicator error">
                        <div class="icon">"✗"</div>
                        <div class="label">"Vernietigen vereist"</div>
                        <div class="value">"3"</div>
                    </div>
                </Panel>
            </div>

            <Panel title="Documenten met acties vereist">
                <ul class="document-list">
                    <li class="document-item">
                        <div class="document-icon" style="background: #FF9800;">"!"</div>
                        <div class="document-info">
                            <h4>"Concept advies windpark"</h4>
                            <div class="meta">"Woo classificatie vereist"</div>
                        </div>
                        <button class="btn btn-primary">"Beoordelen"</button>
                    </li>
                    <li class="document-item">
                        <div class="document-icon" style="background: #FF9800;">"!"</div>
                        <div class="document-info">
                            <h4>"Email correspondentie project"</h4>
                            <div class="meta">"Woo classificatie vereist"</div>
                        </div>
                        <button class="btn btn-primary">"Beoordelen"</button>
                    </li>
                    <li class="document-item">
                        <div class="document-icon" style="background: #F44336;">"✗"</div>
                        <div class="document-info">
                            <h4>"Oude projectdocumentatie 2017"</h4>
                            <div class="meta">"Bewaartermijn verstreken - vernietigen"</div>
                        </div>
                        <button class="btn btn-secondary">"Vernietigen"</button>
                    </li>
                </ul>
            </Panel>
        </main>
    }
}
