-- Migration 008: Woo Publication Workflow (Wet open overheid Compliance)
--
-- This migration implements the Woo (Wet open overheid) publication workflow,
-- which replaced the Wob (Wet openbaarheid van bestuur) in 2022.
-- Woo requires government information to be proactively published.

-- ============================================
-- Woo Publication Requests
-- ============================================

CREATE TABLE IF NOT EXISTS woo_publication_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    object_id UUID NOT NULL REFERENCES information_objects(id) ON DELETE CASCADE,

    -- Publication platform details
    publication_platform VARCHAR NOT NULL DEFAULT 'rijksoverheid' CHECK (publication_platform IN (
        'rijksoverheid',
        'overheid.nl',
        'gemeente',
        'provincie',
        'waterschap',
        'custom'
    )),

    -- Publication status workflow
    publication_status VARCHAR NOT NULL DEFAULT 'pending' CHECK (publication_status IN (
        'pending',
        'assessment',
        'redaction_required',
        'approved',
        'rejected',
        'published',
        'withdrawn',
        'failed',
        'scheduled'
    )),

    -- Publication identifiers
    publicatie_nr VARCHAR UNIQUE,
    woo_reference VARCHAR,

    -- Rejection/withdrawal handling
    refusal_ground VARCHAR CHECK (refusal_ground IN (
        'privacy',
        'national_security',
        'commercial_confidence',
        'ongoing_investigation',
        'international_relations',
        'none'
    )),

    -- Redactions for sensitive information
    redactions JSONB DEFAULT '[]'::JSONB,

    -- Approval tracking
    approved_by UUID NOT NULL,
    approved_at TIMESTAMP WITH TIME ZONE,

    -- Publication tracking
    published_at TIMESTAMP WITH TIME ZONE,
    publication_url TEXT,
    doi VARCHAR,

    -- Legal compliance
    legal_basis VARCHAR,
    consultation_required BOOLEAN DEFAULT FALSE,
    consultation_completed_at TIMESTAMP WITH TIME ZONE,

    -- Imposition reference (if applicable)
    imposition_reference VARCHAR,

    -- Metadata
    publication_summary TEXT,
    access_grants JSONB DEFAULT '{}'::JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for Woo queries
CREATE INDEX IF NOT EXISTS idx_woo_pub_object ON woo_publication_requests(object_id);
CREATE INDEX IF NOT EXISTS idx_woo_pub_status ON woo_publication_requests(publication_status);
CREATE INDEX IF NOT EXISTS idx_woo_pub_published ON woo_publication_requests(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_woo_pub_approved_by ON woo_publication_requests(approved_by);
CREATE INDEX IF NOT EXISTS idx_woo_pub_platform ON woo_publication_requests(publication_platform);
CREATE INDEX IF NOT EXISTS idx_woo_pub_consultation ON woo_publication_requests(consultation_required, consultation_completed_at);

-- Unique index for active publications per object
CREATE UNIQUE INDEX IF NOT EXISTS idx_woo_pub_unique_active
    ON woo_publication_requests(object_id)
    WHERE publication_status IN ('approved', 'published', 'scheduled');

COMMENT ON TABLE woo_publication_requests IS 'Woo (Wet open overheid) publication workflow for government information transparency';

-- ============================================
-- Woo Publication Categories
-- ============================================

CREATE TABLE IF NOT EXISTS woo_publication_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL UNIQUE,
    slug VARCHAR NOT NULL UNIQUE,
    description TEXT,
    parent_id UUID REFERENCES woo_publication_categories(id) ON DELETE SET NULL,
    active BOOLEAN DEFAULT TRUE,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Insert default Woo categories
INSERT INTO woo_publication_categories (name, slug, description, sort_order) VALUES
    ('Besluiten', 'besluiten', 'Officiële besluiten van het bestuur', 1),
    ('Beleidsstukken', 'beleidsstukken', 'Documenten met betrekking tot het beleid', 2),
    ('Rapporten', 'rapporten', 'Onderzoeksrapporten en evaluaties', 3),
    ('Agenda''s en Notulen', 'agenda-notulen', 'Vergaderstukken van bestuursorganen', 4),
    ('Subsidies', 'subsidies', 'Informatie over subsidie-regelingen', 5),
    ('Bestuurlijke Informatie', 'bestuurlijke-informatie', 'Algemene bestuurlijke informatie', 6)
ON CONFLICT (slug) DO NOTHING;

-- ============================================
-- Woo Publication Document Mapping
-- ============================================

CREATE TABLE IF NOT EXISTS woo_document_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    publication_id UUID NOT NULL REFERENCES woo_publication_requests(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES woo_publication_categories(id) ON DELETE CASCADE,
    is_primary BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(publication_id, category_id)
);

CREATE INDEX IF NOT EXISTS idx_woo_doc_cat_publication ON woo_document_categories(publication_id);
CREATE INDEX IF NOT EXISTS idx_woo_doc_cat_category ON woo_document_categories(category_id);

-- ============================================
-- Woo Publication Access Logs
-- ============================================

CREATE TABLE IF NOT EXISTS woo_access_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    publication_id UUID NOT NULL REFERENCES woo_publication_requests(id) ON DELETE CASCADE,
    access_type VARCHAR NOT NULL CHECK (access_type IN ('view', 'download', 'request')),
    user_id UUID,  -- NULL for anonymous access
    session_id VARCHAR,
    ip_address INET,
    user_agent TEXT,
    referrer TEXT,
    accessed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_woo_access_pub ON woo_access_logs(publication_id);
CREATE INDEX IF NOT EXISTS idx_woo_access_date ON woo_access_logs(accessed_at DESC);
CREATE INDEX IF NOT EXISTS idx_woo_access_type ON woo_access_logs(access_type);

-- ============================================
-- Woo Active Publication Requests (Woo verzoeken)
-- ============================================

CREATE TABLE IF NOT EXISTS woo_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Requester information
    requester_name VARCHAR NOT NULL,
    requester_email VARCHAR NOT NULL,
    requester_address TEXT,
    requester_type VARCHAR DEFAULT 'citizen' CHECK (requester_type IN (
        'citizen',
        'organization',
        'journalist',
        'government',
        'other'
    )),

    -- Request details
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    requested_information TEXT[],
    reference_number VARCHAR UNIQUE,

    -- Request status
    request_status VARCHAR NOT NULL DEFAULT 'received' CHECK (request_status IN (
        'received',
        'acknowledged',
        'processing',
        'extension_requested',
        'information_provided',
        'partial_provision',
        'refused',
        'withdrawn',
        'referred',
        'appealed'
    )),

    -- Processing timeline
    received_date DATE NOT NULL DEFAULT CURRENT_DATE,
    acknowledgement_sent_date DATE,
    decision_due_date DATE,
    decision_date DATE,
    decision_extended_to DATE,

    -- Decision details
    decision VARCHAR CHECK (decision IN ('fully_granted', 'partial_grant', 'refused', 'referred')),
    refusal_grounds TEXT[],
    exemption_clauses TEXT[],

    -- Publication of Woo request (Woo obligations apply here too)
    publish_request BOOLEAN DEFAULT TRUE,
    published_request_id UUID REFERENCES woo_publication_requests(id),

    -- Appeal handling
    appeal_deadline DATE,
    appeal_received BOOLEAN DEFAULT FALSE,
    appeal_outcome VARCHAR,

    -- Internal tracking
    assigned_to UUID,
    priority VARCHAR DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high', 'urgent')),

    -- Communication
    communication_summary TEXT,
    documents_provided INT DEFAULT 0,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_woo_request_status ON woo_requests(request_status);
CREATE INDEX IF NOT EXISTS idx_woo_request_due_date ON woo_requests(decision_due_date);
CREATE INDEX IF NOT EXISTS idx_woo_request_assigned ON woo_requests(assigned_to);
CREATE INDEX IF NOT EXISTS idx_woo_request_reference ON woo_requests(reference_number);
CREATE INDEX IF NOT EXISTS idx_woo_request_received ON woo_requests(received_date DESC);

COMMENT ON TABLE woo_requests IS 'Active Woo requests from citizens and organizations (Woo verzoeken)';

-- ============================================
-- Row Level Security (RLS) Policies
-- ============================================

-- Enable RLS
ALTER TABLE woo_publication_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE woo_document_categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE woo_access_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE woo_requests ENABLE ROW LEVEL SECURITY;

-- Public read access for published items
CREATE POLICY woo_public_read ON woo_publication_requests
    FOR SELECT USING (publication_status = 'published');

-- Compliance officer full access
CREATE POLICY woo_compliance_full ON woo_publication_requests
    FOR ALL USING (
        current_setting('app.user_roles', '') LIKE '%woo_officer%' OR
        current_setting('app.user_roles', '') LIKE '%compliance_officer%' OR
        current_setting('app.user_roles', '') LIKE '%admin%'
    );

-- Read access for Woo requests (publicly viewable)
CREATE POLICY woo_requests_read ON woo_requests
    FOR SELECT USING (publish_request = TRUE);

-- Compliance officer full access to requests
CREATE POLICY woo_requests_full ON woo_requests
    FOR ALL USING (
        current_setting('app.user_roles', '') LIKE '%woo_officer%' OR
        current_setting('app.user_roles', '') LIKE '%compliance_officer%' OR
        current_setting('app.user_roles', '') LIKE '%admin%'
    );

-- ============================================
-- Functions and Triggers
-- ============================================

-- Trigger for updated_at on woo_publication_requests
CREATE TRIGGER update_woo_pub_updated_at
    BEFORE UPDATE ON woo_publication_requests
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Trigger for updated_at on woo_requests
CREATE TRIGGER update_woo_request_updated_at
    BEFORE UPDATE ON woo_requests
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to auto-generate publicatie_nr
CREATE OR REPLACE FUNCTION generate_publicatie_nr()
RETURNS TRIGGER AS $$
DECLARE
    year_part VARCHAR;
    seq_num INTEGER;
BEGIN
    IF NEW.publicatie_nr IS NULL AND NEW.publication_status = 'approved' THEN
        year_part := EXTRACT(year FROM CURRENT_DATE)::VARCHAR;
        SELECT COALESCE(MAX(CAST(SUBSTRING(publicatie_nr FROM '-\d+$') AS INTEGER)), 0) + 1
        INTO seq_num
        FROM woo_publication_requests
        WHERE publicatie_nr LIKE year_part || '-%';

        NEW.publicatie_nr := year_part || '-' || LPAD(seq_num::VARCHAR, 5, '0');
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER generate_woo_publicatie_nr
    BEFORE INSERT OR UPDATE ON woo_publication_requests
    FOR EACH ROW
    WHEN (NEW.publicatie_nr IS NULL AND NEW.publication_status = 'approved')
    EXECUTE FUNCTION generate_publicatie_nr();

-- Function to set decision_due_date (standard Woo response time is 8 weeks)
CREATE OR REPLACE FUNCTION set_woo_decision_due_date()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.decision_due_date IS NULL THEN
        -- Standard 8-week response period for Woo requests
        NEW.decision_due_date := NEW.received_date + INTERVAL '8 weeks';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_woo_due_date
    BEFORE INSERT ON woo_requests
    FOR EACH ROW
    WHEN (NEW.decision_due_date IS NULL)
    EXECUTE FUNCTION set_woo_decision_due_date();

-- ============================================
-- Views for common queries
-- ============================================

-- View for upcoming Woo request deadlines
CREATE OR REPLACE VIEW woo_upcoming_deadlines AS
SELECT
    id,
    reference_number,
    title,
    requester_name,
    decision_due_date,
    request_status,
    priority,
    DATE_PART('day', decision_due_date - CURRENT_DATE) as days_until_due
FROM woo_requests
WHERE request_status NOT IN ('information_provided', 'refused', 'withdrawn')
    AND decision_due_date > CURRENT_DATE
ORDER BY decision_due_date ASC;

COMMENT ON VIEW woo_upcoming_deadlines IS 'Woo requests with upcoming decision deadlines';

-- View for published Woo documents
CREATE OR REPLACE VIEW woo_published_documents AS
SELECT
    wpr.id,
    wpr.object_id,
    wpr.publicatie_nr,
    wpr.publication_url,
    wpr.published_at,
    io.title,
    io.description,
    id.name as domain_name,
    ARRAY_AGG(DISTINCT wc.name) as categories
FROM woo_publication_requests wpr
JOIN information_objects io ON wpr.object_id = io.id
JOIN information_domains id ON io.domain_id = id.id
LEFT JOIN woo_document_categories wdc ON wpr.id = wdc.publication_id
LEFT JOIN woo_publication_categories wc ON wdc.category_id = wc.id
WHERE wpr.publication_status = 'published'
GROUP BY wpr.id, wpr.object_id, io.title, io.description, id.name
ORDER BY wpr.published_at DESC;

COMMENT ON VIEW woo_published_documents IS 'All published Woo documents with metadata';

-- ============================================
-- Statistics function
-- ============================================

CREATE OR REPLACE FUNCTION woo_publication_stats()
RETURNS TABLE (
    total_requests BIGINT,
    pending_publication BIGINT,
    published_count BIGINT,
    avg_processing_days NUMERIC,
    overdue_requests BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        (SELECT COUNT(*)::BIGINT FROM woo_requests) as total_requests,
        (SELECT COUNT(*)::BIGINT FROM woo_publication_requests WHERE publication_status IN ('pending', 'assessment')) as pending_publication,
        (SELECT COUNT(*)::BIGINT FROM woo_publication_requests WHERE publication_status = 'published') as published_count,
        (SELECT AVG(DATE_PART('day', decision_date - received_date))::NUMERIC
         FROM woo_requests
         WHERE decision_date IS NOT NULL) as avg_processing_days,
        (SELECT COUNT(*)::BIGINT FROM woo_requests
         WHERE decision_due_date < CURRENT_DATE AND request_status NOT IN ('information_provided', 'refused', 'withdrawn')) as overdue_requests;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION woo_publication_stats IS 'Returns aggregate statistics for Woo publication workflow';
