//! SPARQL client voor de Open Regels Linked Data endpoint
//!
//! Biedt een dunne wrapper om SPARQL SELECT en CONSTRUCT queries
//! te sturen naar regels.overheid.nl of de acc-omgeving.
//!
//! Dit is een native-only module (niet beschikbaar voor WASM builds).

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;
#[cfg(not(target_arch = "wasm32"))]
use serde::Deserialize;
#[cfg(not(target_arch = "wasm32"))]
use tracing::{debug, instrument};

// ── SPARQL response types ────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
pub(crate) struct SparqlResponse {
    pub results: SparqlResults,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize)]
pub(crate) struct SparqlResults {
    pub bindings: Vec<HashMap<String, SparqlValue>>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Deserialize, Clone)]
pub struct SparqlValue {
    pub value: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[cfg(not(target_arch = "wasm32"))]
pub type Bindings = Vec<HashMap<String, SparqlValue>>;

// ── Client ───────────────────────────────────────────────────────────────────

/// HTTP client voor de Open Regels SPARQL endpoints
///
/// # Omgevingen
///
/// Gebruik `OpenRegelsClient::acc()` tijdens ontwikkeling en
/// `OpenRegelsClient::productie()` voor productiegebruik.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct OpenRegelsClient {
    http: Client,
    pub endpoint: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl OpenRegelsClient {
    /// Maak een client aan voor de **acceptatieomgeving** (veilig om mee te experimenteren)
    pub fn acc() -> Self {
        Self::new("https://api.open-regels.triply.cc/datasets/stevengort/DMN-discovery/services/DMN-discovery/sparql")
    }

    /// Maak een client aan voor de **productieomgeving**
    pub fn productie() -> Self {
        Self::new("https://regels.overheid.nl/lab/sparql")
    }

    /// Maak een client aan met een expliciet endpoint URL
    pub fn new(endpoint: &str) -> Self {
        let http = Client::builder()
            .user_agent("iou-modern/iou-regels")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("HTTP client aanmaken mislukt");

        Self {
            http,
            endpoint: endpoint.to_string(),
        }
    }

    /// Voer een SPARQL SELECT query uit; geeft de ruwe bindings terug
    #[instrument(skip(self, sparql), fields(endpoint = %self.endpoint))]
    pub async fn select(&self, sparql: &str) -> Result<Bindings> {
        debug!("SPARQL SELECT:\n{sparql}");

        let response = self
            .http
            .get(&self.endpoint)
            .query(&[("query", sparql)])
            .header("Accept", "application/sparql-results+json")
            .send()
            .await?
            .error_for_status()?;

        let parsed: SparqlResponse = response.json().await?;
        Ok(parsed.results.bindings)
    }

    /// Voer een SPARQL CONSTRUCT query uit; geeft JSON-LD terug
    #[instrument(skip(self, sparql), fields(endpoint = %self.endpoint))]
    pub async fn construct(&self, sparql: &str) -> Result<serde_json::Value> {
        debug!("SPARQL CONSTRUCT:\n{sparql}");

        let response = self
            .http
            .get(&self.endpoint)
            .query(&[("query", sparql)])
            .header("Accept", "application/ld+json")
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    /// Haal een specifieke resource op via content negotiation (JSON-LD)
    pub async fn fetch_resource(&self, uri: &str) -> Result<serde_json::Value> {
        let response = self
            .http
            .get(uri)
            .header("Accept", "application/ld+json, application/json")
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }
}
