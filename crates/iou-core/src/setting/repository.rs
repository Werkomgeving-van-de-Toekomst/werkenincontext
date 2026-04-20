//! Settings repository for CRUD operations
//!
//! Provides database operations for settings management including
//! hierarchical resolution, audit trail, and bulk operations.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, postgres::PgRow};
use thiserror::Error;
use uuid::Uuid;

use crate::setting::{
    Setting, SettingKey, SettingValueType, SettingScope,
    SettingHistory, SettingBulkUpdate, SettingUpdateItem,
    SettingQuery,
};

/// Errors that can occur during settings operations
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Setting not found: {0}")]
    SettingNotFound(String),

    #[error("Setting key already exists at this scope: {0}")]
    SettingAlreadyExists(String),

    #[error("Invalid setting value: {0}")]
    InvalidValue(String),

    #[error("Invalid setting scope: {0}")]
    InvalidScope(String),

    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Repository for settings operations
pub struct SettingsRepository {
    pool: PgPool,
}

impl SettingsRepository {
    /// Create a new SettingsRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Setting CRUD operations
    // ========================================================================

    /// Create or update a setting
    pub async fn upsert_setting(
        &self,
        key: SettingKey,
        value: serde_json::Value,
        scope: SettingScope,
        scope_id: Option<Uuid>,
        description: Option<String>,
        default_value: Option<serde_json::Value>,
        validation_regex: Option<String>,
        is_encrypted: Option<bool>,
        is_public: Option<bool>,
        changed_by: Uuid,
    ) -> Result<Setting, SettingsError> {
        let key_str = self.key_to_string(&key);
        let scope_str = self.scope_to_string(scope);
        let value_type = SettingValueType::from_json(&value);
        let value_type_str = self.value_type_to_string(value_type);

        // Validate against regex if provided
        if let Some(regex) = &validation_regex {
            let value_str = value.as_str()
                .ok_or_else(|| SettingsError::InvalidValue("Value must be a string for regex validation".to_string()))?;
            let re = regex::Regex::new(regex)
                .map_err(|e| SettingsError::InvalidValue(format!("Invalid regex: {}", e)))?;
            if !re.is_match(value_str) {
                return Err(SettingsError::InvalidValue(format!("Value does not match pattern: {}", regex)));
            }
        }

        let row = sqlx::query(
            r#"
            INSERT INTO settings (
                key, value, value_type, scope, scope_id,
                description, default_value, validation_regex,
                is_encrypted, is_public
            )
            VALUES ($1, $2, $3::VARCHAR, $4::VARCHAR, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (key, scope, scope_id) DO UPDATE SET
                value = EXCLUDED.value,
                value_type = EXCLUDED.value_type,
                description = COALESCE(EXCLUDED.description, settings.description),
                validation_regex = COALESCE(EXCLUDED.validation_regex, settings.validation_regex),
                is_encrypted = COALESCE(EXCLUDED.is_encrypted, settings.is_encrypted),
                is_public = COALESCE(EXCLUDED.is_public, settings.is_public),
                updated_at = NOW()
            RETURNING id, key, value, value_type, scope, scope_id, description,
                default_value, validation_regex, is_encrypted, is_public,
                created_at, updated_at
            "#,
        )
        .bind(&key_str)
        .bind(&value)
        .bind(&value_type_str)
        .bind(&scope_str)
        .bind(scope_id)
        .bind(description)
        .bind(&default_value)
        .bind(&validation_regex)
        .bind(is_encrypted.unwrap_or(false))
        .bind(is_public.unwrap_or(false))
        .fetch_one(&self.pool)
        .await?;

        // Log history
        self.log_history(row.get("id"), &value, changed_by).await?;

        Ok(Self::row_to_setting(row)?)
    }

    /// Get a setting by key and scope
    pub async fn get_setting(
        &self,
        key: &SettingKey,
        scope: SettingScope,
        scope_id: Option<Uuid>,
    ) -> Result<Setting, SettingsError> {
        let key_str = self.key_to_string(key);
        let scope_str = self.scope_to_string(scope);

        let row = sqlx::query(
            r#"
            SELECT id, key, value, value_type, scope, scope_id, description,
                default_value, validation_regex, is_encrypted, is_public,
                created_at, updated_at
            FROM settings
            WHERE key = $1 AND scope = $2 AND (scope_id = $3 OR scope_id IS NULL)
            ORDER BY scope_id DESC NULLS LAST
            LIMIT 1
            "#,
        )
        .bind(&key_str)
        .bind(&scope_str)
        .bind(scope_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| SettingsError::SettingNotFound(key_str))?;

        Ok(Self::row_to_setting(row)?)
    }

    /// Get setting with fallback to default
    pub async fn get_setting_with_fallback(
        &self,
        key: &SettingKey,
        scope: SettingScope,
        scope_id: Option<Uuid>,
    ) -> Result<Setting, SettingsError> {
        // Try specific scope first
        match self.get_setting(key, scope, scope_id).await {
            Ok(setting) => Ok(setting),
            Err(SettingsError::SettingNotFound(_)) => {
                // Fall back to global scope
                self.get_setting(key, SettingScope::System, None).await
            }
            Err(e) => Err(e),
        }
    }

    /// List settings by scope
    pub async fn list_settings(
        &self,
        scope: Option<SettingScope>,
        scope_id: Option<Uuid>,
    ) -> Result<Vec<Setting>, SettingsError> {
        let rows = if let Some(scope) = scope {
            let scope_str = self.scope_to_string(scope);
            sqlx::query(
                r#"
                SELECT id, key, value, value_type, scope, scope_id, description,
                    default_value, validation_regex, is_encrypted, is_public,
                    created_at, updated_at
                FROM settings
                WHERE scope = $1 AND (scope_id = $2 OR scope_id IS NULL)
                ORDER BY key, scope_id DESC NULLS LAST
                "#,
            )
            .bind(&scope_str)
            .bind(scope_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, key, value, value_type, scope, scope_id, description,
                    default_value, validation_regex, is_encrypted, is_public,
                    created_at, updated_at
                FROM settings
                ORDER BY scope, key, scope_id DESC NULLS LAST
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(|r| Self::row_to_setting(r)).collect()
    }

    /// Delete a setting
    pub async fn delete_setting(
        &self,
        key: &SettingKey,
        scope: SettingScope,
        scope_id: Option<Uuid>,
    ) -> Result<(), SettingsError> {
        let key_str = self.key_to_string(key);
        let scope_str = self.scope_to_string(scope);

        let result = sqlx::query(
            "DELETE FROM settings WHERE key = $1 AND scope = $2 AND scope_id = $3"
        )
        .bind(&key_str)
        .bind(&scope_str)
        .bind(scope_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(SettingsError::SettingNotFound(key_str));
        }

        Ok(())
    }

    // ========================================================================
    // History operations
    // ========================================================================

    /// Get setting history
    pub async fn get_history(&self, _setting_id: Uuid, _limit: i32) -> Result<Vec<SettingHistory>, SettingsError> {
        // TODO: Implement when setting_history table is properly created
        Ok(Vec::new())
    }

    // ========================================================================
    // Bulk operations
    // ========================================================================

    /// Bulk update settings
    pub async fn bulk_update(
        &self,
        updates: Vec<SettingUpdateItem>,
        changed_by: Uuid,
    ) -> Result<Vec<Setting>, SettingsError> {
        let mut results = Vec::new();

        for update in updates {
            let setting = self.upsert_setting(
                update.key,
                update.value,
                SettingScope::System,
                None,
                None,
                None,
                None,
                None,
                None,
                changed_by,
            ).await?;
            results.push(setting);
        }

        Ok(results)
    }

    // ========================================================================
    // Query operations
    // ========================================================================

    /// Query settings with filters
    pub async fn query(&self, query: SettingQuery) -> Result<Vec<Setting>, SettingsError> {
        // List settings by scope if provided
        let all_settings = self.list_settings(query.scope, query.scope_id).await?;

        // Filter by keys if provided
        let filtered: Vec<Setting> = if let Some(keys) = query.keys {
            all_settings
                .into_iter()
                .filter(|s| keys.contains(&s.key))
                .collect()
        } else {
            all_settings
        };

        Ok(filtered)
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    fn key_to_string(&self, key: &SettingKey) -> String {
        // Convert SettingKey to string using serialization
        serde_json::to_string(key).unwrap_or_else(|_| "unknown".to_string())
    }

    fn scope_to_string(&self, scope: SettingScope) -> String {
        match scope {
            SettingScope::System => "system".to_string(),
            SettingScope::Tenant => "tenant".to_string(),
            SettingScope::Organization => "organization".to_string(),
            SettingScope::Domain => "domain".to_string(),
            SettingScope::User => "user".to_string(),
        }
    }

    fn value_type_to_string(&self, value_type: SettingValueType) -> String {
        match value_type {
            SettingValueType::String => "string".to_string(),
            SettingValueType::Integer => "integer".to_string(),
            SettingValueType::Float => "float".to_string(),
            SettingValueType::Boolean => "boolean".to_string(),
            SettingValueType::Json => "json".to_string(),
            SettingValueType::StringArray => "string_array".to_string(),
        }
    }

    fn string_to_scope(&self, s: &str) -> Result<SettingScope, SettingsError> {
        match s {
            "system" => Ok(SettingScope::System),
            "tenant" => Ok(SettingScope::Tenant),
            "organization" => Ok(SettingScope::Organization),
            "domain" => Ok(SettingScope::Domain),
            "user" => Ok(SettingScope::User),
            _ => Err(SettingsError::InvalidScope(s.to_string())),
        }
    }

    fn string_to_value_type(&self, s: &str) -> Result<SettingValueType, SettingsError> {
        match s {
            "string" => Ok(SettingValueType::String),
            "integer" => Ok(SettingValueType::Integer),
            "float" => Ok(SettingValueType::Float),
            "boolean" => Ok(SettingValueType::Boolean),
            "json" => Ok(SettingValueType::Json),
            "string_array" => Ok(SettingValueType::StringArray),
            _ => Err(SettingsError::InvalidValue(format!("Invalid value type: {}", s))),
        }
    }

    fn row_to_setting(row: PgRow) -> Result<Setting, SettingsError> {
        let key_str: String = row.get("key");
        let key = SettingKey::from(key_str);

        let value_type_str: String = row.get("value_type");
        let value_type = value_type_str.parse()
            .unwrap_or(SettingValueType::String);

        let scope_str: String = row.get("scope");
        let scope = scope_str.parse()
            .unwrap_or(SettingScope::System);

        // Try to get metadata from row, default to empty object
        let metadata: serde_json::Value = row.try_get("metadata").unwrap_or_else(|_| serde_json::json!({}));

        Ok(Setting {
            id: row.get("id"),
            key,
            value: row.get("value"),
            value_type,
            scope,
            scope_id: row.get("scope_id"),
            description: row.get("description"),
            default_value: row.get("default_value"),
            validation_regex: row.get("validation_regex"),
            is_encrypted: row.get("is_encrypted"),
            is_public: row.get("is_public"),
            metadata,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn log_history(&self, _setting_id: Uuid, _new_value: &serde_json::Value, _changed_by: Uuid) -> Result<(), SettingsError> {
        // TODO: Implement when setting_history table is properly created
        Ok(())
    }
}
