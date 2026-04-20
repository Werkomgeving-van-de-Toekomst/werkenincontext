-- Purpose Audit Log
-- Migration: 053_purpose_audit.sql

CREATE TABLE IF NOT EXISTS purpose_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    purpose_id VARCHAR(10) NOT NULL,
    user_id UUID,
    session_id UUID,
    organization_id UUID,
    request_id UUID,
    request_path TEXT,
    request_method VARCHAR(10),
    is_valid BOOLEAN NOT NULL,
    rejection_reason TEXT,
    data_categories JSONB,
    lawful_basis VARCHAR(50),
    validated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    response_time_ms INTEGER,
    user_agent TEXT,
    ip_address INET,

    CONSTRAINT fk_purpose_audit_purpose
        FOREIGN KEY (purpose_id) REFERENCES purposes(id) ON DELETE RESTRICT
);

CREATE INDEX idx_purpose_audit_purpose ON purpose_audit_log(purpose_id);
CREATE INDEX idx_purpose_audit_user ON purpose_audit_log(user_id);
CREATE INDEX idx_purpose_audit_validated_at ON purpose_audit_log(validated_at DESC);
CREATE INDEX idx_purpose_audit_is_valid ON purpose_audit_log(is_valid);

-- Audit log function
CREATE OR REPLACE FUNCTION log_purpose_validation(
    p_purpose_id VARCHAR(10),
    p_user_id UUID,
    p_session_id UUID,
    p_organization_id UUID,
    p_request_id UUID,
    p_request_path TEXT,
    p_request_method VARCHAR(10),
    p_is_valid BOOLEAN,
    p_rejection_reason TEXT DEFAULT NULL,
    p_data_categories JSONB DEFAULT NULL,
    p_lawful_basis VARCHAR(50) DEFAULT NULL,
    p_response_time_ms INTEGER DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL,
    p_ip_address INET DEFAULT NULL
) RETURNS UUID AS $$
DECLARE
    audit_id UUID;
BEGIN
    INSERT INTO purpose_audit_log (
        purpose_id, user_id, session_id, organization_id,
        request_id, request_path, request_method,
        is_valid, rejection_reason, data_categories,
        lawful_basis, response_time_ms, user_agent, ip_address
    ) VALUES (
        p_purpose_id, p_user_id, p_session_id, p_organization_id,
        p_request_id, p_request_path, p_request_method,
        p_is_valid, p_rejection_reason, p_data_categories,
        p_lawful_basis, p_response_time_ms, p_user_agent, p_ip_address
    ) RETURNING id INTO audit_id;

    RETURN audit_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Daily summary view
CREATE OR REPLACE VIEW v_purpose_audit_daily AS
SELECT
    date_trunc('day', validated_at) AS date,
    purpose_id,
    COUNT(*) AS total_validations,
    COUNT(*) FILTER (WHERE is_valid = true) AS successful,
    COUNT(*) FILTER (WHERE is_valid = false) AS failed,
    ROUND(AVG(response_time_ms)::numeric, 2) AS avg_response_time_ms
FROM purpose_audit_log
WHERE validated_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY date_trunc('day', validated_at), purpose_id
ORDER BY date DESC;
