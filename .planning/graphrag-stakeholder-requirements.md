# GraphRAG Stakeholder Connection Analysis

**Created:** 2026-03-16
**Project:** IOU-Modern Integration
**Focus:** Knowledge graph builder for stakeholder connections

## Overview

Build a GraphRAG-based stakeholder connection analysis system integrated into IOU-Modern. The system will extract people, organizations, and their relationships from government documents, then provide query and visualization capabilities to understand stakeholder networks.

## Context

### Existing Infrastructure

IOU-Modern has basic GraphRAG implementation (`crates/iou-ai/src/graphrag.rs`):
- `KnowledgeGraph` with petgraph backend
- Basic entity and relationship types
- Community detection (SCC, Louvain)
- Path finding and neighbor queries

**Gap:** The current implementation is generic. Stakeholder-specific analysis needs:
1. Person/organization extraction from Dutch government documents
2. Relationship types specific to stakeholder analysis (reports_to, collaborates_with, etc.)
3. Temporal relationship tracking
4. Influence scoring and centrality metrics
5. Visual graph exploration UI

## Core Requirements

### GRAP-01: Stakeholder Entity Extraction
Extract persons and organizations from documents with:
- Names, roles, departments
- Contact information
- Organizational hierarchy
- Confidence scores

### GRAP-02: Relationship Extraction
Identify and classify stakeholder relationships:
- **Hierarchical:** reports_to, manages, oversees
- **Collaborative:** works_with, partner_with, advises
- **Temporal:** relationships with validity periods
- **Strength:** relationship weight based on frequency/context

### GRAP-03: Graph Storage & Query
Persistent graph storage with:
- Add/remove entities and relationships
- Find connections between stakeholders (paths)
- Find central influencers (centrality algorithms)
- Community detection for stakeholder clusters

### GRAP-04: Visualization UI
Interactive graph visualization showing:
- Nodes as stakeholders (size = influence)
- Edges as relationships (thickness = strength)
- Clustering by community/department
- Filtering by relationship type, time period
- Expand/collapse subgraphs

### GRAP-05: Question Answering
Natural language queries like:
- "Who connects Minister X to Department Y?"
- "Who are the most influential stakeholders for policy Z?"
- "Which stakeholders collaborate across departments?"
- "Show the decision chain for topic T"

## User Stories

1. **As a policy analyst**, I want to see who influences a decision so I understand the power dynamics.
2. **As a project manager**, I want to find potential collaborators across departments so I can leverage existing connections.
3. **As a compliance officer**, I want to trace decision chains for accountability.
4. **As a communications officer**, I want to identify key stakeholders to target for outreach.

## Technical Approach

### Entity Types

| Type | Attributes | Example |
|------|------------|---------|
| Person | name, role, department, email, phone | "Jan de Vries, beleidsmedewerker MinFin" |
| Organization | name, type, parent_org | "Ministerie van Financiën" |
| Location | name, type | "Den Haag" |
| Document/Case | reference, type, date | "Woo besluit 2024-001" |

### Relationship Types

| Type | Direction | Description |
|------|-----------|-------------|
| reports_to | Person → Person | Organizational hierarchy |
| manages | Person → Organization | Management responsibility |
| collaborates_with | Person ↔ Person | Working relationship |
| advises | Person/Org → Person/Org | Advisory role |
| mentioned_in | Person/Org → Document | Document reference |
| located_in | Person/Org → Location | Geographic association |

### Algorithms

- **Centrality:** PageRank, betweenness, degree
- **Community:** Louvain, label propagation
- **Path:** Dijkstra, BFS for shortest paths
- **Influence:** Eigenvector centrality

## Integration Points

### With IOU-Modern

- **Document Pipeline:** Extract entities during document creation
- **Search API:** Search for stakeholders and their connections
- **Frontend:** Graph visualization component using Dioxus + vis.js/cytoscape.js
- **Database:** PostgreSQL for graph persistence (APOC or agensgraph)

### External Services

- Optional: NER service for entity extraction (spaCy Dutch, or custom)
- Optional: LLM for relationship extraction

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | /api/graph/entities | Add entity to graph |
| POST | /api/graph/relationships | Add relationship |
| GET | /api/graph/stakeholders/:id | Get stakeholder with connections |
| GET | /api/graph/path/:from/:to | Find path between stakeholders |
| GET | /api/graph/central | Get most influential stakeholders |
| GET | /api/graph/communities | Get stakeholder clusters |
| POST | /api/graph/query | Natural language query |

## Success Criteria

| Metric | Target |
|--------|--------|
| Entity extraction precision | >85% |
| Relationship extraction precision | >75% |
| Query response time | <2 seconds for <1000 nodes |
| Graph visualization render | <3 seconds for <500 nodes |

## Out of Scope

- Social media integration
- Real-time relationship updates from external sources
- Advanced ML-based embedding (for initial version)
- International stakeholder analysis (Netherlands focus)

## Constraints

- Dutch language primary
- Must comply with AVG/GDPR for personal data
- Graph visualization must work in browser (WASM)
- Must integrate with existing petgraph-based KnowledgeGraph

---

## References

- Existing GraphRAG: `crates/iou-ai/src/graphrag.rs`
- Entity types: `iou-core::graphrag`
