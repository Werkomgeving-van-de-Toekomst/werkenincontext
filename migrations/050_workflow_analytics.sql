-- Workflow Analytics and AI Suggestions
-- Adds tracking for workflow performance metrics and AI-generated suggestions

-- Workflow performance analytics
CREATE TABLE IF NOT EXISTS workflow_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES approval_stages(id) ON DELETE CASCADE,
    domain_id VARCHAR(255) NOT NULL,
    document_type VARCHAR(255) NOT NULL,
    avg_completion_hours DECIMAL(10,2),
    sla_compliance_pct DECIMAL(5,2),
    bottleneck_stage_id UUID REFERENCES approval_stages(id),
    total_executions INT DEFAULT 0,
    last_analyzed_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for analytics lookups by workflow
CREATE INDEX idx_workflow_analytics_workflow_id ON workflow_analytics(workflow_id);

-- Index for analytics lookups by domain and document type
CREATE INDEX idx_workflow_analytics_domain_type ON workflow_analytics(domain_id, document_type);

-- AI suggestions log for workflow improvements and approvals
CREATE TABLE IF NOT EXISTS workflow_ai_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    stage_id UUID NOT NULL REFERENCES approval_stages(id) ON DELETE CASCADE,
    suggestion_type TEXT NOT NULL,
    suggestion JSONB NOT NULL,
    context JSONB,
    accepted BOOLEAN,
    feedback TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    responded_at TIMESTAMPTZ
);

-- Valid suggestion types
ALTER TABLE workflow_ai_suggestions ADD CONSTRAINT valid_suggestion_type
    CHECK (suggestion_type IN ('approval', 'optimization', 'config', 'bottleneck', 'escalation'));

-- Index for suggestions by document
CREATE INDEX idx_workflow_ai_suggestions_document_id ON workflow_ai_suggestions(document_id);

-- Index for suggestions by stage
CREATE INDEX idx_workflow_ai_suggestions_stage_id ON workflow_ai_suggestions(stage_id);

-- Index for suggestions by type and acceptance
CREATE INDEX idx_workflow_ai_suggestions_type_accepted ON workflow_ai_suggestions(suggestion_type, accepted);

-- Stage completion metrics for analytics
CREATE TABLE IF NOT EXISTS stage_completion_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stage_instance_id UUID NOT NULL REFERENCES document_approval_stages(id) ON DELETE CASCADE,
    stage_id TEXT NOT NULL,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_seconds INT,
    completed_within_sla BOOLEAN,
    approval_type TEXT,
    approver_count INT,
    approval_count INT,
    rejection_count INT,
    delegation_count INT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for metrics by stage definition
CREATE INDEX idx_stage_completion_metrics_stage_id ON stage_completion_metrics(stage_id);

-- Index for metrics by document
CREATE INDEX idx_stage_completion_metrics_document_id ON stage_completion_metrics(document_id);

-- Index for SLA compliance analysis
CREATE INDEX idx_stage_completion_metrics_sla ON stage_completion_metrics(completed_within_sla, duration_seconds);

-- Add updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_workflow_analytics_updated_at
    BEFORE UPDATE ON workflow_analytics
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comment tables for documentation
COMMENT ON TABLE workflow_analytics IS 'Tracks performance metrics for workflow definitions including completion times, SLA compliance, and bottleneck identification';

COMMENT ON TABLE workflow_ai_suggestions IS 'Logs AI-generated suggestions for workflow improvements, approval decisions, and configuration optimizations with user feedback tracking';

COMMENT ON TABLE stage_completion_metrics IS 'Detailed metrics for individual stage executions used for analytics and AI training';

COMMENT ON COLUMN workflow_analytics.avg_completion_hours IS 'Average time in hours for workflows to complete all stages';

COMMENT ON COLUMN workflow_analytics.sla_compliance_pct IS 'Percentage of workflow executions completed within SLA deadlines';

COMMENT ON COLUMN workflow_ai_suggestions.suggestion_type IS 'Type of suggestion: approval (decision help), optimization (workflow improvement), config (configuration), bottleneck (performance issue), escalation (deadline warning)';
