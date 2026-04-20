// =============================================================================
// Named Entity Recognition (NER) Module
// =============================================================================
//
// Local NLP fallback for entity extraction using Candle/transformers.
// Provides faster, cheaper alternative for high-confidence cases.

use crate::{Entity, EntityType, InferenceError};
use regex::Regex;
use std::collections::HashMap;

/// Local NER extractor using rule-based and pattern matching
pub struct LocalNerExtractor {
    // Dutch organization patterns
    org_patterns: Vec<Regex>,
    // Dutch location patterns
    loc_patterns: Vec<Regex>,
    // Dutch law/BWBR patterns
    law_patterns: Vec<Regex>,
    // Person name patterns
    person_patterns: Vec<Regex>,
}

impl LocalNerExtractor {
    /// Create a new local NER extractor
    pub fn new() -> Result<Self, InferenceError> {
        Ok(Self {
            org_patterns: vec![
                Regex::new(r"\b(?:Gemeente|Provincie|Ministerie van|Rijk|Het )(?:\w+\s*){1,3}\b")
                    .map_err(|e| InferenceError::Internal(e.into()))?,
                Regex::new(r"\b\w+(?:sburg|dam|dijk|gaard|hoek|veen)\b")
                    .map_err(|e| InferenceError::Internal(e.into()))?,
            ],
            loc_patterns: vec![
                Regex::new(r"\b(?:in|te|te|uit|vanuit) ([A-Z][a-z]+)\b")
                    .map_err(|e| InferenceError::Internal(e.into()))?,
                Regex::new(r"\b[A-Z][a-z]+(?:straat|laan|weg|plein|park|kade)\b")
                    .map_err(|e| InferenceError::Internal(e.into()))?,
            ],
            law_patterns: vec![
                Regex::new(r"\bBWBR\d{7}\b")
                    .map_err(|e| InferenceError::Internal(e.into()))?,
                Regex::new(r"\b(?:Algemene Wet|Wet|Besluit|Uitvoeringsbesluit).+\b")
                    .map_err(|e| InferenceError::Internal(e.into()))?,
            ],
            person_patterns: vec![
                Regex::new(r"\b[A-Z][a-z]+ [A-Z][a-z]+\b")
                    .map_err(|e| InferenceError::Internal(e.into()))?,
            ],
        })
    }

    /// Extract entities from text
    pub fn extract_entities(&self, text: &str) -> Result<Vec<Entity>, InferenceError> {
        let mut entities = Vec::new();
        let mut seen = HashMap::new();

        // Extract organizations
        for pattern in &self.org_patterns {
            for mat in pattern.find_iter(text) {
                let key = (mat.start(), EntityType::Organisatie);
                if seen.insert(key, mat.as_str()).is_some() {
                    continue;
                }
                entities.push(Entity {
                    entity_type: EntityType::Organisatie,
                    text: mat.as_str().to_string(),
                    start_offset: mat.start(),
                    end_offset: mat.end(),
                    confidence: 0.7,
                    metadata: None,
                });
            }
        }

        // Extract locations
        for pattern in &self.loc_patterns {
            for mat in pattern.find_iter(text) {
                let key = (mat.start(), EntityType::Locatie);
                if seen.insert(key, mat.as_str()).is_some() {
                    continue;
                }
                entities.push(Entity {
                    entity_type: EntityType::Locatie,
                    text: mat.as_str().to_string(),
                    start_offset: mat.start(),
                    end_offset: mat.end(),
                    confidence: 0.6,
                    metadata: None,
                });
            }
        }

        // Extract laws
        for pattern in &self.law_patterns {
            for mat in pattern.find_iter(text) {
                let key = (mat.start(), EntityType::Wet);
                if seen.insert(key, mat.as_str()).is_some() {
                    continue;
                }
                entities.push(Entity {
                    entity_type: EntityType::Wet,
                    text: mat.as_str().to_string(),
                    start_offset: mat.start(),
                    end_offset: mat.end(),
                    confidence: 0.9,
                    metadata: None,
                });
            }
        }

        Ok(entities)
    }

    /// Check if text likely contains Dutch legal content
    pub fn is_dutch_legal(&self, text: &str) -> bool {
        let legal_keywords = [
            "grondslag", "BWBR", "artikel", "besluit", "wet", "Algemene wet",
            "verordening", "AMvB", "Kamerstuk", "Staatsblad",
        ];

        let text_lower = text.to_lowercase();
        legal_keywords.iter().any(|&kw| text_lower.contains(kw))
    }
}

impl Default for LocalNerExtractor {
    fn default() -> Self {
        Self::new().expect("failed to create LocalNerExtractor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bwbr() {
        let extractor = LocalNerExtractor::new().unwrap();
        let text = "Op grond van BWBR0003415, artikel 5, lid 2";
        let entities = extractor.extract_entities(text).unwrap();

        assert!(!entities.is_empty());
        assert_eq!(entities[0].entity_type, EntityType::Wet);
    }

    #[test]
    fn test_dutch_legal_detection() {
        let extractor = LocalNerExtractor::new().unwrap();
        assert!(extractor.is_dutch_legal("BWBR0003415 artikel 5"));
        assert!(!extractor.is_dutch_legal("Hello world"));
    }
}
