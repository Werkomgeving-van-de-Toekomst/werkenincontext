//! Data Verkenner app

use leptos::prelude::*;

use crate::components::{Header, Panel};

#[component]
pub fn DataVerkenner() -> impl IntoView {
    view! {
        <Header/>
        <main class="container">
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
                <Panel title="Datasets">
                    <select style="width: 100%; padding: 10px; margin-bottom: 15px;">
                        <option>"Verkeersintensiteit Flevoland"</option>
                        <option>"Archeologische vindplaatsen"</option>
                        <option>"CBS Bevolkingsstatistiek"</option>
                        <option>"Energieverbruik gemeenten"</option>
                        <option>"Natuurnetwerk Nederland"</option>
                    </select>

                    <div class="compliance-indicator ok">
                        <div class="icon">"📊"</div>
                        <div class="label">"Totaal records"</div>
                        <div class="value">"12.847"</div>
                    </div>
                    <div class="compliance-indicator ok">
                        <div class="icon">"📅"</div>
                        <div class="label">"Laatste update"</div>
                        <div class="value">"2025-01-15"</div>
                    </div>
                    <div class="compliance-indicator ok">
                        <div class="icon">"🏢"</div>
                        <div class="label">"Bron"</div>
                        <div class="value">"CBS"</div>
                    </div>
                </Panel>

                <Panel title="Visualisatie">
                    <div class="chart-container" style="background: #f5f7fa; border-radius: 8px; display: flex; align-items: center; justify-content: center;">
                        <p style="color: #666;">"Chart wordt geladen..."</p>
                    </div>
                </Panel>
            </div>

            <div style="height: 20px;"></div>

            <Panel title="Kaart">
                <div id="map" style="height: 400px; background: #e0e0e0; border-radius: 8px; display: flex; align-items: center; justify-content: center;">
                    <p style="color: #666;">"Kaart wordt geladen..."</p>
                </div>
            </Panel>
        </main>
    }
}
