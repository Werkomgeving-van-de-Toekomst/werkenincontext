//! Purpose Binding - Doelbinding voor AVG/GDPR compliance
//!
//! Deze module implementeert het purpose binding mechanisme volgens IHH01:
//! "Vastlegging en uitwisseling gegevens gebonden aan doelbinding"
//!
//! # Concepts
//!
//! - **Purpose**: Een specifiek doel voor gegevensverwerking (bijv. ZAAK_AFHANDELING)
//! - **LawfulBasis**: De wettelijke grondslag volgens AVG Art. 6
//! - **PurposeRegistry**: Beheert alle beschikbare purposes
//! - **PurposeValidation**: Valideert of een purpose geldig is voor een request

mod approval;
mod categories;
mod registry;
mod validation;

pub use approval::{
    PurposeApprovalError, PurposeApprovalStatus, PurposeApprovalSummary, PurposeRequest,
    purpose_approval_workflow,
};
pub use categories::{standard_purposes, PurposeCategory, STANDARD_PURPOSES};
pub use registry::{Purpose, PurposeError, PurposeId, PurposeRegistry};
pub use validation::{PurposeValidation, ValidationContext, ValidationResult};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Wettelijke grondslag voor gegevensverwerking (AVG Art. 6)
///
/// Elke purpose moet een van deze grondslagen hebben om AVG-compliant te zijn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LawfulBasis {
    /// Art. 6.a: Toestemming van de betrokkene
    Toestemming,

    /// Art. 6.b: Uitvoering van een overeenkomst
    Overeenkomst,

    /// Art. 6.c: Wettelijke verplichting (meest voorkomend voor overheid)
    WettelijkeVerplichting,

    /// Art. 6.d: Bescherming vitale belangen
    VitaleBelangen,

    /// Art. 6.e: Algemeen belang / openbaar gezag (overheidstaken)
    AlgemeenBelang,

    /// Art. 6.f: Gerechtvaardigd belang
    GerechtvaardigdBelang,
}

impl LawfulBasis {
    /// Geeft terug of deze grondslag specifiek is voor de overheid
    pub fn is_government_specific(&self) -> bool {
        matches!(
            self,
            LawfulBasis::WettelijkeVerplichting | LawfulBasis::AlgemeenBelang
        )
    }

    /// Geeft de AVG Artikel referentie
    pub fn avg_article(&self) -> &'static str {
        match self {
            LawfulBasis::Toestemming => "AVG Art. 6.a",
            LawfulBasis::Overeenkomst => "AVG Art. 6.b",
            LawfulBasis::WettelijkeVerplichting => "AVG Art. 6.c",
            LawfulBasis::VitaleBelangen => "AVG Art. 6.d",
            LawfulBasis::AlgemeenBelang => "AVG Art. 6.e",
            LawfulBasis::GerechtvaardigdBelang => "AVG Art. 6.f",
        }
    }
}

/// Data categorieën die bij een purpose kunnen horen
///
/// Deze worden gebruikt om te controleren of een purpose geschikt is
/// voor het type data dat wordt verwerkt.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DataCategory {
    /// Algemene persoonsgegevens (NAW, contact)
    Persoonsgegevens,

    /// Bijzondere persoonsgegevens (AVG Art. 9)
    BijzondereGegevens,

    /// Strafrechtelijke gegevens (AVG Art. 10)
    StrafrechtelijkeGegevens,

    /// Zaak- en dossierdata
    ZaakData,

    /// Beleidsdocumenten en notities
    BeleidsData,

    /// Financiële administratie
    FinancieelData,

    /// BAG/ADRES gegevens
    AdresData,

    /// Documenten en bijlagen
    DocumentData,

    /// Communicatie (email, chat, telefoon)
    CommunicatieData,

    /// Locatie/track-and-trace data
    LocatieData,

    /// Geaggregeerde/anonieme data
    GeaggregeerdeData,
}

impl DataCategory {
    /// Geeft terug of deze categorie bijzondere gegevens bevat
    pub fn requires_extra_protection(&self) -> bool {
        matches!(
            self,
            DataCategory::BijzondereGegevens | DataCategory::StrafrechtelijkeGegevens
        )
    }

    /// Geeft alle categorieën die persoonsgegevens bevatten
    pub fn personal_data_categories() -> &'static [DataCategory] {
        &[
            DataCategory::Persoonsgegevens,
            DataCategory::BijzondereGegevens,
            DataCategory::StrafrechtelijkeGegevens,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lawful_basis_serialization() {
        let basis = LawfulBasis::WettelijkeVerplichting;
        let json = serde_json::to_string(&basis).unwrap();
        assert_eq!(json, "\"wettelijke_verplichting\"");

        let parsed: LawfulBasis = serde_json::from_str("\"algemeen_belang\"").unwrap();
        assert_eq!(parsed, LawfulBasis::AlgemeenBelang);
    }

    #[test]
    fn test_lawful_basis_government_specific() {
        assert!(LawfulBasis::WettelijkeVerplichting.is_government_specific());
        assert!(LawfulBasis::AlgemeenBelang.is_government_specific());
        assert!(!LawfulBasis::Toestemming.is_government_specific());
    }

    #[test]
    fn test_data_category_protection() {
        assert!(DataCategory::BijzondereGegevens.requires_extra_protection());
        assert!(DataCategory::StrafrechtelijkeGegevens.requires_extra_protection());
        assert!(!DataCategory::ZaakData.requires_extra_protection());
    }
}
