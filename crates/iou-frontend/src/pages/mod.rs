//! Page components

mod approval_queue;
mod compliance_dashboard;
mod concept;
mod context_detail;
mod dashboard;
mod data_verkenner;
mod document_creator;
mod document_detail;
mod document_generator;
mod flevoland;
mod graphrag_explorer;
mod home;
mod minfin;
mod nalevingscontrole;
mod provisa_manager;
mod search_results;
mod template_manager;
mod zuidholland;

pub use approval_queue::ApprovalQueue;
pub use compliance_dashboard::ComplianceDashboard;
pub use dashboard::Dashboard;
pub use context_detail::ContextDetail;
pub use document_detail::DocumentDetail;
pub use document_creator::DocumentCreator;
pub use search_results::SearchResults;
pub use graphrag_explorer::GraphRAGExplorer;
pub use data_verkenner::DataVerkenner;
pub use document_generator::DocumentGenerator;
pub use nalevingscontrole::Nalevingscontrole;
pub use provisa_manager::ProvisaManager;
pub use template_manager::TemplateManager;
pub use home::Home;
pub use minfin::*;
pub use concept::*;
pub use zuidholland::*;
pub use flevoland::*;
