# Beta External Pilot Plan (30 Internal Users)

> Objective: Validate NexusShell in real-world daily workflows prior to public release.

## 1. Participant Selection
- Total users: 30 (Engineering 15, DevOps 5, Data 5, Support 5).
- Diversity: Windows/macOS/Linux mix; power users & novices included.
- NDA: All participants sign updated confidentiality agreement.

## 2. Distribution
- Channel: Private GitHub release with invite-only access token.
- Deliverables: Installers (MSI, PKG, DEB), Docker image, full docs.
- Support: Dedicated Slack channel `#nxsh-beta` and PagerDuty schedule.

## 3. Timeline
| Phase | Dates | Milestone |
|-------|-------|-----------|
| Kick-off | Day 0 | Webinar, onboarding guide |
| Week 1 | Day 1–7 | Daily standups, feedback form V1 |
| Midterm | Day 14 | Survey, bug triage sprint |
| Wrap-up | Day 30 | Exit survey, pilot report |

## 4. Success Metrics
- ≥ 20 active users per week.
- ≥ 1 hr average daily usage.
- < 0.5 critical bug reports per user.
- Net Promoter Score ≥ +30.

## 5. Feedback Channels
- GitHub Issues with `beta` label.
- Anonymous Typeform survey (weekly).
- Live Q&A session every Friday.

## 6. Incentives
- Swag pack: T-shirts, stickers.
- Credit in release notes.

## 7. Risk Management
- Rapid hotfix pipeline (< 24 h turnaround).
- Rollback script for configuration migration.

## 8. Exit Criteria
- All P0/P1 issues resolved.
- ≥ 90 % checklist completion in QA_PREVIEW_CHECKLIST.
- Positive trend in NPS week-over-week. 