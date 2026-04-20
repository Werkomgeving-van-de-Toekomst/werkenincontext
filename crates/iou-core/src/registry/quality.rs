//! Data Quality Metrics
//!
//! Tracks and scores data quality for entities in the registry.

use serde::{Deserialize, Serialize};

/// Overall quality score (0-1)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QualityScore {
    /// Completeness: how many required fields are filled
    completeness: f32,

    /// Accuracy: how accurate the data is (validated against rules)
    accuracy: f32,

    /// Consistency: how consistent data is across sources
    consistency: f32,

    /// Timeliness: how recent the data is
    timeliness: f32,
}

impl QualityScore {
    /// Create a new quality score
    pub fn new(
        completeness: f32,
        accuracy: f32,
        consistency: f32,
    ) -> Self {
        Self {
            completeness: completeness.clamp(0.0, 1.0),
            accuracy: accuracy.clamp(0.0, 1.0),
            consistency: consistency.clamp(0.0, 1.0),
            timeliness: 1.0, // Default to current
        }
    }

    /// Set timeliness score
    pub fn with_timeliness(mut self, timeliness: f32) -> Self {
        self.timeliness = timeliness.clamp(0.0, 1.0);
        self
    }

    /// Calculate overall score (weighted average)
    pub fn overall(&self) -> f32 {
        // Weights: completeness 30%, accuracy 30%, consistency 20%, timeliness 20%
        (self.completeness * 0.3
            + self.accuracy * 0.3
            + self.consistency * 0.2
            + self.timeliness * 0.2)
    }

    /// Get individual components
    pub fn completeness(&self) -> f32 {
        self.completeness
    }

    pub fn accuracy(&self) -> f32 {
        self.accuracy
    }

    pub fn consistency(&self) -> f32 {
        self.consistency
    }

    pub fn timeliness(&self) -> f32 {
        self.timeliness
    }

    /// Check if score is excellent (>= 0.9)
    pub fn is_excellent(&self) -> bool {
        self.overall() >= 0.9
    }

    /// Check if score is good (>= 0.75)
    pub fn is_good(&self) -> bool {
        self.overall() >= 0.75
    }

    /// Check if score is acceptable (>= 0.5)
    pub fn is_acceptable(&self) -> bool {
        self.overall() >= 0.5
    }
}

impl Default for QualityScore {
    fn default() -> Self {
        Self {
            completeness: 1.0,
            accuracy: 1.0,
            consistency: 1.0,
            timeliness: 1.0,
        }
    }
}

/// Quality metric for a specific aspect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityMetric {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: f32,

    /// Metric threshold
    pub threshold: QualityThreshold,

    /// When this metric was last calculated
    pub calculated_at: chrono::DateTime<chrono::Utc>,
}

impl DataQualityMetric {
    /// Create a new quality metric
    pub fn new(name: String, value: f32, threshold: QualityThreshold) -> Self {
        Self {
            name,
            value: value.clamp(0.0, 1.0),
            threshold,
            calculated_at: chrono::Utc::now(),
        }
    }

    /// Check if metric passes threshold
    pub fn passes(&self) -> bool {
        self.threshold.evaluate(self.value)
    }
}

/// Quality threshold definition
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum QualityThreshold {
    /// Value must be at least this
    Minimum { value: f32 },

    /// Value must be at most this
    Maximum { value: f32 },

    /// Value must be within range
    Range { min: f32, max: f32 },

    /// Value must exactly equal
    Exact { value: f32 },
}

impl QualityThreshold {
    /// Evaluate if a value passes this threshold
    pub fn evaluate(&self, value: f32) -> bool {
        match self {
            QualityThreshold::Minimum { value: min } => value >= *min,
            QualityThreshold::Maximum { value: max } => value <= *max,
            QualityThreshold::Range { min, max } => value >= *min && value <= *max,
            QualityThreshold::Exact { value } => (value - *value).abs() < 0.001,
        }
    }
}

/// Quality dimensions for scoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityDimension {
    Completeness,
    Accuracy,
    Consistency,
    Timeliness,
    Validity,
    Uniqueness,
}

/// Quality assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    /// Overall score
    pub score: QualityScore,

    /// Individual metrics
    pub metrics: Vec<DataQualityMetric>,

    /// Quality issues found
    pub issues: Vec<QualityIssue>,

    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Quality issue found during assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue description
    pub description: String,

    /// Affected field/attribute
    pub affected_field: Option<String>,

    /// Suggested fix
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_calculation() {
        let score = QualityScore::new(0.8, 0.9, 0.7);
        let overall = score.overall();

        // (0.8 * 0.3) + (0.9 * 0.3) + (0.7 * 0.2) + (1.0 * 0.2) = 0.24 + 0.27 + 0.14 + 0.2 = 0.85
        assert!((overall - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_quality_score_classification() {
        let excellent = QualityScore::new(0.95, 0.95, 0.95);
        assert!(excellent.is_excellent());

        let good = QualityScore::new(0.8, 0.8, 0.8);
        assert!(good.is_good());
        assert!(!good.is_excellent());

        let acceptable = QualityScore::new(0.6, 0.6, 0.6);
        assert!(acceptable.is_acceptable());
        assert!(!acceptable.is_good());

        let poor = QualityScore::new(0.3, 0.3, 0.3);
        assert!(!poor.is_acceptable());
    }

    #[test]
    fn test_quality_threshold_minimum() {
        let threshold = QualityThreshold::Minimum { value: 0.7 };

        assert!(threshold.evaluate(0.8));
        assert!(threshold.evaluate(0.7));
        assert!(!threshold.evaluate(0.6));
    }

    #[test]
    fn test_quality_threshold_maximum() {
        let threshold = QualityThreshold::Maximum { value: 0.3 };

        assert!(threshold.evaluate(0.2));
        assert!(threshold.evaluate(0.3));
        assert!(!threshold.evaluate(0.4));
    }

    #[test]
    fn test_quality_threshold_range() {
        let threshold = QualityThreshold::Range { min: 0.3, max: 0.7 };

        assert!(threshold.evaluate(0.5));
        assert!(threshold.evaluate(0.3));
        assert!(threshold.evaluate(0.7));
        assert!(!threshold.evaluate(0.2));
        assert!(!threshold.evaluate(0.8));
    }

    #[test]
    fn test_quality_metric_passes() {
        let threshold = QualityThreshold::Minimum { value: 0.7 };
        let metric = DataQualityMetric::new("test".to_string(), 0.8, threshold);

        assert!(metric.passes());
    }

    #[test]
    fn test_quality_metric_fails() {
        let threshold = QualityThreshold::Minimum { value: 0.7 };
        let metric = DataQualityMetric::new("test".to_string(), 0.5, threshold);

        assert!(!metric.passes());
    }

    #[test]
    fn test_quality_score_with_timeliness() {
        let score = QualityScore::new(1.0, 1.0, 1.0).with_timeliness(0.5);

        // (1.0 * 0.3) + (1.0 * 0.3) + (1.0 * 0.2) + (0.5 * 0.2) = 0.3 + 0.3 + 0.2 + 0.1 = 0.9
        assert!((score.overall() - 0.9).abs() < 0.01);
    }
}
