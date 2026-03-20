# Architecture Diagrams: IOU-Modern

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:diagram`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-001-DIAG-v1.0 |
| **Document Type** | Architecture Diagrams (C4 + Mermaid) |
| **Project** | IOU-Modern (Project 001) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-03-20 |
| **Last Modified** | 2026-03-20 |
| **Review Cycle** | Per release |
| **Next Review Date** | On major release |
| **Owner** | Solution Architect |

## Revision History

| Version | Date | Author | Changes | Approved By |
|---------|------|--------|---------|-------------|
| 1.0 | 2026-03-20 | ArcKit AI | Initial creation from `/arckit:diagram` command | PENDING |

---

## Executive Summary

This document contains the architecture diagrams for IOU-Modern using the C4 model plus Mermaid diagrams for detailed views.

---

## 1. System Context (C4 Level 1)

```mermaid
flowchart TB
    subgraph "IOU-Modern System"
        IOU[IOU-Modern Platform<br/>Context-driven Information Management]
    end

    subgraph "Users"
        E[Government Employees<br/>~50,000]
        M[Domain Owners<br/>~5,000]
        A[Administrators<br/>~100]
    end

    subgraph "External Systems"
        WM[Woo Portal<br/>Publication Platform]
        CS[Case Management Systems<br/>Sqills, Centric]
        HR[HR Systems<br/>Employee Data]
    end

    subgraph "Regulators"
        AP[Autoriteit Persoonsgegevens<br/>Dutch DPA]
        OOB[Inspectie OOB<br/>Woo Enforcement]
        NA[Nationaal Archief<br/>Record Keeping]
    end

    E -->|Create, Access| IOU
    M -->|Manage, Approve| IOU
    A -->|Configure| IOU

    IOU <-->|Publish Decisions| WM
    IOU <-->|Ingest Documents| CS
    IOU <-->|Sync Employees| HR

    AP -.Audit AVG.- IOU
    OOB -.Audit Woo.- IOU
    NA -.Audit Retention.- IOU
```

---

## 2. Container View (C4 Level 2)

```mermaid
flowchart TB
    subgraph "IOU-Modern Architecture"
        direction TB

        subgraph "Client Layer"
            WEB[Dioxus WASM App<br/>User Interface]
        end

        subgraph "API Layer"
            API[Axum REST API<br/>Authentication & Routing]
        end

        subgraph "Core Services"
            CORE[iou-core<br/>Domain Logic & Entities]
        end

        subgraph "AI Pipeline"
            RES[Research Agent<br/>Context Gathering]
            CON[Content Agent<br/>Document Generation]
            COMP[Compliance Agent<br/>Woo/AVG Checking]
            REV[Review Agent<br/>Quality Assessment]
        end

        subgraph "Knowledge Graph"
            NER[NER Service<br/>Entity Extraction]
            GR[GraphRAG Service<br/>Knowledge Graph]
            EMB[Embedding Service<br/>Vector Search]
        end

        subgraph "Data Layer"
            PG[(PostgreSQL<br/>Transactional Data)]
            DD[(DuckDB<br/>Analytics)]
            S3[(MinIO/S3<br/>Document Storage)]
        end
    end

    WEB -->|HTTPS/TLS 1.3| API
    API --> CORE
    API --> NER
    API --> GR
    API --> EMB

    API -->|Orchestrate| RES
    API -->|Orchestrate| CON
    API -->|Orchestrate| COMP
    API -->|Orchestrate| REV

    RES -->|Read/Write| PG
    CON -->|Read/Write| PG
    COMP -->|Read/Write| PG
    REV -->|Read/Write| PG

    NER --> PG
    GR --> DD
    EMB --> DD

    API --> PG
    API --> DD
    API --> S3
```

---

## 3. Component View (C4 Level 3) - API Gateway

```mermaid
flowchart TB
    subgraph "API Gateway Components"
        direction TB

        subgraph "Middleware Pipeline"
            AUTH[Authentication<br/>DigiD + MFA]
            RBAC[Authorization<br/>Role-Based Access Control]
            RLS[Row-Level Security<br/>Organization Isolation]
            RATE[Rate Limiting<br/>Request Throttling]
            AUDIT[Audit Logging<br/>PII Access Tracking]
        end

        subgraph "Controllers"
            DOM[Domain Controller<br/>CRUD Operations]
            DOC[Document Controller<br/>Workflow Management]
            USR[User Controller<br/>Profile Management]
            SRC[Search Controller<br/>Full-text + Semantic]
            WOO[Woo Controller<br/>Publication Workflow]
            SAR[SAR Controller<br/>Subject Access Requests]
        end
    end

    AUTH --> RBAC --> RLS --> RATE --> AUDIT

    AUDIT --> DOM
    AUDIT --> DOC
    AUDIT --> USR
    AUDIT --> SRC
    AUDIT --> WOO
    AUDIT --> SAR
```

---

## 4. Component View - AI Pipeline

```mermaid
flowchart LR
    subgraph "AI Pipeline Architecture"
        direction LR

        subgraph "Input"
            DOC[Document<br/>Template]
            CONTEXT[Domain<br/>Context]
        end

        subgraph "Research Agent"
            R1[Fetch Domain<br/>Context]
            R2[Search Similar<br/>Documents]
            R3[Extract Key<br/>Information]
        end

        subgraph "Content Agent"
            C1[Apply<br/>Template]
            C2[Generate<br/>Content]
            C3[Format<br/>Output]
        end

        subgraph "Compliance Agent"
            CP1[Woo Check<br/>Classify]
            CP2[AVG Check<br/>PII Scan]
            CP3[Archiefwet<br/>Retention]
            CP4[Score<br/>Compliance]
        end

        subgraph "Review Agent"
            RV1[Quality<br/>Check]
            RV2[Suggest<br/>Improvements]
            RV3[Generate<br/>Report]
        end

        subgraph "Output"
            DRAFT[Draft<br/>Document]
            SCORE[Compliance<br/>Score]
            META[Metadata]
        end
    end

    DOC --> C1
    CONTEXT --> R1

    R1 --> R2 --> R3 --> C2
    C1 --> C2 --> C3 --> CP1
    CP1 --> CP2 --> CP3 --> CP4
    C3 --> RV1
    CP4 --> RV1 --> RV2 --> RV3

    C3 --> DRAFT
    CP4 --> SCORE
    RV3 --> META
```

---

## 5. Component View - Knowledge Graph

```mermaid
flowchart TB
    subgraph "Knowledge Graph Architecture"
        direction TB

        subgraph "Extraction Layer"
            INGEST[Document Ingestion<br/>From S3/MinIO]
            NER_PIPE[NER Pipeline<br/>Regex + Rules]
            ENT_RES[Entity Resolution<br/>Deduplication]
        end

        subgraph "Graph Building"
            NODE[Node Creation<br/>Entity Nodes]
            EDGE[Edge Detection<br/>Relationships]
            COMM[Community Detection<br/>Clustering]
            SUM[Summarization<br/>AI Description]
        end

        subgraph "Query Layer"
            TRAVERSE[Graph Traversal<br/>BFS/DFS]
            REL_SEARCH[Relationship<br/>Search]
            COMM_EXP[Community<br/>Expansion]
            SIM[Similarity<br/>Search]
        end

        subgraph "Storage"
            PG_NODES[(PostgreSQL<br/>Nodes/Edges)]
            DD_VECT[(DuckDB<br/>Vectors)]
        end
    end

    INGEST --> NER_PIPE --> ENT_RES --> NODE --> EDGE --> COMM --> SUM
    NODE --> PG_NODES
    SUM --> DD_VECT

    TRAVERSE --> PG_NODES
    REL_SEARCH --> PG_NODES
    COMM_EXP --> PG_NODES
    SIM --> DD_VECT
```

---

## 6. Data Model View (ERD)

```mermaid
erDiagram
    %% Core Organization
    ORGANIZATION ||--o{ DEPARTMENT : contains
    ORGANIZATION ||--o{ INFORMATION_DOMAIN : owns
    ORGANIZATION ||--o{ USER : employs
    DEPARTMENT ||--o{ USER : contains
    DEPARTMENT }|--|| MANAGER : managed_by
    USER }|--|| MANAGER : is_manager_of

    %% User and Roles
    USER ||--o{ USER_ROLE : has
    ROLE ||--o{ USER_ROLE : assigned_to
    ROLE }|--|| PERMISSION : includes

    %% Domains and Documents
    INFORMATION_DOMAIN ||--o{ INFORMATION_OBJECT : contains
    INFORMATION_DOMAIN ||--o{ DOCUMENT : generates
    INFORMATION_DOMAIN }|--|| OWNER : owned_by
    INFORMATION_DOMAIN ||--o{ PARENT_DOMAIN : is_subdomain_of

    %% Documents
    TEMPLATE ||--o{ DOCUMENT : defines
    DOCUMENT ||--o{ AUDIT_TRAIL : logs
    DOCUMENT ||--|| CURRENT_VERSION : has
    DOCUMENT ||--o| PREVIOUS_VERSION : versioned_from

    %% Knowledge Graph
    INFORMATION_OBJECT ||--o{ ENTITY : extracts
    ENTITY ||--o{ RELATIONSHIP : has_source
    ENTITY ||--o{ RELATIONSHIP_TARGET : has_target
    ENTITY ||--o{ COMMUNITY : member_of

    %% Cross-Domain Relationships
    INFORMATION_DOMAIN ||--o{ DOMAIN_RELATION : relates_from
    INFORMATION_DOMAIN ||--o{ DOMAIN_RELATION_TARGET : relates_to

    %% Vectors
    INFORMATION_DOMAIN ||--o{ CONTEXT_VECTOR : embeds
```

---

## 7. Security Architecture

```mermaid
flowchart TB
    subgraph "Security Layers"
        direction TB

        subgraph "Layer 1: Network"
            LB[Load Balancer<br/>DDoS Protection]
            WAF[Web Application<br/>Firewall]
            TLS[TLS 1.3<br/>Encryption]
        end

        subgraph "Layer 2: Authentication"
            DIG[DigiD<br/>Integration]
            MFA[Multi-Factor<br/>Authentication]
            SSO[Single Sign-On<br/>SAML]
        end

        subgraph "Layer 3: Authorization"
            RBAC[Role-Based<br/>Access Control]
            RLS[Row-Level<br/>Security]
            SCOPED[Domain-Scoped<br/>Permissions]
        end

        subgraph "Layer 4: Data Protection"
            ENC_REST[Encryption<br/>At Rest AES-256]
            ENC_TRANS[Encryption<br/>In Transit]
            PII_TRACK[PII Tracking<br/>Entity Level]
            MASK[Data Masking<br/>For Display]
        end

        subgraph "Layer 5: Monitoring"
            AUD[Audit Trail<br/>All Operations]
            PII_AUDIT[PII Access<br/>Logging]
            ALERT[Security<br/>Alerts]
            BREACH[Breach<br/>Detection]
        end
    end

    LB --> WAF --> TLS --> DIG --> MFA --> RBAC --> RLS --> ENC_REST --> PII_TRACK --> AUD
```

---

## 8. Woo Publication Flow

```mermaid
stateDiagram-v2
    [*] --> Draft: Document Created

    Draft --> ComplianceCheck: AI Agent Analysis
    ComplianceCheck --> Draft: Low Confidence
    ComplianceCheck --> HumanReview: High Confidence

    HumanReview --> ChangesRequested: Reviewer Feedback
    HumanReview --> Approved: Reviewer Approved

    ChangesRequested --> Draft: Author Updates

    Approved --> WooAssessment: is_woo_relevant?
    WooAssessment --> NotWoo: false
    WooAssessment --> Classification: true

    Classification --> Refusal: Not Openbaar
    Classification --> PublicationReady: Openbaar

    Refusal --> Withheld: Document Withheld
    Withheld --> [*]

    PublicationReady --> WooApproval: Human Review Required
    WooApproval --> Rejected: Compliance Concerns
    WooApproval --> Published: Approved for Publication

    Rejected --> Draft: Address Concerns
    Published --> [*]
```

---

## 9. Data Flow: Document Processing

```mermaid
flowchart LR
    subgraph "Source"
        SYS[Source System]
    end

    subgraph "Ingestion"
        ETL[ETL Pipeline]
    end

    subgraph "Processing"
        NER[NER<br/>Entity Extraction]
        CLASS[Classification<br/>AI + Manual]
        COMPL[Compliance<br/>Checking]
    end

    subgraph "Storage"
        PG[(PostgreSQL)]
        S3[(S3/MinIO)]
    end

    subgraph "Knowledge Graph"
        GRAPH[GraphRAG<br/>Knowledge Graph]
    end

    SYS -->|Batch/Real-time| ETL --> S3
    S3 --> NER --> CLASS --> COMPL
    NER --> GRAPH
    CLASS --> PG
    COMPL --> PG
```

---

## 10. Deployment Architecture

```mermaid
flowchart TB
    subgraph "External"
        USERS[Users]
    end

    subgraph "DMZ"
        LB[Load Balancer<br/>HAProxy/Nginx]
        WAF[ModSecurity<br/>WAF]
    end

    subgraph "Kubernetes Cluster"
        direction LR
        subgraph "Application"
            API1[API Pod 1]
            API2[API Pod 2]
            APIN[API Pod N]
        end
        subgraph "Jobs"
            ETL[ETL Job<br/>Scheduled]
            DEL[Deletion Job<br/>Monthly]
        end
    end

    subgraph "Database Cluster"
        direction LR
        PG_PRI[(Primary<br/>PostgreSQL)]
        PG_REP[(Replica<br/>PostgreSQL)]
    end

    subgraph "Storage"
        MINIO[MinIO Cluster<br/>Distributed]
    end

    USERS --> LB --> WAF
    WAF --> API1 & API2 & APIN
    API1 & API2 & APIN --> PG_PRI & PG_REP
    API1 & API2 & APIN --> MINIO
    ETL --> PG_PRI
    DEL --> PG_PRI
    PG_PRI -->|Streaming Replication| PG_REP
```

---

## 11. Sequence Diagram: Subject Access Request

```mermaid
sequenceDiagram
    actor C as Citizen
    participant API as REST API
    participant AUTH as Auth Service
    participant DB as PostgreSQL
    participant S3 as S3/MinIO

    C->>API: POST /api/v1/subject-access-request
    API->>AUTH: Verify Identity (DigiD)
    AUTH-->>API: Identity Confirmed

    API->>DB: Query user.email
    DB-->>API: Email Found

    API->>DB: Query InformationObjects WHERE created_by = user
    DB-->>API: List of Documents

    API->>DB: Query Entities (Person) WHERE name matches
    DB-->>API: List of Person Entities

    API->>DB: Query AuditTrail WHERE user_id = user
    DB-->>API: Access History

    API->>API: Compile SAR Report
    API-->>C: JSON Response (within 30 days)
```

---

## 12. Component Diagram: AI Orchestration

```mermaid
flowchart TB
    subgraph "AI Orchestrator"
        ORCH[Pipeline Orchestrator]

        subgraph "Agent Pool"
            RES[Research Agent]
            CON[Content Agent]
            COMP[Compliance Agent]
            REV[Review Agent]
        end

        subgraph "Shared Services"
            PROMPT[Prompt<br/>Templates]
            LLM[LLM Client<br/>OpenAI/Anthropic]
            VECT[Vector<br/>Store]
        end
    end

    ORCH --> RES
    ORCH --> CON
    ORCH --> COMP
    ORCH --> REV

    RES --> PROMPT
    CON --> PROMPT
    COMP --> PROMPT
    REV --> PROMPT

    PROMPT --> LLM
    LLM --> VECT

    VECT --> RES
    VECT --> CON
```

---

## Diagram Legend

| Symbol | Meaning |
|--------|---------|
| `||--o{| | One-to-many (required to many) |
| `||--o|` | One-to-many (required to optional) |
| `}|--||` | Many-to-one (optional to required) |
| `[(...)]` | Database table |
| `[...]` | Component/Container |
| `-->` | Data flow |
| `-.->` | Dependency/Feedback |

---

## Related Documents

| Document | ID |
|----------|-----|
| Data Model | ARC-001-DATA-v1.0 |
| Requirements | ARC-001-REQ-v1.0 |
| High-Level Design | (This document) |
| ADR | ARC-001-ADR-v1.0 |

---

**END OF ARCHITECTURE DIAGRAMS**

## Generation Metadata

**Generated by**: ArcKit `/arckit:diagram` command
**Generated on**: 2026-03-20
**ArcKit Version**: 4.3.1
**Project**: IOU-Modern (Project 001)
**AI Model**: Claude Opus 4.6
