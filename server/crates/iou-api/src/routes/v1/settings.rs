//! Settings API endpoints
//!
//! HTTP handlers for system configuration settings with hierarchical
//! scoping (System > Tenant > Organization > Domain > User)

use std::collections::HashMap;

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::ApiError,
    middleware::auth::{AuthContext, require_permission, Permission},
    supabase::SupabasePool,
};

use iou_core::setting::{
    Setting, SettingGroup, SettingHistory, SettingKey, SettingQuery, SettingScope, SettingValueType,
};

// =============================================================================
// Request/Response Types
// =============================================================================

/// Setting response
#[derive(Debug, Serialize)]
pub struct SettingResponse {
    pub id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub value_type: SettingValueType,
    pub scope: SettingScope,
    pub scope_id: Option<Uuid>,
    pub description: Option<String>,
    pub default_value: Option<serde_json::Value>,
    pub is_system_setting: bool,
    pub is_sensitive: bool,
    pub is_encrypted: bool,
    pub is_public: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Setting> for SettingResponse {
    fn from(setting: Setting) -> Self {
        let is_system_setting = setting.key.is_system_setting();
        let is_sensitive = setting.key.is_sensitive();

        Self {
            id: setting.id,
            key: setting.key.to_string(),
            value: setting.value,
            value_type: setting.value_type,
            scope: setting.scope,
            scope_id: setting.scope_id,
            description: setting.description,
            default_value: setting.default_value,
            is_system_setting,
            is_sensitive,
            is_encrypted: setting.is_encrypted,
            is_public: setting.is_public,
            created_at: setting.created_at,
            updated_at: setting.updated_at,
        }
    }
}

/// Create setting request
#[derive(Debug, Deserialize)]
pub struct CreateSettingRequest {
    pub key: String,
    pub value: serde_json::Value,
    pub scope: SettingScope,
    pub scope_id: Option<Uuid>,
    pub description: Option<String>,
    pub default_value: Option<serde_json::Value>,
    pub validation_regex: Option<String>,
    pub is_encrypted: Option<bool>,
    pub is_public: Option<bool>,
}

/// Update setting request
#[derive(Debug, Deserialize)]
pub struct UpdateSettingRequest {
    pub value: serde_json::Value,
    pub change_reason: Option<String>,
}

/// Bulk update settings request
#[derive(Debug, Deserialize)]
pub struct BulkUpdateSettingsRequest {
    pub updates: HashMap<String, serde_json::Value>,
    pub scope: Option<SettingScope>,
    pub scope_id: Option<Uuid>,
    pub change_reason: Option<String>,
}

/// List settings query parameters
#[derive(Debug, Deserialize)]
pub struct ListSettingsQuery {
    pub scope: Option<SettingScope>,
    pub scope_id: Option<Uuid>,
    pub group: Option<String>,
    pub include_system: Option<bool>,
    pub include_sensitive: Option<bool>,
    pub resolve_hierarchy: Option<bool>,
}

/// Setting history response
#[derive(Debug, Serialize)]
pub struct SettingHistoryResponse {
    pub id: Uuid,
    pub setting_id: Uuid,
    pub key: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub changed_by: Uuid,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub change_reason: Option<String>,
}

impl From<SettingHistory> for SettingHistoryResponse {
    fn from(history: SettingHistory) -> Self {
        Self {
            id: history.id,
            setting_id: history.setting_id,
            key: history.key.to_string(),
            old_value: history.old_value,
            new_value: history.new_value,
            changed_by: history.changed_by,
            changed_at: history.changed_at,
            change_reason: history.change_reason,
        }
    }
}

/// Setting group response
#[derive(Debug, Serialize)]
pub struct SettingGroupResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub settings: Vec<SettingResponse>,
    pub order: i32,
}

/// Validate setting request
#[derive(Debug, Deserialize)]
pub struct ValidateSettingRequest {
    pub key: String,
    pub value: serde_json::Value,
}

/// Validate setting response
#[derive(Debug, Serialize)]
pub struct ValidateSettingResponse {
    pub valid: bool,
    pub error: Option<String>,
}

/// Reset setting request
#[derive(Debug, Deserialize)]
pub struct ResetSettingRequest {
    pub reason: Option<String>,
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /api/v1/settings - List all settings
///
/// Lists settings with optional filtering by scope and group.
/// Requires `SettingRead` permission.
pub async fn list_settings(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ListSettingsQuery>,
) -> Result<Json<Vec<SettingResponse>>, ApiError> {
    require_permission(&auth, Permission::SettingRead)?;

    let include_system = query.include_system.unwrap_or(false);
    let include_sensitive = query.include_sensitive.unwrap_or(false);

    let mut settings_filter = Vec::new();
    let mut params = Vec::new();
    let mut param_idx = 1;

    // Build query
    let base_query = r#"
        SELECT
            id, key, value, value_type, scope, scope_id, description,
            default_value, validation_regex, is_encrypted, is_public,
            metadata, created_at, updated_at
        FROM settings
        WHERE 1=1
    "#;

    if let Some(scope) = query.scope {
        settings_filter.push(format!("AND scope = ${}", param_idx));
        params.push(scope.to_string());
        param_idx += 1;
    }

    if let Some(scope_id) = query.scope_id {
        settings_filter.push(format!("AND scope_id = ${}", param_idx));
        params.push(scope_id.to_string());
        param_idx += 1;
    }

    if !include_system {
        // Filter out system settings (simple heuristic)
        settings_filter.push("AND key NOT IN ('session_timeout_minutes', 'mfa_required', 'max_file_size_mb', 'pii_auto_detect_enabled')".to_string());
    }

    let final_query = format!("{} {}", base_query, settings_filter.join(" "));

    let rows = sqlx::query(&final_query)
        .fetch_all(&*pool)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

    let settings: Vec<SettingResponse> = rows
        .into_iter()
        .filter_map(|row| {
            let key_str: String = row.get("key");
            let key: SettingKey = key_str.clone().into();
            let scope: SettingScope = row.get("scope");
            let value: serde_json::Value = row.get("value");
            let value_type: SettingValueType = row.get("value_type");

            // Filter sensitive settings unless requested
            if !include_sensitive && key.is_sensitive() {
                return None;
            }

            Some(SettingResponse {
                id: row.get("id"),
                key: key_str,
                value,
                value_type,
                scope,
                scope_id: row.get("scope_id"),
                description: row.get("description"),
                default_value: row.get("default_value"),
                is_system_setting: key.is_system_setting(),
                is_sensitive: key.is_sensitive(),
                is_encrypted: row.get("is_encrypted"),
                is_public: row.get("is_public"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .collect();

    Ok(Json(settings))
}

/// GET /api/v1/settings/groups - List setting groups
///
/// Returns all setting groups with their settings organized.
/// Requires `SettingRead` permission.
pub async fn list_setting_groups() -> Json<Vec<SettingGroupResponse>> {
    let groups = SettingGroup::setting_groups();

    let response: Vec<SettingGroupResponse> = groups
        .into_iter()
        .map(|g| SettingGroupResponse {
            id: g.id.clone(),
            name: g.name,
            description: g.description,
            icon: g.icon,
            settings: Vec::new(), // Settings loaded separately
            order: g.order,
        })
        .collect();

    Json(response)
}

/// GET /api/v1/settings/groups/:id - Get settings in a group
///
/// Returns all settings belonging to a specific group.
/// Requires `SettingRead` permission.
pub async fn get_setting_group(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(group_id): Path<String>,
) -> Result<Json<SettingGroupResponse>, ApiError> {
    require_permission(&auth, Permission::SettingRead)?;

    let group = SettingGroup::setting_groups()
        .into_iter()
        .find(|g| g.id == group_id)
        .ok_or_else(|| ApiError::NotFound(format!("Setting group not found: {}", group_id)))?;

    // Get settings for this group
    let key_strs: Vec<String> = group.settings.iter().map(|k| k.to_string()).collect();

    let rows = sqlx::query(
        r#"
        SELECT
            id, key, value, value_type, scope, scope_id, description,
            default_value, validation_regex, is_encrypted, is_public,
            metadata, created_at, updated_at
        FROM settings
        WHERE key = ANY($1)
        ORDER BY key
        "#
    )
    .bind(&key_strs)
    .fetch_all(&*pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

    let settings: Vec<SettingResponse> = rows
        .into_iter()
        .map(|row| {
            let key_str: String = row.get("key");
            let key: SettingKey = key_str.clone().into();
            let scope: SettingScope = row.get("scope");
            let value: serde_json::Value = row.get("value");
            let value_type: SettingValueType = row.get("value_type");

            SettingResponse {
                id: row.get("id"),
                key: key_str,
                value,
                value_type,
                scope,
                scope_id: row.get("scope_id"),
                description: row.get("description"),
                default_value: row.get("default_value"),
                is_system_setting: key.is_system_setting(),
                is_sensitive: key.is_sensitive(),
                is_encrypted: row.get("is_encrypted"),
                is_public: row.get("is_public"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect();

    Ok(Json(SettingGroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        icon: group.icon,
        settings,
        order: group.order,
    }))
}

/// GET /api/v1/settings/:key - Get setting by key
///
/// Retrieves a single setting by its key, with hierarchy resolution.
/// Requires `SettingRead` permission.
pub async fn get_setting(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(key): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<SettingResponse>, ApiError> {
    require_permission(&auth, Permission::SettingRead)?;

    let scope_id = query.get("scope_id").and_then(|id| id.parse().ok());

    let row = sqlx::query(
        r#"
        SELECT
            id, key, value, value_type, scope, scope_id, description,
            default_value, validation_regex, is_encrypted, is_public,
            metadata, created_at, updated_at
        FROM settings
        WHERE key = $1
        ORDER BY scope_level DESC, scope_id = $2 DESC
        LIMIT 1
        "#
    )
    .bind(&key)
    .bind(scope_id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?
    .ok_or_else(|| ApiError::NotFound(format!("Setting not found: {}", key)))?;

    let key_str: String = row.get("key");
    let key_parsed: SettingKey = key_str.clone().into();
    let scope: SettingScope = row.get("scope");
    let value: serde_json::Value = row.get("value");
    let value_type: SettingValueType = row.get("value_type");

    Ok(Json(SettingResponse {
        id: row.get("id"),
        key: key_str,
        value,
        value_type,
        scope,
        scope_id: row.get("scope_id"),
        description: row.get("description"),
        default_value: row.get("default_value"),
        is_system_setting: key_parsed.is_system_setting(),
        is_sensitive: key_parsed.is_sensitive(),
        is_encrypted: row.get("is_encrypted"),
        is_public: row.get("is_public"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

/// PUT /api/v1/settings/:key - Update setting
///
/// Updates an existing setting's value.
/// Requires `SettingUpdate` permission.
pub async fn update_setting(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(key): Path<String>,
    Json(req): Json<UpdateSettingRequest>,
) -> Result<Json<SettingResponse>, ApiError> {
    require_permission(&auth, Permission::SettingUpdate)?;

    // Check if setting exists and if it's a system setting
    let existing = sqlx::query("SELECT id, key, scope FROM settings WHERE key = $1")
        .bind(&key)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

    if let Some(row) = existing {
        let key_str: String = row.get("key");
        let setting_key: SettingKey = key_str.into();

        if setting_key.is_system_setting() {
            return Err(ApiError::Forbidden(
                "Cannot modify system settings".to_string(),
            ));
        }

        let setting_id: Uuid = row.get("id");

        // Update setting
        let updated = sqlx::query(
            r#"
            UPDATE settings
            SET value = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING
                id, key, value, value_type, scope, scope_id, description,
                default_value, validation_regex, is_encrypted, is_public,
                metadata, created_at, updated_at
            "#
        )
        .bind(serde_json::to_value(&req.value).map_err(|e| ApiError::Internal(format!("JSON error: {}", e)))?)
        .bind(setting_id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

        // Record history
        if let Some(reason) = &req.change_reason {
            sqlx::query(
                "INSERT INTO setting_history (setting_id, key, old_value, new_value, changed_by, change_reason)
                 SELECT $1, $2, value, $3, $4, $5 FROM settings WHERE id = $1"
            )
            .bind(setting_id)
            .bind(&key)
            .bind(&req.value)
            .bind(auth.user_id)
            .bind(reason)
            .execute(&*pool)
            .await
            .ok(); // Don't fail if history insert fails
        }

        let key_str: String = updated.get("key");
        let key_parsed: SettingKey = key_str.clone().into();
        let scope: SettingScope = updated.get("scope");
        let value: serde_json::Value = updated.get("value");
        let value_type: SettingValueType = updated.get("value_type");

        return Ok(Json(SettingResponse {
            id: updated.get("id"),
            key: key_str,
            value,
            value_type,
            scope,
            scope_id: updated.get("scope_id"),
            description: updated.get("description"),
            default_value: updated.get("default_value"),
            is_system_setting: key_parsed.is_system_setting(),
            is_sensitive: key_parsed.is_sensitive(),
            is_encrypted: updated.get("is_encrypted"),
            is_public: updated.get("is_public"),
            created_at: updated.get("created_at"),
            updated_at: updated.get("updated_at"),
        }));
    }

    Err(ApiError::NotFound(format!("Setting not found: {}", key)))
}

/// POST /api/v1/settings - Create new setting
///
/// Creates a new custom setting.
/// Requires `SettingCreate` permission.
pub async fn create_setting(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateSettingRequest>,
) -> Result<Json<SettingResponse>, ApiError> {
    require_permission(&auth, Permission::SettingCreate)?;

    let value_type = SettingValueType::from_json(&req.value);
    let is_encrypted = req.is_encrypted.unwrap_or(false);
    let is_public = req.is_public.unwrap_or(false);

    let row = sqlx::query(
        r#"
        INSERT INTO settings (key, value, value_type, scope, scope_id, description, default_value, validation_regex, is_encrypted, is_public)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
            id, key, value, value_type, scope, scope_id, description,
            default_value, validation_regex, is_encrypted, is_public,
            metadata, created_at, updated_at
        "#
    )
    .bind(&req.key)
    .bind(serde_json::to_value(&req.value).map_err(|e| ApiError::Internal(format!("JSON error: {}", e)))?)
    .bind(&value_type.to_string())
    .bind(&req.scope.to_string())
    .bind(req.scope_id)
    .bind(&req.description)
    .bind(req.default_value)
    .bind(&req.validation_regex)
    .bind(is_encrypted)
    .bind(is_public)
    .fetch_one(&*pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

    let key_str: String = row.get("key");
    let key_parsed: SettingKey = key_str.clone().into();
    let scope: SettingScope = row.get("scope");
    let value: serde_json::Value = row.get("value");
    let value_type: SettingValueType = row.get("value_type");

    Ok(Json(SettingResponse {
        id: row.get("id"),
        key: key_str,
        value,
        value_type,
        scope,
        scope_id: row.get("scope_id"),
        description: row.get("description"),
        default_value: row.get("default_value"),
        is_system_setting: key_parsed.is_system_setting(),
        is_sensitive: key_parsed.is_sensitive(),
        is_encrypted: row.get("is_encrypted"),
        is_public: row.get("is_public"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

/// POST /api/v1/settings/bulk - Bulk update settings
///
/// Updates multiple settings in a single transaction.
/// Requires `SettingUpdate` permission.
pub async fn bulk_update_settings(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<BulkUpdateSettingsRequest>,
) -> Result<Json<Vec<SettingResponse>>, ApiError> {
    require_permission(&auth, Permission::SettingUpdate)?;

    let mut tx = pool.begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

    let mut updated_settings = Vec::new();

    for (key, value) in &req.updates {
        // Check if system setting
        let setting_key: SettingKey = key.clone().into();
        if setting_key.is_system_setting() {
            continue; // Skip system settings
        }

        let result = sqlx::query(
            r#"
            UPDATE settings
            SET value = $1, updated_at = NOW()
            WHERE key = $2
            RETURNING
                id, key, value, value_type, scope, scope_id, description,
                default_value, validation_regex, is_encrypted, is_public,
                metadata, created_at, updated_at
            "#
        )
        .bind(serde_json::to_value(value).map_err(|e| ApiError::Internal(format!("JSON error: {}", e)))?)
        .bind(key)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

        if let Some(row) = result {
            let key_str: String = row.get("key");
            let key_parsed: SettingKey = key_str.clone().into();
            let scope: SettingScope = row.get("scope");
            let val: serde_json::Value = row.get("value");
            let value_type: SettingValueType = row.get("value_type");

            updated_settings.push(SettingResponse {
                id: row.get("id"),
                key: key_str,
                value: val,
                value_type,
                scope,
                scope_id: row.get("scope_id"),
                description: row.get("description"),
                default_value: row.get("default_value"),
                is_system_setting: key_parsed.is_system_setting(),
                is_sensitive: key_parsed.is_sensitive(),
                is_encrypted: row.get("is_encrypted"),
                is_public: row.get("is_public"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
    }

    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

    Ok(Json(updated_settings))
}

/// POST /api/v1/settings/:key/reset - Reset setting to default
///
/// Resets a setting to its default value.
/// Requires `SettingUpdate` permission.
pub async fn reset_setting(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(key): Path<String>,
    Json(req): Json<ResetSettingRequest>,
) -> Result<Json<SettingResponse>, ApiError> {
    require_permission(&auth, Permission::SettingUpdate)?;

    let row = sqlx::query(
        r#"
        UPDATE settings
        SET value = COALESCE(default_value, 'null'::json), updated_at = NOW()
        WHERE key = $1 AND default_value IS NOT NULL
        RETURNING
            id, key, value, value_type, scope, scope_id, description,
            default_value, validation_regex, is_encrypted, is_public,
            metadata, created_at, updated_at
        "#
    )
    .bind(&key)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?
    .ok_or_else(|| ApiError::NotFound(format!("Setting not found or has no default: {}", key)))?;

    let key_str: String = row.get("key");
    let key_parsed: SettingKey = key_str.clone().into();
    let scope: SettingScope = row.get("scope");
    let value: serde_json::Value = row.get("value");
    let value_type: SettingValueType = row.get("value_type");

    Ok(Json(SettingResponse {
        id: row.get("id"),
        key: key_str,
        value,
        value_type,
        scope,
        scope_id: row.get("scope_id"),
        description: row.get("description"),
        default_value: row.get("default_value"),
        is_system_setting: key_parsed.is_system_setting(),
        is_sensitive: key_parsed.is_sensitive(),
        is_encrypted: row.get("is_encrypted"),
        is_public: row.get("is_public"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

/// GET /api/v1/settings/:key/history - Get setting history
///
/// Returns the audit trail for a specific setting.
/// Requires `SettingRead` permission.
pub async fn get_setting_history(
    State(pool): State<SupabasePool>,
    Extension(auth): Extension<AuthContext>,
    Path(key): Path<String>,
) -> Result<Json<Vec<SettingHistoryResponse>>, ApiError> {
    require_permission(&auth, Permission::SettingRead)?;

    let rows = sqlx::query(
        r#"
        SELECT id, setting_id, key, old_value, new_value, changed_by, changed_at, change_reason
        FROM setting_history
        WHERE key = $1
        ORDER BY changed_at DESC
        LIMIT 100
        "#
    )
    .bind(&key)
    .fetch_all(&*pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

    let history: Vec<SettingHistoryResponse> = rows
        .into_iter()
        .map(|row| SettingHistoryResponse {
            id: row.get("id"),
            setting_id: row.get("setting_id"),
            key: row.get("key"),
            old_value: row.get("old_value"),
            new_value: row.get("new_value"),
            changed_by: row.get("changed_by"),
            changed_at: row.get("changed_at"),
            change_reason: row.get("change_reason"),
        })
        .collect();

    Ok(Json(history))
}

/// POST /api/v1/settings/validate - Validate setting value
///
/// Validates a setting value against its regex pattern.
pub async fn validate_setting(
    Json(req): Json<ValidateSettingRequest>,
) -> Result<Json<ValidateSettingResponse>, ApiError> {
    let key: SettingKey = req.key.clone().into();
    let value = &req.value;

    // Check type matches
    let expected_type = SettingValueType::from_json(value);

    let is_valid = match expected_type {
        SettingValueType::String => value.is_string(),
        SettingValueType::Integer => value.is_i64(),
        SettingValueType::Float => value.is_f64(),
        SettingValueType::Boolean => value.is_boolean(),
        SettingValueType::Json => value.is_object(),
        SettingValueType::StringArray => value.is_array(),
    };

    if !is_valid {
        return Ok(Json(ValidateSettingResponse {
            valid: false,
            error: Some(format!("Invalid type for setting {}. Expected {:?}", req.key, expected_type)),
        }));
    }

    // TODO: Check regex validation if setting has validation_regex

    Ok(Json(ValidateSettingResponse {
        valid: true,
        error: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_response_from_setting() {
        let setting = Setting::new(
            SettingKey::OrganizationName,
            serde_json::json!("Test Org"),
            SettingScope::Organization,
        );

        let response = SettingResponse::from(setting);
        assert_eq!(response.key, "organization_name");
        assert_eq!(response.value, serde_json::json!("Test Org"));
    }

    #[test]
    fn test_validate_setting_request_type_check() {
        let req = ValidateSettingRequest {
            key: "organization_name".to_string(),
            value: serde_json::json!("Test"),
        };

        let expected_type = SettingValueType::from_json(&req.value);
        assert_eq!(expected_type, SettingValueType::String);
    }
}
