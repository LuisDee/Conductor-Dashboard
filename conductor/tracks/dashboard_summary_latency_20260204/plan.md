# Implementation Plan: Dashboard Summary Latency Fix

## Phase 1: Fix the Bug (Queries 1-4 Not Executed)

- [x] Write test: unit test for `get_dashboard_summary_counts` verifying all 6 counts are returned with correct values
- [x] Fix `pad_service.py:128-183`: add `await session.scalar()` for `pending_query`, `breaches_query`, `execution_query`, `overdue_query` and assign to `pending_count`, `breaches_count`, `execution_count`, `overdue_count`
- [x] Fix `pad_service.py:141`: replace `datetime.now()` with UTC-consistent pattern
- [x] Lint & format: `ruff check --fix . && ruff format .`
- [x] Verify: run unit tests to confirm all 6 counts returned correctly
- [x] Commit: bug fix for unexecuted summary queries
- [x] Conductor - User Manual Verification 'Phase 1: Fix the Bug' (Protocol in workflow.md)

## Phase 2: Add Expression Indexes via Alembic Migration

- [x] Write Alembic migration creating expression index `ix_oracle_bloomberg_ticker_upper` on `bo_airflow.oracle_bloomberg (UPPER(ticker))`
- [x] Write Alembic migration creating expression index `ix_oracle_position_inst_symbol_upper` on `bo_airflow.oracle_position (UPPER(inst_symbol))`
- [ ] Run migration against dev database and verify indexes created
- [ ] Verify via `EXPLAIN ANALYZE` that the mako_conflicts query uses index scans
- [x] Lint & format
- [ ] Commit: Alembic migration for expression indexes
- [ ] Conductor - User Manual Verification 'Phase 2: Expression Indexes' (Protocol in workflow.md)

## Phase 3: Parallelize Summary Queries

- [x] Write test: unit test verifying parallel execution returns identical results to sequential
- [x] Refactor `get_dashboard_summary_counts` to use `asyncio.gather()` with separate `get_session()` per query
- [x] Extract each count query into its own async helper function (each opens its own session)
- [x] Gather all 6 results concurrently
- [x] Lint & format
- [x] Verify: run unit tests
- [ ] Commit: parallelize dashboard summary queries
- [ ] Conductor - User Manual Verification 'Phase 3: Parallel Queries' (Protocol in workflow.md)

## Phase 4: Cache Identity Resolution (get_by_email)

- [x] Write test: unit test verifying cached `get_by_email` returns same result without DB queries on second call
- [x] Write test: unit test verifying cache expires after TTL
- [x] Add `_identity_cache: ClassVar[dict[str, tuple[IdentityInfo | None, float]]]` to `GoogleIdentityProvider`
- [x] Add TTL check at top of `get_by_email()`: if email in cache and not expired, return cached result
- [x] Store resolved `IdentityInfo` in cache before returning
- [x] Default TTL: 300 seconds (5 minutes)
- [x] Lint & format
- [x] Verify: run unit tests
- [ ] Commit: add TTL cache for identity resolution
- [ ] Conductor - User Manual Verification 'Phase 4: Identity Cache' (Protocol in workflow.md)

## Phase 5: Cache Dashboard Summary Counts

- [x] Write test: unit test verifying summary cache returns same dict within TTL
- [x] Write test: unit test verifying cache expires and refreshes after TTL
- [x] Add module-level `_summary_cache: dict[str, tuple[dict, float]]` in `pad_service.py`
- [x] At top of `get_dashboard_summary_counts()`: check cache, return if valid
- [x] After computing results: store in cache with timestamp
- [x] Default TTL: 30 seconds
- [x] Lint & format
- [x] Verify: run unit tests
- [ ] Commit: parallelize + cache dashboard summary
- [ ] Conductor - User Manual Verification 'Phase 5: Summary Cache' (Protocol in workflow.md)

## Phase 6: Regression Testing & Verification

- [x] Run full unit test suite — 871 passed, 11 pre-existing failures (DB connection / notification formatting), 0 new failures
- [ ] Verify dashboard endpoint via curl against running dev server
- [ ] Verify all 6 counts return correct non-zero values where expected
- [ ] Verify response time improvement (target: <200ms cached, <500ms uncached)
- [ ] Commit: any test fixes or adjustments
- [ ] Conductor - User Manual Verification 'Phase 6: Regression Testing' (Protocol in workflow.md)
