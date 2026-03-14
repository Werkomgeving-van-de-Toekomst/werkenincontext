-- Row-Level Security (RLS) Policies for IOU-Modern
-- Section 03: Authentication and Real-time Implementation
--
-- This migration creates RLS policies to enforce multi-tenant isolation
-- and classification-based access control at the database level.

-- ============================================================
-- Enable RLS on all tables
-- ============================================================

ALTER TABLE information_domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE information_objects ENABLE ROW LEVEL SECURITY;
ALTER TABLE documents ENABLE ROW LEVEL SECURITY;
ALTER TABLE templates ENABLE ROW LEVEL SECURITY;

-- ============================================================
-- Helper Functions for RLS
-- ============================================================

-- Function to extract organization_id from JWT custom claim
CREATE OR REPLACE FUNCTION auth.organization_id()
RETURNS UUID AS $$
  SELECT nullif(current_setting('request.jwt.claim.organization_id', true), '')::uuid
$$ LANGUAGE sql STABLE PARALLEL SAFE;

-- Function to extract user role from JWT
CREATE OR REPLACE FUNCTION auth.user_role()
RETURNS VARCHAR AS $$
  SELECT nullif(current_setting('request.jwt.claim.role', true), '')::varchar
$$ LANGUAGE sql STABLE PARALLEL SAFE;

-- Function to check if user has specific clearance level
CREATE OR REPLACE FUNCTION auth.has_clearance(required_level VARCHAR)
RETURNS BOOLEAN AS $$
  SELECT
    CASE required_level
      WHEN 'openbaar' THEN true -- Everyone has access to public
      WHEN 'intern' THEN
        -- User must have at least 'intern' clearance (intern, vertrouwelijk, or geheim)
        current_setting('request.jwt.claim.clearance', true) IN ('intern', 'vertrouwelijk', 'geheim')
      WHEN 'vertrouwelijk' THEN
        current_setting('request.jwt.claim.clearance', true) IN ('vertrouwelijk', 'geheim')
      WHEN 'geheim' THEN
        current_setting('request.jwt.claim.clearance', true) = 'geheim'
      ELSE false
    END
$$ LANGUAGE sql STABLE PARALLEL SAFE;

-- ============================================================
-- Organization Isolation Policies
-- ============================================================

-- Information Domains: Users can only read/write their own organization's domains
CREATE POLICY org_isolation_select ON information_domains
  FOR SELECT
  TO authenticated
  USING (organization_id = auth.organization_id());

CREATE POLICY org_isolation_insert ON information_domains
  FOR INSERT
  TO authenticated
  WITH CHECK (organization_id = auth.organization_id());

CREATE POLICY org_isolation_update ON information_domains
  FOR UPDATE
  TO authenticated
  USING (organization_id = auth.organization_id())
  WITH CHECK (organization_id = auth.organization_id());

CREATE POLICY org_isolation_delete ON information_domains
  FOR DELETE
  TO authenticated
  USING (organization_id = auth.organization_id());

-- Information Objects: Users can only read/write their own organization's objects
CREATE POLICY org_isolation_select ON information_objects
  FOR SELECT
  TO authenticated
  USING (
    -- Direct access: user's organization
    EXISTS (
      SELECT 1 FROM information_domains
      WHERE information_domains.id = information_objects.domain_id
      AND information_domains.organization_id = auth.organization_id()
    )
  );

CREATE POLICY org_isolation_insert ON information_objects
  FOR INSERT
  TO authenticated
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM information_domains
      WHERE information_domains.id = domain_id
      AND information_domains.organization_id = auth.organization_id()
    )
  );

CREATE POLICY org_isolation_update ON information_objects
  FOR UPDATE
  TO authenticated
  USING (
    EXISTS (
      SELECT 1 FROM information_domains
      WHERE information_domains.id = domain_id
      AND information_domains.organization_id = auth.organization_id()
    )
  )
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM information_domains
      WHERE information_domains.id = domain_id
      AND information_domains.organization_id = auth.organization_id()
    )
  );

CREATE POLICY org_isolation_delete ON information_objects
  FOR DELETE
  TO authenticated
  USING (
    EXISTS (
      SELECT 1 FROM information_domains
      WHERE information_domains.id = domain_id
      AND information_domains.organization_id = auth.organization_id()
    )
  );

-- Documents: Organization isolation
CREATE POLICY org_isolation_select ON documents
  FOR SELECT
  TO authenticated
  USING (organization_id = auth.organization_id());

CREATE POLICY org_isolation_insert ON documents
  FOR INSERT
  TO authenticated
  WITH CHECK (organization_id = auth.organization_id());

CREATE POLICY org_isolation_update ON documents
  FOR UPDATE
  TO authenticated
  USING (organization_id = auth.organization_id())
  WITH CHECK (organization_id = auth.organization_id());

CREATE POLICY org_isolation_delete ON documents
  FOR DELETE
  TO authenticated
  USING (organization_id = auth.organization_id());

-- Templates: Organization isolation
CREATE POLICY org_isolation_select ON templates
  FOR SELECT
  TO authenticated
  USING (organization_id = auth.organization_id());

CREATE POLICY org_isolation_insert ON templates
  FOR INSERT
  TO authenticated
  WITH CHECK (organization_id = auth.organization_id());

CREATE POLICY org_isolation_update ON templates
  FOR UPDATE
  TO authenticated
  USING (organization_id = auth.organization_id())
  WITH CHECK (organization_id = auth.organization_id());

CREATE POLICY org_isolation_delete ON templates
  FOR DELETE
  TO authenticated
  USING (organization_id = auth.organization_id());

-- ============================================================
-- Classification-Based Filtering
-- ============================================================

-- Information Objects: Classification-based access
CREATE POLICY classification_filter ON information_objects
  FOR SELECT
  TO authenticated
  USING (
    classification = 'openbaar'
    OR classification = 'intern'
    OR (classification = 'vertrouwelijk' AND auth.has_clearance('vertrouwelijk'))
    OR (classification = 'geheim' AND auth.has_clearance('geheim'))
    OR created_by = auth.uid() -- Users can always see their own objects
  );

-- Documents: Classification-based access
CREATE POLICY classification_filter ON documents
  FOR SELECT
  TO authenticated
  USING (
    classification IS NULL
    OR classification = 'openbaar'
    OR classification = 'intern'
    OR (classification = 'vertrouwelijk' AND auth.has_clearance('vertrouwelijk'))
    OR (classification = 'geheim' AND auth.has_clearance('geheim'))
  );

-- ============================================================
-- Woo Publication Filtering (Public Access)
-- ============================================================

-- Helper function to check if document should be publicly visible
CREATE OR REPLACE FUNCTION is_woo_public(doc_id VARCHAR)
RETURNS BOOLEAN AS $$
  SELECT EXISTS (
    SELECT 1 FROM information_objects
    WHERE id = doc_id::uuid
    AND is_woo_relevant = true
    AND woo_publication_date IS NOT NULL
    AND woo_publication_date <= CURRENT_TIMESTAMP
  );
$$ LANGUAGE sql STABLE;

-- Information Objects: Public Woo access
CREATE POLICY woo_public_read ON information_objects
  FOR SELECT
  TO public
  USING (
    is_woo_relevant = true
    AND woo_publication_date IS NOT NULL
    AND woo_publication_date <= CURRENT_TIMESTAMP
    AND classification = 'openbaar'
  );

-- Documents: Public Woo access
CREATE POLICY woo_public_read ON documents
  FOR SELECT
  TO public
  USING (
    woo_published = true
    AND (classification IS NULL OR classification = 'openbaar')
  );

-- Authenticated users can see their org's Woo documents
CREATE POLICY woo_authenticated_read ON documents
  FOR SELECT
  TO authenticated
  USING (
    woo_published = true
    OR organization_id = auth.organization_id()
  );

-- ============================================================
-- Owner-Based Policies
-- ============================================================

-- Users can update/delete their own information objects
CREATE POLICY owner_update ON information_objects
  FOR UPDATE
  TO authenticated
  USING (created_by = auth.uid())
  WITH CHECK (created_by = auth.uid());

CREATE POLICY owner_delete ON information_objects
  FOR DELETE
  TO authenticated
  USING (created_by = auth.uid());

-- ============================================================
-- Role-Based Policies
-- ============================================================

-- Domain Managers can manage domains
CREATE POLICY domain_manager_full ON information_domains
  FOR ALL
  TO authenticated
  USING (
    auth.user_role() = 'admin'
    OR auth.user_role() = 'domain_manager'
    OR organization_id = auth.organization_id()
  )
  WITH CHECK (
    auth.user_role() = 'admin'
    OR auth.user_role() = 'domain_manager'
    OR organization_id = auth.organization_id()
  );

-- ============================================================
-- Performance Optimization Indexes
-- ============================================================

-- Create indexes to support RLS policy lookups
CREATE INDEX IF NOT EXISTS idx_documents_org ON documents(organization_id);
CREATE INDEX IF NOT EXISTS idx_documents_woo ON documents(woo_published) WHERE woo_published = true;
CREATE INDEX IF NOT EXISTS idx_documents_classification ON documents(classification);
CREATE INDEX IF NOT EXISTS idx_documents_owner ON documents(created_by);

CREATE INDEX IF NOT EXISTS idx_objects_created_by ON information_objects(created_by);
CREATE INDEX IF NOT EXISTS idx_objects_classification ON information_objects(classification);
CREATE INDEX IF NOT EXISTS idx_objects_woo_relevant ON information_objects(is_woo_relevant) WHERE is_woo_relevant = true;
CREATE INDEX IF NOT EXISTS idx_objects_woo_published ON information_objects(woo_publication_date) WHERE woo_publication_date IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_domains_org ON information_domains(organization_id);

-- Partial indexes for common RLS patterns
CREATE INDEX IF NOT EXISTS idx_documents_active
  ON documents(organization_id)
  WHERE state NOT IN ('archived', 'rejected');

CREATE INDEX IF NOT EXISTS idx_objects_active
  ON information_objects(created_at DESC)
  WHERE classification != 'geheim';

-- ============================================================
-- RLS Policy Functions
-- ============================================================

-- Function to check if RLS is enabled on a table
CREATE OR REPLACE FUNCTION check_rls_enabled(table_name TEXT)
RETURNS BOOLEAN AS $$
  SELECT relrowsecurity FROM pg_class WHERE relname = table_name AND relnamespace = 'public'::regnamespace;
$$ LANGUAGE sql SECURITY DEFINER;

-- Function to list all RLS policies for a table
CREATE OR REPLACE FUNCTION list_rls_policies(table_name TEXT)
RETURNS TABLE(
  policy_name VARCHAR,
  command VARCHAR,
  roles VARCHAR[]
) AS $$
  SELECT
    p.polname::VARCHAR,
    p.polcmd::VARCHAR,
    ARRAY_AGG(r.rolname) AS roles
  FROM pg_policy p
  JOIN pg_class c ON c.oid = p.polrelid
  LEFT JOIN pg_authid r ON r.oid = ANY(p.polroles)
  WHERE c.relname = table_name
  GROUP BY p.polname, p.polcmd;
$$ LANGUAGE sql SECURITY DEFINER;

-- ============================================================
-- Audit Trail for RLS Violations (Optional)
-- ============================================================

-- Function to log RLS policy violations for security monitoring
CREATE OR REPLACE FUNCTION log_rls_violation()
RETURNS TRIGGER AS $$
BEGIN
  -- Log to a security audit table
  INSERT INTO audit_trail (document_id, agent_name, action, details)
  VALUES (
    COALESCE(NEW.id, OLD.id),
    current_user,
    'rls_violation_attempt',
    jsonb_build_object(
      'table', TG_TABLE_NAME,
      'user', auth.uid(),
      'organization', auth.organization_id(),
      'timestamp', now()
    )
  );
  RAISE EXCEPTION 'RLS policy violation detected on %', TG_TABLE_NAME;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================
-- Grant Permissions
-- ============================================================

-- Grant usage on auth functions to authenticated users
GRANT USAGE ON SCHEMA public TO postgres;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO postgres;
GRANT INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO postgres;

-- Grant usage on sequences
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO postgres;
