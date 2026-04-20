// =============================================================================
// Context Quality - Quality scoring and validation
// =============================================================================

use crate::{QualityScore, Context};

/// Quality assessment for context data
pub trait QualityAssessment {
    /// Calculate overall quality score
    fn calculate_quality(&self) -> QualityScore;

    /// Check completeness (required fields present)
    fn check_completeness(&self) -> QualityScore;

    /// Check accuracy (data validation)
    fn check_accuracy(&self) -> QualityScore;

    /// Check consistency (cross-field validation)
    fn check_consistency(&self) -> QualityScore;

    /// Check timeliness (age of data)
    fn check_timeliness(&self) -> QualityScore;
}

/// Quality validator for context
pub struct ContextValidator;

impl ContextValidator {
    pub fn validate(context: &Context) -> Result<ValidationReport, Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check required fields
        if context.actor.actor_id.is_empty() {
            errors.push(ValidationError::MissingField("actor.actor_id"));
        }

        // Check if timestamp is reasonable (not year 0 or far in the future)
        let year_2000 = chrono::DateTime::from_timestamp(946684800, 0).unwrap();
        if context.temporal.aangemaakt_op < year_2000 {
            errors.push(ValidationError::MissingField("temporal.aangemaakt_op"));
        }

        // Check quality thresholds
        if context.quality.kwaliteit < 0.5 {
            warnings.push(ValidationError::LowQuality("overall", context.quality.kwaliteit));
        }

        if errors.is_empty() {
            Ok(ValidationReport {
                is_valid: true,
                warnings,
            })
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub warnings: Vec<ValidationError>,
}

#[derive(Debug)]
pub enum ValidationError {
    MissingField(&'static str),
    LowQuality(&'static str, f64),
    InconsistentValue(&'static str),
}
