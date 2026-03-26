# Backend Evaluation: DuckDB vs Convex vs Supabase

## Project Context

**IOU-Modern** is an information management platform for Dutch government organizations ("Informatie Ondersteunde Werkomgeving"). The platform helps government agencies manage information domains, create Woo-compliant documents, and maintain context-aware relationships between different information objects.

### Current Architecture

| Component | Technology |
|-----------|-------------|
| Backend API | Axum (Rust) |
| Database | DuckDB (embedded analytical) |
| Frontend | Dioxus 0.7 (WebAssembly) |
| AI Agents | Rust multi-agent pipeline |
| Storage | S3/MinIO |
| Graph | petgraph (GraphRAG) |

### Current Database Usage

DuckDB is currently used for:
- Storing information domains (Zaak, Project, Beleid, Expertise)
- Full-text search across documents
- GraphRAG relationship queries
- Document template storage
- Analytics on government data

## Evaluation Goals

**Primary Goal:** Comprehensive comparison of backend solutions to determine whether to migrate from the current DuckDB setup or continue with it.

**Decision Criteria:**
- Must support real-time collaboration features
- Must provide robust user authentication
- Must support Dutch government compliance (Woo, GDPR, Archiefwet)
- Must be self-hostable
- Should ideally maintain compatibility with the existing Rust stack

## Requirements

### Functional Requirements

#### FR1: Real-time Collaboration
- Multiple users must be able to collaborate on documents simultaneously
- Changes should propagate to all connected clients without manual refresh
- Conflict resolution for concurrent edits
- Presence indicators (show who is viewing/editing)

#### FR2: User Authentication & Authorization
- Secure user authentication (multiple providers preferred)
- Role-based access control (RBAC)
- Fine-grained permissions per information domain
- Session management
- Support for Dutch government identity systems (DigiD, eHerkenning) is a plus

#### FR3: Compliance Features
- GDPR compliance (right to be forgotten, data portability)
- Woo compliance (Wet open overheid - transparency)
- Archiefwet compliant (retention policies, automatic deletion)
- Audit logging for all data access
- PII detection and redaction capabilities

#### FR4: Data Model
- Support for complex relational data
- Full-text search capabilities
- JSON/document storage flexibility
- Graph relationship queries (for GraphRAG)
- Large dataset analytics (100K+ records)

### Non-Functional Requirements

#### NFR1: Self-Hosting
- Solution must be deployable on-premises or on own infrastructure
- No dependency on proprietary cloud services
- Data sovereignty (data stays within EU/Netherlands)

#### NFR2: Rust Stack Compatibility
- Can integrate with existing Rust/Axum backend
- Or can replace it without complete rewrite
- TypeScript/JavaScript SDKs for frontend integration

#### NFR3: Performance
- Sub-second query response times for typical operations
- Support for concurrent users (10+ simultaneous)
- Efficient for analytical workloads

#### NFR4: Maintainability
- Active development and community
- Clear documentation
- Migration path from existing DuckDB schema

### Technical Constraints

| Constraint | Priority |
|------------|----------|
| Self-hosting required | High |
| Maintain Rust stack where possible | High |
| EU data residency | High |
| Zero-downtime migration | Medium |
| Cost minimization | Medium |

## Candidates for Evaluation

### 1. DuckDB (Current Solution)
Embedded analytical database, already in use.

**Pros:**
- Already implemented
- Single-file deployment
- Excellent for analytics
- Full SQL support
- Works great with Rust

**Cons:**
- No built-in real-time features
- No built-in authentication
- Embedded (not designed for multi-user concurrent writes)
- Limited for transactional workloads

### 2. Convex
Serverless backend with reactive queries and automatic real-time updates.

**Pros:**
- Reactive queries (automatic real-time)
- Built-in auth integration (Auth0, Clerk)
- Serverless functions in TypeScript
- Zero-config scaling
- Great developer experience

**Cons:**
- Not self-hostable (proprietary cloud)
- Requires TypeScript backend logic
- Vendor lock-in
- US-based (may conflict with EU data residency)

### 3. Supabase
Open-source Firebase alternative built on PostgreSQL.

**Pros:**
- Self-hostable (Docker/Kubernetes)
- PostgreSQL foundation (proven, reliable)
- Built-in real-time subscriptions
- Built-in auth (multiple providers)
- Row-level security
- Edge functions
- Active open-source community

**Cons:**
- More infrastructure to manage
- Postgres not optimized for analytics (but can use extensions)
- May require migration from DuckDB schema

## Evaluation Approach

The evaluation should:

1. **Analyze current DuckDB usage** - Understand current schema, query patterns, and limitations
2. **Research each candidate** - Deep dive into features, limitations, and implementation details
3. **Build comparison matrix** - Score each solution against requirements
4. **Prototype if needed** - Test key features (real-time, auth) with candidates
5. **Recommendation** - Clear recommendation with justification and migration path (if applicable)

## Open Questions

- Is real-time collaboration a must-have or nice-to-have for current users?
- How important is the analytical performance of DuckDB for the application?
- Are there budget constraints for hosted solutions?
- What is the timeline for making this decision?
- Would a hybrid approach (e.g., DuckDB for analytics + Supabase for auth/real-time) make sense?

## Deliverables

1. Detailed comparison matrix of all three solutions
2. Recommendation with clear rationale
3. If migration is recommended: high-level migration plan
4. Risk assessment for each option
