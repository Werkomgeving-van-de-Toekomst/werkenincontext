//! Delegation types for approval authority transfer
//!
//! Defines types for temporary, permanent, and bulk delegations between users,
//! along with resolution logic for determining actual approvers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A delegation of approval authority from one user to another
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Delegation {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub delegation_type: DelegationType,
    pub document_types: Vec<String>,
    pub document_id: Option<Uuid>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl Delegation {
    /// Check if this delegation is currently active
    pub fn is_currently_active(&self) -> bool {
        if !self.is_active {
            return false;
        }
        let now = Utc::now();
        if now < self.starts_at {
            return false;
        }
        if let Some(ends_at) = self.ends_at {
            if now > ends_at {
                return false;
            }
        }
        true
    }

    /// Check if this delegation applies to a specific document type
    pub fn applies_to_document_type(&self, document_type: &str) -> bool {
        // Empty list means applies to all document types
        if self.document_types.is_empty() {
            return true;
        }
        self.document_types.iter().any(|dt| dt == document_type)
    }

    /// Check if this delegation applies to a specific document
    pub fn applies_to_document(&self, document_id: Uuid, document_type: &str) -> bool {
        // If document_id is set, it only applies to that specific document
        if let Some(delegated_doc_id) = self.document_id {
            if delegated_doc_id != document_id {
                return false;
            }
        }
        self.applies_to_document_type(document_type)
    }

    /// Validate the delegation configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.from_user_id == self.to_user_id {
            return Err("Cannot delegate to self".to_string());
        }
        if let Some(ends_at) = self.ends_at {
            if ends_at <= self.starts_at {
                return Err("End time must be after start time".to_string());
            }
        }
        // Validate delegation type matches document configuration
        match self.delegation_type {
            DelegationType::Bulk => {
                if self.document_id.is_some() {
                    return Err("Bulk delegations cannot specify a single document".to_string());
                }
            }
            DelegationType::Permanent => {
                if self.ends_at.is_some() {
                    return Err("Permanent delegations cannot have an end time".to_string());
                }
            }
            DelegationType::Temporary => {
                if self.ends_at.is_none() {
                    return Err("Temporary delegations must have an end time".to_string());
                }
            }
        }
        Ok(())
    }

    /// Create a new temporary delegation
    pub fn new_temporary(
        from_user_id: Uuid,
        to_user_id: Uuid,
        document_types: Vec<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_user_id,
            to_user_id,
            delegation_type: DelegationType::Temporary,
            document_types,
            document_id: None,
            starts_at,
            ends_at: Some(ends_at),
            is_active: true,
            created_at: Utc::now(),
            created_by,
        }
    }

    /// Create a new permanent delegation
    pub fn new_permanent(
        from_user_id: Uuid,
        to_user_id: Uuid,
        document_types: Vec<String>,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            from_user_id,
            to_user_id,
            delegation_type: DelegationType::Permanent,
            document_types,
            document_id: None,
            starts_at: now,
            ends_at: None,
            is_active: true,
            created_at: now,
            created_by,
        }
    }

    /// Create a new bulk delegation
    pub fn new_bulk(
        from_user_id: Uuid,
        to_user_id: Uuid,
        document_types: Vec<String>,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            from_user_id,
            to_user_id,
            delegation_type: DelegationType::Bulk,
            document_types,
            document_id: None,
            starts_at: now,
            ends_at: None,
            is_active: true,
            created_at: now,
            created_by,
        }
    }

    /// Create a new document-specific delegation
    pub fn new_for_document(
        from_user_id: Uuid,
        to_user_id: Uuid,
        document_id: Uuid,
        starts_at: DateTime<Utc>,
        ends_at: Option<DateTime<Utc>>,
        created_by: Uuid,
    ) -> Self {
        let delegation_type = if ends_at.is_some() {
            DelegationType::Temporary
        } else {
            DelegationType::Permanent
        };
        Self {
            id: Uuid::new_v4(),
            from_user_id,
            to_user_id,
            delegation_type,
            document_types: vec![],
            document_id: Some(document_id),
            starts_at,
            ends_at,
            is_active: true,
            created_at: Utc::now(),
            created_by,
        }
    }
}

/// The scope/type of delegation
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DelegationType {
    /// Temporary delegation with a specific date range
    Temporary,
    /// Permanent delegation (no end date)
    Permanent,
    /// Bulk delegation for multiple document types
    Bulk,
}

/// A resolved approver with delegation chain information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResolvedApprover {
    pub user_id: Uuid,
    pub delegation_chain: Vec<Uuid>,
    pub is_delegated: bool,
}

impl ResolvedApprover {
    /// Create a non-delegated approver
    pub fn direct(user_id: Uuid) -> Self {
        Self {
            user_id,
            delegation_chain: Vec::new(),
            is_delegated: false,
        }
    }

    /// Create a delegated approver
    pub fn delegated(user_id: Uuid, chain: Vec<Uuid>) -> Self {
        Self {
            user_id,
            delegation_chain: chain,
            is_delegated: true,
        }
    }

    /// Get the original approver before any delegations
    pub fn original_approver(&self) -> Uuid {
        self.delegation_chain
            .first()
            .copied()
            .unwrap_or(self.user_id)
    }

    /// Get the length of the delegation chain
    pub fn chain_length(&self) -> usize {
        self.delegation_chain.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_delegation_validates_no_self_delegation() {
        let user_id = Uuid::new_v4();
        let delegation = Delegation {
            id: Uuid::new_v4(),
            from_user_id: user_id,
            to_user_id: user_id,
            delegation_type: DelegationType::Temporary,
            document_types: vec![],
            document_id: None,
            starts_at: Utc::now(),
            ends_at: Some(Utc::now() + Duration::hours(24)),
            is_active: true,
            created_at: Utc::now(),
            created_by: user_id,
        };
        assert!(delegation.validate().is_err());
    }

    #[test]
    fn test_delegation_is_currently_active() {
        let now = Utc::now();

        // Active delegation
        let active = Delegation {
            id: Uuid::new_v4(),
            from_user_id: Uuid::new_v4(),
            to_user_id: Uuid::new_v4(),
            delegation_type: DelegationType::Temporary,
            document_types: vec![],
            document_id: None,
            starts_at: now - Duration::hours(1),
            ends_at: Some(now + Duration::hours(1)),
            is_active: true,
            created_at: now,
            created_by: Uuid::new_v4(),
        };
        assert!(active.is_currently_active());

        // Not yet started
        let not_started = Delegation { starts_at: now + Duration::hours(1), ..active.clone() };
        assert!(!not_started.is_currently_active());

        // Expired
        let expired = Delegation { ends_at: Some(now - Duration::hours(1)), ..active.clone() };
        assert!(!expired.is_currently_active());

        // Marked inactive
        let inactive = Delegation { is_active: false, ..active };
        assert!(!inactive.is_currently_active());
    }

    #[test]
    fn test_delegation_applies_to_document_type() {
        let delegation = Delegation {
            id: Uuid::new_v4(),
            from_user_id: Uuid::new_v4(),
            to_user_id: Uuid::new_v4(),
            delegation_type: DelegationType::Temporary,
            document_types: vec!["invoice".to_string(), "contract".to_string()],
            document_id: None,
            starts_at: Utc::now(),
            ends_at: Some(Utc::now() + Duration::hours(24)),
            is_active: true,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        };

        assert!(delegation.applies_to_document_type("invoice"));
        assert!(delegation.applies_to_document_type("contract"));
        assert!(!delegation.applies_to_document_type("purchase_order"));
    }

    #[test]
    fn test_delegation_applies_to_all_document_types_when_empty() {
        let delegation = Delegation {
            id: Uuid::new_v4(),
            from_user_id: Uuid::new_v4(),
            to_user_id: Uuid::new_v4(),
            delegation_type: DelegationType::Permanent,
            document_types: vec![],
            document_id: None,
            starts_at: Utc::now(),
            ends_at: None,
            is_active: true,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        };

        assert!(delegation.applies_to_document_type("invoice"));
        assert!(delegation.applies_to_document_type("contract"));
        assert!(delegation.applies_to_document_type("purchase_order"));
    }

    #[test]
    fn test_delegation_applies_to_specific_document() {
        let doc_id = Uuid::new_v4();
        let delegation = Delegation {
            id: Uuid::new_v4(),
            from_user_id: Uuid::new_v4(),
            to_user_id: Uuid::new_v4(),
            delegation_type: DelegationType::Temporary,
            document_types: vec![],
            document_id: Some(doc_id),
            starts_at: Utc::now(),
            ends_at: Some(Utc::now() + Duration::hours(24)),
            is_active: true,
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        };

        assert!(delegation.applies_to_document(doc_id, "invoice"));
        assert!(!delegation.applies_to_document(Uuid::new_v4(), "invoice"));
    }

    #[test]
    fn test_resolved_approver_delegated() {
        let user_id = Uuid::new_v4();

        let direct = ResolvedApprover::direct(user_id);
        assert!(!direct.is_delegated);
        assert!(direct.delegation_chain.is_empty());
        assert_eq!(direct.user_id, user_id);

        let chain = vec![Uuid::new_v4(), Uuid::new_v4()];
        let delegated = ResolvedApprover::delegated(user_id, chain.clone());
        assert!(delegated.is_delegated);
        assert_eq!(delegated.delegation_chain, chain);
        assert_eq!(delegated.user_id, user_id);
    }

    #[test]
    fn test_resolved_approver_original_approver() {
        let original = Uuid::new_v4();
        let delegate = Uuid::new_v4();

        let direct = ResolvedApprover::direct(delegate);
        assert_eq!(direct.original_approver(), delegate);

        let delegated = ResolvedApprover::delegated(delegate, vec![original]);
        assert_eq!(delegated.original_approver(), original);
    }

    #[test]
    fn test_resolved_approver_chain_length() {
        let direct = ResolvedApprover::direct(Uuid::new_v4());
        assert_eq!(direct.chain_length(), 0);

        let chain = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let delegated = ResolvedApprover::delegated(Uuid::new_v4(), chain);
        assert_eq!(delegated.chain_length(), 3);
    }

    #[test]
    fn test_delegation_type_serialization() {
        let types = vec![
            DelegationType::Temporary,
            DelegationType::Permanent,
            DelegationType::Bulk,
        ];

        for dt in types {
            let serialized = serde_json::to_string(&dt).unwrap();
            let deserialized: DelegationType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(dt, deserialized);
        }
    }

    #[test]
    fn test_new_temporary_delegation() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let starts = Utc::now();
        let ends = starts + Duration::days(7);

        let delegation = Delegation::new_temporary(
            from,
            to,
            vec!["invoice".to_string()],
            starts,
            ends,
            from,
        );

        assert_eq!(delegation.from_user_id, from);
        assert_eq!(delegation.to_user_id, to);
        assert_eq!(delegation.delegation_type, DelegationType::Temporary);
        assert_eq!(delegation.ends_at, Some(ends));
        assert!(delegation.validate().is_ok());
    }

    #[test]
    fn test_new_permanent_delegation() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        let delegation = Delegation::new_permanent(
            from,
            to,
            vec!["contract".to_string()],
            from,
        );

        assert_eq!(delegation.delegation_type, DelegationType::Permanent);
        assert_eq!(delegation.ends_at, None);
        assert!(delegation.validate().is_ok());
    }

    #[test]
    fn test_new_bulk_delegation() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        let delegation = Delegation::new_bulk(
            from,
            to,
            vec!["invoice".to_string(), "contract".to_string()],
            from,
        );

        assert_eq!(delegation.delegation_type, DelegationType::Bulk);
        assert_eq!(delegation.document_types.len(), 2);
        assert!(delegation.validate().is_ok());
    }

    #[test]
    fn test_new_for_document() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let doc_id = Uuid::new_v4();

        let delegation = Delegation::new_for_document(
            from,
            to,
            doc_id,
            Utc::now(),
            Some(Utc::now() + Duration::hours(24)),
            from,
        );

        assert_eq!(delegation.document_id, Some(doc_id));
        assert_eq!(delegation.delegation_type, DelegationType::Temporary);
        assert!(delegation.validate().is_ok());
    }

    #[test]
    fn test_delegation_validate_rejects_invalid_time_range() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let now = Utc::now();

        // End before start
        let delegation = Delegation::new_temporary(
            from,
            to,
            vec![],
            now + Duration::hours(24),
            now, // ends before starts!
            from,
        );
        assert!(delegation.validate().is_err());
    }

    #[test]
    fn test_delegation_validate_bulk_with_document_id() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let doc_id = Uuid::new_v4();

        let mut delegation = Delegation::new_bulk(from, to, vec![], from);
        delegation.document_id = Some(doc_id); // Invalid for bulk

        assert!(delegation.validate().is_err());
    }

    #[test]
    fn test_delegation_validate_permanent_with_end_time() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        let mut delegation = Delegation::new_permanent(from, to, vec![], from);
        delegation.ends_at = Some(Utc::now() + Duration::hours(24)); // Invalid for permanent

        assert!(delegation.validate().is_err());
    }

    #[test]
    fn test_delegation_validate_temporary_without_end_time() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        let mut delegation = Delegation::new_temporary(
            from,
            to,
            vec![],
            Utc::now(),
            Utc::now() + Duration::hours(24),
            from,
        );
        delegation.ends_at = None; // Invalid for temporary

        assert!(delegation.validate().is_err());
    }
}
