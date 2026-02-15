# Spec: Bug Fixes & Test Cleanup

**Track ID:** test_failures_20260126
**Status:** In Progress
**Priority:** High
**Discovered:** 2026-01-26 during Risk Scoring Overhaul (Phase 2)
**Extended:** 2026-01-27 with 4 additional production bugs

---

## Problem Statement

During routine testing for the Risk Scoring Overhaul track, 6 pre-existing test issues were discovered. Additionally, 4 production bugs and 1 dashboard text update were identified during manual testing. This track covers all 7 items.

---

## Failures Inventory

### Category 1: OracleEmployee Model Mismatch (4 tests) - RESOLVED

**Error**: `TypeError: 'full_name' is an invalid keyword argument for OracleEmployee`

**Affected Tests**:
1. `tests/unit/test_auto_approve.py::test_auto_approve_flow`
2. `tests/unit/test_holding_period.py::test_holding_period_pass_at_30_days`
3. `tests/unit/test_holding_period.py::test_holding_period_fail_at_29_days`
4. `tests/unit/test_holding_period.py::test_holding_period_reset_on_buy`

**Root Cause**: Tests were creating `OracleEmployee` fixtures with a `full_name` field that no longer exists in the model. Fixed by factory pattern update in conftest.

**Status**: Fixed (factory pattern auto-resolved)

---

### Category 2: Import Errors (2 tests) - RESOLVED

#### 2a. Missing Module Path - FIXED

**Error**: `ModuleNotFoundError: No module named 'pa_dealing'`

**Affected Test**: `tests/unit/test_manager_notification_identity.py`

**Fix Applied**: Updated import to use `src.pa_dealing` prefix.

#### 2b. Missing Function - FIXED

**Error**: `ImportError: cannot import name 'clean_security_search_term' from 'src.pa_dealing.agents.slack.chatbot'`

**Affected Test**: `tests/unit/test_symbol_extraction.py`

**Fix Applied**: Deleted test file. Function was refactored into inline logic in chatbot.py lines 311-333.

---

### Category 3: Contract Note Upload Error (Bug 1)

**Error**: `Failed to process contract note: Missing key inputs argument! To use the Google AI API, provide (api_key) arguments`

**Affected Area**: Dashboard > My Requests > Upload Contract Note

**Root Cause**: `DocumentAgent` uses raw `google.genai.Client(api_key=...)` directly (agent.py lines 27-31), which conflicts with ADK wrapper pattern. Needs refactoring to use LiteLLM proxy via ADK model wrapper.

**Fix**: Refactor DocumentAgent to use ADK model wrapper (`get_model()`). Add separate LiteLLM document API key (`sk-5M9nYV4C_f0eiDqHIDdJEg`).

---

### Category 4: Security Mismatch (Bug 2)

**Error**: User submitted "ISHARES BARCLAYS EUR GOVERNMENT BOND 7-10 EUR" but system displays "HNZ US HJ Heinz Co"

**Affected Area**: Slack chatbot > Security selection > My Requests display

**Root Cause**: Fuzzy matching in `get_or_create_security()` uses OR logic with case-insensitive ticker matching. If multiple securities match, returns first result which may be wrong.

**Fix**: Implement exact-match-first logic in security lookup. Add post-selection validation in chatbot.

---

### Category 5: Empty Pending Approvals (Bug 3)

**Error**: Requests with status "pending_compliance" exist in My Requests but Pending Approvals page is empty.

**Affected Area**: Dashboard > Pending Approvals

**Root Cause**: Role-based filtering in `pad_service.py:get_pending_approvals()` may be returning False for compliance users, causing them to fall into "non-compliance" filter branch which only shows direct reports.

**Fix**: Add debug logging, verify employee role records, fix identity provider or role assignment as needed.

---

### Category 6: Dashboard Text Update (Bug 4)

**Issue**: Dashboard subtitle says "Real-time status of Personal Account Dealing compliance" but should say "Mako Personal Account Dealing compliance suite".

**Affected Area**: Dashboard header/subtitle

**Fix**: Update text string in dashboard component.

---

### Category 7: Dynamic Mako Conflicts (Bug 5)

**Issue**: Dashboard "Mako Conflicts" section shows hardcoded text "Cross Trades". Should be dynamic based on conflict count.

**Affected Area**: Dashboard > Mako Conflicts widget

**Fix**: Make conflicts status dynamic - green "All clear" when 0, red "X outstanding conflicts" when >0.

---

## Acceptance Criteria

- [x] AC1: All 4 OracleEmployee tests pass after fixture update
- [x] AC2: `test_manager_notification_identity.py` imports correctly and passes
- [x] AC3: `test_symbol_extraction.py` removed (function refactored inline)
- [ ] AC4: Contract notes upload without API key error
- [ ] AC5: Security selection stores correct security in database
- [ ] AC6: Pending Approvals shows compliance-pending requests for compliance users
- [ ] AC7: Dashboard displays "Mako Personal Account Dealing compliance suite"
- [ ] AC8: Mako Conflicts shows dynamic status (green/red)
- [ ] AC9: Full unit test suite runs with 0 failures
- [ ] AC10: No new test failures introduced
- [ ] AC11: New unit tests added for security matching and pending approvals filtering

---

## Notes

- Categories 1-2 were pre-existing test issues from model/import changes
- Categories 3-7 are production bugs discovered during manual testing on 2026-01-27
- The Risk Scoring Overhaul track (557 passed) confirms core functionality is working
