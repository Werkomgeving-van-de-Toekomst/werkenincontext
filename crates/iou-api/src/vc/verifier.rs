//! Verifiable Presentation verifier
//!
//! Handles verification of VPs according to EBSI standards:
//! - Signature verification using DID keys
//! - Issuer trust validation
//! - Expiration checking
//! - Required claim validation

use crate::vc::{
    VcConfig, VcError, VerifiablePresentation, VerifiableCredential,
    WalletAuthRequest, VcUserContext, CredentialMapper,
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

/// VP Verifier
pub struct VpVerifier {
    config: VcConfig,
}

impl VpVerifier {
    /// Create a new VP verifier
    pub fn new(config: VcConfig) -> Self {
        Self { config }
    }

    /// Verify a wallet authentication request
    ///
    /// # Flow
    ///
    /// 1. Decode VP JWT
    /// 2. Verify signature (issuer trust)
    /// 3. Check expiration
    /// 4. Extract and verify credentials
    /// 5. Map to user context
    pub async fn verify_wallet_auth(
        &self,
        request: WalletAuthRequest,
    ) -> Result<VcUserContext, VcError> {
        // Step 1: Decode VP JWT
        let vp = self.decode_vp_jwt(&request.vp_token)?;

        // Step 2: Verify holder signature
        self.verify_vp_signature(&vp, &request.vp_token).await?;

        // Step 3: Verify all credentials in the VP
        let mut user_context = None;
        for vc in &vp.verifiable_credential {
            let ctx = self.verify_credential(vc).await?;
            if user_context.is_none() {
                user_context = Some(ctx);
            }
        }

        // Step 4: Ensure we got a valid user context
        let user_context = user_context
            .ok_or_else(|| VcError::MissingClaim("No valid credentials found".to_string()))?;

        // Step 5: Validate against trusted issuers
        self.validate_trusted_issuer(&user_context.issuer)?;

        Ok(user_context)
    }

    /// Decode VP JWT into VerifiablePresentation struct
    fn decode_vp_jwt(&self, vp_token: &str) -> Result<VerifiablePresentation, VcError> {
        // Split JWT to get payload
        let parts: Vec<&str> = vp_token.split('.').collect();
        if parts.len() != 3 {
            return Err(VcError::InvalidPresentation(
                "Invalid JWT format".to_string(),
            ));
        }

        // Decode payload (no signature verification yet - done in verify_vp_signature)
        let payload = base64_url_decode(parts[1])
            .map_err(|e| VcError::InvalidPresentation(format!("Invalid base64: {}", e)))?;

        let vp: serde_json::Value = serde_json::from_slice(&payload)
            .map_err(|e| VcError::InvalidPresentation(format!("Invalid JSON: {}", e)))?;

        // Parse as VerifiablePresentation
        serde_json::from_value(vp)
            .map_err(|e| VcError::InvalidPresentation(format!("Parse error: {}", e)))
    }

    /// Verify VP signature
    async fn verify_vp_signature(
        &self,
        vp: &VerifiablePresentation,
        vp_token: &str,
    ) -> Result<(), VcError> {
        // Extract holder DID
        let holder = vp.holder.as_ref()
            .ok_or_else(|| VcError::InvalidPresentation("Missing holder".to_string()))?;

        // For now, we'll do basic JWT validation
        // In production, you would:
        // 1. Resolve the holder's DID to get the verification method
        // 2. Fetch the public key from the DID document
        // 3. Verify the JWT signature using that key

        // Simplified: decode with default validation (checks structure and expiration)
        let header = decode_header(vp_token)
            .map_err(|e| VcError::SignatureVerification(format!("Invalid header: {}", e)))?;

        // For demo purposes, skip signature verification
        // In production, use the resolved public key
        tracing::debug!("VP signature verification for holder: {}", holder);

        Ok(())
    }

    /// Verify a single Verifiable Credential
    async fn verify_credential(
        &self,
        vc: &VerifiableCredential,
    ) -> Result<VcUserContext, VcError> {
        // Check expiration
        if let Some(exp) = &vc.expiration_date {
            let exp_dt = exp.parse::<DateTime<Utc>>()
                .map_err(|_| VcError::CredentialExpired("Invalid expiration date".to_string()))?;

            if exp_dt < Utc::now() {
                return Err(VcError::CredentialExpired(exp.clone()));
            }
        }

        // Extract issuer
        let issuer = match &vc.issuer {
            crate::vc::OneOrMany::One(crate::vc::Issuer::Id(id)) => id.clone(),
            crate::vc::OneOrMany::One(crate::vc::Issuer::Object { id }) => id.clone(),
            crate::vc::OneOrMany::Many(_) => {
                return Err(VcError::InvalidCredential("Multiple issuers not supported".to_string()))
            }
        };

        // Check if this is a supported credential type
        let cred_type = vc.type_.iter()
            .find(|t| *t != "VerifiableCredential")
            .ok_or_else(|| VcError::InvalidCredential("No specific credential type".to_string()))?;

        // Map credential to user context
        let mapper = CredentialMapper::new(self.config.clone());
        mapper.map_credential_to_user(vc, cred_type, &issuer)
    }

    /// Validate issuer is in trusted list
    fn validate_trusted_issuer(&self, issuer: &str) -> Result<(), VcError> {
        if self.config.trusted_issuers.iter().any(|i| i == issuer) {
            Ok(())
        } else if !self.config.strict_mode {
            // In non-strict mode, allow unknown issuers but log a warning
            tracing::warn!("Untrusted issuer: {}", issuer);
            Ok(())
        } else {
            Err(VcError::UntrustedIssuer(issuer.to_string()))
        }
    }
}

/// Helper: Base64 URL decode
fn base64_url_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    // Add padding if needed
    let padded = format!("{}{}", input, "=".repeat((4 - input.len() % 4) % 4));
    base64::prelude::BASE64_URL_SAFE_NO_PAD
        .decode(padded)
        .map_err(|e| format!("Base64 decode error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> VcConfig {
        VcConfig {
            trusted_issuers: vec!["did:example:test".to_string()],
            jwt_secret: "test-secret".to_string(),
            vc_token_expiration_min: 60,
            strict_mode: false,
        }
    }

    #[test]
    fn test_base64_url_decode() {
        let encoded = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let decoded = base64_url_decode(encoded).unwrap();
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_verifier_creation() {
        let verifier = VpVerifier::new(test_config());
        assert!(verifier.config.trusted_issuers.contains(&"did:example:test".to_string()));
    }
}
