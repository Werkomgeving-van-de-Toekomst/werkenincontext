#!/usr/bin/env python3
"""Run DuckDB migrations"""

import duckdb
import os
import sys

# Database path
db_path = os.path.join(os.path.dirname(__file__), "..", "data", "iou-modern.duckdb")

# Migration files in order
migrations = [
    "migrations/001_initial_schema.sql",
    "migrations/002_document_creation_tables.sql",
    "migrations/003_camunda_integration.sql",
    "migrations/004_graph_optimization.sql",
    "migrations/030_documents.sql",
    "migrations/031_templates.sql",
    "migrations/040_enhanced_workflow.sql",
]

def execute_sql_file(conn, filepath):
    """Execute all statements from a SQL file"""
    print(f"Running {filepath}...")

    with open(filepath, 'r') as f:
        sql = f.read()

    # Split on semicolons and execute each statement
    # Handle simple BEGIN/END blocks by checking for complete statements
    statements = []
    current = []
    in_create_function = False

    for line in sql.split('\n'):
        stripped = line.strip()
        if not stripped or stripped.startswith('--'):
            continue

        current.append(line)

        # Check for function/procedure creation
        if stripped.startswith('CREATE') and ('FUNCTION' in stripped or 'PROCEDURE' in stripped):
            in_create_function = True

        if stripped == ';' and not in_create_function:
            statements.append('\n'.join(current))
            current = []
        elif stripped == ';' and in_create_function:
            # Check if this is the end of function
            in_create_function = False
            statements.append('\n'.join(current))
            current = []
        elif stripped.startswith('END') and ';' in stripped:
            statements.append('\n'.join(current))
            current = []

    # Add any remaining statement
    if current:
        statements.append('\n'.join(current))

    for i, stmt in enumerate(statements):
        stmt = stmt.strip()
        if not stmt or stmt.startswith('--'):
            continue

        try:
            conn.execute(stmt)
        except Exception as e:
            error_str = str(e)
            if "already exists" not in error_str.lower() and "table" not in error_str.lower():
                print(f"  Warning: {error_str[:100]}")
            # Continue on error for idempotency

    print(f"  Completed {filepath}")

def main():
    print(f"Running migrations on {db_path}")
    print("-" * 50)

    conn = duckdb.connect(db_path)

    for migration in migrations:
        full_path = os.path.join(os.path.dirname(__file__), "..", migration)
        if not os.path.exists(full_path):
            print(f"WARNING: {full_path} not found, skipping")
            continue
        execute_sql_file(conn, full_path)

    # Verify some tables exist
    print("\n" + "-" * 50)
    print("Verifying tables...")
    tables = conn.execute("SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'").fetchall()
    print(f"Found {len(tables)} tables")
    for table in sorted([t[0] for t in tables if not t[0].startswith('i_')])[:20]:
        print(f"  - {table}")

    print("\nMigrations complete!")

if __name__ == "__main__":
    main()
