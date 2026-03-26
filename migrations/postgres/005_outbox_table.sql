-- Transactional Outbox Table for ETL
-- Section 05: Stabilization
--
-- This migration creates the change_outbox table used by the transactional
-- outbox pattern to ensure reliable data transfer from Supabase to DuckDB.

-- Create the change outbox table
CREATE TABLE IF NOT EXISTS change_outbox (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Aggregate identification
    -- aggregate_type: The type of entity that changed (e.g., 'information_domain', 'document')
    -- aggregate_id: The ID of the entity that changed
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,

    -- Event type (e.g., 'created', 'updated', 'deleted')
    event_type TEXT NOT NULL,

    -- Event payload (JSONB for flexibility)
    payload JSONB NOT NULL,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at TIMESTAMPTZ,

    -- Retry tracking
    retry_count INTEGER DEFAULT 0,
    last_error TEXT,

    -- Partitioning hint (for future partitioning if needed)
    -- Partition by created_at month for large-scale deployments
    CHECK (processed = FALSE OR processed_at IS NOT NULL)
);

-- Indexes for efficient outbox processing

-- Primary index for the outbox processor: unprocessed events ordered by creation time
-- This is the critical index for the ETL pipeline
CREATE INDEX IF NOT EXISTS idx_change_outbox_processing
ON change_outbox(processed, created_at ASC)
WHERE processed = FALSE;

-- Index for looking up events by aggregate (useful for debugging and replay)
CREATE INDEX IF NOT EXISTS idx_change_outbox_aggregate
ON change_outbox(aggregate_type, aggregate_id, created_at DESC);

-- Index for event type queries (useful for monitoring)
CREATE INDEX IF NOT EXISTS idx_change_outbox_event_type
ON change_outbox(event_type, created_at DESC);

-- Partial index for failed events (for retry processing)
CREATE INDEX IF NOT EXISTS idx_change_outbox_failed
ON change_outbox(created_at ASC, retry_count)
WHERE processed = FALSE AND retry_count > 0;

-- Function to insert outbox events atomically
CREATE OR REPLACE FUNCTION publish_outbox_event(
    p_aggregate_type TEXT,
    p_aggregate_id UUID,
    p_event_type TEXT,
    p_payload JSONB
) RETURNS UUID AS $$
DECLARE
    v_event_id UUID;
BEGIN
    INSERT INTO change_outbox (aggregate_type, aggregate_id, event_type, payload)
    VALUES (p_aggregate_type, p_aggregate_id, p_event_type, p_payload)
    RETURNING id INTO v_event_id;

    RETURN v_event_id;
END;
$$ LANGUAGE plpgsql VOLATILE SECURITY DEFINER;

-- Grant permissions
GRANT SELECT, INSERT ON change_outbox TO postgres;
GRANT UPDATE ON change_outbox TO postgres;
GRANT EXECUTE ON FUNCTION publish_outbox_event(TEXT, UUID, TEXT, JSONB) TO postgres;

-- Comments for documentation
COMMENT ON TABLE change_outbox IS 'Transactional outbox for reliable ETL from Supabase to DuckDB';
COMMENT ON COLUMN change_outbox.aggregate_type IS 'Type of the entity that changed (e.g., information_domain, document)';
COMMENT ON COLUMN change_outbox.aggregate_id IS 'ID of the entity that changed';
COMMENT ON COLUMN change_outbox.event_type IS 'Type of event (created, updated, deleted)';
COMMENT ON COLUMN change_outbox.payload IS 'Event payload as JSONB';
COMMENT ON COLUMN change_outbox.processed IS 'Whether the event has been processed by ETL';
COMMENT ON COLUMN change_outbox.retry_count IS 'Number of retry attempts for failed processing';
