//! Page components

mod home;
mod dashboard;
mod context_detail;
mod document_detail;
mod search_results;
mod graphrag_explorer;
mod data_verkenner;
mod document_generator;
mod nalevingscontrole;
mod provisa_manager;
mod compliance_dashboard;
mod minfin;
mod concept;
mod zuidholland;
mod flevoland;

pub use home::Home;
pub use dashboard::Dashboard;
pub use context_detail::ContextDetail;
pub use document_detail::DocumentDetail;
pub use search_results::SearchResults;
pub use graphrag_explorer::GraphRAGExplorer;
pub use data_verkenner::DataVerkenner;
pub use document_generator::DocumentGenerator;
pub use nalevingscontrole::Nalevingscontrole;
pub use provisa_manager::ProvisaManager;
pub use compliance_dashboard::ComplianceDashboard;
pub use minfin::*;
pub use concept::*;
pub use zuidholland::*;
pub use flevoland::*;
