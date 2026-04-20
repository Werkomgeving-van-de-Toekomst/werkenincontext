//! Data Entity Registry
//!
//! Central registry for all data entities across the organization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

use super::quality::QualityScore;

/// Unique identifier for an entity in the registry
pub type EntityId = Uuid;

/// External entity ID (from source system)
pub type ExternalId = String;

/// Canonical ID (unified across sources)
pub type CanonicalId = Uuid;

/// Entity types in the registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Natural person (citizen/resident)
    Citizen,

    /// Organization/company
    Organization,

    /// Case/zaak
    Case,

    /// Document
    Document,

    /// Policy decision
    Policy,

    /// Location/address
    Location,

    /// Building/property
    Building,

    /// Asset/equipment
    Asset,

    /// Event/occurrence
    Event,

    /// Custom/other entity type
    Other,
}

/// Data entity in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataEntity {
    /// Unique registry ID
    pub id: EntityId,

    /// Canonical ID (unified across sources)
    pub canonical_id: CanonicalId,

    /// Source system this entity came from
    pub source_id: Uuid,

    /// External ID in source system
    pub external_id: ExternalId,

    /// Entity type
    pub entity_type: EntityType,

    /// Entity name
    pub name: String,

    /// Entity metadata (flexible schema)
    pub metadata: serde_json::Value,

    /// Quality score
    pub quality_score: QualityScore,

    /// When this entity was last synced
    pub last_synced_at: Option<DateTime<Utc>>,

    /// When this entity was created in registry
    pub created_at: DateTime<Utc>,

    /// When this entity was last updated
    pub updated_at: DateTime<Utc>,

    /// Whether this entity is active
    pub is_active: bool,
}

impl DataEntity {
    /// Create a new data entity
    pub fn new(
        source_id: Uuid,
        external_id: ExternalId,
        entity_type: EntityType,
        name: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            canonical_id: Uuid::new_v4(),
            source_id,
            external_id,
            entity_type,
            name,
            metadata: serde_json::json!({}),
            quality_score: QualityScore::default(),
            last_synced_at: None,
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }

    /// Add metadata to the entity
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set quality score
    pub fn with_quality(mut self, score: QualityScore) -> Self {
        self.quality_score = score;
        self
    }

    /// Check if entity is above quality threshold
    pub fn meets_quality_threshold(&self, threshold: f32) -> bool {
        self.quality_score.overall() >= threshold
    }

    /// Get a metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Update the entity's sync timestamp
    pub fn mark_synced(&mut self) {
        self.last_synced_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Deactivate the entity
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
}

/// Entity match result (for deduplication)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatch {
    /// Matched entity
    pub entity: DataEntity,

    /// Match confidence (0-1)
    pub confidence: f32,

    /// Match reason
    pub reason: MatchReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchReason {
    /// Exact external ID match
    ExactExternalId,

    /// Exact canonical ID match
    ExactCanonicalId,

    /// Fuzzy name match
    FuzzyNameMatch,

    /// Metadata attribute match
    AttributeMatch { attribute: String },

    /// Cross-reference match
    CrossReference { reference_type: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_entity_creation() {
        let entity = DataEntity::new(
            Uuid::new_v4(),
            "EXT-123".to_string(),
            EntityType::Citizen,
            "John Doe".to_string(),
        );

        assert!(entity.is_active);
        assert_eq!(entity.external_id, "EXT-123");
        assert_eq!(entity.entity_type, EntityType::Citizen);
    }

    #[test]
    fn test_data_entity_with_metadata() {
        let metadata = serde_json::json!({
            "bsn": "123456789",
            "birth_date": "1990-01-01"
        });

        let entity = DataEntity::new(
            Uuid::new_v4(),
            "EXT-123".to_string(),
            EntityType::Citizen,
            "John Doe".to_string(),
        )
        .with_metadata(metadata.clone());

        assert_eq!(entity.metadata, metadata);
    }

    #[test]
    fn test_data_entity_quality_threshold() {
        let mut entity = DataEntity::new(
            Uuid::new_v4(),
            "EXT-123".to_string(),
            EntityType::Citizen,
            "John Doe".to_string(),
        );

        entity.quality_score = QualityScore::new(0.8, 0.9, 0.7);

        assert!(entity.meets_quality_threshold(0.7));
        assert!(!entity.meets_quality_threshold(0.9));
    }

    #[test]
    fn test_data_entity_mark_synced() {
        let mut entity = DataEntity::new(
            Uuid::new_v4(),
            "EXT-123".to_string(),
            EntityType::Citizen,
            "John Doe".to_string(),
        );

        assert!(entity.last_synced_at.is_none());

        entity.mark_synced();

        assert!(entity.last_synced_at.is_some());
    }

    #[test]
    fn test_data_entity_deactivate() {
        let mut entity = DataEntity::new(
            Uuid::new_v4(),
            "EXT-123".to_string(),
            EntityType::Citizen,
            "John Doe".to_string(),
        );

        assert!(entity.is_active);

        entity.deactivate();

        assert!(!entity.is_active);
    }
}
