//! Purpose Validation
//!
//! Types and logic for validating purpose access requests.

use crate::purpose::{Purpose, PurposeId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Purpose validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurposeValidation {
    /// The purpose ID that was validated
    pub purpose_id: PurposeId,

    /// Whether the purpose is valid
    pub is_valid: bool,

    /// Reason for rejection (if invalid)
    pub rejected_reason: Option<String>,

    /// When the validation occurred
    pub validated_at: DateTime<Utc>,
}

impl PurposeValidation {
    /// Create a successful validation result
    pub fn success(purpose_id: PurposeId) -> Self {
        Self {
            purpose_id,
            is_valid: true,
            rejected_reason: None,
            validated_at: Utc::now(),
        }
    }

    /// Create a failed validation result
    pub fn failure(purpose_id: PurposeId, reason: String) -> Self {
        Self {
            purpose_id,
            is_valid: false,
            rejected_reason: Some(reason),
            validated_at: Utc::now(),
        }
    }
}

/// Validation context for a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationContext {
    /// Request ID for tracing
    pub request_id: Option<String>,

    /// User ID making the request
    pub user_id: Option<String>,

    /// Organization ID
    pub organization_id: Option<String>,

    /// IP address of the requester
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new() -> Self {
        Self {
            request_id: None,
            user_id: None,
            organization_id: None,
            ip_address: None,
            user_agent: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a request ID
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Add a user ID
    pub fn with_user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Extended validation result with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// The basic validation
    pub validation: PurposeValidation,

    /// The context of the validation
    pub context: ValidationContext,

    /// The validated purpose (if successful)
    pub purpose: Option<Purpose>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success(
        purpose_id: PurposeId,
        purpose: Purpose,
        context: ValidationContext,
    ) -> Self {
        Self {
            validation: PurposeValidation::success(purpose_id),
            context,
            purpose: Some(purpose),
        }
    }

    /// Create a failed validation result
    pub fn failure(
        purpose_id: PurposeId,
        reason: String,
        context: ValidationContext,
    ) -> Self {
        Self {
            validation: PurposeValidation::failure(purpose_id, reason),
            context,
            purpose: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_context_new() {
        let context = ValidationContext::new();

        assert!(context.request_id.is_none());
        assert!(context.user_id.is_none());
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn test_validation_context_builder() {
        let context = ValidationContext::new()
            .with_request_id("req-123")
            .with_user_id("user-456")
            .with_metadata("key", "value");

        assert_eq!(context.request_id, Some("req-123".to_string()));
        assert_eq!(context.user_id, Some("user-456".to_string()));
        assert_eq!(context.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_validation_result_success() {
        let purpose = Purpose::new(
            "P001",
            "TEST",
            "Test",
            crate::purpose::LawfulBasis::WettelijkeVerplichting,
            "Owner",
        );
        let context = ValidationContext::new();

        let result = ValidationResult::success("P001".to_string(), purpose, context);

        assert!(result.validation.is_valid);
        assert!(result.purpose.is_some());
        assert!(result.validation.rejected_reason.is_none());
    }

    #[test]
    fn test_validation_result_failure() {
        let context = ValidationContext::new();
        let result = ValidationResult::failure("P999".to_string(), "Not found".to_string(), context);

        assert!(!result.validation.is_valid);
        assert!(result.purpose.is_none());
        assert_eq!(result.validation.rejected_reason, Some("Not found".to_string()));
    }

    #[test]
    fn test_validation_success() {
        let validation = PurposeValidation::success("P001".to_string());

        assert!(validation.is_valid);
        assert_eq!(validation.purpose_id, "P001");
        assert!(validation.rejected_reason.is_none());
    }

    #[test]
    fn test_validation_failure() {
        let validation = PurposeValidation::failure("P001".to_string(), "Expired".to_string());

        assert!(!validation.is_valid);
        assert_eq!(validation.rejected_reason, Some("Expired".to_string()));
    }
}
