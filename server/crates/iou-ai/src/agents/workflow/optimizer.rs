//! Workflow optimizer agent
//!
//! Provides optimization recommendations based on workflow performance data
//! and best practices.

use super::{WorkflowContext, WorkflowAIError};
use iou_core::ai::ollama::{OllamaClient, OllamaModel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Workflow optimizer
pub struct WorkflowOptimizer {
    ollama: OllamaClient,
}

impl WorkflowOptimizer {
    pub fn new(ollama: OllamaClient) -> Self {
        Self { ollama }
    }

    /// Generate optimization report based on workflow data
    pub async fn create_report(
        &self,
        data: &WorkflowOptimizationData,
    ) -> Result<OptimizationReport, WorkflowAIError> {
        let suggestions = self.generate_suggestions(data).await?;
        let priority = self.calculate_priority(data);
        let estimated_impact = self.estimate_impact(data, &suggestions);

        Ok(OptimizationReport {
            workflow_id: data.workflow_id,
            current_sla_compliance: data.sla_compliance_pct,
            current_avg_hours: data.avg_completion_hours.unwrap_or(0.0),
            target_sla_compliance: self.target_sla(data.sla_compliance_pct),
            target_avg_hours: self.target_avg_hours(data.avg_completion_hours.unwrap_or(0.0)),
            priority,
            suggestions,
            estimated_impact,
            generated_at: Utc::now(),
        })
    }

    /// Generate specific optimization suggestions
    async fn generate_suggestions(
        &self,
        data: &WorkflowOptimizationData,
    ) -> Result<Vec<OptimizationSuggestion>, WorkflowAIError> {
        let mut suggestions = Vec::new();

        // Analyze bottlenecks
        if let Some(ref bottleneck) = data.bottleneck {
            suggestions.extend(self.analyze_bottleneck(bottleneck));
        }

        // Analyze stage insights for issues
        for insight in &data.stage_insights {
            if matches!(insight.risk_level, RiskLevel::High | RiskLevel::Critical) {
                suggestions.push(self.suggest_stage_improvement(insight));
            }
        }

        // Generate AI-powered suggestions if significant issues exist
        if data.sla_compliance_pct < 80.0 {
            let ai_suggestions = self.generate_ai_suggestions(data).await?;
            suggestions.extend(ai_suggestions);
        }

        Ok(suggestions)
    }

    /// Analyze bottleneck and generate suggestions
    fn analyze_bottleneck(&self, bottleneck: &BottleneckInfo) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        match bottleneck.issue_type {
            BottleneckType::CriticalSLABreach => {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: SuggestionType::Urgent,
                    title: format!("Critical SLA breaches in '{}'", bottleneck.stage_name),
                    description: format!(
                        "Stage '{}' meets SLA only {:.1}% of the time with avg duration of {:.1}h",
                        bottleneck.stage_name, bottleneck.sla_compliance_pct, bottleneck.avg_duration_hours
                    ),
                    actions: vec![
                        ActionItem {
                            action: "Extend SLA".to_string(),
                            details: format!("Increase SLA from current to {:.0}h to match actual performance", bottleneck.avg_duration_hours * 1.2),
                            effort: Effort::Low,
                            impact: Impact::High,
                        },
                        ActionItem {
                            action: "Split stage".to_string(),
                            details: "Consider splitting this stage into smaller parallel steps".to_string(),
                            effort: Effort::High,
                            impact: Impact::High,
                        },
                    ],
                });
            }
            BottleneckType::HighDelegationRate => {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: SuggestionType::ProcessImprovement,
                    title: format!("High delegation rate in '{}'", bottleneck.stage_name),
                    description: "Approvers frequently delegate, suggesting unclear responsibility or workload issues".to_string(),
                    actions: vec![
                        ActionItem {
                            action: "Clarify approver responsibilities".to_string(),
                            details: "Review and update approver role definitions".to_string(),
                            effort: Effort::Medium,
                            impact: Impact::Medium,
                        },
                        ActionItem {
                            action: "Add approvers".to_string(),
                            details: "Increase pool of approvers to distribute workload".to_string(),
                            effort: Effort::Low,
                            impact: Impact::Medium,
                        },
                    ],
                });
            }
            BottleneckType::HighRejectionRate => {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: SuggestionType::ProcessImprovement,
                    title: format!("High rejection rate in '{}'", bottleneck.stage_name),
                    description: "Documents frequently rejected at this stage, indicating upstream quality issues".to_string(),
                    actions: vec![
                        ActionItem {
                            action: "Add pre-validation".to_string(),
                            details: "Implement validation checks before documents reach this stage".to_string(),
                            effort: Effort::Medium,
                            impact: Impact::High,
                        },
                        ActionItem {
                            action: "Review rejection reasons".to_string(),
                            details: "Analyze common rejection patterns and address root causes".to_string(),
                            effort: Effort::Low,
                            impact: Impact::Medium,
                        },
                    ],
                });
            }
            _ => {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: SuggestionType::Performance,
                    title: format!("Slow approval in '{}'", bottleneck.stage_name),
                    description: format!("Average duration of {:.1}h exceeds typical SLA", bottleneck.avg_duration_hours),
                    actions: vec![
                        ActionItem {
                            action: "Enable parallel approval".to_string(),
                            details: "Convert to parallel_any or parallel_all if applicable".to_string(),
                            effort: Effort::Medium,
                            impact: Impact::Medium,
                        },
                        ActionItem {
                            action: "Set reminder notifications".to_string(),
                            details: "Configure automatic reminders at 50% and 75% of SLA".to_string(),
                            effort: Effort::Low,
                            impact: Impact::Low,
                        },
                    ],
                });
            }
        }

        suggestions
    }

    /// Suggest improvement for a specific stage
    fn suggest_stage_improvement(&self, insight: &StageInsight) -> OptimizationSuggestion {
        OptimizationSuggestion {
            suggestion_type: match insight.risk_level {
                RiskLevel::Critical => SuggestionType::Urgent,
                RiskLevel::High => SuggestionType::HighImpact,
                _ => SuggestionType::Performance,
            },
            title: format!("Improve '{}' stage performance", insight.stage_name),
            description: insight.recommendation.clone(),
            actions: vec![
                ActionItem {
                    action: "Review SLA settings".to_string(),
                    details: "Ensure SLA aligns with actual processing capacity".to_string(),
                    effort: Effort::Low,
                    impact: Impact::Medium,
                },
            ],
        }
    }

    /// Generate AI-powered optimization suggestions
    async fn generate_ai_suggestions(
        &self,
        data: &WorkflowOptimizationData,
    ) -> Result<Vec<OptimizationSuggestion>, WorkflowAIError> {
        let prompt = self.build_optimization_prompt(data);

        let response = self
            .ollama
            .generate(&prompt, OllamaModel::Llama3_2)
            .await
            .map_err(|e| WorkflowAIError::LLM(e.to_string()))?;

        let ai_suggestions: Vec<AIOptimizationSuggestion> = serde_json::from_str(&response)
            .unwrap_or_default();

        Ok(ai_suggestions.into_iter().map(|s| OptimizationSuggestion {
            suggestion_type: SuggestionType::AISuggested,
            title: s.title,
            description: s.description,
            actions: s.actions.into_iter().map(|a| ActionItem {
                action: a.action,
                details: a.details,
                effort: match a.effort.as_str() {
                    "low" => Effort::Low,
                    "medium" => Effort::Medium,
                    _ => Effort::High,
                },
                impact: match a.impact.as_str() {
                    "high" => Impact::High,
                    "medium" => Impact::Medium,
                    _ => Impact::Low,
                },
            }).collect(),
        }).collect())
    }

    /// Build prompt for AI optimization suggestions
    fn build_optimization_prompt(&self, data: &WorkflowOptimizationData) -> String {
        format!(
            r#"Analyze this workflow and provide optimization suggestions.

Workflow Stats:
- Stages: {}
- SLA Compliance: {:.1}%
- Avg Completion: {:.1}h
- Executions: {}

Bottleneck: {}

Provide 1-2 specific optimization suggestions in JSON:
[
  {{
    "title": "Short descriptive title",
    "description": "Full explanation",
    "actions": [
      {{
        "action": "Action name",
        "details": "Implementation details",
        "effort": "low|medium|high",
        "impact": "low|medium|high"
      }}
    ]
  }}
]"#,
            data.total_stages,
            data.sla_compliance_pct,
            data.avg_completion_hours.unwrap_or(0.0),
            data.total_executions,
            data.bottleneck.as_ref().map(|b| &b.stage_name).unwrap_or("None")
        )
    }

    /// Calculate priority level for optimization
    fn calculate_priority(&self, data: &WorkflowOptimizationData) -> Priority {
        if data.sla_compliance_pct < 50.0 {
            Priority::Critical
        } else if data.sla_compliance_pct < 75.0 {
            Priority::High
        } else if data.sla_compliance_pct < 90.0 {
            Priority::Medium
        } else {
            Priority::Low
        }
    }

    /// Estimate impact of implementing suggestions
    fn estimate_impact(&self, data: &WorkflowOptimizationData, _suggestions: &[OptimizationSuggestion]) -> ImpactEstimate {
        let potential_sla_improvement = 10.0; // Simplified estimate
        let current_sla = data.sla_compliance_pct;
        let target_sla = (current_sla + potential_sla_improvement).min(100.0);

        let potential_time_reduction = data.avg_completion_hours.unwrap_or(0.0) * 0.2;

        ImpactEstimate {
            current_sla_compliance_pct: current_sla,
            projected_sla_compliance_pct: target_sla,
            current_avg_hours: data.avg_completion_hours.unwrap_or(0.0),
            projected_avg_hours: data.avg_completion_hours.unwrap_or(0.0) - potential_time_reduction,
            time_to_implement_weeks: 4,
        }
    }

    /// Calculate target SLA compliance
    fn target_sla(&self, current: f64) -> f64 {
        (current + 10.0).min(100.0)
    }

    /// Calculate target average completion hours
    fn target_avg_hours(&self, current: f64) -> f64 {
        if current == 0.0 { 48.0 } else { (current * 0.8).max(24.0) }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Data for workflow optimization (passed from service layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOptimizationData {
    pub workflow_id: Uuid,
    pub total_stages: usize,
    pub avg_completion_hours: Option<f64>,
    pub sla_compliance_pct: f64,
    pub bottleneck: Option<BottleneckInfo>,
    pub stage_insights: Vec<StageInsight>,
    pub total_executions: i32,
}

/// Complete optimization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub workflow_id: Uuid,
    pub current_sla_compliance: f64,
    pub current_avg_hours: f64,
    pub target_sla_compliance: f64,
    pub target_avg_hours: f64,
    pub priority: Priority,
    pub suggestions: Vec<OptimizationSuggestion>,
    pub estimated_impact: ImpactEstimate,
    pub generated_at: DateTime<Utc>,
}

/// Single optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: SuggestionType,
    pub title: String,
    pub description: String,
    pub actions: Vec<ActionItem>,
}

/// Type of suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    Urgent,
    HighImpact,
    Performance,
    ProcessImprovement,
    AISuggested,
}

/// Action item within a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub action: String,
    pub details: String,
    pub effort: Effort,
    pub impact: Impact,
}

/// Effort level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Effort {
    Low,
    Medium,
    High,
}

/// Impact level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Impact {
    Low,
    Medium,
    High,
}

/// Priority level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Impact estimate for implementing suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEstimate {
    pub current_sla_compliance_pct: f64,
    pub projected_sla_compliance_pct: f64,
    pub current_avg_hours: f64,
    pub projected_avg_hours: f64,
    pub time_to_implement_weeks: i32,
}

/// Bottleneck information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckInfo {
    pub stage_id: String,
    pub stage_name: String,
    pub avg_duration_hours: f64,
    pub sla_compliance_pct: f64,
    pub issue_type: BottleneckType,
}

/// Type of bottleneck issue
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BottleneckType {
    CriticalSLABreach,
    LongDelay,
    HighDelegationRate,
    HighRejectionRate,
    SlowApproval,
}

/// Stage insight with recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageInsight {
    pub stage_id: String,
    pub stage_name: String,
    pub avg_duration_hours: Option<f64>,
    pub sla_compliance_pct: Option<f64>,
    pub risk_level: RiskLevel,
    pub recommendation: String,
}

/// Risk level assessment
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    Unknown,
    Low,
    Medium,
    High,
    Critical,
}

/// AI-generated optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AIOptimizationSuggestion {
    title: String,
    description: String,
    actions: Vec<AIActionItem>,
}

/// Action item from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AIActionItem {
    action: String,
    details: String,
    effort: String,
    impact: String,
}
