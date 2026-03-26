//! Cost estimation model for LLM API usage
//!
//! Provides cost estimation for entity extraction using Claude Sonnet 4.5 API.
//! Pricing is current as of 2025 and should be updated if API pricing changes.

/// Cost estimation for Claude Sonnet 4.5 API calls
///
/// Pricing (as of 2025):
/// - Input: $3.00 per 1M tokens
/// - Output: $15.00 per 1M tokens
///
/// These constants should be updated if Claude pricing changes.
pub struct CostEstimator;

impl CostEstimator {
    /// Input cost per million tokens (Claude Sonnet 4.5, 2025 pricing)
    const INPUT_COST_PER_M: f32 = 3.00;

    /// Output cost per million tokens (Claude Sonnet 4.5, 2025 pricing)
    const OUTPUT_COST_PER_M: f32 = 15.00;

    /// Estimate cost for a document with given token counts
    ///
    /// # Arguments
    ///
    /// * `input_tokens` - Number of tokens in the input (document text)
    /// * `output_tokens` - Number of tokens in the output (extracted entities)
    ///
    /// # Returns
    ///
    /// Estimated cost in USD
    ///
    /// # Example
    ///
    /// ```
    /// use iou_ai::stakeholder::CostEstimator;
    ///
    /// // 1K input tokens, 50 output tokens
    /// let cost = CostEstimator::estimate_cost(1000, 50);
    /// assert!(cost < 0.01); // ~$0.006
    /// ```
    pub fn estimate_cost(input_tokens: u32, output_tokens: u32) -> f32 {
        let input_cost = (input_tokens as f32 / 1_000_000.0) * Self::INPUT_COST_PER_M;
        let output_cost = (output_tokens as f32 / 1_000_000.0) * Self::OUTPUT_COST_PER_M;
        input_cost + output_cost
    }

    /// Estimate cost for document based on approximate page count
    ///
    /// Assumes: ~500 tokens per page for Dutch government documents.
    /// Output is typically 5% of input for entity extraction tasks.
    ///
    /// # Arguments
    ///
    /// * `page_count` - Number of pages in the document
    ///
    /// # Returns
    ///
    /// A tuple of (estimated_cost, input_tokens, output_tokens)
    ///
    /// # Example
    ///
    /// ```
    /// use iou_ai::stakeholder::CostEstimator;
    ///
    /// let (cost, input, output) = CostEstimator::estimate_cost_by_pages(10);
    /// assert_eq!(input, 5000); // 10 pages * 500 tokens
    /// assert_eq!(output, 250);  // 5% of input
    /// ```
    pub fn estimate_cost_by_pages(page_count: u32) -> (f32, u32, u32) {
        const TOKENS_PER_PAGE: u32 = 500;
        const OUTPUT_RATIO: u32 = 20; // Output is 1/20 of input (5%)

        let total_tokens = page_count * TOKENS_PER_PAGE;
        let input_tokens = total_tokens;
        let output_tokens = total_tokens / OUTPUT_RATIO;

        let cost = Self::estimate_cost(input_tokens, output_tokens);
        (cost, input_tokens, output_tokens)
    }

    /// Check if estimated cost is within budget
    ///
    /// # Arguments
    ///
    /// * `cost` - The estimated cost
    /// * `max_cost` - Maximum acceptable cost
    ///
    /// # Returns
    ///
    /// `true` if cost is within or equal to the budget, `false` otherwise
    pub fn is_within_budget(cost: f32, max_cost: f32) -> bool {
        cost <= max_cost
    }

    /// Get the current pricing configuration
    ///
    /// # Returns
    ///
    /// A tuple of (input_cost_per_m, output_cost_per_m)
    pub fn get_pricing() -> (f32, f32) {
        (Self::INPUT_COST_PER_M, Self::OUTPUT_COST_PER_M)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimation_1k_tokens() {
        // Short document: 1K input, 50 output (5%)
        let cost = CostEstimator::estimate_cost(1000, 50);
        // Expected: (1000/1M * $3) + (50/1M * $15) = $0.003 + $0.00075 = $0.00375
        assert!(cost > 0.0 && cost < 0.01, "Cost should be ~$0.004, got ${}", cost);
        assert!((cost - 0.004).abs() < 0.001, "Cost should be ~$0.004, got ${}", cost);
    }

    #[test]
    fn test_cost_estimation_5k_tokens() {
        // Medium document: 5K input, 250 output
        let cost = CostEstimator::estimate_cost(5000, 250);
        // Expected: (5000/1M * $3) + (250/1M * $15) = $0.015 + $0.00375 = $0.01875
        assert!(cost > 0.01 && cost < 0.03, "Cost should be ~$0.019, got ${}", cost);
        assert!((cost - 0.019).abs() < 0.003, "Cost should be ~$0.019, got ${}", cost);
    }

    #[test]
    fn test_cost_estimation_10k_tokens() {
        // Long document: 10K input, 500 output
        let cost = CostEstimator::estimate_cost(10000, 500);
        // Expected: (10000/1M * $3) + (500/1M * $15) = $0.03 + $0.0075 = $0.0375
        assert!(cost > 0.02 && cost < 0.05, "Cost should be ~$0.038, got ${}", cost);
        assert!((cost - 0.038).abs() < 0.005, "Cost should be ~$0.038, got ${}", cost);
    }

    #[test]
    fn test_is_within_budget() {
        assert!(CostEstimator::is_within_budget(0.05, 0.10));
        assert!(!CostEstimator::is_within_budget(0.15, 0.10));
        assert!(CostEstimator::is_within_budget(0.10, 0.10));
        assert!(CostEstimator::is_within_budget(0.0, 0.10));
    }

    #[test]
    fn test_cost_estimation_by_pages() {
        let (cost, input, output) = CostEstimator::estimate_cost_by_pages(1);
        assert_eq!(input, 500);
        assert_eq!(output, 25); // 5% of 500
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cost_estimation_by_pages_10() {
        let (cost, input, output) = CostEstimator::estimate_cost_by_pages(10);
        assert_eq!(input, 5000);
        assert_eq!(output, 250); // 5% of 5000
        assert!((cost - 0.019).abs() < 0.005, "Cost should be ~$0.019, got ${}", cost);
    }

    #[test]
    fn test_get_pricing() {
        let (input, output) = CostEstimator::get_pricing();
        assert_eq!(input, 3.00);
        assert_eq!(output, 15.00);
    }

    #[test]
    fn test_zero_tokens() {
        let cost = CostEstimator::estimate_cost(0, 0);
        assert_eq!(cost, 0.0);
    }
}
