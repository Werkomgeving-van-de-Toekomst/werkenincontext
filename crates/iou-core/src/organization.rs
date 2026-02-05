//! Organisatiestructuur: organisaties, afdelingen, gebruikers en rollen

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Overheidsorganisatie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub short_name: Option<String>,
    pub organization_type: OrganizationType,
    pub parent_organization_id: Option<Uuid>,
    pub website: Option<String>,
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Type overheidsorganisatie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrganizationType {
    /// Rijksoverheid (ministeries)
    Rijk,
    /// Provincie
    Provincie,
    /// Gemeente
    Gemeente,
    /// Waterschap
    Waterschap,
    /// Gemeenschappelijke regeling
    Gemeenschappelijk,
    /// Zelfstandig bestuursorgaan
    Zbo,
    /// Overig
    Overig,
}

/// Afdeling binnen een organisatie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub parent_department_id: Option<Uuid>,
    pub manager_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Gebruiker van het systeem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub job_title: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Geef de volledige naam van de gebruiker
    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            _ => self.display_name.clone(),
        }
    }

    /// Geef de initialen van de gebruiker
    pub fn initials(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => {
                let f = first.chars().next().unwrap_or('?');
                let l = last.chars().next().unwrap_or('?');
                format!("{}{}", f.to_uppercase(), l.to_uppercase())
            }
            _ => self
                .display_name
                .chars()
                .take(2)
                .collect::<String>()
                .to_uppercase(),
        }
    }
}

/// Rol binnen het systeem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: serde_json::Value,
    pub is_system_role: bool,
    pub created_at: DateTime<Utc>,
}

/// Koppeling tussen gebruiker en rol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub scope_domain_id: Option<Uuid>, // Optioneel: rol alleen binnen specifiek domein
    pub valid_from: NaiveDate,
    pub valid_until: Option<NaiveDate>,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
}

impl UserRole {
    /// Check of de rol momenteel geldig is
    pub fn is_valid(&self) -> bool {
        let today = Utc::now().date_naive();
        if today < self.valid_from {
            return false;
        }
        if let Some(until) = self.valid_until {
            return today <= until;
        }
        true
    }
}

/// Permissie types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    // Domein permissies
    DomainCreate,
    DomainRead,
    DomainUpdate,
    DomainDelete,
    DomainArchive,

    // Object permissies
    ObjectCreate,
    ObjectRead,
    ObjectUpdate,
    ObjectDelete,
    ObjectClassify,

    // Compliance permissies
    ComplianceAssess,
    ComplianceApprove,
    WooPublish,

    // Admin permissies
    UserManage,
    RoleManage,
    OrganizationManage,
    AuditView,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_full_name() {
        let user = User {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            email: "jan.jansen@flevoland.nl".to_string(),
            display_name: "Jan Jansen".to_string(),
            first_name: Some("Jan".to_string()),
            last_name: Some("Jansen".to_string()),
            department_id: None,
            job_title: Some("Beleidsmedewerker".to_string()),
            phone: None,
            avatar_url: None,
            is_active: true,
            last_login: None,
            created_at: Utc::now(),
        };

        assert_eq!(user.full_name(), "Jan Jansen");
        assert_eq!(user.initials(), "JJ");
    }

    #[test]
    fn test_user_role_validity() {
        let today = Utc::now().date_naive();

        let valid_role = UserRole {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role_id: Uuid::new_v4(),
            scope_domain_id: None,
            valid_from: today - chrono::Duration::days(30),
            valid_until: Some(today + chrono::Duration::days(30)),
            assigned_by: Uuid::new_v4(),
            assigned_at: Utc::now(),
        };

        assert!(valid_role.is_valid());

        let expired_role = UserRole {
            valid_until: Some(today - chrono::Duration::days(1)),
            ..valid_role.clone()
        };

        assert!(!expired_role.is_valid());
    }
}
