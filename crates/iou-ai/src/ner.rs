//! Named Entity Recognition voor Nederlandse overheidstekst
//!
//! Regex-based NER die specifiek is voor Nederlandse overheidscontext.
//! Dit is een pragmatische oplossing die goed werkt voor:
//! - Bekende organisaties (provincies, gemeentes, ministeries)
//! - Nederlandse wetten en regelgeving
//! - Locaties in Nederland
//! - Datums en bedragen

use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;

use iou_core::graphrag::{Entity, EntityType};

lazy_static! {
    // Nederlandse overheidsorganisaties
    static ref RE_PROVINCIE: Regex = Regex::new(
        r"(?i)\b(Provincie\s+)?(Flevoland|Noord-Holland|Zuid-Holland|Utrecht|Gelderland|Overijssel|Drenthe|Groningen|Friesland|Zeeland|Noord-Brabant|Limburg)\b"
    ).unwrap();

    static ref RE_GEMEENTE: Regex = Regex::new(
        r"(?i)\b(Gemeente\s+)?(Almere|Amsterdam|Rotterdam|Den Haag|Utrecht|Eindhoven|Groningen|Tilburg|Almelo|Lelystad|Dronten|Zeewolde|Urk|Noordoostpolder)\b"
    ).unwrap();

    static ref RE_MINISTERIE: Regex = Regex::new(
        r"(?i)\b(Ministerie\s+van\s+)(Binnenlandse Zaken|BZK|Financiën|Infrastructuur en Waterstaat|I&W|Economische Zaken|EZK|Justitie en Veiligheid|J&V|Onderwijs|OCW|Volksgezondheid|VWS|Sociale Zaken|SZW|Buitenlandse Zaken|BZ|Defensie|Landbouw|LNV)\b"
    ).unwrap();

    // Nederlandse wetten
    static ref RE_WET: Regex = Regex::new(
        r"(?i)\b(Wet\s+open\s+overheid|Woo|WOO|Algemene\s+verordening\s+gegevensbescherming|AVG|GDPR|Archiefwet|Omgevingswet|Algemene\s+wet\s+bestuursrecht|Awb|Wet\s+openbaarheid\s+van\s+bestuur|Wob|Gemeentewet|Provinciewet|Waterschapswet)\b"
    ).unwrap();

    // Artikelverwijzingen
    static ref RE_ARTIKEL: Regex = Regex::new(
        r"(?i)\b(artikel|art\.?)\s*(\d+(?:\.\d+)?(?:\s*(?:lid|sub)\s*\d+)?)\b"
    ).unwrap();

    // Datums (Nederlandse notatie)
    static ref RE_DATUM: Regex = Regex::new(
        r"\b(\d{1,2}[-/]\d{1,2}[-/]\d{2,4}|\d{1,2}\s+(?:januari|februari|maart|april|mei|juni|juli|augustus|september|oktober|november|december)\s+\d{4})\b"
    ).unwrap();

    // Geldbedragen
    static ref RE_GELD: Regex = Regex::new(
        r"(?i)\b(€\s*)?(\d{1,3}(?:\.\d{3})*(?:,\d{2})?)\s*(euro|EUR)?\b"
    ).unwrap();

    // Zaaknummers en referenties
    static ref RE_ZAAKNUMMER: Regex = Regex::new(
        r"\b([A-Z]{1,5}[-/]?\d{4}[-/]\d{3,6}|Z[-/]?\d{4}[-/]\d{3,6})\b"
    ).unwrap();

    // Beleidstermen
    static ref RE_BELEID: Regex = Regex::new(
        r"(?i)\b(mobiliteit|duurzaamheid|energietransitie|circulaire economie|klimaatadaptatie|woningbouw|stikstof|biodiversiteit|ruimtelijke ordening|omgevingsvisie)\b"
    ).unwrap();
}

/// Dutch NER extractor using pattern matching
pub struct DutchNerExtractor;

impl DutchNerExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract all entities from text
    pub fn extract_entities(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let now = chrono::Utc::now();

        // Extract organizations (provinces, municipalities, ministries)
        for cap in RE_PROVINCIE.captures_iter(text) {
            entities.push(Entity {
                id: Uuid::new_v4(),
                name: cap.get(0).unwrap().as_str().to_string(),
                entity_type: EntityType::Organization,
                canonical_name: Some(format!("Provincie {}", cap.get(2).map(|m| m.as_str()).unwrap_or(""))),
                description: Some("Nederlandse provincie".to_string()),
                confidence: 0.95,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: now,
            });
        }

        for cap in RE_GEMEENTE.captures_iter(text) {
            entities.push(Entity {
                id: Uuid::new_v4(),
                name: cap.get(0).unwrap().as_str().to_string(),
                entity_type: EntityType::Organization,
                canonical_name: Some(format!("Gemeente {}", cap.get(2).map(|m| m.as_str()).unwrap_or(""))),
                description: Some("Nederlandse gemeente".to_string()),
                confidence: 0.93,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: now,
            });
        }

        for cap in RE_MINISTERIE.captures_iter(text) {
            entities.push(Entity {
                id: Uuid::new_v4(),
                name: cap.get(0).unwrap().as_str().to_string(),
                entity_type: EntityType::Organization,
                canonical_name: None,
                description: Some("Ministerie van de Rijksoverheid".to_string()),
                confidence: 0.97,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: now,
            });
        }

        // Extract laws
        for cap in RE_WET.captures_iter(text) {
            let name = cap.get(0).unwrap().as_str();
            let canonical = match name.to_lowercase().as_str() {
                s if s.contains("woo") || s.contains("open overheid") => "Wet open overheid (Woo)",
                s if s.contains("avg") || s.contains("gegevensbescherming") => "Algemene verordening gegevensbescherming (AVG)",
                s if s.contains("archiefwet") => "Archiefwet",
                s if s.contains("omgevingswet") => "Omgevingswet",
                s if s.contains("awb") || s.contains("bestuursrecht") => "Algemene wet bestuursrecht (Awb)",
                _ => name,
            };

            entities.push(Entity {
                id: Uuid::new_v4(),
                name: name.to_string(),
                entity_type: EntityType::Law,
                canonical_name: Some(canonical.to_string()),
                description: Some("Nederlandse wet of verordening".to_string()),
                confidence: 0.98,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: now,
            });
        }

        // Extract dates
        for cap in RE_DATUM.captures_iter(text) {
            entities.push(Entity {
                id: Uuid::new_v4(),
                name: cap.get(0).unwrap().as_str().to_string(),
                entity_type: EntityType::Date,
                canonical_name: None,
                description: None,
                confidence: 0.90,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: now,
            });
        }

        // Extract money amounts
        for cap in RE_GELD.captures_iter(text) {
            let full_match = cap.get(0).unwrap().as_str();
            // Only include if it looks like a significant amount
            if let Some(amount) = cap.get(2) {
                let amount_str = amount.as_str().replace(".", "").replace(",", ".");
                if let Ok(value) = amount_str.parse::<f64>() {
                    if value > 100.0 {
                        entities.push(Entity {
                            id: Uuid::new_v4(),
                            name: full_match.to_string(),
                            entity_type: EntityType::Money,
                            canonical_name: Some(format!("€ {:.2}", value)),
                            description: None,
                            confidence: 0.85,
                            source_domain_id: None,
                            metadata: serde_json::Value::Object(serde_json::Map::new()),
                            created_at: now,
                        });
                    }
                }
            }
        }

        // Extract policy terms
        for cap in RE_BELEID.captures_iter(text) {
            entities.push(Entity {
                id: Uuid::new_v4(),
                name: cap.get(0).unwrap().as_str().to_string(),
                entity_type: EntityType::Policy,
                canonical_name: None,
                description: Some("Beleidsterm".to_string()),
                confidence: 0.80,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: now,
            });
        }

        // Deduplicate entities by canonical name
        entities.sort_by(|a, b| {
            a.canonical_name.as_ref().unwrap_or(&a.name)
                .cmp(b.canonical_name.as_ref().unwrap_or(&b.name))
        });
        entities.dedup_by(|a, b| {
            a.canonical_name.as_ref().unwrap_or(&a.name)
                == b.canonical_name.as_ref().unwrap_or(&b.name)
        });

        entities
    }

    /// Extract entities with their positions in text
    pub fn extract_with_positions(&self, text: &str) -> Vec<(Entity, usize, usize)> {
        let mut results = Vec::new();
        let now = chrono::Utc::now();

        // For each regex, capture positions
        for cap in RE_PROVINCIE.captures_iter(text) {
            let m = cap.get(0).unwrap();
            results.push((
                Entity {
                    id: Uuid::new_v4(),
                    name: m.as_str().to_string(),
                    entity_type: EntityType::Organization,
                    canonical_name: None,
                    description: Some("Nederlandse provincie".to_string()),
                    confidence: 0.95,
                    source_domain_id: None,
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                    created_at: now,
                },
                m.start(),
                m.end(),
            ));
        }

        // Add similar for other patterns...

        results
    }
}

impl Default for DutchNerExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_province() {
        let ner = DutchNerExtractor::new();
        let text = "De Provincie Flevoland heeft een nieuw beleid voor windenergie.";
        let entities = ner.extract_entities(text);

        assert!(!entities.is_empty());
        assert!(entities.iter().any(|e| e.name.contains("Flevoland")));
    }

    #[test]
    fn test_extract_law() {
        let ner = DutchNerExtractor::new();
        let text = "Dit besluit valt onder de Wet open overheid (Woo) en de AVG.";
        let entities = ner.extract_entities(text);

        let laws: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Law).collect();
        assert!(laws.len() >= 2); // Woo and AVG
    }

    #[test]
    fn test_extract_municipality() {
        let ner = DutchNerExtractor::new();
        let text = "Gemeente Almere en Gemeente Lelystad werken samen aan dit project.";
        let entities = ner.extract_entities(text);

        let orgs: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Organization).collect();
        assert!(orgs.len() >= 2);
    }

    #[test]
    fn test_extract_policy_terms() {
        let ner = DutchNerExtractor::new();
        let text = "Het beleid richt zich op duurzaamheid en de energietransitie.";
        let entities = ner.extract_entities(text);

        let policies: Vec<_> = entities.iter().filter(|e| e.entity_type == EntityType::Policy).collect();
        assert!(!policies.is_empty());
    }
}
