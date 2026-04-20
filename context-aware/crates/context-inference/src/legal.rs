// =============================================================================
// Legal Context Module
// =============================================================================
//
// Extraction and validation of legal context (grondslagen) from documents.
// Handles Dutch law references, BWBR identifiers, and article parsing.

use crate::{InferredGrondslag, InferenceError};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Legal context extractor
pub struct LegalExtractor {
    bwbr_pattern: Regex,
    artikel_pattern: Regex,
}

impl LegalExtractor {
    /// Create a new legal extractor
    pub fn new() -> Result<Self, InferenceError> {
        Ok(Self {
            bwbr_pattern: Regex::new(r"BWBR(\d{7})")
                .map_err(|e| InferenceError::Internal(e.into()))?,
            artikel_pattern: Regex::new(r"(?:artikel|art)\.?\s*(\d+)[:,]?\s*(?:lid\s*(\d+))?")
                .map_err(|e| InferenceError::Internal(e.into()))?,
        })
    }

    /// Extract grondslagen from text
    pub fn extract_grondslagen(&self, text: &str) -> Result<Vec<InferredGrondslag>, InferenceError> {
        let mut grondslagen = Vec::new();

        // Find all BWBR references
        for mat in self.bwbr_pattern.find_iter(text) {
            let bwbr_id = mat.as_str();
            let artikel = self.find_artikel_context(text, mat.start());

            grondslagen.push(InferredGrondslag {
                grondslag_id: bwbr_id.to_string(),
                bron: "Onbekend".to_string(),  // Would need lookup service
                artikel,
                confidence: 0.95,
            });
        }

        Ok(grondslagen)
    }

    /// Find article context near a BWBR reference
    fn find_artikel_context(&self, text: &str, pos: usize) -> Option<String> {
        // Look within 100 chars before and after
        let start = pos.saturating_sub(100);
        let end = (pos + 100).min(text.len());
        let context = &text[start..end];

        self.artikel_pattern
            .find(context)
            .map(|mat| mat.as_str().to_string())
    }

    /// Validate a grondslag ID format
    pub fn validate_grondslag_id(&self, id: &str) -> bool {
        self.bwbr_pattern.is_match(id)
    }

    /// Parse article reference
    pub fn parse_artikel(&self, text: &str) -> Option<ParsedArtikel> {
        let caps = self.artikel_pattern.captures(text)?;
        Some(ParsedArtikel {
            artikel: caps.get(1)?.as_str().parse().ok()?,
            lid: caps.get(2).and_then(|m| m.as_str().parse().ok()),
        })
    }
}

impl Default for LegalExtractor {
    fn default() -> Self {
        Self::new().expect("failed to create LegalExtractor")
    }
}

/// Parsed article reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedArtikel {
    pub artikel: u32,
    pub lid: Option<u32>,
}

/// Known Dutch law sources for BWBR lookup
pub struct KnownWetten;

impl KnownWetten {
    /// Get law name from BWBR ID
    pub fn lookup_name(bwbr: &str) -> Option<&'static str> {
        match bwbr {
            "BWBR0003415" => Some("Algemene wet bestuursrecht"),
            "BWBR0002615" => Some("Wet openbaarheid van bestuur"),
            "BWBR0005537" => Some("Wet bescherming persoonsgegevens"),
            "BWBR0007065" => Some("Archiefwet"),
            "BWBR0018534" => Some("Wet titels onderscheiden functies"),
            _ => None,
        }
    }

    /// List all known BWBR IDs
    pub fn known_ids() -> &'static [&'static str] {
        &[
            "BWBR0003415",
            "BWBR0002615",
            "BWBR0005537",
            "BWBR0007065",
            "BWBR0018534",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bwbr() {
        let extractor = LegalExtractor::new().unwrap();
        let text = "Op grond van BWBR0003415, artikel 5";
        let grondslagen = extractor.extract_grondslagen(text).unwrap();

        assert_eq!(grondslagen.len(), 1);
        assert_eq!(grondslagen[0].grondslag_id, "BWBR0003415");
    }

    #[test]
    fn test_parse_artikel() {
        let extractor = LegalExtractor::new().unwrap();

        assert_eq!(
            extractor.parse_artikel("artikel 5"),
            Some(ParsedArtikel { artikel: 5, lid: None })
        );

        assert_eq!(
            extractor.parse_artikel("art. 5:3"),
            Some(ParsedArtikel { artikel: 5, lid: Some(3) })
        );
    }

    #[test]
    fn test_lookup_name() {
        assert_eq!(
            KnownWetten::lookup_name("BWBR0003415"),
            Some("Algemene wet bestuursrecht")
        );
        assert!(KnownWetten::lookup_name("BWBR9999999").is_none());
    }
}
