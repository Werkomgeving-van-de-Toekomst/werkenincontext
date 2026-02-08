//! Application state management

use uuid::Uuid;

use iou_core::domain::InformationDomain;

/// Global application state
#[derive(Clone, Debug)]
pub struct AppState {
    /// Current user (simplified for demo)
    pub user: Option<UserInfo>,
    /// Currently selected domain context
    pub current_domain: Option<InformationDomain>,
    /// Search query
    pub search_query: String,
    /// Loading state
    pub is_loading: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user: Some(UserInfo::demo()),
            current_domain: None,
            search_query: String::new(),
            is_loading: false,
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
    /// Demo user for development — delegates to Flevoland
    pub fn demo() -> Self {
        Self::flevoland()
    }

    pub fn flevoland() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Jan de Vries".to_string(),
            email: "jan.devries@flevoland.nl".to_string(),
            initials: "JV".to_string(),
            organization: "Provincie Flevoland".to_string(),
            role: "Beleidsmedewerker".to_string(),
        }
    }

    pub fn minfin() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Maria van den Berg".to_string(),
            email: "m.vandenberg@minfin.nl".to_string(),
            initials: "MB".to_string(),
            organization: "Ministerie van Financi\u{00eb}n".to_string(),
            role: "Senior Beleidsadviseur".to_string(),
        }
    }

    pub fn zuidholland() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Pieter Jansen".to_string(),
            email: "p.jansen@pzh.nl".to_string(),
            initials: "PJ".to_string(),
            organization: "Provincie Zuid-Holland".to_string(),
            role: "Programmamanager Mobiliteit".to_string(),
        }
    }

    pub fn concept() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "IOU Platform".to_string(),
            email: "platform@iou-modern.nl".to_string(),
            initials: "IO".to_string(),
            organization: "IOU-Modern".to_string(),
            role: "Platform".to_string(),
        }
    }
}
