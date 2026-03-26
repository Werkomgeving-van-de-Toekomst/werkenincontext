//! Ministerie van Financiën pages

mod dashboard;
mod begrotingsverkenner;
mod financiele_controle;
mod beleidsdocument_generator;
mod kennisnetwerk;

pub use dashboard::MinFinDashboard;
pub use begrotingsverkenner::MinFinBegrotingsverkenner;
pub use financiele_controle::MinFinFinancieleControle;
pub use beleidsdocument_generator::MinFinBeleidsdocumentGenerator;
pub use kennisnetwerk::MinFinKennisnetwerk;
