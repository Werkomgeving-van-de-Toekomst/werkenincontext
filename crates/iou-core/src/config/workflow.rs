//! Workflow configuration type definitions
//!
//! Defines the structure for YAML-based workflow configuration,
//! including approval stages, version storage settings, and SLA policies.

use serde::{Deserialize, Serialize};

/// Root workflow configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowConfig {
    pub approval_stages: Vec<StageConfig>,
    pub version_storage: VersionStorageConfig,
    pub sla: SlaConfig,
}

/// Configuration for a single approval stage
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StageConfig {
    pub stage_id: String,
    pub stage_name: String,
    pub stage_order: i32,
    #[serde(rename = "approval_type")]
    pub approval_type: ApprovalTypeConfig,
    pub approvers: Vec<ApproverConfig>,
    #[serde(rename = "sla_hours")]
    pub sla_hours: i32,
    #[serde(rename = "expiry_action")]
    pub expiry_action: String,
    #[serde(default)]
    pub is_optional: bool,
    pub condition: Option<String>,
}

impl StageConfig {
    /// Validate the stage configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check sla_hours is positive
        if self.sla_hours <= 0 {
            return Err(format!(
                "Stage '{}' has invalid sla_hours: {} (must be positive)",
                self.stage_id, self.sla_hours
            ));
        }

        // Check that non-optional stages have at least one approver
        if !self.is_optional && self.approvers.is_empty() {
            return Err(format!(
                "Stage '{}' is not optional but has no approvers",
                self.stage_id
            ));
        }

        // Validate approvers
        for (idx, approver) in self.approvers.iter().enumerate() {
            approver.validate()
                .map_err(|e| format!("Stage '{}' approver {}: {}", self.stage_id, idx, e))?;
        }

        // Validate stage_order is non-negative
        if self.stage_order < 0 {
            return Err(format!(
                "Stage '{}' has negative stage_order: {}",
                self.stage_id, self.stage_order
            ));
        }

        Ok(())
    }

    /// Parse expiry_action string into components
    pub fn parse_expiry_action(&self) -> (String, Option<String>) {
        let parts: Vec<&str> = self.expiry_action.splitn(2, ':').collect();
        match parts.as_slice() {
            [action] => (action.to_string(), None),
            [action, target] => (action.to_string(), Some(target.to_string())),
            _ => (self.expiry_action.clone(), None),
        }
    }
}

/// Approval type variants for configuration
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalTypeConfig {
    Sequential,
    ParallelAny,
    ParallelAll,
    ParallelMajority,
}

/// Approver configuration (user ID or role)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ApproverConfig {
    #[serde(rename = "user_id")]
    pub user_id: Option<String>,
    pub role: Option<String>,
}

impl ApproverConfig {
    /// Validate that exactly one of user_id or role is set
    pub fn validate(&self) -> Result<(), String> {
        match (&self.user_id, &self.role) {
            (None, None) => Err("Approver must have either user_id or role".to_string()),
            (Some(_), Some(_)) => Err("Approver cannot have both user_id and role".to_string()),
            _ => Ok(()),
        }
    }
}

/// Version storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionStorageConfig {
    #[serde(default = "default_full_versions_keep")]
    #[serde(rename = "full_versions_keep")]
    pub full_versions_keep: i32,
    #[serde(rename = "compress_after_days")]
    pub compress_after_days: i32,
}

fn default_full_versions_keep() -> i32 {
    5
}

impl Default for VersionStorageConfig {
    fn default() -> Self {
        Self {
            full_versions_keep: default_full_versions_keep(),
            compress_after_days: 30,
        }
    }
}

/// SLA calculation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlaConfig {
    #[serde(rename = "weekend_days")]
    pub weekend_days: Vec<String>,
    #[serde(rename = "escalation_hours")]
    pub escalation_hours: Vec<i32>,
}

impl SlaConfig {
    /// Validate the SLA configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate weekend days are valid weekday names
        let valid_days = [
            "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
        ];

        for day in &self.weekend_days {
            let lower = day.to_lowercase();
            if !valid_days.contains(&lower.as_str()) {
                return Err(format!("Invalid weekend day: '{}'", day));
            }
        }

        // Validate escalation hours are positive
        if self.escalation_hours.is_empty() {
            return Err("escalation_hours cannot be empty".to_string());
        }

        for (idx, hours) in self.escalation_hours.iter().enumerate() {
            if *hours <= 0 {
                return Err(format!(
                    "Escalation hour at index {} must be positive, got: {}",
                    idx, hours
                ));
            }
        }

        Ok(())
    }
}

impl WorkflowConfig {
    /// Validate the entire workflow configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate each stage
        for stage in &self.approval_stages {
            stage.validate()?;
        }

        // Check for duplicate stage IDs
        let mut seen_ids = std::collections::HashSet::new();
        for stage in &self.approval_stages {
            if !seen_ids.insert(&stage.stage_id) {
                return Err(format!("Duplicate stage_id: '{}'", stage.stage_id));
            }
        }

        // Check stage_order sequence
        let mut orders: Vec<_> = self.approval_stages.iter().map(|s| s.stage_order).collect();
        orders.sort();
        orders.dedup();
        if orders.len() != self.approval_stages.len() {
            return Err("Duplicate stage_order values detected".to_string());
        }

        // Validate SLA config
        self.sla.validate()?;

        // Validate version storage
        if self.version_storage.full_versions_keep < 1 {
            return Err("full_versions_keep must be at least 1".to_string());
        }
        if self.version_storage.compress_after_days < 0 {
            return Err("compress_after_days cannot be negative".to_string());
        }

        Ok(())
    }
}

/// Domain-specific configuration override
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DomainConfig {
    #[serde(rename = "approval_stages")]
    pub approval_stages: Option<Vec<StageConfig>>,
    pub version_storage: Option<VersionStorageConfig>,
    pub sla: Option<SlaConfig>,
}

impl DomainConfig {
    /// Merge this override with defaults to create a complete config
    pub fn merge_with_defaults(&self, defaults: WorkflowConfig) -> WorkflowConfig {
        WorkflowConfig {
            approval_stages: self.approval_stages.clone().unwrap_or(defaults.approval_stages),
            version_storage: self.version_storage.clone().unwrap_or(defaults.version_storage),
            sla: self.sla.clone().unwrap_or(defaults.sla),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_type_config_serializes_correctly() {
        let types = vec![
            ApprovalTypeConfig::Sequential,
            ApprovalTypeConfig::ParallelAny,
            ApprovalTypeConfig::ParallelAll,
            ApprovalTypeConfig::ParallelMajority,
        ];

        for t in types {
            let serialized = serde_yaml::to_string(&t).unwrap();
            let deserialized: ApprovalTypeConfig = serde_yaml::from_str(&serialized).unwrap();
            assert_eq!(t, deserialized);
        }
    }

    #[test]
    fn test_stage_config_validates_positive_sla_hours() {
        let valid_stage = StageConfig {
            stage_id: "test".to_string(),
            stage_name: "Test".to_string(),
            stage_order: 1,
            approval_type: ApprovalTypeConfig::Sequential,
            approvers: vec![ApproverConfig {
                user_id: Some("user123".to_string()),
                role: None,
            }],
            sla_hours: 72,
            expiry_action: "notify_only".to_string(),
            is_optional: false,
            condition: None,
        };
        assert!(valid_stage.validate().is_ok());

        let mut invalid_stage = valid_stage.clone();
        invalid_stage.sla_hours = -1;
        assert!(invalid_stage.validate().is_err());

        invalid_stage.sla_hours = 0;
        assert!(invalid_stage.validate().is_err());
    }

    #[test]
    fn test_stage_config_allows_empty_approvers_for_optional() {
        let mut stage = StageConfig {
            stage_id: "optional_stage".to_string(),
            stage_name: "Optional".to_string(),
            stage_order: 1,
            approval_type: ApprovalTypeConfig::Sequential,
            approvers: vec![],
            sla_hours: 72,
            expiry_action: "notify_only".to_string(),
            is_optional: true,
            condition: None,
        };
        assert!(stage.validate().is_ok());

        stage.is_optional = false;
        assert!(stage.validate().is_err());
    }

    #[test]
    fn test_approver_config_validates_xor_fields() {
        let both = ApproverConfig {
            user_id: Some("user123".to_string()),
            role: Some("manager".to_string()),
        };
        assert!(both.validate().is_err());

        let neither = ApproverConfig {
            user_id: None,
            role: None,
        };
        assert!(neither.validate().is_err());

        let user_only = ApproverConfig {
            user_id: Some("user123".to_string()),
            role: None,
        };
        assert!(user_only.validate().is_ok());

        let role_only = ApproverConfig {
            user_id: None,
            role: Some("manager".to_string()),
        };
        assert!(role_only.validate().is_ok());
    }

    #[test]
    fn test_version_storage_default_full_versions_keep() {
        let config: VersionStorageConfig = serde_yaml::from_str("compress_after_days: 30").unwrap();
        assert_eq!(config.full_versions_keep, 5);
    }

    #[test]
    fn test_sla_config_validates_weekend_days() {
        let valid = SlaConfig {
            weekend_days: vec!["Saturday".to_string(), "Sunday".to_string()],
            escalation_hours: vec![24, 8, 1],
        };
        assert!(valid.validate().is_ok());

        let invalid = SlaConfig {
            weekend_days: vec!["Funday".to_string()],
            escalation_hours: vec![24],
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_sla_config_case_insensitive_day_validation() {
        let config = SlaConfig {
            weekend_days: vec!["saturday".to_string(), "SUNDAY".to_string()],
            escalation_hours: vec![24],
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_workflow_config_detects_duplicate_stage_ids() {
        let config = WorkflowConfig {
            approval_stages: vec![
                StageConfig {
                    stage_id: "duplicate".to_string(),
                    stage_name: "First".to_string(),
                    stage_order: 1,
                    approval_type: ApprovalTypeConfig::Sequential,
                    approvers: vec![ApproverConfig {
                        user_id: Some("user".to_string()),
                        role: None,
                    }],
                    sla_hours: 72,
                    expiry_action: "notify_only".to_string(),
                    is_optional: false,
                    condition: None,
                },
                StageConfig {
                    stage_id: "duplicate".to_string(),
                    stage_name: "Second".to_string(),
                    stage_order: 2,
                    approval_type: ApprovalTypeConfig::Sequential,
                    approvers: vec![],
                    sla_hours: 48,
                    expiry_action: "notify_only".to_string(),
                    is_optional: true,
                    condition: None,
                },
            ],
            version_storage: VersionStorageConfig::default(),
            sla: SlaConfig {
                weekend_days: vec![],
                escalation_hours: vec![],
            },
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_expiry_action_parsing() {
        let stage = StageConfig {
            stage_id: "test".to_string(),
            stage_name: "Test".to_string(),
            stage_order: 1,
            approval_type: ApprovalTypeConfig::Sequential,
            approvers: vec![],
            sla_hours: 72,
            expiry_action: "notify_only".to_string(),
            is_optional: false,
            condition: None,
        };
        assert_eq!(stage.parse_expiry_action(), ("notify_only".to_string(), None));

        let stage_with_target = StageConfig {
            expiry_action: "escalate_to:manager".to_string(),
            ..stage.clone()
        };
        assert_eq!(
            stage_with_target.parse_expiry_action(),
            ("escalate_to".to_string(), Some("manager".to_string()))
        );
    }

    #[test]
    fn test_domain_config_merges_with_defaults() {
        let defaults = WorkflowConfig {
            approval_stages: vec![StageConfig {
                stage_id: "default_stage".to_string(),
                stage_name: "Default".to_string(),
                stage_order: 1,
                approval_type: ApprovalTypeConfig::Sequential,
                approvers: vec![],
                sla_hours: 72,
                expiry_action: "notify_only".to_string(),
                is_optional: false,
                condition: None,
            }],
            version_storage: VersionStorageConfig {
                full_versions_keep: 5,
                compress_after_days: 30,
            },
            sla: SlaConfig {
                weekend_days: vec!["Saturday".to_string()],
                escalation_hours: vec![24],
            },
        };

        let override_config = DomainConfig {
            approval_stages: None,
            version_storage: Some(VersionStorageConfig {
                full_versions_keep: 10,
                compress_after_days: 60,
            }),
            sla: None,
        };

        let merged = override_config.merge_with_defaults(defaults.clone());
        assert_eq!(merged.approval_stages, defaults.approval_stages); // From defaults
        assert_eq!(merged.version_storage.full_versions_keep, 10); // From override
        assert_eq!(merged.sla.weekend_days, defaults.sla.weekend_days); // From defaults
    }
}
