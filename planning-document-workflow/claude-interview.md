# Interview Transcript: Enhanced Document Workflow

## Interview Questions & Answers

### Feature: Multi-Stage Approvals

**Q1: How should approval stages be defined for different document types?**
**A:** Config file based (YAML/JSON config files that can be hot-reloaded)

**Q5: How should parallel approvals work when multiple approvers are assigned to a stage?**
**A:** Configurable per stage (any/all/majority)

**Q9: Where should workflow configuration files be stored?**
**A:** Hybrid (global defaults with domain overrides)

**Q10: Should workflow configs support hot-reload without service restart?**
**A:** Yes, automatic hot-reload

### Feature: Delegation

**Q2: What delegation capabilities are needed?**
**A:** All three types:
- Bulk delegation (delegate all during date range)
- Per-document-type delegation
- Single-document delegation

### Feature: Approval Expiry & Escalation

**Q3: How should business days be calculated for SLA deadlines?**
**A:** Skip weekends only (simple 24-hour cycles excluding weekends)

**Q6: What should happen when an approval deadline expires?**
**A:** Configurable per document type

**Q13: How should escalation notifications be delivered when deadlines approach?**
**A:** All options:
- Push notification (Supabase realtime)
- Webhook
- UI indicator
- Email

### Feature: Version History with Diff View

**Q4: What diff visualization format should be used?**
**A:** All formats with user preference:
- Unified diff
- Side-by-side
- Inline highlighting

**Q7: How should document versions be stored?**
**A:** Hybrid approach (full for last N versions, compressed for older)

**Q12: How many versions should be kept as full storage before compression?**
**A:** Configurable per document type

**Q11: When restoring a previous version, how should it be handled?**
**A:** In-place revert

## Summary of Decisions

### Workflow Configuration
- YAML/JSON based configuration
- Global defaults with per-domain overrides
- Automatic hot-reload when files change
- Stage-level configuration for:
  - Sequential vs parallel execution
  - Approval quorum (any/all/majority)
  - SLA deadlines
  - Expiry actions

### Delegation Model
- Three delegation types: bulk, per-type, per-document
- Temporary delegation with date ranges
- Audit trail always shows original approver

### SLA & Escalation
- Business days = skip weekends only
- Configurable expiry actions per document type
- Multi-channel escalation: push, webhook, UI, email

### Version Management
- Hybrid storage: configurable N full versions + compressed older
- All diff formats: unified, side-by-side, inline
- In-place revert for restores
- User preference for diff display

## Key Technical Implications

1. **Config Watcher**: Need file system watcher for hot-reload
2. **Delegation Table**: New database table for delegation rules
3. **SLA Calculator**: Simple weekend-skipping logic
4. **Notification Service**: Multi-channel delivery
5. **Version Storage**: Hybrid with configurable threshold
6. **Diff Library**: `similar` crate with multiple output formats
