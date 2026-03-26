<!-- SPLIT_MANIFEST
01-entity-relationship-extraction
02-graph-query-visualization
END_MANIFEST -->

# Project Manifest: GraphRAG Stakeholder Connections

**Created:** 2026-03-16
**Requirements File:** `graphrag-stakeholder-requirements.md`
**Project:** IOU-Modern GraphRAG Integration

---

## Overview

Stakeholder connection analysis using GraphRAG methodology. Extract people and organizations from government documents, identify relationships, and provide query/visualization capabilities to understand stakeholder networks.

## Splits

1. **Entity & Relationship Extraction** - NER pipeline for Dutch stakeholder entities
2. **Graph Query & Visualization** - Storage, APIs, and interactive graph UI

---

## Split Details

### 01: Entity & Relationship Extraction

**Summary:** Extract persons, organizations, and relationships from Dutch government documents.

**Key Deliverables:**
- Dutch NER for stakeholder entities (persons, organizations, roles)
- Relationship extraction (reports_to, collaborates_with, advises)
- Integration with document pipeline
- Entity normalization and deduplication

**User Outcomes:**
- Documents are automatically analyzed for stakeholders
- Stakeholder network is built incrementally
- No manual entity entry required

**Technical Notes:**
- Extend existing `iou-ai` NER module (currently regex-based)
- Consider spaCy Dutch or fine-tuned transformer
- Extract from: Woo documents, policy briefs, meeting notes
- Store in existing KnowledgeGraph structure

### 02: Graph Query & Visualization

**Summary:** Query APIs and interactive UI for exploring stakeholder connections.

**Key Deliverables:**
- REST API for graph queries (paths, centrality, communities)
- Interactive graph visualization (Dioxus component)
- Natural language query interface
- Export capabilities (JSON, GraphML)

**User Outcomes:**
- Find who connects X to Y
- Identify influential stakeholders
- Visualize stakeholder clusters
- Answer "who cares about topic Z?"

**Technical Notes:**
- Extend `KnowledgeGraph` with query methods
- Use vis.js or Cytoscape.js for visualization
- Integrate with Dioxus frontend
- PostgreSQL for graph persistence

---

## Execution Order

```
01-entity-relationship-extraction → 02-graph-query-visualization
```

Split 01 must complete first to populate the graph. Split 02 can be developed in parallel but needs data for testing.

## Success Criteria

- [ ] Entities extracted with >85% precision
- [ ] Relationships classified with >75% precision
- [ ] Path queries return in <2 seconds
- [ ] Graph visualization renders <3 seconds for 500 nodes
- [ ] Natural language queries answered accurately

---

*Manifest created: 2026-03-16*
