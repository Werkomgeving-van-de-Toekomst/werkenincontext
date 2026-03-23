//! Verifiable Credential authentication module
//!
//! This module implements EBSI-compliant Verifiable Presentation (VP) verification
//! for Dutch Wallet (nl-wallet) integration.
//!
//! Voor de **wallet_web**-stroom (sessie starten in de browser), zie [`crate::id`].
//!
//! # Flow
//!
//! 1. User presents Verifiable Presentation via Dutch Wallet app
//! 2. API verifies VP signature, issuer trust, and expiration
//! 3. Extract claims from credentials (Custom MDT)
//! 4. Map credential subject to local user/organization
//! 5. Issue short-lived JWT for subsequent API requests
//!
//! # EBSI Compliance
//!
//! - Supports DID-based authentication (did:key, did:web)
//! - W3C Verifiable Credentials Data Model v1.1
//! - JSON-LD JWT proof verification
//! - SD-JWT for selective disclosure (future)

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::middleware::auth::{AuthContext, JwtService, Role};

pub mod verifier;
pub mod types;
pub mod mapper;

pub use verifier::VpVerifier;
pub use types::*;
pub use mapper::CredentialMapper;

/// VC authentication configuration
#[derive(Clone, Debug)]
pub struct VcConfig {
    /// Trusted issuer DIDs
    pub trusted_issuers: Vec<String>,

    /// JWT secret for issued tokens
    pub jwt_secret: String,

    /// Short-lived token expiration (minutes)
    pub vc_token_expiration_min: i64,

    /// Enable strict mode (reject unknown credential types)
    pub strict_mode: bool,
}

impl Default for VcConfig {
    fn default() -> Self {
        Self {
            trusted_issuers: vec![
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(), // Example
            ],
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".to_string()),
            vc_token_expiration_min: 60, // 1 hour
            strict_mode: false,
        }
    }
}

impl VcConfig {
    /// Laad trustlijst en strictheid uit de omgeving (`VC_TRUSTED_ISSUERS`, `VC_STRICT_MODE`, `JWT_SECRET`).
    pub fn from_env() -> Self {
        let mut c = Self::default();
        if let Ok(s) = std::env::var("VC_TRUSTED_ISSUERS") {
            let issuers: Vec<String> = s
                .split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect();
            if !issuers.is_empty() {
                c.trusted_issuers = issuers;
            }
        }
        if let Ok(v) = std::env::var("VC_STRICT_MODE") {
            c.strict_mode = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(secret) = std::env::var("JWT_SECRET") {
            c.jwt_secret = secret;
        }
        c
    }
}

/// VC authentication error types
#[derive(Debug, Error)]
pub enum VcError {
    #[error("Invalid Verifiable Presentation: {0}")]
    InvalidPresentation(String),

    #[error("Invalid Verifiable Credential: {0}")]
    InvalidCredential(String),

    #[error("Signature verification failed: {0}")]
    SignatureVerification(String),

    #[error("Issuer not trusted: {0}")]
    UntrustedIssuer(String),

    #[error("Credential expired: {0}")]
    CredentialExpired(String),

    #[error("Missing required claim: {0}")]
    MissingClaim(String),

    #[error("User mapping failed: {0}")]
    UserMappingFailed(String),

    #[error("Token issuance failed: {0}")]
    TokenIssuanceFailed(String),
}

/// Wallet authentication request
#[derive(Debug, Deserialize)]
pub struct WalletAuthRequest {
    /// Verifiable Presentation (JWT format)
    pub vp_token: String,

    /// Presentation type (jwt_vc_json, jwt_vp_json, etc.)
    #[serde(default)]
    pub presentation_submission: Option<PresentationSubmission>,

    /// Client ID for OAuth 2.0 / OpenID Connect
    pub client_id: Option<String>,

    /// Redirect URI
    pub redirect_uri: Option<String>,
}

/// Presentation submission metadata
#[derive(Debug, Deserialize)]
pub struct PresentationSubmission {
    pub definition_id: String,
    pub descriptor_map: Vec<DescriptorMap>,
}

/// Descriptor map for VP
#[derive(Debug, Deserialize)]
pub struct DescriptorMap {
    pub id: String,
    pub format: String,
    pub path: String,
}

/// Wallet authentication response
#[derive(Debug, Serialize)]
pub struct WalletAuthResponse {
    /// Access token (short-lived JWT)
    pub access_token: String,

    /// Token type
    pub token_type: String,

    /// Expires in (seconds)
    pub expires_in: i64,

    /// User context extracted from VC
    pub user: VcUserContext,
}

/// User context extracted from Verifiable Credential
#[derive(Debug, Serialize)]
pub struct VcUserContext {
    /// User ID (from credential subject or generated)
    pub id: Uuid,

    /// Organization ID (from credential or mapped)
    pub organization_id: Uuid,

    /// Roles mapped from credential type/attributes
    pub roles: Vec<String>,

    /// Credential type used for authentication
    pub credential_type: String,

    /// Issuer DID
    pub issuer: String,

    /// Additional claims from credential
    pub additional_claims: Value,
}

/// Verifiable Presentation (simplified)
#[derive(Debug, Deserialize, Serialize)]
pub struct VerifiablePresentation {
    /// JSON-LD context
    #[serde(rename = "@context")]
    pub context: serde_json::Value,

    /// Presentation type
    pub type_: Vec<String>,

    /// Verifiable Credentials
    #[serde(default)]
    pub verifiable_credential: Vec<VerifiableCredential>,

    /// Holder (DID)
    pub holder: Option<String>,

    /// Proof (JWT signature)
    pub proof: Option<Proof>,
}

/// Verifiable Credential (simplified)
#[derive(Debug, Deserialize, Serialize)]
pub struct VerifiableCredential {
    /// JSON-LD context
    #[serde(rename = "@context")]
    pub context: serde_json::Value,

    /// Credential type
    #[serde(rename = "type")]
    pub type_: Vec<String>,

    /// Credential ID
    pub id: Option<String>,

    /// Issuer (DID)
    pub issuer: OneOrMany<Issuer>,

    /// Issuance date
    pub issuance_date: String, // ISO 8601

    /// Expiration date
    pub expiration_date: Option<String>, // ISO 8601

    /// Credential subject
    pub credential_subject: OneOrMany<CredentialSubject>,

    /// Proof
    pub proof: Option<Proof>,
}

/// Issuer (can be string or object)
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

/// Issuer variant
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Issuer {
    Id(String),
    Object { id: String },
}

/// Credential subject
#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialSubject {
    /// Subject ID (DID)
    pub id: String,

    /// Additional attributes
    #[serde(flatten)]
    pub attributes: Value,
}

/// Proof (signature)
#[derive(Debug, Deserialize, Serialize)]
pub struct Proof {
    /// Proof type (JwtProof2020, etc.)
    #[serde(rename = "type")]
    pub type_: String,

    /// Proof purpose (authentication, assertionMethod, etc.)
    pub proof_purpose: String,

    /// Verification method (DID URL)
    pub verification_method: String,

    /// Created timestamp
    pub created: String,

    /// Proof value (JWS)
    pub proof_value: Option<String>,

    /// JWT (for JWT proofs)
    pub jwt: Option<String>,
}

/// Custom MDT credential attributes (example structure)
#[derive(Debug, Deserialize, Serialize)]
pub struct CustomMdtAttributes {
    /// Organization identifier
    pub organization_id: Option<String>,

    /// User role within organization
    pub role: Option<String>,

    /// Department
    pub department: Option<String>,

    /// Employee ID
    pub employee_id: Option<String>,

    /// Clearance level
    pub clearance_level: Option<String>,

    /// Additional attributes
    #[serde(flatten)]
    pub additional: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vc_config_default() {
        let config = VcConfig::default();
        assert_eq!(config.vc_token_expiration_min, 60);
        assert!(!config.strict_mode);
    }

    #[test]
    fn test_vc_error_display() {
        let err = VcError::InvalidPresentation("test".to_string());
        // Error format is "Invalid Verifiable Presentation: test"
        assert!(err.to_string().contains("Invalid Verifiable Presentation"));
    }
}
