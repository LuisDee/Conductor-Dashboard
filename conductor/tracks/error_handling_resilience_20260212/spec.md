# Spec: Error Handling & Resilience Hardening

## Problem Statement
Multiple error handling anti-patterns cause cascading failures, type safety violations, credential leaking, and silent data loss. The autopsy identified 67 error-handling findings; this track addresses the HIGH and CRITICAL subset.

## Source
- `.autopsy/REVIEW_REPORT.md` - HIGH: Missing Error Handling (45 findings)
- `.autopsy/ARCHITECTURE_REPORT.md` - Quality Attribute: Reliability

## Findings (Verified Against Code)

### 1. asyncio.gather() Without return_exceptions (CRITICAL)
- **File:** `services/pad_service.py` lines 193-206
- **Issue:** 12 parallel queries in `asyncio.gather()` without `return_exceptions=True`. If ANY query fails, all in-flight tasks cancelled and dashboard summary crashes.
- **Fix:** Add `return_exceptions=True`, handle individual failures gracefully

### 2. API Error Handler Returns `never` (HIGH)
- **File:** `dashboard/src/api/client.ts` lines 60-66
- **Issue:** `handleError` typed as `: never` but used in 30+ catch blocks that expect data return. TypeScript type safety violated.
- **Fix:** Update return type or restructure error handling pattern

### 3. GraphClient Credential Leaking in Error Logs (HIGH)
- **File:** `services/graph_client.py` line 712 + line 540
- **Issue:** `log.error("graph_api_error", error=str(error))` may include Azure `client_secret` in error string. Also `raise SubscriptionError(f"Failed to create subscription: {e}")`.
- **Fix:** Sanitize error messages before logging. Strip sensitive fields from exception strings.

### 4. Unprotected flush() Calls (HIGH)
- **Files:** `services/restricted_instruments.py` line 69, `services/pad_service.py` lines 654/1315, `services/trade_document_processor.py` lines 291/440
- **Issue:** `session.flush()` called without try-except. If flush fails (constraint violation, timeout), exception propagates unhandled, partial transaction state.
- **Fix:** Wrap flush calls in try-except with proper rollback or let session context manager handle it

### 5. Silent Failures - Errors Logged But Not Raised (HIGH)
- **File:** `services/trade_document_processor.py` lines 442-444
- **Issue:** Exception caught, logged as warning, appended to errors list but not raised. Function returns normally with hidden errors.
- **File:** `services/email_ingestion_worker.py` lines 125-137
- **Issue:** Exception logged but function returns `ProcessingResult(status="failed")` instead of raising.
- **Fix:** Either raise exceptions or ensure callers check error state

### 6. Bare except:pass Blocks (MEDIUM)
- **File:** `services/pdf_poller.py` lines 337-338
- **Issue:** `except Exception: pass` swallows all errors during PDF archival to failed bucket. No logging, no metrics.
- **Fix:** Add at minimum error logging

## Acceptance Criteria
- [ ] Dashboard summary handles individual query failures gracefully
- [ ] API error handler has correct TypeScript types
- [ ] No credentials appear in log output or exception messages
- [ ] flush() failures handled with proper transaction management
- [ ] Silent failure patterns either raise or have callers check errors
- [ ] Bare except blocks at minimum log the error
- [ ] All existing tests pass + new tests for error scenarios
