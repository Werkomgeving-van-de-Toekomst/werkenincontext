//! Persistent checkpoint storage for [`super::pipeline::AgentPipelineWithConfig`](super::pipeline::AgentPipelineWithConfig).

use super::pipeline::PipelineCheckpoint;
use async_trait::async_trait;
use uuid::Uuid;

/// Database or other backend for pipeline checkpoints (used by API layer with DuckDB).
#[async_trait]
pub trait PipelineCheckpointStore: Send + Sync {
    async fn save_checkpoint_db(&self, checkpoint: &PipelineCheckpoint) -> Result<(), String>;
    async fn load_latest_checkpoint(&self, document_id: Uuid) -> Result<Option<PipelineCheckpoint>, String>;
}
