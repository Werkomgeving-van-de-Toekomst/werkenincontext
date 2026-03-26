//! Integration tests for workflow configuration system
//!
//! Tests YAML parsing, validation, and configuration merging for workflow settings.

use iou_core::config::{
    WorkflowConfig, StageConfig, ApproverConfig, ApprovalTypeConfig,
    VersionStorageConfig, SlaConfig, DomainConfig,
};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_temp_yaml(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

#[tokio::test]
async fn test_workflow_config_deserializes_from_valid_yaml() {
    let yaml = r#"
approval_stages:
  - stage_id: "review"
    stage_name: "Review"
    stage_order: 1
    approval_type: sequential
    approvers:
      - role: "manager"
    sla_hours: 72
    expiry_action: "notify_only"
    is_optional: false
version_storage:
  full_versions_keep: 5
  compress_after_days: 30
sla:
  weekend_days: [Saturday, Sunday]
  escalation_hours: [24, 8, 1]
"#;

    let config: WorkflowConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.approval_stages.len(), 1);
    assert_eq!(config.approval_stages[0].stage_id, "review");
    assert_eq!(config.version_storage.full_versions_keep, 5);
    assert_eq!(config.sla.weekend_days.len(), 2);
}

#[tokio::test]
async fn test_stage_config_validates_approval_type() {
    let test_cases = [
        ("sequential", ApprovalTypeConfig::Sequential),
        ("parallel_any", ApprovalTypeConfig::ParallelAny),
        ("parallel_all", ApprovalTypeConfig::ParallelAll),
        ("parallel_majority", ApprovalTypeConfig::ParallelMajority),
    ];

    for (approval_type_str, expected) in test_cases {
        let yaml = format!(r#"
stage_id: "test"
stage_name: "Test"
stage_order: 1
approval_type: {}
approvers:
  - role: "manager"
sla_hours: 72
expiry_action: "notify_only"
is_optional: false
"#, approval_type_str);

        let config: StageConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.approval_type, expected);
    }
}

#[tokio::test]
async fn test_stage_config_validates_sla_hours_positive() {
    let mut config = StageConfig {
        stage_id: "test".to_string(),
        stage_name: "Test".to_string(),
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
    };
    assert!(config.validate().is_ok());

    config.sla_hours = -1;
    assert!(config.validate().is_err());

    config.sla_hours = 0;
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_optional_stage_allows_empty_approvers() {
    let mut config = StageConfig {
        stage_id: "optional".to_string(),
        stage_name: "Optional Stage".to_string(),
        stage_order: 1,
        approval_type: ApprovalTypeConfig::Sequential,
        approvers: vec![],
        sla_hours: 72,
        expiry_action: "notify_only".to_string(),
        is_optional: true,
        condition: None,
    };
    assert!(config.validate().is_ok());

    config.is_optional = false;
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_version_storage_default_full_versions_keep() {
    let yaml = r#"
compress_after_days: 30
"#;

    let config: VersionStorageConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.full_versions_keep, 5);
    assert_eq!(config.compress_after_days, 30);
}

#[tokio::test]
async fn test_sla_config_weekend_days_validation() {
    let yaml = r#"
weekend_days:
  - Saturday
  - Sunday
escalation_hours:
  - 24
  - 8
  - 1
"#;

    let config: SlaConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.validate().is_ok());

    // Test invalid day name
    let yaml_invalid = r#"
weekend_days:
  - Funday
escalation_hours: [24]
"#;

    let config_invalid: SlaConfig = serde_yaml::from_str(yaml_invalid).unwrap();
    assert!(config_invalid.validate().is_err());
}

#[tokio::test]
async fn test_sla_config_case_insensitive_weekend_days() {
    let yaml = r#"
weekend_days:
  - saturday
  - SUNDAY
escalation_hours: [24]
"#;

    let config: SlaConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_invalid_yaml_returns_error() {
    let invalid_yaml = r#"
approval_stages:
  - stage_id: "test
"#;  // Missing closing quote

    let result: Result<WorkflowConfig, _> = serde_yaml::from_str(invalid_yaml);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_workflow_config_detects_duplicate_stage_ids() {
    let yaml = r#"
approval_stages:
  - stage_id: "duplicate"
    stage_name: "First"
    stage_order: 1
    approval_type: sequential
    approvers:
      - role: "manager"
    sla_hours: 72
    expiry_action: "notify_only"
    is_optional: false
  - stage_id: "duplicate"
    stage_name: "Second"
    stage_order: 2
    approval_type: sequential
    approvers:
      - role: "director"
    sla_hours: 48
    expiry_action: "notify_only"
    is_optional: false
version_storage:
  full_versions_keep: 5
  compress_after_days: 30
sla:
  weekend_days: []
  escalation_hours: []
"#;

    let config: WorkflowConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.validate().is_err());
    assert!(config.validate().unwrap_err().to_string().contains("Duplicate"));
}

#[tokio::test]
async fn test_domain_config_merges_with_defaults() {
    let defaults_yaml = r#"
approval_stages:
  - stage_id: "default_stage"
    stage_name: "Default"
    stage_order: 1
    approval_type: sequential
    approvers:
      - role: "manager"
    sla_hours: 72
    expiry_action: "notify_only"
    is_optional: false
version_storage:
  full_versions_keep: 5
  compress_after_days: 30
sla:
  weekend_days: [Saturday]
  escalation_hours: [24]
"#;

    let override_yaml = r#"
version_storage:
  full_versions_keep: 10
  compress_after_days: 60
"#;

    let defaults: WorkflowConfig = serde_yaml::from_str(defaults_yaml).unwrap();
    let override_config: DomainConfig = serde_yaml::from_str(override_yaml).unwrap();

    let merged = override_config.merge_with_defaults(defaults);

    assert_eq!(merged.approval_stages.len(), 1); // From defaults
    assert_eq!(merged.approval_stages[0].stage_id, "default_stage");
    assert_eq!(merged.version_storage.full_versions_keep, 10); // From override
    assert_eq!(merged.sla.weekend_days.len(), 1); // From defaults
}

#[tokio::test]
async fn test_approver_xor_validation() {
    let yaml = r#"
user_id: "user123"
role: "manager"
"#;

    let approver: ApproverConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(approver.validate().is_err());

    let yaml_valid = r#"
user_id: "user123"
"#;

    let approver_valid: ApproverConfig = serde_yaml::from_str(yaml_valid).unwrap();
    assert!(approver_valid.validate().is_ok());
}

#[tokio::test]
async fn test_expiry_action_parsing_with_target() {
    let yaml = r#"
stage_id: "test"
stage_name: "Test"
stage_order: 1
approval_type: sequential
approvers: []
sla_hours: 72
expiry_action: "escalate_to:finance_team"
is_optional: false
"#;

    let config: StageConfig = serde_yaml::from_str(yaml).unwrap();
    let (action, target) = config.parse_expiry_action();
    assert_eq!(action, "escalate_to");
    assert_eq!(target, Some("finance_team".to_string()));
}

#[tokio::test]
async fn test_condition_field_optional() {
    let yaml = r#"
stage_id: "test"
stage_name: "Test"
stage_order: 1
approval_type: sequential
approvers:
  - role: "manager"
sla_hours: 72
expiry_action: "notify_only"
is_optional: false
"#;

    let config: StageConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.condition.is_none());
}

#[tokio::test]
async fn test_sla_config_validates_escalation_hours() {
    let yaml = r#"
weekend_days: []
escalation_hours:
  - 24
  - 8
  - -1
"#;

    let config: SlaConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_version_storage_validates_full_versions_keep() {
    let yaml = r#"
full_versions_keep: 0
compress_after_days: 30
"#;

    let config: VersionStorageConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.full_versions_keep == 0); // Can parse

    // But WorkflowConfig validation should catch this
    let workflow_yaml = format!(r#"
approval_stages: []
version_storage:
  full_versions_keep: 0
  compress_after_days: 30
sla:
  weekend_days: []
  escalation_hours: []
"#);

    let workflow: WorkflowConfig = serde_yaml::from_str(&workflow_yaml).unwrap();
    assert!(workflow.validate().is_err());
}
