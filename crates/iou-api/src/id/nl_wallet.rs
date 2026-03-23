//! HTTP-client naar de NL Wallet `verification_server` (`POST /disclosure/sessions`).
//!
//! Zie MinBZK `demo_relying_party` en `wallet_web` `SessionResponse`.

use serde::Deserialize;

/// Antwoord aan de browser zoals `wallet_web` verwacht (`lib/models/relying_party.ts`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct WalletWebSessionResponse {
    pub status_url: String,
    pub session_token: String,
}

#[derive(Debug, Deserialize)]
struct VerificationStartResponse {
    session_token: String,
}

fn join_base_path(base: &str, path: &str) -> Result<String, String> {
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        return Err("lege basis-URL".into());
    }
    let path = path.trim_start_matches('/');
    Ok(format!("{}/{}", base, path))
}

/// Start een disclosure-sessie bij de verification_server en bouw de `status_url` voor polling.
///
/// Verwacht dezelfde JSON als de NL Wallet-server: alleen `session_token` in de response.
pub async fn start_disclosure_session(
    verification_server_base: &str,
    usecase: &str,
) -> Result<WalletWebSessionResponse, String> {
    let url = join_base_path(verification_server_base, "disclosure/sessions")?;

    let body = serde_json::json!({
        "usecase": usecase,
        "dcql_query": serde_json::Value::Null,
        "return_url_template": serde_json::Value::Null,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("verification_server request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!(
            "verification_server {}: {}",
            status.as_u16(),
            text.chars().take(500).collect::<String>()
        ));
    }

    let parsed: VerificationStartResponse = response
        .json()
        .await
        .map_err(|e| format!("invalid JSON from verification_server: {}", e))?;

    let status_url = join_base_path(
        verification_server_base,
        &format!("disclosure/sessions/{}", parsed.session_token),
    )?;

    Ok(WalletWebSessionResponse {
        status_url,
        session_token: parsed.session_token,
    })
}

#[cfg(test)]
mod tests {
    use super::join_base_path;

    #[test]
    fn join_base_path_trims_slashes() {
        assert_eq!(
            join_base_path("http://localhost:3004/", "/disclosure/sessions").unwrap(),
            "http://localhost:3004/disclosure/sessions"
        );
    }
}
