# Hosting Location Decision

**Date:** 2026-03-13
**Phase:** Assessment (Phase 0)
**Updated:** 2026-03-14 (Decision Finalized)

## Decision Matrix

| Criteria | Weight | Rijkscloud | Commercial EU (Hetzner) | On-premises | Chosen |
|----------|--------|------------|-------------------------|-------------|--------|
| EU Data Residency | Critical | ✅ Yes | ✅ Yes | ✅ Yes | All meet requirement |
| Dutch Control | High | ✅ Full | ⚠️ Partial | ✅ Full | On-prem/Hetzner |
| Cost | Medium | ❓ Unknown | ✅ €40-100/mo | ❌ High upfront | Hetzner |
| Time to Deploy | High | ❓ Weeks | ✅ Hours/Days | ❌ Months | Hetzner |
| Compliance Support | High | ✅ Built-in | ✅ Self-managed | ⚠️ Self-managed | Rijkscloud/Hetzner |
| Maintenance Overhead | Medium | ✅ Low | ✅ Low | ❌ High | Hetzner/Rijkscloud |

## Selected Option

**Decision: Self-Hosted Supabase (Commercial EU Infrastructure)**

**Provider:** Hetzner Online GmbH (German-based, EU data residency)

**Rationale:**

1. **EU Data Residency:** ✅ Hetzner data centers are in Germany (EU), GDPR compliant
2. **Dutch Control:** Partial - German jurisdiction but EU GDPR provides consistency
3. **Cost:** Predictable monthly cost (€40-100/month for typical deployment)
4. **Time to Deploy:** Fast - Docker Compose deployment in hours/days
5. **Compliance:** Self-managed but with clear documentation path for BVO/GDPR
6. **Flexibility:** Full control over Supabase configuration and updates

## Infrastructure Specification

### Recommended Configuration

| Component | Specification |
|-----------|---------------|
| **Server** | Hetzner CX41 or equivalent |
| **CPU** | 4 vCPUs |
| **RAM** | 16 GB |
| **Storage** | 160 GB NVMe SSD |
| **Location** | Falkenstein (FSN1) or Nuremberg (NBG1) |
| **Estimated Cost** | ~€45/month |
| **Operating System** | Ubuntu 22.04 LTS |

### Services Deployed

- **Supabase Studio** - Database management interface
- **PostgreSQL** - Primary database (port 5432)
- **Supabase Auth** - Authentication service
- **Supabase Realtime** - WebSocket service (port 4000)
- **Supabase Storage** - S3-compatible storage (port 9000)
- **pgAdmin** - Database administration (optional)
- **Grafana** - Monitoring dashboard (optional)

### Network Configuration

```
┌─────────────────────────────────────────────────────────┐
│                    Hetzner Server                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Supabase    │  │   IOU-API    │  │   DuckDB     │  │
│  │  (PostgreSQL)│  │   (Axum)     │  │  (Analytics) │  │
│  │  Port: 5432  │  │  Port: 8080  │  │   Local file │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│  ┌──────────────┐  ┌──────────────┐                     │
│  │  Realtime    │  │   Grafana    │                     │
│  │  Port: 4000  │  │  Port: 3000  │                     │
│  └──────────────┘  └──────────────┘                     │
└─────────────────────────────────────────────────────────┘
                          │
                          │ HTTPS (Let's Encrypt)
                          ▼
                   ┌─────────────┐
                   │   Internet  │
                   └─────────────┘
```

## Deployment Strategy

### Phase 1: Initial Setup (Week 1)

1. Provision Hetzner server
2. Configure DNS (A record for API domain)
3. Install Docker & Docker Compose
4. Deploy Supabase via `docker-compose.supabase.yml`
5. Configure SSL with Let's Encrypt (Traefik/Caddy)

### Phase 2: Migration (Weeks 2-3)

1. Run Supabase in parallel with DuckDB
2. Set up dual-write pattern
3. Migrate user data to Supabase Auth
4. Configure RLS policies
5. Test real-time subscriptions

### Phase 3: Cutover (Week 4)

1. Switch primary reads to Supabase
2. Enable ETL pipeline to DuckDB
3. Monitor for 48 hours
4. Deprecate direct DuckDB writes

## Backup & Disaster Recovery

### Backup Strategy

| Backup Type | Frequency | Retention | Location |
|-------------|-----------|-----------|----------|
| PostgreSQL Full | Daily | 30 days | Hetzner Storage Box |
| PostgreSQL WAL | Continuous | 7 days | Local |
| DuckDB Export | Weekly | 4 weeks | Hetzner Storage Box |
| Config Backup | On change | Indefinite | Git + Storage Box |

### Recovery Time Objectives

- **RPO (Recovery Point Objective):** 5 minutes (WAL replication)
- **RTO (Recovery Time Objective):** 1 hour (from backup)

### Disaster Recovery

1. **Server Failure:** Restore from Hetzner snapshot (15 minutes)
2. **Data Corruption:** Restore from PostgreSQL backup (1 hour)
3. **Region Outage:** Deploy to backup region (manual, 4-8 hours)

## Compliance Considerations

### GDPR (AVG)

- ✅ EU data residency (Germany)
- ✅ Data processing agreement with Hetzner
- ✅ Right to erasure (Supabase Auth support)
- ✅ Data portability (PostgreSQL export)

### Dutch Government Requirements

- ✅ Bijhoudingsplicht (audit trail via `audit_trail` table)
- ✅ Woo compliance (publication workflow)
- ⚠️ BVO (Basisveiligheidshuis) - Additional hardening required:
  - SIEM integration (recommended: Loki/Grafana)
  - Multi-factor auth for admin access
  - Regular security updates

### Archiefwet

- Automated retention policies (PostgreSQL scheduled jobs)
- Data deletion after retention period expires
- Audit trail preservation (separate from main data)

## Alternative: Rijkscloud (Future Consideration)

### Why Not Chosen Now

| Factor | Status |
|--------|--------|
| Availability | Not yet available for IOU-Modern's use case |
| Pricing | Unknown, likely higher than Hetzner |
| Onboarding | Requires government procurement process |
| Timeline | 3-6 months minimum |

### Migration Path to Rijkscloud

If Rijkscloud becomes available:

1. **Export data** from Supabase (PostgreSQL dump)
2. **Import to Rijkscloud** PostgreSQL instance
3. **Update DNS** to point to Rijkscloud load balancer
4. **Decommission** Hetzner instance

**Estimated migration time:** 1-2 days

## Cost Summary

### Monthly Operating Costs

| Item | Cost (EUR) |
|------|------------|
| Server (CX41) | €45.00 |
| Storage Box (100 GB) | €5.00 |
| Bandwidth (20 TB included) | €0.00 |
| Domain | €10.00 (yearly, ~€0.83/mo) |
| SSL Certificate | €0.00 (Let's Encrypt) |
| **Total** | **~€51/month** |

### One-Time Setup Costs

| Item | Cost (EUR) |
|------|------------|
| Initial setup | €0.00 (in-house) |
| Domain registration | €10.00 |
| **Total** | **~€10** |

## Monitoring & Alerting

### Metrics to Monitor

- Database connection pool utilization
- Query latency (p50, p95, p99)
- RLS policy check performance
- ETL cycle duration
- Disk usage and WAL size
- Real-time subscription count

### Alert Channels

- Email (critical alerts)
- Slack (operational alerts)
- PagerDuty (emergency, if available)

---

**Approved By:** Backend Team Lead
**Date:** 2026-03-14
**Next Review:** 2026-09-14 (6 months)
