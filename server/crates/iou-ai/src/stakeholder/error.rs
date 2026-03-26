//! Error types for stakeholder extraction

use thiserror::Error;

/// Main error type for stakeholder extraction
#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("Document text is empty or inaccessible")]
    EmptyDocument,

    #[error("Baseline extraction failed: {0}")]
    BaselineFailed(String),

    #[error("LLM extraction failed: {0}")]
    LlmFailed(String),

    #[error("Normalization failed: {0}")]
    NormalizationFailed(#[from] NormalizationError),

    #[error("Deduplication failed: {0}")]
    DeduplicationFailed(#[from] DeduplicationError),

    #[error("Cost limit exceeded: {actual:.4} > {limit:.4} USD")]
    CostLimitExceeded { actual: f32, limit: f32 },

    #[error("API rate limit exceeded")]
    RateLimitExceeded,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Error during entity normalization
#[derive(Debug, Error)]
pub enum NormalizationError {
    #[error("API request failed: {0}")]
    ApiRequestFailed(String),

    #[error("API timeout after {0}s")]
    ApiTimeout(u64),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("No canonical name found for: {0}")]
    CanonicalNameNotFound(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Error during entity deduplication
#[derive(Debug, Error)]
pub enum DeduplicationError {
    #[error("Similarity calculation failed: {0}")]
    SimilarityError(String),

    #[error("Clustering failed: {0}")]
    ClusteringError(String),

    #[error("Merge conflict: {0}")]
    MergeConflict(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_error_display() {
        let err = ExtractionError::EmptyDocument;
        assert_eq!(err.to_string(), "Document text is empty or inaccessible");

        let err = ExtractionError::BaselineFailed("test failure".to_string());
        assert_eq!(err.to_string(), "Baseline extraction failed: test failure");
    }

    #[test]
    fn test_normalization_error_display() {
        let err = NormalizationError::CanonicalNameNotFound("Unknown Org".to_string());
        assert_eq!(err.to_string(), "No canonical name found for: Unknown Org");
    }

    #[test]
    fn test_deduplication_error_display() {
        let err = DeduplicationError::MergeConflict("conflicting entities".to_string());
        assert_eq!(err.to_string(), "Merge conflict: conflicting entities");
    }

    #[test]
    fn test_cost_limit_error_formatting() {
        let err = ExtractionError::CostLimitExceeded { actual: 0.15, limit: 0.10 };
        assert_eq!(err.to_string(), "Cost limit exceeded: 0.1500 > 0.1000 USD");
    }

    #[test]
    fn test_error_conversion() {
        let norm_err = NormalizationError::ApiRequestFailed("network error".to_string());
        let ext_err: ExtractionError = norm_err.into();
        assert!(ext_err.to_string().contains("Normalization failed"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let ext_err: ExtractionError = io_err.into();
        assert!(ext_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_serialization_error_conversion() {
        let ser_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let ext_err: ExtractionError = ser_err.into();
        assert!(ext_err.to_string().contains("Serialization error"));
    }
}
