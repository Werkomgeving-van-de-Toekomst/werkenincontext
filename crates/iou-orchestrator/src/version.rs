//! Workflow versioning for compatibility checks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVersion {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub definition_hash: String,
    pub compatible_since: DateTime<Utc>,
    pub deprecated_at: Option<DateTime<Utc>>,
    pub migration_path: HashMap<String, String>, // from_version -> migration_script
}

impl WorkflowVersion {
    /// Create a new workflow version
    pub fn new(version: String, definition_hash: String) -> Self {
        let now = Utc::now();
        Self {
            version,
            created_at: now,
            definition_hash,
            compatible_since: now,
            deprecated_at: None,
            migration_path: HashMap::new(),
        }
    }

    /// Create version 1.0.0 with a hash
    pub fn v1(hash: String) -> Self {
        Self::new("1.0.0".to_string(), hash)
    }

    /// Add a migration path from another version
    pub fn with_migration(mut self, from: String, script: String) -> Self {
        self.migration_path.insert(from, script);
        self
    }

    /// Mark this version as deprecated
    pub fn deprecate(&mut self) {
        self.deprecated_at = Some(Utc::now());
    }

    /// Check if this version is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.deprecated_at.is_some()
    }

    /// Get the migration script for a given source version
    pub fn get_migration(&self, from_version: &str) -> Option<&String> {
        self.migration_path.get(from_version)
    }
}

/// Version compatibility check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionCompatibility {
    /// Versions are identical, fully compatible
    Identical,
    /// Versions are different but compatible
    Compatible,
    /// Versions are incompatible, migration required
    MigrationRequired { from: String, to: String },
    /// Versions are incompatible and no migration available
    Incompatible { from: String, to: String },
}

/// Version registry for managing workflow versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRegistry {
    current_version: String,
    versions: HashMap<String, WorkflowVersion>,
}

impl VersionRegistry {
    /// Create a new version registry
    pub fn new(current_version: String) -> Self {
        Self {
            current_version,
            versions: HashMap::new(),
        }
    }

    /// Register a workflow version
    pub fn register(&mut self, version: WorkflowVersion) {
        let version_str = version.version.clone();
        self.versions.insert(version_str, version);
    }

    /// Get a version by string
    pub fn get(&self, version: &str) -> Option<&WorkflowVersion> {
        self.versions.get(version)
    }

    /// Get the current version
    pub fn current(&self) -> &WorkflowVersion {
        self.versions.get(&self.current_version)
            .expect("Current version should always be registered")
    }

    /// Check compatibility between two versions
    pub fn check_compatibility(&self, from: &str, to: &str) -> VersionCompatibility {
        if from == to {
            return VersionCompatibility::Identical;
        }

        match self.get(from) {
            Some(v) => v,
            None => return VersionCompatibility::Incompatible {
                from: from.to_string(),
                to: to.to_string(),
            },
        };

        let to_version = match self.get(to) {
            Some(v) => v,
            None => return VersionCompatibility::Incompatible {
                from: from.to_string(),
                to: to.to_string(),
            },
        };

        // Check if to version has a migration path from from version
        if let Some(_) = to_version.get_migration(from) {
            return VersionCompatibility::MigrationRequired {
                from: from.to_string(),
                to: to.to_string(),
            };
        }

        // If both versions exist and no migration is defined,
        // we check if they're compatible by major version
        let from_major = Self::extract_major(from);
        let to_major = Self::extract_major(to);

        if from_major == to_major {
            VersionCompatibility::Compatible
        } else {
            VersionCompatibility::Incompatible {
                from: from.to_string(),
                to: to.to_string(),
            }
        }
    }

    /// Extract major version number
    fn extract_major(version: &str) -> Option<u32> {
        version
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
    }

    /// Migrate a checkpoint from one version to another
    pub fn migrate_checkpoint(
        &self,
        checkpoint_json: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<String, String> {
        let compatibility = self.check_compatibility(from_version, to_version);

        match compatibility {
            VersionCompatibility::Identical => Ok(checkpoint_json.to_string()),
            VersionCompatibility::Compatible => Ok(checkpoint_json.to_string()),
            VersionCompatibility::MigrationRequired { .. } => {
                // In a real implementation, this would apply the migration script
                // For now, we'll return the JSON as-is (no-op migration)
                tracing::warn!(
                    "Migration from {} to {} requested but not fully implemented",
                    from_version, to_version
                );
                Ok(checkpoint_json.to_string())
            }
            VersionCompatibility::Incompatible { .. } => {
                Err(format!("Cannot migrate from {} to {}", from_version, to_version))
            }
        }
    }

    /// Set the current version
    pub fn set_current(&mut self, version: String) {
        if self.versions.contains_key(&version) {
            self.current_version = version;
        } else {
            panic!("Cannot set current version to unregistered version: {}", version);
        }
    }
}

impl Default for VersionRegistry {
    fn default() -> Self {
        let mut registry = Self::new("1.0.0".to_string());
        registry.register(WorkflowVersion::v1("initial-v1".to_string()));
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_version_creation() {
        let version = WorkflowVersion::v1("abc123".to_string());
        assert_eq!(version.version, "1.0.0");
        assert_eq!(version.definition_hash, "abc123");
        assert!(!version.is_deprecated());
    }

    #[test]
    fn test_workflow_version_deprecation() {
        let mut version = WorkflowVersion::v1("abc123".to_string());
        assert!(!version.is_deprecated());
        version.deprecate();
        assert!(version.is_deprecated());
    }

    #[test]
    fn test_version_registry_default() {
        let registry = VersionRegistry::default();
        assert_eq!(registry.current().version, "1.0.0");
    }

    #[test]
    fn test_version_check_identical() {
        let registry = VersionRegistry::default();
        let compat = registry.check_compatibility("1.0.0", "1.0.0");
        assert_eq!(compat, VersionCompatibility::Identical);
    }

    #[test]
    fn test_version_check_compatible() {
        let mut registry = VersionRegistry::default();
        registry.register(WorkflowVersion::new("1.1.0".to_string(), "hash2".to_string()));

        let compat = registry.check_compatibility("1.0.0", "1.1.0");
        // Same major version should be compatible
        assert_eq!(compat, VersionCompatibility::Compatible);
    }

    #[test]
    fn test_version_check_incompatible() {
        let mut registry = VersionRegistry::default();
        registry.register(WorkflowVersion::new("2.0.0".to_string(), "hash2".to_string()));

        let compat = registry.check_compatibility("1.0.0", "2.0.0");
        assert!(matches!(compat, VersionCompatibility::Incompatible { .. }));
    }

    #[test]
    fn test_migration_path() {
        let mut version = WorkflowVersion::new("2.0.0".to_string(), "hash2".to_string());
        version = version.with_migration("1.0.0".to_string(), "migrate_v1_to_v2".to_string());

        assert_eq!(
            version.get_migration("1.0.0"),
            Some(&"migrate_v1_to_v2".to_string())
        );
        assert_eq!(version.get_migration("0.9.0"), None);
    }

    #[test]
    fn test_extract_major_version() {
        assert_eq!(VersionRegistry::extract_major("1.0.0"), Some(1));
        assert_eq!(VersionRegistry::extract_major("2.3.4"), Some(2));
        assert_eq!(VersionRegistry::extract_major("invalid"), None);
    }

    #[test]
    fn test_registry_migration_compatible() {
        let mut registry = VersionRegistry::default();
        registry.register(WorkflowVersion::new(
            "1.1.0".to_string(),
            "hash2".to_string(),
        ));

        let result = registry.migrate_checkpoint("{}", "1.0.0", "1.1.0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_migration_incompatible() {
        let mut registry = VersionRegistry::default();
        registry.register(WorkflowVersion::new(
            "2.0.0".to_string(),
            "hash2".to_string(),
        ));

        let result = registry.migrate_checkpoint("{}", "1.0.0", "2.0.0");
        assert!(result.is_err());
    }
}
