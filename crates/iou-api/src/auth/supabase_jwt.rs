//! Supabase JWT Token Verification
//!
//! Provides verification for JWT tokens issued by Supabase Auth.
//! Supabase tokens follow a specific structure with custom claims
//! for organization_id and other application-specific data.

use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::env;

use crate::middleware::{AuthContext, Role};

/// Errors that can occur during Supabase JWT verification
#[derive(Debug, thiserror::Error)]
pub enum SupabaseAuthError {
    #[error("Missing JWT secret")]
    MissingSecret,

    #[error("Invalid token format: {0}")]
    InvalidToken(String),

    #[error("Token signature verification failed")]
    SignatureFailed,

    #[error("Token has expired")]
    TokenExpired,

    #[error("Invalid token issuer: expected {expected}, got {found}")]
    InvalidIssuer { expected: String, found: String },

    #[error("Missing required claim: {0}")]
    MissingClaim(String),

    #[error("Invalid user ID format")]
    InvalidUserId,

    #[error("Invalid organization ID format")]
    InvalidOrganizationId,
}

/// Standard claims in a Supabase JWT token
///
/// Supabase issues JWTs with the following structure:
/// - `sub`: User ID (UUID)
/// - `aud`: Audience (typically "authenticated")
/// - `role`: User role (typically "authenticated")
/// - `email`: User email
/// - `exp`: Expiration timestamp
/// - `iat`: Issued at timestamp
///
/// Custom claims (via JWT hooks in Supabase):
/// - `organization_id`: User's organization UUID
/// - `clearance`: Security clearance level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseClaims {
    /// User ID (UUID from auth.users.id)
    pub sub: String,

    /// Audience (typically "authenticated")
    pub aud: String,

    /// User role (typically "authenticated")
    pub role: String,

    /// User email address
    pub email: String,

    /// Organization ID (custom claim)
    #[serde(default)]
    pub organization_id: Option<String>,

    /// Security clearance level (custom claim)
    #[serde(default)]
    pub clearance: Option<String>,

    /// User roles/permissions (custom claim)
    #[serde(default)]
    pub app_roles: Option<Vec<String>>,

    /// Token expiration timestamp
    pub exp: i64,

    /// Token issued at timestamp
    pub iat: i64,

    /// Token issuer
    pub iss: String,
}

/// JWT verifier for Supabase Auth tokens
pub struct SupabaseJwtVerifier {
    /// Decoding key for verifying JWT signatures
    decoding_key: DecodingKey,

    /// Expected issuer for Supabase tokens
    issuer: String,

    /// Expected audience for Supabase tokens
    audience: String,
}

impl SupabaseJwtVerifier {
    /// Create a new Supabase JWT verifier
    ///
    /// The JWT secret is loaded from the `SUPABASE_JWT_SECRET` environment variable.
    /// This is typically the JWT secret from your Supabase project settings.
    ///
    /// # Errors
    ///
    /// Returns `SupabaseAuthError::MissingSecret` if the environment variable is not set.
    pub fn new() -> Result<Self, SupabaseAuthError> {
        let jwt_secret = env::var("SUPABASE_JWT_SECRET")
            .or_else(|_| env::var("SUPABASE_JWT")) // Fallback alternative name
            .map_err(|_| SupabaseAuthError::MissingSecret)?;

        Self::with_secret(&jwt_secret)
    }

    /// Create a verifier with a specific JWT secret
    ///
    /// This is useful for testing or when the secret is managed externally.
    pub fn with_secret(jwt_secret: &str) -> Result<Self, SupabaseAuthError> {
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());

        Ok(Self {
            decoding_key,
            issuer: "supabase".to_string(),
            audience: "authenticated".to_string(),
        })
    }

    /// Create a verifier with custom issuer and audience
    ///
    /// Use this if your Supabase instance uses non-standard values.
    pub fn with_config(
        jwt_secret: &str,
        issuer: String,
        audience: String,
    ) -> Result<Self, SupabaseAuthError> {
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());

        Ok(Self {
            decoding_key,
            issuer,
            audience,
        })
    }

    /// Verify and decode a Supabase JWT token
    ///
    /// This method:
    /// 1. Verifies the token signature
    /// 2. Checks expiration
    /// 3. Validates issuer and audience
    /// 4. Returns the decoded claims
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The token is malformed
    /// - The signature is invalid
    /// - The token has expired
    /// - The issuer or audience doesn't match
    pub fn verify(&self, token: &str) -> Result<SupabaseClaims, SupabaseAuthError> {
        // Set up validation first (validates algorithm)
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;

        // Decode and verify
        let token_data = decode::<SupabaseClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        SupabaseAuthError::TokenExpired
                    }
                    _ => SupabaseAuthError::SignatureFailed,
                }
            })?;

        Ok(token_data.claims)
    }

    /// Verify a token and convert it to an AuthContext
    ///
    /// This is a convenience method that verifies the token and
    /// converts the Supabase claims to the internal AuthContext format.
    pub fn verify_to_auth_context(&self, token: &str) -> Result<AuthContext, SupabaseAuthError> {
        let claims = self.verify(token)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| SupabaseAuthError::InvalidUserId)?;

        let organization_id = claims.organization_id
            .ok_or_else(|| SupabaseAuthError::MissingClaim("organization_id".to_string()))?
            .parse::<Uuid>()
            .map_err(|_| SupabaseAuthError::InvalidOrganizationId)?;

        // Convert app_roles to our Role enum
        let roles: Vec<Role> = claims
            .app_roles
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| r.parse().ok())
            .collect();

        // Add default role if none specified
        let roles = if roles.is_empty() {
            vec![Role::DomainViewer] // Default role
        } else {
            roles
        };

        Ok(AuthContext {
            user_id,
            email: claims.email,
            organization_id,
            roles,
        })
    }
}

/// Middleware integration helper
///
/// Use this function in your auth middleware to verify Supabase tokens.
pub fn verify_supabase_token(token: &str) -> Result<AuthContext, SupabaseAuthError> {
    let verifier = SupabaseJwtVerifier::new()?;
    verifier.verify_to_auth_context(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    /// Helper to create a test token
    fn create_test_token(
        user_id: &str,
        email: &str,
        organization_id: &str,
        secret: &str,
    ) -> String {
        let claims = SupabaseClaims {
            sub: user_id.to_string(),
            aud: "authenticated".to_string(),
            role: "authenticated".to_string(),
            email: email.to_string(),
            organization_id: Some(organization_id.to_string()),
            clearance: Some("intern".to_string()),
            app_roles: Some(vec!["domain_viewer".to_string()]),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
            iat: chrono::Utc::now().timestamp(),
            iss: "supabase".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn test_supabase_jwt_verification() {
        let secret = "test-secret-key-for-testing";
        let verifier = SupabaseJwtVerifier::with_secret(secret).unwrap();

        let user_id = Uuid::new_v4().to_string();
        let email = "test@example.com";
        let org_id = Uuid::new_v4().to_string();

        let token = create_test_token(&user_id, email, &org_id, secret);

        let claims = verifier.verify(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.organization_id.as_deref(), Some(org_id.as_str()));
    }

    #[test]
    fn test_supabase_jwt_to_auth_context() {
        let secret = "test-secret-key-for-testing";
        let verifier = SupabaseJwtVerifier::with_secret(secret).unwrap();

        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let org_id = Uuid::new_v4();

        let token = create_test_token(&user_id.to_string(), email, &org_id.to_string(), secret);

        let auth_context = verifier.verify_to_auth_context(&token).unwrap();
        assert_eq!(auth_context.user_id, user_id);
        assert_eq!(auth_context.email, email);
        assert_eq!(auth_context.organization_id, org_id);
        assert!(!auth_context.roles.is_empty());
    }

    #[test]
    fn test_invalid_token_rejected() {
        let verifier = SupabaseJwtVerifier::with_secret("test-secret").unwrap();
        let result = verifier.verify("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret_rejected() {
        let verifier = SupabaseJwtVerifier::with_secret("secret-one").unwrap();
        let token = create_test_token(
            &Uuid::new_v4().to_string(),
            "test@example.com",
            &Uuid::new_v4().to_string(),
            "secret-two", // Different secret!
        );

        let result = verifier.verify(&token);
        assert!(matches!(result, Err(SupabaseAuthError::SignatureFailed)));
    }

    #[test]
    fn test_expired_token_rejected() {
        let secret = "test-secret-key";
        let verifier = SupabaseJwtVerifier::with_secret(secret).unwrap();

        let claims = SupabaseClaims {
            sub: Uuid::new_v4().to_string(),
            aud: "authenticated".to_string(),
            role: "authenticated".to_string(),
            email: "test@example.com".to_string(),
            organization_id: Some(Uuid::new_v4().to_string()),
            clearance: None,
            app_roles: None,
            exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(), // Expired
            iat: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp(),
            iss: "supabase".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        ).unwrap();

        let result = verifier.verify(&token);
        assert!(matches!(result, Err(SupabaseAuthError::TokenExpired)));
    }
}
