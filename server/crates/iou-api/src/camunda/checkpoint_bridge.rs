//! [`iou_ai::PipelineCheckpointStore`] backed by DuckDB (`pipeline_checkpoints`).

use async_trait::async_trait;
use iou_ai::{PipelineCheckpoint, PipelineCheckpointStore};
use std::sync::Arc;
use uuid::Uuid;

use crate::db::Database;

#[derive(Clone)]
pub struct DuckdbPipelineCheckpointStore(pub Arc<Database>);

#[async_trait]
impl PipelineCheckpointStore for DuckdbPipelineCheckpointStore {
    async fn save_checkpoint_db(&self, checkpoint: &PipelineCheckpoint) -> Result<(), String> {
        let db = self.0.clone();
        let cp = checkpoint.clone();
        tokio::task::spawn_blocking(move || db.save_pipeline_checkpoint_row(cp.document_id, &cp))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())
    }

    async fn load_latest_checkpoint(&self, document_id: Uuid) -> Result<Option<PipelineCheckpoint>, String> {
        let db = self.0.clone();
        tokio::task::spawn_blocking(move || db.load_latest_pipeline_checkpoint_row(document_id))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())
    }
}
