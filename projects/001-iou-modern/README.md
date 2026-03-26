# IOU-Modern

> **Informatie Ondersteunde Werkomgeving** - Modern Rust/WebAssembly implementatie

## Project Overview

A context-driven information management platform for Dutch government organizations, built with Rust, WebAssembly, and DuckDB.

**Project ID**: 001
**Created**: 2026-03-20
**Status**: Active Development

## Technology Stack

| Component | Technology |
|-----------|-------------|
| Backend API | Axum |
| Database | DuckDB (embedded analytical) + PostgreSQL (transactional) |
| Frontend | Dioxus 0.7 (WebAssembly) |
| AI Agents | Rust multi-agent pipeline (Research, Content, Compliance, Review) |
| Maps | Leaflet.js via wasm-bindgen |
| NLP | Rust regex NER + petgraph |
| GraphRAG | petgraph |

## Domain

IOU-Modern serves Dutch government organizations (Rijk, Provincie, Gemeente, Waterschap, ZBO) with:

- **Information Domain Management**: Organizing information by Zaak, Project, Beleid, Expertise
- **Document Processing**: Multi-agent AI pipeline for document creation and compliance checking
- **Knowledge Graph**: GraphRAG for entity extraction and relationship discovery
- **Woo Compliance**: Wet open overheid (Government Information Act) compliance tracking
- **AVG Compliance**: Algemene verordening gegevensbescherming (GDPR) compliance

## Project Structure

```
iou-modern/
├── crates/
│   ├── iou-core/       # Shared domain models
│   ├── iou-api/        # REST API (Axum + DuckDB)
│   ├── iou-ai/         # AI agents (Research, Content, Compliance, Review)
│   ├── iou-storage/    # S3/MinIO storage client
│   └── iou-frontend/   # Dioxus WASM app
├── migrations/         # DuckDB & PostgreSQL schema
├── templates/          # Document templates (Markdown)
└── data/              # DuckDB database file
```

## Compliance

- **Woo (Wet open overheid)**: Public access to government information
- **AVG (GDPR)**: Data protection and privacy
- **Archiefwet**: Record retention and archival
- **Gemeentelijke Decentralisatie**: Support for decentralized government services
