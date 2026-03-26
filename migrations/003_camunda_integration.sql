-- Camunda / Zeebe integration: pipeline input replay, idempotent jobs, checkpoints

CREATE TABLE IF NOT EXISTS document_pipeline_inputs (
    document_id UUID PRIMARY KEY,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    context_json JSON NOT NULL,
    requested_by VARCHAR NOT NULL,
    workflow_version VARCHAR NOT NULL DEFAULT '1.0.0',
    camunda_process_instance_key BIGINT,
    camunda_process_definition_key BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_document_pipeline_inputs_camunda
    ON document_pipeline_inputs(camunda_process_instance_key);

CREATE TABLE IF NOT EXISTS camunda_job_completions (
    zeebe_job_key BIGINT PRIMARY KEY,
    document_id UUID NOT NULL,
    job_type VARCHAR NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL,
    result_json JSON
);

CREATE INDEX IF NOT EXISTS idx_camunda_job_completions_document
    ON camunda_job_completions(document_id);

CREATE TABLE IF NOT EXISTS pipeline_checkpoints (
    document_id UUID NOT NULL,
    saved_at TIMESTAMPTZ NOT NULL,
    checkpoint_json JSON NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pipeline_checkpoints_document
    ON pipeline_checkpoints(document_id);
