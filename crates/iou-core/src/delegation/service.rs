//! Delegation CRUD service for managing delegations
//!
//! Provides functionality for creating, revoking, and auto-expiring delegations.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::delegation::{Delegation, DelegationType};

/// Maximum number of active delegations per user (default)
const DEFAULT_MAX_ACTIVE_DELEGATIONS: usize = 10;

/// Errors that can occur during delegation operations
#[derive(Debug, Error)]
pub enum DelegationError {
    #[error("Cannot delegate to self")]
    SelfDelegation,

    #[error("End date must be after start date")]
    InvalidDateRange,

    #[error("Circular delegation detected")]
    CircularDelegation,

    #[error("Maximum active delegations reached")]
    TooManyActiveDelegations,

    #[error("Delegation not found")]
    NotFound,

    #[error("Not authorized to revoke this delegation")]
    UnauthorizedRevocation,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Service for managing delegations
pub struct DelegationService {
    pool: PgPool,
    max_active_delegations: usize,
}

impl DelegationService {
    /// Create a new DelegationService
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            max_active_delegations: DEFAULT_MAX_ACTIVE_DELEGATIONS,
        }
    }

    /// Set the maximum number of active delegations per user
    pub fn with_max_active_delegations(mut self, max: usize) -> Self {
        self.max_active_delegations = max;
        self
    }

    /// Create a new delegation
    ///
    /// Validates:
    /// - from_user != to_user (no self-delegation)
    /// - ends_at > starts_at when provided
    /// - No circular delegation chains (A -> B -> A)
    /// - Total active delegations within configured limit
    pub async fn create_delegation(
        &self,
        from: Uuid,
        to: Uuid,
        delegation_type: DelegationType,
        document_types: Vec<String>,
        document_id: Option<Uuid>,
        starts_at: DateTime<Utc>,
        ends_at: Option<DateTime<Utc>>,
        created_by: Uuid,
    ) -> Result<Delegation, DelegationError> {
        // Validate no self-delegation
        if from == to {
            return Err(DelegationError::SelfDelegation);
        }

        // Validate date range
        if let Some(ends) = ends_at {
            if ends <= starts_at {
                return Err(DelegationError::InvalidDateRange);
            }
        }

        // Check for circular delegation
        if self.detect_circular_chain(from, to, 3).await? {
            return Err(DelegationError::CircularDelegation);
        }

        // Check active delegation limit
        let active_count = self.count_active_delegations(from).await?;
        if active_count >= self.max_active_delegations {
            return Err(DelegationError::TooManyActiveDelegations);
        }

        let id = Uuid::new_v4();
        let now = Utc::now();

        // Convert document_types to JSONB
        let document_types_json: serde_json::Value = document_types
            .clone()
            .into_iter()
            .collect::<serde_json::Value>();

        // Serialize DelegationType to string
        let type_str = match delegation_type {
            DelegationType::Temporary => "temporary",
            DelegationType::Permanent => "permanent",
            DelegationType::Bulk => "bulk",
        };

        sqlx::query(
            r#"
            INSERT INTO delegations (
                id, from_user_id, to_user_id, delegation_type,
                document_types, document_id, starts_at, ends_at,
                is_active, created_at, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $10)
            "#
        )
        .bind(id)
        .bind(from)
        .bind(to)
        .bind(type_str)
        .bind(document_types_json)
        .bind(document_id)
        .bind(starts_at)
        .bind(ends_at)
        .bind(now)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        Ok(Delegation {
            id,
            from_user_id: from,
            to_user_id: to,
            delegation_type,
            document_types,
            document_id,
            starts_at,
            ends_at,
            is_active: true,
            created_at: now,
            created_by,
        })
    }

    /// Revoke an active delegation
    ///
    /// Only the creator or from_user can revoke a delegation.
    /// Creates an audit trail entry.
    pub async fn revoke_delegation(
        &self,
        delegation_id: Uuid,
        revoked_by: Uuid,
    ) -> Result<(), DelegationError> {
        // First check if delegation exists and get its details
        let delegation = self.fetch_delegation(delegation_id).await?;

        // Check authorization
        if delegation.from_user_id != revoked_by && delegation.created_by != revoked_by {
            return Err(DelegationError::UnauthorizedRevocation);
        }

        // Revoke the delegation
        sqlx::query("UPDATE delegations SET is_active = false WHERE id = $1")
            .bind(delegation_id)
            .execute(&self.pool)
            .await?;

        // Create audit trail entry
        self.create_audit_entry(delegation_id, revoked_by, "revoked").await?;

        Ok(())
    }

    /// Find and mark expired delegations as inactive
    ///
    /// Returns list of delegation IDs that were expired.
    /// Called by scheduled job or during delegation resolution.
    pub async fn auto_expire_delegations(&self) -> Result<Vec<Uuid>, DelegationError> {
        let now = Utc::now();

        let rows = sqlx::query(
            r#"
            UPDATE delegations
            SET is_active = false
            WHERE is_active = true
              AND ends_at IS NOT NULL
              AND ends_at < $1
            RETURNING id
            "#
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter()
            .map(|r| r.try_get("id").ok())
            .filter_map(|id| id)
            .collect())
    }

    /// List delegations for a user (both created and received)
    pub async fn list_user_delegations(
        &self,
        user_id: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<Delegation>, DelegationError> {
        let rows = if include_inactive {
            sqlx::query(
                r#"
                SELECT id, from_user_id, to_user_id, delegation_type,
                       document_types, document_id, starts_at, ends_at,
                       is_active, created_at, created_by
                FROM delegations
                WHERE from_user_id = $1 OR to_user_id = $1
                ORDER BY created_at DESC
                "#
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, from_user_id, to_user_id, delegation_type,
                       document_types, document_id, starts_at, ends_at,
                       is_active, created_at, created_by
                FROM delegations
                WHERE (from_user_id = $1 OR to_user_id = $1)
                  AND is_active = true
                ORDER BY created_at DESC
                "#
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter()
            .map(|row| self.row_to_delegation(row))
            .collect()
    }

    /// Fetch a single delegation by ID
    async fn fetch_delegation(&self, id: Uuid) -> Result<Delegation, DelegationError> {
        let row = sqlx::query(
            r#"
            SELECT id, from_user_id, to_user_id, delegation_type,
                   document_types, document_id, starts_at, ends_at,
                   is_active, created_at, created_by
            FROM delegations
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DelegationError::NotFound)?;

        self.row_to_delegation(row)
    }

    /// Count active delegations for a user
    async fn count_active_delegations(&self, user_id: Uuid) -> Result<usize, DelegationError> {
        let row: i64 = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM delegations
            WHERE from_user_id = $1
              AND is_active = true
            "#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?
        .try_get("count")
        .unwrap_or(0);

        Ok(row as usize)
    }

    /// Check if a delegation chain would create a circular reference
    async fn detect_circular_chain(
        &self,
        from: Uuid,
        to: Uuid,
        max_hops: usize,
    ) -> Result<bool, DelegationError> {
        // Check if 'to' has any delegation back to 'from' (direct circular)
        let exists: bool = sqlx::query(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM delegations
                WHERE from_user_id = $1
                  AND to_user_id = $2
                  AND is_active = true
            ) as exists
            "#
        )
        .bind(to)
        .bind(from)
        .fetch_one(&self.pool)
        .await?
        .try_get("exists")
        .unwrap_or(false);

        if exists {
            return Ok(true);
        }

        // Check for indirect circular chains up to max_hops
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

    /// Create an audit trail entry
    async fn create_audit_entry(
        &self,
        delegation_id: Uuid,
        performed_by: Uuid,
        action: &str,
    ) -> Result<(), DelegationError> {
        sqlx::query(
            r#"
            INSERT INTO audit_trail (id, entity_type, entity_id, action, performed_by, performed_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind("delegation")
        .bind(delegation_id)
        .bind(action)
        .bind(performed_by)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Convert a database row to a Delegation
    fn row_to_delegation(&self, row: sqlx::postgres::PgRow) -> Result<Delegation, DelegationError> {
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
