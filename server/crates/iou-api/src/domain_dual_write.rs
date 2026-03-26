//! Dual-write implementation for InformationDomain
//!
//! Implements the DualWrite trait for InformationDomain to support
//! writing to both DuckDB and Supabase.

use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;

use iou_core::domain::{DomainStatus, DomainType, InformationDomain};

use super::{db::Database, dual_write::DualWrite, supabase::SupabasePool};

// Helper to convert DomainType to string for database storage
// Note: Matches PostgreSQL CHECK constraint (titlecase: Zaak, Project, etc.)
fn domain_type_to_string(dt: &DomainType) -> String {
    dt.to_string()
}

// Helper to convert DomainStatus to string for database storage
// Note: Matches PostgreSQL CHECK constraint (lowercase: concept, actief, etc.)
fn domain_status_to_string(ds: &DomainStatus) -> String {
    ds.to_string().to_lowercase()
}

#[async_trait]
impl DualWrite for InformationDomain {
    type Id = Uuid;

    async fn write_to_duckdb(&self, db: &Database) -> Result<Uuid> {
        // Use the existing create_domain method
        db.create_domain(self)?;
        Ok(self.id)
    }

    async fn write_to_supabase(&self, db: &SupabasePool) -> Result<Uuid> {
        let domain_type = domain_type_to_string(&self.domain_type);
        let status = domain_status_to_string(&self.status);

        sqlx::query(
            r#"
            INSERT INTO information_domains
                (id, domain_type, name, description, status, organization_id,
                 owner_user_id, parent_domain_id, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                status = EXCLUDED.status,
                owner_user_id = EXCLUDED.owner_user_id,
                parent_domain_id = EXCLUDED.parent_domain_id,
                metadata = EXCLUDED.metadata,
                updated_at = CURRENT_TIMESTAMP
            "#
        )
        .bind(self.id)
        .bind(&domain_type)
        .bind(&self.name)
        .bind(&self.description)
        .bind(&status)
        .bind(self.organization_id)
        .bind(self.owner_user_id)
        .bind(self.parent_domain_id)
        .bind(&self.metadata)
        .bind(self.created_at)
        .bind(self.updated_at)
        .execute(db.inner())
        .await?;

        Ok(self.id)
    }

    async fn update_in_duckdb(&self, db: &Database) -> Result<Uuid> {
        // Update in DuckDB (same as create for now, since we use INSERT with OR CONFLICT)
        db.create_domain(self)?;
        Ok(self.id)
    }

    async fn update_in_supabase(&self, db: &SupabasePool) -> Result<Uuid> {
        let status = domain_status_to_string(&self.status);

        sqlx::query(
            r#"
            UPDATE information_domains
            SET name = $2,
                description = $3,
                status = $4,
                owner_user_id = $5,
                parent_domain_id = $6,
                metadata = $7,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#
        )
        .bind(self.id)
        .bind(&self.name)
        .bind(&self.description)
        .bind(&status)
        .bind(self.owner_user_id)
        .bind(self.parent_domain_id)
        .bind(&self.metadata)
        .execute(db.inner())
        .await?;

        Ok(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_domain_type_to_string() {
        assert_eq!(domain_type_to_string(&DomainType::Zaak), "Zaak");
        assert_eq!(domain_type_to_string(&DomainType::Project), "Project");
        assert_eq!(domain_type_to_string(&DomainType::Beleid), "Beleid");
        assert_eq!(domain_type_to_string(&DomainType::Expertise), "Expertise");
    }

    #[test]
    fn test_domain_status_to_string() {
        assert_eq!(domain_status_to_string(&DomainStatus::Concept), "concept");
        assert_eq!(domain_status_to_string(&DomainStatus::Actief), "actief");
        assert_eq!(domain_status_to_string(&DomainStatus::Afgerond), "afgerond");
        assert_eq!(domain_status_to_string(&DomainStatus::Gearchiveerd), "gearchiveerd");
    }

    #[test]
    fn test_information_domain_new() {
        let domain = InformationDomain::new(
            DomainType::Zaak,
            "Test Domain".to_string(),
            Uuid::new_v4(),
        );

        assert_eq!(domain.domain_type, DomainType::Zaak);
        assert_eq!(domain.name, "Test Domain");
        assert_eq!(domain.status, DomainStatus::Actief);
    }
}
