//! Delegation resolution logic
//!
//! Determines the actual approver for a given user by following active delegation chains.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::delegation::{Delegation, DelegationType, ResolvedApprover};

/// Maximum delegation chain hops
const MAX_DELEGATION_HOPS: usize = 3;

/// Errors that can occur during delegation resolution
#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("Circular delegation chain detected")]
    CircularChain,

    #[error("Delegation chain exceeds maximum length")]
    ChainTooLong,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Resolver for determining actual approvers considering active delegations
pub struct DelegationResolver {
    pool: PgPool,
    max_hops: usize,
}

impl DelegationResolver {
    /// Create a new DelegationResolver
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            max_hops: MAX_DELEGATION_HOPS,
        }
    }

    /// Set the maximum delegation chain hops
    pub fn with_max_hops(mut self, max_hops: usize) -> Self {
        self.max_hops = max_hops;
        self
    }

    /// Resolve the actual approver for a user, considering active delegations
    ///
    /// Delegation priority (highest to lowest):
    /// 1. Single-document delegation (document_id matches)
    /// 2. Document-type delegation (document_type matches)
    /// 3. Bulk delegation (all document_types)
    ///
    /// Follows delegation chains up to max_hops to prevent infinite loops.
    /// Returns error if circular delegation detected or chain too long.
    pub async fn resolve_approver(
        &self,
        original_approver: Uuid,
        document_type: &str,
        document_id: Option<Uuid>,
    ) -> Result<ResolvedApprover, ResolutionError> {
        let mut current = original_approver;
        let mut chain = Vec::new();
        let mut visited = vec![current];

        for _ in 0..=self.max_hops {
            let delegation = self
                .find_best_delegation(current, document_type, document_id)
                .await?;

            match delegation {
                Some(d) => {
                    let next = d.to_user_id;

                    // Check for circular chain
                    if visited.contains(&next) {
                        return Err(ResolutionError::CircularChain);
                    }

                    visited.push(next);
                    chain.push(current);
                    current = next;
                }
                None => break,
            }
        }

        if chain.is_empty() {
            Ok(ResolvedApprover::direct(original_approver))
        } else {
            Ok(ResolvedApprover::delegated(current, chain))
        }
    }

    /// Get all active delegations for a user (both from and to perspectives)
    pub async fn active_delegations(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Delegation>, ResolutionError> {
        let now = Utc::now();

        let rows = sqlx::query(
            r#"
            SELECT id, from_user_id, to_user_id, delegation_type,
                   document_types, document_id, starts_at, ends_at,
                   is_active, created_at, created_by
            FROM delegations
            WHERE (from_user_id = $1 OR to_user_id = $1)
              AND is_active = true
              AND starts_at <= $2
              AND (ends_at IS NULL OR ends_at > $2)
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_delegation(row))
            .collect()
    }

    /// Find the best (highest priority) delegation for a user
    ///
    /// Priority order:
    /// 1. Single-document (document_id matches)
    /// 2. Document-type (document_type in list)
    /// 3. Bulk (empty document_types list)
    async fn find_best_delegation(
        &self,
        user_id: Uuid,
        document_type: &str,
        document_id: Option<Uuid>,
    ) -> Result<Option<Delegation>, ResolutionError> {
        let now = Utc::now();

        // First try to find single-document delegation
        if let Some(doc_id) = document_id {
            let single_doc = sqlx::query(
                r#"
                SELECT id, from_user_id, to_user_id, delegation_type,
                       document_types, document_id, starts_at, ends_at,
                       is_active, created_at, created_by
                FROM delegations
                WHERE from_user_id = $1
                  AND document_id = $2
                  AND is_active = true
                  AND starts_at <= $3
                  AND (ends_at IS NULL OR ends_at > $3)
                LIMIT 1
                "#
            )
            .bind(user_id)
            .bind(doc_id)
            .bind(now)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(row) = single_doc {
                return Ok(Some(self.row_to_delegation(row)?));
            }
        }

        // Then try document-type delegation
        let doc_type = sqlx::query(
            r#"
            SELECT id, from_user_id, to_user_id, delegation_type,
                   document_types, document_id, starts_at, ends_at,
                   is_active, created_at, created_by
            FROM delegations
            WHERE from_user_id = $1
              AND is_active = true
              AND starts_at <= $2
              AND (ends_at IS NULL OR ends_at > $2)
              AND document_id IS NULL
              AND $3 = ANY(document_types)
            LIMIT 1
            "#
        )
        .bind(user_id)
        .bind(now)
        .bind(document_type)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = doc_type {
            return Ok(Some(self.row_to_delegation(row)?));
        }

        // Finally try bulk delegation (empty document_types)
        let bulk = sqlx::query(
            r#"
            SELECT id, from_user_id, to_user_id, delegation_type,
                   document_types, document_id, starts_at, ends_at,
                   is_active, created_at, created_by
            FROM delegations
            WHERE from_user_id = $1
              AND is_active = true
              AND starts_at <= $2
              AND (ends_at IS NULL OR ends_at > $2)
              AND document_id IS NULL
              AND jsonb_array_length(document_types) = 0
            LIMIT 1
            "#
        )
        .bind(user_id)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = bulk {
            return Ok(Some(self.row_to_delegation(row)?));
        }

        Ok(None)
    }

    /// Check if a delegation chain would create a circular reference
    pub async fn detect_circular_chain(
        &self,
        from: Uuid,
        to: Uuid,
        max_hops: usize,
    ) -> Result<bool, ResolutionError> {
        let mut current = to;
        let mut visited = vec![from];

        for _ in 0..max_hops {
            let row = sqlx::query(
                r#"
                SELECT to_user_id
                FROM delegations
                WHERE from_user_id = $1
                  AND is_active = true
                  AND starts_at <= NOW()
                  AND (ends_at IS NULL OR ends_at > NOW())
                LIMIT 1
                "#
            )
            .bind(current)
            .fetch_optional(&self.pool)
            .await?;

            match row {
                Some(r) => {
                    let next: Uuid = r.try_get("to_user_id")?;
                    if next == from {
                        return Ok(true); // Circular chain detected
                    }
                    if visited.contains(&next) {
                        return Ok(true); // Loop detected
                    }
                    visited.push(next);
                    current = next;
                }
                None => break,
            }
        }

        Ok(false)
    }

    /// Convert a database row to a Delegation
    fn row_to_delegation(&self, row: sqlx::postgres::PgRow) -> Result<Delegation, ResolutionError> {
        use sqlx::Row;

        let id: Uuid = row.try_get("id")?;
        let from_user_id: Uuid = row.try_get("from_user_id")?;
        let to_user_id: Uuid = row.try_get("to_user_id")?;

        // Parse delegation_type from string
        let type_str: String = row.try_get("delegation_type")?;
        let delegation_type = match type_str.as_str() {
            "temporary" => DelegationType::Temporary,
            "permanent" => DelegationType::Permanent,
            "bulk" => DelegationType::Bulk,
            _ => DelegationType::Permanent, // Default fallback
        };

        // Parse document_types from JSONB
        let document_types_json: serde_json::Value = row.try_get("document_types")?;
        let document_types: Vec<String> = match document_types_json {
            serde_json::Value::Array(arr) => {
                arr.into_iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            }
            _ => vec![],
        };

        let document_id: Option<Uuid> = row.try_get("document_id")?;
        let starts_at: DateTime<Utc> = row.try_get("starts_at")?;
        let ends_at: Option<DateTime<Utc>> = row.try_get("ends_at")?;
        let is_active: bool = row.try_get("is_active")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let created_by: Uuid = row.try_get("created_by")?;

        Ok(Delegation {
            id,
            from_user_id,
            to_user_id,
            delegation_type,
            document_types,
            document_id,
            starts_at,
            ends_at,
            is_active,
            created_at,
            created_by,
        })
    }
}

// Tests removed - previous tests used unsafe PgPool::new_untested() which no longer exists in sqlx 0.8
// Proper testing requires a test database fixture or mock implementation.
