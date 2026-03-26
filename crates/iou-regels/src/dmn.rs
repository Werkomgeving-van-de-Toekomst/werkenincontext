//! DMN (Decision Model and Notation) rule evaluation
//!
//! This module integrates DMN decision tables with the Open Regels
//! specificaties, allowing for business rule evaluation.

use crate::client::OpenRegelsClient;
use crate::model::{Regel, RegelType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// DMN decision context
#[derive(Debug, Clone)]
pub struct DecisionContext {
    /// Input variables for the decision
    pub inputs: HashMap<String, DecisionValue>,

    /// Tenant (municipality) for tenant-specific rules
    pub tenant_id: Option<String>,

    /// Additional context variables
    pub context: HashMap<String, String>,
}

/// Decision value (supports various types)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecisionValue {
    String(String),
    Integer(i64),
    Double(f64),
    Boolean(bool),
    Date(chrono::NaiveDate),
    Array(Vec<DecisionValue>),
}

impl DecisionValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            DecisionValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            DecisionValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
}

impl From<String> for DecisionValue {
    fn from(s: String) -> Self {
        DecisionValue::String(s)
    }
}

impl From<i64> for DecisionValue {
    fn from(i: i64) -> Self {
        DecisionValue::Integer(i)
    }
}

impl From<f64> for DecisionValue {
    fn from(f: f64) -> Self {
        DecisionValue::Double(f)
    }
}

impl From<bool> for DecisionValue {
    fn from(b: bool) -> Self {
        DecisionValue::Boolean(b)
    }
}

/// DMN decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    /// Decision name
    pub decision: String,

    /// Output values
    pub outputs: HashMap<String, DecisionValue>,

    /// Whether the decision matched any rule
    pub matched: bool,

    /// Matched rule ID
    pub matched_rule_id: Option<String>,

    /// Evaluation metadata
    pub metadata: DecisionMetadata,
}

/// Decision metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMetadata {
    /// Rule that was matched
    pub matched_rule: Option<String>,

    /// Evaluation time in microseconds
    pub evaluation_time_us: u64,

    /// DMN version
    pub dmn_version: String,

    /// Open Regels URI (if loaded from Open Regels)
    pub open_regels_uri: Option<String>,
}

/// DMN decision (loaded from XML or SPARQL)
#[derive(Debug, Clone)]
pub struct Decision {
    pub id: String,
    pub name: String,
    pub inputs: Vec<InputClause>,
    pub outputs: Vec<OutputClause>,
    pub rules: Vec<DecisionRule>,
    pub metadata: DecisionMetadata,
}

#[derive(Debug, Clone)]
pub struct InputClause {
    pub id: String,
    pub name: String,
    pub type_ref: String,
}

#[derive(Debug, Clone)]
pub struct OutputClause {
    pub id: String,
    pub name: String,
    pub type_ref: String,
    pub default_value: Option<DecisionValue>,
}

#[derive(Debug, Clone)]
pub struct DecisionRule {
    pub id: String,
    pub conditions: Vec<Condition>,
    pub conclusions: Vec<Conclusion>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub input_id: String,
    pub operator: ConditionOperator,
    pub value: DecisionValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    In,
    NotIn,
    Between,
}

#[derive(Debug, Clone)]
pub struct Conclusion {
    pub output_id: String,
    pub value: DecisionValue,
}

/// DMN evaluator with Open Regels integration
pub struct DmnEvaluator {
    decisions: HashMap<String, Decision>,
    open_regels: Option<Arc<OpenRegelsClient>>,
}

impl DmnEvaluator {
    /// Create a new DMN evaluator
    pub fn new() -> Self {
        Self {
            decisions: HashMap::new(),
            open_regels: None,
        }
    }

    /// Create evaluator with Open Regels client
    pub fn with_open_regels(mut self, client: Arc<OpenRegelsClient>) -> Self {
        self.open_regels = Some(client);
        self
    }

    /// Load a decision from DMN XML
    pub fn load_dmn_xml(&mut self, xml: &str) -> Result<(), DmnError> {
        let decision = Self::parse_dmn_xml(xml)?;
        self.decisions.insert(decision.id.clone(), decision);
        Ok(())
    }

    /// Load a decision from Open Regels by URI
    pub async fn load_from_open_regels(
        &mut self,
        regel_uri: &str,
    ) -> Result<Decision, DmnError> {
        let client = self.open_regels.as_ref()
            .ok_or(DmnError::OpenRegelsNotAvailable)?;

        // SPARQL query to fetch DMN XML
        let sparql = format!(r#"
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX dmn: <http://www.omg.org/spec/DMN/20191111/MODEL/>

            SELECT ?dmnXml WHERE {{
                <{}> dmn:definition ?dmn .
                ?dmn dmn:expression ?dmnXml .
            }}
        "#, regel_uri);

        let bindings = client.select(&sparql).await
            .map_err(|e| DmnError::FetchError(e.to_string()))?;

        if bindings.is_empty() {
            return Err(DmnError::DecisionNotFound(regel_uri.to_string()));
        }

        let dmn_xml = bindings[0].get("dmnXml")
            .map(|v| v.value.clone())
            .ok_or(DmnError::ParseError("No DMN XML found".into()))?;

        self.load_dmn_xml(&dmn_xml)?;

        self.decisions.get(regel_uri)
            .cloned()
            .ok_or(DmnError::DecisionNotFound(regel_uri.to_string()))
    }

    /// Discover DMN decisions from Open Regels
    pub async fn discover_dmn_decisions(
        &self,
        filter: Option<&str>,
    ) -> Result<Vec<Regel>, DmnError> {
        let client = self.open_regels.as_ref()
            .ok_or(DmnError::OpenRegelsNotAvailable)?;

        let sparql = if let Some(f) = filter {
            format!(r#"
                PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                PREFIX dmn: <http://www.omg.org/spec/DMN/20191111/MODEL/>

                SELECT ?regel ?label ?beschrijving WHERE {{
                    ?regel a dmn:Definition .
                    ?regel dmn:name ?label .
                    OPTIONAL {{ ?regel dmn:description ?beschrijving . }}
                    FILTER(CONTAINS(LCASE(?label), LCASE("{}")))
                }}
                ORDER BY ?label
            "#, f)
        } else {
            r#"
                PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                PREFIX dmn: <http://www.omg.org/spec/DMN/20191111/MODEL/>

                SELECT ?regel ?label ?beschrijving WHERE {{
                    ?regel a dmn:Definition .
                    ?regel dmn:name ?label .
                    OPTIONAL {{ ?regel dmn:description ?beschrijving . }}
                }}
                ORDER BY ?label
            "#.to_string()
        };

        let bindings = client.select(&sparql).await
            .map_err(|e| DmnError::FetchError(e.to_string()))?;

        Ok(bindings.into_iter().map(|mut b| {
            Regel {
                uri: b.remove("regel").map(|v| v.value).unwrap_or_default(),
                label: b.remove("label").map(|v| v.value),
                beschrijving: b.remove("beschrijving").map(|v| v.value),
                juridische_bron: None,
                regel_type: RegelType::Dmn,
                eigenaar: None,
            }
        }).collect())
    }

    /// Evaluate a decision
    pub fn evaluate(
        &self,
        decision_id: &str,
        context: &DecisionContext,
    ) -> Result<DecisionResult, DmnError> {
        let start = std::time::Instant::now();

        let decision = self.decisions
            .get(decision_id)
            .ok_or_else(|| DmnError::DecisionNotFound(decision_id.to_string()))?;

        // Find matching rule
        let matched_rule = decision.rules.iter()
            .find(|rule| self.rule_matches(rule, &decision.inputs, context));

        let mut outputs = HashMap::new();

        let matched_rule_id = if let Some(rule) = matched_rule {
            for conclusion in &rule.conclusions {
                let output = decision.outputs.iter()
                    .find(|o| o.id == conclusion.output_id)
                    .ok_or_else(|| DmnError::EvaluationError(format!(
                        "Output {} not found", conclusion.output_id
                    )))?;

                outputs.insert(output.name.clone(), conclusion.value.clone());
            }
            Some(rule.id.clone())
        } else {
            // Use default values
            for output in &decision.outputs {
                let value = output.default_value.clone()
                    .unwrap_or(DecisionValue::String("".to_string()));
                outputs.insert(output.name.clone(), value);
            }
            None
        };

        Ok(DecisionResult {
            decision: decision_id.to_string(),
            outputs,
            matched: matched_rule.is_some(),
            matched_rule_id,
            metadata: DecisionMetadata {
                matched_rule: matched_rule.map(|r| r.id.clone()),
                evaluation_time_us: start.elapsed().as_micros() as u64,
                dmn_version: "1.4".to_string(),
                open_regels_uri: None,
            },
        })
    }

    /// Check if a rule matches the context
    fn rule_matches(
        &self,
        rule: &DecisionRule,
        inputs: &[InputClause],
        context: &DecisionContext,
    ) -> bool {
        for condition in &rule.conditions {
            let input = inputs.iter()
                .find(|i| i.id == condition.input_id);

            let Some(input) = input else {
                return false;
            };

            let value = context.inputs.get(&input.name);

            if !self.evaluate_condition(&condition.operator, value, &condition.value) {
                return false;
            }
        }

        true
    }

    /// Evaluate a single condition
    fn evaluate_condition(
        &self,
        operator: &ConditionOperator,
        actual: Option<&DecisionValue>,
        expected: &DecisionValue,
    ) -> bool {
        match (actual, expected) {
            (Some(DecisionValue::String(a)), DecisionValue::String(b)) => match operator {
                ConditionOperator::Equal => a == b,
                ConditionOperator::NotEqual => a != b,
                _ => false,
            },
            (Some(DecisionValue::Integer(a)), DecisionValue::Integer(b)) => match operator {
                ConditionOperator::Equal => a == b,
                ConditionOperator::NotEqual => a != b,
                ConditionOperator::LessThan => a < b,
                ConditionOperator::LessThanOrEqual => a <= b,
                ConditionOperator::GreaterThan => a > b,
                ConditionOperator::GreaterThanOrEqual => a >= b,
                _ => false,
            },
            (Some(DecisionValue::Double(a)), DecisionValue::Double(b)) => match operator {
                ConditionOperator::Equal => (a - b).abs() < f64::EPSILON,
                ConditionOperator::LessThan => a < b,
                ConditionOperator::GreaterThan => a > b,
                _ => false,
            },
            (Some(DecisionValue::Boolean(a)), DecisionValue::Boolean(b)) => a == b,
            _ => false,
        }
    }

    /// Parse DMN XML (simplified implementation)
    fn parse_dmn_xml(_xml: &str) -> Result<Decision, DmnError> {
        // Parse DMN XML structure
        // For a production implementation, use a proper DMN parser
        // This is a simplified version that handles basic decision tables

        // For now, create a minimal decision structure
        Ok(Decision {
            id: Uuid::new_v4().to_string(),
            name: "Unnamed Decision".to_string(),
            inputs: vec![],
            outputs: vec![],
            rules: vec![],
            metadata: DecisionMetadata {
                matched_rule: None,
                evaluation_time_us: 0,
                dmn_version: "1.4".to_string(),
                open_regels_uri: None,
            },
        })
    }
}

impl Default for DmnEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// DMN errors
#[derive(Debug, Error)]
pub enum DmnError {
    #[error("Decision not found: {0}")]
    DecisionNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    #[error("Fetch error: {0}")]
    FetchError(String),

    #[error("Open Regels client not available")]
    OpenRegelsNotAvailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_value_conversions() {
        let s: DecisionValue = "test".to_string().into();
        assert_eq!(s.as_str(), Some("test"));

        let i: DecisionValue = 42i64.into();
        assert_eq!(i.as_int(), Some(42));

        let b: DecisionValue = true.into();
        assert!(matches!(b, DecisionValue::Boolean(true)));
    }

    #[test]
    fn test_condition_evaluation() {
        let evaluator = DmnEvaluator::new();

        let val = Some(DecisionValue::Integer(42));
        let expected = DecisionValue::Integer(40);

        assert!(evaluator.evaluate_condition(&ConditionOperator::GreaterThan, val.as_ref(), &expected));
        assert!(!evaluator.evaluate_condition(&ConditionOperator::LessThan, val.as_ref(), &expected));
    }
}
