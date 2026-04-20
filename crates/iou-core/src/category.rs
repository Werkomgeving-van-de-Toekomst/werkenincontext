//! Categories voor hiërarchische classificatie
//!
//! Categories bieden een gestructureerde taxonomie voor het classificeren
//! van informatieobjecten en domeinen. In tegenstelling tot tags zijn
//! categories hiërarchisch en beheerd.

// Re-export repository when server feature is enabled
#[cfg(feature = "server")]
pub mod repository;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

/// Category voor hiërarchische classificatie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category_type: CategoryType,
    pub organization_id: Option<Uuid>,
    pub parent_category_id: Option<Uuid>,
    pub level: i32,
    pub path: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Category {
    pub fn new(code: String, name: String, category_type: CategoryType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            code,
            name,
            description: None,
            category_type,
            organization_id: None,
            parent_category_id: None,
            level: 0,
            path: String::new(),
            icon: None,
            color: None,
            sort_order: 0,
            is_active: true,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
        }
    }

    /// Bereken niveau op basis van parent
    pub fn calculate_level(&mut self, parent_level: Option<i32>) {
        self.level = parent_level.map_or(0, |l| l + 1);
    }

    /// Genereer pad voor hiërarchie weergave
    pub fn generate_path(&mut self, parent_path: Option<&str>) {
        let parent = parent_path.unwrap_or("");
        self.path = if parent.is_empty() {
            self.code.clone()
        } else {
            format!("{}/{}", parent, self.code)
        };
    }

    /// Controleer of category een bladnode is (geen kinderen)
    pub fn is_leaf(&self) -> bool {
        self.level > 0
    }

    /// Volledige naam inclusief parent categories
    pub fn full_name(&self) -> String {
        if self.path.is_empty() {
            self.name.clone()
        } else {
            self.path.replace('/', " > ")
        }
    }
}

/// Type category bepaalt toepassingsgebied
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum CategoryType {
    /// Documenttype (besluit, rapport, memo, etc.)
    DocumentType,
    /// Beleidsveld (ruimtelijke ordening, economie, etc.)
    PolicyArea,
    /// Subsidiecategorie
    Subsidy,
    /// Vergunningtype
    Permit,
    /// Woo categorie (informatie- en inspraakcategorie)
    WooCategory,
    /// Archiefselectielijst
    RetentionSchedule,
    /// Afdeling/organisatie-eenheid
    Department,
    /// Projectcategorie
    Project,
    /// Custom categorie
    Custom,
}

/// Koppeling tussen category en informatieobject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCategory {
    pub id: Uuid,
    pub object_id: Uuid,
    pub category_id: Uuid,
    pub is_primary: bool,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
}

impl ObjectCategory {
    pub fn new(object_id: Uuid, category_id: Uuid, assigned_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            object_id,
            category_id,
            is_primary: false,
            assigned_by,
            assigned_at: Utc::now(),
        }
    }

    pub fn primary(mut self) -> Self {
        self.is_primary = true;
        self
    }
}

/// Koppeling tussen category en informatiedomein
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCategory {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub category_id: Uuid,
    pub assigned_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl DomainCategory {
    pub fn new(domain_id: Uuid, category_id: Uuid, assigned_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            domain_id,
            category_id,
            assigned_by,
            created_at: Utc::now(),
        }
    }
}

/// Category boomstructuur voor UI weergave
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryNode {
    pub category: Category,
    pub children: Vec<CategoryNode>,
    pub object_count: i32,
    pub has_objects_in_subtree: bool,
}

impl CategoryNode {
    pub fn new(category: Category) -> Self {
        Self {
            category,
            children: Vec::new(),
            object_count: 0,
            has_objects_in_subtree: false,
        }
    }

    /// Voeg kind toe aan de boom
    pub fn add_child(&mut self, node: CategoryNode) {
        self.children.push(node);
    }

    /// Recursief zoeken naar category op ID
    pub fn find_by_id(&self, id: Uuid) -> Option<&CategoryNode> {
        if self.category.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_id(id) {
                return Some(found);
            }
        }
        None
    }

    /// Recursief zoeken en muteren
    pub fn find_by_id_mut(&mut self, id: Uuid) -> Option<&mut CategoryNode> {
        if self.category.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_by_id_mut(id) {
                return Some(found);
            }
        }
        None
    }

    /// Tel totaal aantal objecten in subtree
    pub fn total_object_count(&self) -> i32 {
        self.children.iter().fold(self.object_count, |acc, child| {
            acc + child.total_object_count()
        })
    }
}

/// Category migratie voor het bijwerken van taxonomie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMigration {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub source_category_id: Uuid,
    pub target_category_id: Uuid,
    pub migration_strategy: MigrationStrategy,
    pub status: MigrationStatus,
    pub affected_objects_count: i32,
    pub migrated_objects_count: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Strategie voor category migratie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStrategy {
    /// Verplaats alle objecten naar doelcategory
    Move,
    /// Kopieer naar doelcategory (beide behouden)
    Copy,
    /// Verwijder oude toewijzing
    Remove,
    /// Handmatige beoordeling vereist
    Manual,
}

/// Status van migratie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Category statistieken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category_id: Uuid,
    pub category_name: String,
    pub direct_objects_count: i32,
    pub total_objects_in_subtree: i32,
    pub child_categories_count: i32,
    pub depth: i32,
    pub last_used: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_creation() {
        let cat = Category::new("DOC-001".to_string(), "Besluit".to_string(), CategoryType::DocumentType);
        assert_eq!(cat.level, 0);
        assert!(cat.is_active);
    }

    #[test]
    fn test_category_hierarchy() {
        let mut parent = Category::new("POL-001".to_string(), "Ruimtelijke Ordening".to_string(), CategoryType::PolicyArea);
        parent.calculate_level(None);
        parent.generate_path(None);

        assert_eq!(parent.level, 0);
        assert_eq!(parent.path, "POL-001");

        let mut child = Category::new("POL-001-A".to_string(), "Vergunningen".to_string(), CategoryType::PolicyArea);
        child.calculate_level(Some(parent.level));
        child.generate_path(Some(&parent.path));

        assert_eq!(child.level, 1);
        assert_eq!(child.path, "POL-001/POL-001-A");
    }

    #[test]
    fn test_category_tree() {
        let root = Category::new("ROOT".to_string(), "Root".to_string(), CategoryType::DocumentType);
        let mut tree = CategoryNode::new(root);

        let child1 = Category::new("C1".to_string(), "Child 1".to_string(), CategoryType::DocumentType);
        tree.add_child(CategoryNode::new(child1));

        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.total_object_count(), 0);
    }

    #[test]
    fn test_object_category_primary() {
        let obj_cat = ObjectCategory::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()).primary();
        assert!(obj_cat.is_primary);
    }

    #[test]
    fn test_full_name_generation() {
        let mut cat = Category::new("A".to_string(), "Level A".to_string(), CategoryType::DocumentType);
        cat.path = "ROOT/A".to_string();

        assert_eq!(cat.full_name(), "ROOT > A");
    }
}
