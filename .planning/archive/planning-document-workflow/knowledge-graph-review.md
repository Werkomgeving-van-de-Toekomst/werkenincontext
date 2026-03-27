# Knowledge Graph System: Review & Enhancement Requirements

## Project Overview

The IOU-Modern project includes a GraphRAG (Retrieval-Augmented Generation with Graphs) knowledge graph system for visualizing and analyzing relationships between entities in government documents. This document outlines a review of the current implementation and proposes improvements and new features.

## Current Implementation Summary

### Core Components

| Component | File | Status |
|-----------|------|--------|
| Type Definitions | `iou-core/src/graphrag.rs` | ✅ Complete |
| Graph Logic | `iou-ai/src/graphrag.rs` | ⚠️ Partial |
| API Routes | `iou-api/src/routes/graphrag.rs` | ❌ Mock data only |
| Frontend Component | `iou-frontend/src/components/knowledge_graph.rs` | ⚠️ Static |
| Graph Explorer | `iou-frontend/src/pages/graphrag_explorer.rs` | ⚠️ Hardcoded data |

### Current Features

**Working:**
- Entity and Relationship type definitions
- KnowledgeGraph struct with petgraph backend
- Community detection (SCC algorithm)
- Stakeholder analysis methods (influence metrics, fuzzy name search)
- Graph statistics and basic path finding

**Partial/Incomplete:**
- Louvain community detection algorithm (returns early, incomplete implementation)
- Shortest path reconstruction (simplified)

**Missing/Mock:**
- API endpoints return hardcoded mock data
- No database persistence for graph entities/relationships
- Frontend visualizes static data only
- No integration with NER/LLM extraction pipeline
- No real-time updates or data synchronization

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Frontend (Dioxus)                          │
│  ┌──────────────────┐  ┌─────────────────────────────────┐     │
│  │ KnowledgeGraph   │  │ GraphRAGExplorer (vis-network)   │     │
│  │  (static CSS)    │  │  (hardcoded nodes/edges)         │     │
│  └──────────────────┘  └─────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (Mock responses)
┌─────────────────────────────────────────────────────────────────┐
│                         HTTP API (Axum)                          │
│  GET /graphrag/entities      → Mock data                        │
│  GET /graphrag/communities   → Mock data                        │
│  GET /graphrag/relations/:id → Mock data                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (Not connected)
┌─────────────────────────────────────────────────────────────────┐
│                    KnowledgeGraph (petgraph)                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ ✓ add_entity, add_relationship                           │   │
│  │ ✓ detect_communities (SCC)                               │   │
│  │ ⚠ detect_communities_louvain (incomplete)                │   │
│  │ ✓ get_stakeholder_influence, find_stakeholders_by_name   │   │
│  │ ⚠ shortest_path (simplified)                             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (Not connected)
┌─────────────────────────────────────────────────────────────────┐
│                      Storage (PostgreSQL)                        │
│  ❌ No graph persistence                                        │
│  ❌ No entity/relationship tables                               │
└─────────────────────────────────────────────────────────────────┘
```

## Identified Issues

### 1. API Mock Data Problem
- All GraphRAG endpoints return hardcoded mock data
- No connection to actual KnowledgeGraph implementation
- Users cannot see real document relationships

### 2. Incomplete Louvain Algorithm
The `detect_communities_louvain` method has a critical bug:
```rust
if gain > best_gain {
    return LouvainResult { /* early return with incomplete state */ };
}
```
This returns immediately when finding any improvement, rather than continuing the optimization.

### 3. No Data Persistence
- Graph entities and relationships exist only in memory
- No database schema for graph data
- Cannot query historical graph states

### 4. Frontend-Backend Disconnect
- Frontend components don't fetch from API
- vis-network graph uses hardcoded data
- No reactive updates when graph changes

### 5. Missing NER Integration
- GraphRAG types defined but not populated from documents
- No pipeline from document → NER → graph entities
- stakeholder extraction exists separately but not integrated

### 6. Limited Visualization Features
- No filtering by entity type or relationship
- No search functionality in graph view
- No path visualization between nodes
- No community highlighting
- No temporal view (how relationships evolved)

## Proposed Enhancements

### Priority 1: Foundation (Required for functionality)

#### A. Graph Persistence Layer
- Create database schema for entities, relationships, communities
- Implement CRUD operations for graph data
- Add graph snapshot/versioning for temporal analysis

#### B. API Integration
- Connect API endpoints to KnowledgeGraph implementation
- Implement real entity/relationship queries
- Add streaming support for large graphs

#### C. Frontend Data Binding
- Connect GraphRAGExplorer to live API data
- Implement reactive updates
- Add loading states and error handling

### Priority 2: Enhanced Features

#### D. Advanced Visualization
- Entity type filtering with legend toggle
- Relationship type filtering
- Search with auto-complete
- Path highlighting between selected nodes
- Community cluster highlighting
- Node size/weight based on influence metrics

#### E. Graph Algorithms
- Fix Louvain community detection
- Implement proper shortest path with visualization
- Add PageRank calculation for influence ranking
- Centrality metrics (betweenness, closeness, eigenvector)

#### F. Search & Discovery
- Full-text entity search
- Fuzzy name matching API endpoint
- Entity recommendation ("similar entities")
- Relationship path queries ("how are X and Y connected?")

### Priority 3: Advanced Features

#### G. Temporal Analysis
- Graph version comparison (diff)
- Temporal relationship visualization
- "What changed since X" views
- Relationship strength over time

#### H. Interactive Features
- Manual entity/relationship editing
- Collaborative annotations
- Graph layout presets (hierarchical, force-directed, circular)
- Export to formats (JSON, GEXF, GraphML)

#### I. Intelligence
- Relationship type prediction (ML)
- Entity reconciliation (duplicate detection)
- Community summarization (AI-generated)
- Anomaly detection (unusual relationships)

#### J. Integration
- Connect to NER pipeline for auto-population
- Link to stakeholder extraction
- Semantic search integration with embeddings
- Document-to-graph entity linking

## Success Criteria

### Phase 1 (Foundation)
- [ ] API returns real graph data from database
- [ ] Frontend displays live data with <500ms load time
- [ ] Can add/edit entities through UI

### Phase 2 (Enhancement)
- [ ] Louvain algorithm completes correctly
- [ ] Can search and filter entities in graph view
- [ ] Can visualize shortest path between two entities
- [ ] Community highlighting works

### Phase 3 (Advanced)
- [ ] Can compare graphs across time periods
- [ ] Entity reconciliation suggests duplicates
- [ ] Graph export works in 3+ formats

## Technical Constraints

- Must use existing petgraph dependency
- Must maintain compatibility with Dioxus WASM frontend
- Database: PostgreSQL with existing schema
- No additional JavaScript frameworks beyond vis-network
- Graph should support 10,000+ nodes without performance degradation

## Questions for Planning

1. **Priority**: Should we focus on connecting existing components first, or implement new features in parallel?

2. **Database**: Should graph data be stored in:
   - Existing PostgreSQL with new tables
   - Separate graph database (Neo4j, etc.)
   - Hybrid approach (PostgreSQL + graph cache)

3. **NER Integration**: Should we:
   - Build new NER pipeline from scratch
   - Integrate with existing `iou-ai/src/stakeholder/` module
   - Use external NER service (spaCy, OpenAI, etc.)

4. **Visualization Scale**: What is the target:
   - Small graphs (<100 nodes, pure JS rendering)
   - Medium graphs (100-1000 nodes, WebGL rendering)
   - Large graphs (>1000 nodes, server-side aggregation)

5. **Real-time Requirements**: Do we need:
   - WebSocket updates for live graph changes
   - Polling-based refresh
   - Manual refresh only

## Open Questions

- What is the expected size of graphs in production?
- Are there specific government data sources to integrate?
- Should the system support multi-tenant (separate graphs per organization)?
- What are the privacy/security requirements for entity data?
