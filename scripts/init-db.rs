use duckdb::Connection;

fn main() -> anyhow::Result<()> {
    let db_path = "data/iou-modern.duckdb";

    println!("🔧 Initializing database at: {}", db_path);

    let conn = Connection::open(db_path)?;

    // Run document creation tables migration
    let migration = include_str!("../migrations/002_document_creation_tables.sql");

    for statement in migration.split(';') {
        let stmt = statement.trim();
        if !stmt.is_empty() && !stmt.starts_with("--") {
            match conn.execute(stmt, []) {
                Ok(_) => println!("✓ Executed: {}", stmt.chars().take(50).collect::<String>()),
                Err(e) => {
                    let err_str = e.to_string();
                    if !err_str.contains("already exists") && !err_str.contains("Table") {
                        println!("✗ Error: {}", err_str);
                    } else {
                        println!("✓ Skipped (exists): {}", stmt.chars().take(50).collect::<String>());
                    }
                }
            }
        }
    }

    // Verify tables exist
    let tables = conn.prepare("SELECT table_name FROM information_schema.tables WHERE table_name IN ('documents', 'templates', 'audit_trail', 'domain_configs')")?;
    let mut tables = tables.query()?;

    println!("\n📊 Created tables:");
    while let Ok(row) = tables.next() {
        let name: String = row.get(0)?;
        println!("  - {}", name);
    }

    // Insert a test template
    println!("\n📝 Inserting test template...");
    let tmpl_insert = r#"
        INSERT INTO templates (id, name, domain_id, document_type, content, required_variables, optional_sections, version, is_active)
        VALUES (
            'tmpl_woo_besluit_001',
            'WOO Besluit',
            'woo_minfin',
            'woo_besluit',
            '# WOO Besluit

**Referentie:** {{ reference_number }}
**Datum:** {{ date }}

## Besluit

Na afweging wordt besloten tot openbaarmaking.

---
*Dit besluit is automatisch gegenereerd door IOU-Modern*',
            ['reference_number', 'date'],
            ['notes'],
            1,
            true
        )
    "#;

    match conn.execute(tmpl_insert, []) {
        Ok(_) => println!("✓ Test template inserted"),
        Err(e) => println!("✗ Template insert failed: {}", e),
    }

    // Insert domain config
    let config_insert = r#"
        INSERT INTO domain_configs (domain_id, trust_level, required_approval_threshold, auto_approval_threshold)
        VALUES ('woo_minfin', 'high', 0.85, 0.95)
    "#;

    match conn.execute(config_insert, []) {
        Ok(_) => println!("✓ Domain config inserted"),
        Err(e) => println!("✗ Config insert failed: {}", e),
    }

    println!("\n✅ Database initialization complete!");
    Ok(())
}
