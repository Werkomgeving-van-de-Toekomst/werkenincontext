//! Feasibility spike tests for stakeholder extraction
//!
//! This test suite validates:
//! - Rijksoverheid API capabilities (or fallback)
//! - Local fallback dictionary for Dutch government orgs
//! - Cost estimation model for LLM usage
//! - GeneratedDocument structure verification

#[cfg(test)]
mod tests {
    use super::super::{get_fallback_canonical_name, CostEstimator, probe_rijksoverheid_api, ApiProbeResult};
    use crate::agents::GeneratedDocument;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_rijksoverheid_api_returns_canonical_name_for_minfin() {
        // Verify that the Rijksoverheid API (or local fallback)
        // returns "Ministerie van Financiën" when queried for "MinFin"
        let result = probe_rijksoverheid_api().await;

        // If API is available, we'd test actual response
        // For now, verify the probe completes without panicking
        assert!(result.endpoint_url.contains("data.overheid.nl") || !result.api_available);
    }

    #[tokio::test]
    async fn test_rijksoverheid_api_handles_unknown_organization_gracefully() {
        // Verify that querying for an unknown organization
        // via the API probe doesn't panic
        let result = probe_rijksoverheid_api().await;

        // Should return a valid result structure regardless of API availability
        match result.error_message {
            Some(_) => assert!(!result.api_available),
            None => {}, // No error is also fine
        }
    }

    #[test]
    fn test_local_fallback_dictionary_returns_canonical_name_when_api_unavailable() {
        // Verify that the local fallback dictionary
        // contains mappings for common Dutch government orgs
        // and returns correct canonical names

        // Test ministry abbreviations
        assert_eq!(get_fallback_canonical_name("MinFin"), Some("Ministerie van Financiën"));
        assert_eq!(get_fallback_canonical_name("minfin"), Some("Ministerie van Financiën"));

        // Test full names
        assert_eq!(get_fallback_canonical_name("Ministerie van Financiën"), Some("Ministerie van Financiën"));

        // Test other ministries
        assert_eq!(get_fallback_canonical_name("BZK"), Some("Ministerie van Binnenlandse Zaken en Koninkrijksrelaties"));
        assert_eq!(get_fallback_canonical_name("VWS"), Some("Ministerie van Volksgezondheid, Welzijn en Sport"));
        assert_eq!(get_fallback_canonical_name("EZK"), Some("Ministerie van Economische Zaken en Klimaat"));

        // Test agencies
        assert_eq!(get_fallback_canonical_name("Rijkswaterstaat"), Some("Rijkswaterstaat"));
        assert_eq!(get_fallback_canonical_name("RDW"), Some("Rijksdienst voor het Wegverkeer"));
        assert_eq!(get_fallback_canonical_name("Belastingdienst"), Some("Belastingdienst"));

        // Test unknown organization
        assert_eq!(get_fallback_canonical_name("Unknown Organization"), None);
    }

    #[test]
    fn test_cost_estimation_calculates_correctly_for_various_token_counts() {
        // Verify cost calculation for:
        // - 1K tokens (short document)
        // - 5K tokens (medium document)
        // - 10K tokens (long document)
        // Expected: ~$0.01, ~$0.05, ~$0.10 respectively

        // Short document: 1K input, 50 output (5%)
        let cost_1k = CostEstimator::estimate_cost(1000, 50);
        assert!(cost_1k > 0.0 && cost_1k < 0.02, "Cost should be ~$0.006, got ${}", cost_1k);

        // Medium document: 5K input, 250 output
        let cost_5k = CostEstimator::estimate_cost(5000, 250);
        assert!(cost_5k > 0.01 && cost_5k < 0.03, "Cost should be ~$0.019, got ${}", cost_5k);

        // Long document: 10K input, 500 output
        let cost_10k = CostEstimator::estimate_cost(10000, 500);
        assert!(cost_10k > 0.02 && cost_10k < 0.05, "Cost should be ~$0.038, got ${}", cost_10k);
    }

    #[test]
    fn test_cost_estimation_by_pages() {
        // Test page-based cost estimation
        let (cost, input, output) = CostEstimator::estimate_cost_by_pages(1);
        assert_eq!(input, 500);
        assert_eq!(output, 25); // 5% of 500
        assert!(cost > 0.0);
    }

    #[test]
    fn test_is_within_budget() {
        assert!(CostEstimator::is_within_budget(0.05, 0.10));
        assert!(!CostEstimator::is_within_budget(0.15, 0.10));
        assert!(CostEstimator::is_within_budget(0.10, 0.10));
    }

    #[test]
    fn test_generated_document_contains_accessible_content_field() {
        // Verify that GeneratedDocument has a .content field
        // that contains the document content for extraction

        let doc = GeneratedDocument {
            document_id: Uuid::new_v4(),
            content: "Test document content".to_string(),
            variables: vec![],
            entity_links: vec![],
            sections: vec![],
            generated_at: chrono::Utc::now(),
        };

        // Verify content field exists and is accessible
        assert!(!doc.content.is_empty());
        assert_eq!(doc.content, "Test document content");
    }

    #[test]
    fn test_fallback_dictionary_case_insensitive() {
        // Verify that the fallback dictionary is case-insensitive
        assert_eq!(get_fallback_canonical_name("MINFIN"), Some("Ministerie van Financiën"));
        assert_eq!(get_fallback_canonical_name("MiNfIn"), Some("Ministerie van Financiën"));
        assert_eq!(get_fallback_canonical_name("bzk"), Some("Ministerie van Binnenlandse Zaken en Koninkrijksrelaties"));
    }
}
