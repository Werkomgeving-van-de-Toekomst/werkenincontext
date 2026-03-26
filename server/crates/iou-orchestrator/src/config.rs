//! Configuration types for the workflow orchestrator

use crate::context::AgentType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Main configuration for the workflow orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    // Agent timeouts (configurable per document type)
    pub default_timeout_ms: u64,
    pub agent_timeouts: HashMap<String, u64>,

    // Retry policy
    pub max_retries: u32,
    pub retry_policy: RetryPolicy,

    // Checkpoint configuration
    pub checkpoint_interval_ms: u64,
    pub checkpoint_enabled: bool,
    pub checkpoint_retention_days: u32,

    // Parallel execution
    pub enable_parallel: bool,
    pub max_parallel_agents: usize,

    // Human interaction
    pub approval_timeout_hours: u32,
    pub extended_timeout_hours: u32,
    pub escalation_contacts: Vec<String>,

    // Event buffer
    pub event_buffer_size: usize,

    // Workflow versioning
    pub current_workflow_version: String,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 300_000, // 5 minutes
            agent_timeouts: HashMap::new(),
            max_retries: 3,
            retry_policy: RetryPolicy::default(),
            checkpoint_interval_ms: 30_000, // 30 seconds
            checkpoint_enabled: true,
            checkpoint_retention_days: 30,
            enable_parallel: false,
            max_parallel_agents: 2,
            approval_timeout_hours: 24,
            extended_timeout_hours: 72,
            escalation_contacts: Vec::new(),
            event_buffer_size: 100,
            current_workflow_version: "1.0.0".to_string(),
        }
    }
}

impl OrchestratorConfig {
    /// Get timeout for a specific agent
    pub fn get_agent_timeout(&self, agent: AgentType) -> Duration {
        let key = format!("{:?}", agent);
        let ms = self.agent_timeouts.get(&key).copied().unwrap_or(self.default_timeout_ms);
        Duration::from_millis(ms)
    }

    /// Get timeout for a document type
    pub fn get_document_timeout(&self, doc_type: &str) -> Duration {
        let ms = self.agent_timeouts.get(doc_type).copied().unwrap_or(self.default_timeout_ms);
        Duration::from_millis(ms)
    }

    /// Set timeout for a specific agent
    pub fn set_agent_timeout(&mut self, agent: AgentType, ms: u64) {
        let key = format!("{:?}", agent);
        self.agent_timeouts.insert(key, ms);
    }

    /// Check if checkpoints should be enabled
    pub fn is_checkpoint_enabled(&self) -> bool {
        self.checkpoint_enabled
    }

    /// Get the checkpoint interval
    pub fn checkpoint_interval(&self) -> Duration {
        Duration::from_millis(self.checkpoint_interval_ms)
    }
}

/// Retry policy for transient errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_backoff_ms: 1000,
            max_backoff_ms: 16_000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Calculate backoff duration for given attempt
    pub fn backoff(&self, attempt: u32) -> Duration {
        let ms = (self.base_backoff_ms as f64 * self.backoff_multiplier.powi(attempt as i32))
            .min(self.max_backoff_ms as f64) as u64;

        if self.jitter {
            let jitter_ms = (ms as f64 * 0.1) as u64; // ±10%
            let jittered = if jitter_ms > 0 {
                ms - jitter_ms + rand::random::<u64>() % (2 * jitter_ms)
            } else {
                ms
            };
            Duration::from_millis(jittered)
        } else {
            Duration::from_millis(ms)
        }
    }

    /// Check if retry is allowed for this attempt
    pub fn can_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}

/// Timeout configuration for approvals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub standard_timeout_hours: u32,
    pub extended_timeout_hours: u32,
    pub escalation_contacts: Vec<String>,
    pub auto_reject_after_extended: bool,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            standard_timeout_hours: 24,
            extended_timeout_hours: 72,
            escalation_contacts: Vec::new(),
            auto_reject_after_extended: false,
        }
    }
}

impl TimeoutConfig {
    /// Create from OrchestratorConfig
    pub fn from_config(config: &OrchestratorConfig) -> Self {
        Self {
            standard_timeout_hours: config.approval_timeout_hours,
            extended_timeout_hours: config.extended_timeout_hours,
            escalation_contacts: config.escalation_contacts.clone(),
            auto_reject_after_extended: false,
        }
    }
}

/// Document type specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTypeConfig {
    pub document_type: String,
    pub timeout_ms: u64,
    pub require_human_review: bool,
    pub parallel_agents: Vec<AgentType>,
    pub approval_timeout_override: Option<u32>,
}

impl DocumentTypeConfig {
    /// Create a new document type config
    pub fn new(document_type: String) -> Self {
        Self {
            document_type,
            timeout_ms: 300_000,
            require_human_review: false,
            parallel_agents: Vec::new(),
            approval_timeout_override: None,
        }
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set human review requirement
    pub fn with_review(mut self, require: bool) -> Self {
        self.require_human_review = require;
        self
    }

    /// Set parallel agents
    pub fn with_parallel(mut self, agents: Vec<AgentType>) -> Self {
        self.parallel_agents = agents;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.default_timeout_ms, 300_000);
        assert_eq!(config.max_retries, 3);
        assert!(config.checkpoint_enabled);
    }

    #[test]
    fn test_retry_policy_backoff() {
        let policy = RetryPolicy {
            base_backoff_ms: 1000,
            backoff_multiplier: 2.0,
            max_backoff_ms: 8000,
            jitter: false,
            ..Default::default()
        };

        assert_eq!(policy.backoff(0).as_millis(), 1000);
        assert_eq!(policy.backoff(1).as_millis(), 2000);
        assert_eq!(policy.backoff(2).as_millis(), 4000);
        assert_eq!(policy.backoff(3).as_millis(), 8000); // capped
        assert_eq!(policy.backoff(10).as_millis(), 8000); // still capped
    }

    #[test]
    fn test_retry_policy_with_jitter() {
        let policy = RetryPolicy {
            base_backoff_ms: 1000,
            backoff_multiplier: 2.0,
            max_backoff_ms: 8000,
            jitter: true,
            ..Default::default()
        };

        let backoff = policy.backoff(0).as_millis();
        // With jitter, should be approximately 1000 ± 100
        assert!(backoff >= 900 && backoff <= 1100);
    }

    #[test]
    fn test_agent_timeout() {
        let mut config = OrchestratorConfig::default();
        assert_eq!(config.get_agent_timeout(AgentType::Research).as_millis(), 300_000);

        config.set_agent_timeout(AgentType::Research, 600_000);
        assert_eq!(config.get_agent_timeout(AgentType::Research).as_millis(), 600_000);
    }

    #[test]
    fn test_document_type_config_builder() {
        let config = DocumentTypeConfig::new("woo_besluit".to_string())
            .with_timeout(120_000)
            .with_review(true);

        assert_eq!(config.timeout_ms, 120_000);
        assert!(config.require_human_review);
    }
}
