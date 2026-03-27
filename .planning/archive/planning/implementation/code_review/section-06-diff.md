diff --git a/crates/iou-ai/src/agents/mod.rs b/crates/iou-ai/src/agents/mod.rs
index a4bc2a9..d822858 100644
--- a/crates/iou-ai/src/agents/mod.rs
+++ b/crates/iou-ai/src/agents/mod.rs
@@ -7,6 +7,7 @@ pub mod config;
 pub mod research;
 pub mod content;
 pub mod compliance;
+pub mod review;
 
 use thiserror::Error;
 
@@ -39,6 +40,9 @@ pub enum AgentError {
 
     #[error("Compliance check failed: {0}")]
     ComplianceError(String),
+
+    #[error("Review check failed: {0}")]
+    ReviewError(String),
 }
 
 // Re-export agent types
@@ -48,4 +52,8 @@ pub use compliance::{
     ComplianceResult, execute_compliance_agent, ComplianceConfig,
     PiiLocation, PiiType, AccessibilityIssue, AccessibilityLevel
 };
+pub use review::{
+    ReviewDecision, ReviewAction, execute_review_agent, ReviewConfig,
+    QualityIssue, QualityIssueCategory,
+};
 pub use config::{AgentConfig, ResearchAgentConfig};
diff --git a/crates/iou-ai/src/agents/review.rs b/crates/iou-ai/src/agents/review.rs
new file mode 100644
index 0000000..57281ae
--- /dev/null
+++ b/crates/iou-ai/src/agents/review.rs
@@ -0,0 +1,777 @@
+//! Review Agent
+//!
+//! This agent performs the final quality assurance check on generated documents.
+//! It validates completeness, clarity, and compliance scores before determining
+//! whether a document is ready for human approval or requires revision.
+//!
+//! # Critical Security Constraint
+//!
+//! **ALL Woo-relevant documents require human approval regardless of confidence score.**
+//! Auto-approval only applies to internal, non-sensitive documents.
+
+use crate::agents::{AgentError, content::GeneratedDocument, research::ResearchContext, compliance::ComplianceResult};
+use chrono::{DateTime, Utc};
+use iou_core::compliance::{IssueSeverity, WooRefusalGround};
+use serde::{Deserialize, Serialize};
+use uuid::Uuid;
+
+/// Quality issues found during review
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct QualityIssue {
+    /// Issue category
+    pub category: QualityIssueCategory,
+    /// Severity level
+    pub severity: IssueSeverity,
+    /// Human-readable description
+    pub description: String,
+    /// Section reference (if applicable)
+    pub location: Option<String>,
+    /// Suggested fix
+    pub suggested_fix: Option<String>,
+}
+
+/// Categories of quality issues
+#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
+pub enum QualityIssueCategory {
+    /// A required section is missing
+    MissingRequiredSection,
+    /// Phrasing is unclear or ambiguous
+    UnclearPhrasing,
+    /// Terminology is inconsistent
+    InconsistentTerminology,
+    /// Compliance score is too low
+    LowComplianceScore,
+    /// PII was detected but not redacted
+    PiiNotRedacted,
+    /// WCAG accessibility issue
+    AccessibilityIssue,
+    /// Document structure problem
+    StructuralProblem,
+    /// Placeholder text remains
+    PlaceholderText,
+}
+
+/// Review decision output
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ReviewDecision {
+    /// Action to take
+    pub action: ReviewAction,
+
+    /// Whether auto-approval is allowed (never true for Woo documents)
+    pub can_auto_approve: bool,
+
+    /// Overall quality score (0.0 - 1.0)
+    pub overall_quality_score: f32,
+
+    /// Completeness issues found
+    pub completeness_issues: Vec<QualityIssue>,
+
+    /// Clarity issues found
+    pub clarity_issues: Vec<QualityIssue>,
+
+    /// Human-readable feedback summary
+    pub feedback: String,
+
+    /// Suggested improvements
+    pub suggested_improvements: Vec<String>,
+
+    /// When the review was completed
+    pub reviewed_at: DateTime<Utc>,
+}
+
+/// Possible review actions
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+pub enum ReviewAction {
+    /// Document approved (can be auto-approved or sent for human approval)
+    Approve,
+    /// Requires human review before approval
+    RequireHumanApproval,
+    /// Send back to Content Agent for revision
+    RequestRevision,
+    /// Document rejected (critical issues)
+    Reject,
+}
+
+/// Configuration for review behavior
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ReviewConfig {
+    /// Minimum quality score for approval (0.0 - 1.0)
+    pub min_quality_score: f32,
+
+    /// Threshold for auto-approval (only non-Woo documents)
+    pub auto_approval_threshold: f32,
+
+    /// Whether to enable completeness checks
+    pub enable_completeness_check: bool,
+
+    /// Whether to enable clarity checks
+    pub enable_clarity_check: bool,
+}
+
+impl Default for ReviewConfig {
+    fn default() -> Self {
+        Self {
+            min_quality_score: 0.75,
+            auto_approval_threshold: 0.95,
+            enable_completeness_check: true,
+            enable_clarity_check: true,
+        }
+    }
+}
+
+/// Executes the Review Agent
+///
+/// This agent performs the final quality check before human approval.
+///
+/// # Arguments
+/// * `document` - The generated document from Content Agent
+/// * `compliance` - Compliance validation result from Compliance Agent
+/// * `research` - Research context from Research Agent
+///
+/// # Returns
+/// `ReviewDecision` with action, quality score, and feedback
+pub async fn execute_review_agent(
+    document: &GeneratedDocument,
+    compliance: &ComplianceResult,
+    research: &ResearchContext,
+) -> Result<ReviewDecision, AgentError> {
+    let config = ReviewConfig::default();
+    execute_review_agent_with_config(document, compliance, research, &config).await
+}
+
+/// Executes the Review Agent with custom configuration
+pub async fn execute_review_agent_with_config(
+    document: &GeneratedDocument,
+    compliance: &ComplianceResult,
+    research: &ResearchContext,
+    config: &ReviewConfig,
+) -> Result<ReviewDecision, AgentError> {
+    // 1. Check completeness - all required sections present
+    let completeness_issues = if config.enable_completeness_check {
+        check_completeness(document, research)?
+    } else {
+        Vec::new()
+    };
+
+    // 2. Validate compliance score meets threshold
+    let compliance_issues = check_compliance_threshold(compliance, config);
+
+    // 3. Check clarity and consistency
+    let clarity_issues = if config.enable_clarity_check {
+        check_clarity(document)?
+    } else {
+        Vec::new()
+    };
+
+    // 4. Aggregate all issues
+    let all_issues: Vec<QualityIssue> = completeness_issues
+        .into_iter()
+        .chain(compliance_issues)
+        .chain(clarity_issues)
+        .collect();
+
+    // 5. Calculate overall quality score
+    let quality_score = calculate_quality_score(compliance, &all_issues);
+
+    // 6. Determine review action
+    let action = determine_review_action(
+        document,
+        compliance,
+        config,
+        quality_score,
+        &all_issues,
+    );
+
+    // 7. Generate feedback and suggestions
+    let (feedback, suggestions) = generate_feedback(&action, &all_issues, quality_score);
+
+    // 8. Determine if auto-approval is possible
+    let can_auto_approve = can_auto_approve(document, compliance, config, quality_score);
+
+    // Separate issues by category
+    let completeness_issues: Vec<_> = all_issues
+        .iter()
+        .filter(|i| matches!(
+            i.category,
+            QualityIssueCategory::MissingRequiredSection
+                | QualityIssueCategory::LowComplianceScore
+                | QualityIssueCategory::AccessibilityIssue
+        ))
+        .cloned()
+        .collect();
+
+    let clarity_issues: Vec<_> = all_issues
+        .iter()
+        .filter(|i| matches!(
+            i.category,
+            QualityIssueCategory::UnclearPhrasing
+                | QualityIssueCategory::InconsistentTerminology
+                | QualityIssueCategory::PlaceholderText
+                | QualityIssueCategory::StructuralProblem
+        ))
+        .cloned()
+        .collect();
+
+    Ok(ReviewDecision {
+        action,
+        can_auto_approve,
+        overall_quality_score: quality_score,
+        completeness_issues,
+        clarity_issues,
+        feedback,
+        suggested_improvements: suggestions,
+        reviewed_at: Utc::now(),
+    })
+}
+
+/// Check if all required sections are present
+fn check_completeness(
+    document: &GeneratedDocument,
+    research: &ResearchContext,
+) -> Result<Vec<QualityIssue>, AgentError> {
+    let mut issues = Vec::new();
+
+    // Check mandatory sections
+    for required_section in &research.mandatory_sections {
+        let content_lower = document.content.to_lowercase();
+        let section_lower = required_section.to_lowercase();
+
+        // Try to find the section in the document content
+        // Look for headings that match the section name (at any heading level)
+        let found = content_lower.contains(&format!("# {}", section_lower))
+            || content_lower.contains(&format!("## {}", section_lower))
+            || content_lower.contains(&format!("### {}", section_lower));
+
+        // Also check in the sections metadata if available
+        let found_in_metadata = document.sections.iter()
+            .any(|s| s.name.eq_ignore_ascii_case(required_section));
+
+        if !found && !found_in_metadata {
+            issues.push(QualityIssue {
+                category: QualityIssueCategory::MissingRequiredSection,
+                severity: IssueSeverity::High,
+                description: format!("Required section '{}' is missing", required_section),
+                location: Some(format!("Section: {}", required_section)),
+                suggested_fix: Some(format!("Add section '{}'", required_section)),
+            });
+        }
+    }
+
+    Ok(issues)
+}
+
+/// Check if compliance score meets threshold
+fn check_compliance_threshold(
+    compliance: &ComplianceResult,
+    config: &ReviewConfig,
+) -> Vec<QualityIssue> {
+    let mut issues = Vec::new();
+
+    if compliance.score < config.min_quality_score {
+        let severity = if compliance.score < config.min_quality_score * 0.7 {
+            IssueSeverity::Critical
+        } else {
+            IssueSeverity::High
+        };
+
+        issues.push(QualityIssue {
+            category: QualityIssueCategory::LowComplianceScore,
+            severity,
+            description: format!(
+                "Compliance score ({:.2}) below threshold ({:.2})",
+                compliance.score, config.min_quality_score
+            ),
+            location: None,
+            suggested_fix: Some("Review compliance issues and address refusal grounds".to_string()),
+        });
+    }
+
+    // Check for Woo refusal grounds (these are serious)
+    if !compliance.refusal_grounds.is_empty() {
+        issues.push(QualityIssue {
+            category: QualityIssueCategory::LowComplianceScore,
+            severity: IssueSeverity::High,
+            description: format!(
+                "Woo refusal grounds detected: {}",
+                compliance.refusal_grounds.len()
+            ),
+            location: None,
+            suggested_fix: Some("Review Woo compliance requirements".to_string()),
+        });
+    }
+
+    // Check for accessibility issues
+    if !compliance.accessibility_issues.is_empty() {
+        issues.push(QualityIssue {
+            category: QualityIssueCategory::AccessibilityIssue,
+            severity: IssueSeverity::Medium,
+            description: format!(
+                "{} WCAG accessibility issues found",
+                compliance.accessibility_issues.len()
+            ),
+            location: None,
+            suggested_fix: Some("Review and fix accessibility issues".to_string()),
+        });
+    }
+
+    issues
+}
+
+/// Check document clarity and consistency
+fn check_clarity(document: &GeneratedDocument) -> Result<Vec<QualityIssue>, AgentError> {
+    let mut issues = Vec::new();
+    let content = &document.content;
+
+    // Check for placeholder text
+    let placeholders = [
+        "[TODO]",
+        "[PLACEHOLDER]",
+        "[TEE HIER]",
+        "[VUL IN]",
+        "[NOT SPECIFIED]",
+        "[NADER TE BEPALEN]",
+    ];
+
+    for placeholder in &placeholders {
+        if content.contains(placeholder) {
+            issues.push(QualityIssue {
+                category: QualityIssueCategory::PlaceholderText,
+                severity: IssueSeverity::Medium,
+                description: format!("Document contains placeholder: {}", placeholder),
+                location: None,
+                suggested_fix: Some(format!("Replace '{}' with actual content", placeholder)),
+            });
+        }
+    }
+
+    // Check for missing title/heading
+    if !content.starts_with('#') {
+        issues.push(QualityIssue {
+            category: QualityIssueCategory::StructuralProblem,
+            severity: IssueSeverity::Low,
+            description: "Document missing main heading".to_string(),
+            location: None,
+            suggested_fix: Some("Add a main heading (# Title) to the document".to_string()),
+        });
+    }
+
+    Ok(issues)
+}
+
+/// Calculate overall quality score from compliance and issues
+fn calculate_quality_score(
+    compliance: &ComplianceResult,
+    issues: &[QualityIssue],
+) -> f32 {
+    let base_score = compliance.score;
+
+    // Penalize for quality issues
+    let penalty: f32 = issues
+        .iter()
+        .map(|i| match i.severity {
+            IssueSeverity::Critical => 0.25,
+            IssueSeverity::High => 0.15,
+            IssueSeverity::Medium => 0.05,
+            IssueSeverity::Low => 0.01,
+        })
+        .sum();
+
+    (base_score - penalty).max(0.0).min(1.0)
+}
+
+/// Determine the appropriate review action
+fn determine_review_action(
+    document: &GeneratedDocument,
+    compliance: &ComplianceResult,
+    config: &ReviewConfig,
+    quality_score: f32,
+    issues: &[QualityIssue],
+) -> ReviewAction {
+    // Check for critical issues - always reject
+    let has_critical = issues.iter().any(|i| matches!(i.severity, IssueSeverity::Critical));
+    if has_critical {
+        return ReviewAction::Reject;
+    }
+
+    // Check for unredacted PII - reject
+    // Note: In production, PII should be redacted by Compliance Agent
+    // If we have PII detected but no redacted content, that's a problem
+    if !compliance.pii_detected.is_empty() && compliance.redacted_content.is_none() {
+        return ReviewAction::Reject;
+    }
+
+    // CRITICAL: ALL Woo documents require human approval
+    // Check this BEFORE the high severity check
+    if requires_human_approval(document, compliance) {
+        return ReviewAction::RequireHumanApproval;
+    }
+
+    // Check for high severity issues - request revision
+    let has_high_issues = issues.iter().any(|i| matches!(i.severity, IssueSeverity::High));
+    if has_high_issues {
+        return ReviewAction::RequestRevision;
+    }
+
+    // Check if quality is acceptable
+    if quality_score < config.min_quality_score {
+        return ReviewAction::RequestRevision;
+    }
+
+    // Document passes all checks
+    ReviewAction::Approve
+}
+
+/// Check if document requires human approval
+fn requires_human_approval(
+    document: &GeneratedDocument,
+    compliance: &ComplianceResult,
+) -> bool {
+    // CRITICAL: ALL Woo documents require human approval
+    // Check if any Woo refusal grounds are present
+    if !compliance.refusal_grounds.is_empty() {
+        return true;
+    }
+
+    // Check document metadata if available (for future expansion)
+    // For now, we assume Woo relevance if refusal grounds exist
+
+    false
+}
+
+/// Check if auto-approval is allowed
+fn can_auto_approve(
+    document: &GeneratedDocument,
+    compliance: &ComplianceResult,
+    config: &ReviewConfig,
+    quality_score: f32,
+) -> bool {
+    // NEVER auto-approve Woo documents
+    if !compliance.refusal_grounds.is_empty() {
+        return false;
+    }
+
+    // Auto-approve only if quality exceeds threshold
+    quality_score >= config.auto_approval_threshold
+}
+
+/// Generate human-readable feedback and suggestions
+fn generate_feedback(
+    action: &ReviewAction,
+    issues: &[QualityIssue],
+    quality_score: f32,
+) -> (String, Vec<String>) {
+    let mut suggestions = Vec::new();
+
+    for issue in issues {
+        if let Some(fix) = &issue.suggested_fix {
+            suggestions.push(fix.clone());
+        }
+    }
+
+    let feedback = match action {
+        ReviewAction::Approve => {
+            format!(
+                "Document approved with quality score {:.2}. No critical issues found.",
+                quality_score
+            )
+        }
+        ReviewAction::RequireHumanApproval => {
+            format!(
+                "Document requires human approval. Quality score: {:.2}. {} issue(s) found requiring review.",
+                quality_score,
+                issues.len()
+            )
+        }
+        ReviewAction::RequestRevision => {
+            let top_issue = issues
+                .first()
+                .map(|i| i.description.as_str())
+                .unwrap_or("See details");
+            format!(
+                "Document requires revision. Found {} issue(s). Top priority: {}",
+                issues.len(),
+                top_issue
+            )
+        }
+        ReviewAction::Reject => {
+            let critical_issues: Vec<_> = issues
+                .iter()
+                .filter(|i| matches!(i.severity, IssueSeverity::Critical))
+                .map(|i| i.description.as_str())
+                .collect();
+            format!(
+                "Document rejected due to critical issues. {}",
+                critical_issues.join("; ")
+            )
+        }
+    };
+
+    (feedback, suggestions)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn create_test_document(content: &str) -> GeneratedDocument {
+        GeneratedDocument {
+            document_id: Uuid::new_v4(),
+            content: content.to_string(),
+            variables: vec![],
+            entity_links: vec![],
+            sections: vec![],
+            generated_at: Utc::now(),
+        }
+    }
+
+    fn create_test_research() -> ResearchContext {
+        ResearchContext {
+            document_type: "test".to_string(),
+            domain_context: crate::agents::research::DomainContext {
+                domain_id: "test-domain".to_string(),
+                domain_name: "Test Domain".to_string(),
+                organizational_unit: None,
+                applicable_regulations: vec![],
+                standard_procedures: vec![],
+            },
+            mandatory_sections: vec![],  // Default: no mandatory sections
+            optional_sections: vec![],
+            suggested_sections: vec![],
+            similar_documents: vec![],
+            related_entities: vec![],
+            provisa_guidelines: vec![],
+            suggested_variables: vec![],
+            timestamp: Utc::now(),
+        }
+    }
+
+    fn create_test_compliance(score: f32) -> ComplianceResult {
+        ComplianceResult {
+            is_compliant: score >= 0.8,
+            score,
+            refusal_grounds: vec![],
+            pii_detected: vec![],
+            accessibility_issues: vec![],
+            issues: vec![],
+            redacted_content: None,
+            assessed_at: Utc::now(),
+            original_storage_key: None,
+            redacted_storage_key: None,
+        }
+    }
+
+    #[test]
+    fn test_review_config_default() {
+        let config = ReviewConfig::default();
+        assert_eq!(config.min_quality_score, 0.75);
+        assert_eq!(config.auto_approval_threshold, 0.95);
+        assert!(config.enable_completeness_check);
+        assert!(config.enable_clarity_check);
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_completeness_check_pass() {
+        let document = create_test_document("# Test\n\n## Introduction\n\nContent.\n\n## Conclusion\n\nEnd.");
+        let research = ResearchContext {
+            mandatory_sections: vec!["Introduction".to_string(), "Conclusion".to_string()],
+            ..create_test_research()
+        };
+        let compliance = create_test_compliance(0.9);
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Should have no completeness issues
+        assert!(result.completeness_issues.is_empty());
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_completeness_check_fail() {
+        let document = create_test_document("# Test\n\nSome content but missing sections.");
+        let research = ResearchContext {
+            mandatory_sections: vec!["Introduction".to_string(), "Conclusion".to_string()],
+            ..create_test_research()
+        };
+        let compliance = create_test_compliance(0.9);
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Should have completeness issues for missing Introduction and Conclusion
+        assert!(!result.completeness_issues.is_empty());
+        assert!(result
+            .completeness_issues
+            .iter()
+            .any(|i| i.category == QualityIssueCategory::MissingRequiredSection));
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_compliance_threshold_pass() {
+        let document = create_test_document("# Test\n\nContent.");
+        let research = create_test_research();
+        let compliance = create_test_compliance(0.85);
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Should not have low compliance score issues (0.85 >= 0.75 threshold)
+        assert!(!result
+            .completeness_issues
+            .iter()
+            .any(|i| i.category == QualityIssueCategory::LowComplianceScore));
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_compliance_threshold_fail() {
+        let document = create_test_document("# Test\n\nContent.");
+        let research = create_test_research();
+        let compliance = create_test_compliance(0.5);  // Below 0.75 threshold
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Should have low compliance score issue in completeness_issues
+        assert!(!result.completeness_issues.is_empty(),
+            "Expected some completeness issues for low compliance score");
+        assert!(result
+            .completeness_issues
+            .iter()
+            .any(|i| i.category == QualityIssueCategory::LowComplianceScore),
+            "Expected LowComplianceScore issue");
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_woo_requires_human_approval() {
+        let document = create_test_document("# Test\n\nContent.");
+        let research = ResearchContext {
+            mandatory_sections: vec![],
+            ..create_test_research()
+        };
+
+        // Create compliance with Woo refusal grounds
+        let mut compliance = create_test_compliance(0.98);
+        compliance.refusal_grounds = vec![WooRefusalGround::PersoonlijkeLevenssfeer];
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Woo documents should NEVER auto-approve
+        assert!(!result.can_auto_approve, "Woo documents must never auto-approve");
+        assert_eq!(result.action, ReviewAction::RequireHumanApproval);
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_non_woo_can_auto_approve() {
+        let document = create_test_document("# Test\n\nContent.");
+        let research = ResearchContext {
+            mandatory_sections: vec![],
+            ..create_test_research()
+        };
+        let compliance = create_test_compliance(0.98);  // High score, no Woo issues
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Non-Woo documents with high scores can auto-approve
+        assert!(result.can_auto_approve,
+            "Non-Woo document with 0.98 score should auto-approve. Got quality_score: {}", result.overall_quality_score);
+        assert_eq!(result.action, ReviewAction::Approve);
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_detects_placeholder_text() {
+        let document = create_test_document("# Test\n\n[TODO] Add content here.");
+        let research = ResearchContext {
+            mandatory_sections: vec![],
+            ..create_test_research()
+        };
+        let compliance = create_test_compliance(0.9);
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Should detect placeholder text
+        assert!(result
+            .clarity_issues
+            .iter()
+            .any(|i| i.category == QualityIssueCategory::PlaceholderText));
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_missing_heading() {
+        let document = create_test_document("Test document without heading.");
+        let research = ResearchContext {
+            mandatory_sections: vec![],
+            ..create_test_research()
+        };
+        let compliance = create_test_compliance(0.9);
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Should detect missing heading
+        assert!(result
+            .clarity_issues
+            .iter()
+            .any(|i| i.category == QualityIssueCategory::StructuralProblem));
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_quality_score_calculation() {
+        let document = create_test_document("# Test\n\nContent.");
+        let research = ResearchContext {
+            mandatory_sections: vec![],
+            ..create_test_research()
+        };
+        let compliance = create_test_compliance(0.9);
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Quality score should be close to compliance score (no issues)
+        assert!(result.overall_quality_score >= 0.85,
+            "Quality score {} should be >= 0.85", result.overall_quality_score);
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_feedback_generation() {
+        let document = create_test_document("# Test\n\nContent.");
+        let research = ResearchContext {
+            mandatory_sections: vec![],
+            ..create_test_research()
+        };
+        let compliance = create_test_compliance(0.98);
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Feedback should mention quality score
+        assert!(result.feedback.contains("0.9") || result.feedback.contains("1.0"));
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_request_revision_for_low_score() {
+        let document = create_test_document("# Test\n\nContent.");
+        let research = ResearchContext {
+            mandatory_sections: vec![],
+            ..create_test_research()
+        };
+        let compliance = create_test_compliance(0.6);  // Low score
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Should request revision
+        assert_eq!(result.action, ReviewAction::RequestRevision);
+    }
+
+    #[tokio::test]
+    async fn test_review_action_equality() {
+        assert_eq!(ReviewAction::Approve, ReviewAction::Approve);
+        assert_ne!(ReviewAction::Approve, ReviewAction::Reject);
+    }
+
+    #[tokio::test]
+    async fn test_review_agent_suggested_improvements() {
+        let document = create_test_document("# Test\n\n[TODO] fix this.");
+        let research = ResearchContext {
+            mandatory_sections: vec![],
+            ..create_test_research()
+        };
+        let compliance = create_test_compliance(0.9);
+
+        let result = execute_review_agent(&document, &compliance, &research).await.unwrap();
+
+        // Should have suggested improvements
+        assert!(!result.suggested_improvements.is_empty());
+    }
+}
