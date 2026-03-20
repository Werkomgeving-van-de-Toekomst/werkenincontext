# Deep Project Interview: GraphRAG Stakeholder Connections

**Date:** 2026-03-16
**Requirements File:** `graphrag-stakeholder-requirements.md`

## User Intent

Build a GraphRAG-based stakeholder connection analysis system integrated into IOU-Modern, inspired by modern GraphRAG implementations in Rust.

## Interview Summary

### Q1: What specific GraphRAG capability?
**A:** Knowledge graph builder - focus on building and querying a knowledge graph of people, organizations, and their connections.

### Q2: Standalone or integrated?
**A:** IOU-Modern integration - leverage existing GraphRAG infrastructure in `crates/iou-ai/src/graphrag.rs`

### Q3: Replace existing deep-project work?
**A:** Start fresh - focus on GraphRAG stakeholder connections as the priority.

## Synthesis

**Feature Focus:**
- Stakeholder entity extraction (persons, organizations)
- Relationship extraction (hierarchical, collaborative, temporal)
- Graph query and visualization
- Question answering over stakeholder networks

**Technical Approach:**
- Extend existing `KnowledgeGraph` (petgraph-based)
- Add stakeholder-specific entity/relationship types
- Build visualization UI (Dioxus + graph library)
- Integrate with document pipeline for extraction

**Proposed Splits:**
1. **Entity & Relationship Extraction** - NER pipeline for Dutch stakeholders
2. **Graph Query & Visualization** - Storage, APIs, and interactive UI
