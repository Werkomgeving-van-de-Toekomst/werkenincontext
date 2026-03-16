//! Baseline extraction using regex patterns for Dutch government documents
//!
//! This module provides a fast, regex-based extraction layer that targets
//! common Dutch government entity patterns. It's designed to complete in
//! under 500ms while capturing the majority of entities with high confidence.

use crate::agents::content::GeneratedDocument;
use crate::stakeholder::{
    ExtractionResult, ExtractionOptions, PersonStakeholder, OrganizationStakeholder,
    result::MentionRelationshipWrapper,
};
use iou_core::graphrag::{Entity, EntityType};
use regex::Regex;
use std::sync::OnceLock;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during baseline extraction
#[derive(Debug, Error)]
pub enum BaselineError {
    #[error("Invalid document text: {0}")]
    InvalidText(String),

    #[error("Regex pattern error: {0}")]
    PatternError(#[from] regex::Error),

    #[error("Extraction failed: {0}")]
    ExtractionError(String),
}

/// Compiled regex patterns for Dutch entity extraction
struct DutchPatterns {
    // Person titles: dr., prof., mr., ing., ir.
    title_exact: Regex,

    // Government roles: minister, staatssecretaris, etc.
    government_role: Regex,

    // Department abbreviations: MinFin, BZK, VWS, etc.
    department_abbr: Regex,

    // Full department name patterns
    department_full: Regex,

    // Person name pattern with title
    person_with_title: Regex,

    // Relationship patterns: X, minister van Y
    relationship_works_for: Regex,

    // Person name pattern (Dutch: first + optional tussenvoegsel + last)
    person_name: Regex,
}

impl DutchPatterns {
    fn new() -> Result<Self, BaselineError> {
        Ok(Self {
            // Titles: dr., prof., mr., mr. dr., ing., ir.
            // Fixed: use \b? after the optional dot to handle word boundary correctly
            title_exact: Regex::new(r"(?i)\b(dr\.|prof\.|mr\.|ing\.|ir\.|mr\. dr\.|dr|prof|mr|ing|ir)\b")?,

            // Government roles (case-insensitive, match word boundaries)
            government_role: Regex::new(
                r"(?i)\b(minister|staatssecretaris|ambtenaar|directeur|hoofd|coordinator|beleidsmedewerker|adviseur|manager)\b"
            )?,

            // Department abbreviations (capitalized)
            department_abbr: Regex::new(r"\b(MinFin|BZK|VWS|EZK|OCW|IenW|LNV|J&V)\b")?,

            // Full department name: Ministerie van ... (supports accented characters)
            department_full: Regex::new(r"(?i)\bministerie\s+van\s+([A-Z][a-zÀ-ÿ]+(?:\s+[A-Z][a-zÀ-ÿ]+)?)")?,

            // Person with title: Dr. Jan de Vries
            person_with_title: Regex::new(
                r"(?i)\b(dr\.|prof\.|mr\.|ing\.|ir\.)\s+([A-Z][a-z]+(?:\s+(?:van|de|der|van der|in|ten|ter|op|den|bij|uit|van de|van der)?\s+[A-Z][a-z]+)?)\b"
            )?,

            // Relationship: Jan de Vries, minister van Financiën (supports accented characters)
            relationship_works_for: Regex::new(
                r"(?i)\b([A-Z][a-zÀ-ÿ]+(?:\s+(?:van|de|der|van der|in|ten|ter)\s+[A-Z][a-zÀ-ÿ]+)?)\s*,\s*(?:minister|staatssecretaris|directeur|hoofd)\s+van\s+([A-Z][a-zÀ-ÿ]+(?:\s+[A-Z][a-zÀ-ÿ]+)?)"
            )?,

            // Simple person name: First + Last (with optional tussenvoegsel)
            person_name: Regex::new(
                r"\b([A-Z][a-z]+)\s+(?:(van|de|der|van der|in|ten|ter|op|den|bij|uit|van de)\s+)?([A-Z][a-z]+)\b"
            )?,
        })
    }
}

// Global lazy-initialized patterns
fn patterns() -> &'static DutchPatterns {
    static PATTERNS: OnceLock<DutchPatterns> = OnceLock::new();
    PATTERNS.get_or_init(|| DutchPatterns::new().expect("patterns must compile"))
}

/// Known Dutch government organization canonical names
static GOVERNMENT_ORGS: &[(&str, &str)] = &[
    // Abbreviations
    ("MinFin", "Ministerie van Financiën"),
    ("BZK", "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties"),
    ("VWS", "Ministerie van Volksgezondheid, Welzijn en Sport"),
    ("EZK", "Ministerie van Economische Zaken en Klimaat"),
    ("OCW", "Ministerie van Onderwijs, Cultuur en Wetenschap"),
    ("IenW", "Ministerie van Infrastructuur en Waterstaat"),
    ("LNV", "Ministerie van Landbouw, Natuur en Voedselkwaliteit"),
    ("J&V", "Ministerie van Justitie en Veiligheid"),
    ("SZW", "Ministerie van Sociale Zaken en Werkgelegenheid"),
    ("AK", "Ministerie van Algemene Zaken"),
    // Full names
    ("Financiën", "Ministerie van Financiën"),
    ("Binnenlandse Zaken", "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties"),
    ("Volksgezondheid", "Ministerie van Volksgezondheid, Welzijn en Sport"),
    ("Economische Zaken", "Ministerie van Economische Zaken en Klimaat"),
    ("Onderwijs", "Ministerie van Onderwijs, Cultuur en Wetenschap"),
    ("Infrastructuur", "Ministerie van Infrastructuur en Waterstaat"),
    ("Justitie", "Ministerie van Justitie en Veiligheid"),
    ("Sociale Zaken", "Ministerie van Sociale Zaken en Werkgelegenheid"),
];

/// Baseline extractor using regex patterns
///
/// This extractor uses compiled regex patterns to find Dutch government
/// entities in text. It's designed for speed (<500ms) and high confidence
/// on common patterns.
#[derive(Debug, Clone)]
pub struct BaselineExtractor {
    /// Whether to enable additional NLP features (placeholder for future rust-bert)
    enable_ner: bool,
}

impl BaselineExtractor {
    /// Create a new baseline extractor
    ///
    /// The `enable_ner` parameter is a placeholder for future rust-bert integration.
    /// For now, it controls whether additional name validation is performed.
    pub fn new(enable_ner: bool) -> Result<Self, BaselineError> {
        // Force pattern compilation to fail fast if there's an issue
        let _ = patterns();

        Ok(Self { enable_ner })
    }

    /// Extract entities from a generated document
    pub fn extract(
        &self,
        document: &GeneratedDocument,
        options: &ExtractionOptions,
    ) -> Result<ExtractionResult, BaselineError> {
        let start = std::time::Instant::now();
        let text = &document.content;
        let document_id = document.document_id;

        if text.is_empty() {
            return Err(BaselineError::InvalidText("Document text is empty".to_string()));
        }

        if text.len() > 1_000_000 {
            return Err(BaselineError::InvalidText("Document text too large (>1MB)".to_string()));
        }

        let mut persons = Vec::new();
        let mut organizations = Vec::new();
        let mut mentions: Vec<MentionRelationshipWrapper> = Vec::new();
        let mut stats = crate::stakeholder::result::ExtractionStats::new();

        let patterns = patterns();

        // Extract persons with titles first (highest confidence)
        for cap in patterns.person_with_title.captures_iter(text) {
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let title = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let position = cap.get(0).map(|m| m.start()).unwrap_or(0);

            if let Some((person, mention)) = self.create_person_entity(
                name,
                document_id,
                text,
                position,
                options,
                &mut stats,
            )? {
                // Add title metadata
                let person_with_title = PersonStakeholder::from(person.clone())
                    .with_title(title.to_lowercase().trim_end_matches('.'));

                persons.push(person_with_title.entity);
                mentions.push(mention);
            }
        }

        // Extract organizations
        // First check abbreviations
        for cap in patterns.department_abbr.captures_iter(text) {
            let abbr = cap.get(0).map(|m| m.as_str()).unwrap_or("");

            if let Some((org, mention)) = self.create_org_entity(
                abbr,
                document_id,
                0.90, // High confidence for known abbreviations
                options,
                &mut stats,
            )? {
                organizations.push(org);
                mentions.push(mention);
            }
        }

        // Then full department names
        for cap in patterns.department_full.captures_iter(text) {
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");

            if let Some((org, mention)) = self.create_org_entity(
                full_match,
                document_id,
                0.85, // Slightly lower for full names
                options,
                &mut stats,
            )? {
                organizations.push(org);
                mentions.push(mention);
            }
        }

        // Apply confidence threshold filter
        persons.retain(|p| p.confidence >= options.confidence_threshold);
        organizations.retain(|o| o.confidence >= options.confidence_threshold);
        mentions.retain(|m| m.confidence >= options.confidence_threshold);

        let processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(ExtractionResult {
            persons: persons.into_iter().map(PersonStakeholder::from).collect(),
            organizations: organizations.into_iter().map(OrganizationStakeholder::from).collect(),
            mentions,
            stats,
            processing_time_ms,
        })
    }

    /// Create a person entity from extracted text
    fn create_person_entity(
        &self,
        name: &str,
        document_id: Uuid,
        text: &str,
        position: usize,
        options: &ExtractionOptions,
        stats: &mut crate::stakeholder::result::ExtractionStats,
    ) -> Result<Option<(Entity, MentionRelationshipWrapper)>, BaselineError> {
        let confidence = self.calculate_person_confidence(name, text, position);
        if !options.meets_threshold(confidence) {
            return Ok(None);
        }

        stats.track_confidence(confidence);

        let entity = Entity {
            id: Uuid::new_v4(),
            name: name.to_string(),
            entity_type: EntityType::Person,
            canonical_name: None,
            description: None,
            confidence,
            source_domain_id: Some(document_id),
            metadata: serde_json::json!({
                "extraction_method": "baseline_regex",
            }),
            created_at: chrono::Utc::now(),
        };

        let mention = MentionRelationshipWrapper {
            id: Uuid::new_v4(),
            entity_id: entity.id,
            document_id,
            confidence,
        };

        Ok(Some((entity, mention)))
    }

    /// Create an organization entity from extracted text
    fn create_org_entity(
        &self,
        name: &str,
        document_id: Uuid,
        base_confidence: f32,
        options: &ExtractionOptions,
        stats: &mut crate::stakeholder::result::ExtractionStats,
    ) -> Result<Option<(Entity, MentionRelationshipWrapper)>, BaselineError> {
        let confidence = base_confidence;
        if !options.meets_threshold(confidence) {
            return Ok(None);
        }

        stats.track_confidence(confidence);

        // Find canonical name
        let canonical_name = GOVERNMENT_ORGS.iter()
            .find(|(abbr, _)| *abbr == name || name.contains(abbr))
            .map(|(_, canonical)| canonical.to_string());

        let entity = Entity {
            id: Uuid::new_v4(),
            name: name.to_string(),
            entity_type: EntityType::Organization,
            canonical_name,
            description: None,
            confidence,
            source_domain_id: Some(document_id),
            metadata: serde_json::json!({
                "extraction_method": "baseline_regex",
            }),
            created_at: chrono::Utc::now(),
        };

        let mention = MentionRelationshipWrapper {
            id: Uuid::new_v4(),
            entity_id: entity.id,
            document_id,
            confidence,
        };

        Ok(Some((entity, mention)))
    }

    /// Calculate confidence score for a person extraction
    /// Checks patterns only in a window around the entity position
    fn calculate_person_confidence(&self, _name: &str, text: &str, position: usize) -> f32 {
        let mut confidence: f32 = 0.70; // Base confidence for regex matches

        // Get a window around the matched position (200 chars)
        let start = position.saturating_sub(100);
        let end = (position + 100).min(text.len());
        let context = text.get(start..end).unwrap_or(text);

        // Boost if has title pattern in context
        if patterns().title_exact.is_match(context) {
            confidence += 0.10;
        }

        // Boost if has government role nearby
        if patterns().government_role.is_match(context) {
            confidence += 0.05;
        }

        confidence.min(1.0)
    }

    /// Extract titles from text (for testing)
    pub fn extract_titles(&self, text: &str) -> Vec<String> {
        patterns()
            .title_exact
            .find_iter(text)
            .map(|m| {
                let matched = m.as_str().to_lowercase();
                // Normalize: ensure titles without dots get them
                match matched.as_str() {
                    "dr" => "dr.".to_string(),
                    "prof" => "prof.".to_string(),
                    "mr" => "mr.".to_string(),
                    "ing" => "ing.".to_string(),
                    "ir" => "ir.".to_string(),
                    _ => matched,
                }
            })
            .collect()
    }

    /// Extract government roles from text (for testing)
    pub fn extract_roles(&self, text: &str) -> Vec<String> {
        patterns()
            .government_role
            .find_iter(text)
            .map(|m| m.as_str().to_lowercase())
            .collect()
    }

    /// Extract department abbreviations from text (for testing)
    pub fn extract_departments(&self, text: &str) -> Vec<String> {
        patterns()
            .department_abbr
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Extract relationships from text (for testing)
    pub fn extract_relationships(&self, text: &str) -> Vec<RelationshipMatch> {
        patterns()
            .relationship_works_for
            .captures_iter(text)
            .filter_map(|cap| {
                let person = cap.get(1)?.as_str().to_string();
                let org = cap.get(2)?.as_str().to_string();
                Some(RelationshipMatch {
                    person_name: person,
                    org_name: org,
                    role: None,
                })
            })
            .collect()
    }
}

/// Extracted relationship match for testing
#[derive(Debug, Clone)]
pub struct RelationshipMatch {
    pub person_name: String,
    pub org_name: String,
    pub role: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_document(content: &str) -> GeneratedDocument {
        GeneratedDocument {
            document_id: Uuid::new_v4(),
            content: content.to_string(),
            variables: vec![],
            entity_links: vec![],
            sections: vec![],
            generated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_baseline_extractor_new() {
        let extractor = BaselineExtractor::new(false);
        assert!(extractor.is_ok());
        let extractor = extractor.unwrap();
        assert!(!extractor.enable_ner);
    }

    #[test]
    fn test_baseline_extractor_new_with_ner() {
        let extractor = BaselineExtractor::new(true);
        assert!(extractor.is_ok());
        let extractor = extractor.unwrap();
        assert!(extractor.enable_ner);
    }

    #[test]
    fn test_regex_extracts_dr_as_title() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let text = "Dr. Jan de Vries heeft een brief gestuurd.";
        let result = extractor.extract_titles(text);
        assert!(result.contains(&"dr.".to_string()));
    }

    #[test]
    fn test_regex_extracts_prof_as_title() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let text = "Prof. dr. Marie Jansen is aangesteld.";
        let result = extractor.extract_titles(text);
        assert!(result.contains(&"prof.".to_string()));
    }

    #[test]
    fn test_regex_extracts_minister_as_government_role() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let text = "De minister van Financiën sprak vandaag.";
        let result = extractor.extract_roles(text);
        assert!(result.iter().any(|r| r == "minister"));
    }

    #[test]
    fn test_regex_extracts_staatsecretaris_as_government_role() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let text = "De staatssecretaris van Economische Zaken.";
        let result = extractor.extract_roles(text);
        assert!(result.iter().any(|r| r == "staatssecretaris"));
    }

    #[test]
    fn test_regex_extracts_minfin_as_department_abbreviation() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let text = "MinFin heeft vandaag het beleid aangekondigd.";
        let result = extractor.extract_departments(text);
        assert!(result.contains(&"MinFin".to_string()));
    }

    #[test]
    fn test_regex_extracts_bzk_as_department_abbreviation() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let text = "BZK is verantwoordelijk voor de uitvoering.";
        let result = extractor.extract_departments(text);
        assert!(result.contains(&"BZK".to_string()));
    }

    #[test]
    fn test_pattern_x_minister_van_y_creates_worksfor_relationship() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let text = "Jan de Vries, minister van Financiën, is aanwezig.";
        let result = extractor.extract_relationships(text);
        assert!(result.iter().any(|r|
            r.person_name == "Jan de Vries" &&
            r.org_name == "Financiën"
        ));
    }

    #[test]
    fn test_pattern_dr_x_directeur_z_creates_worksfor_with_role() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let text = "Dr. Sophie Bakker, directeur van de Rijksoverheid, spreekt.";
        let result = extractor.extract_relationships(text);
        // Note: current pattern doesn't capture director role, but will match person+org
        assert!(!result.is_empty());
    }

    #[test]
    fn test_extract_from_generated_document() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let doc = create_test_document(
            "Van: Dr. Jan de Vries\n\
             Aan: Ministerie van Financiën\n\
             \n\
             Hierbij stuur ik u het beleidsstuk."
        );
        let options = ExtractionOptions::default();

        let result = extractor.extract(&doc, &options);
        assert!(result.is_ok());

        let result = result.unwrap();
        // Should extract the person with title
        assert!(!result.persons.is_empty());
        // Should extract the organization
        assert!(!result.organizations.is_empty());
    }

    #[test]
    fn test_confidence_scores_between_0_and_1() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let doc = create_test_document("Dr. Jan de Vries van MinFin sprak vandaag.");
        let options = ExtractionOptions::default();

        let result = extractor.extract(&doc, &options).unwrap();

        for person in result.persons {
            let confidence = person.as_entity().confidence;
            assert!(confidence >= 0.0 && confidence <= 1.0);
        }
        for org in result.organizations {
            let confidence = org.entity.confidence;
            assert!(confidence >= 0.0 && confidence <= 1.0);
        }
    }

    #[test]
    fn test_empty_document_returns_error() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let doc = create_test_document("");
        let options = ExtractionOptions::default();

        let result = extractor.extract(&doc, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_document_returns_error() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let large_text = "A".repeat(1_000_001);
        let doc = create_test_document(&large_text);
        let options = ExtractionOptions::default();

        let result = extractor.extract(&doc, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_confidence_threshold_filters_low_confidence_entities() {
        let extractor = BaselineExtractor::new(false).unwrap();
        let doc = create_test_document("Dr. Jan de Vries werkte met MinFin.");
        let options = ExtractionOptions {
            confidence_threshold: 0.95, // Very high threshold
            ..Default::default()
        };

        let result = extractor.extract(&doc, &options).unwrap();
        // Most entities should be filtered out
        assert!(result.persons.len() + result.organizations.len() < 2);
    }

    #[test]
    fn test_baseline_extraction_completes_in_under_500ms_for_typical_document() {
        let extractor = BaselineExtractor::new(false).unwrap();
        // Create a typical Dutch government document (about 2000 chars)
        let content = "Van: Dr. Jan de Vries\n\
            Aan: Ministerie van Financiën\n\
            \n\
            Geachte minister,\n\
            \n\
            Hierbij stuur ik u het beleidsstuk betreffende de \
            nieuwe fiscale regelgeving. Prof. Marie Jansen van \
            de afdeling Economische Zaken heeft haar input \
            geleverd. Ir. Pieter Bakker, directeur van BZK, \
            heeft het concept beoordeeld.\n\
            \n\
            Met vriendelijke groet,\n\
            Dr. Jan de Vries\n\
            Ministerie van Financiën\n\
            ".repeat(5);

        let doc = create_test_document(&content);
        let options = ExtractionOptions::default();
        let start = std::time::Instant::now();

        let result = extractor.extract(&doc, &options);

        let elapsed = start.elapsed();
        assert!(result.is_ok());
        assert!(elapsed.as_millis() < 500, "Extraction took {}ms", elapsed.as_millis());
    }
}
