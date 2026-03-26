//! Configuration system for workflow definitions
//!
//! Provides YAML-based configuration loading with hot-reload support
//! for approval stages, SLA settings, and version storage policies.

pub mod workflow;
pub mod watcher;

pub use workflow::{
    WorkflowConfig, StageConfig, ApproverConfig, ApprovalTypeConfig,
    VersionStorageConfig, SlaConfig, DomainConfig,
};
pub use watcher::{ConfigWatcher, ConfigChangeEvent, ConfigError};
