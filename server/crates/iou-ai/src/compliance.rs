//! Compliance assessment voor Woo, AVG en Archiefwet
//!
//! Rule-based compliance checking voor Nederlandse overheidsinformatie.

use iou_core::compliance::{
    ArchivalValue, Classification, ComplianceIssue, ComplianceStatus, IssueSeverity,
    PrivacyLevel, RetentionPolicy, WooAssessment, WooDisclosureClass,
    ArchiefComplianceStatus, AvgComplianceStatus, WooComplianceStatus,
};
use iou_core::domain::DomainType;
use iou_core::objects::{InformationObject, ObjectType};

/// Compliance assessor for Dutch government documents
pub struct ComplianceAssessor;

impl ComplianceAssessor {
    pub fn new() -> Self {
        Self
    }

    /// Assess Woo (Wet open overheid) relevance
    pub fn assess_woo_relevance(
        &self,
        content: &str,
        object_type: ObjectType,
        classification: Classification,
    ) -> WooAssessment {
        let mut score: f32 = 0.0;
        let mut reasons = Vec::new();

        // Besluiten zijn altijd Woo-relevant
        if object_type == ObjectType::Besluit {
            score += 0.8;
            reasons.push("Besluiten zijn standaard Woo-relevant");
        }

        // Openbare classificatie = Woo-relevant
        if classification == Classification::Openbaar {
            score += 0.3;
            reasons.push("Document heeft openbare classificatie");
        }

        // Zoek naar Woo-relevante termen
        let woo_terms = [
            ("besluit", 0.15),
            ("vergunning", 0.12),
            ("beschikking", 0.12),
            ("bezwaar", 0.10),
            ("subsidie", 0.08),
            ("aanvraag", 0.05),
            ("beleid", 0.05),
            ("advies", 0.05),
        ];

        let content_lower = content.to_lowercase();
        for (term, weight) in woo_terms {
            if content_lower.contains(term) {
                score += weight;
                reasons.push(term);
            }
        }

        // Cap at 1.0
        score = score.min(1.0);

        let suggested_class = if score > 0.7 {
            WooDisclosureClass::Openbaar
        } else if score > 0.4 {
            WooDisclosureClass::GedeeltelijkOpenbaar
        } else {
            WooDisclosureClass::NogNietBeoordeeld
        };

        WooAssessment {
            is_relevant: score > 0.5,
            confidence: score,
            suggested_class,
            reasoning: reasons.join(", "),
        }
    }

    /// Calculate retention period based on domain and object type
    pub fn calculate_retention(
        &self,
        domain_type: DomainType,
        object_type: ObjectType,
    ) -> RetentionPolicy {
        let (years, value) = match (domain_type, object_type) {
            // Besluiten: permanent bewaren
            (_, ObjectType::Besluit) => (20, ArchivalValue::Permanent),

            // Zaken
            (DomainType::Zaak, ObjectType::Document) => (10, ArchivalValue::Tijdelijk),
            (DomainType::Zaak, ObjectType::Email) => (5, ArchivalValue::Tijdelijk),

            // Projecten
            (DomainType::Project, ObjectType::Document) => (10, ArchivalValue::Tijdelijk),
            (DomainType::Project, _) => (7, ArchivalValue::Tijdelijk),

            // Beleid: langer bewaren
            (DomainType::Beleid, ObjectType::Document) => (15, ArchivalValue::Permanent),
            (DomainType::Beleid, _) => (10, ArchivalValue::Tijdelijk),

            // Expertise: korter bewaren
            (DomainType::Expertise, _) => (5, ArchivalValue::Tijdelijk),

            // Fallback
            _ => (7, ArchivalValue::Tijdelijk),
        };

        RetentionPolicy {
            retention_years: years,
            archival_value: value,
            selection_list_ref: Some("Selectielijst provincies 2024".to_string()),
            destruction_date: None,
            transfer_date: None,
        }
    }

    /// Assess privacy level based on content
    pub fn assess_privacy_level(&self, content: &str) -> PrivacyLevel {
        let content_lower = content.to_lowercase();

        // Check for special category data (Art. 9 AVG)
        let special_terms = [
            "gezondheidsgegeven",
            "medisch",
            "religie",
            "geloof",
            "politieke",
            "vakbond",
            "seksueel",
            "biometrisch",
            "genetisch",
            "ras",
            "etnisch",
        ];

        for term in special_terms {
            if content_lower.contains(term) {
                return PrivacyLevel::Bijzonder;
            }
        }

        // Check for criminal data (Art. 10 AVG)
        let criminal_terms = [
            "strafblad",
            "veroordeling",
            "delict",
            "strafrechtelijk",
            "verdacht",
            "boete",
        ];

        for term in criminal_terms {
            if content_lower.contains(term) {
                return PrivacyLevel::Strafrechtelijk;
            }
        }

        // Check for normal personal data
        let personal_terms = [
            "bsn",
            "burgerservicenummer",
            "geboortedatum",
            "adres",
            "telefoonnummer",
            "e-mail",
            "persoonsgegevens",
        ];

        for term in personal_terms {
            if content_lower.contains(term) {
                return PrivacyLevel::Normaal;
            }
        }

        PrivacyLevel::Geen
    }

    /// Full compliance assessment
    pub fn assess_compliance(&self, object: &InformationObject, content: &str) -> ComplianceStatus {
        let mut issues = Vec::new();

        // Woo assessment
        let woo = self.assess_woo_relevance(content, object.object_type, object.classification);
        let woo_status = if woo.is_relevant && !object.is_woo_relevant {
            issues.push(ComplianceIssue {
                severity: IssueSeverity::Medium,
                category: "Woo".to_string(),
                description: "Document lijkt Woo-relevant maar is niet als zodanig gemarkeerd".to_string(),
                recommended_action: "Markeer document als Woo-relevant".to_string(),
            });
            WooComplianceStatus::ActionRequired
        } else if object.is_woo_relevant {
            WooComplianceStatus::PendingReview
        } else {
            WooComplianceStatus::NotApplicable
        };

        // AVG assessment
        let privacy = self.assess_privacy_level(content);
        let avg_status = if privacy != PrivacyLevel::Geen && object.privacy_level == PrivacyLevel::Geen {
            issues.push(ComplianceIssue {
                severity: if privacy == PrivacyLevel::Bijzonder {
                    IssueSeverity::High
                } else {
                    IssueSeverity::Medium
                },
                category: "AVG".to_string(),
                description: format!(
                    "Document bevat mogelijk {} persoonsgegevens",
                    match privacy {
                        PrivacyLevel::Bijzonder => "bijzondere",
                        PrivacyLevel::Strafrechtelijk => "strafrechtelijke",
                        _ => "normale",
                    }
                ),
                recommended_action: "Controleer en markeer privacy niveau".to_string(),
            });
            AvgComplianceStatus::ActionRequired
        } else {
            AvgComplianceStatus::Compliant
        };

        // Archief assessment
        let archief_status = if object.retention_period.is_none() {
            issues.push(ComplianceIssue {
                severity: IssueSeverity::Low,
                category: "Archiefwet".to_string(),
                description: "Geen bewaartermijn ingesteld".to_string(),
                recommended_action: "Stel bewaartermijn in volgens selectielijst".to_string(),
            });
            ArchiefComplianceStatus::MissingRetentionPeriod
        } else {
            ArchiefComplianceStatus::Compliant
        };

        // Calculate overall score
        let issue_weights: f32 = issues
            .iter()
            .map(|i| match i.severity {
                IssueSeverity::Critical => 0.4,
                IssueSeverity::High => 0.25,
                IssueSeverity::Medium => 0.15,
                IssueSeverity::Low => 0.05,
            })
            .sum();

        let overall_score = (1.0 - issue_weights).max(0.0);

        ComplianceStatus {
            woo: woo_status,
            avg: avg_status,
            archief: archief_status,
            overall_score,
            issues,
            assessed_at: chrono::Utc::now(),
        }
    }
}

impl Default for ComplianceAssessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_woo_assessment_besluit() {
        let assessor = ComplianceAssessor::new();
        let result = assessor.assess_woo_relevance(
            "Dit is een besluit over de vergunningsaanvraag.",
            ObjectType::Besluit,
            Classification::Intern,
        );

        assert!(result.is_relevant);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_retention_calculation() {
        let assessor = ComplianceAssessor::new();

        let besluit_retention =
            assessor.calculate_retention(DomainType::Zaak, ObjectType::Besluit);
        assert_eq!(besluit_retention.retention_years, 20);
        assert_eq!(besluit_retention.archival_value, ArchivalValue::Permanent);

        let email_retention = assessor.calculate_retention(DomainType::Zaak, ObjectType::Email);
        assert_eq!(email_retention.retention_years, 5);
    }

    #[test]
    fn test_privacy_assessment() {
        let assessor = ComplianceAssessor::new();

        assert_eq!(
            assessor.assess_privacy_level("Gewone tekst zonder gevoelige informatie"),
            PrivacyLevel::Geen
        );

        assert_eq!(
            assessor.assess_privacy_level("Document met BSN 123456789"),
            PrivacyLevel::Normaal
        );

        assert_eq!(
            assessor.assess_privacy_level("Medische gegevens van de patiënt"),
            PrivacyLevel::Bijzonder
        );

        assert_eq!(
            assessor.assess_privacy_level("Strafrechtelijke veroordeling van betrokkene"),
            PrivacyLevel::Strafrechtelijk
        );
    }
}
