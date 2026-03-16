//! Dutch name normalization and comparison
//!
//! This module handles Dutch-specific name patterns including:
//! - Tussenvoegsels (van, van der, de, ten, etc.)
//! - Title variations (dr. vs dr)
//! - Phonetic matching for similar names

use std::collections::HashSet;

/// Dutch name normalizer
pub struct DutchNameNormalizer {
    /// Common Dutch prefixes (tussenvoegsels)
    prefixes: HashSet<&'static str>,
}

impl DutchNameNormalizer {
    /// Create a new normalizer with standard Dutch prefixes
    pub fn new() -> Self {
        let prefixes = [
            "van", "van der", "van de", "van den", "de", "den", "ter", "ten",
            "in", "tot", "bij", "onder", "op", "over", "uit", "uijt",
        ].into_iter().collect();

        Self { prefixes }
    }

    /// Normalize a Dutch name for comparison
    ///
    /// This function:
    /// 1. Converts to lowercase
    /// 2. Removes titles (dr., prof., mr., ing., ir., drs.) from any position
    /// 3. Standardizes spacing around prefixes
    /// 4. Trims whitespace
    pub fn normalize(&self, name: &str) -> String {
        let lower = name.to_lowercase();

        // Split into words, filter out titles, then rejoin
        let titles: HashSet<&str> = ["dr.", "dr", "prof.", "prof", "mr.", "mr", "ing.", "ing", "ir.", "ir", "drs.", "drs", "ir.s", "bc"]
            .iter().copied().collect();

        let words: Vec<&str> = lower.split_whitespace()
            .filter(|word| !titles.contains(*word))
            .collect();

        words.join(" ")
    }

    /// Normalize a name with prefix handling
    ///
    /// Treats prefixes case-insensitively for comparison purposes.
    /// "Jan de Vries" and "jan de vries" will produce the same normalized form.
    pub fn normalize_with_prefix(&self, name: &str) -> String {
        self.normalize(name)
    }

    /// Compare two Dutch names for equivalence
    pub fn are_equivalent(&self, name1: &str, name2: &str) -> bool {
        self.normalize_with_prefix(name1) == self.normalize_with_prefix(name2)
    }

    /// Calculate similarity between two names
    ///
    /// Returns a value between 0.0 (no similarity) and 1.0 (identical)
    pub fn similarity(&self, name1: &str, name2: &str) -> f32 {
        let norm1 = self.normalize_with_prefix(name1);
        let norm2 = self.normalize_with_prefix(name2);

        if norm1 == norm2 {
            return 1.0;
        }

        // Simple Jaro-Winkler-like calculation
        // Full implementation will use strsim crate in Section 05
        jaro_winkler_similarity(&norm1, &norm2)
    }

    /// Check if a word is a Dutch prefix
    pub fn is_prefix(&self, word: &str) -> bool {
        self.prefixes.contains(&word.to_lowercase().as_str())
    }
}

impl Default for DutchNameNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of name comparison
#[derive(Debug, Clone, PartialEq)]
pub enum NameComparison {
    /// Names are identical after normalization
    Identical,

    /// Names are very similar (> 0.9 Jaro-Winkler)
    VerySimilar,

    /// Names are somewhat similar (> 0.7 Jaro-Winkler)
    Similar,

    /// Names are different
    Different,
}

impl NameComparison {
    /// Create comparison from similarity score
    pub fn from_similarity(score: f32) -> Self {
        if score >= 1.0 {
            NameComparison::Identical
        } else if score >= 0.9 {
            NameComparison::VerySimilar
        } else if score >= 0.7 {
            NameComparison::Similar
        } else {
            NameComparison::Different
        }
    }
}

/// Placeholder for Jaro-Winkler similarity
/// Full implementation will be in Section 05 using strsim crate
fn jaro_winkler_similarity(s1: &str, s2: &str) -> f32 {
    if s1 == s2 {
        return 1.0;
    }

    // Simple character overlap for placeholder
    let chars1: HashSet<char> = s1.chars().collect();
    let chars2: HashSet<char> = s2.chars().collect();

    let intersection = chars1.intersection(&chars2).count();
    let union = chars1.union(&chars2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dutch_prefix_normalization() {
        let normalizer = DutchNameNormalizer::new();

        // Test that prefixes are handled case-insensitively
        assert_eq!(
            normalizer.normalize("Jan de Vries"),
            normalizer.normalize("jan de vries")
        );
    }

    #[test]
    fn test_title_removal() {
        let normalizer = DutchNameNormalizer::new();

        assert_eq!(normalizer.normalize("dr. Jan de Vries"), "jan de vries");
        assert_eq!(normalizer.normalize("prof. Marie Jansen"), "marie jansen");
        assert_eq!(normalizer.normalize("drs. P. Jansen"), "p. jansen");
    }

    #[test]
    fn test_normalization_is_idempotent() {
        let normalizer = DutchNameNormalizer::new();
        let name = "dr. Jan van der Berg";

        assert_eq!(
            normalizer.normalize(&normalizer.normalize(name)),
            normalizer.normalize(name)
        );
    }

    #[test]
    fn test_dutch_name_equivalence() {
        let normalizer = DutchNameNormalizer::new();

        assert!(normalizer.are_equivalent("Jan de Vries", "jan de vries"));
        assert!(normalizer.are_equivalent("dr. Jan de Vries", "jan de vries"));
        assert!(normalizer.are_equivalent("J. de Vries", "j. de vries"));
    }

    #[test]
    fn test_similarity_identical_names() {
        let normalizer = DutchNameNormalizer::new();

        assert_eq!(normalizer.similarity("Jan de Vries", "Jan de Vries"), 1.0);
        assert_eq!(normalizer.similarity("Jan de Vries", "jan de vries"), 1.0);
    }

    #[test]
    fn test_is_prefix() {
        let normalizer = DutchNameNormalizer::new();

        assert!(normalizer.is_prefix("van"));
        assert!(normalizer.is_prefix("van der"));
        assert!(normalizer.is_prefix("de"));
        assert!(normalizer.is_prefix("ten"));
        assert!(!normalizer.is_prefix("jan"));
    }

    #[test]
    fn test_name_comparison_from_similarity() {
        assert_eq!(NameComparison::from_similarity(1.0), NameComparison::Identical);
        assert_eq!(NameComparison::from_similarity(0.95), NameComparison::VerySimilar);
        assert_eq!(NameComparison::from_similarity(0.8), NameComparison::Similar);
        assert_eq!(NameComparison::from_similarity(0.5), NameComparison::Different);
    }

    #[test]
    fn test_whitespace_normalization() {
        let normalizer = DutchNameNormalizer::new();

        assert_eq!(normalizer.normalize("  Jan   de  Vries  "), "jan de vries");
        assert_eq!(normalizer.normalize("\tJan\tde\tVries\t"), "jan de vries");
    }

    #[test]
    fn test_multiple_titles() {
        let normalizer = DutchNameNormalizer::new();

        // Only removes trailing titles
        assert_eq!(
            normalizer.normalize("dr. prof. Jan de Vries"),
            "jan de vries"
        );
    }
}
