# Architecture Diagram: Deployment - Metadata Registry Service

> **Template Origin**: Official | **ArcKit Version**: 4.3.1 | **Command**: `/arckit:diagram deployment`

## Document Control

| Field | Value |
|-------|-------|
| **Document ID** | ARC-002-DIAG-008-v1.0 |
| **Document Type** | Architecture Diagram |
| **Project** | Metadata Registry Service (Project 002) |
| **Classification** | OFFICIAL |
| **Status** | DRAFT |
| **Version** | 1.0 |
| **Created Date** | 2026-04-19 |
| **Last Modified** | 2026-04-19 |
| **Review Cycle** | On-Demand |
| **Next Review Date** | 2026-05-19 |
| **Owner** | Enterprise Architect |
| **Reviewed By** | PENDING |
| **Approved By** | PENDING |
| **Distribution** | Project Team, Architecture Team, DevOps, SRE |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---------|------|--------|---------|-------------|---------------|
| 1.0 | 2026-04-19 | ArcKit AI | Initial creation from `/arckit:diagram deployment` command | PENDING | PENDING |

---

## Diagram Purpose

This deployment diagram shows the Kubernetes-based architecture for the Metadata Registry Service, including multi-region deployment, scaling strategy, disaster recovery, and observability integration.

---

## Multi-Region Deployment Architecture

```mermaid
C4Deployment
    title Deployment Diagram - Metadata Registry Service

    Person(developer, "Developer", "Deploys via CI/CD")
    Person(sre, "SRE Team", "Operates and monitors")
    Person(user, "Metadata Steward", "Uses admin UI")

    Deployment_Node(mreg_eu, "EU-Central Region", "Frankfurt, Germany") {
        Deployment_Node(k8s_prod, "Kubernetes Cluster", "Production") {
            ContainerDb(api_pod_1, "REST API", "Pod 1", "replica 1/3")
            ContainerDb(api_pod_2, "REST API", "Pod 2", "replica 2/3")
            ContainerDb(api_pod_3, "REST API", "Pod 3", "replica 3/3")

            ContainerDb(graphql_pod_1, "GraphQL API", "Pod 1", "replica 1/2")
            ContainerDb(graphql_pod_2, "GraphQL API", "Pod 2", "replica 2/2")

            ContainerDb(gitops_pod, "GitOps Sync", "Single instance", "")

            ContainerDb(admin_pod, "Admin UI", "WASM served", "")

            ContainerDb(arango_db_1, "ArangoDB", "Primary", "Coordinator")
            ContainerDb(arango_db_2, "ArangoDB", "DB Server 1", "Data node")
            ContainerDb(arango_db_3, "ArangoDB", "DB Server 2", "Data node")
            ContainerDb(arango_db_4, "ArangoDB", "DB Server 3", "Data node")
        }

            Deployment_Node(monitoring, "Observability Stack", "") {
                Container(prometheus, "Prometheus", "Metrics collection")
                Container(grafana, "Grafana", "Dashboards")
                Container(loki, "Loki", "Log aggregation")
                Container(tempo, "Tempo", "Distributed tracing")
            }

            Deployment_Node(gitlab, "GitLab", "GitOps repository") {
                Container(repo, "metadata-registry", "YAML definitions")
            }

            Deployment_Node(woo, "Woo Portal", "Government publication") {
                Container(woo_api, "Woo API", "Publication endpoint")
            }

            Deployment_Node(cdd, "CDD+", "Long-term archive") {
                Container(cdd_api, "CDD+ API", "Archive endpoint")
            }
        }

        Deployment_Node(k8s_dr, "Kubernetes Cluster", "DR Site") {
            ContainerDb(api_dr, "REST API", "Hot standby", "0/1")
            ContainerDb(arango_dr, "ArangoDB", "Async replica", "Standby")
        }
    }

    Rel(developer, k8s_prod, "Deploys via", "GitOps CI/CD")
    Rel(sre, k8s_prod, "Monitors via", "Prometheus/Grafana")
    Rel(user, admin_pod, "HTTPS", "Browser")

    Rel(api_pod_1, arango_db_1, "Read/Write", "arangors")
    Rel(graphql_pod_1, arango_db_1, "Read/Write", "AQL")
    Rel(gitops_pod, arango_db_1, "Sync", "arangors")
    Rel(admin_pod, api_pod_1, "API calls", "REST/JSON")
    Rel(admin_pod, graphql_pod_1, "Queries", "GraphQL")

    Rel(gitops_pod, repo, "Pull/Push", "SSH/HTTPS")
    Rel(api_pod_1, woo_api, "Publish", "REST/OAuth")
    Rel(api_pod_1, cdd_api, "Archive", "REST/mTLS")

    Rel(api_pod_1, prometheus, "Exposes metrics", "/metrics")
    Rel(api_pod_1, loki, "Sends logs", "Loki push API")
    Rel(api_pod_1, tempo, "Sends traces", "OTLP")

    Rel(arango_db_1, arango_dr, "Replication", "ArangoSync")

    UpdateLayoutConfig($c4BoundaryInRow="2")
```

---

## Kubernetes Namespace Structure

```
metadata-registry/
├── Namespace: metadata-registry-prod
│   ├── Deployment: metadata-api (3 replicas)
│   ├── Deployment: metadata-graphql (2 replicas)
│   ├── Deployment: metadata-gitops (1 replica)
│   ├── Deployment: metadata-admin (2 replicas)
│   ├── StatefulSet: arangodb (4 replicas: 1 coordinator, 3 DB servers)
│   ├── Service: metadata-api (ClusterIP)
│   ├── Service: metadata-graphql (ClusterIP)
│   ├── Service: metadata-admin (LoadBalancer)
│   ├── Service: arangodb (ClusterIP)
│   ├── Ingress: metadata-registry (external routing)
│   ├── ConfigMap: metadata-config
│   └── AuthConfig: arango-auth
│
├── Namespace: metadata-registry-monitoring
│   ├── Deployment: prometheus
│   ├── Deployment: grafana
│   ├── Deployment: loki
│   ├── Deployment: tempo
│   └── Service: * (ClusterIP)
│
└── Namespace: metadata-registry-dr
    ├── Deployment: metadata-api-dr (0 replicas, scaled up on failover)
    ├── StatefulSet: arangodb-dr (1 replica, async follower)
    └── Service: arangodb-dr (ClusterIP)
```

---

## Resource Specifications

### API Pods

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: metadata-api
  namespace: metadata-registry-prod
spec:
  replicas: 3
  selector:
    matchLabels:
      app: metadata-api
  template:
    metadata:
      labels:
        app: metadata-api
        version: v1.0.0
    spec:
      containers:
      - name: metadata-api
        image: registry.gitlab.com/iou-modern/metadata-api:1.0.0
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        env:
        - name: ARANGO_DB_URL
          valueFrom:
            configMapKeyRef:
              name: arango-config
              key: url
        # Credentials loaded from Kubernetes auth config
        - name: ARANGO_DB_CREDENTIALS
          valueFrom:
            configMapKeyRef:
              name: arango-auth
              key: credentials
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
```

### ArangoDB Cluster

```yaml
apiVersion: "arangodb.com/v1"
kind: "ArangoDeployment"
metadata:
  name: arangodb
  namespace: metadata-registry-prod
spec:
  mode: Cluster
  image: arangodb/arangodb:3.11.x
  replicas: 4
  agents:
    count: 3
    resources:
      requests:
        memory: "4Gi"
        cpu: "2"
      limits:
        memory: "8Gi"
        cpu: "4"
    storage:
      size: 100Gi
      storageClass: fast-ssd
  dbservers:
    count: 3
    resources:
      requests:
        memory: "8Gi"
        cpu: "4"
      limits:
        memory: "16Gi"
        cpu: "8"
    storage:
      size: 500Gi
      storageClass: fast-ssd
  coordinators:
    count: 1
    resources:
      requests:
        memory: "2Gi"
        cpu: "1"
      limits:
        memory: "4Gi"
        cpu: "2"
  sync: true
  syncMasterEndpoints:
  - "https://arangodb-dr.metadata-registry-dr.svc:8529"
```

---

## Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: metadata-api-hpa
  namespace: metadata-registry-prod
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: metadata-api
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30
      - type: Pods
        value: 2
        periodSeconds: 60
      selectPolicy: Max
```

---

## Multi-Region Strategy

### Primary Region: EU-Central (Frankfurt)

**Purpose**: Active production traffic
**Components**:
- 3x API replicas
- 2x GraphQL replicas
- 1x GitOps service
- 2x Admin UI replicas
- 4x ArangoDB (1 coordinator, 3 DB servers)

### DR Region: EU-West (Ireland)

**Purpose**: Disaster recovery
**Components**:
- 0x API replicas (scale on failover)
- 1x ArangoDB async follower

### Failover Procedure

```
┌─────────────────────────────────────────────────────────┐
│                    FAILOVER DECISION                    │
│                  (Manual or Automated)                  │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│  1. Scale up API pods in DR region (0 → 3 replicas)    │
│     kubectl scale deployment metadata-api-dr --replicas=3│
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│  2. Promote ArangoDB follower to primary               │
│     arangodbr --server.dr-arangodb --database.metadata  │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│  3. Update DNS/load balancer to DR region              │
│     DNS CNAME: api.metadata-registry.nl → eu-west      │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│  4. Verify DR region health                            │
│     - API health checks                                 │
│     - ArangoDB cluster status                           │
│     - Smoke tests                                       │
└─────────────────────────────────────────────────────────┘
```

**RTO (Recovery Time Objective)**: <4 hours
**RPO (Recovery Point Objective)**: <1 hour (ArangoSync interval)

---

## Observability Integration

### Metrics (Prometheus)

```yaml
# Exposed by all pods at :9090/metrics
- http_requests_total (endpoint, method, status)
- http_request_duration_seconds (endpoint, quantile)
- arangodb_query_duration_seconds (collection, operation)
- arangodb_connection_pool_size
- git_sync_last_success_timestamp
- entity_count (entity_type)
```

### Logging (Loki)

```yaml
# Structured JSON logs
{
  "timestamp": "2024-04-19T10:00:00Z",
  "level": "info",
  "service": "metadata-api",
  "pod": "metadata-api-7d8f9c5b-x4k2p",
  "trace_id": "abc123",
  "span_id": "def456",
  "message": "Entity created",
  "entity_type": "gebeurtenis",
  "entity_key": "evt-001",
  "user_id": "user-123",
  "organisation_id": "org-456"
}
```

### Tracing (Tempo)

```yaml
# Distributed tracing for request flows
Trace: GET /api/v2/gebeurtenissen
├── Span: HTTP request handler
├── Span: ArangoDB query (gebeurtenis)
├── Span: Validation check
├── Span: Row-level security filter
└── Span: Response serialization
```

### Dashboards (Grafana)

1. **API Performance**: Request rate, latency, error rate
2. **Database Health**: Query performance, connection pool, replication lag
3. **GitOps Status**: Last sync time, sync errors, sync duration
4. **Resource Usage**: CPU, memory, network per pod
5. **Business Metrics**: Entity counts by type, organization activity

---

## Security

### Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: metadata-api-policy
  namespace: metadata-registry-prod
spec:
  podSelector:
    matchLabels:
      app: metadata-api
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx  # Allow from ingress
    ports:
    - protocol: TCP
      port: 8080
  - from:
    - namespaceSelector:
        matchLabels:
          name: metadata-registry-prod
    ports:
    - protocol: TCP
      port: 9090  # Metrics for Prometheus
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: metadata-registry-prod
    ports:
    - protocol: TCP
      port: 8529  # ArangoDB
```

### Authentication Configuration

```yaml
# External Secrets Operator for auth management
apiVersion: v1
kind: ConfigMap
metadata:
  name: arango-auth
  namespace: metadata-registry-prod
data:
  # References to external auth provider
  auth_provider: "oauth2"
  auth_endpoint: "https://auth.example.com/oauth/token"
```

---

## Disaster Recovery Plan

### Backup Strategy

| Component | Frequency | Retention | Location |
|-----------|-----------|-----------|----------|
| ArangoDB snapshots | Hourly | 30 days | Primary region S3 |
| ArangoDB backups | Daily | 7 years | DR region object storage |
| Git repository | Continuous | 7 years | GitLab geo-replication |

### Recovery Runbook

1. **Data Loss <1 hour**: Restore from latest ArangoDB snapshot
2. **Data Loss <1 day**: Restore from daily backup + replay ArangoSync logs
3. **Region Failure**: Failover to DR region per procedure above
4. **Complete Data Loss**: Rebuild from Git repository (source of truth)

---

## Related Documents

- **ARC-002-REQ-v1.1**: NFR-MREG-A-2 (Disaster Recovery)
- **ARC-002-ADR-005**: Sovereign Technology (EU deployment)
- **ARC-002-DIAG-003**: Component Diagram (Container specifications)
