# Integration Notes: Opus Review Feedback

## Decisions on Feedback Integration

### Critical Gaps - Integrating

**1. Configuration Migration Strategy**
- **Action**: Add to Phase 1 as new subsection
- **Approach**: Create migration script to convert `domain_configs` table to YAML format
- **Reasoning**: Existing data must be preserved; prevents breaking existing workflows

**2. Approver Resolution Service**
- **Action**: Add `ApproverResolver` service to Phase 2
- **Approach**: Map role-based approvers to actual user IDs
- **Reasoning**: Essential for role-based approval assignments

**3. State Machine Integration**
- **Action**: Add explicit state transition diagram to Phase 2.1
- **Approach**: Document how single-stage and multi-stage workflows coexist
- **Reasoning**: Critical for backward compatibility

**4. Webhook Configuration**
- **Action**: Add webhook_configs table to Phase 1 schema
- **Approach**: Store webhook URLs per domain/document type
- **Reasoning**: Required for escalation functionality

**5. Error Handling Strategy**
- **Action**: Add error types and handling patterns throughout
- **Approach**: Define result types and error propagation
- **Reasoning**: Production systems need robust error handling

### High Priority Risks - Integrating

**Config Hot-Reload Race Conditions**
- **Action**: Add workflow_config_versions table to track active config per stage
- **Reasoning**: Prevents inconsistent state during config changes

**Circular Delegation Chains**
- **Action**: Add cycle detection to DelegationResolver::create_delegation
- **Reasoning**: Prevents infinite loops in delegation

**Orphaned Stage Instances**
- **Action**: Add StageCleanupService to Phase 4
- **Reasoning**: Background cleanup prevents resource leaks

**Performance of Deadline Checks**
- **Action**: Add query optimization notes and indexing strategy
- **Reasoning**: Ensures scalability

### Not Integrating (With Rationale)

**Event Sourcing for Stage Transitions**
- **Reasoning**: Significant architectural change, adds complexity
- **Alternative**: Enhanced audit trail provides sufficient observability

**Command Pattern for State Transitions**
- **Reasoning**: May be over-engineering for this scope
- **Alternative**: Transaction boundaries provide similar guarantees

**Property-Based Tests**
- **Reasoning**: Valuable but not critical for initial implementation
- **Alternative**: Can be added in testing phase

**Metrics Collection**
- **Reasoning**: Important but can be added post-MVP
- **Alternative**: Use existing logging initially

## Modifications to Plan

### Additions to Phase 1:
- Config migration script subsection
- webhook_configs table
- workflow_config_versions table
- Error type definitions

### Additions to Phase 2:
- ApproverResolver service
- Explicit state transition documentation
- Error handling patterns

### Additions to Phase 3:
- Cycle detection in delegation service
- Enhanced validation

### Additions to Phase 4:
- StageCleanupService for orphaned stages
- Performance optimization notes

### Additions to Phase 5:
- Version archival job (not just compression)
- Compression format specification (gzip)

## Summary

Integrating 9 critical items, 4 high-priority risk mitigations.
Deferring 4 suggestions that add significant scope or can be post-MVP.
Estimated timeline adjustment acknowledged: +1-2 weeks.
