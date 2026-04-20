//! Purpose Registry
//!
//! Beheert alle beschikbare purposes en valideert toegang.

use crate::purpose::{categories::standard_purposes, LawfulBasis};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;

/// Purpose identifier (e.g., "P001", "P002", "CUSTOM")
pub type PurposeId = String;

/// Errors that can occur during purpose operations
#[derive(Debug, Error)]
pub enum PurposeError {
    #[error("Purpose not found: {0}")]
    NotFound(String),

    #[error("Purpose already exists: {0}")]
    AlreadyExists(String),

    #[error("Purpose {id} expired (valid until: {valid_until:?})")]
    Expired { id: String, valid_until: Option<NaiveDate> },

    #[error("Purpose {0} is inactive")]
    Inactive(String),

    #[error("Purpose {purpose} cannot be used for category {category}")]
    CategoryMismatch { purpose: String, category: String },
}

/// Purpose data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purpose {
    /// Purpose ID (e.g., P001, P002)
    pub id: PurposeId,

    /// Purpose name
    pub name: String,

    /// Purpose description
    pub description: String,

    /// Wettelijke grondslag (AVG Art. 6)
    pub lawful_basis: LawfulBasis,

    /// Data categories this purpose can use
    pub data_categories: Vec<String>,

    /// Purpose owner
    pub owner: String,

    /// Whether this purpose requires approval before use
    pub requires_approval: bool,

    /// Valid from date (optional)
    pub valid_from: Option<NaiveDate>,

    /// Valid until date (optional)
    pub valid_until: Option<NaiveDate>,

    /// Whether this purpose is currently active
    #[serde(default)]
    pub is_active: bool,
}

impl Purpose {
    /// Create a new Purpose
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        lawful_basis: LawfulBasis,
        owner: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            lawful_basis,
            data_categories: Vec::new(),
            owner: owner.into(),
            requires_approval: false,
            valid_from: None,
            valid_until: None,
            is_active: true,
        }
    }

    /// Add data categories to this purpose
    pub fn with_data_categories(mut self, categories: Vec<String>) -> Self {
        self.data_categories = categories;
        self
    }

    /// Set validity period
    pub fn with_validity(mut self, from: Option<NaiveDate>, until: Option<NaiveDate>) -> Self {
        self.valid_from = from;
        self.valid_until = until;
        self
    }

    /// Check if this purpose is currently valid (not expired)
    pub fn is_valid_now(&self) -> bool {
        if !self.is_active {
            return false;
        }

        let now = Utc::now().date_naive();

        if let Some(from) = self.valid_from {
            if now < from {
                return false;
            }
        }

        if let Some(until) = self.valid_until {
            if now > until {
                return false;
            }
        }

        true
    }

    /// Check if this purpose can use a specific data category
    pub fn can_use_data_category(&self, category: &str) -> bool {
        self.data_categories.contains(&category.to_string())
    }
}

/// Purpose registry - manages all available purposes
#[derive(Debug)]
pub struct PurposeRegistry {
    purposes: RwLock<HashMap<PurposeId, Purpose>>,
}

impl PurposeRegistry {
    /// Create a new PurposeRegistry with standard purposes
    pub fn new() -> Self {
        let mut registry = Self {
            purposes: RwLock::new(standard_purposes()),
        };

        // Mark all standard purposes as not requiring approval
        // (they're pre-approved by DPO)
        for purpose in registry.purposes.write().unwrap().values_mut() {
            purpose.requires_approval = false;
        }

        registry
    }

    /// Get a purpose by ID
    pub fn get(&self, id: &str) -> Result<Purpose, PurposeError> {
        self.purposes
            .read()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| PurposeError::NotFound(id.to_string()))
    }

    /// Validate a purpose ID - returns the validated purpose if valid
    pub fn validate(&self, id: &str) -> Result<Purpose, PurposeError> {
        let purpose = self.get(id)?;

        if !purpose.is_valid_now() {
            return Err(PurposeError::Expired {
                id: id.to_string(),
                valid_until: purpose.valid_until,
            });
        }

        if !purpose.is_active {
            return Err(PurposeError::Inactive(id.to_string()));
        }

        Ok(purpose)
    }

    /// Validate that a purpose can be used for a specific data category
    pub fn validate_for_category(
        &self,
        purpose_id: &str,
        category: &str,
    ) -> Result<Purpose, PurposeError> {
        let purpose = self.validate(purpose_id)?;

        if !purpose.can_use_data_category(category) {
            return Err(PurposeError::CategoryMismatch {
                purpose: purpose_id.to_string(),
                category: category.to_string(),
            });
        }

        Ok(purpose)
    }

    /// Register a custom purpose
    pub fn register(&self, purpose: Purpose) -> Result<(), PurposeError> {
        let mut purposes = self.purposes.write().unwrap();

        if purposes.contains_key(&purpose.id) {
            return Err(PurposeError::AlreadyExists(purpose.id));
        }

        purposes.insert(purpose.id.clone(), purpose);
        Ok(())
    }

    /// List all purposes
    pub fn list_all(&self) -> Vec<Purpose> {
        self.purposes
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// List only active purposes
    pub fn list_active(&self) -> Vec<Purpose> {
        self.purposes
            .read()
            .unwrap()
            .values()
            .filter(|p| p.is_valid_now())
            .cloned()
            .collect()
    }

    /// Find purposes by lawful basis
    pub fn find_by_lawful_basis(&self, basis: LawfulBasis) -> Vec<Purpose> {
        self.purposes
            .read()
            .unwrap()
            .values()
            .filter(|p| p.lawful_basis == basis)
            .cloned()
            .collect()
    }

    /// Check if a purpose is a standard purpose
    pub fn is_standard(&self, id: &str) -> bool {
        id.starts_with('P')
            && id.len() == 4
            && id[1..].parse::<u32>().map_or(false, |n| (1..=15).contains(&n))
    }
}

impl Default for PurposeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new_has_standard_purposes() {
        let registry = PurposeRegistry::new();
        let purposes = registry.list_all();

        assert_eq!(purposes.len(), 15);
    }

    #[test]
    fn test_registry_get_p001() {
        let registry = PurposeRegistry::new();
        let purpose = registry.get("P001").unwrap();

        assert_eq!(purpose.id, "P001");
        assert_eq!(purpose.name, "ZAAK_AFHANDELING");
    }

    #[test]
    fn test_registry_get_not_found() {
        let registry = PurposeRegistry::new();
        let result = registry.get("P999");

        assert!(matches!(result, Err(PurposeError::NotFound(_))));
    }

    #[test]
    fn test_registry_validate_active() {
        let registry = PurposeRegistry::new();
        let purpose = registry.validate("P001").unwrap();

        assert_eq!(purpose.id, "P001");
    }

    #[test]
    fn test_registry_validate_for_category() {
        let registry = PurposeRegistry::new();

        // P001 can use zaak_data
        let result = registry.validate_for_category("P001", "zaak_data");
        assert!(result.is_ok());

        // But not financieel_data
        let result = registry.validate_for_category("P001", "financieel_data");
        assert!(matches!(result, Err(PurposeError::CategoryMismatch { .. })));
    }

    #[test]
    fn test_purpose_is_valid_now() {
        let purpose = Purpose::new("P099", "TEST", "Test", LawfulBasis::WettelijkeVerplichting, "Owner");

        assert!(purpose.is_valid_now());

        // Expired purpose
        let expired = Purpose::new(
            "P099",
            "TEST",
            "Test",
            LawfulBasis::WettelijkeVerplichting,
            "Owner",
        )
        .with_validity(None, Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()));

        assert!(!expired.is_valid_now());
    }

    #[test]
    fn test_purpose_can_use_data_category() {
        let purpose = Purpose::new("P099", "TEST", "Test", LawfulBasis::WettelijkeVerplichting, "Owner")
            .with_data_categories(vec!["test_data".to_string()]);

        assert!(purpose.can_use_data_category("test_data"));
        assert!(!purpose.can_use_data_category("other_data"));
    }

    #[test]
    fn test_registry_find_by_lawful_basis() {
        let registry = PurposeRegistry::new();
        let wettelijk = registry.find_by_lawful_basis(LawfulBasis::WettelijkeVerplichting);

        assert!(!wettelijk.is_empty());
        assert!(wettelijk.iter().all(|p| p.lawful_basis == LawfulBasis::WettelijkeVerplichting));
    }

    #[test]
    fn test_registry_list_active() {
        let registry = PurposeRegistry::new();
        let active = registry.list_active();

        // All standard purposes should be active
        assert_eq!(active.len(), 15);
    }

    #[test]
    fn test_registry_is_standard() {
        let registry = PurposeRegistry::new();

        assert!(registry.is_standard("P001"));
        assert!(registry.is_standard("P015"));
        assert!(!registry.is_standard("P016"));
        assert!(!registry.is_standard("CUSTOM"));
    }

    #[test]
    fn test_registry_register_custom() {
        let registry = PurposeRegistry::new();

        let custom = Purpose::new(
            "C001",
            "CUSTOM",
            "Custom purpose",
            LawfulBasis::Toestemming,
            "Owner",
        );

        registry.register(custom.clone()).unwrap();

        let retrieved = registry.get("C001").unwrap();
        assert_eq!(retrieved.name, "CUSTOM");
    }

    #[test]
    fn test_registry_register_duplicate_fails() {
        let registry = PurposeRegistry::new();

        let duplicate = Purpose::new(
            "P001",
            "DUPLICATE",
            "Duplicate",
            LawfulBasis::Toestemming,
            "Owner",
        );

        let result = registry.register(duplicate);
        assert!(matches!(result, Err(PurposeError::AlreadyExists(_))));
    }
}
