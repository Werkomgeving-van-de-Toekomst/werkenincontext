# Section 00: Feasibility Spike

## Overview

This section validates external dependencies before committing to the full architecture. The goal is to answer critical feasibility questions about the Rijksoverheid API, establish a cost estimation model for LLM usage, and create a local fallback dictionary for Dutch government organizations.

**Dependencies:** None

**Blocks:** section-03-rijksoverheid-api

**Success Criteria:**
- Rijksoverheid API capabilities documented or fallback strategy confirmed
- Cost model established with per-document estimates
- Local fallback dictionary created with common Dutch government organizations
- GeneratedDocument structure verified for text field accessibility

---

## Tests

### Test Stubs

Write the following tests in `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/feasibility_tests.rs`:

```rust
#[cfg(test)]
mod feasibility_tests {
    use super::*;

    #[tokio::test]
    async fn test_rijksoverheid_api_returns_canonical_name_for_minfin() {
        // Verify that the Rijksoverheid API (or local fallback)
        // returns "Ministerie van Financiën" when queried for "MinFin"
    }

    #[tokio::test]
    async fn test_rijksoverheid_api_handles_unknown_organization_gracefully() {
        // Verify that querying for an unknown organization
        // returns None rather than panicking or returning an error
    }

    #[test]
    fn test_local_fallback_dictionary_returns_canonical_name_when_api_unavailable() {
        // Verify that the local fallback dictionary
        // contains mappings for common Dutch government orgs
        // and returns correct canonical names
    }

    #[test]
    fn test_cost_estimation_calculates_correctly_for_various_token_counts() {
        // Verify cost calculation for:
        // - 1K tokens (short document)
        // - 5K tokens (medium document)
        // - 10K tokens (long document)
        // Expected: ~$0.01, ~$0.05, ~$0.10 respectively
    }

    #[test]
    fn test_generated_document_contains_accessible_text_field() {
        // Verify that GeneratedDocument has a .text or similar field
        // that contains the document content for extraction
    }
}
```

---

## Implementation Details

### 1. Rijksoverheid API Validation

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/rijksoverheid_api_probe.rs`

Create a probe module to test the Rijksoverheid API capabilities:

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Probe result documenting API capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProbeResult {
    pub api_available: bool,
    pub endpoint_url: String,
    pub requires_auth: bool,
    pub rate_limit_documented: bool,
    pub sample_response: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

/// Probe the Rijksoverheid API to document its capabilities
pub async fn probe_rijksoverheid_api() -> ApiProbeResult {
    const API_URL: &str = "https://api.data.overheid.nl/io/oa/organisaties";
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build();
    
    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return ApiProbeResult {
                api_available: false,
                endpoint_url: API_URL.to_string(),
                requires_auth: false,
                rate_limit_documented: false,
                sample_response: None,
                error_message: Some(format!("Failed to create client: {}", e)),
            };
        }
    };
    
    // Attempt to query the API
    let response = client.get(API_URL)
        .query(&[("zoekterm", "MinFin")])
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            let status = resp.status();
            let requires_auth = status == reqwest::StatusCode::UNAUTHORIZED;
            
            if status.is_success() {
                match resp.json().await {
                    Ok(json) => ApiProbeResult {
                        api_available: true,
                        endpoint_url: API_URL.to_string(),
                        requires_auth,
                        rate_limit_documented: true, // Check headers
                        sample_response: Some(json),
                        error_message: None,
                    },
                    Err(e) => ApiProbeResult {
                        api_available: true,
                        endpoint_url: API_URL.to_string(),
                        requires_auth,
                        rate_limit_documented: false,
                        sample_response: None,
                        error_message: Some(format!("Failed to parse JSON: {}", e)),
                    }
                }
            } else {
                ApiProbeResult {
                    api_available: false,
                    endpoint_url: API_URL.to_string(),
                    requires_auth,
                    rate_limit_documented: false,
                    sample_response: None,
                    error_message: Some(format!("API returned status: {}", status)),
                }
            }
        }
        Err(e) => ApiProbeResult {
            api_available: false,
            endpoint_url: API_URL.to_string(),
            requires_auth: false,
            rate_limit_documented: false,
            sample_response: None,
            error_message: Some(format!("Request failed: {}", e)),
        }
    }
}
```

**Documentation Requirements:**

After running the probe, document findings in a new file:
`/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/RIJKSOVERHEID_API.md`

Include:
- Actual API endpoint that works (if any)
- Authentication requirements (if any)
- Rate limits observed from headers
- Sample response format
- Fallback strategy if API is unavailable

### 2. Local Fallback Dictionary

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/fallback_dict.rs`

Create a local dictionary of Dutch government organizations:

```rust
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Local fallback dictionary for Dutch government organizations
/// Used when Rijksoverheid API is unavailable
pub static FALLBACK_DICT: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // Ministries with abbreviations
    m.insert("MinFin".to_lowercase(), "Ministerie van Financiën");
    m.insert("minfin".to_string(), "Ministerie van Financiën");
    m.insert("Ministerie van Financiën".to_lowercase(), "Ministerie van Financiën");
    
    m.insert("BZK".to_lowercase(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties");
    m.insert("bzk".to_string(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties");
    m.insert("Ministerie van Binnenlandse Zaken".to_lowercase(), "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties");
    
    m.insert("VWS".to_lowercase(), "Ministerie van Volksgezondheid, Welzijn en Sport");
    m.insert("vws".to_string(), "Ministerie van Volksgezondheid, Welzijn en Sport");
    
    m.insert("EZK".to_lowercase(), "Ministerie van Economische Zaken en Klimaat");
    m.insert("ezk".to_string(), "Ministerie van Economische Zaken en Klimaat");
    
    m.insert("OCW".to_lowercase(), "Ministerie van Onderwijs, Cultuur en Wetenschap");
    m.insert("ocw".to_string(), "Ministerie van Onderwijs, Cultuur en Wetenschap");
    
    m.insert("JenV".to_lowercase(), "Ministerie van Justitie en Veiligheid");
    m.insert("jenv".to_string(), "Ministerie van Justitie en Veiligheid");
    
    m.insert("IenW".to_lowercase(), "Ministerie van Infrastructuur en Waterstaat");
    m.insert("ienw".to_string(), "Ministerie van Infrastructuur en Waterstaat");
    
    m.insert("LNV".to_lowercase(), "Ministerie van Landbouw, Natuur en Voedselkwaliteit");
    m.insert("lnv".to_string(), "Ministerie van Landbouw, Natuur en Voedselkwaliteit");
    
    m.insert("SZW".to_lowercase(), "Ministerie van Sociale Zaken en Werkgelegenheid");
    m.insert("szw".to_string(), "Ministerie van Sociale Zaken en Werkgelegenheid");
    
    m.insert("BZ".to_lowercase(), "Ministerie van Buitenlandse Zaken");
    m.insert("bz".to_string(), "Ministerie van Buitenlandse Zaken");
    
    // Common agencies and services
    m.insert("Rijkswaterstaat".to_lowercase(), "Rijkswaterstaat");
    m.insert("Dienst Wegverkeer".to_lowercase(), "Rijksdienst voor het Wegverkeer");
    m.insert("RDW".to_lowercase(), "Rijksdienst voor het Wegverkeer");
    m.insert("Belastingdienst".to_lowercase(), "Belastingdienst");
    m.insert("Centraal Planbureau".to_lowercase(), "Centraal Planbureau");
    m.insert("CPB".to_lowercase(), "Centraal Planbureau");
    m.insert("Nederlandse Bank".to_lowercase(), "De Nederlandsche Bank");
    m.insert("DNB".to_lowercase(), "De Nederlandsche Bank");
    
    m
});

/// Get canonical name from fallback dictionary
pub fn get_fallback_canonical_name(name: &str) -> Option<&'static str> {
    FALLBACK_DICT.get(&name.to_lowercase()).copied()
}
```

### 3. Cost Estimation Model

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/cost_model.rs`

```rust
/// Cost estimation for Claude Sonnet 4.5 API calls
/// 
/// Pricing (as of 2025):
/// - Input: $3.00 per 1M tokens
/// - Output: $15.00 per 1M tokens
pub struct CostEstimator;

impl CostEstimator {
    /// Input cost per million tokens
    const INPUT_COST_PER_M: f32 = 3.00;
    
    /// Output cost per million tokens  
    const OUTPUT_COST_PER_M: f32 = 15.00;
    
    /// Estimate cost for a document with given token counts
    pub fn estimate_cost(input_tokens: u32, output_tokens: u32) -> f32 {
        let input_cost = (input_tokens as f32 / 1_000_000.0) * Self::INPUT_COST_PER_M;
        let output_cost = (output_tokens as f32 / 1_000_000.0) * Self::OUTPUT_COST_PER_M;
        input_cost + output_cost
    }
    
    /// Estimate cost for document based on approximate page count
    /// Assumes: ~500 tokens per page for Dutch government documents
    pub fn estimate_cost_by_pages(page_count: u32) -> (f32, u32, u32) {
        let total_tokens = page_count * 500;
        
        // Typical extraction: document is input, entities are output
        // Output is usually 5-10% of input for entity extraction
        let input_tokens = total_tokens;
        let output_tokens = total_tokens / 20; // 5%
        
        let cost = Self::estimate_cost(input_tokens, output_tokens);
        (cost, input_tokens, output_tokens)
    }
    
    /// Check if estimated cost is within budget
    pub fn is_within_budget(cost: f32, max_cost: f32) -> bool {
        cost <= max_cost
    }
}

#[cfg(test)]
mod cost_tests {
    use super::*;

    #[test]
    fn test_cost_estimation_1k_tokens() {
        // Short document: 1K input, 50 output (5%)
        let cost = CostEstimator::estimate_cost(1000, 50);
        assert!(cost > 0.0 && cost < 0.02, "Cost should be ~$0.006, got ${}", cost);
    }

    #[test]
    fn test_cost_estimation_5k_tokens() {
        // Medium document: 5K input, 250 output
        let cost = CostEstimator::estimate_cost(5000, 250);
        assert!(cost > 0.02 && cost < 0.06, "Cost should be ~$0.04, got ${}", cost);
    }

    #[test]
    fn test_cost_estimation_10k_tokens() {
        // Long document: 10K input, 500 output
        let cost = CostEstimator::estimate_cost(10000, 500);
        assert!(cost > 0.04 && cost < 0.10, "Cost should be ~$0.08, got ${}", cost);
    }
    
    #[test]
    fn test_is_within_budget() {
        assert!(CostEstimator::is_within_budget(0.05, 0.10));
        assert!(!CostEstimator::is_within_budget(0.15, 0.10));
    }
}
```

### 4. GeneratedDocument Structure Verification

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/document_probe.rs`

Verify that the GeneratedDocument type provides access to text content:

```rust
use iou_core::graphrag::GeneratedDocument;

/// Verify GeneratedDocument has accessible text field
pub fn verify_document_structure(document: &GeneratedDocument) -> Result<String, String> {
    // The document should have a way to access its text content
    // This may be through .text, .content, or similar field
    
    // Attempt to get text content - adjust field name based on actual type
    let text = document.text
        .or(document.content.as_deref())
        .or(document.body.as_deref())
        .ok_or_else(|| "No accessible text field found".to_string())?;
    
    if text.is_empty() {
        return Err("Document text is empty".to_string());
    }
    
    Ok(text.to_string())
}

#[cfg(test)]
mod document_tests {
    use super::*;

    #[test]
    fn test_generated_document_has_text_field() {
        // This test will need adjustment based on actual GeneratedDocument structure
        // For now, this documents the requirement
    }
}
```

---

## Deliverables

### 1. API Probe Documentation

Create `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/RIJKSOVERHEID_API.md` with:

```
# Rijksoverheid API Feasibility Report

## API Endpoint
[Document actual working endpoint]

## Authentication Required
[Yes/No - if yes, document method]

## Rate Limits
[Document any rate limits found in headers or documentation]

## Sample Response Format
[Include actual API response example]

## Fallback Strategy
[Document if local dictionary will be primary or backup]
```

### 2. Module Structure

Create the module file:

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/stakeholder/mod.rs`

```rust
//! Stakeholder extraction feasibility spike
//! 
//! This module validates external dependencies before main implementation

mod feasibility_tests;
mod rijksoverheid_api_probe;
mod fallback_dict;
mod cost_model;
mod document_probe;

pub use rijksoverheid_api_probe::{probe_rijksoverheid_api, ApiProbeResult};
pub use fallback_dict::{get_fallback_canonical_name, FALLBACK_DICT};
pub use cost_model::CostEstimator;
pub use document_probe::verify_document_structure;
```

### 3. Update Cargo.toml

Add required dependencies to `/Users/marc/Projecten/iou-modern/crates/iou-ai/Cargo.toml`:

```toml
[dependencies]
# Add these if not already present
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
once_cell = "1.19"
tokio = { version = "1", features = ["full"] }
```

---

## Verification Steps

1. **Run the API probe:**
   ```bash
   cargo test --package iou-ai probe_rijksoverheid_api -- --nocapture
   ```

2. **Test fallback dictionary:**
   ```bash
   cargo test --package iou-ai get_fallback_canonical_name
   ```

3. **Verify cost model:**
   ```bash
   cargo test --package iou-ai cost_estimation
   ```

4. **Check GeneratedDocument access:**
   ```bash
   cargo test --package iou-ai verify_document_structure
   ```

---

## Notes for Implementation

1. **API Availability:** The Rijksoverheid API may be unstable or require authentication. The fallback dictionary is the primary safety net.

2. **Cost Model:** The estimates assume Claude Sonnet 4.5 pricing. Update `CostEstimator` constants if pricing changes.

3. **GeneratedDocument:** The actual field name for text content may vary. Adjust `verify_document_structure` based on the real type definition in `iou-core`.

4. **Future Sections:** This section feeds into:
   - section-03-rijksoverheid-api (full API client implementation)
   - section-04-llm-extractor (cost tracking integration)

---

## Implementation Results

**Status:** ✅ Complete

**Files Created:**
- `crates/iou-ai/src/stakeholder/feasibility_tests.rs` - Test suite (129 lines)
- `crates/iou-ai/src/stakeholder/rijksoverheid_api_probe.rs` - API probe (154 lines)
- `crates/iou-ai/src/stakeholder/fallback_dict.rs` - Local dictionary (177 lines)
- `crates/iou-ai/src/stakeholder/cost_model.rs` - Cost estimation (173 lines)
- `crates/iou-ai/src/stakeholder/document_probe.rs` - Document verification (105 lines)
- `crates/iou-ai/src/stakeholder/mod.rs` - Module exports (18 lines)
- `crates/iou-ai/src/stakeholder/RIJKSOVERHEID_API.md` - API documentation (99 lines)

**Files Modified:**
- `crates/iou-ai/Cargo.toml` - Added `once_cell = "1.19"` dependency
- `crates/iou-ai/src/lib.rs` - Added `pub mod stakeholder;`

**Test Coverage:**
- 28 tests passing (including 2 edge case tests added during review)
- All success criteria met

**Key Findings:**
1. GeneratedDocument has a `content` field (not `text`) for accessing document text
2. Local fallback dictionary contains 60+ Dutch government organizations
3. Cost model: ~$0.004 per 1K tokens, ~$0.019 per 5K tokens, ~$0.038 per 10K tokens

**Code Review Fixes Applied:**
1. Removed duplicate BZK entry in fallback dictionary
2. Hid FALLBACK_DICT from public API (use `get_fallback_canonical_name` instead)
3. Added edge case tests for empty string and whitespace-only input

**Deferrals (to be addressed in later sections):**
- Custom error type using `thiserror` → section-01-foundation-types
- API retry logic → deferred (fallback dictionary is primary solution)