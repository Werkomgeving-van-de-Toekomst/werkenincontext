#!/usr/bin/env python3
import duckdb
import os

os.chdir('/Users/marc/Projecten/iou-modern')

# Create database connection
conn = duckdb.connect('data/iou-modern.duckdb')

# Create templates table
conn.execute("""
DROP TABLE IF EXISTS templates;
CREATE TABLE templates (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    domain_id VARCHAR NOT NULL,
    document_type VARCHAR NOT NULL,
    content TEXT NOT NULL,
    required_variables VARCHAR[] NOT NULL DEFAULT [],
    optional_sections VARCHAR[] NOT NULL DEFAULT [],
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_active BOOLEAN NOT NULL DEFAULT true
);
""")

# Insert test template
conn.execute("""
INSERT INTO templates VALUES (
    'tmpl_woo_besluit_001',
    'WOO Besluit',
    'woo_minfin',
    'woo_besluit',
    '# WOO Besluit\n\n**Referentie:** {{ reference_number }}\n\n## Besluit\n\nNa afweging wordt besloten tot openbaarmaking.',
    ['reference_number', 'date'],
    ['notes'],
    1,
    now(),
    now(),
    true
)
""")

# Insert WOO Info template
conn.execute("""
INSERT INTO templates VALUES (
    'tmpl_woo_info_001',
    'WOO Informatie',
    'woo_minfin',
    'woo_info',
    '# Informatie\n\n**Document:** {{ title }}\n\n{{ description }}',
    ['title', 'description'],
    ['contact_person'],
    1,
    now(),
    now(),
    true
)
""")

# Verify
result = conn.execute('SELECT id, name, domain_id, document_type FROM templates')
print("Templates created:")
for row in result.fetchall():
    print(f"  - {row[1]} ({row[3]})")

print("\n✅ Database initialized!")
