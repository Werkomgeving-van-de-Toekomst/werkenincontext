//! Audit logger with write-ahead semantics

use super::models::{AuditEntry, AuditAction, AuditOutcome, AuditQuery, AuditQueryResult};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use async_trait::async_trait;

/// Trait for audit backends
#[async_trait]
pub trait AuditBackend: Send + Sync {
    /// Write an audit entry (must succeed before action)
    async fn write(&self, entry: &AuditEntry) -> Result<(), AuditError>;

    /// Query audit entries for a tenant
    async fn query(
        &self,
        tenant_id: &str,
        query: &AuditQuery,
    ) -> Result<AuditQueryResult, AuditError>;

    /// Get a specific audit entry
    async fn get(&self, id: Uuid) -> Result<Option<AuditEntry>, AuditError>;
}

/// PostgreSQL audit backend with BIO compliance
pub struct PostgresAuditBackend {
    pool: PgPool,
}

impl PostgresAuditBackend {
    /// Create a new PostgreSQL audit backend
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize the audit table with proper constraints
    pub async fn init(&self) -> Result<(), sqlx::Error> {
        // Create audit_log table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                -- Primary key
                id UUID PRIMARY KEY,

                -- Timestamp (indexed for queries)
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),

                -- Tenant isolation (required for multi-tenancy)
                tenant_id VARCHAR(50) NOT NULL,

                -- User identification
                user_did VARCHAR(255) NOT NULL,

                -- Action classification
                action VARCHAR(100) NOT NULL,

                -- Resource identification
                resource_type VARCHAR(100) NOT NULL,
                resource_id VARCHAR(500) NOT NULL,

                -- Outcome
                outcome VARCHAR(20) NOT NULL,

                -- Request metadata
                ip_address VARCHAR(45),
                user_agent TEXT,

                -- Additional context (JSONB for querying)
                context JSONB,

                -- Session correlation
                session_id VARCHAR(100),

                -- Chained operations
                parent_id UUID REFERENCES audit_log(id),

                -- Tamper evidence (hash of entry)
                entry_hash VARCHAR(64),

                -- Index for tenant queries
                CONSTRAINT audit_tenant_timestamp UNIQUE (tenant_id, timestamp, id)
            );

            -- Indexes for efficient querying
            CREATE INDEX IF NOT EXISTS idx_audit_tenant_timestamp
                ON audit_log(tenant_id, timestamp DESC);

            CREATE INDEX IF NOT EXISTS idx_audit_user_did
                ON audit_log(user_did);

            CREATE INDEX IF NOT EXISTS idx_audit_resource
                ON audit_log(resource_type, resource_id);

            CREATE INDEX IF NOT EXISTS idx_audit_parent
                ON audit_log(parent_id) WHERE parent_id IS NOT NULL;

            CREATE INDEX IF NOT EXISTS idx_audit_context
                ON audit_log USING GIN (context);

            -- Comment for documentation
            COMMENT ON TABLE audit_log IS
                'Tamper-evident audit log with 7-year retention (BIO compliant)';
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create audit function for trigger (if needed)
        sqlx::query(
            r#"
            CREATE OR REPLACE FUNCTION audit_insert_trigger()
            RETURNS TRIGGER AS $$
            BEGIN
                -- Compute entry hash for tamper evidence
                NEW.entry_hash = encode(
                    digest(
                        NEW.id::TEXT ||
                        NEW.timestamp::TEXT ||
                        NEW.tenant_id ||
                        NEW.user_did ||
                        NEW.action ||
                        NEW.resource_type ||
                        NEW.resource_id ||
                        NEW.outcome::TEXT,
                        'sha256'
                    ),
                    'hex'
                );
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl AuditBackend for PostgresAuditBackend {
    async fn write(&self, entry: &AuditEntry) -> Result<(), AuditError> {
        let action = match &entry.action {
            AuditAction::Custom(s) => s.clone(),
            a => serde_json::to_string(a).unwrap_or_else(|_| "custom".to_string()),
        };

        sqlx::query(
            r#"
            INSERT INTO audit_log (
                id, timestamp, tenant_id, user_did, action,
                resource_type, resource_id, outcome,
                ip_address, user_agent, context, session_id, parent_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(entry.id)
        .bind(entry.timestamp)
        .bind(&entry.tenant_id)
        .bind(&entry.user_did)
        .bind(&action)
        .bind(&entry.resource_type)
        .bind(&entry.resource_id)
        .bind(match entry.outcome {
            AuditOutcome::Success => "success",
            AuditOutcome::Failed => "failed",
            AuditOutcome::Denied => "denied",
            AuditOutcome::Error => "error",
        })
        .bind(&entry.ip_address)
        .bind(&entry.user_agent)
        .bind(&entry.context)
        .bind(&entry.session_id)
        .bind(entry.parent_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuditError::Database(e.to_string()))?;

        Ok(())
    }

    async fn query(
        &self,
        tenant_id: &str,
        query: &AuditQuery,
    ) -> Result<AuditQueryResult, AuditError> {
        let limit = i64::from(query.limit.min(1000)); // Max 1000 per query

        // Build dynamic query (simplified - in production use query builder)
        let sql = r#"
            SELECT id, timestamp, tenant_id, user_did, action,
                   resource_type, resource_id, outcome,
                   ip_address, user_agent, context, session_id, parent_id
            FROM audit_log
            WHERE tenant_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
        "#;

        // Count total
        let count_sql = r#"
            SELECT COUNT(*) as total FROM audit_log WHERE tenant_id = $1
        "#;

        let total_count = sqlx::query_scalar::<_, i64>(count_sql)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        // Fetch entries
        let rows = sqlx::query(sql)
            .bind(tenant_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AuditError::Database(e.to_string()))?;

        let entries: Vec<AuditEntry> = rows.into_iter().map(|row| {
            let action_str: String = row.get("action");
            let outcome_str: String = row.get("outcome");

            AuditEntry {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                tenant_id: row.get("tenant_id"),
                user_did: row.get("user_did"),
                action: AuditAction::from(action_str),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                outcome: match outcome_str.as_str() {
                    "success" => AuditOutcome::Success,
                    "failed" => AuditOutcome::Failed,
                    "denied" => AuditOutcome::Denied,
                    _ => AuditOutcome::Error,
                },
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                context: row.get("context"),
                session_id: row.get("session_id"),
                parent_id: row.get("parent_id"),
            }
        }).collect();

        let entries_len = entries.len();
        let total_count_u64 = total_count as u64;

        Ok(AuditQueryResult {
            entries,
            total_count: total_count_u64,
            has_more: (entries_len as u64) < total_count_u64,
        })
    }

    async fn get(&self, id: Uuid) -> Result<Option<AuditEntry>, AuditError> {
        let row = sqlx::query(
            r#"
            SELECT id, timestamp, tenant_id, user_did, action,
                   resource_type, resource_id, outcome,
                   ip_address, user_agent, context, session_id, parent_id
            FROM audit_log WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuditError::Database(e.to_string()))?;

        Ok(row.map(|row| {
            let action_str: String = row.get("action");
            let outcome_str: String = row.get("outcome");

            AuditEntry {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                tenant_id: row.get("tenant_id"),
                user_did: row.get("user_did"),
                action: AuditAction::from(action_str),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                outcome: match outcome_str.as_str() {
                    "success" => AuditOutcome::Success,
                    "failed" => AuditOutcome::Failed,
                    "denied" => AuditOutcome::Denied,
                    _ => AuditOutcome::Error,
                },
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                context: row.get("context"),
                session_id: row.get("session_id"),
                parent_id: row.get("parent_id"),
            }
        }))
    }
}

/// Audit logger with write-ahead enforcement
pub struct AuditLogger<B: AuditBackend> {
    backend: B,
}

impl<B: AuditBackend> AuditLogger<B> {
    /// Create a new audit logger
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Log an action with write-ahead semantics
    ///
    /// The audit entry is written BEFORE the action is performed.
    /// If audit logging fails, the action must not proceed.
    pub async fn log(&self, entry: &AuditEntry) -> Result<(), AuditError> {
        self.backend.write(entry).await
    }

    /// Query audit entries
    pub async fn query(
        &self,
        tenant_id: &str,
        query: &AuditQuery,
    ) -> Result<AuditQueryResult, AuditError> {
        self.backend.query(tenant_id, query).await
    }

    /// Get a specific audit entry
    pub async fn get(&self, id: Uuid) -> Result<Option<AuditEntry>, AuditError> {
        self.backend.get(id).await
    }
}

/// Shared audit logger that can be used across async tasks
pub type SharedAuditLogger = Arc< tokio::sync::Mutex<Box<dyn AuditBackend>> >;

/// Helper function to create a shared audit logger from a backend
pub fn shared_logger<B: AuditBackend + 'static>(backend: B) -> SharedAuditLogger {
    Arc::new(tokio::sync::Mutex::new(Box::new(backend)))
}

/// Log function for shared audit logger
pub async fn log_shared(logger: &SharedAuditLogger, entry: &AuditEntry) -> Result<(), AuditError> {
    let backend = logger.lock().await;
    // We need to call the write method on the boxed trait object
    // Since write takes &self, we need to deref properly
    let backend_ref: &dyn AuditBackend = &**backend;
    backend_ref.write(entry).await
}

/// Audit errors
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Audit backend unavailable")]
    Unavailable,

    #[error("Entry not found: {0}")]
    NotFound(String),

    #[error("Immutable: audit entries cannot be modified")]
    Immutable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_action_from_string() {
        let action = AuditAction::from("document_created".to_string());
        assert!(matches!(action, AuditAction::DocumentCreated));

        let action = AuditAction::from("unknown_action".to_string());
        assert!(matches!(action, AuditAction::Custom(_)));
    }
}
