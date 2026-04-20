# Security Design: Metadata Registry Service

> **Document ID**: ARC-002-SEC-v1.0 | **Status**: DRAFT

## Document Control

| Field | Value |
|-------|-------|
| **Document Type** | Security Design Specification |
| **Project** | Metadata Registry Service (Project 002) |
| **Version** | 1.0 |
| **Created Date** | 2026-04-19 |

---

## 1. Security Overview

### 1.1 Threat Model

| Threat Category | Description | Mitigation |
|-----------------|-------------|------------|
| Unauthorized access | API access without valid credentials | OAuth 2.0/OIDC authentication |
| Data exposure | Sensitive metadata exposed to wrong users | Row-Level Security (RLS) |
| Data tampering | Unauthorized modification of metadata | Audit trail, immutability |
| Data exfiltration | Bulk export of sensitive data | Rate limiting, quota enforcement |
| Injection attacks | Malicious AQL/GraphQL queries | Input validation, query sanitization |
| DoS attacks | Service disruption | Rate limiting, circuit breakers |

### 1.2 Security Principles

1. **Defense in Depth**: Multiple security layers
2. **Least Privilege**: Minimum required access
3. **Audit Everything**: Complete traceability
4. **Secure by Default**: Deny unless explicitly allowed

---

## 2. Authentication

### 2.1 OAuth 2.0 / OpenID Connect Flow

```
┌─────────────┐                ┌──────────────┐
│   Client    │                │  OAuth Provider│
│  (Browser)  │                │    (eHerkenning)│
└──────┬──────┘                └──────┬───────┘
       │                               │
       │ 1. Request authorization       │
       ├───────────────────────────────>│
       │                               │
       │ 2. Redirect to login          │
       │<───────────────────────────────┤
       │                               │
       │ 3. User credentials            │
       ├───────────────────────────────>│
       │                               │
       │ 4. Authorization code          │
       │<───────────────────────────────┤
       │                               │
       │ 5. Exchange code for token     │
       ├───────────────────────────────>│
       │                               │
       │ 6. Access token + Refresh token│
       │<───────────────────────────────┤
       │                               │
       │ 7. API request with token      │
       ├───────────────────────────────>│
```

### 2.2 Token Format

```json
// JWT Header
{
  "alg": "RS256",
  "typ": "JWT",
  "kid": "key-2024-04-19"
}

// JWT Payload
{
  "iss": "https://auth.metadata-registry.nl",
  "sub": "user-456",
  "aud": "api://metadata-registry",
  "exp": 1713528600,
  "iat": 1713525000,
  "org_id": "org-123",
  "roles": ["metadata_steward", "domain_owner"],
  "email": "user@example.nl"
}
```

### 2.3 Token Validation

```rust
use jsonwebtoken::{decode, Validation, DecodingKey};

pub struct JwtClaims {
    pub sub: String,
    pub org_id: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

pub fn validate_token(token: &str, secret: &[u8]) -> Result<JwtClaims> {
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::new(jsonwebtoken::Algorithm::RS256)
    )?;
    
    Ok(token_data.claims)
}
```

### 2.4 MFA Requirements

| User Role | MFA Required | Methods |
|-----------|--------------|---------|
| Standard | No | Password only |
| Domain Owner | Yes | TOTP app |
| Woo Officer | Yes | TOTP + Hardware key |
| Admin | Yes | TOTP + Hardware key |

---

## 3. Authorization

### 3.1 Role-Based Access Control (RBAC)

| Role | Permissions |
|------|-------------|
| `metadata_viewer` | Read entities in own organization |
| `metadata_steward` | CRUD entities in own organization |
| `domain_owner` | Approve changes, approve Woo publications |
| `woo_officer` | Approve/reject Woo publications |
| `dpo` | View all entities, access audit logs, process SARs |
| `admin` | Full system access |

### 3.2 Object-Level Authorization

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum RechtType {
    Lezen,
    Bewerken,
    Goedkeuren,
    Verwijderen,
}

pub struct AuthorizationCheck {
    pub user_id: String,
    pub roles: Vec<String>,
    pub organisatie_id: String,
}

impl AuthorizationCheck {
    pub async fn can_access_entity(
        &self,
        entity_key: &str,
        required: RechtType,
        db: &Database,
    ) -> Result<bool> {
        // Check direct grants
        let direct = self.check_direct_grant(entity_key, required, db).await?;
        if direct { return Ok(true); }
        
        // Check inherited zaak rights
        let inherited = self.check_inherited_grant(entity_key, required, db).await?;
        if inherited { return Ok(true); }
        
        // Check organization rights
        let org = self.check_organization_rights(required).await?;
        
        Ok(org)
    }
}
```

### 3.3 Row-Level Security (RLS)

**Implementation in Repository Layer:**

```rust
impl Repository<Gebeurtenis> for GebeurtenisRepository {
    async fn list(&self, filter: &RepositoryFilter, auth: &AuthorizationCheck) 
        -> Result<Vec<Gebeurtenis>>
    {
        let conn = self.pool.connection().await?;
        let db = conn.db();
        
        let mut aql = r#"
            FOR doc IN gebeurtenis
              FILTER doc.organisatie_id == @org_id
              FILTER doc.geldig_vanaf <= @now
              FILTER doc.geldig_tot >= @now
        "#.to_string();
        
        // Apply RLS filter
        if !auth.is_dpo() && !auth.is_admin() {
            aql += r#"
                // Additional RLS filters can be added here
            "#;
        }
        
        aql += "SORT doc.aangemaakt_op DESC LIMIT @limit RETURN doc";
        
        let cursor = db.aql_bind(
            &aql,
            json!({
                "org_id": auth.organisatie_id,
                "now": filter.geldig_on.unwrap_or_else(Utc::now),
                "limit": filter.limit.unwrap_or(50)
            })
        ).await?;
        
        // ... process cursor
    }
}
```

---

## 4. Data Protection

### 4.1 Encryption

| Data Type | At Rest | In Transit |
|-----------|---------|------------|
| ArangoDB data | ArangoDB encryption (RocksDB) | TLS 1.3 |
| API traffic | N/A | TLS 1.3 |
| Git repository | GPG encryption (optional) | SSH/HTTPS |
| Backup data | AES-256 (S3 SSE-KMS) | TLS 1.3 |

### 4.2 PII Detection and Handling

```rust
pub struct PIIDetector {
    bsn_pattern: Regex,
    email_pattern: Regex,
    phone_pattern: Regex,
    iban_pattern: Regex,
    passport_pattern: Regex,
}

impl PIIDetector {
    pub fn scan(&self, text: &str) -> Vec<PIIMatch> {
        let mut matches = Vec::new();
        
        // BSN (Dutch social security number)
        for m in self.bsn_pattern.find_iter(text) {
            if validate_bsn_checksum(m.as_str()) {
                matches.push(PIIMatch {
                    field: "bsn",
                    value: m.as_str().to_string(),
                    confidence: 0.95,
                    start: m.start(),
                    end: m.end(),
                });
            }
        }
        
        // Email
        for m in self.email_pattern.find_iter(text) {
            matches.push(PIIMatch {
                field: "email",
                value: m.as_str().to_string(),
                confidence: 0.90,
                start: m.start(),
                end: m.end(),
            });
        }
        
        // ... other patterns
        
        matches
    }
}
```

### 4.3 PII Response Actions

| Confidence Level | Action |
|------------------|--------|
| > 0.9 | Auto-flag as AVG data, require DPO approval |
| 0.7 - 0.9 | Flag for review |
| < 0.7 | Log only |

---

## 5. Audit Logging

### 5.1 Audit Events

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: AuditEventType,
    pub actor: ActorInfo,
    pub resource: ResourceInfo,
    pub action: Action,
    pub outcome: Outcome,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuditEventType {
    EntityCreated,
    EntityUpdated,
    EntityDeleted,
    EntityAccessed,
    AuthenticationSuccess,
    AuthenticationFailed,
    AuthorizationFailed,
    ExportRequested,
    SarRequested,
}

pub async fn log_audit_event(
    db: &Database,
    event: AuditEvent,
) -> Result<()> {
    let collection = db.collection("audit");
    collection.create_entity(&event).await?;
    Ok(())
}
```

### 5.2 Audit Log Retention

| Event Type | Retention Period |
|------------|------------------|
| Authentication | 1 year |
| Authorization | 2 years |
| Entity CRUD | 7 years (AVG requirement) |
| SAR requests | 7 years |
| PII access | 7 years |
| Export requests | 7 years |

---

## 6. API Security

### 6.1 Rate Limiting

```rust
use governor::{Quota, RateLimiter};

pub struct RateLimiter {
    limiter: RateLimiter<...>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(nonzero!(requests_per_minute));
        Self {
            limiter: RateLimiter::direct(quota),
        }
    }
    
    pub fn check(&self) -> Result<(), RateLimitError> {
        self.limiter.check()
    }
}

// Apply to API middleware
async fn rate_limit_middleware(
    req: ServiceRequest,
    next: Next<...>,
) -> Result<ServiceResponse, Error> {
    let limiter = get_rate_limiter_for_user(&req)?;
    limiter.check()?;
    next.call(req).await
}
```

### 6.2 Request Validation

```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGebeurtenisRequest {
    #[validate(length(min = 1, max = 255))]
    pub naam: String,
    
    #[validate(length(max = 5000))]
    pub omschrijving: Option<String>,
    
    #[validate(custom = "validate_gebeurtenistype")]
    pub gebeurtenistype: String,
    
    #[validate(custom = "validate_time_validity")]
    pub geldig_vanaf: DateTime<Utc>,
    
    #[validate(custom = "validate_time_validity")]
    pub geldig_tot: DateTime<Utc>,
    
    #[validate(length(min = 1, max = 50))]
    pub organisatie_id: String,
}

pub fn validate_time_validity(vanaf: DateTime<Utc>, tot: DateTime<Utc) 
    -> Result<(), validator::ValidationError>
{
    if tot <= vanaf {
        return Err(validator::ValidationError::new("invalid_time_validity"));
    }
    Ok(())
}
```

---

## 7. Network Security

### 7.1 Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: metadata-api-policy
spec:
  podSelector:
    matchLabels:
      app: metadata-api
  policyTypes:
  - Ingress
  - Egress
  ingress:
  # Allow from ingress only
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  # Allow ArangoDB access
  - to:
    - namespaceSelector:
        matchLabels:
          name: metadata-registry-prod
    ports:
    - protocol: TCP
      port: 8529
  # Allow external API calls (Woo, CDD+)
  - to:
    - namespaceSelector: {}
    ports:
    - protocol: TCP
      port: 443
```

### 7.2 TLS Configuration

```rust
use rustls::ServerConfig;

pub fn tls_config() -> ServerConfig {
    ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(
            vec![load_certificate()],
            load_private_key(),
        )
        .expect("Invalid TLS config")
}

// Minimum TLS version: 1.3
// Allowed cipher suites: TLS_AES_256_GCM_SHA384, TLS_AES_128_GCM_SHA256
// HSTS: max-age=31536000; includeSubDomains
```

---

## 8. Compliance

### 8.1 AVG/GDPR Compliance

| Requirement | Implementation |
|-------------|----------------|
| Lawful basis | Grondslag entity for all processing |
| Data minimization | Required fields only |
| Purpose limitation | Zaak/dossier context tracking |
| Storage limitation | Time-based validity + archival |
| Accuracy | Audit trail for all changes |
| Integrity | Row-Level Security |
| Confidentiality | Encryption at rest and in transit |
| Accountability | Complete audit logging |

### 8.2 SAR (Subject Access Request)

```rust
pub async fn process_sar(
    db: &Database,
    user_id: &str,
    dpo_id: &str,
) -> Result<SARReport> {
    // Collect all data for user
    let entities = db.get_entities_accessible_by_user(user_id).await?;
    let audit_logs = db.get_audit_logs_for_user(user_id).await?;
    let auth_logs = db.get_auth_logs_for_user(user_id).await?;
    
    // Generate report
    let report = SARReport {
        user_id: user_id.to_string(),
        generated_at: Utc::now(),
        generated_by: dpo_id.to_string(),
        entities: entities,
        audit_logs: audit_logs,
        auth_logs: auth_logs,
    };
    
    // Log SAR request
    log_audit_event(db, AuditEventType::SarRequested, ...).await?;
    
    Ok(report)
}
```

### 8.3 Right to Erasure

```rust
pub async fn process_erasure_request(
    db: &Database,
    user_id: &str,
    request_id: &str,
) -> Result<()> {
    // Anonymize instead of delete (for audit purposes)
    db.anonymize_user_references(user_id).await?;
    
    // Log erasure
    log_audit_event(db, AuditEventType::DataErased, ...).await?;
    
    Ok(())
}
```

---

## 9. Security Testing

### 9.1 Required Tests

| Test Type | Frequency | Coverage |
|-----------|-----------|----------|
| SAST | Every commit | 100% |
| Dependency scan | Daily | All dependencies |
| Penetration testing | Quarterly | All endpoints |
| Authorization matrix testing | Release | All roles × all endpoints |
| PII detection testing | Release | All PII types |

### 9.2 Security Headers

```http
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'self'
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Referrer-Policy: no-referrer
Permissions-Policy: geolocation=(), microphone=()
```

---

## 10. Related Documents

- ARC-002-DLD-v1.0: Detailed Design
- ARC-002-REQ-v1.1: Security Requirements
- ARC-002-ADR-005: Sovereign Technology
