//! Agentic tools voor de Open Regels integratie
//!
//! Elke publieke methode is ontworpen als een tool die een LLM-agent
//! kan aanroepen via een tool_use loop (Anthropic API, Ollama, etc.).
//!
//! # Patroon
//!
//! ```text
//! LLM besluit tool te roepen
//!   → OpenRegelsTools::zoek_regels("beslagvrije voet")
//!   → [Vec<Regel>] als JSON terug naar LLM
//!   → LLM kiest specifieke regel-URI
//!   → OpenRegelsTools::haal_regel_details(uri)
//!   → [RegelDetail met JSON-LD] → LLM redeneert over toepassing
//! ```

use anyhow::Result;
use tracing::info;

use crate::client::OpenRegelsClient;
use crate::model::{Regel, RegelDetail, bindings_naar_regels};

/// Verzameling van agentic tools voor het Open Regels regelregister
///
/// Instantieer met [`OpenRegelsTools::acc()`] voor ontwikkeling of
/// [`OpenRegelsTools::productie()`] voor productiegebruik.
#[derive(Clone)]
pub struct OpenRegelsTools {
    client: OpenRegelsClient,
}

impl OpenRegelsTools {
    /// Gebruik de acceptatieomgeving (aanbevolen tijdens ontwikkeling)
    pub fn acc() -> Self {
        Self { client: OpenRegelsClient::acc() }
    }

    /// Gebruik de productieomgeving
    pub fn productie() -> Self {
        Self { client: OpenRegelsClient::productie() }
    }

    /// Gebruik een aangepaste client (bijv. voor tests)
    pub fn with_client(client: OpenRegelsClient) -> Self {
        Self { client }
    }

    // ── Tool: zoek_regels ────────────────────────────────────────────────────

    /// Zoek regelspecificaties (services) op basis van een vrije zoekterm
    ///
    /// Zoekt in titels en beschrijvingen (case-insensitief).
    /// Geeft maximaal 20 resultaten terug als compacte [`Regel`] structs.
    ///
    /// # Voorbeelden van zoektermen
    /// - `"zorgtoeslag"` — zorgtoeslag berekening
    /// - `"participatie"` — participatiewet regels
    /// - `"aow"` — AOW leeftijd berekening
    /// - `"boom"` — boomkap regels
    pub async fn zoek_regels(&self, zoekterm: &str) -> Result<Vec<Regel>> {
        info!("zoek_regels: '{zoekterm}'");

        let query = format!(
            r#"
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX cpsv:    <http://purl.org/vocab/cpsv#>

SELECT DISTINCT ?regel ?label ?beschrijving ?wet ?eigenaar
WHERE {{
  ?regel a cpsv:PublicService .
  OPTIONAL {{ ?regel dcterms:title       ?label        }}
  OPTIONAL {{ ?regel dcterms:description ?beschrijving }}
  OPTIONAL {{ ?regel dcterms:publisher   ?eigenaar     }}
  FILTER (
    CONTAINS(LCASE(STR(?label)),        LCASE("{term}"))
    || CONTAINS(LCASE(STR(?beschrijving)), LCASE("{term}"))
  )
}}
LIMIT 20
"#,
            term = zoekterm
        );

        let bindings = self.client.select(&query).await?;
        Ok(bindings_naar_regels(bindings))
    }

    // ── Tool: regels_voor_wet ────────────────────────────────────────────────

    /// Haal alle regels op die gekoppeld zijn aan een specifieke wet
    ///
    /// Gebruik een Juriconnect deeplink als `juriconnect_url`, bijv.:
    /// - `"https://wetten.overheid.nl/BWBR0015703"` (Participatiewet)
    /// - `"https://wetten.overheid.nl/BWBR0005537"` (Algemene wet bestuursrecht)
    pub async fn regels_voor_wet(&self, juriconnect_url: &str) -> Result<Vec<Regel>> {
        info!("regels_voor_wet: '{juriconnect_url}'");

        let query = format!(
            r#"
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX cpsv:    <http://purl.org/vocab/cpsv#>
PREFIX eli:     <http://data.europa.eu/eli/ontology#>

SELECT ?regel ?label ?beschrijving ?wet ?eigenaar
WHERE {{
  ?regel a cpsv:PublicService ;
         dcterms:source <{wet}> .
  OPTIONAL {{ ?regel dcterms:title       ?label        }}
  OPTIONAL {{ ?regel dcterms:description ?beschrijving }}
  OPTIONAL {{ ?regel dcterms:publisher   ?eigenaar     }}
  BIND(<{wet}> AS ?wet)
}}
LIMIT 50
"#,
            wet = juriconnect_url
        );

        let bindings = self.client.select(&query).await?;
        Ok(bindings_naar_regels(bindings))
    }

    // ── Tool: regels_voor_domein ─────────────────────────────────────────────

    /// Haal regels op voor een specifiek beleidsdomein
    ///
    /// Zoekt in titels, beschrijvingen en onderwerpen, bijv.:
    /// `"zorg"`, `"participatie"`, `"belasting"`, `"subsidie"`
    pub async fn regels_voor_domein(&self, domein: &str) -> Result<Vec<Regel>> {
        info!("regels_voor_domein: '{domein}'");

        let query = format!(
            r#"
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX cpsv:    <http://purl.org/vocab/cpsv#>

SELECT DISTINCT ?regel ?label ?beschrijving ?wet ?eigenaar
WHERE {{
  ?regel a cpsv:PublicService .
  OPTIONAL {{ ?regel dcterms:title       ?label        }}
  OPTIONAL {{ ?regel dcterms:description ?beschrijving }}
  OPTIONAL {{ ?regel dcterms:subject     ?onderwerp    }}
  OPTIONAL {{ ?regel dcterms:publisher   ?eigenaar     }}
  FILTER (
    CONTAINS(LCASE(STR(?onderwerp)), LCASE("{domein}"))
    || CONTAINS(LCASE(STR(?label)),     LCASE("{domein}"))
    || CONTAINS(LCASE(STR(?beschrijving)), LCASE("{domein}"))
  )
}}
LIMIT 30
"#,
            domein = domein
        );

        let bindings = self.client.select(&query).await?;
        Ok(bindings_naar_regels(bindings))
    }

    // ── Tool: haal_regel_details ─────────────────────────────────────────────

    /// Haal de volledige regelspecificatie op als JSON-LD
    ///
    /// Gebruik dit nadat `zoek_regels` of `regels_voor_wet` een
    /// interessante URI heeft opgeleverd. De JSON-LD payload bevat
    /// de volledige FLINT- of DMN-logica die de agent kan gebruiken
    /// voor redenering en uitleg.
    pub async fn haal_regel_details(&self, regel_uri: &str) -> Result<RegelDetail> {
        info!("haal_regel_details: '{regel_uri}'");

        let query = format!(
            r#"
CONSTRUCT {{ <{uri}> ?p ?o }}
WHERE     {{ <{uri}> ?p ?o }}
"#,
            uri = regel_uri
        );

        let json_ld = self.client.construct(&query).await?;

        let meta_query = format!(
            r#"
PREFIX dcterms: <http://purl.org/dc/terms/>

SELECT ?label ?beschrijving ?eigenaar
WHERE {{
  OPTIONAL {{ <{uri}> dcterms:title       ?label        }}
  OPTIONAL {{ <{uri}> dcterms:description ?beschrijving }}
  OPTIONAL {{ <{uri}> dcterms:publisher   ?eigenaar     }}
}}
LIMIT 1
"#,
            uri = regel_uri
        );

        let mut bindings = self.client.select(&meta_query).await?;
        let meta = bindings.pop();

        let get = |key: &str| -> Option<String> {
            meta.as_ref()?.get(key).map(|v| v.value.clone())
        };

        use crate::model::RegelType;

        let regel = crate::model::Regel {
            uri: regel_uri.to_string(),
            label: get("label"),
            beschrijving: get("beschrijving"),
            juridische_bron: None,
            regel_type: RegelType::default(),
            eigenaar: get("eigenaar"),
        };

        Ok(RegelDetail {
            regel,
            json_ld,
            opgehaald_op: chrono::Utc::now(),
        })
    }

    // ── Tool: beschikbare_regelsets ──────────────────────────────────────────

    /// Geef een overzicht van beschikbare regelsets (PublicServices) in het register
    ///
    /// Handig als eerste stap in een agentic workflow om te ontdekken
    /// welke regelsets beschikbaar zijn.
    pub async fn beschikbare_regelsets(&self) -> Result<Vec<Regel>> {
        info!("beschikbare_regelsets");

        let query = r#"
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX cpsv:    <http://purl.org/vocab/cpsv#>

SELECT DISTINCT ?regel ?label ?beschrijving ?eigenaar
WHERE {
  ?regel a cpsv:PublicService .
  OPTIONAL { ?regel dcterms:title       ?label        }
  OPTIONAL { ?regel dcterms:description ?beschrijving }
  OPTIONAL { ?regel dcterms:publisher   ?eigenaar     }
}
ORDER BY ?label
LIMIT 50
"#;

        let bindings = self.client.select(query).await?;
        Ok(bindings_naar_regels(bindings))
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test — vereist netwerkverbinding
    #[tokio::test]
    #[ignore = "vereist netwerkverbinding"]
    async fn test_zoek_regels_integratie() {
        let tools = OpenRegelsTools::acc();
        let regels = tools.zoek_regels("zorgtoeslag").await.unwrap();
        assert!(!regels.is_empty(), "Verwacht ten minste één regel");
        assert!(regels[0].uri.starts_with("https://"));
    }

    #[tokio::test]
    #[ignore = "vereist netwerkverbinding"]
    async fn test_beschikbare_regelsets() {
        let tools = OpenRegelsTools::acc();
        let sets = tools.beschikbare_regelsets().await.unwrap();
        println!("Beschikbare regelsets: {}", sets.len());
        for s in &sets {
            println!("  - {} ({})", s.label.as_deref().unwrap_or("?"), s.uri);
        }
    }
}
