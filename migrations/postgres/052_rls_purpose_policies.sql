-- Row-Level Security Policies for Purpose Binding
-- Migration: 052_rls_purpose_policies.sql

-- Session Functions for Purpose Context
CREATE OR REPLACE FUNCTION set_current_purpose(purpose_id VARCHAR(10))
RETURNS VOID AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM purposes
        WHERE id = purpose_id AND is_active = true
    ) THEN
        RAISE EXCEPTION 'Purpose % is not active or does not exist', purpose_id;
    END IF;
    PERFORM set_config('request.current.purpose', purpose_id, false);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION current_purpose()
RETURNS VARCHAR(10) AS $$
BEGIN
    RETURN NULLIF(current_setting('request.current.purpose', true), '');
END;
$$ LANGUAGE plpgsql STABLE;

CREATE OR REPLACE FUNCTION set_authorized_purposes(purpose_ids VARCHAR(10)[])
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('request.authorized.purposes', purpose_ids::text, false);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION authorized_purposes()
RETURNS VARCHAR(10)[] AS $$
BEGIN
    RETURN NULLIF(current_setting('request.authorized.purposes', true), '')::VARCHAR(10)[];
END;
$$ LANGUAGE plpgsql STABLE;

CREATE OR REPLACE FUNCTION can_use_purpose(target_purpose VARCHAR(10))
RETURNS BOOLEAN AS $$
DECLARE
    authorized VARCHAR(10)[];
BEGIN
    authorized := authorized_purposes();
    IF authorized IS NULL THEN
        RETURN false;
    END IF;
    RETURN target_purpose = ANY(authorized);
END;
$$ LANGUAGE plpgsql STABLE;

-- RLS Policies for Information Objects
ALTER TABLE information_objects ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS purpose_based_select ON information_objects;
DROP POLICY IF EXISTS purpose_based_insert ON information_objects;
DROP POLICY IF EXISTS purpose_based_update ON information_objects;
DROP POLICY IF EXISTS purpose_based_delete ON information_objects;

CREATE POLICY purpose_based_select ON information_objects
FOR SELECT
USING (
    current_purpose() IS NOT NULL
    AND can_use_purpose(current_purpose())
    AND purpose_id = current_purpose()
);

CREATE POLICY purpose_based_insert ON information_objects
FOR INSERT
WITH CHECK (
    current_purpose() IS NOT NULL
    AND can_use_purpose(current_purpose())
    AND purpose_id = current_purpose()
    AND EXISTS (
        SELECT 1 FROM purposes
        WHERE id = purpose_id AND is_active = true
    )
);

CREATE POLICY purpose_based_update ON information_objects
FOR UPDATE
USING (
    current_purpose() IS NOT NULL
    AND can_use_purpose(current_purpose())
    AND purpose_id = current_purpose()
)
WITH CHECK (
    current_purpose() IS NOT NULL
    AND can_use_purpose(current_purpose())
    AND purpose_id = current_purpose()
);

CREATE POLICY purpose_based_delete ON information_objects
FOR DELETE
USING (
    current_purpose() IS NOT NULL
    AND can_use_purpose(current_purpose())
    AND purpose_id = current_purpose()
);

ALTER TABLE information_objects FORCE ROW LEVEL SECURITY;
