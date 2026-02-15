# Spec: Stale Test Fixes (16 Pre-existing Failures)

## Type
Bug Fix (Test Maintenance)

## Overview
16 unit tests are failing due to UI code changes that were not reflected in their corresponding tests. These span 3 categories:
1. **Slack Block Kit UI tests** (12 failures) — tests hard-code block indices and button counts that are now stale after SMF16 escalation, advisory warnings, manager comments input, and risk factor blocks were added.
2. **Infrastructure-dependent tests** (3 failures) — tests require the `slack-mock` container running but don't gracefully skip when it's unavailable.
3. **DB session isolation** (1 failure) — test queries a different session than the handler writes to.

## Root Cause Analysis

### Category A: Compliance View (2 failures)
**File:** `tests/unit/test_compliance_view.py`
- `build_compliance_compact_blocks()` in `ui.py` now returns **4 buttons** (Approve, Decline, **Escalate to SMF16**, Dashboard) but test expects 3.
- Dashboard button index shifted from `[2]` to `[3]`.

### Category B: Manager Approval View (8 failures)
**File:** `tests/unit/test_manager_approval_view.py` + `test_manager_notification_redesign.py`
- `build_manager_approval_compact_blocks()` in `ui.py` now includes:
  - Advisory warning blocks (conditional, inserted after header)
  - Risk factor breakdown blocks (conditional)
  - Manager comments `input` block (always present)
- Tests use **hard-coded block indices** (e.g., `blocks[7]`) instead of searching by block type.
- Footer block with "⏰ Must execute within 2 business days" is expected by tests but **not implemented** in `ui.py`.

### Category C: Infrastructure (3 failures)
**Files:** `test_auto_approve.py`, `test_unauthorized_access.py`
- Tests call `wait_for_message()` against `slack-mock` container.
- When `slack-mock` is not running: `ClientConnectorDNSError` / `ConnectError`.

### Category D: DB Session (1 failure)
**File:** `test_insider_info.py`
- `test_checked_insider_checkbox_allows_request` writes via handler (one session) then queries via test fixture session.
- Transaction isolation prevents the test session from seeing uncommitted data.

## Acceptance Criteria
- All 16 tests pass when run with `docker compose up -d db`
- Infrastructure-dependent tests (Category C) are skipped gracefully when slack-mock is not available
- No hard-coded block indices in UI tests — use type-based block lookup
- Footer block added to `build_manager_approval_compact_blocks()`
- Compliance view tests updated for 4-button layout (including SMF16 escalation)
- DB session isolation fixed for insider info test

## Out of Scope
- Changing the actual UI behavior (buttons, block structure) — only fix tests to match current UI
- E2E Playwright tests
- conductor_dashboard import errors (separate module not in Docker image)
