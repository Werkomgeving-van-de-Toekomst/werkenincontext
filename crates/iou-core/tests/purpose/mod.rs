//! Purpose registry tests
//!
//! Integration tests for the purpose registry including validation,
//! category matching, and approval workflows

use iou_core::purpose::{
    Purpose, PurposeRegistry, PurposeError, LawfulBasis,
};
use chrono::NaiveDate;
use std::sync::Arc;

/// Test helper to create a test purpose
fn create_test_purpose(id: &str, name: &str) -> Purpose {
    Purpose::new(
        id,
        name,
        format!("Test purpose: {}", name),
        LawfulBasis::WettelijkeVerplichting,
        "test-owner",
    )
}

/// Test helper to create a custom purpose with data categories
fn create_custom_purpose(
    id: &str,
    name: &str,
    categories: Vec<String>,
) -> Purpose {
    Purpose::new(
        id,
        name,
        format!("Custom purpose: {}", name),
        LawfulBasis::Toestemming,
        "custom-owner",
    ).with_data_categories(categories)
}

#[test]
fn test_purpose_registry_validation_with_expiry() {
    let registry = PurposeRegistry::new();

    // Create an expired purpose
    let expired = Purpose::new(
        "P099",
        "EXPIRED",
        "Expired test purpose",
        LawfulBasis::WettelijkeVerplichting,
        "owner",
    ).with_validity(None, Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()));

    registry.register(expired).expect("Failed to register expired purpose");

    let result = registry.validate("P099");
    assert!(matches!(result, Err(PurposeError::Expired { .. })));
}

#[test]
fn test_purpose_registry_validation_with_future_start() {
    let registry = PurposeRegistry::new();

    // Create a purpose that starts in the future
    let future = Purpose::new(
        "P100",
        "FUTURE",
        "Future test purpose",
        LawfulBasis::WettelijkeVerplichting,
        "owner",
    ).with_validity(
        Some(NaiveDate::from_ymd_opt(2099, 1, 1).unwrap()),
        None,
    );

    registry.register(future).expect("Failed to register future purpose");

    let result = registry.validate("P100");
    assert!(matches!(result, Err(PurposeError::Expired { .. })));
}

#[test]
fn test_purpose_registry_validation_active_period() {
    let registry = PurposeRegistry::new();

    // Create a purpose within its valid period
    let valid = Purpose::new(
        "P101",
        "VALID",
        "Valid test purpose",
        LawfulBasis::WettelijkeVerplichting,
        "owner",
    ).with_validity(
        Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        Some(NaiveDate::from_ymd_opt(2099, 12, 31).unwrap()),
    );

    registry.register(valid).expect("Failed to register valid purpose");

    let result = registry.validate("P101");
    assert!(result.is_ok());
}

#[test]
fn test_purpose_can_use_data_category() {
    let purpose = create_custom_purpose(
        "C001",
        "Data Access",
        vec!["personal_data".to_string(), "financial_data".to_string()],
    );

    assert!(purpose.can_use_data_category("personal_data"));
    assert!(purpose.can_use_data_category("financial_data"));
    assert!(!purpose.can_use_data_category("health_data"));
}

#[test]
fn test_purpose_registry_category_validation() {
    let registry = PurposeRegistry::new();

    let custom = create_custom_purpose(
        "C002",
        "Limited Access",
        vec!["public_data".to_string()],
    );

    registry.register(custom).expect("Failed to register custom purpose");

    // Should succeed - purpose can use the category
    let result = registry.validate_for_category("C002", "public_data");
    assert!(result.is_ok());

    // Should fail - purpose cannot use this category
    let result = registry.validate_for_category("C002", "restricted_data");
    assert!(matches!(result, Err(PurposeError::CategoryMismatch { .. })));
}

#[test]
fn test_purpose_registry_find_by_lawful_basis() {
    let registry = PurposeRegistry::new();

    // Standard purposes include different lawful bases
    let wettelijk = registry.find_by_lawful_basis(LawfulBasis::WettelijkeVerplichting);
    let toestemming = registry.find_by_lawful_basis(LawfulBasis::Toestemming);

    assert!(!wettelijk.is_empty());
    assert!(!toestemming.is_empty());

    // All returned purposes should have the correct lawful basis
    assert!(wettelijk.iter().all(|p| p.lawful_basis == LawfulBasis::WettelijkeVerplichting));
    assert!(toestemming.iter().all(|p| p.lawful_basis == LawfulBasis::Toestemming));
}

#[test]
fn test_purpose_registry_list_active_filters_inactive() {
    let registry = PurposeRegistry::new();

    let inactive = Purpose::new(
        "I001",
        "INACTIVE",
        "Inactive purpose",
        LawfulBasis::WettelijkeVerplichting,
        "owner",
    );
    let mut inactive_purpose = inactive;
    inactive_purpose.is_active = false;

    registry.register(inactive_purpose).expect("Failed to register inactive purpose");

    let all = registry.list_all();
    let active = registry.list_active();

    // All purposes should include inactive ones
    assert!(all.iter().any(|p| p.id == "I001"));

    // Active purposes should exclude inactive ones
    assert!(!active.iter().any(|p| p.id == "I001"));
}

#[test]
fn test_purpose_registry_register_duplicate_fails() {
    let registry = PurposeRegistry::new();

    let purpose1 = create_test_purpose("D001", "Duplicate");
    let purpose2 = create_test_purpose("D001", "Duplicate 2");

    registry.register(purpose1).expect("Failed to register first purpose");

    let result = registry.register(purpose2);
    assert!(matches!(result, Err(PurposeError::AlreadyExists(_))));
}

#[test]
fn test_purpose_registry_get_not_found() {
    let registry = PurposeRegistry::new();

    let result = registry.get("NONEXISTENT");
    assert!(matches!(result, Err(PurposeError::NotFound(_))));
}

#[test]
fn test_purpose_registry_is_standard() {
    let registry = PurposeRegistry::new();

    // Standard purposes are P001-P015
    assert!(registry.is_standard("P001"));
    assert!(registry.is_standard("P015"));

    // P016 is not standard
    assert!(!registry.is_standard("P016"));

    // Custom IDs are not standard
    assert!(!registry.is_standard("C001"));
    assert!(!registry.is_standard("CUSTOM"));
}

#[test]
fn test_purpose_with_approval_requirement() {
    let mut purpose = create_test_purpose("A001", "Approval Required");
    purpose.requires_approval = true;

    assert!(purpose.requires_approval);

    // Purpose should still be valid even with approval requirement
    assert!(purpose.is_valid_now());
}

#[test]
fn test_purpose_registry_concurrent_access() {
    use std::thread;

    let registry = Arc::new(PurposeRegistry::new());
    let mut handles = vec![];

    // Spawn multiple threads reading from the registry
    for _ in 0..10 {
        let registry_clone = Arc::clone(&registry);
        handles.push(thread::spawn(move || {
            let purposes = registry_clone.list_all();
            assert!(!purposes.is_empty());
        }));
    }

    // All threads should complete successfully
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_purpose_data_categories_management() {
    let purpose = create_test_purpose("CAT001", "Categories Test");

    assert!(!purpose.can_use_data_category("any_category"));

    let with_categories = purpose.with_data_categories(vec![
        "cat1".to_string(),
        "cat2".to_string(),
        "cat3".to_string(),
    ]);

    assert!(with_categories.can_use_data_category("cat1"));
    assert!(with_categories.can_use_data_category("cat2"));
    assert!(with_categories.can_use_data_category("cat3"));
    assert!(!with_categories.can_use_data_category("cat4"));
}

#[test]
fn test_purpose_with_complete_validity_period() {
    let purpose = Purpose::new(
        "V001",
        "Validity Period",
        "Test validity period",
        LawfulBasis::WettelijkeVerplichting,
        "owner",
    ).with_validity(
        Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
    );

    assert_eq!(purpose.valid_from, Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
    assert_eq!(purpose.valid_until, Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()));
}

#[test]
fn test_purpose_registry_custom_purpose_lifecycle() {
    let registry = PurposeRegistry::new();

    // Create custom purpose
    let custom = create_custom_purpose(
        "LIFECYCLE",
        "Lifecycle Test",
        vec!["test_data".to_string()],
    );

    // Register
    registry.register(custom.clone()).expect("Failed to register custom purpose");

    // Get
    let retrieved = registry.get("LIFECYCLE").expect("Failed to get custom purpose");
    assert_eq!(retrieved.name, "Lifecycle Test");

    // Validate
    let validated = registry.validate("LIFECYCLE").expect("Failed to validate custom purpose");
    assert_eq!(validated.id, "LIFECYCLE");

    // Validate with category
    let result = registry.validate_for_category("LIFECYCLE", "test_data");
    assert!(result.is_ok());

    // Fail with wrong category
    let result = registry.validate_for_category("LIFECYCLE", "wrong_data");
    assert!(matches!(result, Err(PurposeError::CategoryMismatch { .. })));

    // Check it's not a standard purpose
    assert!(!registry.is_standard("LIFECYCLE"));
}

#[test]
fn test_purpose_error_display() {
    let err = PurposeError::NotFound("P999".to_string());
    assert_eq!(format!("{}", err), "Purpose not found: P999");

    let err = PurposeError::AlreadyExists("P001".to_string());
    assert_eq!(format!("{}", err), "Purpose already exists: P001");

    let err = PurposeError::Inactive("P002".to_string());
    assert_eq!(format!("{}", err), "Purpose P002 is inactive");

    let err = PurposeError::CategoryMismatch {
        purpose: "P003".to_string(),
        category: "health_data".to_string(),
    };
    assert!(format!("{}", err).contains("P003"));
    assert!(format!("{}", err).contains("health_data"));
}
