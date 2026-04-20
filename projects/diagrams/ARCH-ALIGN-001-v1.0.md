# Architecture Comparison: Current vs Aligned

**Document ID**: ARCH-ALIGN-001-DIAG-v1.0
**Created**: 2026-04-20

## Current Architecture (As-Is)

```mermaid
C4Context
    title Current Architecture - Inconsistent Stacks

    Enterprise(minjus, "Ministry of Justice") {
        System(metadata, "Metadata Registry (002)", "Rust + ArangoDB") {
            Container(md_api, "REST/GraphQL API", "Rust/actix-web")
            ContainerDb(md_db, "ArangoDB", "Graph Database")
            Container(md_ui, "Admin UI", "Dioxus/WASM")
        }

        System(context, "Context-Aware Data (003)", "Java + PostgreSQL") {
            Container(ctx_api, "Context Capture", "Java/Spring Boot")
            ContainerDb(ctx_db, "PostgreSQL", "pgvector")
            Container(ctx_infer, "Inference", "Python/FastAPI")
            Container(ctx_ui, "Context Portal", "React")
        }

        System_Ext(legacy, "Legacy Systems", "Various")
    }

    Rel(legacy, md_api, "Query metadata", "REST")
    Rel(legacy, ctx_api, "Submit context", "REST")
    Rel(md_api, ctx_api, "Sync types", "REST (async)")

    Legend
        Ldec = "Integration Complexity: HIGH\nTwo languages, two databases, two frontend frameworks"
        Ltech = "Tech Stack Inconsistency"
    end
```

## Aligned Architecture Option A: Converged Stack

```mermaid
C4Context
    title Aligned Architecture - Unified Rust/ArangoDB Stack

    Enterprise(minjus, "Ministry of Justice") {
        System(platform, "Metadata & Context Platform", "Rust + ArangoDB") {
            Container(unified_api, "Unified API Gateway", "Rust/actix-web")
            ContainerDb(unified_db, "ArangoDB", "Graph + Context")
            Container(unified_ui, "Unified Admin Portal", "Dioxus/WASM")

            ContainerUni(metadata, "Metadata Service", "Rust crate")
            ContainerUni(context, "Context Service", "Rust crate")
            ContainerUni(inference, "Inference Service", "Rust + Claude SDK")
        }

        System_Ext(legacy, "Legacy Systems", "Various")
    }

    Rel(legacy, unified_api, "Query/Submit", "REST/GraphQL")
    Rel(unified_api, metadata, "Route to metadata", "Internal")
    Rel(unified_api, context, "Route to context", "Internal")
    Rel(unified_api, inference, "Request inference", "Internal")

    Legend
        Ldec = "Integration Complexity: LOW\nSingle language, single database, single framework"
        Ltech = "Unified Tech Stack"
    end
```

## Aligned Architecture Option B: Service Boundaries

```mermaid
C4Context
    title Aligned Architecture - Service Boundaries with API Gateway

    Enterprise(minjus, "Ministry of Justice") {
        Container(gateway, "API Gateway", "Kong/Envoy", "Unified entry point")

        System(metadata, "Metadata Registry (002)", "Rust + ArangoDB") {
            Container(md_api, "Metadata API", "Rust/actix-web")
            ContainerDb(md_db, "ArangoDB", "Graph Database")
        }

        System(context, "Context-Aware Data (003)", "Java + PostgreSQL") {
            Container(ctx_api, "Context API", "Java/Spring Boot")
            ContainerDb(ctx_db, "PostgreSQL", "pgvector")
        }

        Container(broker, "Message Broker", "Kafka/RabbitMQ", "Async events")
    }

    System_Ext(legacy, "Legacy Systems", "Various")

    Rel(legacy, gateway, "All requests", "REST")
    Rel(gateway, md_api, "Route metadata", "REST")
    Rel(gateway, ctx_api, "Route context", "REST")

    Rel(md_api, broker, "Publish events", "Kafka")
    Rel(ctx_api, broker, "Publish/Subscribe", "Kafka")

    Legend
        Ldec = "Integration Complexity: MEDIUM\nSeparate stacks with clear boundaries"
        Ltech = "Service-Oriented Pattern"
    end
```

## Technology Stack Comparison Table

| Layer | Project 002 (Current) | Project 003 (Current) | Option A (Unified) | Option B (Boundaries) |
|-------|---------------------|---------------------|-------------------|---------------------|
| **Backend** | Rust | Java | Rust | Rust + Java |
| **API Framework** | actix-web | Spring Boot | actix-web | Both |
| **Database** | ArangoDB | PostgreSQL | ArangoDB | Both |
| **Frontend** | Dioxus | React | Dioxus | Both |
| **AI/ML** | None | Python | Rust SDK | Python |
| **Language Count** | 1 | 2 | 1 | 3 |
| **Database Count** | 1 | 1 | 1 | 2 |

## Effort Comparison

| Option | Migration Effort | Cost | Timeline | Operational Complexity |
|--------|-----------------|------|----------|---------------------|
| **A: Unified** | HIGH (rewrite 003) | €1.2-1.8M | 6-9 months | LOW (single stack) |
| **B: Boundaries** | LOW (add gateway) | €200-400k | 2-3 months | MEDIUM (multiple stacks) |
| **C: DB Only** | MEDIUM (migrate 002) | €600-900k | 4-6 months | MEDIUM (2 languages, 1 DB) |

---

**Visual Summary**: Option A provides the cleanest long-term architecture but requires significant upfront investment. Option B minimizes disruption while establishing clear integration patterns.
