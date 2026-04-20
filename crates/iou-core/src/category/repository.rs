//! Category repository for CRUD operations
//!
//! Provides database operations for category management including
//! hierarchical operations, tree traversal, and statistics.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, postgres::PgRow};
use thiserror::Error;
use uuid::Uuid;

use crate::category::{
    Category, CategoryType, ObjectCategory, DomainCategory,
    CategoryNode, CategoryMigration, MigrationStrategy, MigrationStatus,
    CategoryStats,
};

/// Errors that can occur during category operations
#[derive(Debug, Error)]
pub enum CategoryError {
    #[error("Category not found: {0}")]
    CategoryNotFound(Uuid),

    #[error("Category code already exists: {0}")]
    CategoryAlreadyExists(String),

    #[error("Cannot create cycle in category hierarchy")]
    CycleDetected,

    #[error("Invalid parent category: {0}")]
    InvalidParentCategory(Uuid),

    #[error("Object not found: {0}")]
    ObjectNotFound(Uuid),

    #[error("Domain not found: {0}")]
    DomainNotFound(Uuid),

    #[error("Invalid category type: {0}")]
    InvalidCategoryType(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Repository for category operations
pub struct CategoryRepository {
    pool: PgPool,
}

impl CategoryRepository {
    /// Create a new CategoryRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Category CRUD operations
    // ========================================================================

    /// Create a new category
    pub async fn create_category(
        &self,
        code: String,
        name: String,
        category_type: CategoryType,
        organization_id: Option<Uuid>,
        parent_category_id: Option<Uuid>,
        description: Option<String>,
        icon: Option<String>,
        color: Option<String>,
        sort_order: Option<i32>,
    ) -> Result<Category, CategoryError> {
        // Check for cycles if parent is specified
        if let Some(parent_id) = parent_category_id {
            if self.would_create_cycle(parent_id).await? {
                return Err(CategoryError::CycleDetected);
            }
        }

        let category_type_str = category_type.to_string();

        let row = sqlx::query(
            r#"
            INSERT INTO categories (code, name, category_type, organization_id, parent_category_id, description, icon, color, sort_order)
            VALUES ($1, $2, $3::VARCHAR, $4, $5, $6, $7, $8, $9)
            RETURNING
                id, code, name, description, category_type, organization_id, parent_category_id,
                level, path, icon, color, sort_order, is_active, metadata, created_at, updated_at
            "#,
        )
        .bind(&code)
        .bind(&name)
        .bind(&category_type_str)
        .bind(organization_id)
        .bind(parent_category_id)
        .bind(description)
        .bind(icon.as_deref())
        .bind(color.as_deref())
        .bind(sort_order.unwrap_or(0))
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::row_to_category(row)?)
    }

    /// Get category by ID
    pub async fn get_category(&self, category_id: Uuid) -> Result<Category, CategoryError> {
        let row = sqlx::query(
            r#"
            SELECT
                id, code, name, description, category_type, organization_id, parent_category_id,
                level, path, icon, color, sort_order, is_active, metadata, created_at, updated_at
            FROM categories
            WHERE id = $1
            "#,
        )
        .bind(category_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(CategoryError::CategoryNotFound(category_id))?;

        Ok(Self::row_to_category(row)?)
    }

    /// Get category by type and code
    pub async fn get_category_by_code(
        &self,
        category_type: CategoryType,
        code: &str,
    ) -> Result<Category, CategoryError> {
        let category_type_str = category_type.to_string();

        let row = sqlx::query(
            r#"
            SELECT
                id, code, name, description, category_type, organization_id, parent_category_id,
                level, path, icon, color, sort_order, is_active, metadata, created_at, updated_at
            FROM categories
            WHERE category_type = $1 AND code = $2
            "#,
        )
        .bind(&category_type_str)
        .bind(code)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| CategoryError::CategoryNotFound(Uuid::nil()))?;

        Ok(Self::row_to_category(row)?)
    }

    /// List categories by type
    pub async fn list_categories(
        &self,
        category_type: Option<CategoryType>,
        organization_id: Option<Uuid>,
        include_inactive: bool,
    ) -> Result<Vec<Category>, CategoryError> {
        let rows = if let Some(category_type) = category_type {
            let category_type_str = category_type.to_string();
            if include_inactive {
                sqlx::query(
                    r#"
                    SELECT
                        id, code, name, description, category_type, organization_id, parent_category_id,
                        level, path, icon, color, sort_order, is_active, metadata, created_at, updated_at
                    FROM categories
                    WHERE category_type = $1
                    ORDER BY sort_order, path
                    "#,
                )
                .bind(&category_type_str)
                .fetch_all(&self.pool)
                .await?
            } else {
                sqlx::query(
                    r#"
                    SELECT
                        id, code, name, description, category_type, organization_id, parent_category_id,
                        level, path, icon, color, sort_order, is_active, metadata, created_at, updated_at
                    FROM categories
                    WHERE category_type = $1 AND is_active = TRUE
                    ORDER BY sort_order, path
                    "#,
                )
                .bind(&category_type_str)
                .fetch_all(&self.pool)
                .await?
            }
        } else {
            sqlx::query(
                r#"
                SELECT
                    id, code, name, description, category_type, organization_id, parent_category_id,
                    level, path, icon, color, sort_order, is_active, metadata, created_at, updated_at
                FROM categories
                WHERE is_active = TRUE OR $1
                ORDER BY category_type, sort_order, path
                "#,
            )
            .bind(include_inactive)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(|r| Self::row_to_category(r)).collect()
    }

    /// Update category
    pub async fn update_category(
        &self,
        category_id: Uuid,
        name: Option<String>,
        description: Option<Option<String>>,
        icon: Option<Option<String>>,
        color: Option<Option<String>>,
        sort_order: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<Category, CategoryError> {
        let current = self.get_category(category_id).await?;

        let new_name = name.unwrap_or(current.name);
        let new_description = description.unwrap_or(current.description);
        let new_icon = icon.unwrap_or(current.icon);
        let new_color = color.unwrap_or(current.color);
        let new_sort_order = sort_order.unwrap_or(current.sort_order);
        let new_is_active = is_active.unwrap_or(current.is_active);

        let row = sqlx::query(
            r#"
            UPDATE categories
            SET name = $1, description = $2, icon = $3, color = $4, sort_order = $5, is_active = $6
            WHERE id = $7
            RETURNING
                id, code, name, description, category_type, organization_id, parent_category_id,
                level, path, icon, color, sort_order, is_active, metadata, created_at, updated_at
            "#,
        )
        .bind(&new_name)
        .bind(&new_description)
        .bind(new_icon.as_deref())
        .bind(new_color.as_deref())
        .bind(new_sort_order)
        .bind(new_is_active)
        .bind(category_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::row_to_category(row)?)
    }

    /// Delete category (cascades to object_categories and domain_categories)
    pub async fn delete_category(&self, category_id: Uuid) -> Result<(), CategoryError> {
        // Check if category has children
        let child_count: i64 = sqlx::query("SELECT COUNT(*) FROM categories WHERE parent_category_id = $1")
            .bind(category_id)
            .fetch_one(&self.pool)
            .await?
            .get("count");

        if child_count > 0 {
            return Err(CategoryError::InvalidParentCategory(category_id));
        }

        let result = sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(category_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(CategoryError::CategoryNotFound(category_id));
        }

        Ok(())
    }

    // ========================================================================
    // Category tree operations
    // ========================================================================

    /// Get category tree for a type
    pub async fn get_category_tree(
        &self,
        category_type: CategoryType,
    ) -> Result<CategoryNode, CategoryError> {
        // Get all active categories for this type
        let categories = self.list_categories(Some(category_type), None, false).await?;

        // Build tree structure
        let mut by_id: std::collections::HashMap<Uuid, CategoryNode> = std::collections::HashMap::new();

        // First pass: create nodes and collect parent relationships
        let mut parent_relationships: Vec<(Uuid, Option<Uuid>)> = Vec::new();
        for cat in categories {
            let id = cat.id;
            let parent_id = cat.parent_category_id;
            let node = CategoryNode::new(cat);
            by_id.insert(id, node);
            parent_relationships.push((id, parent_id));
        }

        // Second pass: build hierarchy
        let mut root_ids: Vec<Uuid> = Vec::new();
        for (id, parent_id) in parent_relationships {
            if let Some(parent_id) = parent_id {
                // Clone child first to avoid borrow checker issue
                let child_clone = by_id.get(&id).cloned();
                if let Some(parent) = by_id.get_mut(&parent_id) {
                    if let Some(child) = child_clone {
                        parent.add_child(child);
                    }
                }
            } else {
                root_ids.push(id);
            }
        }

        // Return first root node (or create virtual root if multiple)
        if let Some(first_root_id) = root_ids.first() {
            Ok(by_id.remove(first_root_id).unwrap())
        } else {
            // Return empty virtual root
            Ok(CategoryNode {
                category: Category::new("__ROOT__".to_string(), "Root".to_string(), category_type),
                children: Vec::new(),
                object_count: 0,
                has_objects_in_subtree: false,
            })
        }
    }

    /// Move category to new parent
    pub async fn move_category(
        &self,
        category_id: Uuid,
        new_parent_id: Option<Uuid>,
    ) -> Result<Category, CategoryError> {
        // Check for cycles
        if let Some(parent_id) = new_parent_id {
            if self.would_create_cycle_for_move(category_id, parent_id).await? {
                return Err(CategoryError::CycleDetected);
            }
        }

        // The trigger will automatically update path and level
        let row = sqlx::query(
            r#"
            UPDATE categories
            SET parent_category_id = $1
            WHERE id = $2
            RETURNING
                id, code, name, description, category_type, organization_id, parent_category_id,
                level, path, icon, color, sort_order, is_active, metadata, created_at, updated_at
            "#,
        )
        .bind(new_parent_id)
        .bind(category_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::row_to_category(row)?)
    }

    // ========================================================================
    // Object categorization operations
    // ========================================================================

    /// Add category to object
    pub async fn categorize_object(
        &self,
        object_id: Uuid,
        category_id: Uuid,
        is_primary: bool,
        assigned_by: Uuid,
    ) -> Result<ObjectCategory, CategoryError> {
        // Verify object exists
        let object_exists = sqlx::query("SELECT 1 FROM information_objects WHERE id = $1")
            .bind(object_id)
            .fetch_optional(&self.pool)
            .await?;

        if object_exists.is_none() {
            return Err(CategoryError::ObjectNotFound(object_id));
        }

        // If setting as primary, remove existing primary
        if is_primary {
            sqlx::query("UPDATE object_categories SET is_primary = FALSE WHERE object_id = $1")
                .bind(object_id)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query(
            r#"
            INSERT INTO object_categories (object_id, category_id, is_primary, assigned_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (object_id, category_id) DO UPDATE SET
                is_primary = EXCLUDED.is_primary,
                assigned_by = EXCLUDED.assigned_by
            RETURNING id, object_id, category_id, is_primary, assigned_by, assigned_at
            "#,
        )
        .bind(object_id)
        .bind(category_id)
        .bind(is_primary)
        .bind(assigned_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(ObjectCategory {
            id: row.get("id"),
            object_id: row.get("object_id"),
            category_id: row.get("category_id"),
            is_primary: row.get("is_primary"),
            assigned_by: row.get("assigned_by"),
            assigned_at: row.get("assigned_at"),
        })
    }

    /// Remove category from object
    pub async fn uncategorize_object(
        &self,
        object_id: Uuid,
        category_id: Uuid,
    ) -> Result<(), CategoryError> {
        let result = sqlx::query(
            "DELETE FROM object_categories WHERE object_id = $1 AND category_id = $2"
        )
        .bind(object_id)
        .bind(category_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(CategoryError::ObjectNotFound(object_id));
        }

        Ok(())
    }

    /// Get categories for object
    pub async fn get_object_categories(&self, object_id: Uuid) -> Result<Vec<Category>, CategoryError> {
        let rows = sqlx::query(
            r#"
            SELECT
                c.id, c.code, c.name, c.description, c.category_type, c.organization_id,
                c.parent_category_id, c.level, c.path, c.icon, c.color, c.sort_order,
                c.is_active, c.metadata, c.created_at, c.updated_at
            FROM categories c
            INNER JOIN object_categories oc ON c.id = oc.category_id
            WHERE oc.object_id = $1
            ORDER BY oc.is_primary DESC, c.sort_order, c.path
            "#,
        )
        .bind(object_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| Self::row_to_category(r)).collect()
    }

    // ========================================================================
    // Domain categorization operations
    // ========================================================================

    /// Add category to domain
    pub async fn categorize_domain(
        &self,
        domain_id: Uuid,
        category_id: Uuid,
        assigned_by: Uuid,
    ) -> Result<DomainCategory, CategoryError> {
        let row = sqlx::query(
            r#"
            INSERT INTO domain_categories (domain_id, category_id, assigned_by)
            VALUES ($1, $2, $3)
            ON CONFLICT (domain_id, category_id) DO NOTHING
            RETURNING id, domain_id, category_id, assigned_by, created_at
            "#,
        )
        .bind(domain_id)
        .bind(category_id)
        .bind(assigned_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(DomainCategory {
            id: row.get("id"),
            domain_id: row.get("domain_id"),
            category_id: row.get("category_id"),
            assigned_by: row.get("assigned_by"),
            created_at: row.get("created_at"),
        })
    }

    /// Get categories for domain
    pub async fn get_domain_categories(&self, domain_id: Uuid) -> Result<Vec<Category>, CategoryError> {
        let rows = sqlx::query(
            r#"
            SELECT
                c.id, c.code, c.name, c.description, c.category_type, c.organization_id,
                c.parent_category_id, c.level, c.path, c.icon, c.color, c.sort_order,
                c.is_active, c.metadata, c.created_at, c.updated_at
            FROM categories c
            INNER JOIN domain_categories dc ON c.id = dc.category_id
            WHERE dc.domain_id = $1
            ORDER BY c.sort_order, c.path
            "#,
        )
        .bind(domain_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| Self::row_to_category(r)).collect()
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get category statistics
    pub async fn get_category_stats(&self, category_id: Uuid) -> Result<CategoryStats, CategoryError> {
        let row = sqlx::query(
            r#"
            SELECT
                c.id as category_id, c.name as category_name, c.level as depth,
                COUNT(DISTINCT oc.object_id) as direct_objects_count,
                COUNT(DISTINCT child.id) as child_count,
                MAX(oc.assigned_at) as last_used
            FROM categories c
            LEFT JOIN object_categories oc ON c.id = oc.category_id
            LEFT JOIN categories child ON c.id = child.parent_category_id
            WHERE c.id = $1
            GROUP BY c.id, c.name, c.level
            "#,
        )
        .bind(category_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(CategoryError::CategoryNotFound(category_id))?;

        // Calculate total objects in subtree (including children recursively)
        let total_objects: i64 = sqlx::query(
            r#"
            WITH RECURSIVE category_tree AS (
                SELECT id FROM categories WHERE id = $1
                UNION ALL
                SELECT c.id FROM categories c
                INNER JOIN category_tree ct ON c.parent_category_id = ct.id
            )
            SELECT COUNT(DISTINCT oc.object_id)
            FROM category_tree ct
            LEFT JOIN object_categories oc ON ct.id = oc.category_id
            "#
        )
        .bind(category_id)
        .fetch_one(&self.pool)
        .await?
        .get("count");

        Ok(CategoryStats {
            category_id: row.get("category_id"),
            category_name: row.get("category_name"),
            direct_objects_count: row.get::<i32, _>("direct_objects_count"),
            total_objects_in_subtree: total_objects as i32,
            child_categories_count: row.get::<i32, _>("child_count"),
            depth: row.get::<i32, _>("depth"),
            last_used: row.get("last_used"),
        })
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    fn row_to_category(row: PgRow) -> Result<Category, CategoryError> {
        let category_type_str: String = row.get("category_type");
        let category_type = category_type_str.parse()
            .map_err(|_| CategoryError::InvalidCategoryType(category_type_str))?;

        Ok(Category {
            id: row.get("id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            category_type,
            organization_id: row.get("organization_id"),
            parent_category_id: row.get("parent_category_id"),
            level: row.get("level"),
            path: row.get("path"),
            icon: row.get("icon"),
            color: row.get("color"),
            sort_order: row.get("sort_order"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Check if adding a parent would create a cycle
    async fn would_create_cycle(&self, parent_id: Uuid) -> Result<bool, CategoryError> {
        let mut current = parent_id;
        let max_depth = 100;
        let mut depth = 0;

        while depth < max_depth {
            let parent = sqlx::query("SELECT parent_category_id FROM categories WHERE id = $1")
                .bind(current)
                .fetch_optional(&self.pool)
                .await?;

            match parent.and_then(|p| p.get::<Option<Uuid>, _>("parent_category_id")) {
                Some(pid) if pid == parent_id => return Ok(true),
                Some(pid) => current = pid,
                None => break,
            }
            depth += 1;
        }

        Ok(false)
    }

    /// Check if moving category to new parent would create a cycle
    async fn would_create_cycle_for_move(
        &self,
        category_id: Uuid,
        new_parent_id: Uuid,
    ) -> Result<bool, CategoryError> {
        let mut current = new_parent_id;
        let max_depth = 100;
        let mut depth = 0;

        while depth < max_depth {
            if current == category_id {
                return Ok(true);
            }

            let parent = sqlx::query("SELECT parent_category_id FROM categories WHERE id = $1")
                .bind(current)
                .fetch_optional(&self.pool)
                .await?;

            match parent.and_then(|p| p.get::<Option<Uuid>, _>("parent_category_id")) {
                Some(pid) => current = pid,
                None => break,
            }
            depth += 1;
        }

        Ok(false)
    }
}
