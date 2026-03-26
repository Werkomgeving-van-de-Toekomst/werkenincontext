//! Tenant context and validation

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Municipality identifier (tenant ID)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TenantId(String);

impl TenantId {
    pub fn new(id: impl Into<String>) -> Result<Self, TenantError> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(Self(id))
    }

    fn validate(id: &str) -> Result<(), TenantError> {
        if id.is_empty() {
            return Err(TenantError::InvalidFormat("Tenant ID cannot be empty".into()));
        }
        if id.len() > 50 {
            return Err(TenantError::InvalidFormat("Tenant ID too long".into()));
        }
        // Must be lowercase alphanumeric with hyphens and apostrophes (for names like 's-hertogenbosch)
        if !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '\'') {
            return Err(TenantError::InvalidFormat("Invalid characters (must be lowercase alphanumeric with hyphens/apostrophes)".into()));
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Level of Assurance
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoA {
    Low = 0,
    Substantial = 1,
    High = 2,
}

impl LoA {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(LoA::Low),
            "substantial" => Some(LoA::Substantial),
            "high" => Some(LoA::High),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LoA::Low => "low",
            LoA::Substantial => "substantial",
            LoA::High => "high",
        }
    }
}

/// Tenant context from VC claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub holder_did: String,
    pub roles: Vec<String>,
    pub loa: LoA,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

impl TenantContext {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|&r| self.has_role(r))
    }

    pub fn meets_loa(&self, minimum: LoA) -> bool {
        self.loa >= minimum
    }

    /// Create TenantContext from VC claims
    #[cfg(feature = "server")]
    pub fn from_vc_claims(
        claims: &crate::ssi::verifiable_credential::Claims,
    ) -> Result<Self, TenantError> {
        let municipality = claims.municipality
            .clone()
            .ok_or_else(|| TenantError::InvalidFormat("Missing municipality claim".into()))?;

        let loa = claims.loa
            .as_ref()
            .and_then(|s| LoA::from_str(s))
            .unwrap_or(LoA::Low);

        Ok(TenantContext {
            tenant_id: TenantId::new(municipality)?,
            holder_did: claims.did.clone(),
            roles: claims.roles.clone(),
            loa,
            display_name: claims.display_name.clone(),
            email: claims.email.clone(),
        })
    }
}

/// Tenant errors
#[derive(Debug, Error)]
pub enum TenantError {
    #[error("Invalid tenant format: {0}")]
    InvalidFormat(String),

    #[error("Tenant not found: {0}")]
    NotFound(String),

    #[error("Access denied to tenant: {0}")]
    AccessDenied(String),

    #[error("Cross-tenant access not allowed")]
    CrossTenantAccess,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_id_validation() {
        assert!(TenantId::new("utrecht").is_ok());
        assert!(TenantId::new("s-hertogenbosch").is_ok());
        assert!(TenantId::new("").is_err());
        assert!(TenantId::new("INVALID").is_err());
    }

    #[test]
    fn test_loa_ordering() {
        assert!(LoA::Low < LoA::Substantial);
        assert!(LoA::Substantial < LoA::High);
    }

    #[test]
    fn test_tenant_context_role_check() {
        let ctx = TenantContext {
            tenant_id: TenantId::new("test").unwrap(),
            holder_did: "did:example:123".to_string(),
            roles: vec!["citizen".to_string(), "editor".to_string()],
            loa: LoA::Substantial,
            display_name: None,
            email: None,
        };

        assert!(ctx.has_role("citizen"));
        assert!(!ctx.has_role("admin"));
        assert!(ctx.has_any_role(&["admin", "citizen"]));
    }
}
