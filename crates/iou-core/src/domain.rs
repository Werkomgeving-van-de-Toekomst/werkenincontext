//! Informatiedomeinen - de centrale organiserende eenheden in IOU
//!
//! Een informatiedomein is de context waarbinnen informatie wordt georganiseerd.
//! Er zijn vier typen: zaak, project, beleid en expertise.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

/// Type informatiedomein
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DomainType {
    /// Zaak: uitvoerend werk (vergunningen, subsidies, bezwaren)
    Zaak,
    /// Project: tijdelijke samenwerkingsinitiatieven
    Project,
    /// Beleid: beleidsontwikkeling en -evaluatie
    Beleid,
    /// Expertise: kennisdeling en samenwerking
    Expertise,
}

/// Status van een informatiedomein
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DomainStatus {
    /// Concept: nog niet actief
    Concept,
    /// Actief: in behandeling
    Actief,
    /// Afgerond: voltooid
    Afgerond,
    /// Gearchiveerd: in archief
    Gearchiveerd,
}

impl Default for DomainStatus {
    fn default() -> Self {
        Self::Actief
    }
}

/// Informatiedomein - de centrale organiserende eenheid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationDomain {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub domain_type: DomainType,
    pub name: String,
    pub description: Option<String>,
    pub status: DomainStatus,
    pub organization_id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub parent_domain_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InformationDomain {
    pub fn new(
        domain_type: DomainType,
        name: String,
        organization_id: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            domain_type,
            name,
            description: None,
            status: DomainStatus::default(),
            organization_id,
            owner_user_id: None,
            parent_domain_id: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Zaak - uitvoerend werk zoals vergunningen, subsidies, bezwaren
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub case_number: String,
    pub case_type: String,
    pub subject: String,
    pub start_date: NaiveDate,
    pub target_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub legal_basis: Option<String>,
    pub retention_period: Option<i32>,
    pub disclosure_class: Option<String>,
}

impl Case {
    /// Bereken of de zaak binnen de termijn is
    pub fn is_within_deadline(&self) -> Option<bool> {
        self.target_date.map(|target| {
            let today = Utc::now().date_naive();
            today <= target
        })
    }

    /// Bereken dagen tot deadline
    pub fn days_until_deadline(&self) -> Option<i64> {
        self.target_date.map(|target| {
            let today = Utc::now().date_naive();
            (target - today).num_days()
        })
    }
}

/// Project - tijdelijk samenwerkingsinitiatief
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub project_code: String,
    pub project_name: String,
    pub start_date: NaiveDate,
    pub planned_end_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,
    pub budget: Option<f64>,
    pub project_manager_id: Option<Uuid>,
}

impl Project {
    /// Bereken of het project op schema ligt
    pub fn is_on_schedule(&self) -> Option<bool> {
        if self.actual_end_date.is_some() {
            return None; // Project is afgerond
        }
        self.planned_end_date.map(|planned| {
            let today = Utc::now().date_naive();
            today <= planned
        })
    }
}

/// Beleidstopic - beleidsontwikkeling en -evaluatie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTopic {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub policy_area: String,
    pub policy_phase: PolicyPhase,
    pub responsible_department_id: Option<Uuid>,
    pub review_date: Option<NaiveDate>,
}

/// Fase van beleidsontwikkeling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum PolicyPhase {
    /// Agendavorming
    Agendavorming,
    /// Beleidsvoorbereiding
    Voorbereiding,
    /// Besluitvorming
    Besluitvorming,
    /// Uitvoering
    Uitvoering,
    /// Evaluatie
    Evaluatie,
    /// Beëindiging
    Beeindiging,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_type_serialization() {
        let dt = DomainType::Zaak;
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, "\"zaak\"");

        let parsed: DomainType = serde_json::from_str("\"project\"").unwrap();
        assert_eq!(parsed, DomainType::Project);
    }

    #[test]
    fn test_case_deadline() {
        let mut case = Case {
            id: Uuid::new_v4(),
            domain_id: Uuid::new_v4(),
            case_number: "Z-2025-001".to_string(),
            case_type: "vergunning".to_string(),
            subject: "Omgevingsvergunning bouw".to_string(),
            start_date: Utc::now().date_naive(),
            target_date: Some(Utc::now().date_naive() + chrono::Duration::days(30)),
            end_date: None,
            legal_basis: Some("Omgevingswet".to_string()),
            retention_period: Some(20),
            disclosure_class: Some("openbaar".to_string()),
        };

        assert!(case.is_within_deadline().unwrap());
        assert!(case.days_until_deadline().unwrap() > 0);

        // Verplaats deadline naar verleden
        case.target_date = Some(Utc::now().date_naive() - chrono::Duration::days(5));
        assert!(!case.is_within_deadline().unwrap());
    }
}
