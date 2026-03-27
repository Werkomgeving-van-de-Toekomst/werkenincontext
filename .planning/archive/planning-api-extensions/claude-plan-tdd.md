# TDD Plan: Agent Orchestration API Extensions

This document defines the tests to write BEFORE implementing each section of the plan.

**Testing Approach (from codebase analysis):**
- Uses `#[tokio::test]` for async tests
- Minimal mocking - real implementations with test fixtures
- In-memory DuckDB (`:memory:`) for database tests
- Tests located in `tests/` directory alongside code

---

## Section 1: Foundation & Configuration

### Tests for S3 Client Module (`crates/iou-core/src/storage/s3.rs`)

**Test: S3Client::new_from_env creates client with valid credentials**
- Given: Required environment variables are set
- When: S3Client::new_from_env() is called
- Then: Returns Ok(S3Client)

**Test: S3Client::new_from_env fails with missing credentials**
- Given: S3_ACCESS_KEY is not set
- When: S3Client::new_from_env() is called
- Then: Returns Err(S3Error)

**Test: S3Client::validate succeeds with valid connection**
- Given: S3Client with valid MinIO/S3 endpoint
- When: validate() is called
- Then: Returns Ok(())

**Test: S3Client::validate fails with invalid endpoint**
- Given: S3Client with invalid endpoint
- When: validate() is called
- Then: Returns Err(S3Error)

### Tests for WebSocket Module Structure

**Test: ConnectionState limiting allows up to 10 connections**
- Given: ConnectionLimiter for a document
- When: 10 permits are acquired
- Then: All acquisitions succeed

**Test: ConnectionState limiting rejects 11th connection**
- Given: ConnectionLimiter with 10 active connections
- When: 11th permit acquisition is attempted
- Then: Returns ApiError::TooManyRequests

### Tests for Configuration Loading

**Test: S3 config loads from environment**
- Given: S3_* environment variables are set
- When: Config is loaded
- Then: S3 section contains correct values

**Test: JWT secret loads from environment**
- Given: JWT_SECRET environment variable is set
- When: Config is loaded
- Then: JWT secret is not the default hardcoded value

---

## Section 2: Orchestrator Integration

### Tests for Orchestrator Wrapper (`crates/iou-api/src/orchestrator/wrapper.rs`)

**Test: WorkflowOrchestrator starts workflow from initial state**
- Given: New document with valid template
- When: start_workflow() is called
- Then: WorkflowStateMachine is in Running state
- And: Status broadcast is sent

**Test: WorkflowOrchestrator transitions states correctly**
- Given: Workflow in Research state
- When: Research agent completes
- Then: State transitions to Content
- And: Status broadcast is sent

**Test: WorkflowOrchestrator applies overall timeout**
- Given: Workflow running longer than 8 minutes
- When: Cumulative time exceeds timeout
- Then: Workflow is cancelled
- And: Failed status is broadcast

**Test: WorkflowOrchestrator handles agent failure**
- Given: Agent returns error
- When: Error is not transient
- Then: Workflow enters Failed state
- And: Error is broadcast

**Test: WorkflowOrchestrator retries transient errors**
- Given: Agent returns transient error
- When: Retry limit not exceeded
- Then: Agent is retried with exponential backoff

### Tests for Idempotency

**Test: create_document returns existing document for duplicate id**
- Given: Document with ID already exists
- When: create_document() is called with same ID
- Then: Returns existing document without creating new workflow
- And: Returns 200 OK with existing document data

**Test: create_document creates new document for unique id**
- Given: Document ID does not exist
- When: create_document() is called
- Then: Creates new document
- And: Returns 201 Created

### Tests for State Mapping

**Test: WorkflowState maps correctly to DocumentState**
- Given: WorkflowStateMachine in Completed state
- When: State is queried
- Then: Returns DocumentState::Approved

**Test: All WorkflowState variants have DocumentState mapping**
- Given: Each WorkflowState variant
- When: Mapped through conversion function
- Then: Returns valid DocumentState

---

## Section 3: Authentication Integration

### Tests for Role-Based Access Control

**Test: create_document with ObjectCreator role succeeds**
- Given: AuthContext with ObjectCreator role
- When: create_document() is called
- Then: Request proceeds to handler
- And: Returns 201 Created

**Test: create_document with DomainEditor role succeeds**
- Given: AuthContext with DomainEditor role
- When: create_document() is called
- Then: Request proceeds to handler
- And: Returns 201 Created

**Test: create_document with DomainViewer role fails**
- Given: AuthContext with DomainViewer role only
- When: create_document() is called
- Then: Returns 403 Forbidden
- And: Error message indicates insufficient permissions

**Test: approve_document with ObjectApprover role succeeds**
- Given: AuthContext with ObjectApprover role
- When: approve_document() is called
- Then: Request proceeds to handler

**Test: approve_document without ObjectApprover role fails**
- Given: AuthContext without ObjectApprover role
- When: approve_document() is called
- Then: Returns 403 Forbidden

### Tests for Organization-Based Access Control

**Test: create_document validates organization match**
- Given: AuthContext with org_id = A
- And: Request with org_id = B
- When: create_document() is called
- Then: Returns 403 Forbidden
- And: Error message indicates organization mismatch

**Test: document_query filters by organization**
- Given: AuthContext with org_id = A
- And: Documents exist for org A and org B
- When: list_documents() is called
- Then: Only documents from org A are returned

**Test: audit_log includes user_id**
- Given: AuthContext with user_id
- When: Document action is performed
- Then: Audit entry contains user_id
- And: Audit entry contains timestamp

### Tests for JWT Configuration

**Test: JWT secret from environment is used**
- Given: JWT_SECRET environment variable set
- When: JWT token is validated
- Then: Uses environment secret, not hardcoded value

**Test: Invalid JWT token is rejected**
- Given: Malformed JWT token
- When: Request is made with invalid token
- Then: Returns 401 Unauthorized

---

## Section 4: S3 Storage Integration

### Tests for S3 Upload

**Test: upload succeeds for document under 10MB**
- Given: Document of 5MB
- When: upload() is called
- Then: Returns Ok(())
- And: Document is accessible in S3

**Test: upload fails for document over 10MB**
- Given: Document of 11MB
- When: upload() is called
- Then: Returns Err(ApiError::PayloadTooLarge)

**Test: upload retry on transient S3 error**
- Given: S3 returns 503 Service Unavailable
- When: upload() is called with retry logic
- Then: Retries with exponential backoff
- And: Eventually succeeds on retry

**Test: upload propagates permanent S3 errors**
- Given: S3 returns 403 Forbidden
- When: upload() is called
- Then: Returns Err immediately without retry

### Tests for S3 Download (Streaming)

**Test: download_stream streams without buffering entire file**
- Given: 10MB document in S3
- When: download_stream() is called
- Then: Returns AsyncRead implementation
- And: Memory usage remains constant (not O(10MB))

**Test: download_stream returns 404 for missing document**
- Given: Document key does not exist in S3
- When: download_stream() is called
- Then: Returns Err(ApiError::NotFound)

**Test: download proxy endpoint streams response**
- Given: Valid document_id in database
- When: GET /api/documents/{id}/download is called
- Then: Returns 200 OK with streaming response
- And: Content-Type header matches document type

### Tests for Error Conversion

**Test: S3Error::HttpFailWithBody(404) maps to ApiError::NotFound**
- Given: S3 returns 404
- When: Error is converted
- Then: Returns ApiError::NotFound

**Test: S3Error::HttpFailWithBody(413) maps to ApiError::PayloadTooLarge**
- Given: S3 returns 413
- When: Error is converted
- Then: Returns ApiError::PayloadTooLarge

**Test: Generic S3Error maps to ApiError::Internal**
- Given: Unknown S3 error
- When: Error is converted
- Then: Returns ApiError::Internal

---

## Section 5: WebSocket Support

### Tests for WebSocket Handler

**Test: WebSocket upgrade succeeds**
- Given: Valid WebSocket upgrade request
- When: ws_document_handler is called
- Then: Connection is upgraded
- And: Returns 101 Switching Protocols

**Test: WebSocket receives status broadcasts**
- Given: Connected WebSocket for document_id
- When: Status broadcast is sent for that document_id
- Then: WebSocket receives JSON message
- And: Message contains correct document_id

**Test: WebSocket ignores other document broadcasts**
- Given: Connected WebSocket for document_id = A
- When: Status broadcast is sent for document_id = B
- Then: WebSocket does not receive message

### Tests for Connection Limits

**Test: 10th WebSocket connection is accepted**
- Given: 9 existing connections for document_id
- When: 10th connection attempt is made
- Then: Connection succeeds

**Test: 11th WebSocket connection is rejected**
- Given: 10 existing connections for document_id
- When: 11th connection attempt is made
- Then: Connection is closed
- And: Returns error indicating limit reached

### Tests for Bounded Channel

**Test: Lagging receiver receives Lagged error**
- Given: Bounded channel with capacity 100
- And: Receiver has not consumed messages
- When: 101st message is sent
- Then: Slow receiver gets Lagged error
- And: Receiver can resync with latest messages

### Tests for Idle Timeout

**Test: WebSocket closes after 5 minutes of inactivity**
- Given: Connected WebSocket
- And: No messages received for 5 minutes
- When: Idle timeout check runs
- Then: WebSocket is closed
- And: Close frame is sent

**Test: Heartbeat keeps connection alive**
- Given: Connected WebSocket
- And: Ping/Pong exchanged every 30 seconds
- When: 5 minutes pass
- Then: Connection remains open

### Tests for Panic Recovery

**Test: Send task panic does not crash connection handler**
- Given: WebSocket connection
- When: Send task panics
- Then: Receive task is aborted
- And: Resources are cleaned up

**Test: Receive task panic does not crash connection handler**
- Given: WebSocket connection
- When: Receive task panics
- Then: Send task is aborted
- And: Resources are cleaned up

### Tests for Graceful Disconnect

**Test: WebSocket closes gracefully on completion**
- Given: Connected WebSocket for document
- When: Document workflow completes
- Then: Completion status is sent
- And: WebSocket closes gracefully

**Test: WebSocket closes on client close frame**
- Given: Connected WebSocket
- When: Client sends Close frame
- Then: Server acknowledges close
- And: Connection terminates

---

## Section 6: Integration & Testing

### End-to-End Tests

**Test: Document creation → async pipeline → completion → WebSocket updates**
- Given: Authenticated user with ObjectCreator role
- When: POST /api/documents/create is called
- Then: Returns 201 Created
- And: Workflow starts asynchronously
- And: WebSocket clients receive Started, Progress, Completed messages
- And: Document is stored in S3 on completion

**Test: Unauthorized user cannot create document**
- Given: User without required role
- When: POST /api/documents/create is called
- Then: Returns 403 Forbidden
- And: No workflow is started

**Test: User cannot access documents from other organization**
- Given: User from organization A
- And: Document from organization B
- When: GET /api/documents/{id} is called
- Then: Returns 403 Forbidden or 404 Not Found

**Test: Oversized document is rejected**
- Given: Document payload exceeding 10MB
- When: POST /api/documents/create is called
- Then: Returns 413 Payload Too Large
- And: No upload to S3 is attempted

**Test: Workflow timeout propagates correctly**
- Given: Workflow that exceeds 8 minute timeout
- When: Timeout elapses
- Then: Workflow is cancelled
- And: WebSocket clients receive Failed status
- And: Database reflects failed state

### Concurrency Tests

**Test: Multiple concurrent document creations**
- Given: 10 simultaneous create requests
- When: All requests are processed
- Then: All documents are created
- And: All workflows execute independently

**Test: Concurrent WebSocket connections receive correct updates**
- Given: 5 WebSocket connections for document A
- And: 3 connections for document B
- When: Both documents have status updates
- Then: Only connections for respective documents receive updates

### Performance Tests

**Test: S3 streaming download does not buffer large file**
- Given: 10MB document in S3
- When: Download endpoint streams response
- Then: Memory usage during download is < 1MB
- And: Download completes in reasonable time

**Test: Broadcast channel performance under load**
- Given: 100 status updates per second
- And: 10 WebSocket subscribers
- When: Load test runs for 1 minute
- Then: No message loss for non-lagging receivers
- And: Lagging receivers receive Lagged errors

---

## Mock Infrastructure

### Test Helpers

**File: tests/mocks/s3.rs**
- MockS3Client implementing S3 operations
- In-memory storage for testing
- Configurable failure modes

**File: tests/mocks/orchestrator.rs**
- MockWorkflowStateMachine for testing
- Controllable state transitions
- Configurable delays and failures

**File: tests/helpers/websocket.rs**
- WebSocket test client
- Message capture and verification
- Connection lifecycle helpers
