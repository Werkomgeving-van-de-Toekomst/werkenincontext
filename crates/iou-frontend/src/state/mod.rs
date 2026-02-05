//! Application state management using Leptos signals

use leptos::prelude::*;
use uuid::Uuid;

use iou_core::domain::InformationDomain;

/// Global application state
#[derive(Clone)]
pub struct AppState {
    /// Current user (simplified for demo)
    pub user: RwSignal<Option<UserInfo>>,
    /// Currently selected domain context
    pub current_domain: RwSignal<Option<InformationDomain>>,
    /// Search query
    pub search_query: RwSignal<String>,
    /// Loading state
    pub is_loading: RwSignal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user: RwSignal::new(Some(UserInfo::demo())),
            current_domain: RwSignal::new(None),
            search_query: RwSignal::new(String::new()),
            is_loading: RwSignal::new(false),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplified user info
#[derive(Clone, Debug)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub initials: String,
    pub organization: String,
    pub role: String,
}

impl UserInfo {
    /// Demo user for development
    pub fn demo() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Jan de Vries".to_string(),
            email: "jan.devries@flevoland.nl".to_string(),
            initials: "JV".to_string(),
            organization: "Provincie Flevoland".to_string(),
            role: "Beleidsmedewerker".to_string(),
        }
    }
}
