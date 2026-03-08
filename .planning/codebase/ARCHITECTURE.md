# Architecture

**Analysis Date:** 2026-03-08

## Pattern Overview

**Overall:** Clean Architecture with Domain-Driven Design + Microservices

**Key Characteristics:**
- Hexagonal architecture with clear separation of concerns
- Rust workspace with loosely coupled crates
- Event-driven workflow engine for document processing
- Multi-agent AI pipeline for document creation
- Embedded DuckDB for analytical workloads
- WebAssembly frontend with reactive state

## Layers

**[API Layer - `crates/iou-api/src/]`:**
- Purpose: HTTP API server and route handling
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/`
- Contains: Axum routes, middleware, database access, workflow coordination
- Depends on: `iou-core` (domain models), `iou-ai` (AI services)
- Used by: HTTP clients, frontend

**[Domain Layer - `crates/iou-core/src/`]**
- Purpose: Core business logic and domain models
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-core/src/`
- Contains: Domain entities, value objects, workflows, document types
- Depends on: None (pure domain)
- Used by: API, AI services, storage

**[AI Layer - `crates/iou-ai/src/`]**
- Purpose: AI/ML services and multi-agent pipeline
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/`
- Contains: NER, GraphRAG, compliance assessment, document generation agents
- Depends on: `iou-core` (domain models)
- Used by: API layer for document creation

**[Storage Layer - `crates/iou-storage/src/`]**
- Purpose: External storage abstraction (S3/MinIO)
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-storage/src/`
- Contains: Storage client, metadata management
- Depends on: External SDKs
- Used by: API layer for document storage

**[Frontend Layer - `crates/iou-frontend/src/`]**
- Purpose: WebAssembly UI application
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/`
- Contains: Dioxus components, pages, API client, state management
- Depends on: Web APIs, HTTP client
- Used by: End users

**[Rules Layer - `crates/iou-regels/src/`]**
- Purpose: Business rules and domain-specific logic
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-regels/src/`
- Contains: Compliance rules, validation logic
- Depends on: `iou-core` (domain models)
- Used by: AI layer and API layer

## Data Flow

**[Request Processing Flow]:**

1. HTTP Request → API Router (`/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs`)
2. Authentication Middleware (Optional)
3. Route Handler → Domain Service
4. Domain Service → Database/External Services
5. Response Generation → HTTP Response

**[Document Creation Flow]:**

```
User Request → Research Agent (Query Knowledge Graph) →
Content Agent (Generate Document) →
Compliance Agent (Validate PII/Woo) →
Review Agent (Quality Check) →
Workflow Engine →
Storage
```

**[State Management]:**
- Global state via Dioxus signals (`AppState` in `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/state/mod.rs`)
- Domain entities managed as Rust structs
- Workflow state persisted in DuckDB

## Key Abstractions

**[InformationDomain]:**
- Purpose: Central organizing unit for information contexts
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-core/src/domain.rs`
- Pattern: Domain entity with subtypes (Case, Project, PolicyTopic)
- Four types: Zaak, Project, Beleid, Expertise

**[Multi-Agent Pipeline]:**
- Purpose: Coordinated document generation with maker-checker pattern
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/agents/pipeline.rs`
- Pattern: Sequential pipeline with retry and checkpointing
- Four agents: Research, Content, Compliance, Review

**[Knowledge Graph (GraphRAG)]:**
- Purpose: Automatic relation detection and semantic analysis
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-ai/src/graphrag.rs`
- Pattern: Graph-based knowledge representation
- Algorithms: Community detection, path finding, similarity scoring

**[Workflow Engine]:**
- Purpose: State management for document processing
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/workflows/mod.rs`
- Pattern: State machine with transitions
- Supports: Document approval, task assignment

## Entry Points

**[API Server Entry Point]:**
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-api/src/main.rs`
- Triggers: HTTP requests on port 8000
- Responsibilities: HTTP routing, middleware, database initialization

**[Frontend Application Entry Point]:**
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-frontend/src/main.rs`
- Triggers: Dioxus WASM launch
- Responsibilities: UI initialization, routing, state management

**[Domain Library Entry Point]:**
- Location: `/Users/marc/Projecten/iou-modern/crates/iou-core/src/lib.rs`
- Triggers: Import by other crates
- Responsibilities: Model exports and re-exports

## Error Handling

**Strategy:** Error hierarchy with typed errors
- API layer: Axum responses with HTTP status codes
- Domain layer: Domain-specific error types
- AI layer: Agent-specific errors with retry logic
- Storage layer: Provider-agnostic errors

**Patterns:**
- `anyhow` for quick error propagation
- `thiserror` for domain-specific errors
- Error context preservation through layers

## Cross-Cutting Concerns

**Logging:** Tracing with structured logs
- Subsystem-based loggers
- JSON output for production
- Console formatting for development

**Validation:** Multiple layers
- Domain validation in entity methods
- API validation via Axum extractors
- Input sanitization in middleware

**Authentication:** JWT-based with optional middleware
- Bearer token extraction
- Claims validation
- Context injection for routes

---

*Architecture analysis: 2026-03-08*