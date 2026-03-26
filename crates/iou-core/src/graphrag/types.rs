//! GraphRAG types voor kennisgraaf en relaties
//!
//! Dit module ondersteunt de automatische detectie van relaties tussen
//! informatiedomeinen via Named Entity Recognition en graph-analyse.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

/// Entiteit geëxtraheerd uit tekst
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub entity_type: EntityType,
    pub canonical_name: Option<String>,
    pub description: Option<String>,
    pub confidence: f32,
    pub source_domain_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Type entiteit (NER labels)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    /// Persoon
    #[strum(serialize = "PER")]
    Person,
    /// Organisatie
    #[strum(serialize = "ORG")]
    Organization,
    /// Locatie
    #[strum(serialize = "LOC")]
    Location,
    /// Wettelijke verwijzing
    #[strum(serialize = "LAW")]
    Law,
    /// Datum
    #[strum(serialize = "DATE")]
    Date,
    /// Geldbedrag
    #[strum(serialize = "MONEY")]
    Money,
    /// Beleidsterme
    #[strum(serialize = "POLICY")]
    Policy,
    /// Overig
    #[strum(serialize = "MISC")]
    Miscellaneous,
}

/// Relatie tussen twee entiteiten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub source_entity_id: Uuid,
    pub target_entity_id: Uuid,
    pub relationship_type: RelationshipType,
    pub weight: f32,
    pub confidence: f32,
    pub context: Option<String>,
    pub source_domain_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Type relatie tussen entiteiten
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    /// Werkt voor / is onderdeel van
    WorksFor,
    /// Gevestigd in / bevindt zich in
    LocatedIn,
    /// Is onderwerp van (wet/beleid)
    SubjectTo,
    /// Verwijst naar
    RefersTo,
    /// Heeft betrekking op
    RelatesTo,
    /// Is eigenaar van
    OwnerOf,
    /// Rapporteert aan
    ReportsTo,
    /// Werkt samen met
    CollaboratesWith,
    /// Volgt op
    Follows,
    /// Is onderdeel van
    PartOf,
    /// Onbekend
    Unknown,
}

/// Community (cluster) van gerelateerde entiteiten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub level: i32,
    pub parent_community_id: Option<Uuid>,
    pub member_entity_ids: Vec<Uuid>,
    pub summary: Option<String>,
    pub keywords: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Relatie tussen twee informatiedomeinen (afgeleid van entiteiten)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRelation {
    pub id: Uuid,
    pub from_domain_id: Uuid,
    pub to_domain_id: Uuid,
    pub relation_type: DomainRelationType,
    pub strength: f32,
    pub discovery_method: DiscoveryMethod,
    pub shared_entities: Vec<Uuid>,
    pub explanation: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Type relatie tussen domeinen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum DomainRelationType {
    /// Domeinen delen dezelfde entiteiten
    SharedEntities,
    /// Domeinen behoren tot dezelfde community
    SameCommunity,
    /// Domeinen zijn semantisch vergelijkbaar
    SemanticSimilarity,
    /// Domeinen overlappen in tijd
    TemporalOverlap,
    /// Domeinen hebben dezelfde stakeholders
    SharedStakeholders,
    /// Handmatig gelinkt
    ManualLink,
}

/// Methode waarmee relatie is ontdekt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    /// Automatisch via GraphRAG
    Automatic,
    /// Handmatig door gebruiker
    Manual,
    /// Via AI suggestie (geaccepteerd)
    AiSuggestion,
    /// Via regelgebaseerde matching
    RuleBased,
}

/// Context vector voor semantische zoekfunctionaliteit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextVector {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub embedding: Vec<f32>,
    pub model_name: String,
    pub model_version: String,
    pub created_at: DateTime<Utc>,
}

impl ContextVector {
    /// Bereken cosine similarity met andere vector
    pub fn cosine_similarity(&self, other: &ContextVector) -> f32 {
        if self.embedding.len() != other.embedding.len() {
            return 0.0;
        }

        let dot_product: f32 = self
            .embedding
            .iter()
            .zip(other.embedding.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

/// Resultaat van GraphRAG analyse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysisResult {
    pub domain_id: Uuid,
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub communities: Vec<Uuid>,
    pub related_domains: Vec<DomainRelation>,
    pub keywords: Vec<String>,
    pub analyzed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let vec1 = ContextVector {
            id: Uuid::new_v4(),
            domain_id: Uuid::new_v4(),
            embedding: vec![1.0, 0.0, 0.0],
            model_name: "test".to_string(),
            model_version: "1.0".to_string(),
            created_at: Utc::now(),
        };

        let vec2 = ContextVector {
            embedding: vec![1.0, 0.0, 0.0],
            ..vec1.clone()
        };

        let vec3 = ContextVector {
            embedding: vec![0.0, 1.0, 0.0],
            ..vec1.clone()
        };

        // Identieke vectoren: similarity = 1.0
        assert!((vec1.cosine_similarity(&vec2) - 1.0).abs() < 0.001);

        // Orthogonale vectoren: similarity = 0.0
        assert!(vec1.cosine_similarity(&vec3).abs() < 0.001);
    }

    #[test]
    fn test_entity_type_serialization() {
        let et = EntityType::Organization;
        let json = serde_json::to_string(&et).unwrap();
        assert_eq!(json, "\"ORGANIZATION\"");
    }
}
