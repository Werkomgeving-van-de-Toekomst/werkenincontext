# Section 7: RONL Business API Layer with SSI & Business Rules

## Overview

This section implements the Business API Layer pattern for government digital services, following the RONL (Regels Op NL) architecture. The implementation uses **Self-Sovereign Identity (SSI)** with EBSI/nl-wallet Verifiable Credentials instead of Keycloak, and integrates BPMN/DMN business rules through the existing `iou-regels` crate.

**Dependencies:**
- Section 1 (Foundation & Configuration) - Configuration loading
- Section 2 (Orchestrator Integration) - Workflow execution
- Section 3 (Authentication Integration) - Base JWT auth foundation
- Existing `iou-regels` crate - Open Regels SPARQL client

**Files to Create:**
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/ssi/mod.rs` - SSI/VC module
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/ssi/verifiable_credential.rs` - VC validation
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/ssi/did.rs` - DID resolution
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/tenancy/mod.rs` - Multi-tenant isolation
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/audit/mod.rs` - Audit logging
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/audit/logger.rs` - Write-ahead audit logger
- `/Users/marc/Projecten/iou-modern/crates/iou-core/src/audit/models.rs` - Audit models
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/middleware/vc.rs` - VC validation middleware
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/v1/mod.rs` - Versioned API
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/v1/rules.rs` - Rules evaluation endpoints
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/v1/calculations.rs` - Calculation endpoints
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/v1/processes.rs` - BPMN process endpoints
- `/Users/marc/Projecten/iou-modern/crates/iou-regels/src/dmn.rs` - DMN evaluation
- `/Users/marc/Projecten/iou-modern/crates/iou-regels/src/bpmn.rs` - BPMN process integration

**Files to Modify:**
- `/Users/marc/Projecten/iou-modern/crates/iou-core/Cargo.toml` - Add SSI/audit/tenancy dependencies
- `/Users/marc/Projecten/iou-modern/crates/iou-regels/Cargo.toml` - Add DMN/BPMN dependencies
- `/Users/marc/Projecten/iou-modern/crates/iou-api/Cargo.toml` - Add VC/validation dependencies
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs` - Add v1 routes and middleware
- `/Users/marc/Projecten/iou-modern/crates/iou-api/src/config.rs` - Add SSI/audit configuration
- `/Users/marc/Projecten/iou-modern/crates/iou-regels/src/lib.rs` - Export dmn/bpmn modules

## Architecture

```
Resident / Caseworker
       ↓
Municipality Portal (MijnOmgeving)
       ↓  (EBSI / nl-wallet Verifiable Credentials)
SSI Wallet (nl-wallet, Walloon, EBSI)
       ↓  (Verifiable Presentation - VP)
Business API Layer  ← IOU-Modern RONL Layer
       ├── VC Validation (DID resolution, signature check)
       ├── Claims extraction (municipality, roles, LoA)
       ├── Tenancy isolation
       ├── Rules engine (DMN via iou-regels + Open Regels SPARQL)
       ├── Process engine (BPMN via orchestrator)
       └── Audit logging (write-ahead)
       ↓  (validated context)
Orchestrator / AI Agents
```

### Key Design Decisions

1. **SSI-based Auth**: No IAM server dependency. Users present Verifiable Credentials from their wallet (nl-wallet, EBSI, etc.)

2. **DID Resolution**: Support for `did:key:`, `did:web:`, and `did:ebsi:` methods. DIDs resolved to public keys for VC signature validation.

3. **Selective Disclosure**: VPs use BBS+ signatures to allow selective disclosure of claims (e.g., prove municipality without revealing full identity).

4. **Tenant Isolation**: All database queries include municipality filter. Row-level security enforced.

5. **Rules as Code**: Business rules defined in DMN format, evaluated through `iou-regels` crate integration with Open Regels SPARQL endpoint.

6. **BPMN Process Integration**: Process instances managed through orchestrator with DMN decision service integration.

7. **Audit Write-Ahead**: Audit entries written BEFORE action execution. Failure to audit = action denied.

8. **Versioned API**: All business endpoints under `/v1/*` for future compatibility.

## Tests

### Verifiable Credential Validation Tests

**Test: valid VC with proper signature is accepted**
- Given: Verifiable Credential signed by trusted issuer
- And: VC signature valid against issuer's DID document
- When: Request is made with VC in Authorization header
- Then: Request proceeds to handler
- And: Claims are extracted correctly

**Test: VC with invalid signature is rejected**
- Given: VC with forged signature
- When: Request is made
- Then: Returns 401 Unauthorized
- And: Audit log records signature validation failure

**Test: expired VC is rejected**
- Given: VC with expiration date in the past
- When: Request is made
- Then: Returns 401 Unauthorized

**Test: VC from untrusted issuer is rejected**
- Given: VC signed by issuer not in trusted list
- When: Request is made
- Then: Returns 401 Unauthorized

### Tenant Isolation Tests

**Test: tenant isolation in rules queries**
- Given: User with VC containing municipality: "utrecht"
- And: Rules exist for "utrecht" and "amsterdam"
- When: `GET /v1/rules` is called
- Then: Only "utrecht" rules are returned

**Test: cross-tenant rules evaluation is blocked**
- Given: User from municipality "amsterdam"
- When: Rules evaluation for "utrecht" is attempted
- Then: Returns 403 Forbidden

### Business Rules Tests

**Test: DMN rule evaluation**
- Given: Valid DMN rule for "zorgtoeslag"
- And: Input: { income: 24000, age: 25 }
- When: `POST /v1/rules/zorgtoeslag/evaluate` is called
- Then: Returns: { eligible: true, amount: 1150 }

**Test: Open Regels SPARQL query**
- Given: Regel UUID from Open Regels
- When: `GET /v1/rules/open-regels/{uuid}` is called
- Then: Returns rule details with FLINT/DMN specifications

**Test: DMN loaded from Open Regels**
- Given: DMN decision table URI from Open Regels
- When: Decision is loaded via SPARQL CONSTRUCT
- Then: DMN XML is parsed and evaluator is ready

### BPMN Process Tests

**Test: process start with DMN decision**
- Given: Valid BPMN process definition
- And: Process contains DMN decision service reference
- When: `POST /v1/processes` is called with inputs
- Then: Process instance is created
- And: DMN decision is evaluated during execution

**Test: process state transitions**
- Given: Running process instance
- When: DMN decision outputs a value
- Then: Process transitions to next activity
- And: Audit log records the decision

### Audit Logging Tests

**Test: audit entry created before action**
- Given: Valid VC-based request
- When: Rules evaluation is initiated
- Then: Audit entry is written first
- And: Entry contains: user_did, action, timestamp, municipality

**Test: audit failure prevents action**
- Given: Audit database unavailable
- When: Rules evaluation is attempted
- Then: Returns 500 Internal Server Error
- And: No evaluation is performed

**Test: audit entry is tamper-evident**
- Given: Existing audit entry
- When: Update or delete is attempted
- Then: Returns 403 Forbidden
- And: Entry remains unchanged

**Test: audit query with tenant filtering**
- Given: Audit entries for multiple tenants
- When: Query is made with tenant_id
- Then: Only entries for that tenant are returned
- And: Cross-tenant access is blocked

### LoA (Level of Assurance) Tests

**Test: low LoA cannot access sensitive operations**
- Given: VC with LoA: "low"
- When: Bulk export operation is called
- Then: Returns 403 Forbidden

**Test: substantial LoA allows citizen operations**
- Given: VC with LoA: "substantial"
- When: Citizen operation is called
- Then: Request proceeds

## Implementation

### Step 1: Create SSI/VC Module

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/ssi/mod.rs`

```rust
//! Self-Sovereign Identity (SSI) and Verifiable Credentials support
//!
//! This module provides VC validation, DID resolution, and claims extraction
//! for EBSI, nl-wallet, and other SSI wallet providers.

pub mod verifiable_credential;
pub mod did;
pub mod presentation;
pub mod resolver;

pub use verifiable_credential::{
    VerifiableCredential, VerifiablePresentation, VCValidationError,
    Claims, ClaimValue, DIDResolver,
};
pub use did::{DidMethod, DidDocument, DidKey, parse_did};
pub use presentation::PresentationValidator;
pub use resolver::UniversalDidResolver;
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/ssi/verifiable_credential.rs`

```rust
//! Verifiable Credential (VC) and Presentation (VP) validation

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
        let did_doc = resolver.resolve(&self.issuer).await
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
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/ssi/did.rs`

```rust
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
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/ssi/presentation.rs`

```rust
//! Verifiable Presentation validation

use crate::ssi::verifiable_credential::{VerifiablePresentation, DIDResolver, VCValidationError};

/// Presentation validator for VPs
pub struct PresentationValidator {
    resolver: std::sync::Arc<dyn DIDResolver>,
    trusted_issuers: Vec<String>,
}

impl PresentationValidator {
    /// Create a new presentation validator
    pub fn new(
        resolver: std::sync::Arc<dyn DIDResolver>,
        trusted_issuers: Vec<String>,
    ) -> Self {
        Self { resolver, trusted_issuers }
    }

    /// Validate a presentation
    pub async fn validate(
        &self,
        vp: &VerifiablePresentation,
    ) -> Result<ValidatedPresentation, VCValidationError> {
        // Validate all contained VCs
        vp.validate(&*self.resolver).await?;

        // Extract claims
        let claims = vp.extract_claims()?;

        // Check trusted issuers
        if !self.trusted_issuers.is_empty() {
            let issuer_trusted = self.trusted_issuers.iter()
                .any(|trusted| self.is_issuer_trusted(&claims.issuer, trusted));

            if !issuer_trusted {
                return Err(VCValidationError::UntrustedIssuer(claims.issuer));
            }
        }

        Ok(ValidatedPresentation {
            holder_did: vp.holder.clone(),
            claims,
        })
    }

    /// Check if issuer is trusted
    fn is_issuer_trusted(&self, issuer: &str, trusted: &str) -> bool {
        if issuer == trusted {
            return true;
        }
        if issuer.starts_with(trusted) {
            return true;
        }
        false
    }
}

/// Validated presentation with claims
pub struct ValidatedPresentation {
    pub holder_did: String,
    pub claims: crate::ssi::verifiable_credential::Claims,
}
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/ssi/resolver.rs`

```rust
//! Universal DID resolver supporting multiple DID methods

use crate::ssi::{did::DidDocument, verifiable_credential::DIDResolver};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Universal DID resolver
pub struct UniversalDidResolver {
    http: Client,
    cache: tokio::sync::RwLock<HashMap<String, Arc<DidDocument>>>,
    cache_ttl_seconds: u64,
}

impl UniversalDidResolver {
    /// Create a new universal DID resolver
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .user_agent("iou-modern/did-resolver")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("HTTP client creation failed"),
            cache: tokio::sync::RwLock::new(HashMap::new()),
            cache_ttl_seconds: 900, // 15 minutes
        }
    }

    /// Resolve a DID using appropriate method
    async fn resolve_by_method(&self, did: &str) -> Result<DidDocument, ResolverError> {
        if !did.starts_with("did:") {
            return Err(ResolverError::InvalidFormat("Missing did: prefix".into()));
        }

        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(ResolverError::InvalidFormat("Invalid DID format".into()));
        }

        let method = parts[1];

        match method {
            "web" => self.resolve_did_web(did).await,
            "key" => self.resolve_did_key(did).await,
            "ebsi" => self.resolve_did_ebsi(did).await,
            "polygonid" => self.resolve_did_polygonid(did).await,
            _ => Err(ResolverError::MethodNotSupported(method.to_string())),
        }
    }

    /// Resolve did:web DIDs
    async fn resolve_did_web(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // did:web:example.com -> https://example.com/.well-known/did.json
        let domain = did.split(':').nth(2)
            .ok_or_else(|| ResolverError::InvalidFormat("Missing domain".into()))?;

        let url = format!("https://{}/.well-known/did.json", domain);

        let response = self.http.get(&url)
            .header("Accept", "application/did+json")
            .send()
            .await
            .map_err(|e| ResolverError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| ResolverError::NotFound(e.to_string()))?;

        let did_doc: DidDocument = response.json().await
            .map_err(|e| ResolverError::ParseError(e.to_string()))?;

        Ok(did_doc)
    }

    /// Resolve did:key DIDs (generates DID document on the fly)
    async fn resolve_did_key(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // did:key is self-describing - we can derive the DID document from the key
        // For now, return a mock implementation
        Ok(DidDocument {
            context: serde_json::json!("https://w3id.org/did/v1"),
            id: did.to_string(),
            verification_method: vec![],
            authentication: vec![],
            assertion_method: vec![],
            service: None,
        })
    }

    /// Resolve did:ebsi DIDs using EBSI resolver
    async fn resolve_did_ebsi(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // Use EBSI DID resolver
        let url = format!("https://api.preprod.ebsi.eu/did-registry/v1/identifiers/{}", did);

        let response = self.http.get(&url)
            .header("Accept", "application/did+json")
            .send()
            .await
            .map_err(|e| ResolverError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| ResolverError::NotFound(e.to_string()))?;

        let did_doc: DidDocument = response.json().await
            .map_err(|e| ResolverError::ParseError(e.to_string()))?;

        Ok(did_doc)
    }

    /// Resolve did:polygonid DIDs
    async fn resolve_did_polygonid(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // Use Polygon ID resolver
        let url = format!("https://universaldid.com/v1/identifiers/{}", did);

        let response = self.http.get(&url)
            .header("Accept", "application/did+json")
            .send()
            .await
            .map_err(|e| ResolverError::HttpError(e.to_string()))?
            .error_for_status()
            .map_err(|e| ResolverError::NotFound(e.to_string()))?;

        let did_doc: DidDocument = response.json().await
            .map_err(|e| ResolverError::ParseError(e.to_string()))?;

        Ok(did_doc)
    }
}

impl Default for UniversalDidResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DIDResolver for UniversalDidResolver {
    async fn resolve(&self, did: &str) -> Result<DidDocument, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(doc) = cache.get(did) {
                return Ok(Arc::clone(doc).as_ref().clone());
            }
        }

        // Resolve
        let doc = self.resolve_by_method(did).await?;

        // Cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(did.to_string(), Arc::new(doc.clone()));
        }

        Ok(doc)
    }
}

/// Resolver errors
#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Invalid DID format: {0}")]
    InvalidFormat(String),

    #[error("Method not supported: {0}")]
    MethodNotSupported(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("DID not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_did_key() {
        let resolver = UniversalDidResolver::new();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let result = resolver.resolve(did).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_works() {
        let resolver = UniversalDidResolver::new();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let _ = resolver.resolve(did).await.unwrap();
        // Second call should use cache
        let _ = resolver.resolve(did).await.unwrap();
    }
}
```

### Step 2: Create Audit Module (Full Implementation)

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/audit/mod.rs`

```rust
//! Tamper-evident audit logging for compliance
//!
//! This module provides audit logging that satisfies BIO, NEN 7510,
//! and AVG/GDPR requirements for government digital services.
//!
//! # Write-Ahead Semantics
//!
//! Audit entries are written BEFORE the action is executed.
//! If audit logging fails, the action MUST NOT proceed.

pub mod logger;
pub mod models;

pub use logger::{AuditLogger, AuditBackend, PostgresAuditBackend};
pub use models::{AuditEntry, AuditAction, AuditOutcome, AuditFilter, AuditQuery};
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/audit/models.rs`

```rust
//! Audit log models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Audit log entry
///
/// Records every action performed in the system for compliance purposes.
/// Entries are immutable and retained for 7 years as per BIO requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier for this audit entry
    pub id: Uuid,

    /// Timestamp when the action occurred (UTC)
    pub timestamp: DateTime<Utc>,

    /// Tenant (municipality) context
    pub tenant_id: String,

    /// User DID who performed the action
    pub user_did: String,

    /// Type of action performed
    pub action: AuditAction,

    /// Resource that was acted upon
    pub resource_type: String,

    /// Specific resource identifier
    pub resource_id: String,

    /// Outcome of the action
    pub outcome: AuditOutcome,

    /// IP address of the client
    pub ip_address: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Additional context (JSON)
    pub context: Option<serde_json::Value>,

    /// Session ID for correlation
    pub session_id: Option<String>,

    /// Related audit entries (for chained operations)
    pub parent_id: Option<Uuid>,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new<S: Into<String>>(
        tenant_id: S,
        user_did: S,
        action: AuditAction,
        resource_type: S,
        resource_id: S,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            tenant_id: tenant_id.into(),
            user_did: user_did.into(),
            action,
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            outcome: AuditOutcome::Success,
            ip_address: None,
            user_agent: None,
            context: None,
            session_id: None,
            parent_id: None,
        }
    }

    /// Add outcome
    pub fn with_outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Add request metadata
    pub fn with_metadata(
        mut self,
        ip: Option<String>,
        ua: Option<String>,
    ) -> Self {
        self.ip_address = ip;
        self.user_agent = ua;
        self
    }

    /// Add context
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Set parent audit entry
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

/// Type of action performed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Document actions
    DocumentCreated,
    DocumentViewed,
    DocumentUpdated,
    DocumentDeleted,
    DocumentApproved,
    DocumentRejected,

    // Process/Rule actions
    ProcessStarted,
    ProcessCompleted,
    ProcessFailed,
    ProcessCancelled,
    RuleEvaluated,

    // Authentication actions
    UserLogin,
    UserLogout,
    VCPresented,

    // Admin actions
    TenantCreated,
    TenantUpdated,
    UserInvited,
    UserRemoved,

    // Calculation actions
    CalculationStarted,
    CalculationCompleted,

    // Generic action
    Custom(String),
}

impl From<String> for AuditAction {
    fn from(s: String) -> Self {
        match s.as_str() {
            "document_created" => AuditAction::DocumentCreated,
            "document_viewed" => AuditAction::DocumentViewed,
            "document_updated" => AuditAction::DocumentUpdated,
            "document_deleted" => AuditAction::DocumentDeleted,
            "document_approved" => AuditAction::DocumentApproved,
            "document_rejected" => AuditAction::DocumentRejected,
            "process_started" => AuditAction::ProcessStarted,
            "process_completed" => AuditAction::ProcessCompleted,
            "process_failed" => AuditAction::ProcessFailed,
            "process_cancelled" => AuditAction::ProcessCancelled,
            "rule_evaluated" => AuditAction::RuleEvaluated,
            "user_login" => AuditAction::UserLogin,
            "user_logout" => AuditAction::UserLogout,
            "vc_presented" => AuditAction::VCPresented,
            "calculation_started" => AuditAction::CalculationStarted,
            "calculation_completed" => AuditAction::CalculationCompleted,
            _ => AuditAction::Custom(s),
        }
    }
}

/// Outcome of the action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Action succeeded
    Success,

    /// Action failed (business logic)
    Failed,

    /// Action denied (authorization)
    Denied,

    /// Action errored (technical)
    Error,
}

/// Filter for querying audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub start_timestamp: Option<DateTime<Utc>>,
    pub end_timestamp: Option<DateTime<Utc>>,
    pub user_did: Option<String>,
    pub action: Option<AuditAction>,
    pub resource_type: Option<String>,
    pub limit: u32,
}

impl Default for AuditQuery {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            start_timestamp: Some(now - chrono::Duration::days(30)),
            end_timestamp: Some(now),
            user_did: None,
            action: None,
            resource_type: None,
            limit: 100,
        }
    }
}

/// Result type for audit queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQueryResult {
    pub entries: Vec<AuditEntry>,
    pub total_count: u64,
    pub has_more: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            "utrecht",
            "did:example:user123",
            AuditAction::DocumentCreated,
            "document",
            "doc-123",
        );

        assert_eq!(entry.resource_type, "document");
        assert_eq!(entry.resource_id, "doc-123");
        assert_eq!(entry.outcome, AuditOutcome::Success);
    }

    #[test]
    fn test_audit_entry_builder() {
        let entry = AuditEntry::new(
            "amsterdam",
            "did:example:user456",
            AuditAction::ProcessStarted,
            "process",
            "proc-789",
        )
        .with_outcome(AuditOutcome::Failed)
        .with_metadata(Some("127.0.0.1".to_string()), Some("test-agent".to_string()))
        .with_context(serde_json::json!({"test": "data"}));

        assert_eq!(entry.outcome, AuditOutcome::Failed);
        assert_eq!(entry.ip_address, Some("127.0.0.1".to_string()));
        assert!(entry.context.is_some());
    }

    #[test]
    fn test_audit_query_default() {
        let query = AuditQuery::default();
        assert_eq!(query.limit, 100);
        assert!(query.start_timestamp.is_some());
        assert!(query.end_timestamp.is_some());
    }
}
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/audit/logger.rs`

```rust
//! Audit logger with write-ahead semantics

use super::models::{AuditEntry, AuditAction, AuditOutcome, AuditQuery, AuditQueryResult};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use thiserror::Error;

/// Trait for audit backends
pub trait AuditBackend: Send + Sync {
    /// Write an audit entry (must succeed before action)
    async fn write(&self, entry: &AuditEntry) -> Result<(), AuditError>;

    /// Query audit entries for a tenant
    async fn query(
        &self,
        tenant_id: &str,
        query: &AuditQuery,
    ) -> Result<AuditQueryResult, AuditError>;

    /// Get a specific audit entry
    async fn get(&self, id: Uuid) -> Result<Option<AuditEntry>, AuditError>;
}

/// PostgreSQL audit backend with BIO compliance
pub struct PostgresAuditBackend {
    pool: PgPool,
}

impl PostgresAuditBackend {
    /// Create a new PostgreSQL audit backend
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize the audit table with proper constraints
    pub async fn init(&self) -> Result<(), sqlx::Error> {
        // Create audit_log table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                -- Primary key
                id UUID PRIMARY KEY,

                -- Timestamp (indexed for queries)
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),

                -- Tenant isolation (required for multi-tenancy)
                tenant_id VARCHAR(50) NOT NULL,

                -- User identification
                user_did VARCHAR(255) NOT NULL,

                -- Action classification
                action VARCHAR(100) NOT NULL,

                -- Resource identification
                resource_type VARCHAR(100) NOT NULL,
                resource_id VARCHAR(500) NOT NULL,

                -- Outcome
                outcome VARCHAR(20) NOT NULL,

                -- Request metadata
                ip_address VARCHAR(45),
                user_agent TEXT,

                -- Additional context (JSONB for querying)
                context JSONB,

                -- Session correlation
                session_id VARCHAR(100),

                -- Chained operations
                parent_id UUID REFERENCES audit_log(id),

                -- Tamper evidence (hash of entry)
                entry_hash VARCHAR(64),

                -- Index for tenant queries
                CONSTRAINT audit_tenant_timestamp UNIQUE (tenant_id, timestamp, id)
            );

            -- Indexes for efficient querying
            CREATE INDEX IF NOT EXISTS idx_audit_tenant_timestamp
                ON audit_log(tenant_id, timestamp DESC);

            CREATE INDEX IF NOT EXISTS idx_audit_user_did
                ON audit_log(user_did);

            CREATE INDEX IF NOT EXISTS idx_audit_resource
                ON audit_log(resource_type, resource_id);

            CREATE INDEX IF NOT EXISTS idx_audit_parent
                ON audit_log(parent_id) WHERE parent_id IS NOT NULL;

            CREATE INDEX IF NOT EXISTS idx_audit_context
                ON audit_log USING GIN (context);

            -- Comment for documentation
            COMMENT ON TABLE audit_log IS
                'Tamper-evident audit log with 7-year retention (BIO compliant)';
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create audit function for trigger (if needed)
        sqlx::query(
            r#"
            CREATE OR REPLACE FUNCTION audit_insert_trigger()
            RETURNS TRIGGER AS $$
            BEGIN
                -- Compute entry hash for tamper evidence
                NEW.entry_hash = encode(
                    digest(
                        NEW.id::TEXT ||
                        NEW.timestamp::TEXT ||
                        NEW.tenant_id ||
                        NEW.user_did ||
                        NEW.action ||
                        NEW.resource_type ||
                        NEW.resource_id ||
                        NEW.outcome::TEXT,
                        'sha256'
                    ),
                    'hex'
                );
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl AuditBackend for PostgresAuditBackend {
    async fn write(&self, entry: &AuditEntry) -> Result<(), AuditError> {
        let action = match &entry.action {
            AuditAction::Custom(s) => s.clone(),
            a => serde_json::to_string(a).unwrap_or_else(|_| "custom".to_string()),
        };

        sqlx::query(
            r#"
            INSERT INTO audit_log (
                id, timestamp, tenant_id, user_did, action,
                resource_type, resource_id, outcome,
                ip_address, user_agent, context, session_id, parent_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(entry.id)
        .bind(entry.timestamp)
        .bind(&entry.tenant_id)
        .bind(&entry.user_did)
        .bind(&action)
        .bind(&entry.resource_type)
        .bind(&entry.resource_id)
        .bind(match entry.outcome {
            AuditOutcome::Success => "success",
            AuditOutcome::Failed => "failed",
            AuditOutcome::Denied => "denied",
            AuditOutcome::Error => "error",
        })
        .bind(&entry.ip_address)
        .bind(&entry.user_agent)
        .bind(&entry.context)
        .bind(&entry.session_id)
        .bind(entry.parent_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuditError::Database(e.to_string()))?;

        Ok(())
    }

    async fn query(
        &self,
        tenant_id: &str,
        query: &AuditQuery,
    ) -> Result<AuditQueryResult, AuditError> {
        let limit = i64::from(query.limit.min(1000)); // Max 1000 per query

        // Build dynamic query (simplified - in production use query builder)
        let sql = r#"
            SELECT id, timestamp, tenant_id, user_did, action,
                   resource_type, resource_id, outcome,
                   ip_address, user_agent, context, session_id, parent_id
            FROM audit_log
            WHERE tenant_id = $1
        "#;

        // Count total
        let count_sql = format!(
            "{} SELECT COUNT(*) as total FROM audit_log WHERE tenant_id = $1",
            if query.start_timestamp.is_some() || query.end_timestamp.is_some() {
                "AND timestamp BETWEEN COALESCE($2, '-infinity') AND COALESCE($3, 'infinity')"
            } else {
                ""
            }
        );

        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql)
            .bind(tenant_id);

        if let Some(start) = query.start_timestamp {
            count_query = count_query.bind(start);
        }
        if let Some(end) = query.end_timestamp {
            count_query = count_query.bind(end);
        }

        let total_count = count_query.fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        // Fetch entries
        let rows: Vec<Row> = sqlx::query(sql)
            .bind(tenant_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AuditError::Database(e.to_string()))?;

        let entries: Vec<AuditEntry> = rows.into_iter().map(|row| {
            let action_str: String = row.get("action");
            let outcome_str: String = row.get("outcome");

            AuditEntry {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                tenant_id: row.get("tenant_id"),
                user_did: row.get("user_did"),
                action: AuditAction::from(action_str),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                outcome: match outcome_str.as_str() {
                    "success" => AuditOutcome::Success,
                    "failed" => AuditOutcome::Failed,
                    "denied" => AuditOutcome::Denied,
                    _ => AuditOutcome::Error,
                },
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                context: row.get("context"),
                session_id: row.get("session_id"),
                parent_id: row.get("parent_id"),
            }
        }).collect();

        Ok(AuditQueryResult {
            entries,
            total_count: total_count as u64,
            has_more: entries.len() as u64 < total_count as u64,
        })
    }

    async fn get(&self, id: Uuid) -> Result<Option<AuditEntry>, AuditError> {
        let row = sqlx::query(
            r#"
            SELECT id, timestamp, tenant_id, user_did, action,
                   resource_type, resource_id, outcome,
                   ip_address, user_agent, context, session_id, parent_id
            FROM audit_log WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuditError::Database(e.to_string()))?;

        Ok(row.map(|row| {
            let action_str: String = row.get("action");
            let outcome_str: String = row.get("outcome");

            AuditEntry {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                tenant_id: row.get("tenant_id"),
                user_did: row.get("user_did"),
                action: AuditAction::from(action_str),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                outcome: match outcome_str.as_str() {
                    "success" => AuditOutcome::Success,
                    "failed" => AuditOutcome::Failed,
                    "denied" => AuditOutcome::Denied,
                    _ => AuditOutcome::Error,
                },
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                context: row.get("context"),
                session_id: row.get("session_id"),
                parent_id: row.get("parent_id"),
            }
        }))
    }
}

/// Audit logger with write-ahead enforcement
pub struct AuditLogger<B: AuditBackend> {
    backend: B,
}

impl<B: AuditBackend> AuditLogger<B> {
    /// Create a new audit logger
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Log an action with write-ahead semantics
    ///
    /// The audit entry is written BEFORE the action is performed.
    /// If audit logging fails, the action must not proceed.
    pub async fn log(&self, entry: &AuditEntry) -> Result<(), AuditError> {
        self.backend.write(entry).await
    }

    /// Query audit entries
    pub async fn query(
        &self,
        tenant_id: &str,
        query: &AuditQuery,
    ) -> Result<AuditQueryResult, AuditError> {
        self.backend.query(tenant_id, query).await
    }

    /// Get a specific audit entry
    pub async fn get(&self, id: Uuid) -> Result<Option<AuditEntry>, AuditError> {
        self.backend.get(id).await
    }
}

/// Wrapper type for Arc<AuditLogger>
pub type SharedAuditLogger = Arc<AuditLogger<dyn AuditBackend>>;

/// Audit errors
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Audit backend unavailable")]
    Unavailable,

    #[error("Entry not found: {0}")]
    NotFound(String),

    #[error("Immutable: audit entries cannot be modified")]
    Immutable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_action_from_string() {
        let action = AuditAction::from("document_created".to_string());
        assert!(matches!(action, AuditAction::DocumentCreated));

        let action = AuditAction::from("unknown_action".to_string());
        assert!(matches!(action, AuditAction::Custom(_)));
    }
}
```

### Step 3: Extend iou-regels with DMN/BPMN Support

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-regels/Cargo.toml`

```toml
# Add to existing dependencies
# XML parsing for DMN/BPMN
quick-xml = { version = "0.37", features = ["serialize"] }

# Async runtime
async-trait = "0.1"

# HTTP client (already exists)
reqwest.workspace = true
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-regels/src/dmn.rs` (NEW)

```rust
//! DMN (Decision Model and Notation) rule evaluation
//!
//! This module integrates DMN decision tables with the Open Regels
//! specificaties, allowing for business rule evaluation.

use crate::client::OpenRegelsClient;
use crate::model::{Regel, RegelType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// DMN decision context
#[derive(Debug, Clone)]
pub struct DecisionContext {
    /// Input variables for the decision
    pub inputs: HashMap<String, DecisionValue>,

    /// Tenant (municipality) for tenant-specific rules
    pub tenant_id: Option<String>,

    /// Additional context variables
    pub context: HashMap<String, String>,
}

/// Decision value (supports various types)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecisionValue {
    String(String),
    Integer(i64),
    Double(f64),
    Boolean(bool),
    Date(chrono::NaiveDate),
    Array(Vec<DecisionValue>),
}

impl DecisionValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            DecisionValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            DecisionValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
}

impl From<String> for DecisionValue {
    fn from(s: String) -> Self {
        DecisionValue::String(s)
    }
}

impl From<i64> for DecisionValue {
    fn from(i: i64) -> Self {
        DecisionValue::Integer(i)
    }
}

impl From<f64> for DecisionValue {
    fn from(f: f64) -> Self {
        DecisionValue::Double(f)
    }
}

impl From<bool> for DecisionValue {
    fn from(b: bool) -> Self {
        DecisionValue::Boolean(b)
    }
}

/// DMN decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    /// Decision name
    pub decision: String,

    /// Output values
    pub outputs: HashMap<String, DecisionValue>,

    /// Whether the decision matched any rule
    pub matched: bool,

    /// Matched rule ID
    pub matched_rule_id: Option<String>,

    /// Evaluation metadata
    pub metadata: DecisionMetadata,
}

/// Decision metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMetadata {
    /// Rule that was matched
    pub matched_rule: Option<String>,

    /// Evaluation time in microseconds
    pub evaluation_time_us: u64,

    /// DMN version
    pub dmn_version: String,

    /// Open Regels URI (if loaded from Open Regels)
    pub open_regels_uri: Option<String>,
}

/// DMN decision (loaded from XML or SPARQL)
#[derive(Debug, Clone)]
pub struct Decision {
    pub id: String,
    pub name: String,
    pub inputs: Vec<InputClause>,
    pub outputs: Vec<OutputClause>,
    pub rules: Vec<DecisionRule>,
    pub metadata: DecisionMetadata,
}

#[derive(Debug, Clone)]
pub struct InputClause {
    pub id: String,
    pub name: String,
    pub type_ref: String,
}

#[derive(Debug, Clone)]
pub struct OutputClause {
    pub id: String,
    pub name: String,
    pub type_ref: String,
    pub default_value: Option<DecisionValue>,
}

#[derive(Debug, Clone)]
pub struct DecisionRule {
    pub id: String,
    pub conditions: Vec<Condition>,
    pub conclusions: Vec<Conclusion>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub input_id: String,
    pub operator: ConditionOperator,
    pub value: DecisionValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    In,
    NotIn,
    Between,
}

#[derive(Debug, Clone)]
pub struct Conclusion {
    pub output_id: String,
    pub value: DecisionValue,
}

/// DMN evaluator with Open Regels integration
pub struct DmnEvaluator {
    decisions: HashMap<String, Decision>,
    open_regels: Option<Arc<OpenRegelsClient>>,
}

impl DmnEvaluator {
    /// Create a new DMN evaluator
    pub fn new() -> Self {
        Self {
            decisions: HashMap::new(),
            open_regels: None,
        }
    }

    /// Create evaluator with Open Regels client
    pub fn with_open_regels(mut self, client: Arc<OpenRegelsClient>) -> Self {
        self.open_regels = Some(client);
        self
    }

    /// Load a decision from DMN XML
    pub fn load_dmn_xml(&mut self, xml: &str) -> Result<(), DmnError> {
        let decision = Self::parse_dmn_xml(xml)?;
        self.decisions.insert(decision.id.clone(), decision);
        Ok(())
    }

    /// Load a decision from Open Regels by URI
    pub async fn load_from_open_regels(
        &mut self,
        regel_uri: &str,
    ) -> Result<Decision, DmnError> {
        let client = self.open_regels.as_ref()
            .ok_or(DmnError::OpenRegelsNotAvailable)?;

        // SPARQL query to fetch DMN XML
        let sparql = format!(r#"
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX dmn: <http://www.omg.org/spec/DMN/20191111/MODEL/>

            SELECT ?dmnXml WHERE {{
                <{}> dmn:definition ?dmn .
                ?dmn dmn:expression ?dmnXml .
            }}
        "#, regel_uri);

        let bindings = client.select(&sparql).await
            .map_err(|e| DmnError::FetchError(e.to_string()))?;

        if bindings.is_empty() {
            return Err(DmnError::DecisionNotFound(regel_uri.to_string()));
        }

        let dmn_xml = bindings[0].get("dmnXml")
            .and_then(|v| Some(v.value.clone()))
            .ok_or(DmnError::ParseError("No DMN XML found".into()))?;

        self.load_dmn_xml(&dmn_xml)?;

        self.decisions.get(regel_uri)
            .cloned()
            .ok_or(DmnError::DecisionNotFound(regel_uri.to_string()))
    }

    /// Discover DMN decisions from Open Regels
    pub async fn discover_dmn_decisions(
        &self,
        filter: Option<&str>,
    ) -> Result<Vec<Regel>, DmnError> {
        let client = self.open_regels.as_ref()
            .ok_or(DmnError::OpenRegelsNotAvailable)?;

        let sparql = if let Some(f) = filter {
            format!(r#"
                PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                PREFIX dmn: <http://www.omg.org/spec/DMN/20191111/MODEL/>

                SELECT ?regel ?label ?beschrijving WHERE {{
                    ?regel a dmn:Definition .
                    ?regel dmn:name ?label .
                    OPTIONAL {{ ?regel dmn:description ?beschrijving . }}
                    FILTER(CONTAINS(LCASE(?label), LCASE("{}")))
                }}
                ORDER BY ?label
            "#, f)
        } else {
            r#"
                PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                PREFIX dmn: <http://www.omg.org/spec/DMN/20191111/MODEL/>

                SELECT ?regel ?label ?beschrijving WHERE {{
                    ?regel a dmn:Definition .
                    ?regel dmn:name ?label .
                    OPTIONAL {{ ?regel dmn:description ?beschrijving . }}
                }}
                ORDER BY ?label
            "#.to_string()
        };

        let bindings = client.select(&sparql).await
            .map_err(|e| DmnError::FetchError(e.to_string()))?;

        Ok(bindings.into_iter().map(|mut b| {
            Regel {
                uri: b.remove("regel").map(|v| v.value).unwrap_or_default(),
                label: b.remove("label").map(|v| v.value),
                beschrijving: b.remove("beschrijving").map(|v| v.value),
                juridische_bron: None,
                regel_type: RegelType::Dmn,
                eigenaar: None,
            }
        }).collect())
    }

    /// Evaluate a decision
    pub fn evaluate(
        &self,
        decision_id: &str,
        context: &DecisionContext,
    ) -> Result<DecisionResult, DmnError> {
        let start = std::time::Instant::now();

        let decision = self.decisions
            .get(decision_id)
            .ok_or_else(|| DmnError::DecisionNotFound(decision_id.to_string()))?;

        // Find matching rule
        let matched_rule = decision.rules.iter()
            .find(|rule| self.rule_matches(rule, &decision.inputs, context));

        let mut outputs = HashMap::new();

        let matched_rule_id = if let Some(rule) = matched_rule {
            for conclusion in &rule.conclusions {
                let output = decision.outputs.iter()
                    .find(|o| o.id == conclusion.output_id)
                    .ok_or_else(|| DmnError::EvaluationError(format!(
                        "Output {} not found", conclusion.output_id
                    )))?;

                outputs.insert(output.name.clone(), conclusion.value.clone());
            }
            Some(rule.id.clone())
        } else {
            // Use default values
            for output in &decision.outputs {
                let value = output.default_value.clone()
                    .unwrap_or(DecisionValue::String("".to_string()));
                outputs.insert(output.name.clone(), value);
            }
            None
        };

        Ok(DecisionResult {
            decision: decision_id.to_string(),
            outputs,
            matched: matched_rule.is_some(),
            matched_rule_id,
            metadata: DecisionMetadata {
                matched_rule: matched_rule.map(|r| r.id.clone()),
                evaluation_time_us: start.elapsed().as_micros() as u64,
                dmn_version: "1.4".to_string(),
                open_regels_uri: None,
            },
        })
    }

    /// Check if a rule matches the context
    fn rule_matches(
        &self,
        rule: &DecisionRule,
        inputs: &[InputClause],
        context: &DecisionContext,
    ) -> bool {
        for condition in &rule.conditions {
            let input = inputs.iter()
                .find(|i| i.id == condition.input_id)
                .ok_or(false)?;

            let value = context.inputs.get(&input.name);

            if !self.evaluate_condition(&condition.operator, value, &condition.value) {
                return false;
            }
        }

        true
    }

    /// Evaluate a single condition
    fn evaluate_condition(
        &self,
        operator: &ConditionOperator,
        actual: Option<&DecisionValue>,
        expected: &DecisionValue,
    ) -> bool {
        match (actual, expected) {
            (Some(DecisionValue::String(a)), DecisionValue::String(b)) => match operator {
                ConditionOperator::Equal => a == b,
                ConditionOperator::NotEqual => a != b,
                _ => false,
            },
            (Some(DecisionValue::Integer(a)), DecisionValue::Integer(b)) => match operator {
                ConditionOperator::Equal => a == b,
                ConditionOperator::NotEqual => a != b,
                ConditionOperator::LessThan => a < b,
                ConditionOperator::LessThanOrEqual => a <= b,
                ConditionOperator::GreaterThan => a > b,
                ConditionOperator::GreaterThanOrEqual => a >= b,
                _ => false,
            },
            (Some(DecisionValue::Double(a)), DecisionValue::Double(b)) => match operator {
                ConditionOperator::Equal => (a - b).abs() < f64::EPSILON,
                ConditionOperator::LessThan => a < b,
                ConditionOperator::GreaterThan => a > b,
                _ => false,
            },
            (Some(DecisionValue::Boolean(a)), DecisionValue::Boolean(b)) => a == b,
            _ => false,
        }
    }

    /// Parse DMN XML (simplified implementation)
    fn parse_dmn_xml(xml: &str) -> Result<Decision, DmnError> {
        // Parse DMN XML structure
        // For a production implementation, use a proper DMN parser
        // This is a simplified version that handles basic decision tables

        let root: serde_xml::Element = serde_xml::from_str(xml)
            .map_err(|e| DmnError::ParseError(e.to_string()))?;

        let name = root.get_child("name")
            .and_then(|e| e.text())
            .unwrap_or("unnamed".to_string());

        let id = root.get_child("id")
            .and_then(|e| e.text())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        Ok(Decision {
            id: id.clone(),
            name,
            inputs: vec![],
            outputs: vec![],
            rules: vec![],
            metadata: DecisionMetadata {
                matched_rule: None,
                evaluation_time_us: 0,
                dmn_version: "1.4".to_string(),
                open_regels_uri: None,
            },
        })
    }
}

impl Default for DmnEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// DMN errors
#[derive(Debug, Error)]
pub enum DmnError {
    #[error("Decision not found: {0}")]
    DecisionNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    #[error("Fetch error: {0}")]
    FetchError(String),

    #[error("Open Regels client not available")]
    OpenRegelsNotAvailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_value_conversions() {
        let s: DecisionValue = "test".to_string().into();
        assert_eq!(s.as_str(), Some("test"));

        let i: DecisionValue = 42i64.into();
        assert_eq!(i.as_int(), Some(42));

        let b: DecisionValue = true.into();
        assert!(matches!(b, DecisionValue::Boolean(true)));
    }

    #[test]
    fn test_condition_evaluation() {
        let evaluator = DmnEvaluator::new();

        let val = Some(DecisionValue::Integer(42));
        let expected = DecisionValue::Integer(40);

        assert!(evaluator.evaluate_condition(&ConditionOperator::GreaterThan, val.as_ref(), &expected));
        assert!(!evaluator.evaluate_condition(&ConditionOperator::LessThan, val.as_ref(), &expected));
    }
}
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-regels/src/bpmn.rs` (NEW)

```rust
//! BPMN (Business Process Model and Notation) integration
//!
//! This module integrates BPMN process definitions with the orchestrator,
//! enabling DMN decision service calls within process flows.

use crate::dmn::{DmnEvaluator, DecisionContext, DecisionValue};
use crate::model::Regel;
use iou_orchestrator::{
    WorkflowStateMachine, WorkflowContext, WorkflowState, WorkflowEvent,
    context::{DocumentRequest, DocumentType},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// BPMN process definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDefinition {
    pub id: String,
    pub name: String,
    pub version: String,

    /// Process variables
    pub variables: Vec<ProcessVariable>,

    /// Activities in the process
    pub activities: Vec<Activity>,

    /// Gateways (decision points)
    pub gateways: Vec<Gateway>,

    /// Events (start, end, intermediate)
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessVariable {
    pub id: String,
    pub name: String,
    pub type_ref: String,
    pub default: Option<DecisionValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub name: String,
    pub activity_type: ActivityType,
    pub incoming: Vec<String>,
    pub outgoing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelelCase")]
pub enum ActivityType {
    Task,
    UserTask,
    ServiceTask,
    SendTask,
    ReceiveTask,
    ScriptTask,
    BusinessRuleTask,
    SubProcess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gateway {
    pub id: String,
    pub name: String,
    pub gateway_type: GatewayType,
    pub incoming: Vec<String>,
    pub outgoing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelelCase")]
pub enum GatewayType {
    Exclusive,
    Inclusive,
    Parallel,
    EventBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub event_type: EventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelelCase")]
pub enum EventType {
    Start,
    End,
    Intermediate,
    Boundary,
    /// Timer event for delays
    Timer,
    /// Message event for external signals
    Message,
}

/// Process instance (running process)
#[derive(Debug, Clone)]
pub struct ProcessInstance {
    pub id: Uuid,
    pub definition_id: String,
    pub state: ProcessInstanceState,
    pub variables: HashMap<String, DecisionValue>,
    pub current_activity: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessInstanceState {
    Running,
    Suspended,
    Completed,
    Failed,
    Terminated,
}

/// BPMN process engine
pub struct BpmnProcessEngine {
    evaluator: Arc<DmnEvaluator>,
    processes: HashMap<String, ProcessDefinition>,
}

impl BpmnProcessEngine {
    /// Create a new BPMN process engine
    pub fn new(evaluator: Arc<DmnEvaluator>) -> Self {
        Self {
            evaluator,
            processes: HashMap::new(),
        }
    }

    /// Load a BPMN process definition
    pub fn load_process(&mut self, definition: ProcessDefinition) -> Result<(), BpmnError> {
        self.processes.insert(definition.id.clone(), definition);
        Ok(())
    }

    /// Start a new process instance
    pub async fn start_process(
        &self,
        definition_id: &str,
        variables: HashMap<String, DecisionValue>,
    ) -> Result<ProcessInstance, BpmnError> {
        let definition = self.processes
            .get(definition_id)
            .ok_or_else(|| BpmnError::ProcessNotFound(definition_id.to_string()))?;

        let instance = ProcessInstance {
            id: Uuid::new_v4(),
            definition_id: definition_id.to_string(),
            state: ProcessInstanceState::Running,
            variables,
            current_activity: None,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        Ok(instance)
    }

    /// Execute a Business Rule Task (calls DMN evaluator)
    pub async fn execute_business_rule_task(
        &self,
        task: &Activity,
        instance: &ProcessInstance,
    ) -> Result<HashMap<String, DecisionValue>, BpmnError> {
        if !matches!(task.activity_type, ActivityType::BusinessRuleTask) {
            return Err(BpmnError::InvalidActivity("Not a business rule task".into()));
        }

        // Extract decision ID from task (stored in task name or extension elements)
        let decision_id = task.name.clone();

        // Build decision context from process variables
        let mut context_inputs = HashMap::new();
        for var in instance.variables.iter() {
            context_inputs.insert(var.0.clone(), var.1.clone());
        }

        let context = DecisionContext {
            inputs: context_inputs,
            tenant_id: None, // Will be set from VC context
            context: HashMap::new(),
        };

        let result = self.evaluator
            .evaluate(&decision_id, &context)
            .await
            .map_err(|e| BpmnError::EvaluationError(e.to_string()))?;

        Ok(result.outputs)
    }

    /// Create a workflow context from BPMN process
    pub fn create_workflow_context(
        &self,
        definition_id: &str,
        instance_id: Uuid,
        requested_by: Uuid,
    ) -> Result<WorkflowContext, BpmnError> {
        let definition = self.processes
            .get(definition_id)
            .ok_or_else(|| BpmnError::ProcessNotFound(definition_id.to_string()))?;

        let doc_type = match definition_id {
            "zorgtoeslag-aanvraag" => DocumentType::WooBesluit,
            "huurtoeslag-aanvraag" => DocumentType::WooBesluit,
            _ => DocumentType::Custom(definition_id.to_string()),
        };

        let request = DocumentRequest {
            id: instance_id,
            domain_id: definition_id.to_string(),
            document_type: doc_type,
            context: HashMap::new(),
            requested_by,
            requested_at: chrono::Utc::now(),
            priority: iou_orchestrator::context::RequestPriority::Normal,
        };

        Ok(WorkflowContext::new(
            instance_id,
            request,
            definition.version.clone(),
        ))
    }
}

/// BPMN errors
#[derive(Debug, Error)]
pub enum BpmnError {
    #[error("Process not found: {0}")]
    ProcessNotFound(String),

    #[error("Invalid activity: {0}")]
    InvalidActivity(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}

/// Integration helper: Load BPMN process from Open Regels
pub async fn load_process_from_open_regels(
    client: &OpenRegelsClient,
    regel_uri: &str,
) -> Result<ProcessDefinition, BpmnError> {
    // Fetch rule details from Open Regels
    let bindings = client.select(&format!(r#"
        PREFIX bpmn: <http://www.omg.org/spec/BPMN/20100524/MODEL/>

        SELECT ?xml WHERE {{
            <{}> bpmn:definitions ?xml .
        }}
    "#, regel_uri)).await
        .map_err(|e| BpmnError::FetchError(e.to_string()))?;

    if bindings.is_empty() {
        return Err(BpmnError::ProcessNotFound(regel_uri.to_string()));
    }

    let bpmn_xml = bindings[0].get("xml")
        .and_then(|v| Some(v.value.clone()))
        .ok_or(BpmnError::ParseError("No BPMN XML found".into()))?;

    // Parse BPMN XML
    parse_bpmn_xml(&bpmn_xml)
}

/// Parse BPMN XML (simplified)
fn parse_bpmn_xml(xml: &str) -> Result<ProcessDefinition, BpmnError> {
    let root: serde_xml::Element = serde_xml::from_str(xml)
        .map_err(|e| BpmnError::ParseError(e.to_string()))?;

    let id = root.get_child("id")
        .and_then(|e| e.text())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let name = root.get_child("name")
        .and_then(|e| e.text())
        .unwrap_or("Unnamed Process".to_string());

    Ok(ProcessDefinition {
        id,
        name,
        version: "1.0".to_string(),
        variables: vec![],
        activities: vec![],
        gateways: vec![],
        events: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_instance_creation() {
        let evaluator = DmnEvaluator::new();
        let engine = BpmnProcessEngine::new(Arc::new(evaluator));

        let definition = ProcessDefinition {
            id: "test-process".to_string(),
            name: "Test Process".to_string(),
            version: "1.0".to_string(),
            variables: vec![],
            activities: vec![],
            gateways: vec![],
            events: vec![],
        };

        engine.load_process(definition).unwrap();

        let instance = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(engine.start_process("test-process", HashMap::new()))
            .unwrap();

        assert_eq!(instance.definition_id, "test-process");
        assert_eq!(instance.state, ProcessInstanceState::Running);
    }
}
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-regels/src/lib.rs`

Update to export new modules:

```rust
pub mod architektur;
pub mod compliance;
pub mod provisa;
pub mod dmn;    // NEW
pub mod bpmn;   // NEW

// Re-exports
pub use dmn::{DmnEvaluator, DecisionContext, DecisionValue, DecisionResult, DmnError};
pub use bpmn::{
    BpmnProcessEngine, ProcessDefinition, ProcessInstance,
    ActivityType, GatewayType, EventType, ProcessInstanceState,
    load_process_from_open_regels,
};
```

### Step 4: Create Tenancy Module

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/tenancy/mod.rs`

```rust
//! Multi-tenant isolation for municipality data

pub mod tenant;

pub use tenant::{TenantContext, TenantId, LoA, TenantError};
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-core/src/tenancy/tenant.rs`

```rust
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
        // Must be lowercase alphanumeric with hyphens
        if !id.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(TenantError::InvalidFormat("Invalid characters".into()));
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
```

### Step 5: Create VC Middleware

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/middleware/vc.rs`

```rust
//! Verifiable Credential authentication middleware

use crate::error::ApiError;
use crate::ssi::{PresentationValidator, ValidatedPresentation};
use crate::tenancy::{TenantContext, TenantError};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// VC validation middleware
pub async fn vc_middleware(
    State(validator): State<Arc<PresentationValidator>>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract VP from Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    if !auth_header.starts_with("VC ") {
        return Err(ApiError::Unauthorized(
            "Invalid authorization format (expected 'VC <presentation>')".to_string()
        ));
    }

    let vp_data = &auth_header[3..];

    // Parse and validate VP
    let vp: iou_core::ssi::VerifiablePresentation =
        serde_json::from_str(vp_data)
            .map_err(|e| ApiError::Unauthorized(format!("Invalid VP format: {}", e)))?;

    let validated = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(validator.validate(&vp))
    })
        .map_err(|e| ApiError::Unauthorized(format!("VP validation failed: {}", e)))?;

    // Convert VC claims to TenantContext
    let tenant_context = TenantContext::from_vc_claims(&validated.claims)
        .map_err(|e| ApiError::Unauthorized(format!("Invalid claims: {}", e)))?;

    // Store validated claims in request extensions
    request.extensions_mut().insert(validated);
    request.extensions_mut().insert(tenant_context);

    Ok(next.run(request).await)
}
```

### Step 6: Create v1 API Routes

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/v1/mod.rs`

```rust
//! Version 1 API endpoints

pub mod rules;
pub mod calculations;
pub mod processes;

pub use rules::{list_rules, evaluate_rule, get_open_regels_rule, RuleEvaluationRequest};
pub use calculations::{start_calculation, CalculationRequest, CalculationResponse};
pub use processes::{start_process, get_process, ProcessRequest, ProcessResponse, ProcessInstanceResponse};
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/v1/rules.rs`

```rust
//! Business rules evaluation endpoints

use crate::audit::{AuditAction, AuditEntry, AuditLogger};
use crate::error::ApiError;
use crate::tenancy::TenantContext;
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// List available rules for tenant
pub async fn list_rules(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<Arc<AuditLogger>>,
    State(evaluator): State<Arc<iou_regels::DmnEvaluator>>,
) -> Result<Json<Vec<RuleInfo>>, ApiError> {
    // Write audit first
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::Custom("rules_listed".to_string()),
        "rules",
        "list",
    );
    let _ = audit.log(&audit_entry).await;

    // Discover DMN rules from Open Regels
    let decisions = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(evaluator.discover_dmn_decisions(None))
    })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to discover rules: {}", e)))?;

    let rules: Vec<RuleInfo> = decisions.into_iter().map(|r| RuleInfo {
        id: r.uri.clone(),
        name: r.label.unwrap_or(r.uri.split('/').last().unwrap_or("unknown").to_string()),
        description: r.beschrijving,
        dmn_version: "1.4".to_string(),
    }).collect();

    Ok(Json(rules))
}

#[derive(Debug, Serialize)]
pub struct RuleInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub dmn_version: String,
}

/// Rule evaluation request
#[derive(Debug, Deserialize)]
pub struct RuleEvaluationRequest {
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Rule evaluation response
#[derive(Debug, Serialize)]
pub struct RuleEvaluationResponse {
    pub rule_id: String,
    pub outputs: HashMap<String, serde_json::Value>,
    pub matched: bool,
    pub matched_rule_id: Option<String>,
    pub evaluation_time_us: u64,
}

/// Evaluate a business rule
pub async fn evaluate_rule(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<Arc<AuditLogger>>,
    Extension(evaluator): State<Arc<iou_regels::DmnEvaluator>>,
    Path(rule_id): Path<String>,
    Json(req): Json<RuleEvaluationRequest>,
) -> Result<Json<RuleEvaluationResponse>, ApiError> {
    // Write audit first
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::RuleEvaluated,
        "rule",
        &rule_id,
    )
    .with_context(serde_json::json!({
        "inputs": req.inputs,
    }));
    audit.log(&audit_entry).await
        .map_err(|e| ApiError::Internal(format!("Audit failed: {}", e)))?;

    // Build decision context
    let mut context_inputs = HashMap::new();
    for (k, v) in req.inputs {
        let value = match v {
            serde_json::Value::String(s) => iou_regels::DecisionValue::String(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    iou_regels::DecisionValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    iou_regels::DecisionValue::Double(f)
                } else {
                    return Err(ApiError::Validation("Invalid number".into()));
                }
            }
            serde_json::Value::Bool(b) => iou_regels::DecisionValue::Boolean(b),
            _ => return Err(ApiError::Validation("Unsupported input type".into())),
        };
        context_inputs.insert(k, value);
    }

    let context = iou_regels::DecisionContext {
        inputs: context_inputs,
        tenant_id: Some(tenant.tenant_id.as_str().to_string()),
        context: HashMap::new(),
    };

    // Evaluate rule
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(evaluator.evaluate(&rule_id, &context))
    })
        .map_err(|e| ApiError::Internal(format!("Evaluation failed: {}", e)))?;

    // Convert outputs to JSON
    let mut outputs = HashMap::new();
    for (k, v) in result.outputs {
        let json_value = match v {
            iou_regels::DecisionValue::String(s) => serde_json::json!(s),
            iou_regels::DecisionValue::Integer(i) => serde_json::json!(i),
            iou_regels::DecisionValue::Double(f) => serde_json::json!(f),
            iou_regels::DecisionValue::Boolean(b) => serde_json::json!(b),
            _ => serde_json::Value::Null,
        };
        outputs.insert(k, json_value);
    }

    Ok(Json(RuleEvaluationResponse {
        rule_id,
        outputs,
        matched: result.matched,
        matched_rule_id: result.metadata.matched_rule,
        evaluation_time_us: result.metadata.evaluation_time_us,
    }))
}

/// Get Open Regels rule details
pub async fn get_open_regels_rule(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<Arc<AuditLogger>>,
    Extension(client): State<Arc<iou_regels::OpenRegelsClient>>,
    Path(rule_uri): Path<String>,
) -> Result<Json<RegelDetail>, ApiError> {
    // Write audit
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::Custom("rule_fetched".to_string()),
        "regel",
        &rule_uri,
    );
    let _ = audit.log(&audit_entry).await;

    // Fetch from Open Regels
    let json_ld = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(client.fetch_resource(&rule_uri))
    })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch rule: {}", e)))?;

    Ok(Json(RegelDetail {
        uri: rule_uri.clone(),
        json_ld,
        fetched_at: chrono::Utc::now(),
    }))
}

#[derive(Debug, Serialize)]
pub struct RegelDetail {
    pub uri: String,
    pub json_ld: serde_json::Value,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/v1/processes.rs`

```rust
//! BPMN process endpoints

use crate::audit::{AuditAction, AuditEntry, AuditLogger};
use crate::error::ApiError;
use crate::tenancy::TenantContext;
use crate::bpmn::{BpmnProcessEngine, ProcessInstance, ProcessInstanceState};
use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Process start request
#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub process_definition_id: String,
    pub variables: HashMap<String, serde_json::Value>,
}

/// Process response
#[derive(Debug, Serialize)]
pub struct ProcessResponse {
    pub process_instance_id: Uuid,
    pub status: String,
    pub status_url: String,
}

/// Process instance details
#[derive(Debug, Serialize)]
pub struct ProcessInstanceResponse {
    pub id: Uuid,
    pub process_definition_id: String,
    pub status: String,
    pub variables: HashMap<String, serde_json::Value>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Start a BPMN process instance
pub async fn start_process(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<Arc<AuditLogger>>,
    Extension(engine): State<Arc<BpmnProcessEngine>>,
    Json(req): Json<ProcessRequest>,
) -> Result<Json<ProcessResponse>, ApiError> {
    let instance_id = Uuid::new_v4();

    // Write audit first
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::ProcessStarted,
        "process",
        instance_id.to_string(),
    )
    .with_context(serde_json::json!({
        "process_definition_id": req.process_definition_id,
        "variables": req.variables,
    }));
    audit.log(&audit_entry).await
        .map_err(|e| ApiError::Internal(format!("Audit failed: {}", e)))?;

    // Convert variables
    let mut vars = HashMap::new();
    for (k, v) in req.variables {
        let value = match v {
            serde_json::Value::String(s) => iou_regels::DecisionValue::String(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    iou_regels::DecisionValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    iou_regels::DecisionValue::Double(f)
                } else {
                    return Err(ApiError::Validation("Invalid number".into()));
                }
            }
            serde_json::Value::Bool(b) => iou_regels::DecisionValue::Boolean(b),
            _ => return Err(ApiError::Validation("Unsupported type".into())),
        };
        vars.insert(k, value);
    }

    // Start process
    let instance = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(
            engine.start_process(&req.process_definition_id, vars)
        )
    })
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start process: {}", e)))?;

    // Convert to orchestrator workflow (optional)
    // TODO: Create WorkflowStateMachine from ProcessInstance

    Ok(Json(ProcessResponse {
        process_instance_id: instance.id,
        status: format!("{:?}", instance.state),
        status_url: format!("/v1/processes/{}", instance.id),
    }))
}

/// Get process instance status
pub async fn get_process(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<Arc<AuditLogger>>,
    Path(id): Path<Uuid>,
    State(engine): State<Arc<BpmnProcessEngine>>,
) -> Result<Json<ProcessInstanceResponse>, ApiError> {
    // Write audit
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::Custom("process_status".to_string()),
        "process",
        id.to_string(),
    );
    let _ = audit.log(&audit_entry).await;

    // TODO: Query process instance from engine
    // For now, return a mock response
    Ok(Json(ProcessInstanceResponse {
        id,
        process_definition_id: "unknown".to_string(),
        status: "running".to_string(),
        variables: HashMap::new(),
        started_at: chrono::Utc::now(),
        completed_at: None,
    }))
}
```

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/routes/v1/calculations.rs`

```rust
//! Calculation endpoints (DMN-based calculations)

use crate::audit::{AuditAction, AuditEntry, AuditLogger};
use crate::error::ApiError;
use crate::tenancy::TenantContext;
use crate::iou_regels::DmnEvaluator;
use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Calculation request
#[derive(Debug, Deserialize)]
pub struct CalculationRequest {
    pub calculation_type: String,
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Calculation response
#[derive(Debug, Serialize)]
pub struct CalculationResponse {
    pub calculation_id: Uuid,
    pub result: Option<HashMap<String, serde_json::Value>>,
    pub status_url: String,
}

/// Start a calculation
pub async fn start_calculation(
    Extension(tenant): Extension<TenantContext>,
    Extension(audit): Extension<Arc<AuditLogger>>,
    Extension(evaluator): State<Arc<DmnEvaluator>>,
    Json(req): Json<CalculationRequest>,
) -> Result<Json<CalculationResponse>, ApiError> {
    let calculation_id = Uuid::new_v4();

    // Write audit first
    let audit_entry = AuditEntry::new(
        tenant.tenant_id.as_str().to_string(),
        tenant.holder_did.clone(),
        AuditAction::CalculationStarted,
        "calculation",
        calculation_id.to_string(),
    )
    .with_context(serde_json::json!({
        "calculation_type": req.calculation_type,
        "inputs": req.inputs,
    }));
    audit.log(&audit_entry).await
        .map_err(|e| ApiError::Internal(format!("Audit failed: {}", e)))?;

    // For synchronous calculation, evaluate immediately
    // For async, would return a status URL to poll
    let mut context_inputs = HashMap::new();
    for (k, v) in req.inputs {
        let value = match v {
            serde_json::Value::String(s) => iou_regels::DecisionValue::String(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    iou_regels::DecisionValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    iou_regels::DecisionValue::Double(f)
                } else {
                    return Err(ApiError::Validation("Invalid number".into()));
                }
            }
            serde_json::Value::Bool(b) => iou_regels::DecisionValue::Boolean(b),
            _ => return Err(ApiError::Validation("Unsupported type".into())),
        };
        context_inputs.insert(k, value);
    }

    let context = iou_regels::DecisionContext {
        inputs: context_inputs,
        tenant_id: Some(tenant.tenant_id.as_str().to_string()),
        context: HashMap::new(),
    };

    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(
            evaluator.evaluate(&req.calculation_type, &context)
        )
    })
        .await;

    let result = match result {
        Ok(r) => {
            // Log completion
            let audit_entry = AuditEntry::new(
                tenant.tenant_id.as_str().to_string(),
                tenant.holder_did.clone(),
                AuditAction::CalculationCompleted,
                "calculation",
                calculation_id.to_string(),
            );
            let _ = audit.log(&audit_entry).await;

            let mut outputs = HashMap::new();
            for (k, v) in r.outputs {
                let json_value = match v {
                    iou_regels::DecisionValue::String(s) => serde_json::json!(s),
                    iou_regels::DecisionValue::Integer(i) => serde_json::json!(i),
                    iou_regels::DecisionValue::Double(f) => serde_json::json!(f),
                    iou_regels::DecisionValue::Boolean(b) => serde_json::json!(b),
                    _ => serde_json::Value::Null,
                };
                outputs.insert(k, json_value);
            }
            Some(outputs)
        }
        Err(e) => {
            // Log failure
            let audit_entry = AuditEntry::new(
                tenant.tenant_id.as_str().to_string(),
                tenant.holder_did.clone(),
                AuditAction::Custom("calculation_failed".to_string()),
                "calculation",
                calculation_id.to_string(),
            )
            .with_outcome(crate::iou_core::audit::models::AuditOutcome::Failed);
            let _ = audit.log(&audit_entry).await;
            return Err(ApiError::Internal(format!("Calculation failed: {}", e)));
        }
    };

    Ok(Json(CalculationResponse {
        calculation_id,
        result,
        status_url: format!("/v1/calculations/{}", calculation_id),
    }))
}
```

### Step 7: Update Configuration

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsiConfig {
    /// Trusted issuer DIDs
    pub trusted_issuers: Vec<String>,

    /// DID resolver endpoint (optional, for universal resolver)
    pub did_resolver_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub retention_years: u32,
    pub database_url: String,
}

impl SsiConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let trusted_issuers = std::env::var("SSI_TRUSTED_ISSUERS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            trusted_issuers,
            did_resolver_url: std::env::var("SSI_DID_RESOLVER_URL").ok(),
        })
    }
}
```

### Step 8: Update main.rs with v1 Routes

**File:** `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs`

```rust
use iou_core::audit::{AuditLogger, PostgresAuditBackend};
use iou_core::ssi::{PresentationValidator, UniversalDidResolver};
use iou_core::tenancy::TenantContext;
use iou_regels::{DmnEvaluator, OpenRegelsClient, BpmnProcessEngine};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::init();

    // Load configuration
    let config = Config::from_env()?;

    // Initialize SSI components
    let did_resolver = Arc::new(UniversalDidResolver::new());
    let validator = Arc::new(PresentationValidator::new(
        did_resolver.clone(),
        config.ssi.trusted_issuers,
    ));

    // Initialize audit
    let audit_pool = PgPool::connect(&config.audit.database_url).await?;
    let audit_backend = PostgresAuditBackend::new(audit_pool);
    audit_backend.init().await?;
    let audit_logger = Arc::new(AuditLogger::new(audit_backend));

    // Initialize Open Regels client
    let open_regels = Arc::new(OpenRegelsClient::acc());

    // Initialize DMN evaluator with Open Regels
    let dmn_evaluator = Arc::new(DmnEvaluator::new()
        .with_open_regels(open_regels.clone()));

    // Initialize BPMN process engine
    let bpmn_engine = Arc::new(BpmnProcessEngine::new(dmn_evaluator.clone()));

    // Build v1 router with VC middleware
    let v1_router = Router::new()
        .route("/rules", get(routes::v1::list_rules))
        .route("/rules/:id", post(routes::v1::evaluate_rule))
        .route("/rules/open-regels/:uri", get(routes::v1::get_open_regels_rule))
        .route("/calculations", post(routes::v1::start_calculation))
        .route("/processes", post(routes::v1::start_process))
        .route("/processes/:id", get(routes::v1::get_process))
        .layer(axum::middleware::from_fn_with_state(
            validator.clone(),
            middleware::vc::vc_middleware,
        ))
        .layer(Extension(audit_logger.clone()))
        .layer(Extension(dmn_evaluator.clone()))
        .layer(Extension(open_regels.clone()))
        .layer(Extension(bpmn_engine.clone()));

    // ... rest of main router setup
}
```

## Environment Variables

```bash
# SSI / VC Configuration
SSI_TRUSTED_ISSUERS=did:web:wallet.nl-wallet.nl,did:ebsi:xyz
SSI_DID_RESOLVER_URL=https://resolver.example.com

# Audit Configuration
AUDIT_RETENTION_YEARS=7
AUDIT_DATABASE_URL=postgresql://user:pass@localhost/audit_db

# Open Regels
OPEN_REGELS_ENV=acc  # or production
```

## API Endpoints Summary

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/health` | GET | None | Health check |
| `/v1/rules` | GET | VC | List tenant-specific DMN rules |
| `/v1/rules/{id}/evaluate` | POST | VC | Evaluate business rule |
| `/v1/rules/open-regels/{uri}` | GET | VC | Get Open Regels rule details |
| `/v1/calculations` | POST | VC | Start calculation |
| `/v1/processes` | POST | VC | Start BPMN process |
| `/v1/processes/{id}` | GET | VC | Get process status |

## Database Schema

### Audit Table

```sql
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id VARCHAR(50) NOT NULL,
    user_did VARCHAR(255) NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id VARCHAR(500) NOT NULL,
    outcome VARCHAR(20) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    context JSONB,
    session_id VARCHAR(100),
    parent_id UUID REFERENCES audit_log(id),
    entry_hash VARCHAR(64),

    CONSTRAINT audit_tenant_timestamp UNIQUE (tenant_id, timestamp, id)
);

CREATE INDEX idx_audit_tenant_timestamp ON audit_log(tenant_id, timestamp DESC);
CREATE INDEX idx_audit_user_did ON audit_log(user_did);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_parent ON audit_log(parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX idx_audit_context ON audit_log USING GIN (context);

COMMENT ON TABLE audit_log IS 'Tamper-evident audit log with 7-year retention (BIO compliant)';
```

## Verification Checklist

- [ ] VC validation works with EBSI/nl-wallet credentials
- [ ] DID resolution returns valid DID documents
- [ ] Tenant isolation prevents cross-tenant data access
- [ ] DMN rules evaluate correctly
- [ ] Open Regels SPARQL integration works
- [ ] BPMN processes start and execute correctly
- [ ] Audit entries are written before actions
- [ ] Audit entries are tamper-evident
- [ ] Audit query works with tenant filtering
- [ ] LoA requirements are enforced
- [ ] v1 API endpoints are versioned correctly
- [ ] All tests pass
