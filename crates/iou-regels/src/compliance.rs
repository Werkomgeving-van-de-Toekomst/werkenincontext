//! Koppeling van Open Regels regelspecificaties aan iou-core compliance types
//!
//! Vertaalt regelsets uit het register naar compliance-relevante
//! informatie die past in het bestaande iou-core compliance model.

use iou_core::compliance::{ComplianceIssue, IssueSeverity, RetentionPolicy, ArchivalValue};

// PROVISA conversion is always available (WASM-compatible)
use crate::provisa::{ProvisaBeoordeling, Archiefwaarde as Provisawaarde};

// Open Regels model integration is only available for native builds
#[cfg(not(target_arch = "wasm32"))]
use crate::model::{Regel, RegelType};

/// Converteer PROVISA beoordeling naar iou-core RetentionPolicy
impl From<ProvisaBeoordeling> for RetentionPolicy {
    fn from(beoordeling: ProvisaBeoordeling) -> Self {
        let jaren = beoordeling.bewaartermijn
            .as_ref()
            .and_then(|bt| bt.jaren)
            .unwrap_or(0) as i32; // 0 = permanent/ongedaan

        Self {
            retention_years: jaren,
            archival_value: match beoordeling.uiteindelijke_waarde {
                Provisawaarde::Permanent => ArchivalValue::Permanent,
                Provisawaarde::Tijdelijk => ArchivalValue::Tijdelijk,
            },
            selection_list_ref: beoordeling.bewaartermijn
                .as_ref()
                .map(|bt| bt.selectielijst_ref.clone()),
            destruction_date: beoordeling.vernietigingsdatum,
            transfer_date: beoordeling.overbrengingsdatum,
        }
    }
}

/// Genereer compliance issues uit een PROVISA beoordeling
pub fn provisa_compliance_issues(beoordeling: &ProvisaBeoordeling, _creatie_datum: chrono::NaiveDate) -> Vec<ComplianceIssue> {
    let mut issues = Vec::new();

    // Check of bewaartermijn ontbreekt
    if beoordeling.bewaartermijn.is_none() {
        issues.push(ComplianceIssue {
            severity: IssueSeverity::Medium,
            category: "PROVISA".to_string(),
            description: "Geen bewaartermijn gevonden in selectielijst".to_string(),
            recommended_action: "Handmatige beoordeling nodig voor bewaartermijn".to_string(),
        });
    }

    // Check of vernietiging nodig is
    if let Some(vernietiging) = beoordeling.vernietigingsdatum {
        let vandaag = chrono::Utc::now().date_naive();
        if vandaag > vernietiging {
            issues.push(ComplianceIssue {
                severity: IssueSeverity::High,
                category: "PROVISA - Vernietiging".to_string(),
                description: format!("Document had vernietigd moeten worden op {}", vernietiging),
                recommended_action: "Document vernietigen volgens Archiefwet".to_string(),
            });
        }
    }

    // Check of overbrenging nodig is
    if let Some(overbrenging) = beoordeling.overbrengingsdatum {
        let vandaag = chrono::Utc::now().date_naive();
        if vandaag > overbrenging {
            issues.push(ComplianceIssue {
                severity: IssueSeverity::High,
                category: "PROVISA - Archiefoverbrenging".to_string(),
                description: format!("Document had overgebracht moeten worden naar archief op {}", overbrenging),
                recommended_action: "Document overbrengen naar Nationaal Archief".to_string(),
            });
        }
    }

    issues
}

/// Beoordeelt of een regelspecificatie relevant is voor een bepaald
/// compliance domein op basis van de juridische bron en het regeltype
#[cfg(not(target_arch = "wasm32"))]
pub struct RegelComplianceMapper;

#[cfg(not(target_arch = "wasm32"))]
impl RegelComplianceMapper {
    pub fn new() -> Self {
        Self
    }

    /// Controleer of de regel een AVG-grondslag heeft
    ///
    /// Zoekt naar AVG-gerelateerde BWB-identifiers in de juridische bron
    pub fn is_avg_relevant(&self, regel: &Regel) -> bool {
        let Some(ref bron) = regel.juridische_bron else {
            return false;
        };
        // AVG is EU-verordening 2016/679; UAVG is BWBR0040940
        let avg_markers = ["2016/679", "bwbr0040940", "avg", "privacy", "persoonsgegevens"];
        let uri_lower = bron.uri.to_lowercase();
        avg_markers.iter().any(|m| uri_lower.contains(m))
    }

    /// Controleer of de regel een Woo-grondslag heeft
    pub fn is_woo_relevant(&self, regel: &Regel) -> bool {
        let Some(ref bron) = regel.juridische_bron else {
            return false;
        };
        // Woo is BWBR0045754
        let woo_markers = ["bwbr0045754", "open overheid", "woo", "openbaarheid"];
        let uri_lower = bron.uri.to_lowercase();
        woo_markers.iter().any(|m| uri_lower.contains(m))
    }

    /// Genereer een compliance issue als een relevante regel ontbreekt
    /// in de huidige configuratie
    pub fn ontbrekende_regel_als_issue(
        &self,
        regel: &Regel,
        context: &str,
    ) -> Option<ComplianceIssue> {
        // Alleen FLINT en DMN regels zijn machine-uitvoerbaar en dus kritisch
        if !matches!(regel.regel_type, RegelType::Flint | RegelType::Dmn) {
            return None;
        }

        let label = regel.label.as_deref().unwrap_or(&regel.uri);

        Some(ComplianceIssue {
            severity: IssueSeverity::Medium,
            category: "Open Regels".to_string(),
            description: format!(
                "Regelspecificatie '{label}' is van toepassing op {context} \
                 maar wordt niet actief bewaakt"
            ),
            recommended_action: format!(
                "Koppel regelspecificatie {} aan het betreffende proces",
                regel.uri
            ),
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for RegelComplianceMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{JuriconnectRef, RegelType};

    fn maak_testregel(bron_uri: &str, regel_type: RegelType) -> Regel {
        Regel {
            uri: "https://regels.overheid.nl/test".to_string(),
            label: Some("Testregel".to_string()),
            beschrijving: None,
            juridische_bron: Some(JuriconnectRef::new(bron_uri)),
            regel_type,
            eigenaar: None,
        }
    }

    #[test]
    fn test_avg_detectie() {
        let mapper = RegelComplianceMapper::new();
        let regel = maak_testregel(
            "https://wetten.overheid.nl/BWBR0040940",
            RegelType::Flint,
        );
        assert!(mapper.is_avg_relevant(&regel));
    }

    #[test]
    fn test_woo_detectie() {
        let mapper = RegelComplianceMapper::new();
        let regel = maak_testregel(
            "https://wetten.overheid.nl/BWBR0045754",
            RegelType::Dmn,
        );
        assert!(mapper.is_woo_relevant(&regel));
    }

    #[test]
    fn test_tekst_regel_geen_issue() {
        let mapper = RegelComplianceMapper::new();
        let regel = maak_testregel("https://wetten.overheid.nl/BWBR0015703", RegelType::Tekst);
        // Tekst-regels zijn niet machine-uitvoerbaar → geen issue
        assert!(mapper.ontbrekende_regel_als_issue(&regel, "bijstandsuitkering").is_none());
    }

    #[test]
    fn test_flint_regel_geeft_issue() {
        let mapper = RegelComplianceMapper::new();
        let regel = maak_testregel("https://wetten.overheid.nl/BWBR0015703", RegelType::Flint);
        let issue = mapper.ontbrekende_regel_als_issue(&regel, "bijstandsuitkering");
        assert!(issue.is_some());
        assert_eq!(issue.unwrap().category, "Open Regels");
    }
}
