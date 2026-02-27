//! IOU-Modern Frontend - Dioxus WebAssembly Application
//!
//! Context-driven information management dashboard voor Nederlandse overheid.

use dioxus::prelude::*;

mod api;
mod components;
mod pages;
mod state;

use pages::*;

const STYLE: &str = include_str!("../assets/style.css");

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home,
    #[route("/flevoland")]
    FlevolandDashboard,
    #[route("/flevoland/provisa")]
    FlevolandProvisa,
    #[route("/flevoland/hotspots")]
    FlevolandHotspots,
    #[route("/flevoland/archief")]
    FlevolandArchief,
    #[route("/flevoland/architectuur")]
    FlevolandArchitectuur,
    #[route("/flevoland/compliance-dashboard")]
    FlevolandComplianceDashboard,
    #[route("/flevoland/provisa-beheer")]
    FlevolandProvisaBeheer,
    #[route("/context/:id")]
    ContextDetail { id: String },
    #[route("/documents/:id")]
    DocumentDetail { id: String },
    #[route("/search")]
    SearchResults,
    #[route("/graphrag")]
    GraphRAGExplorer,
    #[route("/apps/data-verkenner")]
    DataVerkenner,
    #[route("/apps/document-generator")]
    DocumentGenerator,
    #[route("/apps/nalevingscontrole")]
    Nalevingscontrole,
    #[route("/minfin")]
    MinFinDashboard,
    #[route("/minfin/begrotingsverkenner")]
    MinFinBegrotingsverkenner,
    #[route("/minfin/financiele-controle")]
    MinFinFinancieleControle,
    #[route("/minfin/beleidsdocument-generator")]
    MinFinBeleidsdocumentGenerator,
    #[route("/minfin/kennisnetwerk")]
    MinFinKennisnetwerk,
    #[route("/concept")]
    ConceptDashboard,
    #[route("/concept/context-model")]
    ConceptContextModel,
    #[route("/concept/ai-tooling")]
    ConceptAiTooling,
    #[route("/concept/architectuur")]
    ConceptArchitectuur,
    #[route("/concept/werkwijze")]
    ConceptWerkwijze,
    #[route("/concept/meerjarenplan")]
    ConceptMeerjarenplanConclusies,
    #[route("/zuidholland")]
    ZuidHolland,
    #[route("/zuidholland/mobiliteitsverkenner")]
    ZHMobiliteitsverkenner,
    #[route("/zuidholland/havenmonitor")]
    ZHHavenmonitor,
    #[route("/zuidholland/stakeholder-dossier")]
    ZHStakeholderDossier,
    #[route("/zuidholland/project-portfolio")]
    ZHProjectPortfolio,
    #[route("/zuidholland/kennisnetwerk")]
    ZHKennisnetwerk,
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(state::AppState::new()));

    rsx! {
        document::Style { {STYLE} }
        Router::<Route> {}
    }
}
