# Stable Rollout Plan (Phased 10% → 100%)

> Purpose: Deliver NexusShell to all users in a controlled and measurable fashion, minimizing risk.

## 1. Prerequisites
- QA_PREVIEW_CHECKLIST at 100 % completion.
- Beta Pilot exit criteria satisfied.
- Release candidate tagged `rc-YY.MM.DD` and binaries signed.

## 2. Rollout Stages
| Stage | Audience | Percentage | Duration | Gate |
|-------|----------|------------|----------|------|
| A | Early Adopters | 10 % | 24 h | No blocking P0/P1 issues |
| B | Power Users | 30 % | 48 h | Error rate < 0.1 % |
| C | General Availability | 100 % | — | SLA metrics green |

Stages progress automatically via GitHub Actions Gates workflow:
- Monitors crash dumps, Prometheus metrics, user feedback volume.
- Auto-halts if thresholds exceeded; requires manual approval to resume.

## 3. Communication Plan
- Announce each phase in Slack `#nxsh-release` and mailing list.
- Public changelog and upgrade instructions posted on Confluence.

## 4. Monitoring
- Grafana dashboard `nxsh-rollout-overview` tracks adoption and error metrics.
- Sentry release health alerts for crash-free sessions < 99 %.

## 5. Rollback Strategy
- Maintain previous stable Docker tag `stable-prev` for instant downgrade.
- `nxsh update --rollback` command available to users.

## 6. Post-Rollout Review
- Collect metrics for 7 days post 100 %.
- Conduct retrospective meeting; document lessons learned. 