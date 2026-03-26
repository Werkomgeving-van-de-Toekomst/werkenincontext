#!/bin/bash
# Seed script for document creation system
# Seeds templates and test data into DuckDB

set -e

DB_PATH="${DATABASE_PATH:-data/iou-modern.duckdb}"
API_URL="${API_URL:-http://localhost:8000/api}"

echo "🌱 Seeding IOU-Modern document creation system..."

# Create data directory
mkdir -p data

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    echo "📁 Database not found. Starting API server to create schema..."
    echo "Please run: cargo run --bin iou-api"
    echo "Then run this script again."
    exit 1
fi

# Seed templates using duckdb CLI if available
if command -v duckdb &> /dev/null; then
    echo "📝 Seeding templates via DuckDB..."

    duckdb "$DB_PATH" <<SQL
-- Insert WOO Besluit template
INSERT INTO templates (id, name, domain_id, document_type, content, required_variables, optional_sections, version, is_active)
VALUES (
    'tmpl_woo_besluit_001',
    'WOO Besluit',
    'woo_minfin',
    'woo_besluit',
    '# {{ document_type }}

**Referentie:** {{ reference_number }}
**Datum:** {{ date }}
**Gemeente:** {{ municipality }}

## 1. Aanvraag

Op {{ request_date }} heeft {{ requester }} een verzoek ingediend op grond van de Wet open overheid.

### 1.1 Onderwerp van het verzoek

{{ request_subject }}

{% if additional_details %}
### 1.2 Aanvullende details

{{ additional_details }}
{% endif %}

## 2. Beoordeling

### 2.1 Reikwijdte van het verzoek

Het verzoek betreft de volgende informatie:

{{ request_scope }}

### 2.2 Openbaarmaking

Na afweging van alle belangen wordt besloten tot:

{% if approval_granted %}
**Openbaarmaking** van de gevraagde informatie.
{% else %}
**Gedeeltelijke weigering** op grond van {{ refusal_ground }}.
{% endif %}

## 3. Besluit

Inhoudende:

{% if approval_granted %}
1. Het verzoek toe te kennen
2. De gevraagde informatie openbaar te maken
{% else %}
1. Het verzoek gedeeltelijk af te wijzen
2. De volgende informatie openbaar te maken: {{ disclosed_info }}
{% endif %}

---
*Dit besluit is automatisch gegenereerd door IOU-Modern*',
    ['reference_number', 'date', 'municipality', 'requester', 'request_subject', 'request_scope', 'approval_granted'],
    ['additional_details', 'refusal_ground', 'disclosed_info', 'author_name'],
    1,
    true
) ON CONFLICT (id) DO NOTHING;

-- Insert WOO Info template
INSERT INTO templates (id, name, domain_id, document_type, content, required_variables, optional_sections, version, is_active)
VALUES (
    'tmpl_woo_info_001',
    'WOO Informatie',
    'woo_minfin',
    'woo_info',
    '# Informatieoverzicht

**Document:** {{ title }}
**Referentie:** {{ reference }}
**Datum:** {{ date }}

## Beschrijving

{{ description }}

## Beschikbare Informatie

De volgende informatie is beschikbaar over dit onderwerp:

{{ information_summary }}

## Contact

Voor meer informatie kunt u contact opnemen met {{ contact_person }}.

---
*Dit document is automatisch gegenereerd door IOU-Modern*',
    ['title', 'reference', 'date', 'description'],
    ['information_summary', 'contact_person'],
    1,
    true
) ON CONFLICT (id) DO NOTHING;

-- Insert domain config
INSERT INTO domain_configs (domain_id, trust_level, required_approval_threshold, auto_approval_threshold)
VALUES ('woo_minfin', 'high', 0.85, 0.95)
ON CONFLICT (domain_id) DO NOTHING;

-- Insert test document
INSERT INTO documents (id, domain_id, document_type, state, current_version_key, compliance_score, confidence_score)
VALUES (
    '550e8400-e29b-41d4-a716-446655440000',
    'woo_minfin',
    'woo_besluit',
    'draft',
    'draft_v1',
    0.75,
    0.82
) ON CONFLICT (id) DO NOTHING;

-- Insert audit entries for test document
INSERT INTO audit_trail (id, document_id, agent_name, action, details, execution_time_ms)
VALUES
    ('550e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440000', 'pipeline', 'started', '{"stage": "pipeline"}', 0),
    ('550e8400-e29b-41d4-a716-446655440002', '550e8400-e29b-41d4-a716-446655440000', 'research', 'context_retrieved', '{"sources_found": 3}', 125),
    ('550e8400-e29b-41d4-a716-446655440003', '550e8400-e29b-41d4-a716-446655440000', 'content', 'generated', '{"word_count": 450}', 890)
ON CONFLICT (id) DO NOTHING;

SELECT 'Templates seeded successfully!' as status;
SELECT COUNT(*) as template_count FROM templates;
SELECT COUNT(*) as document_count FROM documents;
SELECT COUNT(*) as audit_count FROM audit_trail;
SQL

else
    echo "⚠️  DuckDB CLI not found. Please install duckdb or use the HTTP API to seed data."
    echo ""
    echo "Alternative: Use curl to seed templates via the API"
    echo "Make sure the API server is running first (cargo run --bin iou-api)"
    echo ""
fi

echo ""
echo "✅ Seeding complete!"
echo ""
echo "📊 Summary:"
echo "  - API URL: $API_URL"
echo "  - Database: $DB_PATH"
echo ""
echo "🧪 Test the API:"
echo "  curl $API_URL/templates"
echo "  curl $API_URL/documents/550e8400-e29b-41d4-a716-446655440000/status"
echo ""
