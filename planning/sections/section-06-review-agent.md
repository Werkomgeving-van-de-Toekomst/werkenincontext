Now I have all the context needed to create the section content. Let me generate the self-contained section for the Review Agent:

# Section 6: Review Agent

## Overview

The Review Agent is the final quality assurance component in the document creation pipeline. It performs completeness checks, clarity validation, and compliance score verification before determining whether a document is ready for human approval or requires revision.

This section is implementable in parallel with section-03 (Research Agent) and section-05 (Compliance Agent), as they only depend on section-01 (Foundation). The Review Agent receives output from both the Content Agent and Compliance Agent.

### Key Responsibilities

1. **Completeness Checks**: Verify all required sections are present based on document type
2. **Clarity Validation**: Assess document clarity, consistency, and readability
3. **Compliance Score Verification**: Ensure the document meets the compliance threshold
4. **Approval Decision Logic**: Determine if auto-approval is possible or human review is required
5. **Iteration Request**: Generate specific feedback for content revision when quality issues are found

### Critical Security Constraint

**ALL Woo-relevant documents require human approval regardless of confidence score.** Auto-approval only applies to internal, non-sensitive documents where legal compliance is not a concern.

## File Structure

```
/Users/marc/Projecten/iou-modern/
├── crates/
│   ├── iou-core/src/
│   │   └── document.rs              # NEW: Document domain types (from section-01)
│   ├── iou-ai/src/
│   │   └── agents/
│   │       └── review.rs            # NEW: Review Agent implementation
└── templates/
    └── section-06-review-tests.md   # Reference: Test specifications
```

## Dependencies

- **section-01-foundation**: Core domain types (`DocumentRequest`, `DomainConfig`, `TrustLevel`), `WorkflowStatus` from `iou-core/src/workflows.rs`, compliance types from `iou-core/src/compliance.rs`
- **section-02-template-system**: `Template` and `GeneratedDocument` types
- **section-03-research-agent**: `ResearchContext` for document structure requirements
- **section-04-content-agent**: `GeneratedDocument` output to review
- **section-05-compliance-agent**: `ComplianceResult` with scores and PII information

## Tests

The following tests should be implemented in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/review.rs` in a `#[cfg(test)]` module.

### 4.4 Review Agent Tests

**Test: execute_review_agent checks completeness of required sections**
```rust
#[tokio::test]
async fn test_review_agent_completeness_check() {
    // Given: A document with all required sections present
    let document = GeneratedDocument {
        content: "# Test Document\n\n## Required Section\n\nContent here.",
        sections: vec!["Required Section".to_string()],
        metadata: DocumentMetadata::default(),
    };
    
    let research_context = ResearchContext {
        required_sections: vec!["Required Section".to_string()],
        optional_sections: vec![],
        // ... other fields
    };
    
    let compliance = ComplianceResult {
        is_compliant: true,
        score: 0.95,
        // ... other fields
    };
    
    // When: Executing review agent
    let result = execute_review_agent(&document, &compliance, &research_context, &domain_config).await;
    
    // Then: Completeness check passes
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert!(!decision.completeness_issues.is_empty());
}
```

**Test: execute_review_agent validates compliance score meets threshold**
```rust
#[tokio::test]
async fn test_review_agent_compliance_threshold() {
    // Given: A document with low compliance score
    let document = GeneratedDocument::default();
    let compliance = ComplianceResult {
        is_compliant: false,
        score: 0.6,  // Below threshold
        refusal_grounds: vec![],
        pii_detected: vec![],
        accessibility_issues: vec![],
    };
    let domain_config = DomainConfig {
        trust_level: TrustLevel::Medium,
        required_approval_threshold: 0.8,
        auto_approval_threshold: 0.95,
    };
    
    // When: Executing review agent
    let result = execute_review_agent(&document, &compliance, &domain_config).await;
    
    // Then: Should require iteration or rejection
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert!(matches!(decision.action, ReviewAction::RequestRevision | ReviewAction::Reject));
}
```

**Test: execute_review_agent requires human approval for ALL Woo documents**
```rust
#[tokio::test]
async fn test_review_agent_woo_requires_human_approval() {
    // Given: A Woo-relevant document with high scores
    let document = GeneratedDocument {
        metadata: DocumentMetadata {
            woo_relevant: true,
            // ... other fields
        },
        // ... other fields
    };
    let compliance = ComplianceResult {
        is_compliant: true,
        score: 0.98,  // Very high score
        // ... other fields
    };
    let domain_config = DomainConfig {
        trust_level: TrustLevel::High,
        // ... other fields
    };
    
    // When: Executing review agent
    let result = execute_review_agent(&document, &compliance, &domain_config).await;
    
    // Then: Should ALWAYS require human approval for Woo documents
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert!(!decision.can_auto_approve, "Woo documents must never auto-approve");
    assert!(matches!(decision.action, ReviewAction::RequireHumanApproval));
}
```

**Test: execute_review_agent allows auto-approval only for non-Woo documents**
```rust
#[tokio::test]
async fn test_review_agent_non_woo_can_auto_approve() {
    // Given: A non-Woo document with high scores
    let document = GeneratedDocument {
        metadata: DocumentMetadata {
            woo_relevant: false,
            // ... other fields
        },
        // ... other fields
    };
    let compliance = ComplianceResult {
        is_compliant: true,
        score: 0.98,
        // ... other fields
    };
    let domain_config = DomainConfig {
        trust_level: TrustLevel::High,
        auto_approval_threshold: 0.95,
        // ... other fields
    };
    
    // When: Executing review agent
    let result = execute_review_agent(&document, &compliance, &domain_config).await;
    
    // Then: Should allow auto-approval for high-scoring non-Woo documents
    assert!(result.is_ok());
    let decision = result.unwrap();
    assert!(decision.can_auto_approve);
}
```

**Test: execute_review_agent returns ReviewDecision with proper action**
```rust
#[tokio::test]
async fn test_review_agent_decision_types() {
    // Test all possible decision outcomes
    
    // 1. Approve: High compliance, no issues, non-Woo
    let approve_decision = create_test_decision(ReviewAction::Approve);
    assert_eq!(approve_decision.action, ReviewAction::Approve);
    
    // 2. RequireHumanApproval: Woo document or medium trust
    let human_decision = create_test_decision(ReviewAction::RequireHumanApproval);
    assert_eq!(human_decision.action, ReviewAction::RequireHumanApproval);
    
    // 3. RequestRevision: Quality issues found
    let revision_decision = create_test_decision(ReviewAction::RequestRevision);
    assert!(!revision_decision.feedback.is_empty());
    
    // 4. Reject: Critical compliance failures
    let reject_decision = create_test_decision(ReviewAction::Reject);
    assert_eq!(reject_decision.action, ReviewAction::Reject);
}
```

## Implementation

### 6.1 Domain Types

The Review Agent uses the following types, defined in `iou-core/src/document.rs` (from section-01-foundation):

```rust
/// Unique identifier for a document generation request
pub type DocumentId = uuid::Uuid;

/// Document state reuses WorkflowStatus from workflows module
pub use crate::workflows::WorkflowStatus as DocumentState;

/// Trust level determines auto-approval behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrustLevel {
    Low,     // Always requires human approval
    Medium,  // Requires approval if compliance_score < 0.8
    High,    // Auto-approval ONLY for non-Woo documents
}

/// Configuration per information domain
pub struct DomainConfig {
    pub domain_id: String,
    pub trust_level: TrustLevel,
    pub required_approval_threshold: f32,  // For Medium trust
    pub auto_approval_threshold: f32,      // For High trust
}
```

And compliance types from `iou-core/src/compliance.rs`:

```rust
// Already defined in iou-core - reuse these:
// - WooRefusalGround
// - WooDisclosureClass  
// - PrivacyLevel
// - ComplianceStatus
// - ComplianceIssue
```

### 6.2 Review Agent Types

Define these types in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/review.rs`:

```rust
use iou_core::{document::*, compliance::*};
use iou_core::workflows::WorkflowStatus;
use serde::{Deserialize, Serialize};

/// Quality issues found during review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub category: QualityIssueCategory,
    pub severity: IssueSeverity,  // Reuse from compliance.rs
    pub description: String,
    pub location: Option<String>,  // Section reference
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityIssueCategory {
    MissingRequiredSection,
    UnclearPhrasing,
    InconsistentTerminology,
    LowComplianceScore,
    PiiNotRedacted,
    AccessibilityIssue,
    StructuralProblem,
}

/// Review decision output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDecision {
    pub action: ReviewAction,
    pub can_auto_approve: bool,
    pub overall_quality_score: f32,  // 0.0 - 1.0
    pub completeness_issues: Vec<QualityIssue>,
    pub clarity_issues: Vec<QualityIssue>,
    pub feedback: String,  // Human-readable summary
    pub suggested_improvements: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewAction {
    Approve,              // Document approved (auto-approve)
    RequireHumanApproval, // Needs human review
    RequestRevision,      // Send back to Content Agent
    Reject,               // Document rejected
}

/// Review context passed to the agent
#[derive(Debug, Clone)]
pub struct ReviewContext {
    pub document: GeneratedDocument,
    pub compliance: ComplianceResult,
    pub research: ResearchContext,
    pub domain_config: DomainConfig,
}
```

### 6.3 Core Review Logic

Implement the main review agent function in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/review.rs`:

```rust
use chrono::{DateTime, Utc};

/// Executes the Review Agent
/// 
/// This agent performs the final quality check before human approval.
/// It coordinates with the Content Agent via the maker-checker pattern
/// when quality issues are found.
pub async fn execute_review_agent(
    document: &GeneratedDocument,
    compliance: &ComplianceResult,
    research: &ResearchContext,
    domain_config: &DomainConfig,
) -> Result<ReviewDecision, AgentError> {
    let start_time = std::time::Instant::now();
    
    // 1. Check completeness - all required sections present
    let completeness_issues = check_completeness(document, research).await?;
    
    // 2. Validate compliance score meets threshold
    let compliance_issues = check_compliance_threshold(compliance, domain_config);
    
    // 3. Check clarity and consistency
    let clarity_issues = check_clarity(document).await?;
    
    // 4. Aggregate all issues
    let all_issues: Vec<QualityIssue> = completeness_issues
        .into_iter()
        .chain(compliance_issues)
        .chain(clarity_issues)
        .collect();
    
    // 5. Calculate overall quality score
    let quality_score = calculate_quality_score(compliance, &all_issues);
    
    // 6. Determine review action
    let action = determine_review_action(
        document,
        compliance,
        domain_config,
        quality_score,
        &all_issues,
    );
    
    // 7. Generate feedback and suggestions
    let (feedback, suggestions) = generate_feedback(&action, &all_issues, document);
    
    // 8. Determine if auto-approval is possible
    let can_auto_approve = can_auto_approve(document, compliance, domain_config, quality_score);
    
    let execution_time = start_time.elapsed().as_millis() as u64;
    
    Ok(ReviewDecision {
        action,
        can_auto_approve,
        overall_quality_score: quality_score,
        completeness_issues: all_issues
            .iter()
            .filter(|i| matches!(i.category, QualityIssueCategory::MissingRequiredSection))
            .cloned()
            .collect(),
        clarity_issues: all_issues
            .iter()
            .filter(|i| matches!(i.category, QualityIssueCategory::UnclearPhrasing | QualityIssueCategory::InconsistentTerminology))
            .cloned()
            .collect(),
        feedback,
        suggested_improvements: suggestions,
    })
}

/// Check if all required sections are present
async fn check_completeness(
    document: &GeneratedDocument,
    research: &ResearchContext,
) -> Result<Vec<QualityIssue>, AgentError> {
    let mut issues = Vec::new();
    
    for required_section in &research.required_sections {
        let found = document.sections.iter()
            .any(|s| s.eq_ignore_ascii_case(required_section));
            
        if !found {
            issues.push(QualityIssue {
                category: QualityIssueCategory::MissingRequiredSection,
                severity: IssueSeverity::High,
                description: format!("Required section '{}' is missing", required_section),
                location: None,
                suggested_fix: Some(format!("Add section '{}'", required_section)),
            });
        }
    }
    
    Ok(issues)
}

/// Check if compliance score meets domain threshold
fn check_compliance_threshold(
    compliance: &ComplianceResult,
    config: &DomainConfig,
) -> Vec<QualityIssue> {
    let mut issues = Vec::new();
    
    let threshold = match config.trust_level {
        TrustLevel::Low => config.required_approval_threshold,
        TrustLevel::Medium => config.required_approval_threshold,
        TrustLevel::High => config.auto_approval_threshold,
    };
    
    if compliance.score < threshold {
        issues.push(QualityIssue {
            category: QualityIssueCategory::LowComplianceScore,
            severity: if compliance.score < threshold * 0.8 {
                IssueSeverity::Critical
            } else {
                IssueSeverity::High
            },
            description: format!(
                "Compliance score ({:.2}) below threshold ({:.2})",
                compliance.score, threshold
            ),
            location: None,
            suggested_fix: Some("Review compliance issues and address refusal grounds".to_string()),
        });
    }
    
    // Check for unredacted PII
    if !compliance.pii_detected.is_empty() {
        issues.push(QualityIssue {
            category: QualityIssueCategory::PiiNotRedacted,
            severity: IssueSeverity::Critical,
            description: format!("{} PII instances detected and not redacted", compliance.pii_detected.len()),
            location: None,
            suggested_fix: Some("Run compliance agent again to redact PII".to_string()),
        });
    }
    
    issues
}

/// Check document clarity and consistency
async fn check_clarity(document: &GeneratedDocument) -> Result<Vec<QualityIssue>, AgentError> {
    let mut issues = Vec::new();
    
    // Simple heuristic checks (can be enhanced with AI)
    let content = &document.content;
    
    // Check for placeholder text
    if content.contains("[TODO]") || content.contains("[PLACEHOLDER]") {
        issues.push(QualityIssue {
            category: QualityIssueCategory::StructuralProblem,
            severity: IssueSeverity::Medium,
            description: "Document contains placeholder text".to_string(),
            location: None,
            suggested_fix: Some("Replace all placeholders with actual content".to_string()),
        });
    }
    
    // Check for very short sections
    for section in &document.sections {
        // Extract section content and check length
        // This is a simplified check
    }
    
    Ok(issues)
}

/// Calculate overall quality score from compliance and issues
fn calculate_quality_score(
    compliance: &ComplianceResult,
    issues: &[QualityIssue],
) -> f32 {
    let base_score = compliance.score;
    
    // Penalize for quality issues
    let penalty: f32 = issues.iter()
        .map(|i| match i.severity {
            IssueSeverity::Critical => 0.2,
            IssueSeverity::High => 0.1,
            IssueSeverity::Medium => 0.05,
            IssueSeverity::Low => 0.01,
        })
        .sum();
    
    (base_score - penalty).max(0.0).min(1.0)
}

/// Determine the appropriate review action
fn determine_review_action(
    document: &GeneratedDocument,
    compliance: &ComplianceResult,
    config: &DomainConfig,
    quality_score: f32,
    issues: &[QualityIssue],
) -> ReviewAction {
    // Check for critical issues - always reject
    let has_critical = issues.iter().any(|i| matches!(i.severity, IssueSeverity::Critical));
    if has_critical {
        return ReviewAction::Reject;
    }
    
    // Check for unredacted PII - reject
    if !compliance.pii_detected.is_empty() {
        return ReviewAction::Reject;
    }
    
    // Check for high severity issues - request revision
    let has_high_issues = issues.iter().any(|i| matches!(i.severity, IssueSeverity::High));
    if has_high_issues {
        return ReviewAction::RequestRevision;
    }
    
    // Check if quality is acceptable
    let threshold = match config.trust_level {
        TrustLevel::Low => config.required_approval_threshold,
        TrustLevel::Medium => config.required_approval_threshold,
        TrustLevel::High => config.auto_approval_threshold,
    };
    
    if quality_score < threshold {
        return ReviewAction::RequestRevision;
    }
    
    // At this point, document passes quality checks
    // Determine if human approval is needed
    if requires_human_approval(document, config) {
        ReviewAction::RequireHumanApproval
    } else {
        ReviewAction::Approve
    }
}

/// Check if document requires human approval
fn requires_human_approval(
    document: &GeneratedDocument,
    config: &DomainConfig,
) -> bool {
    // CRITICAL: ALL Woo documents require human approval
    if document.metadata.woo_relevant {
        return true;
    }
    
    // Low trust domains always require human approval
    if config.trust_level == TrustLevel::Low {
        return true;
    }
    
    false
}

/// Check if auto-approval is allowed
fn can_auto_approve(
    document: &GeneratedDocument,
    compliance: &ComplianceResult,
    config: &DomainConfig,
    quality_score: f32,
) -> bool {
    // NEVER auto-approve Woo documents
    if document.metadata.woo_relevant {
        return false;
    }
    
    // Low trust never auto-approves
    if config.trust_level == TrustLevel::Low {
        return false;
    }
    
    // Medium trust requires human approval
    if config.trust_level == TrustLevel::Medium {
        return false;
    }
    
    // High trust: auto-approve only if quality meets threshold
    quality_score >= config.auto_approval_threshold
}

/// Generate human-readable feedback and suggestions
fn generate_feedback(
    action: &ReviewAction,
    issues: &[QualityIssue],
    document: &GeneratedDocument,
) -> (String, Vec<String>) {
    let mut suggestions = Vec::new();
    
    for issue in issues {
        if let Some(fix) = &issue.suggested_fix {
            suggestions.push(fix.clone());
        }
    }
    
    let feedback = match action {
        ReviewAction::Approve => {
            format!(
                "Document approved with quality score {:.2}. No issues found.",
                document.quality_score.unwrap_or(0.0)
            )
        }
        ReviewAction::RequireHumanApproval => {
            format!(
                "Document requires human approval. Quality score: {:.2}. {} issues found.",
                document.quality_score.unwrap_or(0.0),
                issues.len()
            )
        }
        ReviewAction::RequestRevision => {
            format!(
                "Document requires revision. Found {} issues. Top priority: {}",
                issues.len(),
                issues.first().map(|i| &i.description).unwrap_or(&"See details".to_string())
            )
        }
        ReviewAction::Reject => {
            format!(
                "Document rejected due to critical issues. {}",
                issues.iter()
                    .filter(|i| matches!(i.severity, IssueSeverity::Critical))
                    .map(|i| &i.description)
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        }
    };
    
    (feedback, suggestions)
}

/// Error type for review agent operations
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Missing required context: {0}")]
    MissingContext(String),
    
    #[error("Compliance check failed: {0}")]
    ComplianceCheckFailed(String),
    
    #[error("Document analysis failed: {0}")]
    AnalysisFailed(String),
    
    #[error("AI provider error: {0}")]
    ProviderError(String),
}
```

### 6.4 Module Structure

Add the review agent to `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/mod.rs`:

```rust
pub mod review;

pub use review::{
    execute_review_agent,
    ReviewDecision,
    ReviewAction,
    QualityIssue,
    QualityIssueCategory,
    ReviewContext,
    AgentError,
};
```

## Integration Points

### Reused Types from iou-core

From `/Users/marc/Projecten/iou-modern/crates/iou-core/src/compliance.rs`:
- `WooRefusalGround` - Enum of Woo refusal grounds
- `WooDisclosureClass` - Openbaarheidsklasse
- `PrivacyLevel` - AVG privacy levels
- `ComplianceStatus` - Overall compliance status
- `ComplianceIssue` - Specific compliance problems
- `IssueSeverity` - Low, Medium, High, Critical

From `/Users/marc/Projecten/iou-modern/crates/iou-core/src/workflows.rs`:
- `WorkflowStatus` - Document state (Draft, Submitted, InReview, Approved, Published, Rejected, Archived)

### Input Types from Other Agents

From section-03-research-agent (`ResearchContext`):
```rust
pub struct ResearchContext {
    pub required_sections: Vec<String>,
    pub optional_sections: Vec<String>,
    pub suggested_template: Option<String>,
    pub domain_entities: Vec<Entity>,
}
```

From section-04-content-agent (`GeneratedDocument`):
```rust
pub struct GeneratedDocument {
    pub id: DocumentId,
    pub content: String,
    pub format: DocumentFormat,
    pub sections: Vec<String>,
    pub metadata: DocumentMetadata,
    pub quality_score: Option<f32>,
}
```

From section-05-compliance-agent (`ComplianceResult`):
```rust
pub struct ComplianceResult {
    pub is_compliant: bool,
    pub score: f32,
    pub refusal_grounds: Vec<WooRefusalGround>,
    pub pii_detected: Vec<PiiLocation>,
    pub accessibility_issues: Vec<AccessibilityIssue>,
}
```

## Implementation Checklist

1. [ ] Create `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/review.rs`
2. [ ] Define `ReviewDecision`, `ReviewAction`, `QualityIssue` types
3. [ ] Implement `execute_review_agent` function
4. [ ] Implement `check_completeness` function
5. [ ] Implement `check_compliance_threshold` function
6. [ ] Implement `check_clarity` function
7. [ ] Implement `calculate_quality_score` function
8. [ ] Implement `determine_review_action` function
9. [ ] Implement `requires_human_approval` function (CRITICAL for Woo compliance)
10. [ ] Implement `can_auto_approve` function
11. [ ] Implement `generate_feedback` function
12. [ ] Add module to `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/mod.rs`
13. [ ] Write all tests in `#[cfg(test)]` module
14. [ ] Verify Woo documents ALWAYS require human approval
15. [ ] Run `cargo test` in `iou-ai` crate