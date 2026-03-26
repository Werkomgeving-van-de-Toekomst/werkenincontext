//! Routes voor de identity / NL Wallet-module.

use axum::Json;

use crate::error::ApiError;
use crate::id::nl_wallet::WalletWebSessionResponse;

#[derive(Debug, serde::Deserialize)]
pub struct NlWalletStartBody {
    pub usecase: String,
}

/// `POST /api/id/nl-wallet/sessions`
///
/// Compatibel met [wallet_web](https://github.com/MinBZK/nl-wallet/tree/main/wallet_web):
/// body `{"usecase":"..."}` → response `status_url` + `session_token`.
pub async fn nl_wallet_create_session(
    Json(body): Json<NlWalletStartBody>,
) -> Result<Json<WalletWebSessionResponse>, ApiError> {
    let base = std::env::var("NL_WALLET_VERIFICATION_SERVER_URL")
        .map_err(|_| {
            ApiError::ServiceUnavailable(
                "NL_WALLET_VERIFICATION_SERVER_URL is niet gezet; zie crates/iou-api/src/id/mod.rs"
                    .into(),
            )
        })?;

    let base = base.trim();
    if base.is_empty() {
        return Err(ApiError::ServiceUnavailable(
            "NL_WALLET_VERIFICATION_SERVER_URL is leeg".into(),
        ));
    }

    let usecase = body.usecase.trim();
    if usecase.is_empty() {
        return Err(ApiError::Validation("usecase mag niet leeg zijn".into()));
    }

    let out = crate::id::nl_wallet::start_disclosure_session(base, usecase)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(out))
}
