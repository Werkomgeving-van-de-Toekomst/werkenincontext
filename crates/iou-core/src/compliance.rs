//! Compliance types voor Nederlandse wetgeving
//!
//! Dit module bevat types voor:
//! - Wet open overheid (Woo): openbaarmaking van overheidsinformatie
//! - Algemene verordening gegevensbescherming (AVG): privacy
//! - Archiefwet: bewaring en vernietiging van overheidsinformatie

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Classificatieniveau voor informatiebeveiliging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, Default)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Classification {
    /// Openbaar: vrij toegankelijk
    Openbaar,
    /// Intern: alleen binnen organisatie
    #[default]
    Intern,
    /// Vertrouwelijk: beperkte toegang
    Vertrouwelijk,
    /// Geheim: strikt beperkt
    Geheim,
}

/// Privacy niveau volgens AVG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, Default)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyLevel {
    /// Geen persoonsgegevens
    #[default]
    Geen,
    /// Normale persoonsgegevens
    Normaal,
    /// Bijzondere persoonsgegevens (art. 9 AVG)
    Bijzonder,
    /// Strafrechtelijke gegevens (art. 10 AVG)
    Strafrechtelijk,
}

/// Woo openbaarheidsklasse
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum WooDisclosureClass {
    /// Volledig openbaar
    Openbaar,
    /// Gedeeltelijk openbaar (met gelakte passages)
    GedeeltelijkOpenbaar,
    /// Niet openbaar (met weigeringsgrond)
    NietOpenbaar,
    /// Nog niet beoordeeld
    NogNietBeoordeeld,
}

/// Woo metadata voor een document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WooMetadata {
    /// Is dit document Woo-relevant?
    pub is_relevant: bool,
    /// Openbaarheidsklasse
    pub disclosure_class: Option<WooDisclosureClass>,
    /// Datum van publicatie (indien openbaar)
    pub publication_date: Option<NaiveDate>,
    /// Wettelijke grondslag voor weigering (indien niet openbaar)
    pub refusal_grounds: Vec<WooRefusalGround>,
    /// Toelichting op besluit
    pub explanation: Option<String>,
}

impl Default for WooMetadata {
    fn default() -> Self {
        Self {
            is_relevant: false,
            disclosure_class: None,
            publication_date: None,
            refusal_grounds: Vec::new(),
            explanation: None,
        }
    }
}

/// Weigeringsgronden volgens Woo artikel 5.1 en 5.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum WooRefusalGround {
    /// Art 5.1.1.a: Eenheid van de Kroon
    EenheidKroon,
    /// Art 5.1.1.b: Veiligheid van de Staat
    VeiligheidStaat,
    /// Art 5.1.1.c: Vertrouwelijke bedrijfsgegevens
    Bedrijfsgegevens,
    /// Art 5.1.1.d: Bijzondere persoonsgegevens
    BijzonderePersoonsgegevens,
    /// Art 5.1.2.a: Internationale betrekkingen
    InternationaleBetrekkingen,
    /// Art 5.1.2.b: Economische belangen
    EconomischeBelangen,
    /// Art 5.1.2.c: Opsporing en vervolging
    OpsporingVervolging,
    /// Art 5.1.2.d: Inspectie, controle, toezicht
    InspectieToezicht,
    /// Art 5.1.2.e: Persoonlijke levenssfeer
    PersoonlijkeLevenssfeer,
    /// Art 5.1.2.f: Onevenredige benadeling
    OnevenredigeBenadeling,
    /// Art 5.1.2.g: Functioneren bestuursorgaan
    FunctionerenBestuursorgaan,
    /// Art 5.1.2.h: Concurrentiepositie
    Concurrentiepositie,
    /// Art 5.1.2.i: Misbruik van wettelijke bevoegdheden
    MisbruikBevoegdheden,
    /// Art 5.2: Persoonlijke beleidsopvattingen
    PersoonlijkeBeleidsopvattingen,
}

/// AVG metadata voor privacy compliance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AvgMetadata {
    /// Privacy niveau
    pub privacy_level: PrivacyLevel,
    /// Categorieën betrokkenen
    pub data_subject_categories: Vec<String>,
    /// Verwerkingsdoel
    pub processing_purpose: Option<String>,
    /// Rechtmatige grondslag
    pub legal_basis: Option<AvgLegalBasis>,
    /// Bewaartermijn in jaren
    pub retention_period_years: Option<i32>,
}

/// Rechtmatige grondslagen voor verwerking (art. 6 AVG)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum AvgLegalBasis {
    /// Toestemming van betrokkene
    Toestemming,
    /// Uitvoering overeenkomst
    Overeenkomst,
    /// Wettelijke verplichting
    WettelijkeVerplichting,
    /// Vitale belangen
    VitaleBelangen,
    /// Algemeen belang / openbaar gezag
    AlgemeenBelang,
    /// Gerechtvaardigd belang
    GerechtvaardigdBelang,
}

/// Archiefwet bewaartermijn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Bewaartermijn in jaren
    pub retention_years: i32,
    /// Archiefwaarde
    pub archival_value: ArchivalValue,
    /// Selectielijst referentie
    pub selection_list_ref: Option<String>,
    /// Datum vernietiging (indien tijdelijk te bewaren)
    pub destruction_date: Option<NaiveDate>,
    /// Datum overbrenging naar archief (indien permanent)
    pub transfer_date: Option<NaiveDate>,
}

/// Archiefwaarde volgens selectielijst
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum ArchivalValue {
    /// Permanent te bewaren (overbrengen naar Nationaal Archief)
    Permanent,
    /// Tijdelijk te bewaren (vernietigen na termijn)
    Tijdelijk,
}

/// Compliance status voor een informatieobject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub woo: WooComplianceStatus,
    pub avg: AvgComplianceStatus,
    pub archief: ArchiefComplianceStatus,
    pub overall_score: f32, // 0.0 - 1.0
    pub issues: Vec<ComplianceIssue>,
    pub assessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WooComplianceStatus {
    Compliant,
    PendingReview,
    ActionRequired,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvgComplianceStatus {
    Compliant,
    MissingLegalBasis,
    ExcessiveRetention,
    ActionRequired,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiefComplianceStatus {
    Compliant,
    MissingRetentionPeriod,
    OverdueDestruction,
    PendingTransfer,
    ActionRequired,
}

/// Een compliance probleem dat aandacht vereist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub description: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// AI-generated Woo relevance assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WooAssessment {
    /// Is dit document Woo-relevant?
    pub is_relevant: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Voorgestelde classificatie
    pub suggested_class: WooDisclosureClass,
    /// Redenering achter de beoordeling
    pub reasoning: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_default() {
        let class = Classification::default();
        assert_eq!(class, Classification::Intern);
    }

    #[test]
    fn test_woo_refusal_grounds() {
        let ground = WooRefusalGround::PersoonlijkeLevenssfeer;
        let json = serde_json::to_string(&ground).unwrap();
        assert_eq!(json, "\"persoonlijke_levenssfeer\"");
    }

    #[test]
    fn test_avg_metadata() {
        let mut avg = AvgMetadata::default();
        avg.privacy_level = PrivacyLevel::Bijzonder;
        avg.legal_basis = Some(AvgLegalBasis::WettelijkeVerplichting);
        avg.data_subject_categories.push("burgers".to_string());

        assert_eq!(avg.privacy_level, PrivacyLevel::Bijzonder);
    }
}
