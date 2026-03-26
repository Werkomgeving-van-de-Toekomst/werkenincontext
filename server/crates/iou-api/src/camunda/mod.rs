//! Camunda 8 (Zeebe) integratie: proces starten, berichten, checkpoint-bridge.

mod checkpoint_bridge;
mod gateway;

pub use checkpoint_bridge::DuckdbPipelineCheckpointStore;
pub use gateway::CamundaGateway;
