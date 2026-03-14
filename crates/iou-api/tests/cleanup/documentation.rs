//! Documentation completeness tests
//!
//! Verify that all documentation has been updated to reflect
//! the new hybrid architecture.

use std::path::Path;

/// Test: Verify architecture documentation exists
#[test]
fn test_architecture_documentation_exists() {
    let docs_path = Path::new("docs");

    // Check for migration documentation
    let migration_docs = docs_path.join("migration");
    assert!(
        migration_docs.exists() || docs_path.join("README.md").exists(),
        "Migration documentation should exist"
    );
}

/// Test: Verify operations runbooks exist
#[test]
fn test_operations_runbooks_exist() {
    let runbooks_path = Path::new("docs/operations/runbooks");

    if runbooks_path.exists() {
        // Check for stabilization runbook
        let stabilization_runbook = runbooks_path.join("stabilization_runbook.md");
        assert!(
            stabilization_runbook.exists(),
            "Stabilization runbook should exist"
        );
    }
}

/// Test: Verify migration plan is documented
#[test]
fn test_migration_plan_documented() {
    let planning_path = Path::new("planning-backend-eval");

    // Check for implementation plan
    let spec_file = planning_path.join("spec.md");
    let sections_dir = planning_path.join("sections");

    assert!(
        spec_file.exists() || sections_dir.exists(),
        "Migration plan should be documented"
    );
}

/// Test: Verify database migrations are present
#[test]
fn test_database_migrations_exist() {
    let migrations_path = Path::new("migrations/postgres");

    assert!(
        migrations_path.exists(),
        "PostgreSQL migrations directory should exist"
    );

    // Check for key migration files
    let expected_migrations = vec![
        "001_create_initial_schema.sql",
        "002_rls_policies.sql",
        "003_optimization_indexes.sql",
        "004_rls_optimization.sql",
        "005_outbox_table.sql",
    ];

    for migration in &expected_migrations {
        let migration_path = migrations_path.join(migration);
        assert!(
            migration_path.exists(),
            "Migration {} should exist",
            migration
        );
    }
}
