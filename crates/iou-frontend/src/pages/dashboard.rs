//! Main dashboard page

use leptos::prelude::*;

use crate::components::{Header, Panel, AppCard};
use crate::state::AppState;

#[component]
pub fn Dashboard() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState must be provided");

    // Demo apps data
    let apps = vec![
        ("Data Verkenner", "Verken provinciale datasets", "/apps/data-verkenner", Some("Populair")),
        ("Document Generator", "Genereer compliant documenten", "/apps/document-generator", Some("Nieuw")),
        ("Nalevingscontrole", "Monitor Woo/AVG compliance", "/apps/nalevingscontrole", None),
        ("Tijdlijn Weergave", "Bekijk activiteiten tijdlijn", "/apps/tijdlijn-weergave", None),
        ("GraphRAG Explorer", "Ontdek relaties via kennisgraaf", "/graphrag", Some("AI")),
        ("Samenwerkingscentrum", "Werk samen met anderen", "/apps/samenwerkingscentrum", None),
    ];

    view! {
        <Header/>

        <main class="container">
            // Context Bar
            <div class="context-bar">
                <div class="breadcrumb">
                    <span>"Provincie Flevoland"</span>
                    <span>" › "</span>
                    <span class="current">"Duurzaamheid & Energie"</span>
                </div>

                <select>
                    <option>"Duurzaamheid & Energie"</option>
                    <option>"Windpark Almere"</option>
                    <option>"Omgevingsvergunning Bouw"</option>
                    <option>"Omgevingsvisie 2030"</option>
                </select>

                <div class="search-input">
                    <input
                        type="text"
                        placeholder="Zoeken in context..."
                        on:input=move |ev| {
                            state.search_query.set(event_target_value(&ev));
                        }
                    />
                    <button class="btn btn-primary">"Zoeken"</button>
                </div>
            </div>

            // Dashboard Grid
            <div class="dashboard-grid">
                // Left Column - Apps
                <div>
                    <Panel title="Context Apps">
                        <div class="app-grid">
                            {apps.into_iter().map(|(name, desc, url, badge)| {
                                match badge {
                                    Some(b) => view! {
                                        <AppCard
                                            name=name.to_string()
                                            description=desc.to_string()
                                            endpoint=url.to_string()
                                            badge=b.to_string()
                                        />
                                    }.into_any(),
                                    None => view! {
                                        <AppCard
                                            name=name.to_string()
                                            description=desc.to_string()
                                            endpoint=url.to_string()
                                        />
                                    }.into_any(),
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Panel>

                    <div style="height: 20px;"></div>

                    <Panel title="Compliance Status">
                        <div class="compliance-indicator ok">
                            <div class="icon">"✓"</div>
                            <div class="label">"Woo Compliance"</div>
                            <div class="value">"98%"</div>
                        </div>
                        <div class="compliance-indicator ok">
                            <div class="icon">"✓"</div>
                            <div class="label">"AVG Compliance"</div>
                            <div class="value">"100%"</div>
                        </div>
                        <div class="compliance-indicator warning">
                            <div class="icon">"!"</div>
                            <div class="label">"Bewaartermijnen"</div>
                            <div class="value">"3 acties"</div>
                        </div>
                    </Panel>
                </div>

                // Center Column - Documents
                <div>
                    <Panel title="Recente Documenten">
                        <ul class="document-list">
                            <li class="document-item">
                                <div class="document-icon">"📄"</div>
                                <div class="document-info">
                                    <h4>"Besluit subsidieverlening windpark"</h4>
                                    <div class="meta">"Besluit • 2 dagen geleden"</div>
                                </div>
                                <span class="tag woo">"Woo"</span>
                            </li>
                            <li class="document-item">
                                <div class="document-icon">"📧"</div>
                                <div class="document-info">
                                    <h4>"Re: Voortgang projectplan duurzaamheid"</h4>
                                    <div class="meta">"Email • 3 dagen geleden"</div>
                                </div>
                            </li>
                            <li class="document-item">
                                <div class="document-icon">"📄"</div>
                                <div class="document-info">
                                    <h4>"Advies Omgevingsdienst Flevoland"</h4>
                                    <div class="meta">"Document • 1 week geleden"</div>
                                </div>
                                <span class="tag">"advies"</span>
                            </li>
                            <li class="document-item">
                                <div class="document-icon">"📊"</div>
                                <div class="document-info">
                                    <h4>"Dataset energieverbruik gemeenten"</h4>
                                    <div class="meta">"Data • 2 weken geleden"</div>
                                </div>
                                <span class="tag">"CBS"</span>
                            </li>
                        </ul>
                    </Panel>

                    <div style="height: 20px;"></div>

                    <Panel title="Gerelateerde Domeinen">
                        <div style="display: flex; flex-wrap: wrap; gap: 10px;">
                            <div class="tag">"Windpark Almere"</div>
                            <div class="tag">"Omgevingsvisie 2030"</div>
                            <div class="tag">"Subsidieregeling Energie"</div>
                            <div class="tag">"Klimaatadaptatie"</div>
                        </div>
                    </Panel>
                </div>

                // Right Column - Stakeholders & AI
                <div>
                    <Panel title="Stakeholders">
                        <ul class="document-list">
                            <li class="document-item">
                                <div class="document-icon" style="background: #7CB342;">"👤"</div>
                                <div class="document-info">
                                    <h4>"Gemeente Almere"</h4>
                                    <div class="meta">"Mede-initiatiefnemer"</div>
                                </div>
                            </li>
                            <li class="document-item">
                                <div class="document-icon" style="background: #7CB342;">"👤"</div>
                                <div class="document-info">
                                    <h4>"Omgevingsdienst Flevoland"</h4>
                                    <div class="meta">"Adviseur"</div>
                                </div>
                            </li>
                            <li class="document-item">
                                <div class="document-icon" style="background: #7CB342;">"👤"</div>
                                <div class="document-info">
                                    <h4>"Vattenfall NL"</h4>
                                    <div class="meta">"Aanvrager"</div>
                                </div>
                            </li>
                        </ul>
                    </Panel>

                    <div style="height: 20px;"></div>

                    <Panel title="AI Suggesties">
                        <div class="compliance-indicator ok">
                            <div class="icon">"🤖"</div>
                            <div class="label">"3 nieuwe metadata suggesties"</div>
                        </div>
                        <p style="font-size: 0.875rem; color: #666; margin-top: 10px;">
                            "AI heeft automatisch tags en classificaties voorgesteld voor 3 nieuwe documenten."
                        </p>
                        <button class="btn btn-secondary" style="margin-top: 10px; width: 100%;">
                            "Bekijk suggesties"
                        </button>
                    </Panel>
                </div>
            </div>
        </main>
    }
}
