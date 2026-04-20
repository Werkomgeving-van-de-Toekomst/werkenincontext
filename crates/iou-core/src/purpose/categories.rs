//! Standard Purpose Categories
//!
//! Defines the 15 standard data processing purposes for government organizations
//! per IHH01: "Vastlegging en uitwisseling gegevens gebonden aan doelbinding"

use crate::purpose::{DataCategory, LawfulBasis, Purpose};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Purpose category definition for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurposeCategory {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub lawful_basis: LawfulBasis,
    pub data_categories: Vec<String>,
    pub owner: &'static str,
}

impl PurposeCategory {
    /// Convert to a Purpose instance
    pub fn to_purpose(&self) -> Purpose {
        Purpose::new(
            self.id,
            self.name,
            self.description,
            self.lawful_basis,
            self.owner,
        )
        .with_data_categories(self.data_categories.clone())
    }
}

/// Standard purposes for government data processing (P001-P015)
pub static STANDARD_PURPOSES: Lazy<Vec<PurposeCategory>> = Lazy::new(|| {
    vec![
        PurposeCategory {
            id: "P001",
            name: "ZAAK_AFHANDELING",
            description: "Case processing tasks",
            lawful_basis: LawfulBasis::WettelijkeVerplichting,
            data_categories: vec!["persoonsgegevens".to_string(), "zaak_data".to_string()],
            owner: "Domain Owner",
        },
        PurposeCategory {
            id: "P002",
            name: "WOO_PUBLICATIE",
            description: "Woo publication process",
            lawful_basis: LawfulBasis::WettelijkeVerplichting,
            data_categories: vec!["besluit".to_string(), "document".to_string()],
            owner: "WOO Officer",
        },
        PurposeCategory {
            id: "P003",
            name: "ANALYSE",
            description: "Data analysis and reporting",
            lawful_basis: LawfulBasis::AlgemeenBelang,
            data_categories: vec!["geaggregeerde_data".to_string()],
            owner: "Data Analyst",
        },
        PurposeCategory {
            id: "P004",
            name: "DIENSTVERLENING",
            description: "Service delivery to citizens",
            lawful_basis: LawfulBasis::Overeenkomst,
            data_categories: vec!["persoonsgegevens".to_string(), "zaak_data".to_string()],
            owner: "Service Manager",
        },
        PurposeCategory {
            id: "P005",
            name: "HANDHAVING",
            description: "Enforcement and supervision",
            lawful_basis: LawfulBasis::WettelijkeVerplichting,
            data_categories: vec![
                "persoonsgegevens".to_string(),
                "bijzondere_gegevens".to_string(),
            ],
            owner: "Supervision Officer",
        },
        PurposeCategory {
            id: "P006",
            name: "BELEIDSVORMING",
            description: "Policy development",
            lawful_basis: LawfulBasis::AlgemeenBelang,
            data_categories: vec!["beleids_data".to_string(), "geaggregeerde_data".to_string()],
            owner: "Policy Director",
        },
        PurposeCategory {
            id: "P007",
            name: "ONDERZOEK",
            description: "Statistical research",
            lawful_basis: LawfulBasis::GerechtvaardigdBelang,
            data_categories: vec!["geaggregeerde_data".to_string()],
            owner: "Research Lead",
        },
        PurposeCategory {
            id: "P008",
            name: "ARCHIVERING",
            description: "Archival record keeping",
            lawful_basis: LawfulBasis::WettelijkeVerplichting,
            data_categories: vec!["document_data".to_string(), "zaak_data".to_string()],
            owner: "Archivist",
        },
        PurposeCategory {
            id: "P009",
            name: "CORRECTIE",
            description: "Data correction requests",
            lawful_basis: LawfulBasis::WettelijkeVerplichting,
            data_categories: vec!["persoonsgegevens".to_string()],
            owner: "Data Steward",
        },
        PurposeCategory {
            id: "P010",
            name: "UITVOERING_BESLUIT",
            description: "Decision implementation",
            lawful_basis: LawfulBasis::WettelijkeVerplichting,
            data_categories: vec!["persoonsgegevens".to_string(), "zaak_data".to_string()],
            owner: "Case Manager",
        },
        PurposeCategory {
            id: "P011",
            name: "CONTACT_BURGER",
            description: "Citizen communication",
            lawful_basis: LawfulBasis::Overeenkomst,
            data_categories: vec!["communicatie_data".to_string()],
            owner: "Contact Center",
        },
        PurposeCategory {
            id: "P012",
            name: "FINANCIEN",
            description: "Financial administration",
            lawful_basis: LawfulBasis::WettelijkeVerplichting,
            data_categories: vec!["financieel_data".to_string()],
            owner: "Financial Controller",
        },
        PurposeCategory {
            id: "P013",
            name: "SAMENWERKING",
            description: "Inter-agency collaboration",
            lawful_basis: LawfulBasis::AlgemeenBelang,
            data_categories: vec!["zaak_data".to_string(), "beleids_data".to_string()],
            owner: "Partnership Manager",
        },
        PurposeCategory {
            id: "P014",
            name: "KWIJTALING_VERJARING",
            description: "Statute of limitations",
            lawful_basis: LawfulBasis::WettelijkeVerplichting,
            data_categories: vec![
                "persoonsgegevens".to_string(),
                "financieel_data".to_string(),
            ],
            owner: "Legal Department",
        },
        PurposeCategory {
            id: "P015",
            name: "AUDIT",
            description: "Internal audit and control",
            lawful_basis: LawfulBasis::GerechtvaardigdBelang,
            data_categories: vec!["financieel_data".to_string(), "zaak_data".to_string()],
            owner: "Internal Auditor",
        },
    ]
});

/// Get a HashMap of all standard purposes
pub fn standard_purposes() -> HashMap<String, Purpose> {
    STANDARD_PURPOSES
        .iter()
        .map(|cat| (cat.id.to_string(), cat.to_purpose()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_standard_purposes() {
        let purposes = standard_purposes();
        assert_eq!(purposes.len(), 15);
    }

    #[test]
    fn test_purpose_ids() {
        let purposes = standard_purposes();
        for i in 1..=15 {
            let id = format!("P{:03}", i);
            assert!(purposes.contains_key(&id), "Missing purpose {}", id);
        }
    }

    #[test]
    fn test_government_purposes_use_government_basis() {
        let purposes = standard_purposes();

        // Government-specific purposes
        let gov_purposes = ["P001", "P002", "P005", "P006", "P008", "P009", "P010", "P012", "P014"];

        for id in gov_purposes {
            let purpose = purposes.get(id).expect("Purpose should exist");
            assert!(
                purpose.lawful_basis.is_government_specific(),
                "Purpose {} should use government-specific basis, got {:?}",
                id,
                purpose.lawful_basis
            );
        }
    }

    #[test]
    fn test_to_purpose_conversion() {
        let cat = &STANDARD_PURPOSES[0]; // P001
        let purpose = cat.to_purpose();

        assert_eq!(purpose.id, "P001");
        assert_eq!(purpose.name, "ZAAK_AFHANDELING");
        assert_eq!(purpose.data_categories.len(), 2);
    }
}
