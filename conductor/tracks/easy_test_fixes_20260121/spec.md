# Easy Test Fixes - Schema Consolidation Follow-up

## Overview
Fix 14 easy test failures remaining after database schema consolidation. All are straightforward fixes with clear solutions.

## Problem Statement
After consolidating to dual-schema architecture, 14 tests are failing with simple, fixable issues:
- 11 instrument lookup tests missing database fixture dependency
- 1 Google identity test with mock exhaustion
- 1 auto-approve test (intermittent Slack mock connection)
- 1 unauthorized access test (Slack mock connection)

## Functional Requirements

### FR1: Fix Instrument Lookup Tests (11 tests)
**Tests:** All tests in `tests/test_instrument_lookup.py`
**Error:** `relation "bo_airflow.oracle_bloomberg" does not exist`
**Root Cause:** Tests use `get_session()` directly without fixtures, so tables never get created
**Fix:** Add `session` parameter to all test functions to trigger fixture chain

### FR2: Fix Google Identity Mock Exhaustion (1 test)
**Test:** `test_is_manager_of_uses_google`
**Error:** `StopAsyncIteration`
**Root Cause:** Mock `side_effect` list exhausted when code calls more times than expected
**Fix:** Add more return values to `mock_google_client.is_manager.side_effect`

### FR3: Fix Slack Mock Connection Tests (2 tests)
**Tests:** `test_auto_approve_flow`, `test_unauthorized_manager_approval`
**Error:** `Cannot connect to host localhost:18888`
**Root Cause:** Tests run when Slack mock not started, or port mismatch
**Fix:** Verify Slack mock is running, check port configuration

## Acceptance Criteria
1. [x] All 11 instrument lookup tests pass
2. [x] test_is_manager_of_uses_google passes
3. [x] Slack mock connection tests pass (or skip if infrastructure issue)
4. [x] No new test failures introduced
5. [x] Test count: 359 + 14 = 373 passing

## Out of Scope
- Medium complexity tests (E2E API, audit 403 errors)
- Hard complexity tests (timeouts, complex orchestrator flows)
- Database seeding for Docker environment
