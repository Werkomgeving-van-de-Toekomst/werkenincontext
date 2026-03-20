//! Convenience wrappers for stakeholder entities with typed metadata access
//!
//! These wrappers extend the base Entity type with type-safe accessors for
//! stakeholder-specific metadata stored in the Entity.metadata field.

use chrono::{DateTime, Utc};
use iou_core::graphrag::{Entity, EntityType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Convenience wrapper for person entities
///
/// Provides typed access to person-specific metadata stored in Entity.metadata.
/// The underlying Entity is always accessible via the `entity` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonStakeholder {
    pub entity: Entity,
}

impl PersonStakeholder {
    /// Metadata key for person title
    const METADATA_TITLE: &str = "title";
    /// Metadata key for person role
    const METADATA_ROLE: &str = "role";
    /// Metadata key for department
    const METADATA_DEPARTMENT: &str = "department";
    /// Metadata key for email
    const METADATA_EMAIL: &str = "email";
    /// Metadata key for phone
    const METADATA_PHONE: &str = "phone";
    /// Metadata key for PII classification
    const METADATA_PII_CLASSIFICATION: &str = "pii_classification";

    /// Create a new person stakeholder
    pub fn new(name: String, confidence: f32) -> Self {
        Self {
            entity: Entity {
                id: Uuid::new_v4(),
                name,
                entity_type: EntityType::Person,
                canonical_name: None,
                description: None,
                confidence,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: Utc::now(),
            }
        }
    }

    /// Get the person's title (dr., prof., etc.)
    pub fn title(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_TITLE)?.as_str()
    }

    /// Set the person's title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        let _ = self.entity.metadata.as_object_mut()
            .unwrap()
            .insert(Self::METADATA_TITLE.to_string(), serde_json::json!(title.into()));
        self
    }

    /// Get the person's role
    pub fn role(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_ROLE)?.as_str()
    }

    /// Set the person's role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        let _ = self.entity.metadata.as_object_mut()
            .unwrap()
            .insert(Self::METADATA_ROLE.to_string(), serde_json::json!(role.into()));
        self
    }

    /// Get the department
    pub fn department(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_DEPARTMENT)?.as_str()
    }

    /// Set the department
    pub fn with_department(mut self, department: impl Into<String>) -> Self {
        let _ = self.entity.metadata.as_object_mut()
            .unwrap()
            .insert(Self::METADATA_DEPARTMENT.to_string(), serde_json::json!(department.into()));
        self
    }

    /// Get email address
    pub fn email(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_EMAIL)?.as_str()
    }

    /// Set email address
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        let _ = self.entity.metadata.as_object_mut()
            .unwrap()
            .insert(Self::METADATA_EMAIL.to_string(), serde_json::json!(email.into()));
        self
    }

    /// Get phone number
    pub fn phone(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_PHONE)?.as_str()
    }

    /// Set phone number
    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        let _ = self.entity.metadata.as_object_mut()
            .unwrap()
            .insert(Self::METADATA_PHONE.to_string(), serde_json::json!(phone.into()));
        self
    }

    /// Convert to base Entity
    pub fn into_entity(self) -> Entity {
        self.entity
    }

    /// Borrow as base Entity
    pub fn as_entity(&self) -> &Entity {
        &self.entity
    }
}

impl From<Entity> for PersonStakeholder {
    fn from(entity: Entity) -> Self {
        assert_eq!(entity.entity_type, EntityType::Person);
        Self { entity }
    }
}

impl From<PersonStakeholder> for Entity {
    fn from(stakeholder: PersonStakeholder) -> Self {
        stakeholder.entity
    }
}

/// Convenience wrapper for organization entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationStakeholder {
    pub entity: Entity,
}

impl OrganizationStakeholder {
    /// Metadata keys for organization attributes
    const METADATA_SHORT_NAME: &str = "short_name";
    const METADATA_ORG_TYPE: &str = "org_type";
    const METADATA_PARENT_ORG: &str = "parent_org";
    const METADATA_LOCATION: &str = "location";

    /// Create a new organization stakeholder
    pub fn new(name: String, confidence: f32) -> Self {
        Self {
            entity: Entity {
                id: Uuid::new_v4(),
                name,
                entity_type: EntityType::Organization,
                canonical_name: None,
                description: None,
                confidence,
                source_domain_id: None,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: Utc::now(),
            }
        }
    }

    /// Get short name/abbreviation (e.g., "MinFin")
    pub fn short_name(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_SHORT_NAME)?.as_str()
    }

    /// Set short name
    pub fn with_short_name(mut self, short_name: impl Into<String>) -> Self {
        let _ = self.entity.metadata.as_object_mut()
            .unwrap()
            .insert(Self::METADATA_SHORT_NAME.to_string(), serde_json::json!(short_name.into()));
        self
    }

    /// Get organization type
    pub fn org_type(&self) -> Option<OrgType> {
        self.entity.metadata
            .get(Self::METADATA_ORG_TYPE)?
            .as_str()
            .and_then(|s| OrgType::try_from(s).ok())
    }

    /// Set organization type
    pub fn with_org_type(mut self, org_type: OrgType) -> Self {
        let _ = self.entity.metadata.as_object_mut()
            .unwrap()
            .insert(Self::METADATA_ORG_TYPE.to_string(), serde_json::json!(org_type.to_string()));
        self
    }

    /// Get parent organization
    pub fn parent_org(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_PARENT_ORG)?.as_str()
    }

    /// Set parent organization
    pub fn with_parent_org(mut self, parent: impl Into<String>) -> Self {
        let _ = self.entity.metadata.as_object_mut()
            .unwrap()
            .insert(Self::METADATA_PARENT_ORG.to_string(), serde_json::json!(parent.into()));
        self
    }

    /// Get location
    pub fn location(&self) -> Option<&str> {
        self.entity.metadata.get(Self::METADATA_LOCATION)?.as_str()
    }

    /// Set location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        let _ = self.entity.metadata.as_object_mut()
            .unwrap()
            .insert(Self::METADATA_LOCATION.to_string(), serde_json::json!(location.into()));
        self
    }

    /// Convert to base Entity
    pub fn into_entity(self) -> Entity {
        self.entity
    }
}

impl From<Entity> for OrganizationStakeholder {
    fn from(entity: Entity) -> Self {
        assert_eq!(entity.entity_type, EntityType::Organization);
        Self { entity }
    }
}

impl From<OrganizationStakeholder> for Entity {
    fn from(stakeholder: OrganizationStakeholder) -> Self {
        stakeholder.entity
    }
}

/// Type of organization (Dutch government context)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrgType {
    Ministry,
    Agency,
    Municipal,
    Other,
}

impl std::fmt::Display for OrgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrgType::Ministry => write!(f, "Ministry"),
            OrgType::Agency => write!(f, "Agency"),
            OrgType::Municipal => write!(f, "Municipal"),
            OrgType::Other => write!(f, "Other"),
        }
    }
}

impl TryFrom<&str> for OrgType {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "ministry" | "ministerie" => Ok(OrgType::Ministry),
            "agency" | "dienst" | "agentschap" => Ok(OrgType::Agency),
            "municipal" | "gemeente" => Ok(OrgType::Municipal),
            _ => Ok(OrgType::Other),
        }
    }
}

impl AsRef<str> for OrgType {
    fn as_ref(&self) -> &str {
        match self {
            OrgType::Ministry => "ministry",
            OrgType::Agency => "agency",
            OrgType::Municipal => "municipal",
            OrgType::Other => "other",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_stakeholder_creates_valid_entity() {
        let person = PersonStakeholder::new("Jan de Vries".to_string(), 0.95);

        assert_eq!(person.entity.entity_type, EntityType::Person);
        assert_eq!(person.entity.name, "Jan de Vries");
        assert_eq!(person.entity.confidence, 0.95);
    }

    #[test]
    fn test_person_stakeholder_metadata_accessors() {
        let person = PersonStakeholder::new("Jan de Vries".to_string(), 0.95)
            .with_title("dr.")
            .with_role("minister")
            .with_department("MinFin");

        assert_eq!(person.title(), Some("dr."));
        assert_eq!(person.role(), Some("minister"));
        assert_eq!(person.department(), Some("MinFin"));
    }

    #[test]
    fn test_organization_stakeholder_creates_valid_entity() {
        let org = OrganizationStakeholder::new("Ministerie van Financiën".to_string(), 0.98);

        assert_eq!(org.entity.entity_type, EntityType::Organization);
        assert_eq!(org.entity.name, "Ministerie van Financiën");
    }

    #[test]
    fn test_organization_stakeholder_metadata() {
        let org = OrganizationStakeholder::new("MinFin".to_string(), 0.95)
            .with_short_name("MinFin")
            .with_org_type(OrgType::Ministry);

        assert_eq!(org.short_name(), Some("MinFin"));
        assert_eq!(org.org_type(), Some(OrgType::Ministry));
    }

    #[test]
    fn test_confidence_scores_always_valid() {
        for confidence in [0.0, 0.5, 0.9, 1.0] {
            let person = PersonStakeholder::new("Test".to_string(), confidence);
            assert!(person.entity.confidence >= 0.0 && person.entity.confidence <= 1.0);
        }
    }

    #[test]
    fn test_org_type_from_string() {
        assert_eq!(OrgType::try_from("ministerie"), Ok(OrgType::Ministry));
        assert_eq!(OrgType::try_from("Ministry"), Ok(OrgType::Ministry));
        assert_eq!(OrgType::try_from("dienst"), Ok(OrgType::Agency));
        assert_eq!(OrgType::try_from("gemeente"), Ok(OrgType::Municipal));
        assert_eq!(OrgType::try_from("unknown"), Ok(OrgType::Other));
    }
}
