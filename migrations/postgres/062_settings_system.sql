-- ============================================================
-- Settings System Migration
-- ============================================================
-- Creates tables for hierarchical system configuration
-- Supports scoped settings with audit trail
--
-- Settings can be at different scopes with inheritance:
-- System -> Tenant -> Organization -> Domain -> User
-- ============================================================

-- Enable required extensions if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================
-- SETTINGS table
-- ============================================================
-- Core setting entity with hierarchical scope support
CREATE TABLE IF NOT EXISTS settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR NOT NULL,
    value JSONB NOT NULL,
    value_type VARCHAR NOT NULL CHECK (value_type IN (
        'string', 'integer', 'float', 'boolean', 'json', 'string_array'
    )),
    scope VARCHAR NOT NULL CHECK (scope IN (
        'system',         -- System-wide (all tenants)
        'tenant',         -- Tenant-specific (gemeente/provincie)
        'organization',   -- Organization-specific
        'domain',         -- Information domain-specific
        'user'            -- User-specific
    )),
    scope_id UUID,  -- ID of the scoped entity (organization_id, domain_id, user_id)
    description TEXT,
    default_value JSONB,
    validation_regex VARCHAR,
    is_encrypted BOOLEAN DEFAULT FALSE,
    is_public BOOLEAN DEFAULT FALSE,  -- Can be shown in UI without special perms
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(key, scope, scope_id)
);

-- Indexes for settings
CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key);
CREATE INDEX IF NOT EXISTS idx_settings_scope ON settings(scope);
CREATE INDEX IF NOT EXISTS idx_settings_scope_id ON settings(scope_id) WHERE scope_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_settings_unique ON settings(key, scope, scope_id);
CREATE INDEX IF NOT EXISTS idx_settings_public ON settings(is_public) WHERE is_public = TRUE;

-- GIN index for JSONB queries on value
CREATE INDEX IF NOT EXISTS idx_settings_value ON settings USING gin(value);

-- ============================================================
-- SETTINGS_HISTORY table
-- ============================================================
-- Audit trail for all setting changes
CREATE TABLE IF NOT EXISTS settings_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    setting_id UUID NOT NULL REFERENCES settings(id) ON DELETE CASCADE,
    key VARCHAR NOT NULL,
    old_value JSONB,
    new_value JSONB NOT NULL,
    changed_by UUID NOT NULL,
    changed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    change_reason TEXT
);

-- Indexes for settings_history
CREATE INDEX IF NOT EXISTS idx_settings_history_setting ON settings_history(setting_id);
CREATE INDEX IF NOT EXISTS idx_settings_history_key ON settings_history(key);
CREATE INDEX IF NOT EXISTS idx_settings_history_changed_by ON settings_history(changed_by);
CREATE INDEX IF NOT EXISTS idx_settings_history_timestamp ON settings_history(changed_at DESC);

-- ============================================================
-- SETTINGS_GROUPS table
-- ============================================================
-- Logical grouping of settings for UI organization
CREATE TABLE IF NOT EXISTS settings_groups (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    description TEXT,
    icon VARCHAR,
    order_key INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- SETTINGS_GROUP_ITEMS table
-- ============================================================
-- Links settings to groups for UI display
CREATE TABLE IF NOT EXISTS settings_group_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id VARCHAR NOT NULL REFERENCES settings_groups(id) ON DELETE CASCADE,
    setting_key VARCHAR NOT NULL,
    order_in_group INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(group_id, setting_key)
);

-- Indexes for settings_group_items
CREATE INDEX IF NOT EXISTS idx_settings_group_items_group ON settings_group_items(group_id);
CREATE INDEX IF NOT EXISTS idx_settings_group_items_key ON settings_group_items(setting_key);

-- ============================================================
-- FUNCTIONS
-- ============================================================

-- Function to get effective setting value (resolves hierarchy)
CREATE OR REPLACE FUNCTION get_setting_value(
    p_key VARCHAR,
    p_scope VARCHAR,
    p_scope_id UUID DEFAULT NULL
)
RETURNS TABLE (
    setting_id UUID,
    value JSONB,
    resolved_scope VARCHAR,
    resolved_scope_id UUID,
    is_default BOOLEAN
) AS $$
DECLARE
    rec RECORD;
BEGIN
    -- Try to find setting at exact scope first
    SELECT id, value, scope, scope_id INTO rec
    FROM settings
    WHERE key = p_key
      AND scope = p_scope
      AND (scope_id = p_scope_id OR scope_id IS NULL AND p_scope_id IS NULL)
    LIMIT 1;

    -- If not found, try parent scopes in hierarchy
    IF NOT FOUND THEN
        CASE p_scope
            WHEN 'user' THEN
                -- Try domain, then organization, then tenant, then system
                SELECT id, value, scope, scope_id INTO rec
                FROM settings s
                WHERE s.key = p_key
                  AND (
                      -- Domain scope
                      (s.scope = 'domain' AND s.scope_id IN (
                          SELECT domain_id FROM users WHERE id = p_scope_id
                          UNION
                          SELECT domain_id FROM information_domains WHERE owner_user_id = p_scope_id
                      ))
                      -- Organization scope
                      OR (s.scope = 'organization' AND s.scope_id IN (
                          SELECT organization_id FROM users WHERE id = p_scope_id
                      ))
                      -- Tenant scope (assuming tenant_id in some table)
                      OR (s.scope = 'tenant' AND s.scope_id IN (
                          SELECT organization_id FROM users WHERE id = p_scope_id
                      ))
                      -- System scope (no scope_id)
                      OR (s.scope = 'system' AND s.scope_id IS NULL)
                  )
                ORDER BY
                    CASE s.scope
                        WHEN 'domain' THEN 1
                        WHEN 'organization' THEN 2
                        WHEN 'tenant' THEN 3
                        WHEN 'system' THEN 4
                    END
                LIMIT 1;
            WHEN 'domain' THEN
                -- Try organization, then tenant, then system
                SELECT id, value, scope, scope_id INTO rec
                FROM settings s
                WHERE s.key = p_key
                  AND (
                      (s.scope = 'organization' AND s.scope_id IN (
                          SELECT organization_id FROM information_domains WHERE id = p_scope_id
                      ))
                      OR (s.scope = 'tenant' AND s.scope_id IN (
                          SELECT organization_id FROM information_domains WHERE id = p_scope_id
                      ))
                      OR (s.scope = 'system' AND s.scope_id IS NULL)
                  )
                ORDER BY
                    CASE s.scope
                        WHEN 'organization' THEN 1
                        WHEN 'tenant' THEN 2
                        WHEN 'system' THEN 3
                    END
                LIMIT 1;
            WHEN 'organization' THEN
                -- Try tenant, then system
                SELECT id, value, scope, scope_id INTO rec
                FROM settings s
                WHERE s.key = p_key
                  AND (
                      (s.scope = 'tenant' AND s.scope_id = p_scope_id)
                      OR (s.scope = 'system' AND s.scope_id IS NULL)
                  )
                ORDER BY
                    CASE s.scope
                        WHEN 'tenant' THEN 1
                        WHEN 'system' THEN 2
                    END
                LIMIT 1;
            WHEN 'tenant' THEN
                -- Try system
                SELECT id, value, scope, scope_id INTO rec
                FROM settings s
                WHERE s.key = p_key
                  AND s.scope = 'system'
                  AND s.scope_id IS NULL
                LIMIT 1;
        END CASE;
    END IF;

    -- If still not found, use default if available
    IF NOT FOUND THEN
        -- Return NULL with is_default flag
        RETURN QUERY SELECT NULL::UUID, NULL::JSONB, NULL::VARCHAR, NULL::UUID, TRUE::BOOLEAN;
    ELSE
        -- Return found setting
        RETURN QUERY SELECT rec.id, rec.value, rec.scope, rec.scope_id, FALSE::BOOLEAN;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to create history entry on setting update
CREATE OR REPLACE FUNCTION create_setting_history()
RETURNS TRIGGER AS $$
BEGIN
    -- Only create history if value actually changed
    IF OLD.value IS DISTINCT FROM NEW.value THEN
        INSERT INTO settings_history (
            setting_id,
            key,
            old_value,
            new_value,
            changed_by
        ) VALUES (
            NEW.id,
            NEW.key,
            OLD.value,
            NEW.value,
            -- Get current user from session (requires app to set)
            COALESCE(NULLIF(current_setting('application_user'), ''), '00000000-0000-0000-0000-000000000000')::UUID
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- TRIGGERS
-- ============================================================

-- Auto-update updated_at on settings
DROP TRIGGER IF EXISTS trigger_update_settings_updated_at ON settings;
CREATE TRIGGER trigger_update_settings_updated_at
    BEFORE UPDATE ON settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create history entry on setting update
DROP TRIGGER IF EXISTS trigger_create_setting_history ON settings;
CREATE TRIGGER trigger_create_setting_history
    AFTER UPDATE ON settings
    FOR EACH ROW
    WHEN (OLD.value IS DISTINCT FROM NEW.value)
    EXECUTE FUNCTION create_setting_history();

-- Auto-update updated_at on settings_groups
DROP TRIGGER IF EXISTS trigger_update_settings_groups_updated_at ON settings_groups;
CREATE TRIGGER trigger_update_settings_groups_updated_at
    BEFORE UPDATE ON settings_groups
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================
-- VIEWS
-- ============================================================

-- View: All settings with their resolved values for a scope
CREATE OR REPLACE VIEW v_settings_all AS
SELECT
    s.id,
    s.key,
    s.value,
    s.value_type,
    s.scope,
    s.scope_id,
    s.description,
    s.is_public,
    s.updated_at,
    COALESCE(sg.id, 'default') as group_id,
    COALESCE(sg.name, 'Overig') as group_name
FROM settings s
LEFT JOIN settings_group_items sgi ON s.key = sgi.setting_key
LEFT JOIN settings_groups sg ON sgi.group_id = sg.id AND sg.is_active = TRUE;

-- View: Settings history with change details
CREATE OR REPLACE VIEW v_settings_audit AS
SELECT
    sh.id,
    sh.setting_id,
    sh.key,
    sh.old_value,
    sh.new_value,
    sh.changed_by,
    u.email as changed_by_email,
    u.display_name as changed_by_name,
    sh.changed_at,
    sh.change_reason,
    jsonb_pretty(COALESCE(sh.old_value, '{}'::JSONB)) as old_value_formatted,
    jsonb_pretty(sh.new_value) as new_value_formatted
FROM settings_history sh
LEFT JOIN users u ON sh.changed_by = u.id
ORDER BY sh.changed_at DESC;

-- ============================================================
-- INITIAL DATA
-- ============================================================

-- Insert settings groups
INSERT INTO settings_groups (id, name, description, icon, order_key) VALUES
    ('general', 'Algemeen', 'Algemene organisatie instellingen', 'cog', 1),
    ('documents', 'Documenten', 'Instellingen voor documentbeheer', 'file', 2),
    ('woo', 'Woo', 'Woo publicatie instellingen', 'gavel', 3),
    ('avg', 'AVG', 'AVG compliantie instellingen', 'shield', 4),
    ('search', 'Zoeken', 'Zoek functionaliteit instellingen', 'search', 5),
    ('notifications', 'Notificaties', 'Email notificatie instellingen', 'bell', 6),
    ('security', 'Beveiliging', 'Security en authenticatie', 'lock', 7),
    ('ai', 'AI', 'AI agent instellingen', 'robot', 8)
ON CONFLICT (id) DO NOTHING;

-- Insert settings group items
INSERT INTO settings_group_items (group_id, setting_key, order_in_group) VALUES
    -- General
    ('general', 'organization_name', 1),
    ('general', 'organization_website', 2),
    ('general', 'organization_logo', 3),
    ('general', 'primary_color', 4),
    ('general', 'secondary_color', 5),
    -- Documents
    ('documents', 'default_retention_period', 1),
    ('documents', 'auto_versioning_enabled', 2),
    ('documents', 'max_file_size_mb', 3),
    ('documents', 'allowed_file_types', 4),
    -- Woo
    ('woo', 'woo_auto_publish_enabled', 1),
    ('woo', 'woo_default_refusal_reason', 2),
    ('woo', 'woo_publication_url_template', 3),
    -- AVG
    ('avg', 'pii_auto_detect_enabled', 1),
    ('avg', 'dpia_required_for_domains', 2),
    -- Search
    ('search', 'search_max_results', 1),
    ('search', 'search_timeout_seconds', 2),
    ('search', 'semantic_search_min_confidence', 3),
    -- Notifications
    ('notifications', 'email_notifications_enabled', 1),
    ('notifications', 'smtp_server', 2),
    ('notifications', 'smtp_port', 3),
    ('notifications', 'smtp_use_tls', 4),
    -- Security
    ('security', 'session_timeout_minutes', 1),
    ('security', 'mfa_required', 2),
    ('security', 'max_login_attempts', 3),
    -- AI
    ('ai', 'ai_trust_level', 1),
    ('ai', 'ai_auto_approval_threshold', 2),
    ('ai', 'ai_model_name', 3)
ON CONFLICT (group_id, setting_key) DO NOTHING;

-- Insert system defaults
INSERT INTO settings (key, value, value_type, scope, description, default_value) VALUES
    -- Document defaults
    ('default_retention_period', '10'::JSONB, 'integer', 'system',
     'Standaard bewaartermijn in jaren', '10'::JSONB),
    ('auto_versioning_enabled', 'true'::JSONB, 'boolean', 'system',
     'Automatische versiebeheer aanzetten', 'true'::JSONB),
    ('max_file_size_mb', '100'::JSONB, 'integer', 'system',
     'Maximum bestandsgrootte in MB', '100'::JSONB),
    ('allowed_file_types', '["pdf","docx","doc","odt","txt","jpg","png"]'::JSONB, 'string_array', 'system',
     'Toegestane bestandstypes', '["pdf","docx","doc","odt","txt","jpg","png"]'::JSONB),
    -- Woo defaults
    ('woo_auto_publish_enabled', 'false'::JSONB, 'boolean', 'system',
     'Woo publicatie automatisch aanzetten', 'false'::JSONB),
    -- AVG defaults
    ('pii_auto_detect_enabled', 'true'::JSONB, 'boolean', 'system',
     'PII automatische detectie aanzetten', 'true'::JSONB),
    ('dpia_required_for_domains', 'false'::JSONB, 'boolean', 'system',
     'DPIA vereist voor nieuwe domeinen', 'false'::JSONB),
    -- Search defaults
    ('search_max_results', '100'::JSONB, 'integer', 'system',
     'Maximum zoekresultaten', '100'::JSONB),
    ('search_timeout_seconds', '5'::JSONB, 'integer', 'system',
     'Zoek timeout in seconden', '5'::JSONB),
    ('semantic_search_min_confidence', '0.7'::JSONB, 'float', 'system',
     'Semantisch zoek minimum confidence', '0.7'::JSONB),
    -- Security defaults
    ('session_timeout_minutes', '60'::JSONB, 'integer', 'system',
     'Sessie timeout in minuten', '60'::JSONB),
    ('mfa_required', 'false'::JSONB, 'boolean', 'system',
     'MFA verplicht voor alle gebruikers', 'false'::JSONB),
    ('max_login_attempts', '5'::JSONB, 'integer', 'system',
     'Maximum login pogingen', '5'::JSONB),
    -- AI defaults
    ('ai_trust_level', '"medium"'::JSONB, 'string', 'system',
     'AI agent trust level', '"medium"'::JSONB),
    ('ai_auto_approval_threshold', '0.8'::JSONB, 'float', 'system',
     'Auto-approval threshold (0.0-1.0)', '0.8'::JSONB)
ON CONFLICT (key, scope, scope_id) DO NOTHING;
