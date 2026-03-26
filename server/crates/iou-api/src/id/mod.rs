//! Identiteit en koppeling met [NL Wallet](https://github.com/MinBZK/nl-wallet).
//!
//! # Twee paden
//!
//! 1. **Browser (`wallet_web`)** — Het custom element `nl-wallet-button` doet `POST` naar jouw
//!    backend met `{"usecase":"..."}`. Die backend moet een sessie starten bij de NL Wallet
//!    **verification_server** en antwoorden met `status_url` + `session_token` (zie
//!    [`nl_wallet::start_disclosure_session`] en de upstream
//!    [wallet_web README](https://github.com/MinBZK/nl-wallet/blob/main/wallet_web/README.md)).
//!
//! 2. **API-token** — [`crate::routes::auth::wallet_auth`] (`POST /api/auth/wallet`) valideert
//!    een ingediende Verifiable Presentation en geeft een IOU JWT uit.
//!
//! # Omgeving
//!
//! | Variabele | Doel |
//! |-----------|------|
//! | `NL_WALLET_VERIFICATION_SERVER_URL` | Basis-URL van `verification_server` (bijv. `http://127.0.0.1:3011` na `./scripts/nl-wallet-verification-up.sh`). Vereist voor `POST .../id/nl-wallet/sessions`. Zie repo **`docs/nl-wallet-e2e.md`**. |
//! | `VC_TRUSTED_ISSUERS` | Komma-gescheiden trusted issuer DIDs voor wallet-auth. |
//! | `VC_STRICT_MODE` | `true` / `1` weigert issuers buiten de trustlijst. |

pub mod nl_wallet;
