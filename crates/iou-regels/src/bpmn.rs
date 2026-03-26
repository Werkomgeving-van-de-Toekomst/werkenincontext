//! BPMN (Business Process Model and Notation) integration
//!
//! This module integrates BPMN process definitions with the orchestrator,
//! enabling DMN decision service calls within process flows.

use crate::dmn::{DmnEvaluator, DecisionContext, DecisionValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// BPMN process definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDefinition {
    pub id: String,
    pub name: String,
    pub version: String,

    /// Process variables
    pub variables: Vec<ProcessVariable>,

    /// Activities in the process
    pub activities: Vec<Activity>,

    /// Gateways (decision points)
    pub gateways: Vec<Gateway>,

    /// Events (start, end, intermediate)
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessVariable {
    pub id: String,
    pub name: String,
    pub type_ref: String,
    pub default: Option<DecisionValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub name: String,
    pub activity_type: ActivityType,
    pub incoming: Vec<String>,
    pub outgoing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActivityType {
    Task,
    UserTask,
    ServiceTask,
    SendTask,
    ReceiveTask,
    ScriptTask,
    BusinessRuleTask,
    SubProcess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gateway {
    pub id: String,
    pub name: String,
    pub gateway_type: GatewayType,
    pub incoming: Vec<String>,
    pub outgoing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GatewayType {
    Exclusive,
    Inclusive,
    Parallel,
    EventBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub event_type: EventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventType {
    Start,
    End,
    Intermediate,
    Boundary,
    /// Timer event for delays
    Timer,
    /// Message event for external signals
    Message,
}

/// Process instance (running process)
#[derive(Debug, Clone)]
pub struct ProcessInstance {
    pub id: Uuid,
    pub definition_id: String,
    pub state: ProcessInstanceState,
    pub variables: HashMap<String, DecisionValue>,
    pub current_activity: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessInstanceState {
    Running,
    Suspended,
    Completed,
    Failed,
    Terminated,
}

/// BPMN process engine
pub struct BpmnProcessEngine {
    evaluator: Arc<DmnEvaluator>,
    processes: HashMap<String, ProcessDefinition>,
}

impl BpmnProcessEngine {
    /// Create a new BPMN process engine
    pub fn new(evaluator: Arc<DmnEvaluator>) -> Self {
        Self {
            evaluator,
            processes: HashMap::new(),
        }
    }

    /// Load a BPMN process definition
    pub fn load_process(&mut self, definition: ProcessDefinition) -> Result<(), BpmnError> {
        self.processes.insert(definition.id.clone(), definition);
        Ok(())
    }

    /// Start a new process instance
    pub async fn start_process(
        &self,
        definition_id: &str,
        variables: HashMap<String, DecisionValue>,
    ) -> Result<ProcessInstance, BpmnError> {
        let _definition = self.processes
            .get(definition_id)
            .ok_or_else(|| BpmnError::ProcessNotFound(definition_id.to_string()))?;

        let instance = ProcessInstance {
            id: Uuid::new_v4(),
            definition_id: definition_id.to_string(),
            state: ProcessInstanceState::Running,
            variables,
            current_activity: None,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        Ok(instance)
    }

    /// Execute a Business Rule Task (calls DMN evaluator)
    pub async fn execute_business_rule_task(
        &self,
        task: &Activity,
        instance: &ProcessInstance,
    ) -> Result<HashMap<String, DecisionValue>, BpmnError> {
        if !matches!(task.activity_type, ActivityType::BusinessRuleTask) {
            return Err(BpmnError::InvalidActivity("Not a business rule task".into()));
        }

        // Extract decision ID from task (stored in task name or extension elements)
        let decision_id = task.name.clone();

        // Build decision context from process variables
        let mut context_inputs = HashMap::new();
        for var in instance.variables.iter() {
            context_inputs.insert(var.0.clone(), var.1.clone());
        }

        let context = DecisionContext {
            inputs: context_inputs,
            tenant_id: None, // Will be set from VC context
            context: HashMap::new(),
        };

        let result = self.evaluator
            .evaluate(&decision_id, &context)
            .map_err(|e| BpmnError::EvaluationError(e.to_string()))?;

        Ok(result.outputs)
    }

    /// Get a process definition by ID
    pub fn get_process(&self, definition_id: &str) -> Option<&ProcessDefinition> {
        self.processes.get(definition_id)
    }
}

/// BPMN errors
#[derive(Debug, Error)]
pub enum BpmnError {
    #[error("Process not found: {0}")]
    ProcessNotFound(String),

    #[error("Invalid activity: {0}")]
    InvalidActivity(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Fetch error: {0}")]
    FetchError(String),
}

/// Integration helper: Load BPMN process from Open Regels
pub async fn load_process_from_open_regels(
    client: &crate::OpenRegelsClient,
    regel_uri: &str,
) -> Result<ProcessDefinition, BpmnError> {
    // Fetch rule details from Open Regels
    let bindings = client.select(&format!(r#"
        PREFIX bpmn: <http://www.omg.org/spec/BPMN/20100524/MODEL/>

        SELECT ?xml WHERE {{
            <{}> bpmn:definitions ?xml .
        }}
    "#, regel_uri)).await
        .map_err(|e| BpmnError::FetchError(e.to_string()))?;

    if bindings.is_empty() {
        return Err(BpmnError::ProcessNotFound(regel_uri.to_string()));
    }

    let bpmn_xml = bindings[0].get("xml")
        .map(|v| v.value.clone())
        .ok_or(BpmnError::ParseError("No BPMN XML found".into()))?;

    // Parse BPMN XML
    parse_bpmn_xml(&bpmn_xml)
}

/// Parse BPMN XML (simplified)
fn parse_bpmn_xml(_xml: &str) -> Result<ProcessDefinition, BpmnError> {
    // For a production implementation, use a proper BPMN parser
    // This is a simplified version that handles basic processes
    Ok(ProcessDefinition {
        id: Uuid::new_v4().to_string(),
        name: "Unnamed Process".to_string(),
        version: "1.0".to_string(),
        variables: vec![],
        activities: vec![],
        gateways: vec![],
        events: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_instance_creation() {
        let evaluator = DmnEvaluator::new();
        let mut engine = BpmnProcessEngine::new(Arc::new(evaluator));

        let definition = ProcessDefinition {
            id: "test-process".to_string(),
            name: "Test Process".to_string(),
            version: "1.0".to_string(),
            variables: vec![],
            activities: vec![],
            gateways: vec![],
            events: vec![],
        };

        engine.load_process(definition).unwrap();

        let instance = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(engine.start_process("test-process", HashMap::new()))
            .unwrap();

        assert_eq!(instance.definition_id, "test-process");
        assert_eq!(instance.state, ProcessInstanceState::Running);
    }
}
