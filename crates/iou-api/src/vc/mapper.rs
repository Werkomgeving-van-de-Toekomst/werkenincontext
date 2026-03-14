//! Credential mapper
//!
//! Maps Verifiable Credentials to local user context and roles.

use crate::vc::{
    VcConfig, VcError, VcUserContext, VerifiableCredential,
    CustomMdtAttributes, OneOrMany, Issuer,
};
use crate::middleware::auth::Role;
use serde_json::Value;
use uuid::Uuid;
use md5;

/// Maps credentials to user context
pub struct CredentialMapper {
    config: VcConfig,
}

impl CredentialMapper {
    /// Create a new credential mapper
    pub fn new(config: VcConfig) -> Self {
        Self { config }
    }

    /// Map a credential to user context
    pub fn map_credential_to_user(
        &self,
        vc: &VerifiableCredential,
        cred_type: &str,
        issuer: &str,
    ) -> Result<VcUserContext, VcError> {
        // Extract credential subject
        let subject = match &vc.credential_subject {
            OneOrMany::One(s) => s,
            OneOrMany::Many(_) => {
                return Err(VcError::UserMappingFailed(
                    "Multiple subjects not supported".to_string(),
                ))
            }
        };

        // Generate or extract user ID
        let id = self.extract_user_id(subject, issuer)?;

        // Map organization ID
        let organization_id = self.extract_organization_id(subject)?;

        // Map credential type to roles
        let roles = self.map_credential_type_to_roles(cred_type, subject)?;

        // Extract additional claims
        let additional_claims = self.extract_additional_claims(subject);

        Ok(VcUserContext {
            id,
            organization_id,
            roles: roles.iter().map(|r| r.to_string()).collect(),
            credential_type: cred_type.to_string(),
            issuer: issuer.to_string(),
            additional_claims,
        })
    }

    /// Extract user ID from credential subject
    fn extract_user_id(&self, subject: &crate::vc::CredentialSubject, issuer: &str) -> Result<Uuid, VcError> {
        // Try to get from subject.id if it's a UUID
        if let Ok(uuid) = Uuid::parse_str(&subject.id) {
            return Ok(uuid);
        }

        // Otherwise, generate deterministic UUID from DID + issuer
        use md5::{Md5, Digest};
        let did_and_issuer = format!("{}{}", subject.id, issuer);
        let hash = Md5::digest(did_and_issuer.as_bytes());
        let uuid = Uuid::from_slice(&hash[0..16])
            .map_err(|_| VcError::UserMappingFailed("UUID generation failed".to_string()))?;

        Ok(uuid)
    }

    /// Extract organization ID from credential
    fn extract_organization_id(&self, subject: &crate::vc::CredentialSubject) -> Result<Uuid, VcError> {
        // Try to get organization_id attribute
        if let Some(org_id) = subject.attributes.get("organization_id") {
            if let Some(id_str) = org_id.as_str() {
                if let Ok(uuid) = Uuid::parse_str(id_str) {
                    return Ok(uuid);
                }
            }
        }

        // Fallback: try parsing from subject.id (if DID contains org info)
        // For now, use a default org UUID
        tracing::warn!("No organization_id found, using default");
        Ok(Uuid::new_v4()) // TODO: In production, this should fail or derive from DID
    }

    /// Map credential type and attributes to roles
    fn map_credential_type_to_roles(
        &self,
        cred_type: &str,
        subject: &crate::vc::CredentialSubject,
    ) -> Result<Vec<Role>, VcError> {
        let roles = match cred_type {
            // Custom MDT - check role attribute
            "CustomMdt" | "custom_mdt" | "Mdt" => {
                if let Some(role_str) = subject.attributes.get("role") {
                    self.map_attribute_to_role(role_str)
                } else {
                    // Default role for MDT
                    vec![Role::DomainViewer]
                }
            }

            // PID (Personal Identification) - standard roles
            "Pid" | "PersonalIdentification" => {
                vec![Role::DomainViewer]
            }

            // DiD (Address) - limited roles
            "DiD" | "AddressData" => {
                vec![Role::DomainViewer]
            }

            // EBSI Professional资格
            "EBSProfessionalQualification" => {
                vec![Role::ObjectCreator, Role::DomainEditor]
            }

            // Unknown type
            _ if self.config.strict_mode => {
                return Err(VcError::UserMappingFailed(
                    format!("Unknown credential type: {}", cred_type)
                ))
            }

            // Default: viewer role
            _ => {
                tracing::warn!("Unknown credential type: {}, using default role", cred_type);
                vec![Role::DomainViewer]
            }
        };

        Ok(roles)
    }

    /// Map role attribute string to Role enum
    fn map_attribute_to_role(&self, role_val: &Value) -> Vec<Role> {
        let role_str = role_val.as_str().unwrap_or("");

        match role_str.to_lowercase().as_str() {
            "admin" | "administrator" => vec![Role::Admin],
            "creator" | "object_creator" => vec![Role::ObjectCreator],
            "editor" | "domain_editor" | "object_editor" => vec![Role::ObjectEditor, Role::DomainEditor],
            "approver" | "object_approver" => vec![Role::ObjectApprover],
            "compliance_officer" => vec![Role::ComplianceOfficer],
            "woo_officer" => vec![Role::WooOfficer],
            "manager" | "domain_manager" => vec![Role::DomainManager],
            _ => vec![Role::DomainViewer],
        }
    }

    /// Extract additional claims from credential subject
    fn extract_additional_claims(&self, subject: &crate::vc::CredentialSubject) -> Value {
        // Return all attributes as additional claims
        subject.attributes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_config() -> VcConfig {
        VcConfig::default()
    }

    #[test]
    fn test_map_attribute_to_role() {
        let mapper = CredentialMapper::new(test_config());

        assert_eq!(
            mapper.map_attribute_to_role(&json!("admin")),
            vec![Role::Admin]
        );

        assert_eq!(
            mapper.map_attribute_to_role(&json!("creator")),
            vec![Role::ObjectCreator]
        );

        assert_eq!(
            mapper.map_attribute_to_role(&json!("unknown")),
            vec![Role::DomainViewer]
        );
    }

    #[test]
    fn test_user_id_extraction() {
        let config = test_config();
        let mapper = CredentialMapper::new(config);

        // With valid UUID
        let subject = crate::vc::CredentialSubject {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            attributes: json!({}),
        };
        let result = mapper.extract_user_id(&subject, "did:example:test");
        assert!(result.is_ok());

        // With DID (generates deterministic UUID)
        let subject = crate::vc::CredentialSubject {
            id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            attributes: json!({}),
        };
        let result = mapper.extract_user_id(&subject, "did:example:issuer");
        assert!(result.is_ok());
    }
}
