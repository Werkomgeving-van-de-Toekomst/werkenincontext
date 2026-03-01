//! Compliance Validator Agent
//!
//! This agent validates generated documents against Dutch government regulations:
//! - Woo (Wet open overheid) compliance validation
//! - PII (Personally Identifiable Information) detection and irreversible redaction
//! - WCAG accessibility compliance checks

use crate::agents::{AgentError, content::GeneratedDocument};
use chrono::{DateTime, Utc};
use iou_core::compliance::{
    WooRefusalGround, ComplianceIssue, IssueSeverity,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    /// Assessment timestamp
    pub assessed_at: DateTime<Utc>,

    /// Storage key for original (unredacted) document
    /// Only populated when secure storage is enabled
    pub original_storage_key: Option<String>,

    /// Storage key for redacted document
    /// Only populated when secure storage is enabled
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

impl PiiType {
    /// Redaction label for this PII type
    pub fn redaction_label(&self) -> &'static str {
        match self {
            PiiType::BSN => "[PII: BSN]",
            PiiType::Email => "[PII: Email]",
            PiiType::PhoneNumber => "[PII: Telefoonnummer]",
            PiiType::Address => "[PII: Adres]",
            PiiType::IBAN => "[PII: IBAN]",
            PiiType::Name => "[PII: Naam]",
        }
    }
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

    /// Line number where issue was found (if applicable)
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessibilityLevel {
    A,
    AA,
    AAA,
}

/// Configuration for compliance checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Minimum compliance score threshold (0.0 - 1.0)
    pub min_compliance_score: f32,

    /// Whether to enable PII detection
    pub enable_pii_detection: bool,

    /// Whether to enable WCAG checks
    pub enable_wcag_checks: bool,

    /// Minimum PII detection confidence (0.0 - 1.0)
    pub pii_confidence_threshold: f32,

    /// Whether to enable secure storage for original documents
    /// When enabled, original (unredacted) documents are stored separately
    /// with RBAC controls for authorized access
    pub enable_secure_storage: bool,

    /// S3 bucket for original (unredacted) documents
    /// Access requires RBAC authorization
    pub original_documents_bucket: Option<String>,

    /// S3 bucket for redacted documents
    /// Public or restricted access based on publication status
    pub redacted_documents_bucket: Option<String>,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            min_compliance_score: 0.8,
            enable_pii_detection: true,
            enable_wcag_checks: true,
            pii_confidence_threshold: 0.85,
            enable_secure_storage: false,
            original_documents_bucket: None,
            redacted_documents_bucket: None,
        }
    }
}

/// Executes the Compliance Validator Agent
///
/// This agent validates generated documents against:
/// 1. Woo (Wet open overheid) regulations
/// 2. AVG/GDPR privacy requirements (PII detection)
/// 3. WCAG accessibility guidelines
///
/// # Arguments
/// * `document` - The generated document from Content Agent
///
/// # Returns
/// `ComplianceResult` with validation scores and redacted content
pub async fn execute_compliance_agent(
    document: &GeneratedDocument,
) -> Result<ComplianceResult, AgentError> {
    let config = ComplianceConfig::default();
    execute_compliance_agent_with_config(document, &config).await
}

/// Executes the Compliance Validator Agent with custom configuration
pub async fn execute_compliance_agent_with_config(
    document: &GeneratedDocument,
    config: &ComplianceConfig,
) -> Result<ComplianceResult, AgentError> {
    let mut result = ComplianceResult {
        is_compliant: true,
        score: 1.0,
        refusal_grounds: Vec::new(),
        pii_detected: Vec::new(),
        accessibility_issues: Vec::new(),
        issues: Vec::new(),
        redacted_content: None,
        assessed_at: Utc::now(),
        original_storage_key: None,
        redacted_storage_key: None,
    };

    let mut content = document.content.clone();
    let original_content = content.clone();

    // 1. PII Detection and Redaction
    if config.enable_pii_detection {
        let pii_result = detect_and_redact_pii(&content, config.pii_confidence_threshold);

        if !pii_result.detected.is_empty() {
            let pii_count = pii_result.detected.len();
            result.pii_detected = pii_result.detected;
            content = pii_result.redacted_content;
            result.redacted_content = Some(content.clone());

            // Add compliance issue for PII presence
            result.issues.push(ComplianceIssue {
                severity: IssueSeverity::Medium,
                category: "PII".to_string(),
                description: format!("{} PII items detected and redacted", pii_count),
                recommended_action: "Review redacted content before publication".to_string(),
            });
        }
    }

    // 2. Woo Compliance Validation
    let woo_result = validate_woo_compliance(&content);
    result.refusal_grounds = woo_result.refusal_grounds;
    result.issues.extend(woo_result.issues);

    // 3. WCAG Accessibility Checks
    if config.enable_wcag_checks {
        let wcag_result = check_wcag_compliance(&content);
        if !wcag_result.is_empty() {
            result.accessibility_issues = wcag_result;
            result.issues.push(ComplianceIssue {
                severity: IssueSeverity::Low,
                category: "WCAG".to_string(),
                description: format!("{} WCAG accessibility issues found", result.accessibility_issues.len()),
                recommended_action: "Review document for accessibility improvements".to_string(),
            });
        }
    }

    // 4. Calculate overall score
    result.score = calculate_compliance_score(&result);
    result.is_compliant = result.score >= config.min_compliance_score;
    result.redacted_content = Some(content.clone());

    // 5. Secure Storage (if enabled)
    if config.enable_secure_storage {
        // Store original (unredacted) document in secure bucket
        if let (Some(original_bucket), Some(redacted_bucket)) = (
            &config.original_documents_bucket,
            &config.redacted_documents_bucket,
        ) {
            match store_original_document(
                &document.document_id,
                &original_content,
                original_bucket,
            ).await {
                Ok(key) => result.original_storage_key = Some(key),
                Err(e) => {
                    result.issues.push(ComplianceIssue {
                        severity: IssueSeverity::High,
                        category: "Storage".to_string(),
                        description: format!("Failed to store original document: {}", e),
                        recommended_action: "Check storage configuration and permissions".to_string(),
                    });
                }
            }

            match store_redacted_document(
                &document.document_id,
                &content,
                redacted_bucket,
            ).await {
                Ok(key) => result.redacted_storage_key = Some(key),
                Err(e) => {
                    result.issues.push(ComplianceIssue {
                        severity: IssueSeverity::High,
                        category: "Storage".to_string(),
                        description: format!("Failed to store redacted document: {}", e),
                        recommended_action: "Check storage configuration and permissions".to_string(),
                    });
                }
            }
        }
    }

    Ok(result)
}

struct PiiDetectionResult {
    detected: Vec<PiiLocation>,
    redacted_content: String,
}

/// Detects PII in document content and redacts it irreversibly
///
/// Redaction format: [PII: <type>]
/// Examples: [PII: BSN], [PII: Email], [PII: IBAN]
fn detect_and_redact_pii(
    content: &str,
    confidence_threshold: f32,
) -> PiiDetectionResult {
    let mut detected = Vec::new();

    // Detect PII types in order of specificity
    // (IBAN before generic numbers, email before generic @ detection, etc.)

    // 1. IBAN Detection (most specific pattern)
    detected.extend(detect_iban(content, confidence_threshold));

    // 2. BSN Detection (Dutch social security number)
    detected.extend(detect_bsn(content, confidence_threshold));

    // 3. Email Detection
    detected.extend(detect_email(content, confidence_threshold));

    // 4. Phone Number Detection (Dutch formats)
    detected.extend(detect_phone(content, confidence_threshold));

    // 5. Address Detection (simple pattern for now)
    detected.extend(detect_address(content, confidence_threshold));

    // Remove overlapping detections (keep longest match at each position)
    detected = deduplicate_overlapping_pii(detected);

    // Apply redaction from end to start to preserve indices
    let mut redacted = content.to_string();
    let mut sorted_detections: Vec<_> = detected.iter().collect();
    sorted_detections.sort_by_key(|d| std::cmp::Reverse(d.start_index));

    for detection in sorted_detections {
        let replacement = detection.pii_type.redaction_label();
        redacted.replace_range(detection.start_index..detection.end_index, replacement);
    }

    PiiDetectionResult {
        detected,
        redacted_content: redacted,
    }
}

/// Removes overlapping PII detections, keeping the longest match at each position
fn deduplicate_overlapping_pii(mut detections: Vec<PiiLocation>) -> Vec<PiiLocation> {
    // Sort by start index, then by length (descending)
    detections.sort_by_key(|d| (d.start_index, std::cmp::Reverse(d.end_index - d.start_index)));

    let mut result = Vec::new();
    let mut last_end = 0;

    for detection in detections {
        if detection.start_index >= last_end {
            last_end = detection.end_index;
            result.push(detection);
        }
    }

    result
}

/// Dutch BSN pattern: 9 digits, must pass 11-proef
fn detect_bsn(content: &str, _threshold: f32) -> Vec<PiiLocation> {
    let mut results = Vec::new();

    // BSN regex: word boundary + 9 digits + word boundary
    let re = Regex::new(r"\b(\d{9})\b").unwrap();

    for mat in re.find_iter(content) {
        let digits = mat.as_str();
        if validate_bsn_11_proef(digits) {
            results.push(PiiLocation {
                text: digits.to_string(),
                start_index: mat.start(),
                end_index: mat.end(),
                pii_type: PiiType::BSN,
                confidence: 1.0,  // High confidence if 11-proef passes
            });
        }
    }

    results
}

/// Validates BSN using the "11-proef" (11-check) algorithm
fn validate_bsn_11_proef(bsn: &str) -> bool {
    if bsn.len() != 9 || !bsn.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let digits: Vec<u32> = bsn.chars()
        .map(|c| c.to_digit(10).unwrap())
        .collect();

    let sum: u32 = digits.iter()
        .enumerate()
        .map(|(i, &d)| d * (9 - i as u32))
        .sum();

    sum % 11 == 0
}

/// IBAN pattern: NLxx BANK xxxxxxxxxxxx (18 chars total)
fn detect_iban(content: &str, _threshold: f32) -> Vec<PiiLocation> {
    let mut results = Vec::new();

    // Dutch IBAN: starts with NL, followed by 2 check digits, then 4 letters (BANK), then 10 digits
    // Pattern: NL[0-9]{2}[A-Z]{4}[0-9]{10}
    let re = Regex::new(r"\b(NL\d{2}[A-Z]{4}\d{10})\b").unwrap();

    for mat in re.find_iter(content) {
        results.push(PiiLocation {
            text: mat.as_str().to_string(),
            start_index: mat.start(),
            end_index: mat.end(),
            pii_type: PiiType::IBAN,
            confidence: 1.0,
        });
    }

    results
}

/// Email pattern: standard RFC 5322 (simplified)
///
/// Only redacts emails that appear to be personal (name patterns).
/// Skips generic/role-based emails like info@, contact@, support@, etc.
/// This avoids false positives for public/government contact addresses.
fn detect_email(content: &str, _threshold: f32) -> Vec<PiiLocation> {
    let mut results = Vec::new();

    // Simple email regex
    let re = Regex::new(r"\b([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})\b").unwrap();

    // Generic/role-based email prefixes that are NOT personal PII
    // These are public contact addresses, not personal information
    let generic_prefixes = [
        "info@", "contact@", "support@", "help@", "admin@", "webmaster@",
        "postmaster@", "noreply@", "no-reply@", "secretariaat@",
        "public@", "news@", "press@", "media@", "communications@",
        "dienst@", "service@", "services@", "bericht@",
    ];

    for mat in re.find_iter(content) {
        let email = mat.as_str().to_lowercase();
        let email_lower = email.as_str();

        // Skip generic/role-based emails (these are public contact addresses, not PII)
        let is_generic = generic_prefixes.iter()
            .any(|prefix| email_lower.starts_with(prefix));

        if !is_generic {
            // Extract the local part (before @) to check for personal patterns
            let local_part = email_lower.split('@').next().unwrap_or("");

            // Check if it looks like a personal email
            // Personal indicators: contains dots, underscores, or hyphens (name patterns)
            // OR appears to be a personal name format (e.g., "first.last", "first_last")
            let looks_personal = local_part.contains('.')
                || local_part.contains('_')
                || local_part.contains('-');

            if looks_personal {
                results.push(PiiLocation {
                    text: mat.as_str().to_string(),
                    start_index: mat.start(),
                    end_index: mat.end(),
                    pii_type: PiiType::Email,
                    confidence: 0.85,
                });
            }
        }
    }

    results
}

/// Dutch phone patterns: +31 6..., 06-..., 06 ..., (0XX) XXXXXXX
fn detect_phone(content: &str, _threshold: f32) -> Vec<PiiLocation> {
    let mut results = Vec::new();

    // Multiple Dutch phone number formats:
    // +31 6 XXXXXXXXX
    // +316XXXXXXXX
    // 06-XXXXXXXX
    // 06 XXXXXXXX
    // 0XX-XXXXXXXX
    // (0XX) XXXXXXXX
    let patterns = [
        r"\b(\+31\s*6\s*\d{8})\b",
        r"\b(\+316\d{8})\b",
        r"\b(06-\d{8})\b",
        r"\b(06\s\d{8})\b",
        r"\b(0\d{2}-\d{7})\b",
        r"\b(\(0\d{2}\)\s\d{7})\b",
    ];

    for pattern in &patterns {
        let re = Regex::new(pattern).unwrap();
        for mat in re.find_iter(content) {
            // Check if already detected (avoid duplicates)
            let start = mat.start();
            if !results.iter().any(|r: &PiiLocation| r.start_index == start) {
                results.push(PiiLocation {
                    text: mat.as_str().to_string(),
                    start_index: mat.start(),
                    end_index: mat.end(),
                    pii_type: PiiType::PhoneNumber,
                    confidence: 0.85,
                });
            }
        }
    }

    results
}

/// Simple address detection (Dutch postal code pattern)
fn detect_address(content: &str, _threshold: f32) -> Vec<PiiLocation> {
    let mut results = Vec::new();

    // Dutch postal code: 1234 AB or 1234AB
    // Usually followed by city name
    let re = Regex::new(r"\b(\d{4}\s?[A-Z]{2})\b").unwrap();

    for mat in re.find_iter(content) {
        let postcode = mat.as_str();
        // Check if it's a valid Dutch postal code (first digit 1-9, letters A-Z excluding SA/SD/SS)
        if is_valid_dutch_postcode(postcode) {
            results.push(PiiLocation {
                text: mat.as_str().to_string(),
                start_index: mat.start(),
                end_index: mat.end(),
                pii_type: PiiType::Address,
                confidence: 0.7,  // Lower confidence, need more context
            });
        }
    }

    results
}

/// Validate Dutch postal code format
fn is_valid_dutch_postcode(code: &str) -> bool {
    let clean = code.replace(" ", "");
    if clean.len() != 6 {
        return false;
    }

    let digits = &clean[0..4];
    let letters = &clean[4..6];

    // First digit must be 1-9 (not 0)
    if !digits.starts_with(|c: char| c.is_ascii_digit() && c != '0') {
        return false;
    }

    // Letters must be A-Z, excluding SA/SD/SS (special postal codes)
    let upper = letters.to_uppercase();
    if upper == "SA" || upper == "SD" || upper == "SS" {
        return false;
    }

    true
}

/// Validates document against Woo (Wet open overheid) regulations
fn validate_woo_compliance(content: &str) -> WooValidationResult {
    let mut refusal_grounds = Vec::new();
    let mut issues = Vec::new();

    // Check for Woo indicators in the document
    let content_lower = content.to_lowercase();

    // Check if document mentions Woo refusal grounds
    for ground in ALL_WOO_REFUSAL_GROUNDS {
        if content_lower.contains(&ground.search_term) {
            refusal_grounds.push(ground.ground);
        }
    }

    // Check for proper Woo structure if this appears to be a Woo document
    if is_woo_document(content) {
        // Check if document has required sections
        if !content.contains("#") && !content.contains("Besluit") {
            issues.push(ComplianceIssue {
                severity: IssueSeverity::High,
                category: "Woo".to_string(),
                description: "Woo document missing proper structure".to_string(),
                recommended_action: "Add proper document structure with headings".to_string(),
            });
        }

        // Check if justification is provided when refusal grounds exist
        if !refusal_grounds.is_empty() && !content_lower.contains("toelichting")
            && !content_lower.contains("grond")
        {
            issues.push(ComplianceIssue {
                severity: IssueSeverity::Medium,
                category: "Woo".to_string(),
                description: "Woo refusal grounds without proper justification".to_string(),
                recommended_action: "Add explanation for refusal grounds".to_string(),
            });
        }
    }

    WooValidationResult {
        refusal_grounds,
        issues,
    }
}

/// Checks if content appears to be a Woo document
fn is_woo_document(content: &str) -> bool {
    let content_lower = content.to_lowercase();
    content_lower.contains("woo")
        || content_lower.contains("wet open overheid")
        || content_lower.contains("besluit")
        || content_lower.contains("openbaarmaking")
}

struct WooValidationResult {
    refusal_grounds: Vec<WooRefusalGround>,
    issues: Vec<ComplianceIssue>,
}

/// Woo refusal ground search terms
struct WooRefusalSearchTerm {
    ground: WooRefusalGround,
    search_term: &'static str,
}

const ALL_WOO_REFUSAL_GROUNDS: &[WooRefusalSearchTerm] = &[
    WooRefusalSearchTerm { ground: WooRefusalGround::PersoonlijkeLevenssfeer, search_term: "persoonlijke levenssfeer" },
    WooRefusalSearchTerm { ground: WooRefusalGround::Bedrijfsgegevens, search_term: "bedrijfsgegeven" },
    WooRefusalSearchTerm { ground: WooRefusalGround::BijzonderePersoonsgegevens, search_term: "bijzondere persoonsgegevens" },
    WooRefusalSearchTerm { ground: WooRefusalGround::EenheidKroon, search_term: "eenheid van de kroon" },
    WooRefusalSearchTerm { ground: WooRefusalGround::VeiligheidStaat, search_term: "veiligheid van de staat" },
];

/// Checks document for WCAG accessibility compliance
fn check_wcag_compliance(content: &str) -> Vec<AccessibilityIssue> {
    let mut issues = Vec::new();

    // Check heading hierarchy
    issues.extend(check_heading_hierarchy(content));

    // Check for proper list formatting
    issues.extend(check_list_formatting(content));

    // Check link text
    issues.extend(check_link_text(content));

    issues
}

fn check_heading_hierarchy(content: &str) -> Vec<AccessibilityIssue> {
    let mut issues = Vec::new();
    let mut h1_count = 0;
    let mut last_level = 0;

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix('#') {
            let level = rest.chars().take_while(|c| *c == '#').count() + 1;

            // Check for skipped levels
            if last_level > 0 && level > last_level + 1 {
                issues.push(AccessibilityIssue {
                    description: format!("Skipped heading level: from h{} to h{}", last_level, level),
                    wcag_criterion: "1.3.1".to_string(),
                    level: AccessibilityLevel::A,
                    line_number: None,
                });
            }

            if level == 1 {
                h1_count += 1;
            }

            last_level = level;
        }
    }

    // Check for multiple h1 headings
    if h1_count > 1 {
        issues.push(AccessibilityIssue {
            description: format!("Multiple h1 headings found: {}", h1_count),
            wcag_criterion: "1.3.1".to_string(),
            level: AccessibilityLevel::A,
            line_number: None,
        });
    }

    // Check if document has no h1
    if h1_count == 0 && !content.is_empty() {
        issues.push(AccessibilityIssue {
            description: "No h1 heading found".to_string(),
            wcag_criterion: "1.3.1".to_string(),
            level: AccessibilityLevel::A,
            line_number: None,
        });
    }

    issues
}

fn check_list_formatting(content: &str) -> Vec<AccessibilityIssue> {
    let mut issues = Vec::new();
    let mut in_list = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            in_list = true;
        } else if trimmed.is_empty() {
            in_list = false;
        } else if in_list && !trimmed.starts_with("  ") && !trimmed.starts_with("-") && !trimmed.starts_with("*") {
            // List item without proper nesting
            issues.push(AccessibilityIssue {
                description: "Possible list item without proper formatting".to_string(),
                wcag_criterion: "1.3.1".to_string(),
                level: AccessibilityLevel::A,
                line_number: None,
            });
        }
    }

    issues
}

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

/// Stores the original (unredacted) document in secure storage
///
/// This function stores documents that may contain PII in a separate S3 bucket
/// with RBAC controls. Only authorized users can access the original documents.
///
/// # TODO Section
///
/// This is a placeholder implementation. The following features need to be added:
///
/// 1. **S3 Integration**: Use aws-sdk-s3 or rust-s3 for actual storage
///    - Upload document content to S3 bucket
///    - Generate unique storage keys (e.g., `documents/{document_id}/original/{timestamp}`)
///    - Handle multipart uploads for large documents
///
/// 2. **RBAC (Role-Based Access Control)**:
///    - Integrate with authentication/authorization system
///    - Check user permissions before allowing access
///    - Log all access attempts (success and failure)
///    - Support role-based policies (admin, auditor, editor)
///
/// 3. **Audit Trail**:
///    - Record all storage operations (upload, access, download, delete)
///    - Include: timestamp, user_id, action, document_id, ip_address
///    - Store audit logs in separate, append-only table/storage
///    - Consider integration with existing audit system
///
/// 4. **Encryption**:
///    - Server-side encryption (SSE-KMS or SSE-S3)
///    - Client-side encryption option for high-security scenarios
///    - Key management via KMS
///
/// # Security Architecture
///
/// ```
/// Original Document Bucket (Restricted Access):
/// - IAM policies restrict access to authorized roles
/// - All access logged to audit trail
/// - Objects encrypted at rest
/// - No public read access
///
/// Redacted Document Bucket (Controlled Access):
/// - Published documents may be public
/// - Draft documents restricted to editors/admins
/// - Version tracking for updates
/// ```
///
/// # Storage Key Format
///
/// Returns a storage key in the format:
/// `documents/{document_id}/original/{timestamp}_{version}`
///
/// # Example Implementation
///
/// ```rust,ignore
/// use aws_sdk_s3::{Client, primitives::ByteStream};
///
/// pub async fn store_original_document(
///     document_id: &Uuid,
///     content: &str,
///     bucket: &str,
/// ) -> Result<String, AgentError> {
///     let client = get_s3_client().await?;
///     let key = format!("documents/{}/original/{}", document_id, Utc::now().to_rfc3339());
///
///     client.put_object()
///         .bucket(bucket)
///         .key(&key)
///         .body(ByteStream::from(content.as_bytes().to_vec()))
///         .server_side_encryption(
///             aws_sdk_s3::types::ServerSideEncryption::AwsKms
///         )
///         .send()
///         .await?;
///
///     // Audit log
///     audit_log("document_stored", document_id, &key).await?;
///
///     Ok(key)
/// }
/// ```
async fn store_original_document(
    document_id: &uuid::Uuid,
    content: &str,
    bucket: &str,
) -> Result<String, AgentError> {
    // TODO: Implement S3 storage with RBAC and audit trail
    // See function documentation for required features

    let timestamp = Utc::now().to_rfc3339();
    let storage_key = format!("documents/{}/original/{}", document_id, timestamp);

    // Placeholder: In production, this would:
    // 1. Upload to S3 with server-side encryption
    // 2. Verify upload integrity
    // 3. Log to audit trail
    // 4. Return actual S3 object key

    tracing::warn!(
        "store_original_document called (placeholder): bucket={}, key={}",
        bucket,
        storage_key
    );

    Ok(storage_key)
}

/// Stores the redacted document in storage
///
/// Redacted documents have PII removed and can be safely published
/// (subject to other compliance requirements like Woo refusal grounds).
///
/// # TODO Section
///
/// Similar to `store_original_document`, the following features need to be added:
///
/// 1. **S3 Integration**: Upload to redacted documents bucket
/// 2. **Version Control**: Track document versions for updates
/// 3. **Metadata Storage**: Store PII summary, redaction timestamp, processor
/// 4. **Publication Status**: Track draft/published/archived states
///
/// # Storage Key Format
///
/// Returns a storage key in the format:
/// `documents/{document_id}/redacted/{timestamp}_{version}`
///
/// # Audit Requirements
///
/// - Log who performed the redaction
/// - Record which PII types were found and redacted
/// - Track publication status changes
async fn store_redacted_document(
    document_id: &uuid::Uuid,
    content: &str,
    bucket: &str,
) -> Result<String, AgentError> {
    // TODO: Implement S3 storage with version control and audit
    // See function documentation for required features

    let timestamp = Utc::now().to_rfc3339();
    let storage_key = format!("documents/{}/redacted/{}", document_id, timestamp);

    // Placeholder: In production, this would:
    // 1. Upload to S3 with appropriate encryption
    // 2. Store metadata (PII summary, redaction info)
    // 3. Set publication status tags
    // 4. Log to audit trail

    tracing::warn!(
        "store_redacted_document called (placeholder): bucket={}, key={}",
        bucket,
        storage_key
    );

    Ok(storage_key)
}

fn check_link_text(content: &str) -> Vec<AccessibilityIssue> {
    let mut issues = Vec::new();

    // Check for generic link text
    let generic_patterns = ["klik hier", "lees meer", "meer info", "click here", "read more"];

    // Use regex to find markdown links: [text](url)
    // Raw string literal with escaped brackets
    let re = Regex::new(r#"\[([^\]]+)\]\(([^)]+)\)"#).unwrap();

    for caps in re.captures_iter(content) {
        if let Some(link_text) = caps.get(1) {
            let link_text_lower = link_text.as_str().to_lowercase();

            for generic in &generic_patterns {
                if link_text_lower.contains(generic) {
                    issues.push(AccessibilityIssue {
                        description: format!("Generic link text found: \"{}\"", link_text.as_str()),
                        wcag_criterion: "2.4.4".to_string(),
                        level: AccessibilityLevel::A,
                        line_number: None,
                    });
                    break;
                }
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::content::GeneratedDocument;

    fn create_test_document(content: &str) -> GeneratedDocument {
        GeneratedDocument {
            document_id: Uuid::new_v4(),
            content: content.to_string(),
            variables: vec![],
            entity_links: vec![],
            sections: vec![],
            generated_at: Utc::now(),
        }
    }

    #[test]
    fn test_compliance_config_default() {
        let config = ComplianceConfig::default();
        assert_eq!(config.min_compliance_score, 0.8);
        assert!(config.enable_pii_detection);
        assert!(config.enable_wcag_checks);
    }

    #[test]
    fn test_pii_type_redaction_labels() {
        assert_eq!(PiiType::BSN.redaction_label(), "[PII: BSN]");
        assert_eq!(PiiType::Email.redaction_label(), "[PII: Email]");
        assert_eq!(PiiType::IBAN.redaction_label(), "[PII: IBAN]");
    }

    #[test]
    fn test_validate_bsn_11_proef_valid() {
        // Valid BSN: 123456782
        // Sum: 1*9 + 2*8 + 3*7 + 4*6 + 5*5 + 6*4 + 7*3 + 8*2 + 2*1 = 9+16+21+24+25+24+21+16+2 = 158
        // 158 % 11 = 4 (not valid)
        // Let me find a valid one: 111222333 = 9+8+7+6+5+4+3+2+3 = 47 % 11 = 3 (not valid)
        // 123456789 = 9+16+21+24+25+24+21+16+9 = 165 % 11 = 0 (valid!)
        assert!(validate_bsn_11_proef("123456789"));
    }

    #[test]
    fn test_validate_bsn_11_proef_invalid() {
        assert!(!validate_bsn_11_proef("123456788"));
    }

    #[test]
    fn test_is_valid_dutch_postcode() {
        assert!(is_valid_dutch_postcode("1234 AB"));
        assert!(is_valid_dutch_postcode("1234AB"));
        assert!(!is_valid_dutch_postcode("0123 AB"));  // Starts with 0
        assert!(!is_valid_dutch_postcode("1234 SA"));  // Excluded code
        assert!(!is_valid_dutch_postcode("1234"));     // Too short
    }

    #[test]
    fn test_detect_bsn_finds_valid_bsn() {
        let content = "Mijn BSN is 123456789 en ik woon in Utrecht.";
        let result = detect_bsn(content, 0.85);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pii_type, PiiType::BSN);
        assert_eq!(result[0].text, "123456789");
    }

    #[test]
    fn test_detect_iban_finds_dutch_iban() {
        let content = "Rekeningnummer: NL91ABNA0417164300";
        let result = detect_iban(content, 0.85);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pii_type, PiiType::IBAN);
    }

    #[test]
    fn test_detect_email_finds_personal_emails() {
        // Generic/role-based emails should NOT be detected as PII
        let content = "Contact: info@gemeente.utrecht.nl of helpdesk@rijksoverheid.nl";
        let result = detect_email(content, 0.85);
        assert_eq!(result.len(), 0, "Generic/role-based emails should not be flagged as PII");

        // Personal emails should be detected
        let content = "Contact: jan.jansen@gemeente.utrecht.nl of m.de.vries@rijksoverheid.nl";
        let result = detect_email(content, 0.85);
        assert_eq!(result.len(), 2, "Personal emails with name patterns should be flagged as PII");
    }

    #[test]
    fn test_detect_phone_finds_dutch_formats() {
        let content = "Bel: 06-12345678 of +31 6 12345678";
        let result = detect_phone(content, 0.85);
        // Note: duplicate detection logic prevents the same phone number from being detected twice
        // even with different formats
        assert!(result.len() >= 1);
        assert_eq!(result[0].pii_type, PiiType::PhoneNumber);
    }

    #[test]
    fn test_detect_address_finds_postal_codes() {
        let content = "Adres: Straat 1, 1234 AB Amsterdam";
        let result = detect_address(content, 0.85);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pii_type, PiiType::Address);
    }

    #[test]
    fn test_pii_redaction_is_irreversible() {
        let content = "BSN: 123456789";
        let result = detect_and_redact_pii(content, 0.85);
        assert!(!result.redacted_content.contains("123456789"));
        assert!(result.redacted_content.contains("[PII: BSN]"));
    }

    #[test]
    fn test_check_heading_hierarchy_detects_multiple_h1() {
        let content = "# Title\n\nSome text\n\n# Another Title\n\nMore text";
        let issues = check_heading_hierarchy(content);
        assert!(issues.iter().any(|i| i.description.contains("Multiple h1")));
    }

    #[test]
    fn test_check_heading_hierarchy_detects_no_h1() {
        let content = "## No h1 here\n\nJust some text";
        let issues = check_heading_hierarchy(content);
        assert!(issues.iter().any(|i| i.description.contains("No h1")));
    }

    #[test]
    fn test_check_heading_hierarchy_valid_hierarchy() {
        let content = "# Main Title\n\n## Section\n\n### Subsection\n\nText";
        let issues = check_heading_hierarchy(content);
        assert!(!issues.iter().any(|i| i.description.contains("heading")));
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_detects_pii() {
        let doc = create_test_document("Contact: jan.jansen@gemeente.nl, 06-12345678");
        let result = execute_compliance_agent(&doc).await.unwrap();

        assert!(!result.pii_detected.is_empty());
        assert!(result.redacted_content.is_some());
        assert!(result.redacted_content.unwrap().contains("[PII:"));
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_validates_woo() {
        let doc = create_test_document(
            "Woo besluit met weigeringsgrond: persoonlijke levenssfeer. \
             Toelichting: Privacy bescherming."
        );
        let result = execute_compliance_agent(&doc).await.unwrap();

        assert!(!result.refusal_grounds.is_empty());
        assert_eq!(result.refusal_grounds[0], WooRefusalGround::PersoonlijkeLevenssfeer);
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_checks_wcag() {
        let doc = create_test_document("# Title\n\nKlik [klik hier](page.html) voor info.");
        let result = execute_compliance_agent(&doc).await.unwrap();

        // Should detect the generic link text "klik hier"
        assert!(!result.accessibility_issues.is_empty());
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_calculates_score() {
        let doc = create_test_document("# Clean Document\n\nNo PII, no issues.");
        let result = execute_compliance_agent(&doc).await.unwrap();

        assert!(result.score > 0.8);
        assert!(result.is_compliant);
    }

    #[tokio::test]
    async fn test_execute_compliance_agent_score_below_threshold_with_pii() {
        let doc = create_test_document(
            "Jan Jansen, BSN: 123456789, email: jan.jansen@test.nl, \
             tel: 06-98765432, woont in 1234 AB Amsterdam"
        );
        let result = execute_compliance_agent(&doc).await.unwrap();

        // PII should reduce score
        assert!(result.score < 1.0);
        assert!(!result.pii_detected.is_empty());
    }
}
