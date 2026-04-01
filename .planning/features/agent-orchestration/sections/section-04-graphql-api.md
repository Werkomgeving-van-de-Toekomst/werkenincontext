Now I have all the context. Let me generate the section content for `section-04-graphql-api`.

# Section 04: GraphQL API

## Overview

This section implements the GraphQL API for the agent orchestration system. The GraphQL API provides the primary interface for creating workflows, managing approval requests, and querying workflow status. It integrates with JWT authentication for authorization and uses DataLoader to prevent N+1 query problems.

**Dependencies:** This section depends on `section-03-checkpoint-recovery` being completed first.

**Files to create/modify:**
- `/server/crates/iou-orchestrator/src/graphql/mod.rs` - Module exports
- `/server/crates/iou-orchestrator/src/graphql/schema.rs` - GraphQL schema definitions
- `/server/crates/iou-orchestrator/src/graphql/mutations.rs` - Mutation resolvers
- `/server/crates/iou-orchestrator/src/graphql/queries.rs` - Query resolvers
- `/server/crates/iou-orchestrator/src/graphql/subscriptions.rs` - Subscription resolvers
- `/server/crates/iou-orchestrator/src/graphql/context.rs` - Request context with auth
- `/server/crates/iou-orchestrator/src/graphql/dataloader.rs` - DataLoader for batch loading
- `/server/crates/iou-orchestrator/Cargo.toml` - Add dependencies

---

## Tests

### Authentication Tests

**Test: approve mutation fails without authentication**
- Send GraphQL approve mutation without auth header
- Assert returns UNAUTHORIZED error
- Assert approval not applied in database

**Test: approve mutation gets approverId from JWT context**
- Send GraphQL approve mutation with valid JWT
- Assert approval record has approver_id matching JWT subject
- Assert workflow advanced to next state

**Test: modify input does not include approverId (from auth context)**
- Send GraphQL modify mutation
- Assert approverId extracted from JWT, not input
- Assert modification attributed to authenticated user

### Authorization Tests

**Test: approve mutation validates user can approve this workflow**
- Create approval request requiring "supervisor" role
- Send approve mutation with JWT for "user" role
- Assert returns FORBIDDEN error
- Assert approval not applied

**Test: workflow query returns pending approvals for user's role**
- Create workflow with pending approval
- Send workflow query with user JWT
- Assert response includes approval requests user can approve
- Assert excludes requests for other roles

### Mutation Tests

**Test: createWorkflow mutation creates workflow in database**
- Send GraphQL createWorkflow mutation with valid input
- Query workflows table
- Assert row created with matching document_type and priority

### Subscription Tests

**Test: Subscription sends workflow updates to subscribed clients**
- Create GraphQL subscription for workflow ID
- Trigger workflow state change
- Assert subscription receives update event
- Assert event contains new state

---

## Implementation

### 1. Add Dependencies

**File:** `/server/crates/iou-orchestrator/Cargo.toml`

Add the following dependencies to the `[dependencies]` section:

```toml
async-graphql = { version = "7", features = ["tokio", "uuid", "chrono"] }
async-graphql-axum = "7"
```

Note: `uuid` and `chrono` should already be available from existing dependencies.

### 2. GraphQL Context with Authentication

**File:** `/server/crates/iou-orchestrator/src/graphql/context.rs`

The request context holds shared state and authenticated user information extracted from JWT.

```rust
use async_graphql::{Context, Result, Error};
use std::sync::Arc;
use uuid::Uuid;
use crate::state::AppState;

/// Authenticated user context extracted from JWT
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub roles: Vec<String>,
    pub name: String,
}

/// GraphQL request context
pub struct RequestContext {
    pub state: Arc<AppState>,
    pub auth: Option<AuthContext>,
}

impl RequestContext {
    pub fn new(state: Arc<AppState>, auth: Option<AuthContext>) -> Self {
        Self { state, auth }
    }

    /// Get authenticated user or return error
    pub fn require_auth(&self) -> Result<&AuthContext> {
        self.auth.as_ref()
            .ok_or_else(|| Error::new("Authentication required"))
    }

    /// Check if user has required role
    pub fn require_role(&self, role: &str) -> Result<&AuthContext> {
        let auth = self.require_auth()?;
        if auth.roles.contains(&role.to_string()) {
            Ok(auth)
        } else {
            Err(Error::new(format!("Role '{}' required", role)))
        }
    }
}
```

### 3. GraphQL Schema Types

**File:** `/server/crates/iou-orchestrator/src/graphql/schema.rs`

Define the GraphQL types that map to domain models.

```rust
use async_graphql::{SimpleObject, Enum, InputObject, ID, Json, OneofObject};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::state_machine::WorkflowState;
use crate::scheduler::Priority;
use crate::executor::AgentType;

#[derive(SimpleObject, Clone, Debug)]
pub struct Workflow {
    pub id: ID,
    pub status: WorkflowStatus,
    pub priority: Priority,
    pub current_agent: Option<AgentType>,
    #[graphql(deprecation = "Use pendingApprovals connection")]
    pub pending_approvals: Vec<ApprovalRequest>,
    pub completed_agents: Vec<AgentType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ApprovalRequest {
    pub id: ID,
    pub workflow_id: ID,
    pub agent: AgentType,
    pub result: AgentResult,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub escalated: bool,
    pub approver: Option<String>,
    pub required_role: String,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct AgentResult {
    pub agent: AgentType,
    pub status: String,
    pub output: Option<String>,
    pub metadata: Json,
}

#[derive(Enum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowStatus {
    Created,
    Running,
    AwaitingApproval,
    AwaitingEscalation,
    Completed,
    Failed,
    Cancelled,
    Retrying,
    Archived,
    ParallelExecuting,
    PartialComplete,
    MergeResults,
}

impl From<WorkflowState> for WorkflowStatus {
    fn from(state: WorkflowState) -> Self {
        match state {
            WorkflowState::Created => WorkflowStatus::Created,
            WorkflowState::Running => WorkflowStatus::Running,
            WorkflowState::AwaitingApproval => WorkflowStatus::AwaitingApproval,
            WorkflowState::AwaitingEscalation => WorkflowStatus::AwaitingEscalation,
            WorkflowState::Completed => WorkflowStatus::Completed,
            WorkflowState::Failed => WorkflowStatus::Failed,
            WorkflowState::Cancelled => WorkflowStatus::Cancelled,
            WorkflowState::Retrying => WorkflowStatus::Retrying,
            WorkflowState::Archived => WorkflowStatus::Archived,
            WorkflowState::ParallelExecuting => WorkflowStatus::ParallelExecuting,
            WorkflowState::PartialComplete => WorkflowStatus::PartialComplete,
            WorkflowState::MergeResults => WorkflowStatus::MergeResults,
        }
    }
}

#[derive(Enum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentType {
    Research,
    Content,
    Compliance,
    Review,
}

#[derive(InputObject)]
pub struct CreateWorkflowInput {
    pub document_type: String,
    #[graphql(default)]
    pub priority: Priority,
    #[graphql(default)]
    pub metadata: Option<Json>,
}

#[derive(InputObject)]
pub struct ApprovalInput {
    pub request_id: ID,
    #[graphql(default)]
    pub comment: Option<String>,
}

#[derive(InputObject)]
pub struct ModificationInput {
    pub request_id: ID,
    pub modifications: Vec<ModificationFieldInput>,
    #[graphql(default)]
    pub comment: Option<String>,
}

#[derive(InputObject)]
pub struct ModificationFieldInput {
    pub path: String,
    pub new_value: Json,
}

#[derive(InputObject)]
pub struct RejectInput {
    pub request_id: ID,
    pub reason: String,
}

#[derive(InputObject)]
pub struct WorkflowFilter {
    pub status: Option<WorkflowStatus>,
    pub priority: Option<Priority>,
    pub created_by: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}
```

### 4. Mutation Resolvers

**File:** `/server/crates/iou-orchestrator/src/graphql/mutations.rs`

Implement mutations for workflow creation and approval management.

```rust
use async_graphql::{Result, Context, Error, ErrorExtensions};
use uuid::Uuid;

use super::schema::{
    CreateWorkflowInput, ApprovalInput, ModificationInput, RejectInput, Workflow
};
use super::context::RequestContext;
use crate::state_machine::WorkflowStateMachine;
use crate::event_bus::WorkflowCommand;

pub struct Mutation;

impl Mutation {
    /// Create a new workflow
    pub async fn create_workflow(
        ctx: &Context<'_>,
        input: CreateWorkflowInput,
    ) -> Result<Workflow> {
        let req_ctx = ctx.data::<RequestContext>()?;
        
        let workflow_id = Uuid::new_v4();
        
        // Create workflow in state machine
        let mut state_machine = WorkflowStateMachine::new(workflow_id);
        state_machine.start()?;
        
        // Persist to database via state
        req_ctx.state.workflow_store.create(
            workflow_id,
            input.document_type,
            input.priority,
            input.metadata.unwrap_or_default().0,
        ).await?;
        
        // Send start command via event bus
        req_ctx.state.event_bus.send_command(WorkflowCommand::Start {
            workflow_id,
        }).await?;
        
        // Fetch and return created workflow
        let workflow = req_ctx.state.workflow_store
            .get(workflow_id)
            .await?
            .ok_or_else(|| Error::new("Failed to create workflow"))?;
        
        Ok(workflow.into_graphql())
    }

    /// Approve an approval request
    /// 
    /// The approverId is extracted from the JWT context, not from input.
    /// User must have the required role for this approval.
    pub async fn approve(
        ctx: &Context<'_>,
        input: ApprovalInput,
    ) -> Result<Workflow> {
        let req_ctx = ctx.data::<RequestContext>()?;
        let auth = req_ctx.require_auth()?;
        
        let request_id: Uuid = input.request_id.try_into()?;
        
        // Load approval request
        let request = req_ctx.state.approval_store
            .get(request_id)
            .await?
            .ok_or_else(|| Error::new("Approval request not found"))?;
        
        // Verify user has required role
        if !auth.roles.contains(&request.required_role) {
            return Err(Error::new("Insufficient permissions")
                .extend_with(|_, e| e.set("code", "FORBIDDEN")));
        }
        
        // Check if request is still pending
        if !request.is_pending() {
            return Err(Error::new("Approval request is not pending"));
        }
        
        // Record approval decision
        req_ctx.state.approval_store
            .record_decision(request_id, auth.user_id, true, input.comment)
            .await?;
        
        // Send approval command via event bus
        req_ctx.state.event_bus.send_command(WorkflowCommand::Approve {
            workflow_id: request.workflow_id,
            request_id,
            approver_id: auth.user_id,
            comment: input.comment,
        }).await?;
        
        // Return updated workflow
        let workflow = req_ctx.state.workflow_store
            .get(request.workflow_id)
            .await?
            .ok_or_else(|| Error::new("Workflow not found"))?;
        
        Ok(workflow.into_graphql())
    }

    /// Approve with modifications to agent output
    pub async fn modify(
        ctx: &Context<'_>,
        input: ModificationInput,
    ) -> Result<Workflow> {
        let req_ctx = ctx.data::<RequestContext>()?;
        let auth = req_ctx.require_auth()?;
        
        let request_id: Uuid = input.request_id.try_into()?;
        
        // Load approval request
        let request = req_ctx.state.approval_store
            .get(request_id)
            .await?
            .ok_or_else(|| Error::new("Approval request not found"))?;
        
        // Verify user has required role
        if !auth.roles.contains(&request.required_role) {
            return Err(Error::new("Insufficient permissions")
                .extend_with(|_, e| e.set("code", "FORBIDDEN")));
        }
        
        // Apply modifications to agent output
        let modified_output = req_ctx.state.approval_store
            .apply_modifications(request_id, &input.modifications, auth.user_id)
            .await?;
        
        // Record approval with modifications
        req_ctx.state.approval_store
            .record_decision_with_modifications(
                request_id,
                auth.user_id,
                true,
                modified_output,
                input.comment,
            ).await?;
        
        // Send approve command with modified output
        req_ctx.state.event_bus.send_command(WorkflowCommand::ApproveWithModifications {
            workflow_id: request.workflow_id,
            request_id,
            approver_id: auth.user_id,
            modified_output,
            comment: input.comment,
        }).await?;
        
        // Return updated workflow
        let workflow = req_ctx.state.workflow_store
            .get(request.workflow_id)
            .await?
            .ok_or_else(|| Error::new("Workflow not found"))?;
        
        Ok(workflow.into_graphql())
    }

    /// Reject an approval request
    pub async fn reject(
        ctx: &Context<'_>,
        input: RejectInput,
    ) -> Result<Workflow> {
        let req_ctx = ctx.data::<RequestContext>()?;
        let auth = req_ctx.require_auth()?;
        
        let request_id: Uuid = input.request_id.try_into()?;
        
        // Load approval request
        let request = req_ctx.state.approval_store
            .get(request_id)
            .await?
            .ok_or_else(|| Error::new("Approval request not found"))?;
        
        // Verify user has required role
        if !auth.roles.contains(&request.required_role) {
            return Err(Error::new("Insufficient permissions")
                .extend_with(|_, e| e.set("code", "FORBIDDEN")));
        }
        
        // Record rejection
        req_ctx.state.approval_store
            .record_decision(request_id, auth.user_id, false, Some(input.reason))
            .await?;
        
        // Send reject command
        req_ctx.state.event_bus.send_command(WorkflowCommand::Reject {
            workflow_id: request.workflow_id,
            request_id,
            approver_id: auth.user_id,
            reason: input.reason,
        }).await?;
        
        // Return updated workflow
        let workflow = req_ctx.state.workflow_store
            .get(request.workflow_id)
            .await?
            .ok_or_else(|| Error::new("Workflow not found"))?;
        
        Ok(workflow.into_graphql())
    }

    /// Cancel a workflow
    pub async fn cancel_workflow(
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Workflow> {
        let req_ctx = ctx.data::<RequestContext>()?;
        let workflow_id: Uuid = id.try_into()?;
        
        // Send cancel command
        req_ctx.state.event_bus.send_command(WorkflowCommand::Cancel {
            workflow_id,
        }).await?;
        
        // Return updated workflow
        let workflow = req_ctx.state.workflow_store
            .get(workflow_id)
            .await?
            .ok_or_else(|| Error::new("Workflow not found"))?;
        
        Ok(workflow.into_graphql())
    }
}
```

### 5. Query Resolvers

**File:** `/server/crates/iou-orchestrator/src/graphql/queries.rs`

Implement queries for retrieving workflow and approval information.

```rust
use async_graphql::{Result, Context, Error};
use uuid::Uuid;

use super::schema::{Workflow, ApprovalRequest, WorkflowFilter};
use super::context::RequestContext;

pub struct Query;

impl Query {
    /// Get a single workflow by ID
    pub async fn workflow(
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<Workflow>> {
        let req_ctx = ctx.data::<RequestContext>()?;
        let workflow_id: Uuid = id.try_into()?;
        
        let workflow = req_ctx.state.workflow_store
            .get(workflow_id)
            .await?;
        
        Ok(workflow.map(|w| w.into_graphql()))
    }

    /// List workflows with optional filtering
    pub async fn workflows(
        ctx: &Context<'_>,
        filter: Option<WorkflowFilter>,
    ) -> Result<Vec<Workflow>> {
        let req_ctx = ctx.data::<RequestContext>()?;
        
        let workflows = if let Some(f) = filter {
            req_ctx.state.workflow_store
                .query(f.into())
                .await?
        } else {
            req_ctx.state.workflow_store
                .list_all()
                .await?
        };
        
        Ok(workflows.into_iter().map(|w| w.into_graphql()).collect())
    }

    /// Get pending approvals for the authenticated user
    /// 
    /// Only returns approvals the user is authorized to see based on their roles.
    pub async fn pending_approvals(
        ctx: &Context<'_>,
    ) -> Result<Vec<ApprovalRequest>> {
        let req_ctx = ctx.data::<RequestContext>()?;
        let auth = req_ctx.require_auth()?;
        
        // Get approvals that user's roles can approve
        let approvals = req_ctx.state.approval_store
            .get_pending_for_roles(&auth.roles)
            .await?;
        
        Ok(approvals.into_iter().map(|a| a.into_graphql()).collect())
    }
}
```

### 6. Subscription Resolvers

**File:** `/server/crates/iou-orchestrator/src/graphql/subscriptions.rs`

Implement real-time subscriptions for workflow events.

```rust
use async_graphql::{Result, Context, Subscription, ID, stream};
use futures_util::Stream;
use std::pin::Pin;

use super::schema::{Workflow, ApprovalRequest, AgentResult};
use super::context::RequestContext;
use crate::event_bus::{OrchestratorEvent, EventBusSubscriber};

pub struct Subscription;

impl Subscription {
    /// Subscribe to workflow updates
    pub async fn workflow_updated(
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<impl Stream<Item = Workflow>> {
        let req_ctx = ctx.data::<RequestContext>()?;
        let workflow_id: uuid::Uuid = id.try_into()?;
        
        let mut subscriber = req_ctx.state.event_bus.subscribe();
        
        let stream = stream::channel(move |mut tx| async move {
            while let Ok(event) = subscriber.recv().await {
                match event {
                    OrchestratorEvent::WorkflowStarted { id: wf_id }
                    | OrchestratorEvent::WorkflowCompleted { id: wf_id }
                    | OrchestratorEvent::WorkflowFailed { id: wf_id, .. }
                        if wf_id == workflow_id =>
                    {
                        if let Ok(Some(workflow)) = req_ctx.state.workflow_store
                            .get(workflow_id).await
                        {
                            let _ = tx.send(workflow.into_graphql()).await;
                        }
                    }
                    _ => {}
                }
            }
        });
        
        Ok(stream)
    }

    /// Subscribe to approval required events
    pub async fn approval_required(
        ctx: &Context<'_>,
    ) -> Result<impl Stream<Item = ApprovalRequest>> {
        let req_ctx = ctx.data::<RequestContext>()?;
        let auth = req_ctx.require_auth()?;
        
        let mut subscriber = req_ctx.state.event_bus.subscribe();
        
        let stream = stream::channel(move |mut tx| async move {
            while let Ok(event) = subscriber.recv().await {
                match event {
                    OrchestratorEvent::ApprovalRequired { workflow_id, request } => {
                        // Only send if user has required role
                        if auth.roles.contains(&request.required_role) {
                            let _ = tx.send(request.into_graphql()).await;
                        }
                    }
                    _ => {}
                }
            }
        });
        
        Ok(stream)
    }

    /// Subscribe to agent completion events
    pub async fn agent_completed(
        ctx: &Context<'_>,
        workflow_id: ID,
    ) -> Result<impl Stream<Item = AgentResult>> {
        let req_ctx = ctx.data::<RequestContext>()?;
        let wf_id: uuid::Uuid = workflow_id.try_into()?;
        
        let mut subscriber = req_ctx.state.event_bus.subscribe();
        
        let stream = stream::channel(move |mut tx| async move {
            while let Ok(event) = subscriber.recv().await {
                match event {
                    OrchestratorEvent::AgentCompleted { workflow_id, agent, result } 
                        if workflow_id == wf_id =>
                    {
                        let _ = tx.send(AgentResult {
                            agent,
                            status: result.status.to_string(),
                            output: Some(result.output),
                            metadata: async_graphql::Json(result.metadata),
                        }).await;
                    }
                    _ => {}
                }
            }
        });
        
        Ok(stream)
    }
}
```

### 7. DataLoader for Batch Loading

**File:** `/server/crates/iou-orchestrator/src/graphql/dataloader.rs`

Prevent N+1 queries by batching related data loads.

```rust
use async_graphql::{dataloader::Loader, Result, Error};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use crate::state::AppState;

/// DataLoader for batch loading workflows
pub struct WorkflowLoader {
    state: Arc<AppState>,
}

impl WorkflowLoader {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl Loader<Uuid> for WorkflowLoader {
    type Value = crate::graphql::schema::Workflow;
    type Error = Error;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        // Batch fetch all workflows in a single query
        let workflows = self.state.workflow_store
            .get_batch(keys)
            .await?;
        
        Ok(workflows.into_iter()
            .map(|w| (w.id, w.into_graphql()))
            .collect())
    }
}

/// DataLoader for batch loading approval requests
pub struct ApprovalLoader {
    state: Arc<AppState>,
}

impl ApprovalLoader {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl Loader<Uuid> for ApprovalLoader {
    type Value = crate::graphql::schema::ApprovalRequest;
    type Error = Error;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let approvals = self.state.approval_store
            .get_batch(keys)
            .await?;
        
        Ok(approvals.into_iter()
            .map(|a| (a.id, a.into_graphql()))
            .collect())
    }
}
```

### 8. Module Exports

**File:** `/server/crates/iou-orchestrator/src/graphql/mod.rs`

```rust
use async_graphql::{Schema, EmptySubscription, MergedObject, DefaultMergedObject};
use async_graphql_axum::{GraphQLSubscription, GraphQLRequest};
use axum::{
    response::{IntoResponse, Response},
    Json,
};

pub mod context;
pub mod schema;
pub mod mutations;
pub mod queries;
pub mod subscriptions;
pub mod dataloader;

use context::RequestContext;

/// Build the GraphQL schema
pub fn build_schema() -> Schema<
    queries::Query,
    mutations::Mutation,
    subscriptions::Subscription,
> {
    Schema::build(
        queries::Query,
        mutations::Mutation,
        subscriptions::Subscription,
    )
    .finish()
}

/// GraphQL handler for HTTP requests
pub async fn graphql_handler(
    schema: async_graphql::Extension<Schema<
        queries::Query,
        mutations::Mutation,
        subscriptions::Subscription,
    >>,
    req: GraphQLRequest,
) -> Response {
    schema.execute(req.into_inner()).await.into_response()
}

/// GraphQL WebSocket handler
pub fn graphql_subscription_handler() -> GraphQLSubscription {
    GraphQLSubscription::new()
}
```

### 9. Axum Integration

**File:** `/server/crates/iou-orchestrator/src/http.rs` (or main server file)

Add the GraphQL route to the Axum router:

```rust
use axum::{
    routing::{get, post},
    Router,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

pub fn create_graphql_routes(state: Arc<AppState>) -> Router {
    let schema = Schema::build(
        graphql::queries::Query,
        graphql::mutations::Mutation,
        graphql::subscriptions::Subscription,
    )
    .data(RequestContext::new(state.clone(), None))
    .finish();

    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphql", get(graphql_playground))
        .route("/graphql/subscriptions", get(websocket_subscription_handler))
        .layer(
            Extension::new(state.clone())
        )
        .layer(
            // JWT authentication middleware
            auth_layer(state.clone())
        )
}

async fn graphql_handler(
    schema: Extension<Schema<...>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    // JWT middleware sets auth context
    schema.execute(req.into_inner()).await.into()
}

async fn websocket_subscription_handler(
    ws: WebSocketUpgrade,
    Extension(schema): Extension<Schema<...>>,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
) -> Response {
    let credentials = auth_header.token();
    match validate_jwt(credentials) {
        Ok(auth) => {
            let req_ctx = RequestContext::new(state, Some(auth));
            ws.on_upgrade(|socket| {
                graphql_subscriptions::bind_connection(schema, socket, req_ctx).await
            })
        }
        Err(_) => {
            StatusCode::UNAUTHORIZED.into_response()
        }
    }
}
```

---

## JWT Authentication Integration

The GraphQL API integrates with JWT authentication using middleware that:

1. **Extracts JWT from Authorization header** (`Bearer <token>`)
2. **Validates the JWT** using the shared JWT secret
3. **Creates AuthContext** with user_id, roles, and name
4. **Injects into RequestContext** before resolvers execute

**Authentication flow:**

```rust
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());
    
    let auth = if let Some(header) = auth_header {
        if let Some(token) = header.strip_prefix("Bearer ") {
            match validate_jwt(token, &jwt_secret).await {
                Ok(user) => Some(AuthContext {
                    user_id: user.id,
                    roles: user.roles,
                    name: user.name,
                }),
                Err(_) => return Err(StatusCode::UNAUTHORIZED),
            }
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        None  // Anonymous access for public queries
    };
    
    // Insert auth context into request extensions
    req.extensions_mut().insert(auth);
    
    Ok(next.run(req).await)
}
```

---

## API Reference

### Queries

| Query | Description | Auth Required |
|-------|-------------|---------------|
| `workflow(id: ID!)` | Get single workflow | No (public) |
| `workflows(filter: WorkflowFilter)` | List workflows with filters | No (public) |
| `pendingApprovals` | Get pending approvals for current user | Yes |

### Mutations

| Mutation | Description | Auth Required | Role Check |
|----------|-------------|---------------|------------|
| `createWorkflow(input: CreateWorkflowInput!)` | Create new workflow | Yes | None |
| `approve(input: ApprovalInput!)` | Approve an approval request | Yes | Required role |
| `modify(input: ModificationInput!)` | Approve with modifications | Yes | Required role |
| `reject(requestId: ID!, reason: String!)` | Reject approval request | Yes | Required role |
| `cancelWorkflow(id: ID!)` | Cancel a workflow | Yes | Owner or admin |

### Subscriptions

| Subscription | Description | Auth Required |
|--------------|-------------|---------------|
| `workflowUpdated(id: ID!)` | Real-time workflow status updates | Yes |
| `approvalRequired` | New approval requests for user's roles | Yes |
| `agentCompleted(workflowId: ID!)` | Agent completion events | Yes |

---

## Testing Considerations

When testing the GraphQL API:

1. **Authentication tests** should verify that protected mutations fail without valid JWT
2. **Authorization tests** should verify role-based access control for approvals
3. **Subscription tests** should verify event delivery to connected clients
4. **N+1 query tests** should verify DataLoader batching prevents multiple round-trips
5. **Error handling tests** should verify proper error codes and messages

**Test utilities:**

```rust
// Helper to create test JWT
pub fn create_test_jwt(user_id: Uuid, roles: Vec<String>) -> String {
    // Use test JWT secret
}

// Helper to make authenticated GraphQL request
pub async fn gql_request(
    state: &Arc<TestState>,
    query: &str,
    jwt: Option<String>,
) -> serde_json::Value {
    // Send request and parse response
}
```