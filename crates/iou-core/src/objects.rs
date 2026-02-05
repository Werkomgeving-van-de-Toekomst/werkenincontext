//! Informatieobjecten - documenten, emails, besluiten etc.
//!
//! Elk informatieobject behoort tot een informatiedomein en heeft
//! automatische compliance metadata (classificatie, bewaartermijn, Woo-relevantie).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::compliance::{Classification, PrivacyLevel};

/// Type informatieobject
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ObjectType {
    /// Document (rapporten, notities, memo's)
    Document,
    /// Email correspondentie
    Email,
    /// Chat berichten
    Chat,
    /// Formeel besluit
    Besluit,
    /// Dataset of gestructureerde data
    Data,
}

/// Informatieobject met compliance metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationObject {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub object_type: ObjectType,
    pub title: String,
    pub description: Option<String>,
    pub content_location: String,
    pub content_text: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,

    // Compliance metadata (automatisch ingevuld)
    pub classification: Classification,
    pub retention_period: Option<i32>,
    pub is_woo_relevant: bool,
    pub woo_publication_date: Option<DateTime<Utc>>,
    pub privacy_level: PrivacyLevel,

    // Vindbaarheid
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,

    // Versioning
    pub version: i32,
    pub previous_version_id: Option<Uuid>,

    // Audit
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InformationObject {
    pub fn new(
        domain_id: Uuid,
        object_type: ObjectType,
        title: String,
        content_location: String,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            domain_id,
            object_type,
            title,
            description: None,
            content_location,
            content_text: None,
            mime_type: None,
            file_size: None,
            classification: Classification::default(),
            retention_period: None,
            is_woo_relevant: false,
            woo_publication_date: None,
            privacy_level: PrivacyLevel::default(),
            tags: Vec::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            version: 1,
            previous_version_id: None,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// Voeg een tag toe (case-insensitive, geen duplicaten)
    pub fn add_tag(&mut self, tag: &str) {
        let tag_lower = tag.to_lowercase();
        if !self.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
            self.tags.push(tag.to_string());
        }
    }

    /// Controleer of object Woo-relevant is op basis van type
    pub fn should_be_woo_relevant(&self) -> bool {
        matches!(self.object_type, ObjectType::Besluit)
            || self.classification == Classification::Openbaar
    }

    /// Stel standaard bewaartermijn in op basis van object type
    pub fn default_retention_period(&self) -> i32 {
        match self.object_type {
            ObjectType::Besluit => 20,  // Archiefwet: besluiten 20 jaar
            ObjectType::Document => 10, // Standaard documenten 10 jaar
            ObjectType::Email => 5,     // Email 5 jaar
            ObjectType::Chat => 1,      // Chat 1 jaar
            ObjectType::Data => 10,     // Data 10 jaar
        }
    }
}

/// Verwijzing naar een informatieobject (voor lijsten)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectReference {
    pub id: Uuid,
    pub object_type: ObjectType,
    pub title: String,
    pub classification: Classification,
    pub is_woo_relevant: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&InformationObject> for ObjectReference {
    fn from(obj: &InformationObject) -> Self {
        Self {
            id: obj.id,
            object_type: obj.object_type,
            title: obj.title.clone(),
            classification: obj.classification,
            is_woo_relevant: obj.is_woo_relevant,
            created_at: obj.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_creation() {
        let obj = InformationObject::new(
            Uuid::new_v4(),
            ObjectType::Document,
            "Test document".to_string(),
            "/documents/test.pdf".to_string(),
            Uuid::new_v4(),
        );

        assert_eq!(obj.version, 1);
        assert_eq!(obj.classification, Classification::Intern);
        assert!(!obj.is_woo_relevant);
    }

    #[test]
    fn test_tags() {
        let mut obj = InformationObject::new(
            Uuid::new_v4(),
            ObjectType::Document,
            "Test".to_string(),
            "/test".to_string(),
            Uuid::new_v4(),
        );

        obj.add_tag("subsidie");
        obj.add_tag("SUBSIDIE"); // Duplicaat (case-insensitive)
        obj.add_tag("milieu");

        assert_eq!(obj.tags.len(), 2);
    }

    #[test]
    fn test_besluit_is_woo_relevant() {
        let besluit = InformationObject::new(
            Uuid::new_v4(),
            ObjectType::Besluit,
            "Besluit vergunning".to_string(),
            "/besluiten/123.pdf".to_string(),
            Uuid::new_v4(),
        );

        assert!(besluit.should_be_woo_relevant());
        assert_eq!(besluit.default_retention_period(), 20);
    }
}
