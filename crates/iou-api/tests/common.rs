//! Common test infrastructure

// Re-export test modules
pub mod concurrency;
pub mod helpers;
pub mod integration;
pub mod mocks;
pub mod performance;

// Re-export commonly used types
pub use mocks::MockS3Client;

use std::sync::Arc;
use uuid::Uuid;

/// Generate a random test document ID
pub fn random_document_id() -> Uuid {
    Uuid::new_v4()
}

/// Generate a random test organization ID
pub fn random_organization_id() -> Uuid {
    Uuid::new_v4()
}

/// Generate a random test user ID
pub fn random_user_id() -> Uuid {
    Uuid::new_v4()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_ids_are_unique() {
        let id1 = random_document_id();
        let id2 = random_document_id();

        assert_ne!(id1, id2, "Random IDs should be unique");
        assert_ne!(id1, Uuid::default(), "Random ID should not be nil");
    }
}
