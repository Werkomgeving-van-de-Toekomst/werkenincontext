# Repository Setup

These repositories use `sqlx::query!` for compile-time verified SQL queries.

## Prerequisites

To compile with server feature, you need to either:

1. **Generate sqlx-data.json** (recommended for development):
   ```bash
   # Install sqlx-cli
   cargo install sqlx-cli --no-default-features --features rustls,postgres

   # Set DATABASE_URL
   export DATABASE_URL="postgresql://postgres:postgres@localhost/iou_modern"

   # Generate cache
   cargo sqlx prepare -- --lib

   # Or use offline mode
   export SQLX_OFFLINE=true
   ```

2. **Use offline mode** (for CI/production):
   ```bash
   export SQLX_OFFLINE=true
   cargo build --features server
   ```

## Running Tests

```bash
# Start test database
docker-compose up -d postgres

# Run tests
cargo test --package iou-core --features server
```

## Migrations

Run migrations before testing:
```bash
sqlx migrate run --source-dir migrations/postgres
```
