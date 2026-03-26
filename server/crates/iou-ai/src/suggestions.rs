//! Metadata suggestions for documents
//!
//! Combines NER, compliance assessment, and pattern matching
//! to generate metadata suggestions for information objects.

use uuid::Uuid;

use iou_core::api_types::{MetadataSuggestion, SuggestionSource};
use iou_core::domain::DomainType;
use iou_core::objects::ObjectType;

use crate::compliance::ComplianceAssessor;
use crate::ner::DutchNerExtractor;

/// Metadata suggester combining multiple AI techniques
pub struct MetadataSuggester {
    ner: DutchNerExtractor,
    compliance: ComplianceAssessor,
}

impl MetadataSuggester {
    pub fn new() -> Self {
        Self {
            ner: DutchNerExtractor::new(),
            compliance: ComplianceAssessor::new(),
        }
    }

    /// Generate metadata suggestions for a document
    pub fn suggest_metadata(
        &self,
        content: &str,
        object_type: ObjectType,
        domain_type: Option<DomainType>,
    ) -> Vec<MetadataSuggestion> {
        let mut suggestions = Vec::new();

        // Extract entities and suggest tags
        let entities = self.ner.extract_entities(content);
        for entity in entities.iter().take(5) {
            // Top 5 entities as tags
            suggestions.push(MetadataSuggestion {
                id: Uuid::new_v4(),
                field: "tags".to_string(),
                suggested_value: serde_json::json!(entity.name),
                confidence: entity.confidence,
                reasoning: format!(
                    "Entiteit type {} gedetecteerd met {} confidence",
                    entity.entity_type, entity.confidence
                ),
                source: SuggestionSource::Ner,
            });
        }

        // Suggest classification based on Woo assessment
        let woo =
            self.compliance
                .assess_woo_relevance(content, object_type, iou_core::compliance::Classification::Intern);

        if woo.is_relevant {
            suggestions.push(MetadataSuggestion {
                id: Uuid::new_v4(),
                field: "is_woo_relevant".to_string(),
                suggested_value: serde_json::json!(true),
                confidence: woo.confidence,
                reasoning: woo.reasoning.clone(),
                source: SuggestionSource::Classification,
            });

            suggestions.push(MetadataSuggestion {
                id: Uuid::new_v4(),
                field: "classification".to_string(),
                suggested_value: serde_json::json!(format!("{:?}", woo.suggested_class).to_lowercase()),
                confidence: woo.confidence * 0.9,
                reasoning: "Op basis van Woo-relevantie beoordeling".to_string(),
                source: SuggestionSource::Classification,
            });
        }

        // Suggest retention period
        if let Some(dt) = domain_type {
            let retention = self.compliance.calculate_retention(dt, object_type);
            suggestions.push(MetadataSuggestion {
                id: Uuid::new_v4(),
                field: "retention_period".to_string(),
                suggested_value: serde_json::json!(retention.retention_years),
                confidence: 0.95,
                reasoning: format!(
                    "Standaard bewaartermijn voor {} in {}",
                    object_type, dt
                ),
                source: SuggestionSource::RuleBased,
            });
        }

        // Suggest privacy level
        let privacy = self.compliance.assess_privacy_level(content);
        if privacy != iou_core::compliance::PrivacyLevel::Geen {
            suggestions.push(MetadataSuggestion {
                id: Uuid::new_v4(),
                field: "privacy_level".to_string(),
                suggested_value: serde_json::json!(format!("{:?}", privacy).to_lowercase()),
                confidence: 0.85,
                reasoning: format!("Mogelijk {:?} persoonsgegevens gedetecteerd", privacy),
                source: SuggestionSource::PatternMatching,
            });
        }

        // Detect subject area from policy terms
        let policy_entities: Vec<_> = entities
            .iter()
            .filter(|e| e.entity_type == iou_core::graphrag::EntityType::Policy)
            .collect();

        if !policy_entities.is_empty() {
            let subject_area = policy_entities[0].name.clone();
            suggestions.push(MetadataSuggestion {
                id: Uuid::new_v4(),
                field: "subject_area".to_string(),
                suggested_value: serde_json::json!(subject_area),
                confidence: policy_entities[0].confidence,
                reasoning: "Beleidsterm gedetecteerd in tekst".to_string(),
                source: SuggestionSource::Ner,
            });
        }

        // Sort by confidence
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        suggestions
    }

    /// Suggest related domains based on extracted entities
    pub fn suggest_related_domains(
        &self,
        content: &str,
        _current_domain_id: Uuid,
    ) -> Vec<MetadataSuggestion> {
        let entities = self.ner.extract_entities(content);
        let mut suggestions = Vec::new();

        // In a real implementation, we would query the database
        // for domains that share these entities
        // For now, just return entity-based suggestions

        for entity in entities.iter().take(3) {
            suggestions.push(MetadataSuggestion {
                id: Uuid::new_v4(),
                field: "related_domain".to_string(),
                suggested_value: serde_json::json!({
                    "entity": entity.name,
                    "entity_type": format!("{:?}", entity.entity_type),
                    "reason": "Gedeelde entiteit"
                }),
                confidence: entity.confidence * 0.7,
                reasoning: format!(
                    "Domein deelt mogelijk entiteit '{}' met andere domeinen",
                    entity.name
                ),
                source: SuggestionSource::SemanticSimilarity,
            });
        }

        suggestions
    }
}

impl Default for MetadataSuggester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_metadata() {
        let suggester = MetadataSuggester::new();
        let content = r#"
            De Provincie Flevoland heeft besloten tot het verlenen van een
            subsidie voor het project windpark Almere onder de Omgevingswet.
            Het besluit is openbaar op grond van de Wet open overheid.
        "#;

        let suggestions =
            suggester.suggest_metadata(content, ObjectType::Besluit, Some(DomainType::Zaak));

        assert!(!suggestions.is_empty());

        // Should suggest Woo relevance
        let woo_suggestion = suggestions
            .iter()
            .find(|s| s.field == "is_woo_relevant");
        assert!(woo_suggestion.is_some());

        // Should suggest tags
        let tag_suggestions: Vec<_> = suggestions.iter().filter(|s| s.field == "tags").collect();
        assert!(!tag_suggestions.is_empty());

        // Should suggest retention period
        let retention_suggestion = suggestions
            .iter()
            .find(|s| s.field == "retention_period");
        assert!(retention_suggestion.is_some());
    }

    #[test]
    fn test_privacy_suggestion() {
        let suggester = MetadataSuggester::new();
        let content = "Document bevat BSN 123456789 van de aanvrager.";

        let suggestions =
            suggester.suggest_metadata(content, ObjectType::Document, Some(DomainType::Zaak));

        let privacy_suggestion = suggestions
            .iter()
            .find(|s| s.field == "privacy_level");
        assert!(privacy_suggestion.is_some());
    }
}
