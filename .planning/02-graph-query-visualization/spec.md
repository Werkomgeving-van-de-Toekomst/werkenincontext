# Spec: Graph Query & Visualization

**Split:** 02-graph-query-visualization
**Created:** 2026-03-16
**Estimated Duration:** 1-2 sprints

## Overview

Query APIs and interactive visualization for exploring stakeholder connection networks. Enable users to find paths between stakeholders, identify influencers, and understand network structure.

## Context

### Existing Infrastructure

IOU-Modern has:
- `KnowledgeGraph` with basic queries (neighbors, related_entities, shortest_path)
- Community detection (SCC, Louvain)
- Dioxus WASM frontend

**Gap:** Need:
- Higher-level query APIs
- Graph visualization component
- Centrality and influence metrics
- Natural language query interface

### User Goals

- "Who connects Person A to Organization B?"
- "Who are the most influential stakeholders for Topic X?"
- "Which departments collaborate?"
- "Show the decision chain for Document Y"

## Requirements

### Core Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| GQV-01 | Path Query API | Find connections between stakeholders with path details |
| GQV-02 | Centrality Metrics | PageRank, betweenness, degree centrality for influence |
| GQV-03 | Community Detection | Identify stakeholder clusters and hierarchies |
| GQV-04 | Graph Visualization | Interactive Dioxus component with force-directed layout |
| GQV-05 | Natural Language Query | Answer questions in plain Dutch/English |
| GQV-06 | Export | Export graphs as JSON, GraphML, or visualizations |

### Query Types

**Path Queries:**
```
GET /api/graph/path?from=PERSON_ID&to=ORG_ID
→ Returns: [Person A → advises → Person B → reports_to → Org X]
```

**Influence Queries:**
```
GET /api/graph/central?domain=MinFin&limit=10
→ Returns: Top 10 influencers by PageRank
```

**Community Queries:**
```
GET /api/graph/communities?algorithm=louvain
→ Returns: Detected clusters with member lists
```

**Neighborhood Queries:**
```
GET /api/graph/neighbors/{id}?depth=2
→ Returns: All entities within 2 hops
```

### Visualization Features

| Feature | Description |
|---------|-------------|
| Force-directed layout | Automatic node positioning |
| Node sizing | Size = influence/centrality |
| Edge thickness | Thickness = relationship strength |
| Color coding | By entity type or community |
| Filtering | By relationship type, time period |
| Expand/collapse | Show/hide subgraphs |
| Search | Find and highlight nodes |
| Tooltip | Entity details on hover |

## Technical Approach

### Frontend Stack

- **Framework:** Dioxus (WASM) - consistent with IOU-Modern
- **Graph Library:** vis.js Network or Cytoscape.js
- **State:** Dioxus hooks for graph state
- **API:** REST calls to backend

### Backend APIs

```rust
/// Get path between entities
async fn get_path(
    from: Uuid,
    to: Uuid,
    max_hops: usize,
) -> Result<Vec<PathStep>>

/// Get central entities by PageRank
async fn get_central_entities(
    domain: Option<Uuid>,
    limit: usize,
) -> Vec<EntityWithScore>

/// Get communities
async fn get_communities(
    algorithm: CommunityAlgorithm,
) -> Vec<Community>

/// Get entity neighborhood
async fn get_neighborhood(
    id: Uuid,
    depth: usize,
) -> GraphSubgraph
```

### Graph Algorithms

Extend `KnowledgeGraph` with:

```rust
impl KnowledgeGraph {
    /// PageRank centrality
    pub fn pagerank(&self, iterations: usize, damping: f64) -> Vec<(EntityId, f64)>

    /// Betweenness centrality
    pub fn betweenness(&self) -> Vec<(EntityId, f64)>

    /// Degree centrality
    pub fn degree_centrality(&self) -> Vec<(EntityId, f64)>

    /// Shortest path with reconstruction
    pub fn shortest_path_detailed(&self, from: Uuid, to: Uuid) -> Option<Path>
}
```

### Natural Language Query

Pattern matching for common questions:

| Pattern | Intent | Example |
|---------|--------|---------|
| "wie verbindt {X} aan {Y}" | Path query | "Wie verbindt de minister aan de ambtenaar?" |
| "wie zijn invloedrijkste bij {X}" | Centrality | "Wie zijn de invloedrijkste bij MinFin?" |
| "welke afdelingen werken samen" | Community | "Welke afdelingen werken samen?" |
| "toon het netwerk van {X}" | Neighborhood | "Toon het netwerk van Jan de Vries" |

## UI Design

### Graph Explorer Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Stakeholder Network                     [Search] [Filter]  │
├─────────────────────┬───────────────────────────────────────┤
│                     │                                       │
│  Controls           │         Graph Visualization           │
│  ─────────          │         ────────────────              │
│  [Zoom In/Out]      │                                       │
│  [Reset Layout]     │       [Force-directed graph]          │
│  [Export PNG]        │                                       │
│  [Export JSON]       │                                       │
│                     │                                       │
│  Filters:           │                                       │
│  ☑ Persons          │       Legend:                         │
│  ☑ Organizations    │       ● Person  ○ Organization        │
│  ☑ Show Relations   │       ─ reports_to  ┄ collaborates    │
│                     │                                       │
│  Relationship:       │                                       │
│  [All ▼]            │                                       │
│                     │                                       │
│  Selected:          │                                       │
│  [Entity details]    │                                       │
└─────────────────────┴───────────────────────────────────────┘
```

### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /api/graph/entities | List all entities |
| GET | /api/graph/entities/:id | Get entity details |
| GET | /api/graph/path | Find path between entities |
| GET | /api/graph/central | Get top influencers |
| GET | /api/graph/communities | Get communities |
| GET | /api/graph/neighbors/:id | Get neighborhood |
| POST | /api/graph/query | Natural language query |
| GET | /api/graph/export/:format | Export graph |

## Acceptance Criteria

- [ ] Graph renders with 500+ nodes in <3 seconds
- [ ] Path queries return in <2 seconds
- [ ] Users can navigate/explore the graph intuitively
- [ ] Natural language queries answer common questions
- [ ] Export produces valid JSON/GraphML
- [ ] Visualization works in browser (WASM)

## Success Metrics

| Metric | Target |
|--------|--------|
| Graph render time | <3 seconds for 500 nodes |
| Query response time | <2 seconds |
| NL query accuracy | >80% for common patterns |

## Out of Scope

- Real-time graph updates (polling or WebSocket refresh is fine)
- 3D graph visualization
- Mobile-specific UI optimization
- Social network analysis (beyond stakeholder context)

## Constraints

- Must use Dioxus for frontend
- Must integrate with existing KnowledgeGraph
- Must handle large graphs (1000+ nodes) gracefully
- Browser-based (no server-side rendering)

---

## References

- Requirements: `.planning/graphrag-stakeholder-requirements.md`
- KnowledgeGraph: `crates/iou-ai/src/graphrag.rs`
- vis.js: https://visjs.github.io/vis-network/
- Cytoscape.js: https://js.cytoscape.org/
