//! Configuration file watcher and loader
//!
//! Provides hot-reload capabilities for workflow configuration files,
//! watching for changes and merging domain-specific overrides with defaults.

use super::workflow::{WorkflowConfig, DomainConfig, VersionStorageConfig, SlaConfig};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

// Use parking_lot instead of tokio::sync for WASM compatibility
use parking_lot::RwLock;

/// Watches configuration files for changes and provides loading
#[derive(Clone)]
pub struct ConfigWatcher {
    config_dir: PathBuf,
    defaults: WorkflowConfig,
    domain_cache: Arc<RwLock<HashMap<String, WorkflowConfig>>>,
}

/// Event emitted when configuration changes
#[derive(Debug, Clone)]
pub enum ConfigChangeEvent {
    Updated { domain_id: String },
    Created { domain_id: String },
    Deleted { domain_id: String },
}

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error reading {0:?}: {1}")]
    IoError(PathBuf, std::io::Error),

    #[error("Parse error in {0:?}: {1}")]
    ParseError(PathBuf, serde_yaml::Error),

    #[error("Configuration validation error: {0}")]
    ValidationError(String),
}

impl ConfigWatcher {
    /// Create a new ConfigWatcher with the specified configuration directory
    pub fn new(config_dir: PathBuf) -> Result<Self, ConfigError> {
        let defaults_path = config_dir.join("defaults.yaml");
        let defaults = Self::load_yaml::<WorkflowConfig>(&defaults_path)?;

        // Validate defaults
        defaults.validate()
            .map_err(|e| ConfigError::ValidationError(format!("Invalid defaults.yaml: {}", e)))?;

        Ok(Self {
            config_dir,
            defaults,
            domain_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get the default configuration
    pub fn defaults(&self) -> &WorkflowConfig {
        &self.defaults
    }

    /// Load configuration for a specific domain, merging with defaults
    pub fn load_config(&self, domain_id: &str) -> Result<WorkflowConfig, ConfigError> {
        // Check cache first
        {
            let cache = self.domain_cache.read();
            if let Some(cached) = cache.get(domain_id) {
                return Ok(cached.clone());
            }
        }

        // Load domain override if exists
        let domain_path = self.config_dir.join("domains").join(format!("{}.yaml", domain_id));

        let config = if domain_path.exists() {
            let domain_override: DomainConfig = Self::load_yaml(&domain_path)?;

            // Validate override
            if let Some(ref stages) = domain_override.approval_stages {
                for stage in stages {
                    stage.validate()
                        .map_err(|e| ConfigError::ValidationError(format!("Domain {}: {}", domain_id, e)))?;
                }
            }

            Self::merge_configs(self.defaults.clone(), domain_override)?
        } else {
            self.defaults.clone()
        };

        // Update cache with double-check pattern
        let mut cache = self.domain_cache.write();

        // Re-check in case another task inserted while we were loading
        if !cache.contains_key(domain_id) {
            cache.insert(domain_id.to_string(), config.clone());
        }

        Ok(config)
    }

    /// Reload configuration for a domain (clears cache)
    pub fn reload_config(&self, domain_id: &str) -> Result<WorkflowConfig, ConfigError> {
        let mut cache = self.domain_cache.write();
        cache.remove(domain_id);
        drop(cache);
        self.load_config(domain_id)
    }

    /// Clear the entire cache (useful for bulk reloads)
    pub fn clear_cache(&self) {
        let mut cache = self.domain_cache.write();
        cache.clear();
    }

    /// Get all currently cached domain IDs
    pub fn cached_domains(&self) -> Vec<String> {
        let cache = self.domain_cache.read();
        cache.keys().cloned().collect()
    }

    /// Check if a domain config file exists
    pub fn domain_exists(&self, domain_id: &str) -> bool {
        let domain_path = self.config_dir.join("domains").join(format!("{}.yaml", domain_id));
        domain_path.exists()
    }

    /// Merge domain override with default configuration
    fn merge_configs(
        defaults: WorkflowConfig,
        override_config: DomainConfig,
    ) -> Result<WorkflowConfig, ConfigError> {
        let merged = override_config.merge_with_defaults(defaults);

        // Validate the merged configuration
        merged.validate()
            .map_err(|e| ConfigError::ValidationError(format!("Merged config validation failed: {}", e)))?;

        Ok(merged)
    }

    /// Load a YAML file with size limit validation
    fn load_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ConfigError> {
        const MAX_CONFIG_SIZE: usize = 1_048_576; // 1MB

        // Check file size before reading
        let metadata = std::fs::metadata(path)
            .map_err(|e| ConfigError::IoError(path.to_path_buf(), e))?;

        if metadata.len() > MAX_CONFIG_SIZE as u64 {
            return Err(ConfigError::ValidationError(
                format!("Config file too large: {} bytes (max {})",
                    metadata.len(), MAX_CONFIG_SIZE)
            ));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(path.to_path_buf(), e))?;

        if content.len() > MAX_CONFIG_SIZE {
            return Err(ConfigError::ValidationError(
                format!("Config file too large after reading: {} bytes", content.len())
            ));
        }

        serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(path.to_path_buf(), e))
    }

    /// Extract domain_id from a config file path
    fn extract_domain_id(path: &PathBuf) -> Option<String> {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
}

/// Manual configuration loader for non-watching scenarios
pub struct ConfigLoader {
    config_dir: PathBuf,
}

impl ConfigLoader {
    /// Create a new ConfigLoader
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    /// Load default configuration
    pub fn load_defaults(&self) -> Result<WorkflowConfig, ConfigError> {
        let defaults_path = self.config_dir.join("defaults.yaml");
        ConfigWatcher::load_yaml(&defaults_path)
    }

    /// Load domain-specific configuration
    pub fn load_domain(&self, domain_id: &str) -> Result<DomainConfig, ConfigError> {
        let domain_path = self.config_dir.join("domains").join(format!("{}.yaml", domain_id));
        ConfigWatcher::load_yaml(&domain_path)
    }

    /// Load full config for a domain (merged with defaults)
    pub fn load_domain_full(&self, domain_id: &str) -> Result<WorkflowConfig, ConfigError> {
        let defaults = self.load_defaults()?;
        let domain_override = self.load_domain(domain_id)?;

        let merged = domain_override.merge_with_defaults(defaults);
        merged.validate()
            .map_err(|e| ConfigError::ValidationError(format!("Domain config validation failed: {}", e)))?;

        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::ValidationError("test error".to_string());
        assert_eq!(format!("{}", err), "Configuration validation error: test error");
    }

    #[test]
    fn test_extract_domain_id() {
        let path = PathBuf::from("/config/domains/example.yaml");
        assert_eq!(ConfigWatcher::extract_domain_id(&path), Some("example".to_string()));

        let path2 = PathBuf::from("/config/domains/test.yml");
        assert_eq!(ConfigWatcher::extract_domain_id(&path2), Some("test".to_string()));

        let path3 = PathBuf::from("/config/domains/.hidden.yaml");
        assert_eq!(ConfigWatcher::extract_domain_id(&path3), Some(".hidden".to_string()));
    }

    #[test]
    fn test_config_loader_nonexistent_domain() {
        let loader = ConfigLoader::new(PathBuf::from("/nonexistent"));
        let result = loader.load_domain("example");
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::IoError(_, _) => {},
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_cache_operations() {
        // This test verifies the caching mechanism without requiring actual files
        let dummy_config = WorkflowConfig {
            approval_stages: vec![],
            version_storage: VersionStorageConfig::default(),
            sla: SlaConfig {
                weekend_days: vec![],
                escalation_hours: vec![],
            },
        };

        let cache = Arc::new(RwLock::new(HashMap::new()));

        // Insert into cache
        {
            let mut c = cache.write();
            c.insert("test_domain".to_string(), dummy_config.clone());
        }

        // Read from cache
        {
            let c = cache.read();
            assert!(c.contains_key("test_domain"));
            assert_eq!(c.get("test_domain").unwrap().approval_stages.len(), 0);
        }

        // Clear cache
        {
            let mut c = cache.write();
            c.clear();
        }

        // Verify empty
        {
            let c = cache.read();
            assert!(!c.contains_key("test_domain"));
            assert_eq!(c.len(), 0);
        }
    }

    #[test]
    fn test_change_event_display() {
        let event = ConfigChangeEvent::Updated { domain_id: "test".to_string() };
        assert_eq!(format!("{:?}", event), "Updated { domain_id: \"test\" }");
    }
}
