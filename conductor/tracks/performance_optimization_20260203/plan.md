# Implementation Plan: API & Frontend Performance Optimization

## Phase 1: Auth Caching & Quick Wins (Highest Impact) ✅
- [x] Remove duplicate `is_manager()` call in `src/pa_dealing/api/auth.py`.
- [x] Ensure `provider_google.py` uses the existing `_user_cache` for `lookup_user()`.
- [x] Implement caching for `is_manager()` calls (1-hour TTL).
- [x] Skip full manager resolution in `get_by_email` during the auth flow (only need boolean).
- [x] Unify frontend auth query key to `['current-user']`.
- [x] Fix dev mode `staleTime` to 30s in `dashboard/src/`.

## Phase 2: API Route Optimization ✅
- [x] Fix N+1 query in `/documents` endpoint (grouped query with LEFT JOIN).
- [x] Consolidate sequential stats counts into a single GROUP BY query.
- [x] Implement count-only service methods for dashboard summary.
- [x] Eliminate duplicate identity provider lookups in endpoints.

## Phase 3: Database & ORM Tuning ✅
- [x] Add missing indexes: `notification_outbox(status, next_attempt_at)`, `pad_request(created_at)`, `parsed_trade(match_status, request_id)`.
- [x] Configure `pool_recycle=3600` in engine settings.

## Phase 4: Frontend Flow Optimization ✅
- [x] Prefetch employee list at app root level (ProtectedRoute).
- [x] Parallelize auth, employees, and main data fetching.

## Phase 5: Verification & Rigorous Testing ✅

- [x] Update unit tests to include required fields for `ExtractionRouter` (`tests/unit/test_extraction_router.py`).

- [x] Fix fixture cleanup in `tests/unit/test_google_identity_provider.py` to handle new tables and constraints.

- [x] Verified `fast` tests (smoke + unit) all PASS.

- [x] Implemented heartbeat mechanism in `test-runner.sh` for better visibility.

- [x] Fixed dashboard summary crash due to `OraclePositionUsage` typo.

- [x] Reverted `pool_recycle` to fix `pytest-asyncio` loop attachment issues.

- [x] Verified end-to-end functionality via `curl` on local host and remote DB.
