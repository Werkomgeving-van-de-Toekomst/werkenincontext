//! Data Quality Metrics Engine
//!
//! Comprehensive quality assessment framework for registry entities.
//! Implements quality rules, validation, scoring, and trend analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::quality::{
    QualityAssessment, QualityDimension, QualityIssue, QualityIssue as QualityIssueType,
    QualityScore, QualityThreshold,
};
use crate::registry::{DataEntity, EntityType};

/// Quality rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRule {
    /// Unique rule identifier
    pub id: String,

    /// Rule name
    pub name: String,

    /// Rule description
    pub description: String,

    /// Quality dimension this rule measures
    pub dimension: QualityDimension,

    /// Rule condition
    pub condition: RuleCondition,

    /// Severity when rule fails
    pub severity: IssueSeverity,

    /// Entity types this rule applies to
    pub applies_to: Vec<EntityType>,

    /// Whether rule is enabled
    pub enabled: bool,

    /// Rule weight for scoring (0-1)
    pub weight: f32,
}

/// Rule condition for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RuleCondition {
    /// Required field must be present and non-empty
    #[serde(rename = "required")]
    Required { field: String },

    /// Field value must match pattern
    #[serde(rename = "pattern")]
    Pattern {
        field: String,
        pattern: String,
        case_sensitive: bool,
    },

    /// Field value must be in allowed set
    #[serde(rename = "allowed_values")]
    AllowedValues {
        field: String,
        values: Vec<serde_json::Value>,
    },

    /// Field value must be within range
    #[serde(rename = "range")]
    Range {
        field: String,
        min: Option<serde_json::Value>,
        max: Option<serde_json::Value>,
    },

    /// Field must be valid date/time
    #[serde(rename = "datetime")]
    DateTime {
        field: String,
        format: Option<String>,
        allow_future: bool,
        allow_past: bool,
    },

    /// Field must be valid email
    #[serde(rename = "email")]
    Email { field: String },

    /// Field must be valid URL
    #[serde(rename = "url")]
    Url { field: String },

    /// Cross-field validation
    #[serde(rename = "cross_field")]
    CrossField {
        field1: String,
        field2: String,
        operator: ComparisonOperator,
    },

    /// Custom JavaScript expression
    #[serde(rename = "custom")]
    Custom {
        expression: String,
        required_fields: Vec<String>,
    },

    /// Completeness threshold for metadata
    #[serde(rename = "metadata_completeness")]
    MetadataCompleteness {
        required_fields: Vec<String>,
        threshold: f32,
    },

    /// Reference validation (foreign key check)
    #[serde(rename = "reference")]
    Reference {
        field: String,
        reference_type: String,
        allow_null: bool,
    },
}

/// Comparison operator for cross-field validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

/// Issue severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Quality rule set (collection of rules)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRuleSet {
    /// Rule set identifier
    pub id: String,

    /// Rule set name
    pub name: String,

    /// Rules in this set
    pub rules: Vec<QualityRule>,

    /// Entity type this rule set applies to
    pub entity_type: EntityType,

    /// Minimum passing score (0-1)
    pub passing_threshold: f32,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// Quality calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCalculation {
    /// Entity assessed
    pub entity_id: Uuid,

    /// Overall quality score
    pub score: QualityScore,

    /// Dimension scores
    pub dimension_scores: HashMap<QualityDimension, DimensionScore>,

    /// Rules evaluated
    pub rules_evaluated: usize,

    /// Rules passed
    pub rules_passed: usize,

    /// Rules failed
    pub rules_failed: usize,

    /// Issues found
    pub issues: Vec<QualityIssueResult>,

    /// Assessment timestamp
    pub assessed_at: DateTime<Utc>,

    /// Calculation duration (ms)
    pub duration_ms: u64,
}

/// Score for a single quality dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    /// Dimension type
    pub dimension: QualityDimension,

    /// Score (0-1)
    pub score: f32,

    /// Weight in overall score
    pub weight: f32,

    /// Rules evaluated for this dimension
    pub rules_evaluated: usize,

    /// Rules passed for this dimension
    pub rules_passed: usize,
}

/// Quality issue result from rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssueResult {
    /// Rule that generated this issue
    pub rule_id: String,

    /// Rule name
    pub rule_name: String,

    /// Dimension affected
    pub dimension: QualityDimension,

    /// Severity
    pub severity: IssueSeverity,

    /// Issue message
    pub message: String,

    /// Affected field
    pub affected_field: Option<String>,

    /// Actual value that caused the issue
    pub actual_value: Option<serde_json::Value>,

    /// Expected value or constraint
    pub expected_value: Option<String>,

    /// Suggestion for fixing
    pub suggestion: Option<String>,
}

/// Quality trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrendPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Average quality score
    pub average_score: f32,

    /// Entity count
    pub entity_count: usize,

    /// Issue count
    pub issue_count: usize,
}

/// Quality trend report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrendReport {
    /// Source or entity being tracked
    pub subject_id: Uuid,

    /// Subject type (source, entity_type, etc.)
    pub subject_type: TrendSubjectType,

    /// Time period covered
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    /// Trend data points
    pub data_points: Vec<QualityTrendPoint>,

    /// Overall trend direction
    pub trend_direction: TrendDirection,

    /// Average score change
    pub score_change: f32,

    /// Generated at
    pub generated_at: DateTime<Utc>,
}

/// Type of subject for trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendSubjectType {
    Source,
    EntityType,
    Organization,
    EntireRegistry,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Unknown,
}

/// Quality aggregation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAggregation {
    /// Aggregation identifier
    pub id: String,

    /// Aggregation type
    pub aggregation_type: AggregationType,

    /// Entity count
    pub entity_count: usize,

    /// Average quality score
    pub average_score: f32,

    /// Minimum score
    pub min_score: f32,

    /// Maximum score
    pub max_score: f32,

    /// Median score
    pub median_score: f32,

    /// Score distribution
    pub score_distribution: ScoreDistribution,

    /// Dimension breakdown
    pub dimension_breakdown: HashMap<QualityDimension, f32>,

    /// Issue summary
    pub issue_summary: IssueSummary,

    /// Calculated at
    pub calculated_at: DateTime<Utc>,
}

/// Type of aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationType {
    BySource,
    ByEntityType,
    ByOrganization,
    Global,
    ByTimePeriod,
}

/// Score distribution buckets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDistribution {
    /// Count of excellent scores (0.9-1.0)
    pub excellent: usize,

    /// Count of good scores (0.75-0.9)
    pub good: usize,

    /// Count of acceptable scores (0.5-0.75)
    pub acceptable: usize,

    /// Count of poor scores (0.0-0.5)
    pub poor: usize,
}

/// Issue summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSummary {
    /// Total issues
    pub total: usize,

    /// Breakdown by severity
    pub by_severity: HashMap<IssueSeverity, usize>,

    /// Breakdown by dimension
    pub by_dimension: HashMap<QualityDimension, usize>,

    /// Most common issues
    pub top_issues: Vec<(String, usize)>,
}

// ============================================
// QUALITY ENGINE
// ============================================

/// Quality assessment engine
pub struct QualityEngine {
    /// Available rule sets
    rule_sets: HashMap<String, QualityRuleSet>,

    /// Default passing threshold
    default_threshold: f32,
}

impl QualityEngine {
    /// Create a new quality engine
    pub fn new() -> Self {
        Self {
            rule_sets: HashMap::new(),
            default_threshold: 0.7,
        }
    }

    /// Create with default threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            rule_sets: HashMap::new(),
            default_threshold: threshold,
        }
    }

    /// Register a rule set
    pub fn register_rule_set(&mut self, rule_set: QualityRuleSet) {
        self.rule_sets.insert(rule_set.id.clone(), rule_set);
    }

    /// Get rule set for entity type
    pub fn get_rule_set(&self, entity_type: EntityType) -> Option<&QualityRuleSet> {
        self.rule_sets
            .values()
            .find(|rs| rs.entity_type == entity_type)
    }

    /// Assess quality of an entity
    pub fn assess_entity(&self, entity: &DataEntity) -> QualityCalculation {
        let start = std::time::Instant::now();

        let rule_set = self
            .get_rule_set(entity.entity_type)
            .or_else(|| self.rule_sets.values().next());

        let mut dimension_scores = HashMap::new();
        let mut all_issues = Vec::new();
        let mut rules_evaluated = 0;
        let mut rules_passed = 0;
        let mut rules_failed = 0;

        // Initialize all dimensions
        for dimension in &[
            QualityDimension::Completeness,
            QualityDimension::Accuracy,
            QualityDimension::Consistency,
            QualityDimension::Timeliness,
        ] {
            dimension_scores.insert(
                *dimension,
                DimensionScore {
                    dimension: *dimension,
                    score: 1.0,
                    weight: 0.25,
                    rules_evaluated: 0,
                    rules_passed: 0,
                },
            );
        }

        if let Some(rule_set) = rule_set {
            for rule in &rule_set.rules {
                if !rule.enabled {
                    continue;
                }

                if !rule.applies_to.is_empty() && !rule.applies_to.contains(&entity.entity_type) {
                    continue;
                }

                rules_evaluated += 1;

                match self.evaluate_rule(rule, entity) {
                    Ok(_) => {
                        rules_passed += 1;
                        if let Some(score) = dimension_scores.get_mut(&rule.dimension) {
                            score.rules_passed += 1;
                        }
                    }
                    Err(issue) => {
                        rules_failed += 1;
                        if let Some(score) = dimension_scores.get_mut(&rule.dimension) {
                            // Reduce score based on severity
                            let penalty = match issue.severity {
                                IssueSeverity::Critical => 0.5,
                                IssueSeverity::High => 0.3,
                                IssueSeverity::Medium => 0.15,
                                IssueSeverity::Low => 0.05,
                            };
                            score.score = (score.score - penalty).max(0.0);
                            score.rules_evaluated += 1;
                        }
                        all_issues.push(issue);
                    }
                }
            }
        }

        // Calculate final scores
        let completeness = dimension_scores[&QualityDimension::Completeness].score;
        let accuracy = dimension_scores[&QualityDimension::Accuracy].score;
        let consistency = dimension_scores[&QualityDimension::Consistency].score;
        let timeliness = self.calculate_timeliness(entity);

        let quality_score = QualityScore::new(completeness, accuracy, consistency).with_timeliness(timeliness);

        // Update dimension scores with calculated values
        dimension_scores
            .insert(QualityDimension::Timeliness, DimensionScore {
                dimension: QualityDimension::Timeliness,
                score: timeliness,
                weight: 0.2,
                rules_evaluated: 1,
                rules_passed: if timeliness >= 0.7 { 1 } else { 0 },
            });

        QualityCalculation {
            entity_id: entity.id,
            score: quality_score,
            dimension_scores,
            rules_evaluated,
            rules_passed,
            rules_failed,
            issues: all_issues,
            assessed_at: Utc::now(),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Assess quality of multiple entities
    pub fn assess_batch(&self, entities: &[DataEntity]) -> Vec<QualityCalculation> {
        entities
            .iter()
            .map(|e| self.assess_entity(e))
            .collect()
    }

    /// Aggregate quality metrics
    pub fn aggregate(&self, calculations: &[QualityCalculation]) -> QualityAggregation {
        if calculations.is_empty() {
            return QualityAggregation {
                id: Uuid::new_v4().to_string(),
                aggregation_type: AggregationType::Global,
                entity_count: 0,
                average_score: 0.0,
                min_score: 0.0,
                max_score: 0.0,
                median_score: 0.0,
                score_distribution: ScoreDistribution {
                    excellent: 0,
                    good: 0,
                    acceptable: 0,
                    poor: 0,
                },
                dimension_breakdown: HashMap::new(),
                issue_summary: IssueSummary {
                    total: 0,
                    by_severity: HashMap::new(),
                    by_dimension: HashMap::new(),
                    top_issues: Vec::new(),
                },
                calculated_at: Utc::now(),
            };
        }

        let mut scores: Vec<f32> = calculations
            .iter()
            .map(|c| c.score.overall())
            .collect();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let average_score = scores.iter().sum::<f32>() / scores.len() as f32;
        let min_score = scores.first().copied().unwrap_or(0.0);
        let max_score = scores.last().copied().unwrap_or(0.0);
        let median_score = if scores.len() % 2 == 0 {
            (scores[scores.len() / 2 - 1] + scores[scores.len() / 2]) / 2.0
        } else {
            scores[scores.len() / 2]
        };

        let score_distribution = ScoreDistribution {
            excellent: scores.iter().filter(|&&s| s >= 0.9).count(),
            good: scores.iter().filter(|&&s| s >= 0.75 && s < 0.9).count(),
            acceptable: scores.iter().filter(|&&s| s >= 0.5 && s < 0.75).count(),
            poor: scores.iter().filter(|&&s| s < 0.5).count(),
        };

        // Dimension breakdown
        let mut dimension_breakdown = HashMap::new();
        for dimension in &[
            QualityDimension::Completeness,
            QualityDimension::Accuracy,
            QualityDimension::Consistency,
            QualityDimension::Timeliness,
        ] {
            let dim_scores: Vec<f32> = calculations
                .iter()
                .filter_map(|c| c.dimension_scores.get(dimension))
                .map(|ds| ds.score)
                .collect();

            if !dim_scores.is_empty() {
                let avg = dim_scores.iter().sum::<f32>() / dim_scores.len() as f32;
                dimension_breakdown.insert(*dimension, avg);
            }
        }

        // Issue summary
        let mut by_severity = HashMap::new();
        let mut by_dimension = HashMap::new();
        let mut issue_counts: HashMap<String, usize> = HashMap::new();

        for calc in calculations {
            for issue in &calc.issues {
                *by_severity.entry(issue.severity).or_insert(0) += 1;
                *by_dimension.entry(issue.dimension).or_insert(0) += 1;
                *issue_counts.entry(issue.rule_name.clone()).or_insert(0) += 1;
            }
        }

        let mut top_issues: Vec<_> = issue_counts.into_iter().collect();
        top_issues.sort_by(|a, b| b.1.cmp(&a.1));
        top_issues.truncate(10);

        QualityAggregation {
            id: Uuid::new_v4().to_string(),
            aggregation_type: AggregationType::Global,
            entity_count: calculations.len(),
            average_score,
            min_score,
            max_score,
            median_score,
            score_distribution,
            dimension_breakdown,
            issue_summary: IssueSummary {
                total: calculations.iter().map(|c| c.issues.len()).sum(),
                by_severity,
                by_dimension,
                top_issues,
            },
            calculated_at: Utc::now(),
        }
    }

    /// Evaluate a single rule against an entity
    fn evaluate_rule(&self, rule: &QualityRule, entity: &DataEntity) -> Result<(), QualityIssueResult> {
        match &rule.condition {
            RuleCondition::Required { field } => {
                let value = self.get_field_value(entity, field);
                if value.is_none() || value == Some(serde_json::json!(null)) || value == Some(serde_json::json!("")) {
                    return Err(QualityIssueResult {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        dimension: rule.dimension,
                        severity: rule.severity,
                        message: format!("Required field '{}' is missing or empty", field),
                        affected_field: Some(field.clone()),
                        actual_value: value,
                        expected_value: Some("non-empty value".to_string()),
                        suggestion: Some(format!("Provide a value for field '{}'", field)),
                    });
                }
            }

            RuleCondition::Pattern { field, pattern, case_sensitive: _ } => {
                if let Some(value) = self.get_field_value(entity, field) {
                    if let Some(str_value) = value.as_str() {
                        let re = regex::Regex::new(pattern).unwrap_or_else(|_| regex::Regex::new(r".*").unwrap());
                        if !re.is_match(str_value) {
                            return Err(QualityIssueResult {
                                rule_id: rule.id.clone(),
                                rule_name: rule.name.clone(),
                                dimension: rule.dimension,
                                severity: rule.severity,
                                message: format!("Field '{}' does not match required pattern", field),
                                affected_field: Some(field.clone()),
                                actual_value: Some(value),
                                expected_value: Some(format!("Pattern: {}", pattern)),
                                suggestion: Some(format!("Ensure field '{}' matches pattern: {}", field, pattern)),
                            });
                        }
                    }
                }
            }

            RuleCondition::AllowedValues { field, values } => {
                if let Some(value) = self.get_field_value(entity, field) {
                    if !values.contains(&value) {
                        return Err(QualityIssueResult {
                            rule_id: rule.id.clone(),
                            rule_name: rule.name.clone(),
                            dimension: rule.dimension,
                            severity: rule.severity,
                            message: format!("Field '{}' contains invalid value", field),
                            affected_field: Some(field.clone()),
                            actual_value: Some(value),
                            expected_value: Some(format!("One of: {:?}", values)),
                            suggestion: Some(format!("Use one of the allowed values for field '{}'", field)),
                        });
                    }
                }
            }

            RuleCondition::MetadataCompleteness {
                required_fields,
                threshold,
            } => {
                let metadata = entity.metadata.as_object().unwrap_or(&serde_json::Map::new());
                let present_count = required_fields.iter().filter(|f| metadata.contains_key(*f)).count();
                let completeness = present_count as f32 / required_fields.len() as f32;

                if completeness < *threshold {
                    return Err(QualityIssueResult {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        dimension: rule.dimension,
                        severity: rule.severity,
                        message: format!("Metadata completeness ({:.0}%) below threshold ({:.0}%)", completeness * 100.0, threshold * 100.0),
                        affected_field: Some("metadata".to_string()),
                        actual_value: Some(serde_json::json!(completeness)),
                        expected_value: Some(format!(">= {:.0}", threshold)),
                        suggestion: Some(format!("Add missing metadata fields: {:?}", required_fields)),
                    });
                }
            }

            RuleCondition::Email { field } => {
                if let Some(value) = self.get_field_value(entity, field) {
                    if let Some(str_value) = value.as_str() {
                        let email_regex = regex::Regex::new(
                            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
                        ).unwrap();
                        if !email_regex.is_match(str_value) {
                            return Err(QualityIssueResult {
                                rule_id: rule.id.clone(),
                                rule_name: rule.name.clone(),
                                dimension: rule.dimension,
                                severity: rule.severity,
                                message: format!("Field '{}' is not a valid email address", field),
                                affected_field: Some(field.clone()),
                                actual_value: Some(value),
                                expected_value: Some("valid email address".to_string()),
                                suggestion: Some(format!("Provide a valid email address for field '{}'", field)),
                            });
                        }
                    }
                }
            }

            RuleCondition::Url { field } => {
                if let Some(value) = self.get_field_value(entity, field) {
                    if let Some(str_value) = value.as_str() {
                        if !str_value.starts_with("http://") && !str_value.starts_with("https://") {
                            return Err(QualityIssueResult {
                                rule_id: rule.id.clone(),
                                rule_name: rule.name.clone(),
                                dimension: rule.dimension,
                                severity: rule.severity,
                                message: format!("Field '{}' is not a valid URL", field),
                                affected_field: Some(field.clone()),
                                actual_value: Some(value),
                                expected_value: Some("valid URL starting with http:// or https://".to_string()),
                                suggestion: Some(format!("Provide a valid URL for field '{}'", field)),
                            });
                        }
                    }
                }
            }

            _ => {
                // Other rule types not yet implemented
            }
        }

        Ok(())
    }

    /// Get field value from entity
    fn get_field_value(&self, entity: &DataEntity, field: str) -> Option<serde_json::Value> {
        match field {
            "name" => Some(serde_json::json!(entity.name.clone())),
            "external_id" => Some(serde_json::json!(entity.external_id.clone())),
            "entity_type" => Some(serde_json::json!(entity.entity_type.to_string())),
            _ => entity.metadata.get(field).cloned(),
        }
    }

    /// Calculate timeliness score based on last sync
    fn calculate_timeliness(&self, entity: &DataEntity) -> f32 {
        if let Some(last_synced) = entity.last_synced_at {
            let days_old = (Utc::now() - last_synced).num_days() as f32;

            // Score based on age:
            // - 1 day or less: 1.0
            // - 7 days: 0.8
            // - 30 days: 0.5
            // - 90+ days: 0.0
            if days_old <= 1.0 {
                1.0
            } else if days_old <= 7.0 {
                1.0 - (days_old - 1.0) * 0.2 / 6.0
            } else if days_old <= 30.0 {
                0.8 - (days_old - 7.0) * 0.3 / 23.0
            } else if days_old <= 90.0 {
                0.5 - (days_old - 30.0) * 0.5 / 60.0
            } else {
                0.0
            }
        } else {
            0.0 // Never synced
        }
    }
}

impl Default for QualityEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// DEFAULT RULE SETS
// ============================================

/// Create default rule sets for common entity types
pub struct DefaultRuleSets;

impl DefaultRuleSets {
    /// Create default rule set for Citizen entities
    pub fn citizen() -> QualityRuleSet {
        let now = Utc::now();
        QualityRuleSet {
            id: "citizen-default".to_string(),
            name: "Citizen Default Rules".to_string(),
            entity_type: EntityType::Citizen,
            rules: vec![
                QualityRule {
                    id: "citizen-name-required".to_string(),
                    name: "Citizen Name Required".to_string(),
                    description: "Citizen must have a name".to_string(),
                    dimension: QualityDimension::Completeness,
                    condition: RuleCondition::Required { field: "name".to_string() },
                    severity: IssueSeverity::Critical,
                    applies_to: vec![EntityType::Citizen],
                    enabled: true,
                    weight: 0.3,
                },
                QualityRule {
                    id: "citizen-email-format".to_string(),
                    name: "Email Format Valid".to_string(),
                    description: "Email must be valid format".to_string(),
                    dimension: QualityDimension::Accuracy,
                    condition: RuleCondition::Email { field: "email".to_string() },
                    severity: IssueSeverity::Medium,
                    applies_to: vec![EntityType::Citizen],
                    enabled: true,
                    weight: 0.2,
                },
                QualityRule {
                    id: "citizen-metadata-complete".to_string(),
                    name: "Citizen Metadata Complete".to_string(),
                    description: "Core metadata fields must be present".to_string(),
                    dimension: QualityDimension::Completeness,
                    condition: RuleCondition::MetadataCompleteness {
                        required_fields: vec!["bsn".to_string(), "geboortedatum".to_string()],
                        threshold: 1.0,
                    },
                    severity: IssueSeverity::High,
                    applies_to: vec![EntityType::Citizen],
                    enabled: true,
                    weight: 0.25,
                },
            ],
            passing_threshold: 0.75,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create default rule set for Organization entities
    pub fn organization() -> QualityRuleSet {
        let now = Utc::now();
        QualityRuleSet {
            id: "organization-default".to_string(),
            name: "Organization Default Rules".to_string(),
            entity_type: EntityType::Organization,
            rules: vec![
                QualityRule {
                    id: "org-name-required".to_string(),
                    name: "Organization Name Required".to_string(),
                    description: "Organization must have a name".to_string(),
                    dimension: QualityDimension::Completeness,
                    condition: RuleCondition::Required { field: "name".to_string() },
                    severity: IssueSeverity::Critical,
                    applies_to: vec![EntityType::Organization],
                    enabled: true,
                    weight: 0.3,
                },
                QualityRule {
                    id: "org-website-url".to_string(),
                    name: "Website URL Valid".to_string(),
                    description: "Website must be valid URL".to_string(),
                    dimension: QualityDimension::Accuracy,
                    condition: RuleCondition::Url { field: "website".to_string() },
                    severity: IssueSeverity::Low,
                    applies_to: vec![EntityType::Organization],
                    enabled: true,
                    weight: 0.1,
                },
                QualityRule {
                    id: "org-kvk-number".to_string(),
                    name: "KvK Number Present".to_string(),
                    description: "KvK number should be present".to_string(),
                    dimension: QualityDimension::Completeness,
                    condition: RuleCondition::Required { field: "kvk_nummer".to_string() },
                    severity: IssueSeverity::High,
                    applies_to: vec![EntityType::Organization],
                    enabled: true,
                    weight: 0.2,
                },
            ],
            passing_threshold: 0.7,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get all default rule sets
    pub fn all() -> Vec<QualityRuleSet> {
        vec![Self::citizen(), Self::organization()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::DataEntity;

    #[test]
    fn test_quality_engine_creation() {
        let engine = QualityEngine::new();
        assert_eq!(engine.default_threshold, 0.7);
    }

    #[test]
    fn test_quality_engine_with_threshold() {
        let engine = QualityEngine::with_threshold(0.8);
        assert_eq!(engine.default_threshold, 0.8);
    }

    #[test]
    fn test_assess_entity() {
        let engine = QualityEngine::new();
        engine.register_rule_set(DefaultRuleSets::citizen());

        let entity = DataEntity::new(
            Uuid::new_v4(),
            "EXT-123".to_string(),
            EntityType::Citizen,
            "John Doe".to_string(),
        );

        let result = engine.assess_entity(&entity);
        assert_eq!(result.entity_id, entity.id);
        assert!(result.assessed_at <= Utc::now());
    }

    #[test]
    fn test_default_rule_sets() {
        let rule_sets = DefaultRuleSets::all();
        assert_eq!(rule_sets.len(), 2);

        let citizen = rule_sets.iter().find(|rs| rs.entity_type == EntityType::Citizen);
        assert!(citizen.is_some());

        let org = rule_sets.iter().find(|rs| rs.entity_type == EntityType::Organization);
        assert!(org.is_some());
    }

    #[test]
    fn test_citizen_rule_set() {
        let rule_set = DefaultRuleSets::citizen();
        assert_eq!(rule_set.entity_type, EntityType::Citizen);
        assert!(!rule_set.rules.is_empty());
    }

    #[test]
    fn test_aggregate_empty() {
        let engine = QualityEngine::new();
        let result = engine.aggregate(&[]);
        assert_eq!(result.entity_count, 0);
    }

    #[test]
    fn test_score_distribution() {
        let distribution = ScoreDistribution {
            excellent: 10,
            good: 20,
            acceptable: 15,
            poor: 5,
        };

        let total = distribution.excellent + distribution.good + distribution.acceptable + distribution.poor;
        assert_eq!(total, 50);
    }
}
