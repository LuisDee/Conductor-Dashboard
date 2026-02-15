# Spec Compliance Gaps - Status

## Track Information
- **Created**: 2024-12-24
- **Status**: PLANNING COMPLETE
- **Priority**: HIGH (regulatory compliance)

## Problem Summary

Cross-reference of PAD spec vs implementation identified gaps in:
1. Audit retention (90-day cleanup vs 7-year requirement)
2. Accuracy metrics (no data recorded for dashboard)
3. Response format (missing fields)
4. Response timing (not measured)
5. ~~Uptime monitoring~~ (out of scope - cloud platform handles)
6. Insider information (not enforced in code)
7. MAR/MiFID rules (incomplete)

## Work Items

| Gap | Description | Priority | Status |
|-----|-------------|----------|--------|
| 1 | Enforce 7-year audit retention | P0 | Not Started |
| 2 | Record accuracy metrics for dashboard | P1 | Not Started |
| 3 | Add `recommendation` + `suggested_approver` to response | P2 | Not Started |
| 4 | Add response time logging | P2 | Not Started |
| 6 | Insider information enforcement | P1 | Not Started |
| 7 | MAR/MiFID compliance detection | P1 | Not Started |

## Implementation Order

1. **Gap 1** - Quick win, regulatory critical (30 min)
2. **Gap 6** - Important compliance requirement (2-3 hours)
3. **Gap 7** - Significant new functionality (4-6 hours)
4. **Gap 2** - Enables dashboard metrics (2-3 hours)
5. **Gap 3** - Response format cleanup (1 hour)
6. **Gap 4** - Timing instrumentation (1 hour)

## Files to Create/Modify

### New Files
- `src/pa_dealing/api/middleware.py` - Response timing middleware
- `dashboard/src/pages/AccuracyMetrics.tsx` - Accuracy dashboard
- `alembic/versions/YYYYMMDD_spec_compliance_gaps.py` - Migration

### Modified Files
- `src/pa_dealing/agents/monitoring/jobs.py` - Remove 90-day audit cleanup
- `src/pa_dealing/db/models.py` - Add `ComplianceDecisionOutcome`, insider fields
- `src/pa_dealing/agents/orchestrator/agent.py` - Record decisions, timing
- `src/pa_dealing/agents/orchestrator/schemas.py` - New response types
- `src/pa_dealing/agents/orchestrator/policy_engine.py` - MAR detector
- `src/pa_dealing/agents/slack/handlers.py` - Insider info checkbox
- `src/pa_dealing/agents/slack/chatbot.py` - Insider info confirmation
- `src/pa_dealing/api/routes/requests.py` - Insider info validation
- `src/pa_dealing/api/routes/dashboard.py` - Accuracy metrics endpoint
- `src/pa_dealing/api/main.py` - Add timing middleware

## Success Criteria

- [x] Audit logs no longer deleted at 90 days
- [x] `ComplianceDecisionOutcome` records created for every request
- [x] `/api/accuracy-metrics` returns prohibited/holding/match rates
- [x] Insider information checkbox required in Slack modal
- [x] Requests blocked without insider info confirmation
- [x] MAR checks run on every request
- [x] HIGH severity MAR flags trigger SMF16 escalation
- [x] Response includes `recommendation` and `suggested_approver`
- [x] Response time logged in milliseconds

## Notes

- **Uptime monitoring** is explicitly out of scope - handled by cloud platform/k8s
- **Accuracy "tests"** are NOT the goal - we record data for compliance dashboard
- **Response time alerting** deferred - just log for now
- **7-year retention** is hardcoded, not configurable (regulatory requirement)

See `plan.md` for detailed implementation instructions.
