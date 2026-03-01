Now I have all the context. Let me generate the section content for `section-05-compliance-agent`. This section focuses on the Compliance Validator Agent which handles Woo validation and PII detection.

Based on the plan:

1. The Compliance Agent is in Batch 2 (parallel after foundation)
2. It depends on section-01-foundation
3. It blocks section-07-pipeline-orchestration
4. Key responsibilities: Woo validation, PII detection with irreversible redaction, WCAG compliance checks

Let me extract the relevant tests from the TDD plan and implementation details from the main plan.</think># Section 5: Compliance Validator Agent

## Overview

This section implements the **Compliance Validator Agent**, the third agent in the document creation pipeline. The Compliance Agent validates generated documents against Dutch government regulations, specifically:

- **Woo (Wet open overheid)** compliance validation
- **PII (Personally Identifiable Information)** detection and irreversible redaction
- **WCAG** accessibility compliance checks

The agent reuses existing compliance types from `iou-core/src/compliance.rs` to maintain consistency with the existing IOU-Modern codebase.

## Dependencies

This section depends on:

- **section-01-foundation**: Core domain types, storage configuration, database schema

## Implementation Priority

1. PII detection with irreversible redaction
2. Woo compliance validation (reusing existing types)
3. WCAG accessibility checks
4. Compliance scoring and result aggregation

---

## Tests

### 4.3 Compliance Validator Agent Tests

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/compliance.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::test_utils::*;

    #[tokio::test]
    async fn test_execute_compliance_agent_detects_pii_with_known_patterns() {
        // Test that the agent detects various PII types:
        // - Dutch BSN format (9 digits, valid 11-proef)
        // - Email addresses
        // - Dutch phone numbers
        // - IBAN format
        // - Physical addresses
        // - Personal names
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_redacts_pii_irreversibly() {
        // Test that PII is redacted using [PII: <type>] format
        // Verify redaction is irreversible (original cannot be recovered)
        // Verify format matches: [PII: BSN], [PII: Email], etc.
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_validates_woo_refusal_grounds() {
        // Test that Woo documents are validated against refusal grounds
        // Reuse WooRefusalGround from iou-core/src/compliance.rs
        // Verify invalid refusal grounds are detected
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_checks_wcag_compliance_requirements() {
        // Test that WCAG AA requirements are checked
        // Verify heading structure is correct
        // Verify alternative text for images
        // Verify sufficient color contrast
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_returns_accurate_compliance_result() {
        // Test that ComplianceResult.score is calculated correctly
        // Verify scores range from 0.0 to 1.0
        // Verify weighted scoring for different compliance aspects
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_separates_original_and_redacted_storage() {
        // Test that original document is stored separately from redacted version
        // Verify different S3 keys for original and redacted
        // Verify access control differs for each version
    }
}
```

### A.1 Existing Compliance AI Tests (Type Reuse Verification)

```rust
#[cfg(test)]
mod type_reuse_tests {
    use super::*;
    use iou_core::compliance::{WooMetadata, WooRefusalGround, WooDisclosureClass};

    #[test]
    fn test_reuse_woo_metadata_from_existing_compliance_module() {
        // Verify WooMetadata type can be used
    }

    #[test]
    fn test_reuse_woo_refusal_ground_from_existing_compliance_module() {
        // Verify WooRefusalGround enum can be used
        // Verify all refusal ground variants are accessible
    }

    #[test]
    fn test_reuse_woo_disclosure_class_from_existing_compliance_module() {
        // Verify WooDisclosureClass type can be used
    }

    #[test]
    fn test_reuse_avg_metadata_from_existing_compliance_module() {
        // Verify AvgMetadata (GDPR-related types) can be used
    }

    #[test]
    fn test_reuse_privacy_level_from_existing_compliance_module() {
        // Verify PrivacyLevel enum can be used
    }

    #[test]
    fn test_reuse_compliance_status_from_existing_compliance_module() {
        // Verify ComplianceStatus enum can be used
    }

    #[test]
    fn test_reuse_compliance_issue_from_existing_compliance_module() {
        // Verify ComplianceIssue struct can be used
    }
}
```

### 9.3 Compliance Tests

```rust
#[cfg(test)]
mod compliance_validation_tests {
    use super::*;

    #[tokio::test]
    async fn test_known_woo_violating_documents_fail_compliance() {
        // Test with documents known to violate Woo
        // Verify they fail compliance checks
        // Verify specific refusal grounds are identified
    }

    #[tokio::test]
    async fn test_known_woo_compliant_documents_pass_compliance() {
        // Test with documents known to be Woo-compliant
        // Verify they pass compliance checks
    }

    #[tokio::test]
    async fn test_pii_detection_for_dutch_bsn_format() {
        // BSN: 9-digit Dutch social security number
        // Must pass the "11-proef" (11-check) validation
        // Format: XXXXXXXXX (digits only)
    }

    #[tokio::test]
    async fn test_pii_detection_for_email_addresses() {
        // Standard email format detection
    }

    #[tokio::test]
    async fn test_pii_detection_for_dutch_phone_numbers() {
        // Formats: +31 6 XXXXXXXXX, 06-XXXXXXXX, etc.
    }

    #[tokio::test]
    async fn test_pii_detection_for_iban() {
        // Dutch IBAN: NLxx BANK xxxxxxxxxxxx
        // 18 characters, starts with NL
    }

    #[tokio::test]
    async fn test_refusal_ground_accuracy_for_woo_articles_5_1_and_5_2() {
        // Test specific Woo articles 5.1 and 5.2 refusal grounds
    }

    #[tokio::test]
    async fn test_wcag_aa_compliance_checks_pass() {
        // Verify WCAG AA level requirements are checked
        // Include heading hierarchy, link text, alt text
    }
}
```

### 11.1 PII Redaction Tests (Security)

```rust
#[cfg(test)]
mod pii_redaction_security_tests {
    use super::*;

    #[test]
    fn test_redacted_pii_cannot_be_reversed_to_original_value() {
        // Verify that redaction is truly irreversible
        // No encoding or reversible transformation
    }

    #[test]
    fn test_redacted_format_follows_pii_type_pattern() {
        // Verify format: [PII: <type>]
        // Types: BSN, Email, PhoneNumber, Address, IBAN, Name
    }

    #[test]
    fn test_original_and_redacted_documents_stored_separately() {
        // Verify different storage locations
        // Verify separate access controls
    }

    #[test]
    fn test_only_authorized_roles_can_access_unredacted_originals() {
        // Verify RBAC for unredacted document access
        // Verify audit trail records access
    }

    #[test]
    fn test_audit_trail_records_access_to_unredacted_documents() {
        // Verify all access is logged
        // Include user ID, timestamp, document ID
    }
}
```

---

## Implementation

### File Structure

```
iou-modern/
└── crates/
    └── iou-ai/
        └── src/
            └── agents/
                └── compliance.rs    # NEW: Compliance Validator Agent
```

### Core Types

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/compliance.rs`

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Reuse existing compliance types from iou-core
use iou_core::compliance::{
    WooMetadata, WooRefusalGround, WooDisclosureClass,
    AvgMetadata, PrivacyLevel, ComplianceStatus, ComplianceIssue
};

/// Result of compliance validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    /// Overall compliance status
    pub is_compliant: bool,
    
    /// Compliance score (0.0 - 1.0)
    pub score: f32,
    
    /// Woo-specific refusal grounds detected
    pub refusal_grounds: Vec<WooRefusalGround>,
    
    /// PII locations detected in document
    pub pii_detected: Vec<PiiLocation>,
    
    /// Accessibility/WCAG issues found
    pub accessibility_issues: Vec<AccessibilityIssue>,
    
    /// Detailed compliance issues
    pub issues: Vec<ComplianceIssue>,
    
    /// Redacted document content (if PII found)
    pub redacted_content: Option<String>,
    
    /// Storage reference for original (unredacted) document
    pub original_storage_key: Option<String>,
    
    /// Storage reference for redacted document
    pub redacted_storage_key: Option<String>,
}

/// Location of detected PII in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiLocation {
    /// The text that was detected as PII
    pub text: String,
    
    /// Start index in document
    pub start_index: usize,
    
    /// End index in document
    pub end_index: usize,
    
    /// Type of PII detected
    pub pii_type: PiiType,
    
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f32,
}

/// PII types supported for detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PiiType {
    /// Dutch social security number (9 digits, 11-proef valid)
    BSN,
    /// Email address
    Email,
    /// Phone number (Dutch format)
    PhoneNumber,
    /// Physical address
    Address,
    /// IBAN bank account number
    IBAN,
    /// Personal name
    Name,
}

/// WCAG accessibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityIssue {
    /// Issue description
    pub description: String,
    
    /// WCAG success criterion (e.g., "1.1.1", "2.4.6")
    pub wcag_criterion: String,
    
    /// Issue level
    pub level: AccessibilityLevel,
    
    /// Location in document (if applicable)
    pub location: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessibilityLevel {
    A,
    AA,
    AAA,
}

/// Configuration for compliance checks
#[derive(Debug, Clone)]
pub struct ComplianceConfig {
    /// Minimum compliance score threshold (0.0 - 1.0)
    pub min_compliance_score: f32,
    
    /// Whether to enable PII detection
    pub enable_pii_detection: bool,
    
    /// Whether to enable WCAG checks
    pub enable_wcag_checks: bool,
    
    /// Minimum PII detection confidence (0.0 - 1.0)
    pub pii_confidence_threshold: f32,
    
    /// S3 bucket for storing original (unredacted) documents
    pub original_bucket: String,
    
    /// S3 bucket for storing redacted documents
    pub redacted_bucket: String,
}
```

### Main Agent Function

```rust
/// Executes the Compliance Validator Agent
///
/// This agent validates generated documents against:
/// 1. Woo (Wet open overheid) regulations
/// 2. AVG/GDPR privacy requirements (PII detection)
/// 3. WCAG accessibility guidelines
///
/// # Arguments
/// * `document` - The generated document from Content Agent
/// * `config` - Compliance configuration
///
/// # Returns
/// * `ComplianceResult` with validation scores and redacted content
pub async fn execute_compliance_agent(
    document: &GeneratedDocument,
    config: &ComplianceConfig,
) -> Result<ComplianceResult> {
    let mut result = ComplianceResult {
        is_compliant: true,
        score: 1.0,
        refusal_grounds: Vec::new(),
        pii_detected: Vec::new(),
        accessibility_issues: Vec::new(),
        issues: Vec::new(),
        redacted_content: None,
        original_storage_key: None,
        redacted_storage_key: None,
    };

    // 1. Store original document (for authorized access)
    let original_key = store_original_document(document, config).await?;
    result.original_storage_key = Some(original_key);

    // 2. PII Detection and Redaction
    if config.enable_pii_detection {
        let pii_result = detect_and_redact_pii(&document.content, config).await?;
        result.pii_detected = pii_result.detected;
        result.redacted_content = Some(pii_result.redacted_content);
        
        // Store redacted version
        if !pii_result.detected.is_empty() {
            let redacted_key = store_redacted_document(
                document,
                &pii_result.redacted_content,
                config,
            ).await?;
            result.redacted_storage_key = Some(redacted_key);
        }
    }

    // 3. Woo Compliance Validation
    let woo_result = validate_woo_compliance(document).await?;
    result.refusal_grounds = woo_result.refusal_grounds;
    result.issues.extend(woo_result.issues);

    // 4. WCAG Accessibility Checks
    if config.enable_wcag_checks {
        let wcag_result = check_wcag_compliance(document).await?;
        result.accessibility_issues = wcag_result.issues;
        result.issues.extend(wcag_result.compliance_issues);
    }

    // 5. Calculate overall score
    result.score = calculate_compliance_score(&result);
    result.is_compliant = result.score >= config.min_compliance_score;

    Ok(result)
}

struct PiiDetectionResult {
    detected: Vec<PiiLocation>,
    redacted_content: String,
}
```

### PII Detection Implementation

```rust
/// Detects PII in document content and redacts it irreversibly
///
/// Redaction format: [PII: <type>]
/// Examples: [PII: BSN], [PII: Email], [PII: IBAN]
async fn detect_and_redact_pii(
    content: &str,
    config: &ComplianceConfig,
) -> Result<PiiDetectionResult> {
    let mut detected = Vec::new();
    let mut redacted = content.to_string();

    // Detect PII types in order of specificity
    // (IBAN before generic numbers, email before generic @ detection, etc.)

    // 1. BSN Detection (Dutch social security number)
    detected.extend(detect_bsn(content, config.pii_confidence_threshold));

    // 2. IBAN Detection
    detected.extend(detect_iban(content, config.pii_confidence_threshold));

    // 3. Email Detection
    detected.extend(detect_email(content, config.pii_confidence_threshold));

    // 4. Phone Number Detection (Dutch formats)
    detected.extend(detect_phone(content, config.pii_confidence_threshold));

    // 5. Address Detection
    // (uses entity recognition or pattern matching)

    // 6. Name Detection
    // (uses NER with Dutch name patterns)

    // Apply redaction from end to start to preserve indices
    let mut sorted_detections: Vec<_> = detected.iter().collect();
    sorted_detections.sort_by_key(|d| std::cmp::Reverse(d.start_index));

    for detection in sorted_detections {
        let replacement = format!("[PII: {:?}]", detection.pii_type);
        redacted.replace_range(detection.start_index..detection.end_index, &replacement);
    }

    Ok(PiiDetectionResult {
        detected,
        redacted_content: redacted,
    })
}

/// Dutch BSN pattern: 9 digits, must pass 11-proef
fn detect_bsn(content: &str, threshold: f32) -> Vec<PiiLocation> {
    // BSN regex: \b\d{9}\b
    // Validate with 11-proef: sum of (digit * position) modulo 11 == 0
    // Return PiiLocation::BSN with confidence
}

/// IBAN pattern: NLxx BANK xxxxxxxxxxxx (18 chars total)
fn detect_iban(content: &str, threshold: f32) -> Vec<PiiLocation> {
    // Dutch IBAN starts with NL, 18 characters total
    // Basic format validation
}

/// Email pattern: standard RFC 5322
fn detect_email(content: &str, threshold: f32) -> Vec<PiiLocation> {
    // Standard email regex
}

/// Dutch phone patterns: +31 6..., 06-..., 06 ...
fn detect_phone(content: &str, threshold: f32) -> Vec<PiiLocation> {
    // Multiple Dutch phone number formats
}
```

### Woo Compliance Validation

```rust
/// Validates document against Woo (Wet open overheid) regulations
async fn validate_woo_compliance(
    document: &GeneratedDocument,
) -> Result<WooValidationResult> {
    let mut refusal_grounds = Vec::new();
    let mut issues = Vec::new();

    // Reuse existing WooRefusalGround from iou-core
    // Check for common Woo violations:
    // - Missing required sections for Woo documents
    // - Invalid refusal grounds cited
    // - Inadequate justification for disclosure restrictions

    // Use existing compliance AI from iou-ai/src/compliance.rs
    // to assess Woo relevance and refusal grounds

    Ok(WooValidationResult {
        refusal_grounds,
        issues,
    })
}

struct WooValidationResult {
    refusal_grounds: Vec<WooRefusalGround>,
    issues: Vec<ComplianceIssue>,
}
```

### WCAG Compliance Checking

```rust
/// Checks document for WCAG accessibility compliance
async fn check_wcag_compliance(
    document: &GeneratedDocument,
) -> Result<WcagResult> {
    let mut issues = Vec::new();
    let mut compliance_issues = Vec::new();

    // Check heading hierarchy (h1, h2, h3, ...)
    // - No skipped levels
    // - Exactly one h1
    issues.extend(check_heading_hierarchy(&document.content));

    // Check for alternative text on images
    // - All images must have alt text
    issues.extend(check_image_alt_text(&document.content));

    // Check link text
    // - No "click here" or generic link text
    issues.extend(check_link_text(&document.content));

    // Check list formatting
    issues.extend(check_list_formatting(&document.content));

    Ok(WcagResult {
        issues,
        compliance_issues,
    })
}

fn check_heading_hierarchy(content: &str) -> Vec<AccessibilityIssue> {
    // Parse Markdown heading levels
    // Verify: h1 -> h2 -> h3 (no skipping)
    // Verify: single h1 at document start
}

fn check_image_alt_text(content: &str) -> Vec<AccessibilityIssue> {
    // Find ![]() markdown images
    // Verify alt text is not empty
    // Verify alt text is descriptive
}

fn check_link_text(content: &str) -> Vec<AccessibilityIssue> {
    // Find [text](url) markdown links
    // Flag generic phrases: "click here", "lees meer", etc.
}

fn check_list_formatting(content: &str) -> Vec<AccessibilityIssue> {
    // Verify proper list syntax
    // Check for nested lists
}

struct WcagResult {
    issues: Vec<AccessibilityIssue>,
    compliance_issues: Vec<ComplianceIssue>,
}
```

### Scoring Calculation

```rust
/// Calculates overall compliance score from individual components
fn calculate_compliance_score(result: &ComplianceResult) -> f32 {
    let mut score = 1.0f32;

    // Deduct for PII found (not compliant, but document can be published redacted)
    if !result.pii_detected.is_empty() {
        score *= 0.95; // Slight penalty for PII presence
    }

    // Deduct for Woo refusal grounds
    if !result.refusal_grounds.is_empty() {
        score *= 0.7; // Significant penalty for Woo issues
    }

    // Deduct for accessibility issues
    let wcag_penalty = result.accessibility_issues.len() as f32 * 0.05;
    score = (score - wcag_penalty).max(0.0);

    // Deduct for general compliance issues
    let issue_penalty = result.issues.len() as f32 * 0.03;
    score = (score - issue_penalty).max(0.0);

    score
}
```

### Storage Functions

```rust
/// Stores original (unredacted) document securely
async fn store_original_document(
    document: &GeneratedDocument,
    config: &ComplianceConfig,
) -> Result<String> {
    // Store in S3 bucket with restricted access
    // Key: documents/original/{document_id}/{timestamp}.md
    // Only accessible to authorized roles
}

/// Stores redacted document for public access
async fn store_redacted_document(
    document: &GeneratedDocument,
    redacted_content: &str,
    config: &ComplianceConfig,
) -> Result<String> {
    // Store in S3 bucket with public/read access
    // Key: documents/redacted/{document_id}/{timestamp}.md
    // This version can be downloaded by users
}
```

---

## Integration with Existing Code

### Reused Types from iou-core/src/compliance.rs

The Compliance Agent must reuse the following existing types to maintain consistency:

```rust
// From iou-core/src/compliance.rs
pub use iou_core::compliance::{
    // Woo-specific types
    WooMetadata,           // Metadata about Woo relevance
    WooRefusalGround,      // Enum of refusal ground categories
    WooDisclosureClass,    // Classification of disclosure level
    
    // AVG/GDPR types
    AvgMetadata,           // GDPR-related metadata
    PrivacyLevel,          // Privacy classification
    
    // General compliance types
    ComplianceStatus,      // Overall compliance status
    ComplianceIssue,       // Individual compliance issue
};
```

**Location:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/compliance.rs`

Verify these types exist and match the expected structure before implementing the agent.

---

## Configuration

### Environment Variables

```bash
# PII Detection
COMPLIANCE_PII_DETECTION_ENABLED=true
COMPLIANCE_PII_CONFIDENCE_THRESHOLD=0.85

# WCAG Checks
COMPLIANCE_WCAG_CHECKS_ENABLED=true
COMPLIANCE_MIN_WCAG_LEVEL=AA

# Storage
COMPLIANCE_ORIGINAL_BUCKET=iou-documents-original
COMPLIANCE_REDACTED_BUCKET=iou-documents-public

# Scoring
COMPLIANCE_MIN_SCORE=0.8
```

### Default Configuration

```rust
impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            min_compliance_score: 0.8,
            enable_pii_detection: true,
            enable_wcag_checks: true,
            pii_confidence_threshold: 0.85,
            original_bucket: "iou-documents-original".to_string(),
            redacted_bucket: "iou-documents-public".to_string(),
        }
    }
}
```

---

## Success Criteria

1. **Functional:**
   - PII detection accuracy > 95% for Dutch PII types
   - Redaction is irreversible and follows `[PII: <type>]` format
   - Woo validation reuses existing compliance types correctly
   - WCAG AA checks catch common accessibility issues

2. **Security:**
   - Original and redacted documents stored separately
   - Only authorized roles can access unredacted originals
   - All access to unredacted documents is logged

3. **Integration:**
   - Existing compliance types from `iou-core` are reused
   - No duplication of compliance logic

---

## Next Steps

After implementing this section:

1. **section-06-review-agent** can be implemented in parallel
2. Both agents feed into **section-07-pipeline-orchestration**

The Compliance Agent produces a `ComplianceResult` that is consumed by:
- The Review Agent (for quality checks)
- The Pipeline (for audit trail logging)
- The API (for status reporting)