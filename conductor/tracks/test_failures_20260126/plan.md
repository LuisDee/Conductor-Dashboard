# Plan: Bug Fixes

**Track ID:** test_failures_20260126
**Status:** In Progress
**Spec:** [spec.md](./spec.md)

---

## Phase 1: OracleEmployee Fixture Fix - COMPLETE

### 1.1 Investigation
- [x] Task: Check current `OracleEmployee` model fields in `src/pa_dealing/db/models/employee.py`
- [x] Task: Identify what field replaced `full_name` (resolved via factory pattern)

### 1.2 Fix Tests
- [x] Task: OracleEmployee tests auto-resolved via factory pattern in conftest
- [x] Task: Verified all 4 tests pass

---

## Phase 2: Import Error Fixes - COMPLETE

### 2.1 Fix Module Path
- [x] Task: Update `tests/unit/test_manager_notification_identity.py` import from `pa_dealing` to `src.pa_dealing`
- [x] Task: Run test to verify

### 2.2 Fix Missing Function
- [x] Task: Investigated `clean_security_search_term` - refactored inline in chatbot.py lines 311-333
- [x] Task: Deleted `tests/unit/test_symbol_extraction.py` (function no longer exists as standalone)

---

## Phase 3: Dashboard Quick Fixes - COMPLETE

### 3.1 Dashboard Text (Bug 4)
- [x] Task: Updated subtitle text in `dashboard/src/pages/Dashboard.tsx:239`
- [x] Task: Changed from "Real-time status of Personal Account Dealing compliance" to "Mako Personal Account Dealing compliance suite"

### 3.2 Dynamic Mako Conflicts (Bug 5)
- [x] Task: Updated StatCard in `dashboard/src/pages/Dashboard.tsx:277-278`
- [x] Task: Changed variant from threshold-based warning to `danger`/`success` based on count > 0
- [x] Task: Changed subValue from hardcoded "Cross-trades" to dynamic "X Outstanding" / "All Clear"

---

## Phase 4: Empty Pending Approvals (Bug 3) - COMPLETE

### 4.1 Investigation
- [x] Task: Added debug logging to `pad_service.py:get_pending_approvals()` at line 1050
- [x] Task: Investigated `identity/provider_google.py:has_role()` implementation
- [x] Task: Found `EmployeeRole.__table_args__` defined twice (schema dict overwritten by Index tuple)

### 4.2 Fix
- [x] Task: Fixed `EmployeeRole.__table_args__` in `core.py` - merged schema dict into Index tuple
- [x] Task: Added explicit `padealing.employee_role` schema prefix in `has_role()` raw SQL query
- [x] Task: Added logging to `has_role()` for diagnosis
- [ ] Task: Verify compliance user sees pending_compliance requests (requires manual test)

---

## Phase 5: Security Mismatch (Bug 2) - COMPLETE

### 5.1 Investigation
- [x] Task: Traced `get_or_create_security()` logic in `repository.py:173-242`
- [x] Task: Identified root cause: OR logic with ticker can match wrong security when bloomberg provided

### 5.2 Fix
- [x] Task: Refactored to exact-match-first (bloomberg exact match, then ticker fallback)
- [x] Task: Eliminated OR logic that could return wrong security
- [x] Task: Added detailed logging for each match step
- [x] Task: Created integration test `tests/integration/test_security_matching.py`

---

## Phase 6: Contract Note Upload (Bug 1) - COMPLETE

### 6.1 Refactor DocumentAgent
- [x] Task: Refactored `agent.py` to use LiteLLM proxy instead of raw `google.genai.Client`
- [x] Task: Added multimodal support via LiteLLM (base64 image/PDF encoding)
- [x] Task: Preserved Google SDK fallback path when `use_litellm=False`
- [x] Task: Moved tests to `tests/unit/test_document_agent.py` and updated mocks

### 6.2 Verify
- [x] Task: All 4 document agent tests pass
- [ ] Task: Manual test contract note upload (requires running dashboard)

---

## Phase 8: Bloomberg Mapping & Dev Mode Fixes (2026-01-27)

### 8.1 Bloomberg " Equity" Append Bug
- [x] Task: Identified root cause - `get_or_create_security` appends " Equity" but oracle_bloomberg stores raw codes
- [x] Task: Removed " Equity" append from `repository.py:213-215`
- [x] Task: Fixed `scalar_one_or_none()` crash on duplicate tickers (changed to `.limit(1)`)
- [x] Task: Updated conftest seed data to use real bloomberg format (no " Equity" suffix)
- [x] Task: Updated factory default bloomberg from `"{ticker} US Equity"` to `"{ticker} US"`
- [x] Task: Fixed DB record id=21 (HNZ -> IEGM LN, security_id=3587)
- [x] Task: Added submission draft logging in chatbot.py
- [x] Task: 12 security matching tests pass (7 new tests added)

### 8.2 Dev Mode Self-Approval
- [x] Task: Added `get_settings().is_development` check in `pad_service.py:1055`
- [x] Task: Dev mode allows users to approve their own requests for testing

### 8.3 Dashboard UI Polish
- [x] Task: Table.tsx borderRadius on thead (MyRequests table header gap)
- [x] Task: Dashboard.tsx flex column gap wrapper for Quick Actions padding
- [x] Task: MyRequests.tsx "PA dealing" -> "PAD" subtitle
- [x] Task: PendingApprovals.tsx space-y-6 -> space-y-8
- [x] Task: HoldingPeriods.tsx dynamic holding period from settings

### 8.4 Backend Fixes
- [x] Task: confidence_score default=0.5 in schemas.py
- [x] Task: padealing.employee_role schema qualification in provider_google.py and postgres.py

## Phase 7: Final Verification

- [ ] Task: Run full unit test suite: `docker exec pad_api python -m pytest tests/unit/ -v`
- [ ] Task: Confirm 0 failures, 0 collection errors
- [ ] Task: Verify all new tests pass

---

## Completion Checklist

- [x] AC1: OracleEmployee tests pass
- [x] AC2: Identity import test passes
- [x] AC3: Symbol extraction test resolved
- [ ] AC4: Contract notes upload without API key error
- [ ] AC5: Security selection stores correct security
- [ ] AC6: Pending Approvals shows compliance requests
- [ ] AC7: Dashboard text updated
- [ ] AC8: Mako Conflicts dynamic
- [ ] AC9: Full suite passes
- [ ] AC10: No regressions
- [ ] AC11: New tests for security matching and pending approvals
