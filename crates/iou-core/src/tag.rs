//! Tags voor flexibele classificatie en vindbaarheid
//!
//! Tags bieden een flexibele manier om informatieobjecten en domeinen
//! te labelen zonder vooraf gedefinieerde taxonomie. Ze ondersteunen
//! zowel vrije tags als gecontroleerde vocabulaires.

// Re-export repository when server feature is enabled
#[cfg(feature = "server")]
pub mod repository;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

/// Tag voor classificatie van informatieobjecten en domeinen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub normalized_name: String,
    pub tag_type: TagType,
    pub organization_id: Option<Uuid>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub parent_tag_id: Option<Uuid>,
    pub usage_count: i32,
    pub is_system_tag: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tag {
    pub fn new(name: String, tag_type: TagType) -> Self {
        let normalized = Self::normalize_name(&name);
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            normalized_name: normalized,
            tag_type,
            organization_id: None,
            description: None,
            color: None,
            parent_tag_id: None,
            usage_count: 0,
            is_system_tag: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Normaliseer tagnaam voor consistentie (lowercase, spaties -> dashes)
    pub fn normalize_name(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }

    /// Increment gebruikstelling
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
        self.updated_at = Utc::now();
    }

    /// Decrement gebruikstelling
    pub fn decrement_usage(&mut self) {
        self.usage_count = (self.usage_count - 1).max(0);
        self.updated_at = Utc::now();
    }
}

/// Type tag bepaalt gebruik en validatie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum TagType {
    /// Vrije tag - gebruikers kunnen aanmaken
    Free,
    /// Gecontroleerde vocabulaire - beheerde lijst
    Controlled,
    /// Beleidsagenda - gangbare politieke thema's
    Policy,
    /// Subsidie - subsidiecategorieën
    Subsidy,
    /// Vergunning - vergunningtypes
    Permit,
    /// Afdeling - interne afdelingsindeling
    Department,
    /// Project fase - projectmanagement fases
    ProjectPhase,
    /// Zaakstatus - zaakspecifieke statussen
    CaseStatus,
    /// Custom - extensie door organisatie
    Custom,
}

/// Koppeling tussen tag en informatieobject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectTag {
    pub id: Uuid,
    pub object_id: Uuid,
    pub tag_id: Uuid,
    pub tagged_by: Uuid,
    pub confidence: f32,
    pub is_auto_assigned: bool,
    pub created_at: DateTime<Utc>,
}

impl ObjectTag {
    pub fn new(object_id: Uuid, tag_id: Uuid, tagged_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            object_id,
            tag_id,
            tagged_by,
            confidence: 1.0,
            is_auto_assigned: false,
            created_at: Utc::now(),
        }
    }

    pub fn with_auto_assign(mut self, confidence: f32) -> Self {
        self.is_auto_assigned = true;
        self.confidence = confidence;
        self
    }
}

/// Koppeling tussen tag en informatiedomein
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainTag {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub tag_id: Uuid,
    pub tagged_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl DomainTag {
    pub fn new(domain_id: Uuid, tag_id: Uuid, tagged_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            domain_id,
            tag_id,
            tagged_by,
            created_at: Utc::now(),
        }
    }
}

/// Tag suggestie op basis van context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSuggestion {
    pub tag_id: Uuid,
    pub tag_name: String,
    pub confidence: f32,
    pub reason: SuggestionReason,
}

impl TagSuggestion {
    pub fn new(tag_id: Uuid, tag_name: String, confidence: f32, reason: SuggestionReason) -> Self {
        Self {
            tag_id,
            tag_name,
            confidence,
            reason,
        }
    }
}

/// Reden voor tag suggestie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionReason {
    /// Tag wordt veel gebruikt in vergelijkbare objecten
    FrequentlyUsed,
    /// Tag is voorgesteld door AI op basis van inhoud
    AiSuggested,
    /// Tag is vereist voor dit type domein
    DomainRequired,
    /// Tag is recent gebruikt door dezelfde gebruiker
    RecentlyUsed,
    /// Tag is gerelateerd aan andere tags op dit object
    RelatedTags,
}

/// Tag statistieken voor analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStats {
    pub tag_id: Uuid,
    pub tag_name: String,
    pub usage_count: i32,
    pub objects_count: i32,
    pub domains_count: i32,
    pub last_used: Option<DateTime<Utc>>,
    pub trending_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_normalization() {
        assert_eq!(Tag::normalize_name("Subsidie"), "subsidie");
        assert_eq!(Tag::normalize_name("Omgevingsvergunning"), "omgevingsvergunning");
        assert_eq!(Tag::normalize_name("Woo - Openbaar"), "woo-openbaar");
        assert_eq!(Tag::normalize_name("  Veel  Spaties  "), "veel-spacies");
    }

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("Subsidie".to_string(), TagType::Policy);
        assert_eq!(tag.normalized_name, "subsidie");
        assert_eq!(tag.usage_count, 0);
        assert!(!tag.is_system_tag);
    }

    #[test]
    fn test_tag_usage_tracking() {
        let mut tag = Tag::new("Test".to_string(), TagType::Free);
        tag.increment_usage();
        assert_eq!(tag.usage_count, 1);
        tag.decrement_usage();
        assert_eq!(tag.usage_count, 0);
    }

    #[test]
    fn test_object_tag_auto_assign() {
        let obj_tag = ObjectTag::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4())
            .with_auto_assign(0.85);

        assert!(obj_tag.is_auto_assigned);
        assert_eq!(obj_tag.confidence, 0.85);
    }
}
