//! Table-specific sync operations

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use super::EtlError;

/// Trait for table-specific sync operations
#[async_trait]
pub trait TableSync: Send + Sync {
    /// Get the table name
    fn table_name(&self) -> &str;

    /// Extract data from Supabase
    async fn extract(
        &self,
        pool: &PgPool,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Value>, EtlError>;

    /// Load data into DuckDB analytics
    async fn load(
        &self,
        data: &[Value],
    ) -> Result<usize, EtlError>;
}

/// Result of a table sync operation
#[derive(Debug, Clone)]
pub struct TableSyncResult {
    pub table: String,
    pub records_synced: usize,
    pub duration_ms: u64,
}

/// Sync implementation for information_domains
pub struct InformationDomainsSync;

#[async_trait::async_trait]
impl TableSync for InformationDomainsSync {
    fn table_name(&self) -> &str {
        "information_domains"
    }

    async fn extract(
        &self,
        _pool: &PgPool,
        _since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Value>, EtlError> {
        Ok(vec![])
    }

    async fn load(&self, _data: &[Value]) -> Result<usize, EtlError> {
        Ok(0)
    }
}

/// Sync implementation for information_objects
pub struct InformationObjectsSync;

#[async_trait::async_trait]
impl TableSync for InformationObjectsSync {
    fn table_name(&self) -> &str {
        "information_objects"
    }

    async fn extract(
        &self,
        _pool: &PgPool,
        _since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Value>, EtlError> {
        Ok(vec![])
    }

    async fn load(&self, _data: &[Value]) -> Result<usize, EtlError> {
        Ok(0)
    }
}

/// Sync implementation for documents
pub struct DocumentsSync;

#[async_trait::async_trait]
impl TableSync for DocumentsSync {
    fn table_name(&self) -> &str {
        "documents"
    }

    async fn extract(
        &self,
        _pool: &PgPool,
        _since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Value>, EtlError> {
        Ok(vec![])
    }

    async fn load(&self, _data: &[Value]) -> Result<usize, EtlError> {
        Ok(0)
    }
}

/// Sync implementation for templates
pub struct TemplatesSync;

#[async_trait::async_trait]
impl TableSync for TemplatesSync {
    fn table_name(&self) -> &str {
        "templates"
    }

    async fn extract(
        &self,
        _pool: &PgPool,
        _since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Value>, EtlError> {
        Ok(vec![])
    }

    async fn load(&self, _data: &[Value]) -> Result<usize, EtlError> {
        Ok(0)
    }
}

/// Sync implementation for audit_trail
pub struct AuditTrailSync;

#[async_trait::async_trait]
impl TableSync for AuditTrailSync {
    fn table_name(&self) -> &str {
        "audit_trail"
    }

    async fn extract(
        &self,
        _pool: &PgPool,
        _since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Value>, EtlError> {
        Ok(vec![])
    }

    async fn load(&self, _data: &[Value]) -> Result<usize, EtlError> {
        Ok(0)
    }
}
