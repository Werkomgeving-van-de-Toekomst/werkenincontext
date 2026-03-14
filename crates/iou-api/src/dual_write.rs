//! Dual-Write Pattern for DuckDB + Supabase
//!
//! Implements a dual-write strategy that writes to both databases
//! simultaneously with configurable read source for gradual migration.

use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;

use super::{db::Database, supabase::SupabasePool};

/// Read source selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadSource {
    DuckDb,
    Supabase,
}

impl ReadSource {
    /// Get current read source from environment
    pub fn from_env() -> Self {
        match std::env::var("READ_SOURCE").as_deref() {
            Ok("supabase") => ReadSource::Supabase,
            _ => ReadSource::DuckDb,  // Default to DuckDB for safety
        }
    }

    /// Get the name of this read source
    pub fn name(&self) -> &'static str {
        match self {
            ReadSource::DuckDb => "duckdb",
            ReadSource::Supabase => "supabase",
        }
    }
}

/// Result of a dual-write operation
#[derive(Debug)]
pub enum DualWriteResult<T> {
    Success(T),
    PartialSuccess {
        duckdb: Option<T>,
        supabase: Option<T>,
        errors: Vec<anyhow::Error>,
    },
    Failed(Vec<anyhow::Error>),
}

impl<T> DualWriteResult<T> {
    /// Check if both writes succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, DualWriteResult::Success(_))
    }

    /// Get the value if successful, or the best available
    pub fn value(self) -> Option<T> {
        match self {
            DualWriteResult::Success(v) => Some(v),
            DualWriteResult::PartialSuccess { duckdb, supabase, .. } => {
                duckdb.or(supabase)
            }
            DualWriteResult::Failed(_) => None,
        }
    }

    /// Get the number of successful writes
    pub fn success_count(&self) -> usize {
        match self {
            DualWriteResult::Success(_) => 2,
            DualWriteResult::PartialSuccess { duckdb, supabase, .. } => {
                duckdb.is_some() as usize + supabase.is_some() as usize
            }
            DualWriteResult::Failed(_) => 0,
        }
    }
}

/// Trait for types that support dual-write
#[async_trait]
pub trait DualWrite: Send + Sync {
    type Id: Send + Sync + PartialEq + std::fmt::Display + Clone;

    /// Write to DuckDB
    async fn write_to_duckdb(&self, db: &Database) -> Result<Self::Id>;

    /// Write to Supabase
    async fn write_to_supabase(&self, db: &SupabasePool) -> Result<Self::Id>;

    /// Dual-write to both databases
    async fn dual_write(
        &self,
        duckdb: &Database,
        supabase: &SupabasePool,
    ) -> DualWriteResult<Self::Id> {
        let (duckdb_result, supabase_result) = tokio::join!(
            self.write_to_duckdb(duckdb),
            self.write_to_supabase(supabase)
        );

        match (duckdb_result, supabase_result) {
            (Ok(d_id), Ok(s_id)) => {
                // Verify IDs match (should be the same UUID)
                if d_id != s_id {
                    let error = anyhow::anyhow!(
                        "ID mismatch in dual-write: DuckDB={}, Supabase={}",
                        d_id, s_id
                    );
                    tracing::error!("{}", error);
                    return DualWriteResult::Failed(vec![error]);
                }
                DualWriteResult::Success(d_id)
            }
            (Ok(d_id), Err(e)) => DualWriteResult::PartialSuccess {
                duckdb: Some(d_id),
                supabase: None,
                errors: vec![e],
            },
            (Err(e), Ok(s_id)) => DualWriteResult::PartialSuccess {
                duckdb: None,
                supabase: Some(s_id),
                errors: vec![e],
            },
            (Err(e1), Err(e2)) => DualWriteResult::Failed(vec![e1, e2]),
        }
    }

    /// Update in DuckDB
    async fn update_in_duckdb(&self, db: &Database) -> Result<Self::Id>;

    /// Update in Supabase
    async fn update_in_supabase(&self, db: &SupabasePool) -> Result<Self::Id>;

    /// Dual-update to both databases
    async fn dual_update(
        &self,
        duckdb: &Database,
        supabase: &SupabasePool,
    ) -> DualWriteResult<Self::Id> {
        let (duckdb_result, supabase_result) = tokio::join!(
            self.update_in_duckdb(duckdb),
            self.update_in_supabase(supabase)
        );

        match (duckdb_result, supabase_result) {
            (Ok(d_id), Ok(_)) => DualWriteResult::Success(d_id),
            (Ok(d_id), Err(e)) => DualWriteResult::PartialSuccess {
                duckdb: Some(d_id),
                supabase: None,
                errors: vec![e],
            },
            (Err(e), Ok(s_id)) => DualWriteResult::PartialSuccess {
                duckdb: None,
                supabase: Some(s_id),
                errors: vec![e],
            },
            (Err(e1), Err(e2)) => DualWriteResult::Failed(vec![e1, e2]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_source_from_env_default() {
        // Clear the env var if set
        let _ = unsafe { std::env::set_var("READ_SOURCE", "") };
        let source = ReadSource::from_env();
        assert_eq!(source, ReadSource::DuckDb);
    }

    #[test]
    fn test_read_source_from_env_supabase() {
        unsafe { std::env::set_var("READ_SOURCE", "supabase") };
        let source = ReadSource::from_env();
        assert_eq!(source, ReadSource::Supabase);
        unsafe { std::env::remove_var("READ_SOURCE") };
    }

    #[test]
    fn test_read_source_name() {
        assert_eq!(ReadSource::DuckDb.name(), "duckdb");
        assert_eq!(ReadSource::Supabase.name(), "supabase");
    }
}
