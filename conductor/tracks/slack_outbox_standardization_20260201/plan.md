# Plan: Slack Transactional Outbox Standardization & Manager Fallback

## Phase 1: Audit & Preparation [DONE]
- Identify direct Slack calls.
- Verify worker health.

## Phase 2: Refactor SlackAgent [DONE]
- Implement `@requires_session` decorator.
- Mandate `AsyncSession` for critical notifications.
- Remove direct-send escape hatches for business logic.

## Phase 3: Refactor Monitoring Jobs [DONE]
- Integrate Transactional Outbox into all scheduled alerts.

## Phase 4: Cleanup Slack Handlers [DONE]
- Unified notification logic in `handlers.py`.

## Phase 5: Verification & Testing [DONE]
- Unit tests for outbox and agent session hardening.

## Phase 6: Knowledge Management [DONE]
- Create `slack-outbox` skill.
- Document "Gold Standard" protocols.

## Phase 7: Operational Reliability & System Health [DONE]
- Admin System Health API.
- System Health Dashboard (Frontend).
- Failure Alerting (Infrastructure Alerts).

---

## Phase 8: Conversational Integrity & Manager Fallback [IN PROGRESS]

### 8.1: Conversational Guardrails (Fixing the "Eager LLM")
- Add `insider_info_asked` and `related_party_asked` to `DraftRequest` model.
- Update `before_tool_callback` to block `set_compliance_flags` if questions weren't physically delivered to Slack.
- Implement "Point-of-Send" state updates in `handlers.py` (only update flags on successful Slack delivery).
- Harden `set_compliance_flags` docstring with Negative Constraints.

### 8.2: Data Integrity & UI Payload Fix
- Update `show_preview` tool to include `is_leveraged` and `is_derivative` in the payload.
- Ensure the trade summary correctly reflects the "Yes" status for leveraged/derivative products.

### 8.3: Manager Fallback Infrastructure (Database)
- Create migration for `padealing.manager_override` (manual assignments).
- Create migration for `padealing.manager_resolution_failure` (diagnostic logs).
- Add `pending_manager_assignment` status to `PADRequest` workflow.

### 8.4: Identity Provider Chain of Responsibility
- Refactor `GoogleIdentityProvider` to use the fallback chain:
    1. Check Override Table (SQL).
    2. Query Google Admin API (Remote).
    3. Log failure to diagnostic table (SQL).
- Correct method call: `get_by_employee_id` -> `get_by_id`.

### 8.5: Revised Submission Workflow
- Update `_notify_manager_of_new_request` to handle lookup failures gracefully.
- Instead of rolling back the trade, transition to `pending_manager_assignment`.
- Queue a "Compliance Intervention" alert to the outbox when resolution fails.

### 8.6: Fallback UI & Operational visibility
- Add "Unassigned Managers" widget to Compliance Dashboard.
- Create "Assign Manager" modal to verify emails and trigger outbox notifications manually.
- Add "Failed Manager Lookups" table to System Health page.

### 8.7: Deep Integration Testing
- Create `tests/integration/test_submission_to_outbox_flow.py`.
- Verify atomic creation of Trade + Outbox records.
- Simulate Google API 403/NotFound errors and verify transition to fallback flow.
