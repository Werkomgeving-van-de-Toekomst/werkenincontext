//! Domeinmodellen voor Open Regels regelspecificaties
//!
//! Modellering conform het vocabulaire van regels.overheid.nl
//! en de FLINT/DMN specificatiestandaarden.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Enumeraties ───────────────────────────────────────────────────────────────

/// Type regelspecificatie conform Open Regels standaard
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum RegelType {
    /// FLINT — Formal Language for Identifying Normative Text
    Flint,
    /// Decision Model and Notation
    Dmn,
    /// ReSpec publicatieformaat
    ReSpec,
    /// Leesbare tekst (nog niet geformaliseerd)
    #[default]
    Tekst,
    /// Onbekend type
    Onbekend,
}

impl RegelType {
    pub fn from_uri(uri: &str) -> Self {
        let lower = uri.to_lowercase();
        if lower.contains("flint") {
            Self::Flint
        } else if lower.contains("dmn") {
            Self::Dmn
        } else if lower.contains("respec") {
            Self::ReSpec
        } else {
            Self::Onbekend
        }
    }
}

// ── Juriconnect ──────────────────────────────────────────────────────────────

/// Verwijzing naar een wettekst via de Juriconnect standaard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuriconnectRef {
    /// Volledige Juriconnect URI (wetten.overheid.nl deeplink)
    pub uri: String,
    /// Mensleesbare aanduiding, bijv. "Participatiewet art. 31"
    pub label: Option<String>,
    /// BWB-identifier, bijv. "BWBR0015703"
    pub bwb_id: Option<String>,
}

impl JuriconnectRef {
    pub fn new(uri: impl Into<String>) -> Self {
        let uri = uri.into();
        // Extraheer BWB-id uit URL indien aanwezig
        let bwb_id = uri
            .split('/')
            .find(|s| s.starts_with("BWBR") || s.starts_with("BWBV"))
            .map(|s| s.to_string());

        Self {
            uri,
            label: None,
            bwb_id,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

// ── Hoofdmodellen ─────────────────────────────────────────────────────────────

/// Een regelspecificatie uit het Open Regels register
///
/// Beknopte weergave — geschikt als zoekresultaat of lijstitem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regel {
    /// Linked Data URI van de regelspecificatie
    pub uri: String,
    /// Mensleesbaar label
    pub label: Option<String>,
    /// Korte beschrijving van de regel
    pub beschrijving: Option<String>,
    /// Verwijzing naar de juridische grondslag
    pub juridische_bron: Option<JuriconnectRef>,
    /// Type specificatie (FLINT, DMN, etc.)
    pub regel_type: RegelType,
    /// Publicerende organisatie
    pub eigenaar: Option<String>,
}

impl Regel {
    /// Maak een minimale Regel aan vanuit een URI
    pub fn from_uri(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            label: None,
            beschrijving: None,
            juridische_bron: None,
            regel_type: RegelType::default(),
            eigenaar: None,
        }
    }
}

/// Volledige regeldetails inclusief JSON-LD payload
///
/// Wordt gebruikt wanneer een agent de volledige FLINT-logica
/// nodig heeft om een besluit te onderbouwen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegelDetail {
    /// Beknopte regelinfo
    pub regel: Regel,
    /// Volledige JSON-LD representatie van de regelspecificatie
    pub json_ld: serde_json::Value,
    /// Tijdstip waarop de details zijn opgehaald
    pub opgehaald_op: DateTime<Utc>,
}

// ── Conversie vanuit SPARQL bindings ─────────────────────────────────────────

use std::collections::HashMap;
use crate::client::SparqlValue;

/// Converteer SPARQL-bindings naar een lijst van [`Regel`] structs
pub fn bindings_naar_regels(bindings: Vec<HashMap<String, SparqlValue>>) -> Vec<Regel> {
    bindings
        .into_iter()
        .map(|mut b| {
            let uri = b.remove("regel").map(|v| v.value).unwrap_or_default();
            let juridische_bron = b
                .remove("wet")
                .map(|v| JuriconnectRef::new(v.value));

            let regel_type = b
                .get("regelType")
                .map(|v| RegelType::from_uri(&v.value))
                .unwrap_or_default();

            Regel {
                uri,
                label: b.remove("label").map(|v| v.value),
                beschrijving: b.remove("beschrijving").map(|v| v.value),
                juridische_bron,
                regel_type,
                eigenaar: b.remove("eigenaar").map(|v| v.value),
            }
        })
        .collect()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_juriconnect_bwb_extractie() {
        let r = JuriconnectRef::new(
            "https://wetten.overheid.nl/BWBR0015703/2024-01-01"
        );
        assert_eq!(r.bwb_id.as_deref(), Some("BWBR0015703"));
    }

    #[test]
    fn test_regel_type_from_uri() {
        assert_eq!(RegelType::from_uri("https://example.org/flint/regel"), RegelType::Flint);
        assert_eq!(RegelType::from_uri("https://example.org/dmn/besluit"), RegelType::Dmn);
        assert_eq!(RegelType::from_uri("https://example.org/onbekend"), RegelType::Onbekend);
    }

    #[test]
    fn test_bindings_naar_regels() {
        let mut b: HashMap<String, SparqlValue> = HashMap::new();
        b.insert("regel".into(), SparqlValue {
            value: "https://regels.overheid.nl/id/regel/bvv".into(),
            kind: "uri".into(),
        });
        b.insert("label".into(), SparqlValue {
            value: "Beslagvrije voet".into(),
            kind: "literal".into(),
        });

        let regels = bindings_naar_regels(vec![b]);
        assert_eq!(regels.len(), 1);
        assert_eq!(regels[0].label.as_deref(), Some("Beslagvrije voet"));
    }
}
