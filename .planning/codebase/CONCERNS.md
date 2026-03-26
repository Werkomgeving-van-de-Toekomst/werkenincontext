# Codebase Concerns

**Analysis Date:** 2026-03-08

## Tech Debt

### Authentication Implementation
- **Issue**: Password verification is not implemented - accepts any login
- **Files**: `[crates/iou-api/src/middleware/auth.rs:434]`
- **Impact**: No actual security verification, system vulnerable to unauthorized access
- **Fix approach**: Implement bcrypt password hashing and verification

### Storage Implementation
- **Issue**: Document storage uses placeholder implementations
- **Files**: `[crates/iou-ai/src/agents/compliance.rs:847,873]`
- **Impact**: Documents not actually stored, no persistence or audit trail
- **Fix approach**: Implement S3 integration with proper RBAC and audit logging

### API Keys Configuration
- **Issue**: Hardcoded API keys in fallback code
- **Files**: `[crates/iou-frontend/src/components/map_3d.rs:257,276]`
- **Impact**: Security risk if fallback keys are accidentally used
- **Fix approach**: Remove hardcoded keys, enforce environment variable requirement

### Database Methods
- **Issue**: Missing `list_templates` database method
- **Files**: `[crates/iou-api/src/routes/apps.rs]`
- **Impact**: API endpoint may fail when called
- **Fix approach**: Implement missing database method

## Known Bugs

### JavaScript Return Values
- **Issue**: JavaScript code in Rust webview returns null in some cases
- **Files**: `[crates/iou-frontend/src/pages/flevoland/mod.rs:956]`
- **Symptoms**: File upload functionality may fail silently
- **Trigger**: When no file is selected in file input
- **Workaround**: Check for null returns in calling code

## Security Considerations

### Authentication Bypass
- **Risk**: Login accepts any email/password combination
- **Files**: `[crates/iou-api/src/middleware/auth.rs:430-477]`
- **Current mitigation**: JWT tokens still require proper secret
- **Recommendations**: Implement bcrypt verification and rate limiting

### Audit Logging
- **Risk**: Important events not logged to database
- **Files**: `[crates/iou-api/src/middleware/auth.rs:541]`
- **Current mitigation**: Basic logging exists but not persistent
- **Recommendations**: Implement proper database audit trail

### Process Execution
- **Risk**: Multiple shell command executions without input sanitization
- **Files**: Various files in crates/iou-frontend/src/components/ and crates/iou-ai/src/
- **Current mitigation**: Appears to be safe usage but worth reviewing
- **Recommendations**: Add input validation for all command executions

## Performance Bottlenecks

### Large Flevoland Module
- **Problem**: 2471 lines in single module
- **Files**: `[crates/iou-frontend/src/pages/flevoland/mod.rs]`
- **Cause**: All province-specific logic in one file
- **Improvement path**: Split into sub-modules by functionality

### Database Operations
- **Problem**: Large database file (1501 lines)
- **Files**: `[crates/iou-api/src/db.rs]`
- **Cause**: All database logic in single file
- **Improvement path**: Split into separate modules for different entities

## Fragile Areas

### Map 3D Component
- **Files**: `[crates/iou-frontend/src/components/map_3d.rs]`
- **Why fragile**: Heavy use of unwrap() calls, 1724 lines of complex logic
- **Safe modification**: Add proper error handling, break into smaller components
- **Test coverage**: Has some tests but edge cases may not be covered

### Compliance Agent
- **Files**: `[crates/iou-ai/src/agents/compliance.rs]`
- **Why fragile**: Complex business logic with many TODO items
- **Safe modification**: Implement TODO items first, add comprehensive tests
- **Test coverage**: Limited test coverage for complex compliance rules

## Scaling Limits

### Document Processing
- **Current capacity**: Limited by sequential processing
- **Limit**: Large document batches will timeout
- **Scaling path**: Implement parallel processing queues

### Memory Usage
- **Current capacity**: Unknown (no monitoring)
- **Limit**: Large documents or many concurrent requests
- **Scaling path**: Add memory monitoring and streaming for large files

## Dependencies at Risk

### AI/ML Dependencies
- **Risk**: ONNX Runtime commented out, may affect AI features
- **Impact**: Advanced AI processing disabled
- **Migration plan**: Evaluate if needed or find alternatives

## Missing Critical Features

### PROVISA Integration
- **Problem**: Compliance guidelines not integrated
- **Files**: `[crates/iou-ai/src/agents/research.rs:152,181]`
- **Blocks**: Full compliance automation
- **Priority**: High - compliance is core functionality

### Semantic Search
- **Problem**: Similarity scores are hardcoded
- **Files**: `[crates/iou-ai/src/agents/research.rs:208,221,299]`
- **Blocks**: Accurate document similarity
- **Priority**: Medium - affects research quality

### Document Persistence
- **Problem**: Documents processed but not stored
- **Files**: Multiple placeholder implementations
- **Blocks**: Document history and retrieval
- **Priority**: High - fundamental feature missing

## Test Coverage Gaps

### AI Agent Testing
- **What's not tested**: Complex agent workflows
- **Files**: `[crates/iou-ai/src/agents/]`
- **Risk**: Logic errors in compliance and review processes
- **Priority**: High - business logic critical

### Error Handling
- **What's not tested**: Edge cases and error scenarios
- **Files**: Throughout codebase
- **Risk**: System may fail unexpectedly
- **Priority**: Medium - stability concerns

---

*Concerns audit: 2026-03-08*