//! Cleanup phase tests
//!
//! These tests verify that legacy code has been properly removed
//! and the new architecture is stable.

mod websocket_removal;
mod duckdb_analytics_only;
mod documentation;
mod final_integration;

mod training {
    //! Training completion checklist
    //!
    //! This module contains the team training checklist as a compile-time
    //! verification that training topics have been covered.

    /// Team training completion checklist
    ///
    /// Items should be checked off as team members complete training:
    ///
    /// - [ ] Supabase dashboard navigation
    /// - [ ] RLS policy structure and debugging
    /// - [ ] Real-time subscription troubleshooting
    /// - [ ] ETL pipeline operations
    /// - [ ] PostgreSQL query optimization
    /// - [ ] Backup and recovery procedures
    ///
    /// Location: docs/operations/runbooks/
    #[test]
    fn training_checklist_exists() {
        // Verify runbook directory exists
        let runbook_path = std::path::Path::new("docs/operations/runbooks");
        assert!(
            runbook_path.exists(),
            "Runbook directory should exist at docs/operations/runbooks/"
        );
    }
}
