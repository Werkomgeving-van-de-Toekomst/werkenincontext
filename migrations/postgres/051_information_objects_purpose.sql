-- Add Purpose to Information Objects
-- Migration: 051_information_objects_purpose.sql

ALTER TABLE information_objects
ADD COLUMN IF NOT EXISTS purpose_id VARCHAR(10) NOT NULL DEFAULT 'P001';

ALTER TABLE information_objects
ADD CONSTRAINT fk_information_object_purpose
    FOREIGN KEY (purpose_id) REFERENCES purposes(id) ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_information_objects_purpose ON information_objects(purpose_id);
CREATE INDEX IF NOT EXISTS idx_information_objects_domain_purpose ON information_objects(domain_id, purpose_id);

COMMENT ON COLUMN information_objects.purpose_id IS 'Purpose ID for AVG/GDPR Art. 5(1)(b) purpose binding';

CREATE OR REPLACE VIEW v_information_objects_with_purpose AS
SELECT
    io.id, io.domain_id, io.object_type, io.title,
    io.description, io.classification, io.privacy_level,
    io.purpose_id, p.name AS purpose_name, p.lawful_basis,
    io.created_at, io.updated_at
FROM information_objects io
JOIN purposes p ON io.purpose_id = p.id;
