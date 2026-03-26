//! Decentralized Identifier (DID) resolution and document parsing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error as ThisError;

/// Supported DID methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DidMethod {
    Key,
    Web,
    Ebsi,
    PolygonId, // For European identity systems
    Custom(String),
}

impl std::str::FromStr for DidMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "key" => Ok(DidMethod::Key),
            "web" => Ok(DidMethod::Web),
            "ebsi" => Ok(DidMethod::Ebsi),
            "polygonid" => Ok(DidMethod::PolygonId),
            other => Ok(DidMethod::Custom(other.to_string())),
        }
    }
}

/// DID Document (W3C DID Core specification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,

    pub id: String,

    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,

    pub authentication: Vec<serde_json::Value>,

    #[serde(rename = "assertionMethod")]
    pub assertion_method: Vec<serde_json::Value>,

    pub service: Option<Vec<Service>>,
}

/// Verification method (public key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub key_type: String,
    pub controller: String,
    #[serde(rename = "publicKeyJwk")]
    pub public_key_jwk: Option<Jwk>,
    #[serde(rename = "publicKeyBase58")]
    pub public_key_base58: Option<String>,
}

/// JSON Web Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,
    pub kid: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub crv: Option<String>,
}

/// Service endpoint in DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub service_type: String,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}

/// Parse DID from string
pub fn parse_did(did: &str) -> Result<(DidMethod, String, String), DidError> {
    if !did.starts_with("did:") {
        return Err(DidError::InvalidFormat("Missing 'did:' prefix".into()));
    }

    let parts: Vec<&str> = did.split(':').collect();
    if parts.len() < 3 {
        return Err(DidError::InvalidFormat("DID must have at least method".into()));
    }

    let method = parts[1].parse::<DidMethod>()
        .map_err(|_| DidError::InvalidFormat(format!("Unknown method: {}", parts[1])))?;

    let method_id = parts[2..].join(":");

    Ok((method, method_id.to_string(), did.to_string()))
}

/// DID key generation (for testing)
pub fn generate_did_key(public_key: &str) -> String {
    format!("did:key:z{}", public_key)
}

/// DID errors
#[derive(Debug, ThisError)]
pub enum DidError {
    #[error("Invalid DID format: {0}")]
    InvalidFormat(String),

    #[error("Method not supported: {0}")]
    MethodNotSupported(String),

    #[error("Resolution failed: {0}")]
    ResolutionFailed(String),
}

/// Type alias for did:key methods
pub type DidKey = String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_did_key() {
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let (method, id, full) = parse_did(did).unwrap();
        assert_eq!(method, DidMethod::Key);
        assert_eq!(full, did);
    }

    #[test]
    fn test_parse_did_web() {
        let did = "did:web:example.com";
        let (method, id, full) = parse_did(did).unwrap();
        assert_eq!(method, DidMethod::Web);
        assert_eq!(id, "example.com");
    }

    #[test]
    fn test_invalid_did() {
        let result = parse_did("not-a-did");
        assert!(result.is_err());
    }
}
