-- Migration 007: Data Subject Rights (AVG/GDPR Compliance)
--
-- This migration implements AVG (GDPR) Articles 15, 16, and 17:
-- - Article 15: Right of access (Subject Access Request)
-- - Article 16: Right to rectification
-- - Article 17: Right to erasure ("right to be forgotten")
--
-- These tables support legal compliance for Dutch government regulations.

-- ============================================
-- Subject Access Requests (AVG Article 15)
-- ============================================

CREATE TABLE IF NOT EXISTS subject_access_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requesting_user_id UUID NOT NULL,
    subject_user_id UUID NOT NULL,
    request_type VARCHAR NOT NULL DEFAULT 'full' CHECK (request_type IN ('full', 'partial', 'specific')),
    status VARCHAR NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending',
        'processing',
        'completed',
        'failed',
        'expired',
        'withdrawn'
    )),
    requested_fields TEXT[],
    response_data JSONB,
    response_format VARCHAR DEFAULT 'json' CHECK (response_format IN ('json', 'csv', 'pdf')),
    error_message TEXT,
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT (CURRENT_TIMESTAMP + INTERVAL '30 days'),
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Ensure users can only request their own data (unless admin)
    CONSTRAINT check_same_user CHECK (requesting_user_id = subject_user_id)
);

-- Indexes for SAR queries
CREATE INDEX IF NOT EXISTS idx_sar_requesting_user ON subject_access_requests(requesting_user_id);
CREATE INDEX IF NOT EXISTS idx_sar_status ON subject_access_requests(status);
CREATE INDEX IF NOT EXISTS idx_sar_expires_at ON subject_access_requests(expires_at);
CREATE INDEX IF NOT EXISTS idx_sar_created_at ON subject_access_requests(created_at DESC);

-- Comment for documentation
COMMENT ON TABLE subject_access_requests IS 'AVG Article 15: Subject Access Requests - users can request all their personal data';

-- ============================================
-- Data Rectification Requests (AVG Article 16)
-- ============================================

CREATE TABLE IF NOT EXISTS data_rectification_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requesting_user_id UUID NOT NULL,
    object_id UUID NOT NULL REFERENCES information_objects(id) ON DELETE CASCADE,
    field_name VARCHAR NOT NULL,
    old_value TEXT,
    new_value TEXT NOT NULL,
    justification TEXT,
    supporting_documents TEXT[],
    status VARCHAR NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending',
        'under_review',
        'approved',
        'rejected',
        'expired',
        'withdrawn'
    )),
    reviewed_by UUID,
    review_notes TEXT,
    reviewed_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT (CURRENT_TIMESTAMP + INTERVAL '30 days'),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for rectification queries
CREATE INDEX IF NOT EXISTS idx_rectification_requesting_user ON data_rectification_requests(requesting_user_id);
CREATE INDEX IF NOT EXISTS idx_rectification_object ON data_rectification_requests(object_id);
CREATE INDEX IF NOT EXISTS idx_rectification_status ON data_rectification_requests(status);
CREATE INDEX IF NOT EXISTS idx_rectification_reviewed_by ON data_rectification_requests(reviewed_by);

COMMENT ON TABLE data_rectification_requests IS 'AVG Article 16: Data Rectification Requests - users can correct inaccurate personal data';

-- ============================================
-- Data Erasure Requests (AVG Article 17)
-- ============================================

CREATE TABLE IF NOT EXISTS data_erasure_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requesting_user_id UUID NOT NULL,
    object_id UUID NOT NULL REFERENCES information_objects(id) ON DELETE CASCADE,
    erasure_type VARCHAR NOT NULL DEFAULT 'anonymization' CHECK (erasure_type IN (
        'anonymization',
        'deletion',
        'pseudonymization'
    )),
    legal_basis VARCHAR,
    retention_check BOOLEAN DEFAULT FALSE,
    retention_override_reason TEXT,
    status VARCHAR NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending',
        'legal_review',
        'compliance_required',
        'approved',
        'rejected',
        'expired',
        'withdrawn',
        'completed'
    )),
    reviewed_by UUID,
    review_notes TEXT,
    reviewed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT (CURRENT_TIMESTAMP + INTERVAL '30 days'),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for erasure queries
CREATE INDEX IF NOT EXISTS idx_erasure_requesting_user ON data_erasure_requests(requesting_user_id);
CREATE INDEX IF NOT EXISTS idx_erasure_object ON data_erasure_requests(object_id);
CREATE INDEX IF NOT EXISTS idx_erasure_status ON data_erasure_requests(status);
CREATE INDEX IF NOT EXISTS idx_erasure_reviewed_by ON data_erasure_requests(reviewed_by);
CREATE INDEX IF NOT EXISTS idx_erasure_legal_review ON data_erasure_requests(status, retention_check)
    WHERE status = 'legal_review';

COMMENT ON TABLE data_erasure_requests IS 'AVG Article 17: Data Erasure Requests - users can request deletion of their personal data';

-- ============================================
-- Audit Log for Data Subject Rights
-- ============================================

CREATE TABLE IF NOT EXISTS data_subject_rights_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_type VARCHAR NOT NULL CHECK (request_type IN ('sar', 'rectification', 'erasure')),
    request_id UUID NOT NULL,
    user_id UUID NOT NULL,
    action VARCHAR NOT NULL,
    actor_id UUID,
    actor_role VARCHAR,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for audit queries
CREATE INDEX IF NOT EXISTS idx_dsar_audit_request ON data_subject_rights_audit(request_id);
CREATE INDEX IF NOT EXISTS idx_dsar_audit_user ON data_subject_rights_audit(user_id);
CREATE INDEX IF NOT EXISTS idx_dsar_audit_type ON data_subject_rights_audit(request_type);
CREATE INDEX IF NOT EXISTS idx_dsar_audit_created_at ON data_subject_rights_audit(created_at DESC);

COMMENT ON TABLE data_subject_rights_audit IS 'Audit trail for all data subject rights requests';

-- ============================================
-- Row Level Security (RLS) Policies
-- ============================================

-- Enable RLS on all DSAR tables
ALTER TABLE subject_access_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE data_rectification_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE data_erasure_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE data_subject_rights_audit ENABLE ROW LEVEL SECURITY;

-- SAR policies: users can see their own requests
CREATE POLICY sar_select_own ON subject_access_requests
    FOR SELECT USING (requesting_user_id = current_setting('app.current_user_id')::UUID);

CREATE POLICY sar_insert_own ON subject_access_requests
    FOR INSERT WITH CHECK (requesting_user_id = current_setting('app.current_user_id')::UUID);

-- Rectification policies: users can see their own requests
CREATE POLICY rectification_select_own ON data_rectification_requests
    FOR SELECT USING (requesting_user_id = current_setting('app.current_user_id')::UUID);

CREATE POLICY rectification_insert_own ON data_rectification_requests
    FOR INSERT WITH CHECK (requesting_user_id = current_setting('app.current_user_id')::UUID);

-- Erasure policies: users can see their own requests
CREATE POLICY erasure_select_own ON data_erasure_requests
    FOR SELECT USING (requesting_user_id = current_setting('app.current_user_id')::UUID);

CREATE POLICY erasure_insert_own ON data_erasure_requests
    FOR INSERT WITH CHECK (requesting_user_id = current_setting('app.current_user_id')::UUID);

-- Audit policies: users can see their own audit entries
CREATE POLICY audit_select_own ON data_subject_rights_audit
    FOR SELECT USING (user_id = current_setting('app.current_user_id')::UUID);

-- Admin policies (for compliance officers)
CREATE POLICY sar_admin_all ON subject_access_requests
    FOR ALL USING (
        current_setting('app.user_roles', '') LIKE '%compliance_officer%' OR
        current_setting('app.user_roles', '') LIKE '%admin%'
    );

CREATE POLICY rectification_admin_all ON data_rectification_requests
    FOR ALL USING (
        current_setting('app.user_roles', '') LIKE '%compliance_officer%' OR
        current_setting('app.user_roles', '') LIKE '%admin%'
    );

CREATE POLICY erasure_admin_all ON data_erasure_requests
    FOR ALL USING (
        current_setting('app.user_roles', '') LIKE '%compliance_officer%' OR
        current_setting('app.user_roles', '') LIKE '%admin%'
    );

CREATE POLICY audit_admin_all ON data_subject_rights_audit
    FOR ALL USING (
        current_setting('app.user_roles', '') LIKE '%compliance_officer%' OR
        current_setting('app.user_roles', '') LIKE '%admin%'
    );

-- ============================================
-- Functions for automatic expiration handling
-- ============================================

-- Function to expire old pending requests
CREATE OR REPLACE FUNCTION expire_old_dsar_requests()
RETURNS void AS $$
BEGIN
    -- Expire old SAR requests
    UPDATE subject_access_requests
    SET status = 'expired', updated_at = CURRENT_TIMESTAMP
    WHERE status = 'pending' AND expires_at < CURRENT_TIMESTAMP;

    -- Expire old rectification requests
    UPDATE data_rectification_requests
    SET status = 'expired', updated_at = CURRENT_TIMESTAMP
    WHERE status = 'pending' AND expires_at < CURRENT_TIMESTAMP;

    -- Expire old erasure requests
    UPDATE data_erasure_requests
    SET status = 'expired', updated_at = CURRENT_TIMESTAMP
    WHERE status = 'pending' AND expires_at < CURRENT_TIMESTAMP;
END;
$$ LANGUAGE plpgsql;

-- Comment for documentation
COMMENT ON FUNCTION expire_old_dsar_requests IS 'Automatically expire pending DSAR requests past their expiration date';

-- ============================================
-- Trigger for updated_at
-- ============================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for each table
CREATE TRIGGER update_sar_updated_at
    BEFORE UPDATE ON subject_access_requests
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rectification_updated_at
    BEFORE UPDATE ON data_rectification_requests
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_erasure_updated_at
    BEFORE UPDATE ON data_erasure_requests
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- Views for common queries
-- ============================================

-- View for pending DSAR requests (for compliance officers)
CREATE OR REPLACE VIEW pending_dsar_requests AS
SELECT
    'sar' as request_type,
    sar.id,
    sar.requesting_user_id,
    sar.created_at,
    sar.expires_at
FROM subject_access_requests sar
WHERE sar.status = 'pending'

UNION ALL

SELECT
    'rectification' as request_type,
    drr.id,
    drr.requesting_user_id,
    drr.created_at,
    drr.expires_at
FROM data_rectification_requests drr
WHERE drr.status = 'pending'

UNION ALL

SELECT
    'erasure' as request_type,
    der.id,
    der.requesting_user_id,
    der.created_at,
    der.expires_at
FROM data_erasure_requests der
WHERE der.status = 'pending'

ORDER BY created_at ASC;

COMMENT ON VIEW pending_dsar_requests IS 'Unified view of all pending data subject rights requests';
