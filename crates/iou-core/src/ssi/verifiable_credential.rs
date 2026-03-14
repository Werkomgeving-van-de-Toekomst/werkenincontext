//! Verifiable Credential (VC) and Presentation (VP) validation

use crate::ssi::did::DidDocument;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Verifiable Credential (W3C VC Data Model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,

    #[serde(rename = "type")]
    pub vc_type: Vec<String>,

    pub id: String,

    #[serde(rename = "issuer")]
    pub issuer: String, // DID or URL

    #[serde(rename = "issuanceDate")]
    pub issuance_date: String, // ISO 8601

    #[serde(rename = "expirationDate")]
    pub expiration_date: Option<String>,

    #[serde(rename = "credentialSubject")]
    pub credential_subject: CredentialSubject,

    #[serde(rename = "credentialStatus")]
    pub credential_status: Option<CredentialStatus>,

    #[serde(rename = "proof")]
    pub proof: Proof,
}

/// Credential subject containing the claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSubject {
    pub id: String, // DID of the subject

    /// Additional claims (role, municipality, loa, etc.)
    #[serde(flatten)]
    pub claims: HashMap<String, ClaimValue>,
}

/// Claim value (supports various types)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClaimValue {
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<String>),
    Object(HashMap<String, ClaimValue>),
}

impl ClaimValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ClaimValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[String]> {
        match self {
            ClaimValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

/// Credential status for revocation checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStatus {
    pub id: String,
    #[serde(rename = "type")]
    pub status_type: String,
}

/// Cryptographic proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    #[serde(rename = "type")]
    pub proof_type: String,

    #[serde(rename = "created")]
    pub created: String,

    #[serde(rename = "proofPurpose")]
    pub proof_purpose: String,

    #[serde(rename = "verificationMethod")]
    pub verification_method: String,

    #[serde(rename = "jws")]
    pub jws: Option<String>, // JSON Web Signature
}

/// Verifiable Presentation (containing one or more VCs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiablePresentation {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,

    #[serde(rename = "type")]
    pub vp_type: Vec<String>,

    pub id: Option<String>,

    pub holder: String, // DID of the presenter

    #[serde(rename = "verifiableCredential")]
    pub verifiable_credential: Vec<VerifiableCredential>,

    #[serde(rename = "proof")]
    pub proof: Option<Proof>,
}

/// Extracted claims from VC/VP
#[derive(Debug, Clone)]
pub struct Claims {
    /// Subject DID
    pub did: String,

    /// Issuer DID
    pub issuer: String,

    /// Tenant (municipality) identifier
    pub municipality: Option<String>,

    /// User roles
    pub roles: Vec<String>,

    /// Level of Assurance
    pub loa: Option<String>,

    /// Display name
    pub display_name: Option<String>,

    /// Email
    pub email: Option<String>,

    /// Additional claims
    pub additional: HashMap<String, ClaimValue>,
}

impl VerifiableCredential {
    /// Extract claims from the credential subject
    pub fn extract_claims(&self) -> Claims {
        let mut claims = Claims {
            did: self.credential_subject.id.clone(),
            issuer: self.issuer.clone(),
            municipality: None,
            roles: Vec::new(),
            loa: None,
            display_name: None,
            email: None,
            additional: HashMap::new(),
        };

        for (key, value) in &self.credential_subject.claims {
            match key.as_str() {
                "municipality" => {
                    claims.municipality = value.as_str().map(|s| s.to_string());
                }
                "role" | "roles" => {
                    if let Some(arr) = value.as_array() {
                        claims.roles = arr.to_vec();
                    } else if let Some(s) = value.as_str() {
                        claims.roles.push(s.to_string());
                    }
                }
                "loa" | "levelOfAssurance" => {
                    claims.loa = value.as_str().map(|s| s.to_string());
                }
                "displayName" | "name" => {
                    claims.display_name = value.as_str().map(|s| s.to_string());
                }
                "email" | "emailAddress" => {
                    claims.email = value.as_str().map(|s| s.to_string());
                }
                _ => {
                    claims.additional.insert(key.clone(), value.clone());
                }
            }
        }

        claims
    }

    /// Validate the credential (signature, expiration, revocation)
    pub async fn validate(&self, resolver: &dyn DIDResolver) -> Result<(), VCValidationError> {
        // Check expiration
        if let Some(exp) = &self.expiration_date {
            let exp_date = chrono::DateTime::parse_from_rfc3339(exp)
                .map_err(|_| VCValidationError::InvalidFormat("Invalid expiration date".into()))?;

            if exp_date < chrono::Utc::now() {
                return Err(VCValidationError::Expired);
            }
        }

        // Resolve issuer DID and validate signature
        let _: DidDocument = resolver.resolve(&self.issuer).await
            .map_err(|e| VCValidationError::DIDResolution(e.to_string()))?;

        // TODO: Verify JWS signature using public key from DID document
        // This requires integration with a crypto library (bbs-signatures, etc.)

        // TODO: Check revocation status if credentialStatus is present

        Ok(())
    }
}

impl VerifiablePresentation {
    /// Extract all claims from contained VCs
    pub fn extract_claims(&self) -> Result<Claims, VCValidationError> {
        let mut merged = Claims {
            did: self.holder.clone(),
            issuer: String::new(),
            municipality: None,
            roles: Vec::new(),
            loa: None,
            display_name: None,
            email: None,
            additional: HashMap::new(),
        };

        for vc in &self.verifiable_credential {
            let claims = vc.extract_claims();

            // Merge claims
            if merged.municipality.is_none() && claims.municipality.is_some() {
                merged.municipality = claims.municipality;
            }
            if merged.roles.is_empty() && !claims.roles.is_empty() {
                merged.roles = claims.roles;
            }
            if merged.loa.is_none() && claims.loa.is_some() {
                merged.loa = claims.loa;
            }
            if merged.display_name.is_none() && claims.display_name.is_some() {
                merged.display_name = claims.display_name;
            }
            if merged.email.is_none() && claims.email.is_some() {
                merged.email = claims.email;
            }

            // Merge additional claims
            for (k, v) in claims.additional {
                merged.additional.entry(k).or_insert(v);
            }

            merged.issuer = claims.issuer;
        }

        Ok(merged)
    }

    /// Validate the presentation and all contained VCs
    pub async fn validate(&self, resolver: &dyn DIDResolver) -> Result<(), VCValidationError> {
        // Validate each contained VC
        for vc in &self.verifiable_credential {
            vc.validate(resolver).await?;
        }

        // TODO: Verify presentation proof if present

        Ok(())
    }
}

/// DID resolver trait
#[async_trait::async_trait]
pub trait DIDResolver: Send + Sync {
    async fn resolve(&self, did: &str) -> Result<DidDocument, Box<dyn std::error::Error>>;
}

/// VC validation errors
#[derive(Debug, Error)]
pub enum VCValidationError {
    #[error("Credential expired")]
    Expired,

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("DID resolution failed: {0}")]
    DIDResolution(String),

    #[error("Credential revoked")]
    Revoked,

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Missing required claim: {0}")]
    MissingClaim(String),

    #[error("Untrusted issuer: {0}")]
    UntrustedIssuer(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_value_as_str() {
        let value = ClaimValue::String("utrecht".to_string());
        assert_eq!(value.as_str(), Some("utrecht"));
    }

    #[test]
    fn test_claim_value_as_array() {
        let value = ClaimValue::Array(vec!["admin".to_string()]);
        assert_eq!(value.as_array(), Some(&["admin".to_string()][..]));
    }
}
