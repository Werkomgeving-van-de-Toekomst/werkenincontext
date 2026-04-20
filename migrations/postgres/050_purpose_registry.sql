-- Purpose Registry Table
-- Migration: 050_purpose_registry.sql
-- Purpose: Implement IHH01 - Purpose binding for AVG/GDPR compliance

CREATE TABLE IF NOT EXISTS purposes (
    id VARCHAR(10) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    lawful_basis VARCHAR(50) NOT NULL,
    data_categories JSONB NOT NULL DEFAULT '[]',
    owner VARCHAR(255) NOT NULL,
    requires_approval BOOLEAN NOT NULL DEFAULT false,
    valid_from DATE,
    valid_until DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_lawful_basis CHECK (
        lawful_basis IN (
            'toestemming', 'overeenkomst', 'wettelijke_verplichting',
            'vitale_belangen', 'algemeen_belang', 'gerechtvaardigd_belang'
        )
    )
);

CREATE INDEX idx_purposes_active ON purposes(is_active) WHERE is_active = true;
CREATE INDEX idx_purposes_lawful_basis ON purposes(lawful_basis);

INSERT INTO purposes (id, name, description, lawful_basis, data_categories, owner) VALUES
    ('P001', 'ZAAK_AFHANDELING', 'Case processing tasks', 'wettelijke_verplichting', '["persoonsgegevens", "zaak_data"]', 'Domain Owner'),
    ('P002', 'WOO_PUBLICATIE', 'Woo publication process', 'wettelijke_verplichting', '["besluit", "document"]', 'WOO Officer'),
    ('P003', 'ANALYSE', 'Data analysis and reporting', 'algemeen_belang', '["geaggregeerde_data"]', 'Data Analyst'),
    ('P004', 'DIENSTVERLENING', 'Service delivery to citizens', 'overeenkomst', '["persoonsgegevens", "zaak_data"]', 'Service Manager'),
    ('P005', 'HANDHAVING', 'Enforcement and supervision', 'wettelijke_verplichting', '["persoonsgegevens", "bijzondere_gegevens"]', 'Supervision Officer'),
    ('P006', 'BELEIDSVORMING', 'Policy development', 'algemeen_belang', '["beleids_data", "geaggregeerde_data"]', 'Policy Director'),
    ('P007', 'ONDERZOEK', 'Statistical research', 'gerechtvaardigd_belang', '["geaggregeerde_data"]', 'Research Lead'),
    ('P008', 'ARCHIVERING', 'Archival record keeping', 'wettelijke_verplichting', '["document_data", "zaak_data"]', 'Archivist'),
    ('P009', 'CORRECTIE', 'Data correction requests', 'wettelijke_verplichting', '["persoonsgegevens"]', 'Data Steward'),
    ('P010', 'UITVOERING_BESLUIT', 'Decision implementation', 'wettelijke_verplichting', '["persoonsgegevens", "zaak_data"]', 'Case Manager'),
    ('P011', 'CONTACT_BURGER', 'Citizen communication', 'overeenkomst', '["communicatie_data"]', 'Contact Center'),
    ('P012', 'FINANCIEN', 'Financial administration', 'wettelijke_verplichting', '["financieel_data"]', 'Financial Controller'),
    ('P013', 'SAMENWERKING', 'Inter-agency collaboration', 'algemeen_belang', '["zaak_data", "beleids_data"]', 'Partnership Manager'),
    ('P014', 'KWIJTALING_VERJARING', 'Statute of limitations', 'wettelijke_verplichting', '["persoonsgegevens", "financieel_data"]', 'Legal Department'),
    ('P015', 'AUDIT', 'Internal audit and control', 'gerechtvaardigd_belang', '["financieel_data", "zaak_data"]', 'Internal Auditor')
ON CONFLICT (id) DO NOTHING;
