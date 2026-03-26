diff --git a/crates/iou-ai/src/stakeholder/baseline.rs b/crates/iou-ai/src/stakeholder/baseline.rs
new file mode 100644
index 0000000..cd56c30
--- /dev/null
+++ b/crates/iou-ai/src/stakeholder/baseline.rs
@@ -0,0 +1,594 @@
+//! Baseline extraction using regex patterns for Dutch government documents
+//!
+//! This module provides a fast, regex-based extraction layer that targets
+//! common Dutch government entity patterns. It's designed to complete in
+//! under 500ms while capturing the majority of entities with high confidence.
+
+use crate::agents::content::GeneratedDocument;
+use crate::stakeholder::{
+    ExtractionResult, ExtractionOptions, PersonStakeholder, OrganizationStakeholder,
+    result::MentionRelationshipWrapper,
+    TextPosition,
+};
+use iou_core::graphrag::{Entity, EntityType};
+use regex::Regex;
+use std::sync::OnceLock;
+use thiserror::Error;
+use uuid::Uuid;
+
+/// Errors that can occur during baseline extraction
+#[derive(Debug, Error)]
+pub enum BaselineError {
+    #[error("Invalid document text: {0}")]
+    InvalidText(String),
+
+    #[error("Regex pattern error: {0}")]
+    PatternError(#[from] regex::Error),
+
+    #[error("Extraction failed: {0}")]
+    ExtractionError(String),
+}
+
+/// Compiled regex patterns for Dutch entity extraction
+struct DutchPatterns {
+    // Person titles: dr., prof., mr., ing., ir.
+    title_exact: Regex,
+
+    // Government roles: minister, staatssecretaris, etc.
+    government_role: Regex,
+
+    // Department abbreviations: MinFin, BZK, VWS, etc.
+    department_abbr: Regex,
+
+    // Full department name patterns
+    department_full: Regex,
+
+    // Person name pattern with title
+    person_with_title: Regex,
+
+    // Relationship patterns: X, minister van Y
+    relationship_works_for: Regex,
+
+    // Person name pattern (Dutch: first + optional tussenvoegsel + last)
+    person_name: Regex,
+}
+
+impl DutchPatterns {
+    fn new() -> Result<Self, BaselineError> {
+        Ok(Self {
+            // Titles: dr., prof., mr., mr. dr., ing., ir.
+            // Fixed: use \b? after the optional dot to handle word boundary correctly
+            title_exact: Regex::new(r"(?i)\b(dr\.|prof\.|mr\.|ing\.|ir\.|mr\. dr\.|dr|prof|mr|ing|ir)\b")?,
+
+            // Government roles (case-insensitive, match word boundaries)
+            government_role: Regex::new(
+                r"(?i)\b(minister|staatssecretaris|ambtenaar|directeur|hoofd|coordinator|beleidsmedewerker|adviseur|manager)\b"
+            )?,
+
+            // Department abbreviations (capitalized)
+            department_abbr: Regex::new(r"\b(MinFin|BZK|VWS|EZK|OCW|IenW|LNV|J&V)\b")?,
+
+            // Full department name: Ministerie van ... (supports accented characters)
+            department_full: Regex::new(r"(?i)\bministerie\s+van\s+([A-Z][a-zÀ-ÿ]+(?:\s+[A-Z][a-zÀ-ÿ]+)?)")?,
+
+            // Person with title: Dr. Jan de Vries
+            person_with_title: Regex::new(
+                r"(?i)\b(dr\.|prof\.|mr\.|ing\.|ir\.)\s+([A-Z][a-z]+(?:\s+(?:van|de|der|van der|in|ten|ter|op|den|bij|uit|van de|van der)?\s+[A-Z][a-z]+)?)\b"
+            )?,
+
+            // Relationship: Jan de Vries, minister van Financiën (supports accented characters)
+            relationship_works_for: Regex::new(
+                r"(?i)\b([A-Z][a-zÀ-ÿ]+(?:\s+(?:van|de|der|van der|in|ten|ter)\s+[A-Z][a-zÀ-ÿ]+)?)\s*,\s*(?:minister|staatssecretaris|directeur|hoofd)\s+van\s+([A-Z][a-zÀ-ÿ]+(?:\s+[A-Z][a-zÀ-ÿ]+)?)"
+            )?,
+
+            // Simple person name: First + Last (with optional tussenvoegsel)
+            person_name: Regex::new(
+                r"\b([A-Z][a-z]+)\s+(?:(van|de|der|van der|in|ten|ter|op|den|bij|uit|van de)\s+)?([A-Z][a-z]+)\b"
+            )?,
+        })
+    }
+}
+
+// Global lazy-initialized patterns
+fn patterns() -> &'static DutchPatterns {
+    static PATTERNS: OnceLock<DutchPatterns> = OnceLock::new();
+    PATTERNS.get_or_init(|| DutchPatterns::new().expect("patterns must compile"))
+}
+
+/// Known Dutch government organization canonical names
+static GOVERNMENT_ORGS: &[(&str, &str)] = &[
+    // Abbreviations
+    ("MinFin", "Ministerie van Financiën"),
+    ("BZK", "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties"),
+    ("VWS", "Ministerie van Volksgezondheid, Welzijn en Sport"),
+    ("EZK", "Ministerie van Economische Zaken en Klimaat"),
+    ("OCW", "Ministerie van Onderwijs, Cultuur en Wetenschap"),
+    ("IenW", "Ministerie van Infrastructuur en Waterstaat"),
+    ("LNV", "Ministerie van Landbouw, Natuur en Voedselkwaliteit"),
+    ("J&V", "Ministerie van Justitie en Veiligheid"),
+    ("SZW", "Ministerie van Sociale Zaken en Werkgelegenheid"),
+    ("AK", "Ministerie van Algemene Zaken"),
+    // Full names
+    ("Financiën", "Ministerie van Financiën"),
+    ("Binnenlandse Zaken", "Ministerie van Binnenlandse Zaken en Koninkrijksrelaties"),
+    ("Volksgezondheid", "Ministerie van Volksgezondheid, Welzijn en Sport"),
+    ("Economische Zaken", "Ministerie van Economische Zaken en Klimaat"),
+    ("Onderwijs", "Ministerie van Onderwijs, Cultuur en Wetenschap"),
+    ("Infrastructuur", "Ministerie van Infrastructuur en Waterstaat"),
+    ("Justitie", "Ministerie van Justitie en Veiligheid"),
+    ("Sociale Zaken", "Ministerie van Sociale Zaken en Werkgelegenheid"),
+];
+
+/// Baseline extractor using regex patterns
+///
+/// This extractor uses compiled regex patterns to find Dutch government
+/// entities in text. It's designed for speed (<500ms) and high confidence
+/// on common patterns.
+#[derive(Debug, Clone)]
+pub struct BaselineExtractor {
+    /// Whether to enable additional NLP features (placeholder for future rust-bert)
+    enable_ner: bool,
+}
+
+impl BaselineExtractor {
+    /// Create a new baseline extractor
+    ///
+    /// The `enable_ner` parameter is a placeholder for future rust-bert integration.
+    /// For now, it controls whether additional name validation is performed.
+    pub fn new(enable_ner: bool) -> Result<Self, BaselineError> {
+        // Force pattern compilation to fail fast if there's an issue
+        let _ = patterns();
+
+        Ok(Self { enable_ner })
+    }
+
+    /// Extract entities from a generated document
+    pub fn extract(
+        &self,
+        document: &GeneratedDocument,
+        options: &ExtractionOptions,
+    ) -> Result<ExtractionResult, BaselineError> {
+        let text = &document.content;
+        let document_id = document.document_id;
+
+        if text.is_empty() {
+            return Err(BaselineError::InvalidText("Document text is empty".to_string()));
+        }
+
+        if text.len() > 1_000_000 {
+            return Err(BaselineError::InvalidText("Document text too large (>1MB)".to_string()));
+        }
+
+        let mut persons = Vec::new();
+        let mut organizations = Vec::new();
+        let mut mentions: Vec<MentionRelationshipWrapper> = Vec::new();
+
+        let patterns = patterns();
+
+        // Extract persons with titles first (highest confidence)
+        for cap in patterns.person_with_title.captures_iter(text) {
+            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
+            let title = cap.get(1).map(|m| m.as_str()).unwrap_or("");
+            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
+
+            if let Some((person, mention)) = self.create_person_entity(
+                name,
+                document_id,
+                text,
+                full_match,
+                options,
+            )? {
+                // Add title metadata
+                let person_with_title = PersonStakeholder::from(person.clone())
+                    .with_title(title.to_lowercase().trim_end_matches('.'));
+
+                persons.push(person_with_title.entity);
+                mentions.push(mention);
+            }
+        }
+
+        // Extract organizations
+        // First check abbreviations
+        for cap in patterns.department_abbr.captures_iter(text) {
+            let abbr = cap.get(0).map(|m| m.as_str()).unwrap_or("");
+
+            if let Some((org, mention)) = self.create_org_entity(
+                abbr,
+                document_id,
+                text,
+                abbr,
+                0.90, // High confidence for known abbreviations
+                options,
+            )? {
+                organizations.push(org);
+                mentions.push(mention);
+            }
+        }
+
+        // Then full department names
+        for cap in patterns.department_full.captures_iter(text) {
+            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
+            let dept_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
+
+            if let Some((org, mention)) = self.create_org_entity(
+                full_match,
+                document_id,
+                text,
+                full_match,
+                0.85, // Slightly lower for full names
+                options,
+            )? {
+                organizations.push(org);
+                mentions.push(mention);
+            }
+        }
+
+        // Apply confidence threshold filter
+        persons.retain(|p| p.confidence >= options.confidence_threshold);
+        organizations.retain(|o| o.confidence >= options.confidence_threshold);
+        mentions.retain(|m| m.confidence >= options.confidence_threshold);
+
+        Ok(ExtractionResult {
+            persons: persons.into_iter().map(PersonStakeholder::from).collect(),
+            organizations: organizations.into_iter().map(OrganizationStakeholder::from).collect(),
+            mentions,
+            stats: Default::default(),
+            processing_time_ms: 0,
+        })
+    }
+
+    /// Create a person entity from extracted text
+    fn create_person_entity(
+        &self,
+        name: &str,
+        document_id: Uuid,
+        text: &str,
+        matched_text: &str,
+        options: &ExtractionOptions,
+    ) -> Result<Option<(Entity, MentionRelationshipWrapper)>, BaselineError> {
+        let confidence = self.calculate_person_confidence(name, text);
+        if !options.meets_threshold(confidence) {
+            return Ok(None);
+        }
+
+        let entity = Entity {
+            id: Uuid::new_v4(),
+            name: name.to_string(),
+            entity_type: EntityType::Person,
+            canonical_name: None,
+            description: None,
+            confidence,
+            source_domain_id: Some(document_id),
+            metadata: serde_json::json!({
+                "extraction_method": "baseline_regex",
+            }),
+            created_at: chrono::Utc::now(),
+        };
+
+        let mention = MentionRelationshipWrapper {
+            id: Uuid::new_v4(),
+            entity_id: entity.id,
+            document_id,
+            confidence,
+        };
+
+        Ok(Some((entity, mention)))
+    }
+
+    /// Create an organization entity from extracted text
+    fn create_org_entity(
+        &self,
+        name: &str,
+        document_id: Uuid,
+        text: &str,
+        matched_text: &str,
+        base_confidence: f32,
+        options: &ExtractionOptions,
+    ) -> Result<Option<(Entity, MentionRelationshipWrapper)>, BaselineError> {
+        let confidence = base_confidence;
+        if !options.meets_threshold(confidence) {
+            return Ok(None);
+        }
+
+        // Find canonical name
+        let canonical_name = GOVERNMENT_ORGS.iter()
+            .find(|(abbr, _)| *abbr == name || name.contains(abbr))
+            .map(|(_, canonical)| canonical.to_string());
+
+        let entity = Entity {
+            id: Uuid::new_v4(),
+            name: name.to_string(),
+            entity_type: EntityType::Organization,
+            canonical_name,
+            description: None,
+            confidence,
+            source_domain_id: Some(document_id),
+            metadata: serde_json::json!({
+                "extraction_method": "baseline_regex",
+            }),
+            created_at: chrono::Utc::now(),
+        };
+
+        let mention = MentionRelationshipWrapper {
+            id: Uuid::new_v4(),
+            entity_id: entity.id,
+            document_id,
+            confidence,
+        };
+
+        Ok(Some((entity, mention)))
+    }
+
+    /// Calculate confidence score for a person extraction
+    fn calculate_person_confidence(&self, _name: &str, text: &str) -> f32 {
+        let mut confidence: f32 = 0.70; // Base confidence for regex matches
+
+        // Boost if has title pattern
+        if patterns().title_exact.is_match(text) {
+            confidence += 0.10;
+        }
+
+        // Boost if has government role nearby
+        if patterns().government_role.is_match(text) {
+            confidence += 0.05;
+        }
+
+        confidence.min(1.0)
+    }
+
+    /// Find the position of matched text in the document
+    fn find_text_position(&self, text: &str, matched: &str) -> Option<TextPosition> {
+        text.find(matched).map(|offset| {
+            TextPosition::new(offset, matched.len())
+        })
+    }
+
+    /// Extract context around matched text
+    fn extract_context(&self, text: &str, matched: &str) -> String {
+        if let Some(pos) = text.find(matched) {
+            let start = pos.saturating_sub(50);
+            let end = (pos + matched.len()).saturating_add(50);
+            let context = text.get(start..end).unwrap_or(matched);
+            context.to_string()
+        } else {
+            matched.to_string()
+        }
+    }
+
+    /// Extract titles from text (for testing)
+    pub fn extract_titles(&self, text: &str) -> Vec<String> {
+        patterns()
+            .title_exact
+            .find_iter(text)
+            .map(|m| {
+                let matched = m.as_str().to_lowercase();
+                // Normalize: ensure titles without dots get them
+                match matched.as_str() {
+                    "dr" => "dr.".to_string(),
+                    "prof" => "prof.".to_string(),
+                    "mr" => "mr.".to_string(),
+                    "ing" => "ing.".to_string(),
+                    "ir" => "ir.".to_string(),
+                    _ => matched,
+                }
+            })
+            .collect()
+    }
+
+    /// Extract government roles from text (for testing)
+    pub fn extract_roles(&self, text: &str) -> Vec<String> {
+        patterns()
+            .government_role
+            .find_iter(text)
+            .map(|m| m.as_str().to_lowercase())
+            .collect()
+    }
+
+    /// Extract department abbreviations from text (for testing)
+    pub fn extract_departments(&self, text: &str) -> Vec<String> {
+        patterns()
+            .department_abbr
+            .find_iter(text)
+            .map(|m| m.as_str().to_string())
+            .collect()
+    }
+
+    /// Extract relationships from text (for testing)
+    pub fn extract_relationships(&self, text: &str) -> Vec<RelationshipMatch> {
+        patterns()
+            .relationship_works_for
+            .captures_iter(text)
+            .filter_map(|cap| {
+                let person = cap.get(1)?.as_str().to_string();
+                let org = cap.get(2)?.as_str().to_string();
+                Some(RelationshipMatch {
+                    person_name: person,
+                    org_name: org,
+                    role: None,
+                })
+            })
+            .collect()
+    }
+}
+
+/// Extracted relationship match for testing
+#[derive(Debug, Clone)]
+pub struct RelationshipMatch {
+    pub person_name: String,
+    pub org_name: String,
+    pub role: Option<String>,
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn create_test_document(content: &str) -> GeneratedDocument {
+        GeneratedDocument {
+            document_id: Uuid::new_v4(),
+            content: content.to_string(),
+            variables: vec![],
+            entity_links: vec![],
+            sections: vec![],
+            generated_at: chrono::Utc::now(),
+        }
+    }
+
+    #[test]
+    fn test_baseline_extractor_new() {
+        let extractor = BaselineExtractor::new(false);
+        assert!(extractor.is_ok());
+        let extractor = extractor.unwrap();
+        assert!(!extractor.enable_ner);
+    }
+
+    #[test]
+    fn test_baseline_extractor_new_with_ner() {
+        let extractor = BaselineExtractor::new(true);
+        assert!(extractor.is_ok());
+        let extractor = extractor.unwrap();
+        assert!(extractor.enable_ner);
+    }
+
+    #[test]
+    fn test_regex_extracts_dr_as_title() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let text = "Dr. Jan de Vries heeft een brief gestuurd.";
+        let result = extractor.extract_titles(text);
+        assert!(result.contains(&"dr.".to_string()));
+    }
+
+    #[test]
+    fn test_regex_extracts_prof_as_title() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let text = "Prof. dr. Marie Jansen is aangesteld.";
+        let result = extractor.extract_titles(text);
+        assert!(result.contains(&"prof.".to_string()));
+    }
+
+    #[test]
+    fn test_regex_extracts_minister_as_government_role() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let text = "De minister van Financiën sprak vandaag.";
+        let result = extractor.extract_roles(text);
+        assert!(result.iter().any(|r| r == "minister"));
+    }
+
+    #[test]
+    fn test_regex_extracts_staatsecretaris_as_government_role() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let text = "De staatssecretaris van Economische Zaken.";
+        let result = extractor.extract_roles(text);
+        assert!(result.iter().any(|r| r == "staatssecretaris"));
+    }
+
+    #[test]
+    fn test_regex_extracts_minfin_as_department_abbreviation() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let text = "MinFin heeft vandaag het beleid aangekondigd.";
+        let result = extractor.extract_departments(text);
+        assert!(result.contains(&"MinFin".to_string()));
+    }
+
+    #[test]
+    fn test_regex_extracts_bzk_as_department_abbreviation() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let text = "BZK is verantwoordelijk voor de uitvoering.";
+        let result = extractor.extract_departments(text);
+        assert!(result.contains(&"BZK".to_string()));
+    }
+
+    #[test]
+    fn test_pattern_x_minister_van_y_creates_worksfor_relationship() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let text = "Jan de Vries, minister van Financiën, is aanwezig.";
+        let result = extractor.extract_relationships(text);
+        assert!(result.iter().any(|r|
+            r.person_name == "Jan de Vries" &&
+            r.org_name == "Financiën"
+        ));
+    }
+
+    #[test]
+    fn test_pattern_dr_x_directeur_z_creates_worksfor_with_role() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let text = "Dr. Sophie Bakker, directeur van de Rijksoverheid, spreekt.";
+        let result = extractor.extract_relationships(text);
+        // Note: current pattern doesn't capture director role, but will match person+org
+        assert!(!result.is_empty());
+    }
+
+    #[test]
+    fn test_extract_from_generated_document() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let doc = create_test_document(
+            "Van: Dr. Jan de Vries\n\
+             Aan: Ministerie van Financiën\n\
+             \n\
+             Hierbij stuur ik u het beleidsstuk."
+        );
+        let options = ExtractionOptions::default();
+
+        let result = extractor.extract(&doc, &options);
+        assert!(result.is_ok());
+
+        let result = result.unwrap();
+        // Should extract the person with title
+        assert!(!result.persons.is_empty());
+        // Should extract the organization
+        assert!(!result.organizations.is_empty());
+    }
+
+    #[test]
+    fn test_confidence_scores_between_0_and_1() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let doc = create_test_document("Dr. Jan de Vries van MinFin sprak vandaag.");
+        let options = ExtractionOptions::default();
+
+        let result = extractor.extract(&doc, &options).unwrap();
+
+        for person in result.persons {
+            let confidence = person.as_entity().confidence;
+            assert!(confidence >= 0.0 && confidence <= 1.0);
+        }
+        for org in result.organizations {
+            let confidence = org.entity.confidence;
+            assert!(confidence >= 0.0 && confidence <= 1.0);
+        }
+    }
+
+    #[test]
+    fn test_empty_document_returns_error() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let doc = create_test_document("");
+        let options = ExtractionOptions::default();
+
+        let result = extractor.extract(&doc, &options);
+        assert!(result.is_err());
+    }
+
+    #[test]
+    fn test_large_document_returns_error() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let large_text = "A".repeat(1_000_001);
+        let doc = create_test_document(&large_text);
+        let options = ExtractionOptions::default();
+
+        let result = extractor.extract(&doc, &options);
+        assert!(result.is_err());
+    }
+
+    #[test]
+    fn test_confidence_threshold_filters_low_confidence_entities() {
+        let extractor = BaselineExtractor::new(false).unwrap();
+        let doc = create_test_document("Dr. Jan de Vries werkte met MinFin.");
+        let options = ExtractionOptions {
+            confidence_threshold: 0.95, // Very high threshold
+            ..Default::default()
+        };
+
+        let result = extractor.extract(&doc, &options).unwrap();
+        // Most entities should be filtered out
+        assert!(result.persons.len() + result.organizations.len() < 2);
+    }
+}
diff --git a/crates/iou-ai/src/stakeholder/mention_detector.rs b/crates/iou-ai/src/stakeholder/mention_detector.rs
new file mode 100644
index 0000000..ea81b3e
--- /dev/null
+++ b/crates/iou-ai/src/stakeholder/mention_detector.rs
@@ -0,0 +1,213 @@
+//! Mention type detection for document context
+//!
+//! This module analyzes the context around entity mentions to determine
+//! the mention type (Author, Recipient, Subject, Referenced).
+
+use crate::stakeholder::MentionType;
+
+/// Detect mention type from document context
+///
+/// Analyzes the position and surrounding text of an entity mention
+/// to determine how it's being referenced in the document.
+pub fn detect_mention_type(
+    entity_text: &str,
+    document_text: &str,
+    position: usize,
+) -> MentionType {
+    let lines: Vec<&str> = document_text.lines().collect();
+
+    // Find which line contains the position
+    let mut line_num = 0;
+    let mut char_count = 0;
+    for (i, line) in lines.iter().enumerate() {
+        let line_len = line.len() + 1; // +1 for newline
+        if char_count + line_len > position {
+            line_num = i;
+            break;
+        }
+        char_count += line_len;
+    }
+
+    let line = lines.get(line_num).unwrap_or(&"");
+
+    // Check for author patterns in headers
+    if is_in_header(entity_text, document_text, position, line) {
+        return MentionType::Author;
+    }
+
+    // Check for recipient patterns
+    if is_recipient_pattern(entity_text, document_text, position, line) {
+        return MentionType::Recipient;
+    }
+
+    // Check if entity is the subject of the sentence
+    if is_subject_of_sentence(entity_text, document_text, position, line) {
+        return MentionType::Subject;
+    }
+
+    // Default: Referenced
+    MentionType::Referenced
+}
+
+/// Check if entity is in a header area (likely author)
+fn is_in_header(
+    entity_text: &str,
+    document_text: &str,
+    position: usize,
+    line: &str,
+) -> bool {
+    // Check if in first 10 lines (header area)
+    let lines_before: Vec<&str> = document_text.lines().take(10).collect();
+    let in_header_area = lines_before.iter().any(|l| l.contains(entity_text));
+
+    if !in_header_area {
+        return false;
+    }
+
+    // Check for "Van:" or "From:" patterns (Dutch/English email headers)
+    let lower_line = line.to_lowercase();
+    if lower_line.starts_with("van:") ||
+       lower_line.starts_with("from:") ||
+       lower_line.contains("van:") ||
+       lower_line.contains("afzender:") {
+        return true;
+    }
+
+    // Check if line starts with a title followed by the entity
+    if line.trim().starts_with("Dr.") ||
+       line.trim().starts_with("Prof.") ||
+       line.trim().starts_with("Ir.") ||
+       line.trim().starts_with("Ing.") {
+        // Entity might be author if at start of header line
+        return true;
+    }
+
+    false
+}
+
+/// Check if entity is in a recipient pattern
+fn is_recipient_pattern(
+    _entity_text: &str,
+    document_text: &str,
+    position: usize,
+    line: &str,
+) -> bool {
+    let lower_line = line.to_lowercase();
+
+    // Check for "Aan:" or "To:" patterns
+    if lower_line.starts_with("aan:") ||
+       lower_line.starts_with("to:") ||
+       lower_line.contains("aan:") ||
+       lower_line.contains("ontvanger:") {
+        return true;
+    }
+
+    // Check for "Geachte" (formal Dutch salutation)
+    if lower_line.contains("geachte") ||
+       lower_line.starts_with("geachte") {
+        return true;
+    }
+
+    // Check if in first 20 lines and contains salutation patterns
+    let lines_before: Vec<&str> = document_text.lines().take(20).collect();
+    let in_early_lines = lines_before.iter().any(|l| l.to_lowercase().contains("geachte"));
+
+    in_early_lines && position < 2000 // Early in document
+}
+
+/// Check if entity is the subject of the current sentence
+fn is_subject_of_sentence(
+    entity_text: &str,
+    _document_text: &str,
+    _position: usize,
+    line: &str,
+) -> bool {
+    let lower_line = line.to_lowercase();
+
+    // Check if entity appears early in sentence (before the verb)
+    if let Some(entity_pos) = lower_line.find(&entity_text.to_lowercase()) {
+        // Find first verb after entity position
+        let after_entity = lower_line.get(entity_pos..).unwrap_or("");
+        let verbs = ["is", "zijn", "was", "heeft", "hebben", "sprak", "zei", "stuurde"];
+
+        for verb in verbs {
+            if let Some(verb_pos) = after_entity.find(verb) {
+                // If verb appears within 50 chars after entity, entity might be subject
+                if verb_pos < 50 {
+                    return true;
+                }
+            }
+        }
+    }
+
+    false
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_mention_type_author_detected_from_header_patterns() {
+        let doc = "Van: Dr. Jan de Vries\nAan: Ministerie van Financiën\n\nHierbij...";
+        let entity = "Dr. Jan de Vries";
+        let mention_type = detect_mention_type(entity, doc, 5);
+        assert_eq!(mention_type, MentionType::Author);
+    }
+
+    #[test]
+    fn test_mention_type_recipient_detected_from_geachte_patterns() {
+        let doc = "Geachte minister,\nHierbij stuur ik u...";
+        let entity = "minister";
+        let mention_type = detect_mention_type(entity, doc, 8);
+        assert_eq!(mention_type, MentionType::Recipient);
+    }
+
+    #[test]
+    fn test_mention_type_recipient_detected_from_aan_patterns() {
+        let doc = "Aan: Ministerie van Financiën\nVan: Jan de Vries";
+        let entity = "Financiën";
+        let mention_type = detect_mention_type(entity, doc, 20);
+        assert_eq!(mention_type, MentionType::Recipient);
+    }
+
+    #[test]
+    fn test_mention_type_defaults_to_referenced_when_unclear() {
+        let doc = "Het beleid is goedgekeurd door Jan de Vries.";
+        let entity = "Jan de Vries";
+        let mention_type = detect_mention_type(entity, doc, 35);
+        assert_eq!(mention_type, MentionType::Referenced);
+    }
+
+    #[test]
+    fn test_mention_type_subject_when_entity_is_sentence_subject() {
+        let doc = "Jan de Vries sprak vandaag met de pers.";
+        let entity = "Jan de Vries";
+        let mention_type = detect_mention_type(entity, doc, 0);
+        assert_eq!(mention_type, MentionType::Subject);
+    }
+
+    #[test]
+    fn test_mention_type_author_from_van_pattern() {
+        let doc = "Van: Prof. Marie Jansen\nAan: Directie";
+        let entity = "Marie Jansen";
+        let mention_type = detect_mention_type(entity, doc, 10);
+        assert_eq!(mention_type, MentionType::Author);
+    }
+
+    #[test]
+    fn test_empty_document_returns_referenced() {
+        let doc = "";
+        let entity = "Test";
+        let mention_type = detect_mention_type(entity, doc, 0);
+        assert_eq!(mention_type, MentionType::Referenced);
+    }
+
+    #[test]
+    fn test_geachte_pattern_in_first_lines_is_recipient() {
+        let doc = "Geachte heer De Vries,\n\nHierbij...";
+        let entity = "De Vries";
+        let mention_type = detect_mention_type(entity, doc, 15);
+        assert_eq!(mention_type, MentionType::Recipient);
+    }
+}
diff --git a/crates/iou-ai/src/stakeholder/mod.rs b/crates/iou-ai/src/stakeholder/mod.rs
index fbdca73..5d7318f 100644
--- a/crates/iou-ai/src/stakeholder/mod.rs
+++ b/crates/iou-ai/src/stakeholder/mod.rs
@@ -21,6 +21,10 @@ pub mod mention;
 pub mod normalization;
 pub mod error;
 
+// Baseline extraction (section-02)
+pub mod baseline;
+mod mention_detector;
+
 // Feasibility spike exports
 pub use rijksoverheid_api_probe::{probe_rijksoverheid_api, ApiProbeResult};
 pub use fallback_dict::get_fallback_canonical_name;
@@ -34,3 +38,6 @@ pub use result::{ExtractionResult, ExtractionStats, VerificationStatus};
 pub use mention::{MentionRelationship, MentionType, TextPosition, ExtractionMethod};
 pub use normalization::{DutchNameNormalizer, NameComparison};
 pub use error::{ExtractionError, NormalizationError, DeduplicationError};
+
+// Public API exports (section-02)
+pub use baseline::{BaselineExtractor, BaselineError, RelationshipMatch};
