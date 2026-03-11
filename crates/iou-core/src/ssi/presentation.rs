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
