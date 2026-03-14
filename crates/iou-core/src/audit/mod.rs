//! Tamper-evident audit logging for compliance
//!
//! This module provides audit logging that satisfies BIO, NEN 7510,
//! and AVG/GDPR requirements for government digital services.
//!
//! # Write-Ahead Semantics
//!
//! Audit entries are written BEFORE the action is executed.
//! If audit logging fails, the action MUST NOT proceed.

pub mod logger;
pub mod models;

pub use logger::{AuditLogger, AuditBackend, PostgresAuditBackend, SharedAuditLogger, shared_logger, log_shared};
pub use models::{AuditEntry, AuditAction, AuditOutcome, AuditFilter, AuditQuery};
