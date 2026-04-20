//! Tag repository for CRUD operations
//!
//! Provides database operations for tag management including
//! creation, retrieval, update, deletion, and statistics.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, postgres::PgRow};
use thiserror::Error;
use uuid::Uuid;

use crate::tag::{
    Tag, TagType, ObjectTag, DomainTag,
    TagSuggestion, SuggestionReason, TagStats,
};

#[derive(Debug, Clone)]
struct ObjectTagRow {
    id: Uuid,
    object_id: Uuid,
    tag_id: Uuid,
    tagged_by: Uuid,
    confidence: f32,
    is_auto_assigned: bool,
    created_at: DateTime<Utc>,
}

impl ObjectTagRow {
    fn into_object_tag(self) -> ObjectTag {
        ObjectTag {
            id: self.id,
            object_id: self.object_id,
            tag_id: self.tag_id,
            tagged_by: self.tagged_by,
            confidence: self.confidence,
            is_auto_assigned: self.is_auto_assigned,
            created_at: self.created_at,
        }
    }
}

/// Errors that can occur during tag operations
#[derive(Debug, Error)]
pub enum TagError {
    #[error("Tag not found: {0}")]
    TagNotFound(Uuid),

    #[error("Tag already exists: {0}")]
    TagAlreadyExists(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(Uuid),

    #[error("Domain not found: {0}")]
    DomainNotFound(Uuid),

    #[error("Invalid tag type: {0}")]
    InvalidTagType(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Repository for tag operations
pub struct TagRepository {
    pool: PgPool,
}

impl TagRepository {
    /// Create a new TagRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Tag CRUD operations
    // ========================================================================

    /// Create a new tag
    pub async fn create_tag(
        &self,
        name: String,
        tag_type: TagType,
        organization_id: Option<Uuid>,
        description: Option<String>,
        color: Option<String>,
        parent_tag_id: Option<Uuid>,
        is_system_tag: bool,
    ) -> Result<Tag, TagError> {
        let tag_type_str = tag_type.to_string();

        let row = sqlx::query(
            r#"
            INSERT INTO tags (name, tag_type, organization_id, description, color, parent_tag_id, is_system_tag)
            VALUES ($1, $2::VARCHAR, $3, $4, $5, $6, $7)
            RETURNING
                id, name, normalized_name, tag_type, organization_id,
                description, color, parent_tag_id,
                usage_count, is_system_tag,
                created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(&tag_type_str)
        .bind(organization_id)
        .bind(description)
        .bind(color.as_deref())
        .bind(parent_tag_id)
        .bind(is_system_tag)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::row_to_tag(row)?)
    }

    /// Get tag by ID
    pub async fn get_tag(&self, tag_id: Uuid) -> Result<Tag, TagError> {
        let row = sqlx::query(
            r#"
            SELECT
                id, name, normalized_name, tag_type, organization_id,
                description, color, parent_tag_id,
                usage_count, is_system_tag,
                created_at, updated_at
            FROM tags
            WHERE id = $1
            "#,
        )
        .bind(tag_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(TagError::TagNotFound(tag_id))?;

        Ok(Self::row_to_tag(row)?)
    }

    /// Get tag by normalized name
    pub async fn get_tag_by_name(&self, normalized_name: &str) -> Result<Tag, TagError> {
        let row = sqlx::query(
            r#"
            SELECT
                id, name, normalized_name, tag_type, organization_id,
                description, color, parent_tag_id,
                usage_count, is_system_tag,
                created_at, updated_at
            FROM tags
            WHERE normalized_name = $1
            "#,
        )
        .bind(normalized_name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| TagError::TagNotFound(Uuid::nil()))?;

        Ok(Self::row_to_tag(row)?)
    }

    /// List tags by type
    pub async fn list_tags(
        &self,
        tag_type: Option<TagType>,
        organization_id: Option<Uuid>,
        include_system: bool,
    ) -> Result<Vec<Tag>, TagError> {
        let rows = if let Some(tag_type) = tag_type {
            let tag_type_str = tag_type.to_string();
            sqlx::query(
                r#"
                SELECT
                    id, name, normalized_name, tag_type, organization_id,
                    description, color, parent_tag_id,
                    usage_count, is_system_tag,
                    created_at, updated_at
                FROM tags
                WHERE tag_type = $1 AND (is_system_tag = TRUE OR $2 = TRUE)
                ORDER BY usage_count DESC, name
                "#,
            )
            .bind(&tag_type_str)
            .bind(include_system)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT
                    id, name, normalized_name, tag_type, organization_id,
                    description, color, parent_tag_id,
                    usage_count, is_system_tag,
                    created_at, updated_at
                FROM tags
                WHERE is_system_tag = FALSE OR $1 = TRUE
                ORDER BY tag_type, usage_count DESC, name
                "#,
            )
            .bind(include_system)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(|r| Self::row_to_tag(r)).collect()
    }

    /// Search tags by name
    pub async fn search_tags(&self, search_term: &str, limit: i32) -> Result<Vec<Tag>, TagError> {
        let pattern = format!("%{}%", search_term);

        let rows = sqlx::query(
            r#"
            SELECT
                id, name, normalized_name, tag_type, organization_id,
                description, color, parent_tag_id,
                usage_count, is_system_tag,
                created_at, updated_at
            FROM tags
            WHERE name ILIKE $1 OR normalized_name ILIKE $1
            ORDER BY usage_count DESC, name
            LIMIT $2
            "#,
        )
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| Self::row_to_tag(r)).collect()
    }

    /// Update tag
    pub async fn update_tag(
        &self,
        tag_id: Uuid,
        name: Option<String>,
        description: Option<Option<String>>,
        color: Option<Option<String>>,
    ) -> Result<Tag, TagError> {
        let current = self.get_tag(tag_id).await?;

        let new_name = name.unwrap_or(current.name);
        let new_description = description.unwrap_or(current.description);
        let new_color = color.unwrap_or(current.color);

        let row = sqlx::query(
            r#"
            UPDATE tags
            SET name = $1, description = $2, color = $3
            WHERE id = $4
            RETURNING
                id, name, normalized_name, tag_type, organization_id,
                description, color, parent_tag_id,
                usage_count, is_system_tag,
                created_at, updated_at
            "#,
        )
        .bind(&new_name)
        .bind(new_description)
        .bind(new_color.as_deref())
        .bind(tag_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::row_to_tag(row)?)
    }

    /// Delete tag (cascades to object_tags and domain_tags)
    pub async fn delete_tag(&self, tag_id: Uuid) -> Result<(), TagError> {
        let result = sqlx::query("DELETE FROM tags WHERE id = $1")
            .bind(tag_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TagError::TagNotFound(tag_id));
        }

        Ok(())
    }

    // ========================================================================
    // Object tagging operations
    // ========================================================================

    /// Add tag to object
    pub async fn tag_object(
        &self,
        object_id: Uuid,
        tag_id: Uuid,
        tagged_by: Uuid,
    ) -> Result<ObjectTag, TagError> {
        // Verify object exists
        let object_exists = sqlx::query("SELECT 1 FROM information_objects WHERE id = $1")
            .bind(object_id)
            .fetch_optional(&self.pool)
            .await?;

        if object_exists.is_none() {
            return Err(TagError::ObjectNotFound(object_id));
        }

        // Increment tag usage
        sqlx::query("UPDATE tags SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(tag_id)
            .execute(&self.pool)
            .await?;

        let row = sqlx::query(
            r#"
            INSERT INTO object_tags (object_id, tag_id, tagged_by)
            VALUES ($1, $2, $3)
            ON CONFLICT (object_id, tag_id) DO NOTHING
            RETURNING id, object_id, tag_id, tagged_by, created_at
            "#,
        )
        .bind(object_id)
        .bind(tag_id)
        .bind(tagged_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(ObjectTag {
            id: row.get("id"),
            object_id: row.get("object_id"),
            tag_id: row.get("tag_id"),
            tagged_by: row.get("tagged_by"),
            confidence: 1.0,
            is_auto_assigned: false,
            created_at: row.get("created_at"),
        })
    }

    /// Remove tag from object
    pub async fn untag_object(&self, object_id: Uuid, tag_id: Uuid) -> Result<(), TagError> {
        // Get tag_id for usage count update
        let result = sqlx::query("DELETE FROM object_tags WHERE object_id = $1 AND tag_id = $2")
            .bind(object_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() > 0 {
            // Decrement tag usage
            sqlx::query("UPDATE tags SET usage_count = usage_count - 1 WHERE id = $1")
                .bind(tag_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Get tags for object
    pub async fn get_object_tags(&self, object_id: Uuid) -> Result<Vec<Tag>, TagError> {
        let rows = sqlx::query(
            r#"
            SELECT
                t.id, t.name, t.normalized_name, t.tag_type, t.organization_id,
                t.description, t.color, t.parent_tag_id,
                t.usage_count, t.is_system_tag,
                t.created_at, t.updated_at
            FROM tags t
            INNER JOIN object_tags ot ON t.id = ot.tag_id
            WHERE ot.object_id = $1
            ORDER BY t.usage_count DESC, t.name
            "#,
        )
        .bind(object_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| Self::row_to_tag(r)).collect()
    }

    // ========================================================================
    // Domain tagging operations
    // ========================================================================

    /// Add tag to domain
    pub async fn tag_domain(
        &self,
        domain_id: Uuid,
        tag_id: Uuid,
        tagged_by: Uuid,
    ) -> Result<DomainTag, TagError> {
        let row = sqlx::query(
            r#"
            INSERT INTO domain_tags (domain_id, tag_id, tagged_by)
            VALUES ($1, $2, $3)
            ON CONFLICT (domain_id, tag_id) DO NOTHING
            RETURNING id, domain_id, tag_id, tagged_by, created_at
            "#,
        )
        .bind(domain_id)
        .bind(tag_id)
        .bind(tagged_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(DomainTag {
            id: row.get("id"),
            domain_id: row.get("domain_id"),
            tag_id: row.get("tag_id"),
            tagged_by: row.get("tagged_by"),
            created_at: row.get("created_at"),
        })
    }

    /// Get tags for domain
    pub async fn get_domain_tags(&self, domain_id: Uuid) -> Result<Vec<Tag>, TagError> {
        let rows = sqlx::query(
            r#"
            SELECT
                t.id, t.name, t.normalized_name, t.tag_type, t.organization_id,
                t.description, t.color, t.parent_tag_id,
                t.usage_count, t.is_system_tag,
                t.created_at, t.updated_at
            FROM tags t
            INNER JOIN domain_tags dt ON t.id = dt.tag_id
            WHERE dt.domain_id = $1
            ORDER BY t.usage_count DESC, t.name
            "#,
        )
        .bind(domain_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| Self::row_to_tag(r)).collect()
    }

    // ========================================================================
    // Statistics and suggestions
    // ========================================================================

    /// Get tag statistics
    pub async fn get_tag_stats(&self, tag_id: Uuid) -> Result<TagStats, TagError> {
        let row = sqlx::query(
            r#"
            SELECT
                t.id as tag_id, t.name as tag_name, t.usage_count,
                COUNT(DISTINCT ot.object_id) as object_count
            FROM tags t
            LEFT JOIN object_tags ot ON t.id = ot.tag_id
            WHERE t.id = $1
            GROUP BY t.id, t.name, t.usage_count
            "#,
        )
        .bind(tag_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(TagError::TagNotFound(tag_id))?;

        Ok(TagStats {
            tag_id: row.get("tag_id"),
            tag_name: row.get("tag_name"),
            usage_count: row.get("usage_count"),
            objects_count: row.get::<i32, _>("object_count"),
            domains_count: 0, // TODO: implement domain count
            last_used: None,
            trending_score: 0.0,
        })
    }

    /// Get popular tags
    pub async fn get_popular_tags(&self, limit: Option<i32>) -> Result<Vec<TagStats>, TagError> {
        let limit = limit.unwrap_or(20);

        let rows = sqlx::query(
            r#"
            SELECT
                t.id as tag_id, t.name as tag_name, t.usage_count,
                COUNT(DISTINCT ot.object_id) as object_count,
                MAX(ot.created_at) as last_used
            FROM tags t
            LEFT JOIN object_tags ot ON t.id = ot.tag_id
            WHERE t.is_system_tag = FALSE
            GROUP BY t.id, t.name, t.usage_count
            ORDER BY t.usage_count DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| {
            Ok(TagStats {
                tag_id: r.get("tag_id"),
                tag_name: r.get("tag_name"),
                usage_count: r.get("usage_count"),
                objects_count: r.get::<i32, _>("object_count"),
                domains_count: 0,
                last_used: r.get("last_used"),
                trending_score: 0.0,
            })
        }).collect()
    }

    /// Get trending tags (tags with recent activity)
    pub async fn get_trending_tags(&self, limit: i32, days: i32) -> Result<Vec<Tag>, TagError> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT t.id, t.name, t.normalized_name, t.tag_type, t.organization_id,
                t.description, t.color, t.parent_tag_id,
                t.usage_count, t.is_system_tag,
                t.created_at, t.updated_at
            FROM tags t
            INNER JOIN object_tags ot ON t.id = ot.tag_id
            WHERE ot.created_at > NOW() - INTERVAL '1 day' * $1
            ORDER BY t.usage_count DESC
            LIMIT $2
            "#,
        )
        .bind(days)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| Self::row_to_tag(r)).collect()
    }

    /// Get tag suggestions for an object
    pub async fn get_tag_suggestions(
        &self,
        domain_id: Option<Uuid>,
        limit: i32,
    ) -> Result<Vec<TagSuggestion>, TagError> {
        let suggestions = if let Some(domain_id) = domain_id {
            // Suggest tags already used in this domain
            let rows = sqlx::query(
                r#"
                SELECT DISTINCT
                    t.id, t.name, t.usage_count,
                    COUNT(DISTINCT ot2.object_id) as domain_object_count
                FROM tags t
                INNER JOIN domain_tags dt ON t.id = dt.tag_id
                INNER JOIN object_tags ot2 ON t.id = ot2.tag_id
                WHERE dt.domain_id = $1
                GROUP BY t.id, t.name, t.usage_count
                ORDER BY domain_object_count DESC
                LIMIT $2
                "#,
            )
            .bind(domain_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

            rows.into_iter().map(|r| {
                TagSuggestion {
                    tag_id: r.get("id"),
                    tag_name: r.get("name"),
                    reason: SuggestionReason::FrequentlyUsed,
                    confidence: 0.8,
                }
            }).collect()
        } else {
            // Suggest popular tags globally
            let stats = self.get_popular_tags(Some(limit)).await?;
            stats.into_iter().map(|s| TagSuggestion {
                tag_id: s.tag_id,
                tag_name: s.tag_name,
                reason: SuggestionReason::FrequentlyUsed,
                confidence: 0.5,
            }).collect()
        };

        Ok(suggestions)
    }

    /// Merge one tag into another
    pub async fn merge_tags(&self, source_id: Uuid, target_id: Uuid) -> Result<(), TagError> {
        // Update all object_tags
        sqlx::query(
            "UPDATE object_tags SET tag_id = $1 WHERE tag_id = $2"
        )
        .bind(target_id)
        .bind(source_id)
        .execute(&self.pool)
        .await?;

        // Update all domain_tags
        sqlx::query(
            "UPDATE domain_tags SET tag_id = $1 WHERE tag_id = $2"
        )
        .bind(target_id)
        .bind(source_id)
        .execute(&self.pool)
        .await?;

        // Delete source tag
        self.delete_tag(source_id).await?;

        Ok(())
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    fn row_to_tag(row: PgRow) -> Result<Tag, TagError> {
        let tag_type_str: String = row.get("tag_type");
        let tag_type = tag_type_str.parse()
            .map_err(|_| TagError::InvalidTagType(tag_type_str))?;

        Ok(Tag {
            id: row.get("id"),
            name: row.get("name"),
            normalized_name: row.get("normalized_name"),
            tag_type,
            organization_id: row.get("organization_id"),
            description: row.get("description"),
            color: row.get("color"),
            parent_tag_id: row.get("parent_tag_id"),
            usage_count: row.get("usage_count"),
            is_system_tag: row.get("is_system_tag"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}
