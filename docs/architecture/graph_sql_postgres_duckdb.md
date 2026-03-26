# GraphRAG in SQL: PostgreSQL + DuckDB

Dit project gebruikt **geen native graph database**. De kennisgraaf is een **property graph in relationele tabellen**: knopen (`entities`) en gerichte randen (`entity_relationships`), aangevuld met communities en lidmaatschap. Dat sluit aan bij het idee “GraphRAG zonder graph DB” (SQL/ontology‑achtige lagen bovenop relationele data).

## Rolverdeling

| Laag | Database | Gebruik |
|------|----------|---------|
| **Writes, integriteit, multi-hop via SQL** | **PostgreSQL** (Supabase) | FK’s, consistente graph‑mutaties, API/transactionele pad |
| **Analytische graph‑stats, dashboards, batch‑RAG‑features** | **DuckDB** (embedded file) | Snelle aggregaties, scans, export; zelfde logische schema‑shape |
| **Runtime agent / petgraph** | **In-memory** (`iou_ai::graphrag::KnowledgeGraph`) | Algoritmes (components, paths) op geladen data — los van persistente stores |

## Migraties

- **PostgreSQL:** `migrations/postgres/006_graphrag_entities.sql` — tabellen, FK’s, B‑tree indexen op bron/doel/type/domein, views `v_entity_graph_degree` en `v_relationship_type_stats`.
- **DuckDB:** `migrations/001_initial_schema.sql` (basis tabellen) + `migrations/004_graph_optimization.sql` (indexen + dezelfde views voor OLAP).

Na wijzigingen aan DuckDB start de API `initialize_schema()` opnieuw; statements die “already exists” melden worden genegeerd (bestaand patroon in `db.rs`).

## Querypatronen (aanbevolen)

1. **Eén hop uitgaand:** `WHERE source_entity_id = $1` (index `idx_entity_relationships_source`).
2. **Eén hop inkomend:** `WHERE target_entity_id = $1` (index `idx_entity_relationships_target`).
3. **Filter op relatietype:** combineer met `(source_entity_id, relationship_type)` index.
4. **Multi-hop:** PostgreSQL **recursive CTE** (`WITH RECURSIVE`) over `entity_relationships`; houd `max depth` en `cycle` detection in de query of applicatie.
5. **Ranking / voorselectie:** gebruik `v_entity_graph_degree` om kandidaten te sorteren op degree voordat je duurdere LLM‑stappen doet.

## ETL / dual-write

Voor één **bron van waarheid** schrijf je graph‑mutaties primair naar **Postgres** en synchroniseer je periodiek naar DuckDB (zelfde kolommen). Het bestaande ETL/outbox‑patroon in `iou-api` kun je uitbreiden met `entities` / `entity_relationships` payloads. DuckDB blijft **read‑mostly** voor analytics.

## Verdere optimalisatie

- **`pg_trgm`** + GIN op `entities.name` als je fuzzy name‑match in SQL wilt.
- **Materialized views** in Postgres voor zware community‑ of 2‑hop statistieken (met `REFRESH` via job).
- **DuckDB** `EXPORT` naar Parquet voor offline graph‑analyse.
