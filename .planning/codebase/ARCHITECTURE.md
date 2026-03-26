# Architecture

**Analysis Date:** 2026-03-08

## Pattern Overview

**Overall:** Modular Monolith with Domain-Driven Design

**Key Characteristics:**
- Multi-crate workspace with clear domain boundaries
- REST API backend with Axum web framework
- WebAssembly frontend with Dioxus
- Embedded DuckDB for data storage
- Domain-driven design with rich domain models
- Event-driven workflows for document processing

## Layers

**Domain Layer:**
- Purpose: Core business logic and domain models
- Location: `crates/iou-core/src/`
- Contains: Domain entities, value objects, domain services
- Depends on: Standard Rust libraries
- Used by: API layer, AI layer, workflows

**API Layer:**
- Purpose: HTTP API endpoints and request handling
- Location: `crates/iou-api/src/`
- Contains: Route handlers, middleware, request/response DTOs
- Depends on: Domain layer, database layer
- Used by: Frontend, external clients

**Database Layer:**
- Purpose: Data persistence and querying
- Location: `crates/iou-api/src/db.rs`
- Contains: Database connection, schema management, query functions
- Depends on: Domain layer (for types)
- Used by: API layer, workflows

**Workflow Layer:**
- Purpose: Document workflow management
- Location: `crates/iou-api/src/workflows/`
- Contains: Workflow definitions, execution engine, state transitions
- Depends on: Domain layer, database layer
- Used by: API layer

**AI Layer:**
- Purpose: AI-powered document analysis and processing
- Location: `crates/iou-ai/src/`
- Contains: NER, compliance checking, document analysis
- Depends on: Domain layer
- Used by: API layer, workflows

**Frontend Layer:**
- Purpose: User interface
- Location: `crates/iou-frontend/src/`
- Contains: Pages, components, state management, API clients
- Depends on: API layer (via HTTP)
- Used by: End users

## Data Flow

**Document Creation Flow:**

1. User submits document via frontend (`DocumentCreator` component)
2. Frontend calls API endpoint (`/documents/create`)
3. API validates and creates document record in DuckDB
4. Workflow engine starts approval workflow
5. AI services analyze document for compliance
6. Document moves through approval states
7. Final document is stored and indexed

**State Management:**
- Workflow engine tracks document state
- Database stores current state and history
- Frontend subscribes to state changes via API

## Key Abstractions

**Domain Entities:**
- Purpose: Represent core business concepts
- Examples: `InformationDomain`, `InformationObject`, `WorkflowExecution`
- Pattern: Rich domain models with behavior

**Repositories:**
- Purpose: Data access patterns
- Examples: Database queries in `db.rs`
- Pattern: Query functions per entity type

**Services:**
- Purpose: Business logic coordination
- Examples: Workflow engine, AI analysis services
- Pattern: Stateless services with clear interfaces

**Value Objects:**
- Purpose: Immutable domain concepts
- Examples: `DocumentId`, `WorkflowStatus`, `TrustLevel`
- Pattern: Simple structs with validation

## Entry Points

**API Entry Point:**
- Location: `crates/iou-api/src/main.rs`
- Triggers: HTTP requests to various endpoints
- Responsibilities: Request routing, middleware application, service orchestration

**Frontend Entry Point:**
- Location: `crates/iou-frontend/src/main.rs`
- Triggers: Browser navigation
- Responsibilities: Route handling, state initialization, component rendering

**Workflow Entry Point:**
- Location: `crates/iou-api/src/workflows/mod.rs`
- Triggers: API calls, document events
- Responsibilities: Workflow execution, state transitions, notifications

## Error Handling

**Strategy:** Structured error handling with custom error types

**Patterns:**
- Custom `ApiError` type for API responses
- `anyhow::Result` for internal operations
- HTTP status codes mapped to error types
- Logging for error tracking

## Cross-Cutting Concerns

**Logging:** Structured logging with tracing
**Validation:** Input validation in API layer
**Authentication:** JWT-based authentication middleware

---

*Architecture analysis: 2026-03-08*