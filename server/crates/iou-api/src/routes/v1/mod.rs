//! Version 1 API endpoints

pub mod rules;
pub mod calculations;
pub mod processes;
pub mod data_subject_rights;
pub mod woo;
pub mod categories;
pub mod tags;
pub mod settings;

pub use rules::{list_rules, evaluate_rule, get_open_regels_rule, RuleEvaluationRequest};
pub use calculations::{start_calculation, CalculationRequest, CalculationResponse};
pub use processes::{start_process, get_process, ProcessRequest, ProcessResponse, ProcessInstanceResponse};

// Data Subject Rights exports
pub use data_subject_rights::{
    create_sar, get_sar, list_my_dsar,
    create_rectification, get_rectification, approve_rectification,
    create_erasure, get_erasure, approve_erasure,
    list_pending_dsar,
};

// Woo publication exports
pub use woo::{
    request_woo_publication, list_woo_publications, get_woo_publication,
    approve_woo_publication, publish_woo_publication, withdraw_woo_publication,
    create_woo_request, list_woo_requests, get_woo_request,
    get_woo_statistics, get_woo_deadlines, get_published_woo_documents,
    mark_consultation_complete,
};
