# Incident Response Runbook

> This document defines the standardized procedure for detecting, triaging, and resolving production incidents in NexusShell.

## 1. Classification Matrix
| Severity | Impact | Example |
|----------|--------|---------|
| P0 | Total service outage / data loss | All downloads return 5xx |
| P1 | Major functionality degraded | Build pipeline failure |
| P2 | Minor bug, workaround exists | UI glitch |

## 2. Response Flow
1. **Detection** – Alert from monitoring or user report.
2. **Acknowledgement** – On-call engineer acknowledges within 5 minutes.
3. **Containment** – Stop further impact (e.g., disable feature flag).
4. **Root-Cause Analysis** – Collect logs, metrics, heap dumps.
5. **Remediation** – Deploy hotfix or rollback.
6. **Communication** – Post interim updates every 30 minutes in #status.
7. **Post-Mortem** – Within 24 hours, write a detailed report.

## 3. Checklists
### 3.1 On-call Engineer
- [ ] Confirm alert validity.
- [ ] Create incident channel and ticket.
- [ ] Page secondary engineers if unresolved after 15 minutes.

### 3.2 Incident Commander
- [ ] Assign roles (scribe, comms).
- [ ] Approve remediation plan.
- [ ] Close incident after validating fix.

## 4. Tooling
- **PagerDuty**: On-call scheduling and escalation.
- **Grafana / Prometheus**: Metrics & alerting.
- **Sentry**: Crash reporting.

## 5. References
- Security policy – `docs/SECURITY.md`
- SLA targets – `monitoring/grafana/nxsh_sla.json` 