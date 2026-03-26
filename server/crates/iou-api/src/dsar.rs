//! Data Subject Rights Database Operations
//!
//! Supabase/PostgreSQL implementation for AVG/GDPR Articles 15, 16, 17
//! and Woo publication workflow.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, postgres::types::PgInterval, FromRow, Row};
use uuid::Uuid;

// ============================================
// Shared Enums and Types
// ============================================

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SarType {
    Full,
    Partial,
    Specific,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SarFormat {
    Json,
    Csv,
    Pdf,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ErasureType {
    Anonymization,
    Deletion,
    Pseudonymization,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PublicationPlatform {
    Rijksoverheid,
    OverheidNl,
    Gemeente,
    Provincie,
    Waterschap,
    Custom,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WooRequestType {
    Information,
    Documents,
    Consultation,
    Other,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequesterType {
    Citizen,
    Organization,
    Journalist,
    Government,
    Other,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WooPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WooRefusalGround {
    Privacy,
    NationalSecurity,
    CommercialConfidence,
    OngoingInvestigation,
    InternationalRelations,
    None,
}

// ============================================
// Database Row Types
// ============================================

/// Database row for Subject Access Request
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubjectAccessRequestRow {
    pub id: Uuid,
    pub requesting_user_id: Uuid,
    pub request_type: String,
    pub status: String,
    pub requested_fields: Option<Vec<String>>,
    pub response_data: Option<Value>,
    pub error_message: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Database row for Rectification Request
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RectificationRequestRow {
    pub id: Uuid,
    pub requesting_user_id: Uuid,
    pub object_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub justification: Option<String>,
    pub supporting_documents: Option<Vec<String>>,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Database row for Erasure Request
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ErasureRequestRow {
    pub id: Uuid,
    pub requesting_user_id: Uuid,
    pub object_id: Uuid,
    pub erasure_type: String,
    pub legal_basis: Option<String>,
    pub retention_check: bool,
    pub justification: Option<String>,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Response containing all pending DSAR requests
#[derive(Debug, Serialize)]
pub struct PendingDsarResponse {
    pub sar: Vec<SubjectAccessRequestRow>,
    pub rectifications: Vec<RectificationRequestRow>,
    pub erasures: Vec<ErasureRequestRow>,
}

/// Database row for Woo Publication
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WooPublicationRow {
    pub id: Uuid,
    pub object_id: Uuid,
    pub publication_platform: String,
    pub publication_status: String,
    pub category_ids: Option<Vec<String>>,
    pub legal_basis: Option<String>,
    pub publication_summary: Option<String>,
    pub consultation_required: bool,
    pub consultation_completed_at: Option<DateTime<Utc>>,
    pub redactions: Option<Value>,
    pub refusal_ground: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub publication_url: Option<String>,
    pub doi: Option<String>,
    pub imposition_reference: Option<String>,
    pub publicatie_nr: Option<String>,
    pub woo_publication_date: Option<chrono::NaiveDate>,
    pub withdrawn_at: Option<DateTime<Utc>>,
    pub withdrawal_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row for Woo Request (active Woo verzoeken)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WooRequestRow {
    pub id: Uuid,
    pub reference_number: String,
    pub requester_name: String,
    pub requester_email: String,
    pub requester_address: Option<String>,
    pub requester_type: String,
    pub request_type: String,
    pub title: String,
    pub description: String,
    pub requested_information: Option<Vec<String>>,
    pub priority: String,
    pub request_status: String,
    pub assigned_to: Option<Uuid>,
    pub decision: Option<String>,
    pub refusal_grounds: Option<Vec<String>>,
    pub decision_date: Option<chrono::NaiveDate>,
    pub decision_due_date: chrono::NaiveDate,
    pub consultation_extension: Option<i32>,
    pub documents_provided: Option<Vec<String>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Woo Statistics Response
#[derive(Debug, Serialize)]
pub struct WooStatistics {
    pub total_requests: i64,
    pub pending_publication: i64,
    pub published_count: i64,
    pub avg_processing_days: f64,
    pub overdue_requests: i64,
    pub upcoming_deadlines: Vec<WooDeadlineSummary>,
}

#[derive(Debug, Serialize)]
pub struct WooDeadlineSummary {
    pub request_id: Uuid,
    pub reference_number: String,
    pub title: String,
    pub days_until_due: i64,
}

/// Redaction for sensitive information in Woo publications
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Redaction {
    pub field_name: String,
    pub reason: String,
    pub position: Option<(usize, usize)>,
}

// ============================================
// DSAR Database Repository
// ============================================

pub struct DsarRepository {
    pool: PgPool,
}

impl DsarRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============================================
    // Subject Access Requests (SAR)
    // ============================================

    pub async fn create_sar(
        &self,
        id: Uuid,
        user_id: Uuid,
        request_type: SarType,
        requested_fields: Option<Vec<String>>,
        format: SarFormat,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Result<()> {
        let request_type_str = match request_type {
            SarType::Full => "full",
            SarType::Partial => "partial",
            SarType::Specific => "specific",
        };
        let format_str = match format {
            SarFormat::Json => "json",
            SarFormat::Csv => "csv",
            SarFormat::Pdf => "pdf",
        };

        sqlx::query(
            r#"
            INSERT INTO subject_access_requests
                (id, requesting_user_id, subject_user_id, request_type, status,
                 requested_fields, response_format, expires_at, created_at)
            VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(user_id) // subject_user_id = requesting_user_id
        .bind(request_type_str)
        .bind(requested_fields)
        .bind(format_str)
        .bind(expires_at)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        tracing::info!("Created SAR request: id={}, user_id={}", id, user_id);
        Ok(())
    }

    pub async fn get_sar(&self, id: Uuid) -> Result<Option<SubjectAccessRequestRow>> {
        let row = sqlx::query_as::<sqlx::Postgres, SubjectAccessRequestRow>(
            "SELECT id, requesting_user_id, request_type, status, requested_fields,
                   response_data, error_message, expires_at, completed_at, created_at
            FROM subject_access_requests
            WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_user_sar(
        &self,
        user_id: Uuid,
        status_filter: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<SubjectAccessRequestRow>> {
        let rows = if let Some(status) = status_filter {
            sqlx::query_as::<sqlx::Postgres, SubjectAccessRequestRow>(
                "SELECT id, requesting_user_id, request_type, status, requested_fields,
                       response_data, error_message, expires_at, completed_at, created_at
                FROM subject_access_requests
                WHERE requesting_user_id = $1 AND status = $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4"
            )
            .bind(user_id)
            .bind(status)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<sqlx::Postgres, SubjectAccessRequestRow>(
                "SELECT id, requesting_user_id, request_type, status, requested_fields,
                       response_data, error_message, expires_at, completed_at, created_at
                FROM subject_access_requests
                WHERE requesting_user_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3"
            )
            .bind(user_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows)
    }

    // ============================================
    // Rectification Requests
    // ============================================

    pub async fn create_rectification(
        &self,
        id: Uuid,
        user_id: Uuid,
        object_id: Uuid,
        field_name: &str,
        old_value: Option<String>,
        new_value: &str,
        justification: Option<String>,
        supporting_documents: Option<Vec<String>>,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO data_rectification_requests
                (id, requesting_user_id, object_id, field_name, old_value, new_value,
                 justification, supporting_documents, status, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', $9, $10)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(object_id)
        .bind(field_name)
        .bind(old_value)
        .bind(new_value)
        .bind(justification)
        .bind(supporting_documents)
        .bind(expires_at)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        tracing::info!("Created rectification request: id={}, object_id={}, field={}", id, object_id, field_name);
        Ok(())
    }

    pub async fn get_rectification(&self, id: Uuid) -> Result<Option<RectificationRequestRow>> {
        let row = sqlx::query_as::<sqlx::Postgres, RectificationRequestRow>(
            "SELECT id, requesting_user_id, object_id, field_name, old_value, new_value,
                   justification, status, reviewed_by, reviewed_at, expires_at, created_at
            FROM data_rectification_requests
            WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_user_rectifications(
        &self,
        user_id: Uuid,
        status_filter: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<RectificationRequestRow>> {
        let rows = if let Some(status) = status_filter {
            sqlx::query_as::<sqlx::Postgres, RectificationRequestRow>(
                "SELECT id, requesting_user_id, object_id, field_name, old_value, new_value,
                       justification, status, reviewed_by, reviewed_at, expires_at, created_at
                FROM data_rectification_requests
                WHERE requesting_user_id = $1 AND status = $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4"
            )
            .bind(user_id)
            .bind(status)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<sqlx::Postgres, RectificationRequestRow>(
                "SELECT id, requesting_user_id, object_id, field_name, old_value, new_value,
                       justification, status, reviewed_by, reviewed_at, expires_at, created_at
                FROM data_rectification_requests
                WHERE requesting_user_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3"
            )
            .bind(user_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows)
    }

    pub async fn approve_rectification(
        &self,
        id: Uuid,
        reviewer_id: Uuid,
        approved: bool,
        notes: Option<String>,
    ) -> Result<()> {
        let status = if approved { "approved" } else { "rejected" };
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE data_rectification_requests
            SET status = $1, reviewed_by = $2, reviewed_at = $3, review_notes = $4
            WHERE id = $5
            "#,
        )
        .bind(status)
        .bind(reviewer_id)
        .bind(now)
        .bind(notes)
        .bind(id)
        .execute(&self.pool)
        .await?;

        // If approved, update the information_objects table
        if approved {
            if let Some(rect) = self.get_rectification(id).await? {
                // Apply the change
                sqlx::query(
                    r#"
                    UPDATE information_objects
                    SET description = COALESCE($1, description),
                        updated_at = $2
                    WHERE id = $3
                    "#,
                )
                .bind(Some(rect.new_value.clone()))
                .bind(now)
                .bind(rect.object_id)
                .execute(&self.pool)
                .await?;
            }
        }

        tracing::info!("Rectification request {} {}: reviewer={}", id, status, reviewer_id);
        Ok(())
    }

    // ============================================
    // Erasure Requests
    // ============================================

    pub async fn create_erasure(
        &self,
        id: Uuid,
        user_id: Uuid,
        object_id: Uuid,
        erasure_type: ErasureType,
        legal_basis: Option<String>,
        justification: Option<String>,
        retention_check: bool,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Result<()> {
        let erasure_type_str = match erasure_type {
            ErasureType::Anonymization => "anonymization",
            ErasureType::Deletion => "deletion",
            ErasureType::Pseudonymization => "pseudonymization",
        };

        let status = if retention_check {
            "legal_review"
        } else {
            "pending"
        };

        sqlx::query(
            r#"
            INSERT INTO data_erasure_requests
                (id, requesting_user_id, object_id, erasure_type, legal_basis,
                 retention_check, justification, status, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(object_id)
        .bind(erasure_type_str)
        .bind(legal_basis)
        .bind(retention_check)
        .bind(justification)
        .bind(status)
        .bind(expires_at)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        tracing::info!("Created erasure request: id={}, object_id={}, type={}", id, object_id, erasure_type_str);
        Ok(())
    }

    pub async fn get_erasure(&self, id: Uuid) -> Result<Option<ErasureRequestRow>> {
        let row = sqlx::query_as::<sqlx::Postgres, ErasureRequestRow>(
            "SELECT id, requesting_user_id, object_id, erasure_type, legal_basis,
                   retention_check, status, reviewed_by, reviewed_at,
                   completed_at, expires_at, created_at
            FROM data_erasure_requests
            WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_user_erasures(
        &self,
        user_id: Uuid,
        status_filter: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<ErasureRequestRow>> {
        let rows = if let Some(status) = status_filter {
            sqlx::query_as::<sqlx::Postgres, ErasureRequestRow>(
                "SELECT id, requesting_user_id, object_id, erasure_type, legal_basis,
                       retention_check, status, reviewed_by, reviewed_at,
                       completed_at, expires_at, created_at
                FROM data_erasure_requests
                WHERE requesting_user_id = $1 AND status = $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4"
            )
            .bind(user_id)
            .bind(status)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<sqlx::Postgres, ErasureRequestRow>(
                "SELECT id, requesting_user_id, object_id, erasure_type, legal_basis,
                       retention_check, status, reviewed_by, reviewed_at,
                       completed_at, expires_at, created_at
                FROM data_erasure_requests
                WHERE requesting_user_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3"
            )
            .bind(user_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows)
    }

    pub async fn approve_erasure(
        &self,
        id: Uuid,
        reviewer_id: Uuid,
        approved: bool,
        notes: Option<String>,
        completed: bool,
    ) -> Result<()> {
        let status = if approved {
            if completed { "completed" } else { "approved" }
        } else {
            "rejected"
        };
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE data_erasure_requests
            SET status = $1, reviewed_by = $2, reviewed_at = $3
            WHERE id = $4
            "#,
        )
        .bind(status)
        .bind(reviewer_id)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if completed {
            sqlx::query(
                r#"
                UPDATE data_erasure_requests
                SET completed_at = $1
                WHERE id = $2
                "#,
            )
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;

            // Perform the erasure
            if let Some(erasure) = self.get_erasure(id).await? {
                self.perform_erasure(&erasure).await?;
            }
        }

        tracing::info!("Erasure request {} {}: reviewer={}, completed={}", id, status, reviewer_id, completed);
        Ok(())
    }

    async fn perform_erasure(&self, erasure: &ErasureRequestRow) -> Result<()> {
        match erasure.erasure_type.as_str() {
            "anonymization" => {
                sqlx::query(
                    r#"
                    UPDATE information_objects
                    SET title = '[GEPERSONALISEERD]',
                        description = '[GEPERSONALISEERD]',
                        content_text = '[GEPERSONALISEERD]',
                        updated_at = $1
                    WHERE id = $2
                    "#,
                )
                .bind(Utc::now())
                .bind(erasure.object_id)
                .execute(&self.pool)
                .await?;
            }
            "deletion" => {
                sqlx::query(
                    r#"
                    UPDATE information_objects
                    SET title = '[VERWIJDERD]',
                        description = NULL,
                        content_text = NULL,
                        updated_at = $1
                    WHERE id = $2
                    "#,
                )
                .bind(Utc::now())
                .bind(erasure.object_id)
                .execute(&self.pool)
                .await?;
            }
            "pseudonymization" => {
                sqlx::query(
                    r#"
                    UPDATE information_objects
                    SET title = 'PSEUDO-' || SUBSTR(MD5(random()::text)::text, 1, 8),
                        updated_at = $1
                    WHERE id = $2
                    "#,
                )
                .bind(Utc::now())
                .bind(erasure.object_id)
                .execute(&self.pool)
                .await?;
            }
            _ => {
                tracing::warn!("Unknown erasure type: {}", erasure.erasure_type);
            }
        }

        tracing::info!("Performed {} on object {}", erasure.erasure_type, erasure.object_id);
        Ok(())
    }

    // ============================================
    // Pending Requests (Admin/Compliance)
    // ============================================

    pub async fn get_pending_dsar(&self) -> Result<PendingDsarResponse> {
        // Get pending SARs
        let sar = sqlx::query_as::<sqlx::Postgres, SubjectAccessRequestRow>(
            "SELECT id, requesting_user_id, request_type, status, requested_fields,
                   response_data, error_message, expires_at, completed_at, created_at
            FROM subject_access_requests
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT 50"
        )
        .fetch_all(&self.pool)
        .await?;

        // Get pending rectifications
        let rectifications = sqlx::query_as::<sqlx::Postgres, RectificationRequestRow>(
            "SELECT id, requesting_user_id, object_id, field_name, old_value, new_value,
                   justification, status, reviewed_by, reviewed_at, expires_at, created_at
            FROM data_rectification_requests
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT 50"
        )
        .fetch_all(&self.pool)
        .await?;

        // Get pending erasures
        let erasures = sqlx::query_as::<sqlx::Postgres, ErasureRequestRow>(
            "SELECT id, requesting_user_id, object_id, erasure_type, legal_basis,
                   retention_check, status, reviewed_by, reviewed_at,
                   completed_at, expires_at, created_at
            FROM data_erasure_requests
            WHERE status IN ('pending', 'legal_review')
            ORDER BY created_at ASC
            LIMIT 50"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PendingDsarResponse { sar, rectifications, erasures })
    }

    // ============================================
    // Audit Logging
    // ============================================

    pub async fn log_audit(
        &self,
        request_type: &str,
        request_id: Uuid,
        user_id: Uuid,
        action: &str,
        details: Option<String>,
        ip_address: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO data_subject_rights_audit
                (request_type, request_id, user_id, action, details, ip_address)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(request_type)
        .bind(request_id)
        .bind(user_id)
        .bind(action)
        .bind(details)
        .bind(ip_address)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ============================================
    // User Data Aggregation (for SAR)
    // ============================================

    pub async fn get_user_data(&self, user_id: Uuid) -> Result<Value> {
        // Get all personal data for the user
        let objects = sqlx::query(
            r#"
            SELECT id, title, description, content_text, domain_id,
                   object_type, created_at, updated_at
            FROM information_objects
            WHERE created_by = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let objects_data: Vec<Value> = objects
            .into_iter()
            .map(|r| -> Result<Value> {
                Ok(serde_json::json!({
                    "id": r.try_get::<Uuid, _>(0)?,
                    "title": r.try_get::<String, _>(1)?,
                    "description": r.try_get::<Option<String>, _>(2)?,
                    "object_type": r.try_get::<String, _>(3)?,
                    "domain_id": r.try_get::<Uuid, _>(4)?,
                    "created_at": r.try_get::<DateTime<Utc>, _>(5)?,
                    "updated_at": r.try_get::<DateTime<Utc>, _>(6)?,
                }))
            })
            .collect::<Result<Vec<_>>>()?;

        // Get audit entries for this user
        let audit_entries = sqlx::query(
            r#"
            SELECT action, details, created_at
            FROM data_subject_rights_audit
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let audit_data: Vec<Value> = audit_entries
            .into_iter()
            .map(|r| -> Result<Value> {
                Ok(serde_json::json!({
                    "action": r.try_get::<String, _>(0)?,
                    "details": r.try_get::<Option<String>, _>(1)?,
                    "created_at": r.try_get::<DateTime<Utc>, _>(2)?,
                }))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(serde_json::json!({
            "user_id": user_id,
            "export_date": Utc::now(),
            "personal_objects": objects_data,
            "audit_log": audit_data,
            "total_objects": objects_data.len(),
            "audit_entries_count": audit_data.len(),
        }))
    }

    // ============================================
    // Object Information
    // ============================================

    pub async fn get_object(&self, id: Uuid) -> Result<Option<ObjectInfo>> {
        let row = sqlx::query(
            "SELECT id, title, description, content_text, domain_id, retention_period, created_at
             FROM information_objects
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(ObjectInfo {
                id: r.try_get::<Uuid, _>(0)?,
                title: r.try_get::<String, _>(1)?,
                description: r.try_get::<Option<String>, _>(2)?,
                content_text: r.try_get::<Option<String>, _>(3)?,
                domain_id: r.try_get::<Uuid, _>(4)?,
                retention_period: r.try_get::<Option<String>, _>(5)?,
                created_at: r.try_get::<DateTime<Utc>, _>(6)?,
            })),
            None => Ok(None),
        }
    }
}

pub struct ObjectInfo {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub content_text: Option<String>,
    pub domain_id: Uuid,
    pub retention_period: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================
// Woo Publication Repository
// ============================================

pub struct WooRepository {
    pool: PgPool,
}

impl WooRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============================================
    // Woo Publication Operations
    // ============================================

    pub async fn create_publication(
        &self,
        id: Uuid,
        object_id: Uuid,
        platform: PublicationPlatform,
        category_ids: Option<Vec<Uuid>>,
        legal_basis: Option<String>,
        summary: Option<String>,
        consultation_required: bool,
        redactions: Option<Vec<Redaction>>,
        created_at: DateTime<Utc>,
    ) -> Result<()> {
        let platform_str = match platform {
            PublicationPlatform::Rijksoverheid => "rijksoverheid",
            PublicationPlatform::OverheidNl => "overheid_nl",
            PublicationPlatform::Gemeente => "gemeente",
            PublicationPlatform::Provincie => "provincie",
            PublicationPlatform::Waterschap => "waterschap",
            PublicationPlatform::Custom => "custom",
        };

        let redactions_json = redactions.map(|r| serde_json::to_value(r).unwrap());

        sqlx::query(
            r#"
            INSERT INTO woo_publication_requests
                (id, object_id, publication_platform, publication_status,
                 legal_basis, publication_summary, consultation_required,
                 redactions, approved_by, created_at, updated_at)
            VALUES ($1, $2, $3, 'pending', $4, $5, $6, $7, $8, NULL, $9, $9)
            "#,
        )
        .bind(id)
        .bind(object_id)
        .bind(platform_str)
        .bind(legal_basis)
        .bind(summary)
        .bind(consultation_required)
        .bind(redactions_json)
        .bind(created_at)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        tracing::info!("Created Woo publication: id={}, object_id={}", id, object_id);
        Ok(())
    }

    pub async fn get_publication(&self, id: Uuid) -> Result<Option<WooPublicationRow>> {
        let row = sqlx::query_as::<sqlx::Postgres, WooPublicationRow>(
            "SELECT id, object_id, publication_platform, publication_status,
                   category_ids, legal_basis, publication_summary, consultation_required,
                   consultation_completed_at, redactions, refusal_ground, approved_by,
                   approved_at, published_at, publication_url, doi, imposition_reference,
                   publicatie_nr, woo_publication_date, withdrawn_at, withdrawal_reason,
                   created_at, updated_at
            FROM woo_publication_requests
            WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_publications(
        &self,
        status: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<WooPublicationRow>, i64)> {
        let (rows, total) = if let Some(status_filter) = status {
            let rows = sqlx::query_as::<sqlx::Postgres, WooPublicationRow>(
                "SELECT id, object_id, publication_platform, publication_status,
                       category_ids, legal_basis, publication_summary, consultation_required,
                       consultation_completed_at, redactions, refusal_ground, approved_by,
                       approved_at, published_at, publication_url, doi, imposition_reference,
                       publicatie_nr, woo_publication_date, withdrawn_at, withdrawal_reason,
                       created_at, updated_at
                FROM woo_publication_requests
                WHERE publication_status = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3"
            )
            .bind(status_filter)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM woo_publication_requests WHERE publication_status = $1"
            )
            .bind(status_filter)
            .fetch_one(&self.pool)
            .await?;

            (rows, total)
        } else {
            let rows = sqlx::query_as::<sqlx::Postgres, WooPublicationRow>(
                "SELECT id, object_id, publication_platform, publication_status,
                       category_ids, legal_basis, publication_summary, consultation_required,
                       consultation_completed_at, redactions, refusal_ground, approved_by,
                       approved_at, published_at, publication_url, doi, imposition_reference,
                       publicatie_nr, woo_publication_date, withdrawn_at, withdrawal_reason,
                       created_at, updated_at
                FROM woo_publication_requests
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2"
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM woo_publication_requests"
            )
            .fetch_one(&self.pool)
            .await?;

            (rows, total)
        };

        Ok((rows, total))
    }

    pub async fn approve_publication(
        &self,
        id: Uuid,
        approver_id: Uuid,
        approved: bool,
        refusal_ground: Option<WooRefusalGround>,
        notes: Option<String>,
    ) -> Result<()> {
        let status = if approved { "approved" } else { "rejected" };
        let refusal_ground_str = refusal_ground.map(|g| match g {
            WooRefusalGround::Privacy => "privacy",
            WooRefusalGround::NationalSecurity => "national_security",
            WooRefusalGround::CommercialConfidence => "commercial_confidence",
            WooRefusalGround::OngoingInvestigation => "ongoing_investigation",
            WooRefusalGround::InternationalRelations => "international_relations",
            WooRefusalGround::None => "none",
        });

        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE woo_publication_requests
            SET publication_status = $1, approved_by = $2, approved_at = $3,
                refusal_ground = $4, updated_at = $3
            WHERE id = $5
            "#,
        )
        .bind(status)
        .bind(approver_id)
        .bind(now)
        .bind(refusal_ground_str)
        .bind(id)
        .execute(&self.pool)
        .await?;

        tracing::info!("Woo publication {} {} by {}", id, status, approver_id);
        Ok(())
    }

    pub async fn publish_publication(
        &self,
        id: Uuid,
        publication_url: Option<String>,
        doi: Option<String>,
        imposition_reference: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE woo_publication_requests
            SET publication_status = 'published', published_at = $1,
                publication_url = $2, doi = $3, imposition_reference = $4,
                woo_publication_date = $5, updated_at = $1
            WHERE id = $6
            "#,
        )
        .bind(now)
        .bind(publication_url)
        .bind(doi)
        .bind(imposition_reference)
        .bind(now.date_naive())
        .bind(id)
        .execute(&self.pool)
        .await?;

        tracing::info!("Woo publication {} published", id);
        Ok(())
    }

    pub async fn withdraw_publication(
        &self,
        id: Uuid,
        reason: String,
    ) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE woo_publication_requests
            SET publication_status = 'withdrawn', withdrawn_at = $1,
                withdrawal_reason = $2, updated_at = $1
            WHERE id = $3
            "#,
        )
        .bind(now)
        .bind(reason)
        .bind(id)
        .execute(&self.pool)
        .await?;

        tracing::info!("Woo publication {} withdrawn", id);
        Ok(())
    }

    pub async fn mark_consultation_complete(&self, id: Uuid) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE woo_publication_requests
            SET consultation_completed_at = $1, updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        tracing::info!("Woo publication {} consultation completed", id);
        Ok(())
    }

    // ============================================
    // Woo Request (Active Verzoeken) Operations
    // ============================================

    pub async fn create_request(
        &self,
        id: Uuid,
        reference_number: String,
        requester_name: String,
        requester_email: String,
        requester_address: Option<String>,
        requester_type: RequesterType,
        request_type: WooRequestType,
        title: String,
        description: String,
        requested_information: Option<Vec<String>>,
        priority: WooPriority,
        decision_due_date: chrono::NaiveDate,
        created_at: DateTime<Utc>,
    ) -> Result<()> {
        let requester_type_str = match requester_type {
            RequesterType::Citizen => "citizen",
            RequesterType::Organization => "organization",
            RequesterType::Journalist => "journalist",
            RequesterType::Government => "government",
            RequesterType::Other => "other",
        };

        let request_type_str = match request_type {
            WooRequestType::Information => "information",
            WooRequestType::Documents => "documents",
            WooRequestType::Consultation => "consultation",
            WooRequestType::Other => "other",
        };

        let priority_str = match priority {
            WooPriority::Low => "low",
            WooPriority::Normal => "normal",
            WooPriority::High => "high",
            WooPriority::Urgent => "urgent",
        };

        sqlx::query(
            r#"
            INSERT INTO woo_requests
                (id, reference_number, requester_name, requester_email, requester_address,
                 requester_type, title, description, requested_information,
                 priority, request_status, decision_due_date, received_date, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'received', $12, CURRENT_DATE, $13, $13)
            "#,
        )
        .bind(id)
        .bind(&reference_number)
        .bind(requester_name)
        .bind(requester_email)
        .bind(requester_address)
        .bind(requester_type_str)
        .bind(title)
        .bind(description)
        .bind(requested_information)
        .bind(priority_str)
        .bind(decision_due_date)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        tracing::info!("Created Woo request: id={}, reference={}", id, reference_number);
        Ok(())
    }

    pub async fn get_request(&self, id: Uuid) -> Result<Option<WooRequestRow>> {
        let row = sqlx::query_as::<sqlx::Postgres, WooRequestRow>(
            "SELECT id, reference_number, requester_name, requester_email, requester_address,
                   requester_type, title, description, requested_information,
                   priority, request_status, assigned_to, decision, refusal_grounds,
                   decision_date, decision_due_date, consultation_extension,
                   documents_provided, notes, created_at, updated_at
            FROM woo_requests
            WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_requests(
        &self,
        status: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<WooRequestRow>, i64)> {
        let (rows, total) = if let Some(status_filter) = status {
            let rows = sqlx::query_as::<sqlx::Postgres, WooRequestRow>(
                "SELECT id, reference_number, requester_name, requester_email, requester_address,
                       requester_type, title, description, requested_information,
                       priority, request_status, assigned_to, decision, refusal_grounds,
                       decision_date, decision_due_date, consultation_extension,
                       documents_provided, notes, created_at, updated_at
                FROM woo_requests
                WHERE request_status = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3"
            )
            .bind(status_filter)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM woo_requests WHERE request_status = $1"
            )
            .bind(status_filter)
            .fetch_one(&self.pool)
            .await?;

            (rows, total)
        } else {
            let rows = sqlx::query_as::<sqlx::Postgres, WooRequestRow>(
                "SELECT id, reference_number, requester_name, requester_email, requester_address,
                       requester_type, title, description, requested_information,
                       priority, request_status, assigned_to, decision, refusal_grounds,
                       decision_date, decision_due_date, consultation_extension,
                       documents_provided, notes, created_at, updated_at
                FROM woo_requests
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2"
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM woo_requests"
            )
            .fetch_one(&self.pool)
            .await?;

            (rows, total)
        };

        Ok((rows, total))
    }

    pub async fn get_statistics(&self) -> Result<WooStatistics> {
        let total_requests: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM woo_requests"
        )
        .fetch_one(&self.pool)
        .await?;

        let pending_publication: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM woo_publication_requests WHERE publication_status IN ('pending', 'assessment')"
        )
        .fetch_one(&self.pool)
        .await?;

        let published_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM woo_publication_requests WHERE publication_status = 'published'"
        )
        .fetch_one(&self.pool)
        .await?;

        // Calculate average processing days
        let avg_days: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(EXTRACT(DAY FROM (decision_date - received_date)))::NUMERIC
             FROM woo_requests
             WHERE decision_date IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_processing_days = avg_days.unwrap_or(0.0);

        // Count overdue requests
        let overdue_requests: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM woo_requests
             WHERE decision_due_date < CURRENT_DATE AND request_status NOT IN ('information_provided', 'refused', 'withdrawn')"
        )
        .fetch_one(&self.pool)
        .await?;

        // Get upcoming deadlines
        let deadline_rows = sqlx::query(
            r#"
            SELECT id, reference_number, title, decision_due_date
            FROM woo_requests
            WHERE decision_due_date > CURRENT_DATE AND decision_due_date <= CURRENT_DATE + INTERVAL '14 days'
            AND request_status NOT IN ('information_provided', 'refused', 'withdrawn')
            ORDER BY decision_due_date ASC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let upcoming_deadlines: Vec<WooDeadlineSummary> = deadline_rows
            .into_iter()
            .map(|r| -> Result<WooDeadlineSummary> {
                let due_date: chrono::NaiveDate = r.try_get::<chrono::NaiveDate, _>(3)?;
                let days_until = (due_date - Utc::now().date_naive()).num_days();
                Ok(WooDeadlineSummary {
                    request_id: r.try_get::<Uuid, _>(0)?,
                    reference_number: r.try_get::<String, _>(1)?,
                    title: r.try_get::<String, _>(2)?,
                    days_until_due: days_until,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(WooStatistics {
            total_requests,
            pending_publication,
            published_count,
            avg_processing_days,
            overdue_requests,
            upcoming_deadlines,
        })
    }

    pub async fn get_upcoming_deadlines(&self) -> Result<Vec<WooDeadlineSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT id, reference_number, title, decision_due_date
            FROM woo_requests
            WHERE decision_due_date > CURRENT_DATE AND decision_due_date <= CURRENT_DATE + INTERVAL '28 days'
            AND request_status NOT IN ('information_provided', 'refused', 'withdrawn')
            ORDER BY decision_due_date ASC
            LIMIT 20
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| -> Result<WooDeadlineSummary> {
                let due_date: chrono::NaiveDate = r.try_get::<chrono::NaiveDate, _>(3)?;
                let days_until = (due_date - Utc::now().date_naive()).num_days();
                Ok(WooDeadlineSummary {
                    request_id: r.try_get::<Uuid, _>(0)?,
                    reference_number: r.try_get::<String, _>(1)?,
                    title: r.try_get::<String, _>(2)?,
                    days_until_due: days_until,
                })
            })
            .collect()
    }

    pub async fn get_published_documents(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<WooPublicationRow>, i64)> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM woo_publication_requests WHERE publication_status = 'published'"
        )
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query_as::<sqlx::Postgres, WooPublicationRow>(
            "SELECT id, object_id, publication_platform, publication_status,
                   category_ids, legal_basis, publication_summary, consultation_required,
                   consultation_completed_at, redactions, refusal_ground, approved_by,
                   approved_at, published_at, publication_url, doi, imposition_reference,
                   publicatie_nr, woo_publication_date, withdrawn_at, withdrawal_reason,
                   created_at, updated_at
            FROM woo_publication_requests
            WHERE publication_status = 'published'
            ORDER BY published_at DESC
            LIMIT $1 OFFSET $2"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok((rows, total))
    }
}
