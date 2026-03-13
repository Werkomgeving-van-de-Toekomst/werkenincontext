# Performance Baseline Documentation

**Status:** DRAFT - Pending measurements
**Date:** 2026-03-13
**Action Required:** Run tests and populate tables before section-02
**Phase:** Assessment (Phase 0)

## Query Latency Baseline

Measured from migration_tests baseline.rs:

| Endpoint | p50 | p95 | p99 | Notes |
|----------|-----|-----|-----|-------|
| information_domains | - | - | - | To be measured |
| information_objects | - | - | - | To be measured |
| documents | - | - | - | To be measured |
| search | - | - | - | To be measured |

## Concurrent User Capacity

**Current Baseline:** To be measured via load testing

| Metric | Value |
|--------|-------|
| Max concurrent users | - |
| Degradation point | - |
| P95 latency at capacity | - |

## Database Size

| Metric | Value |
|--------|-------|
| Database file | `data/iou_modern.duckdb` |
| Current size | - |
| Growth rate | - |

## WebSocket Baseline

| Metric | Value |
|--------|-------|
| Implementation | Custom Axum WebSocket |
| Location | `crates/iou-api/src/websockets/` |
| Features | Document status broadcasts |

## Notes

- Run `cargo test --test baseline` to refresh baseline
- Update this file after initial measurement
