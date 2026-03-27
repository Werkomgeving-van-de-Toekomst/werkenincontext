# Authentication Implementation Inventory

**Date:** 2026-03-13
**Phase:** Assessment (Phase 0)
**Updated:** 2026-03-14 (Post-Implementation)

## Current Implementation

### Architecture Overview

IOU-Modern uses **Supabase Auth** as the primary authentication provider, with JWT-based token verification and Row-Level Security (RLS) for multi-tenant isolation.

| Component | Location | Implementation |
|-----------|----------|----------------|
| JWT Middleware | `crates/iou-api/src/middleware/` | Supabase JWT verification |
| JWT Verifier | `crates/iou-api/src/auth/supabase_jwt.rs` | SupabaseJwtVerifier |
| User Schema | Supabase `auth.users` + custom claims | Supabase managed |
| Session Storage | JWT stateless tokens | No server-side storage |
| RLS Policies | PostgreSQL database level | `migrations/postgres/002_rls_policies.sql` |

---

## JWT Token Structure

### Standard Claims (Supabase)

| Claim | Type | Description |
|-------|------|-------------|
| `sub` | String (UUID) | User ID from `auth.users.id` |
| `aud` | String | Audience (typically `"authenticated"`) |
| `role` | String | User role (typically `"authenticated"`) |
| `email` | String | User email address |
| `exp` | i64 | Token expiration timestamp |
| `iat` | i64 | Token issued at timestamp |
| `iss` | String | Token issuer (typically `"supabase"`) |

### Custom Claims (Application-Specific)

| Claim | Type | Description |
|-------|------|-------------|
| `organization_id` | String (UUID) | User's organization UUID for multi-tenancy |
| `clearance` | String | Security clearance level (`openbaar`, `intern`, `vertrouwelijk`, `geheim`) |
| `app_roles` | String[] | Array of role strings (e.g., `["domain_viewer", "object_creator"]`) |

### Example JWT Payload

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "aud": "authenticated",
  "role": "authenticated",
  "email": "user@example.com",
  "organization_id": "660e8400-e29b-41d4-a716-446655440000",
  "clearance": "intern",
  "app_roles": ["domain_viewer", "object_creator"],
  "exp": 1798456789,
  "iat": 1798453189,
  "iss": "supabase"
}
```

---

## Protected Endpoints

All routes under `/api/*` require authentication. The JWT middleware:

1. Extracts `Authorization: Bearer <token>` header
2. Verifies token signature using `SUPABASE_JWT_SECRET`
3. Validates issuer (`supabase`) and audience (`authenticated`)
4. Checks expiration time
5. Converts claims to `AuthContext` for downstream use

**Protected routes include:**
- `/api/documents/*` - Document CRUD operations
- `/api/domains/*` - Information domain management
- `/api/objects/*` - Information object management
- `/api/templates/*` - Template management
- `/api/admin/*` - Admin operations (requires elevated roles)

---

## Role-Based Access Control (RBAC)

### Role Definitions

| Role | Description | Permissions |
|------|-------------|--------------|
| `domain_viewer` | Read-only access to domains | View domains, objects |
| `domain_editor` | Edit access within organization | Create/edit objects |
| `object_creator` | Can create new objects | Create documents, objects |
| `object_approver` | Can approve documents | Approve/reject workflow |
| `admin` | Full administrative access | All operations |

### Role Enforcement

- Roles are checked at API handler level via `AuthContext`
- RLS policies enforce organization isolation at database level
- Classification-based filtering restricts access by clearance level

---

## Multi-Tenancy Structure

### Organization Isolation

- Each user belongs to exactly one organization (`organization_id`)
- RLS policies ensure users can only access their organization's data
- Organization membership is enforced at both API and database levels

### RLS Helper Functions

```sql
-- Extract organization_id from JWT
auth.organization_id() RETURNS UUID

-- Extract user role from JWT
auth.user_role() RETURNS VARCHAR

-- Check clearance level
auth.has_clearance(required_level VARCHAR) RETURNS BOOLEAN
```

---

## Migration from Legacy Auth

### Previous Implementation (Before Migration)

| Component | Status | Notes |
|-----------|--------|-------|
| Local user storage | Removed | No local user table |
| Custom JWT signing | Removed | Now uses Supabase JWT |
| Session management | Removed | Stateless JWT only |
| Password hashing | Removed | Handled by Supabase |

### Migration Path

1. **Phase 2 (Auth & Real-time):** Supabase Auth configured, RLS policies created
2. **Phase 4 (Cutover):** All auth traffic routed through Supabase
3. **Phase 6 (Cleanup):** Legacy auth code removed

---

## Configuration

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `SUPABASE_JWT_SECRET` | Yes | JWT secret for token verification |
| `SUPABASE_JWT` | No | Alternative name for JWT secret |
| `SUPABASE_URL` | Yes | Supabase project URL |
| `SUPABASE_ANON_KEY` | Yes | Supabase anonymous key |

### JWT Configuration

```rust
// Algorithm: HS256
// Issuer: "supabase"
// Audience: "authenticated"
// Secret: From SUPABASE_JWT_SECRET environment variable
```

---

## Security Considerations

### Token Validation

- Signature verification using HS256
- Expiration time enforced
- Issuer and audience validated
- Organization_id required for all operations

### RLS Policy Enforcement

- Organization isolation at database level
- Classification-based access control
- Owner-based policies for user data
- Woo publication access for public documents

### Clearance Levels

| Level | Description | Access |
|-------|-------------|--------|
| `openbaar` | Public | All users |
| `intern` | Internal | Authenticated users |
| `vertrouwelijk` | Confidential | Users with clearance |
| `geheim` | Secret | Users with geheim clearance |

---

## References

- Implementation: `crates/iou-api/src/auth/supabase_jwt.rs`
- RLS Policies: `migrations/postgres/002_rls_policies.sql`
- Middleware: `crates/iou-api/src/middleware/`
