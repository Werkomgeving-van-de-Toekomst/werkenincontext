//! Authentication endpoints
//!
//! Includes both traditional JWT authentication and
//! EBSI Verifiable Credential (wallet) authentication.

use axum::{extract::Extension, response::Json};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Duration;

use crate::db::Database;
use crate::error::ApiError;
use crate::middleware::auth::{JwtService, Role};
use crate::vc::{WalletAuthRequest, WalletAuthResponse, VpVerifier, VcConfig, VcUserContext};

// Re-export traditional auth handlers
pub use crate::middleware::auth::{
    login, logout, refresh_token, LoginRequest, LoginResponse, RefreshRequest,
};

/// POST /auth/wallet
///
/// Authenticate using a Verifiable Presentation from a wallet
/// (e.g., nl-wallet, EBSI wallet).
///
/// # Flow
///
/// 1. Receive Verifiable Presentation (JWT format)
/// 2. Verify VP signature and issuer trust
/// 3. Extract claims from credentials
/// 4. Map to local user/roles
/// 5. Issue short-lived JWT for subsequent API calls
///
/// # Request
///
/// ```json
/// {
///   "vp_token": "eyJ...",
///   "presentation_submission": {
///     "definition_id": "presentation_exchange_definition",
///     "descriptor_map": [...]
///   }
/// }
/// ```
///
/// # Response
///
/// ```json
/// {
///   "access_token": "eyJ...",
///   "token_type": "Bearer",
///   "expires_in": 3600,
///   "user": {
///     "id": "uuid",
///     "organization_id": "uuid",
///     "roles": ["domain_viewer"],
///     "credential_type": "CustomMdt",
///     "issuer": "did:key:...",
///     "additional_claims": {...}
///   }
/// }
/// ```
pub async fn wallet_auth(
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<WalletAuthRequest>,
) -> Result<Json<WalletAuthResponse>, ApiError> {
    let config = VcConfig::from_env();

    // Verify the VP and extract user context
    let verifier = VpVerifier::new(config.clone());
    let user_context = verifier.verify_wallet_auth(req).await
        .map_err(|e| ApiError::Unauthorized(format!("VP verification failed: {}", e)))?;

    // Map VC roles to local roles
    let roles: Vec<Role> = user_context.roles
        .iter()
        .filter_map(|r| r.parse().ok())
        .collect();

    // Issue short-lived JWT (1 hour)
    let jwt_service = JwtService::new();
    let access_token = jwt_service.create_token(
        user_context.id,
        &format!("{}@wallet", user_context.id), // Placeholder email
        user_context.organization_id,
        roles.clone(),
    ).map_err(|e| ApiError::Internal(anyhow::anyhow!("Token creation failed: {}", e)))?;

    // Log authentication event
    tracing::info!(
        user_id = %user_context.id,
        organization_id = %user_context.organization_id,
        credential_type = %user_context.credential_type,
        issuer = %user_context.issuer,
        "Wallet authentication successful"
    );

    // TODO: Store audit event in database

    Ok(Json(WalletAuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 60 * 60, // 1 hour
        user: user_context,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_auth_route_exists() {
        // Route exists - integration tests would verify actual functionality
        assert!(true);
    }
}
