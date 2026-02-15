# Spec: Dashboard Summary Latency Fix

## Problem Statement

The `GET /dashboard/summary` endpoint is slow and contains a critical bug. This endpoint returns 6 count metrics displayed on the main dashboard. Every page load triggers this endpoint, making its latency directly visible to all users.

### Observed Issues

1. **BUG: 4 of 6 queries never executed** - `pending_count`, `breaches_count`, `execution_count`, `overdue_count` are referenced in the return statement but never assigned. Only `conflicts_count` and `unassigned_count` have `await session.scalar()` calls. This causes a `NameError` at runtime.
2. **Auth bottleneck on every request** - `get_current_user()` calls `identity.get_by_email()` which triggers fuzzy matching (fetches ALL employees sharing a surname initial, scores each with RapidFuzz) plus 3+ DB queries. The resolved `IdentityInfo` result is never cached across requests.
3. **`func.upper()` join bypasses indexes** - The mako_conflicts query joins `OraclePosition.inst_symbol` to `OracleBloomberg.ticker` via `func.upper()` on both sides, forcing full table scans despite existing B-tree indexes on both columns.
4. **Sequential query execution** - All 6 count queries run sequentially within a single `AsyncSession`. Since they are independent read-only counts, they could run in parallel.
5. **No summary-level caching** - The dashboard counts are identical for all users yet re-queried from the database on every dashboard load.
6. **Double session creation per request** - `get_current_user()` opens its own session, then the route handler opens another via `PADServiceDep`.

### Current Response Shape

```json
{
    "success": true,
    "message": null,
    "data": {
        "pending_approvals": 19,
        "active_breaches": 8,
        "pending_execution": 0,
        "overdue_execution": 0,
        "mako_conflicts": 1,
        "unassigned_managers": 0
    }
}
```

---

## Root Cause Analysis

### Issue 1: Unexecuted Queries (BUG)

**File**: `src/pa_dealing/services/pad_service.py:122-192`

Queries 1-4 are constructed as SQLAlchemy `select()` statements but never executed with `await session.scalar()`. The return dict references undefined variables (`pending_count`, `breaches_count`, `execution_count`, `overdue_count`).

### Issue 2: Auth Identity Resolution Not Cached

**File**: `src/pa_dealing/api/auth.py:107-109`, `src/pa_dealing/identity/provider_google.py:200-334`

Every request triggers:
- Google Admin API call (has in-memory TTL cache on singleton - OK)
- `find_best_match()` fuzzy matching (fetches all employees by surname initial from 3 joined tables, scores with RapidFuzz) - **NOT cached**
- `_get_sql_anchor_data()` - **NOT cached**
- `get_roles()` from employee_role table - **NOT cached**
- `is_manager()` Google API call (has TTL cache - OK)

The final `IdentityInfo` dataclass result is never cached. Same email resolves identically every time.

### Issue 3: func.upper() Bypassing Indexes

**File**: `src/pa_dealing/services/pad_service.py:168-171`

```python
.join(MakoPosition, func.upper(MakoPosition.inst_symbol) == func.upper(Security.ticker))
```

Both `oracle_position.inst_symbol` and `oracle_bloomberg.ticker` have standard B-tree indexes, but `UPPER()` wrapping prevents the planner from using them. PostgreSQL expression indexes on `UPPER(column)` would allow index scans.

Both tables are READ-ONLY reference tables in the `bo_airflow` schema, synced externally.

### Issue 4: Sequential Execution

**File**: `src/pa_dealing/services/pad_service.py:122-192`

All queries execute sequentially within one `AsyncSession`. Per SQLAlchemy docs, `asyncio.gather()` requires a separate `AsyncSession` per concurrent task (sessions are not thread/task-safe). The existing `get_session()` factory supports this pattern.

### Issue 5: No Summary Caching

Dashboard counts are not user-specific - all users see the same numbers. A short TTL cache (30-60s) on the summary dict would eliminate redundant queries when multiple users view the dashboard simultaneously.

### Issue 6: Datetime Inconsistency (Minor)

**File**: `src/pa_dealing/services/pad_service.py:141`

`datetime.now()` used without timezone, inconsistent with UTC-aware patterns used elsewhere in the codebase.

---

## Affected Files

| File | Lines | Issue |
|------|-------|-------|
| `src/pa_dealing/services/pad_service.py` | 122-192 | Bug + sequential queries + func.upper() |
| `src/pa_dealing/api/auth.py` | 107-109 | Uncached identity resolution |
| `src/pa_dealing/identity/provider_google.py` | 200-334 | get_by_email() result not cached |
| `src/pa_dealing/identity/fuzzy_matcher.py` | 179-215 | _fetch_candidates() runs every request |
| `src/pa_dealing/db/models/market.py` | 50-79 | OraclePosition missing UPPER index |

---

## Acceptance Criteria

### AC1: All 6 Summary Counts Returned Correctly
- [ ] `pending_approvals`, `active_breaches`, `pending_execution`, `overdue_execution` queries are executed and values assigned
- [ ] Return dict uses correct variable names matching executed queries
- [ ] Endpoint returns all 6 counts without errors

### AC2: Expression Indexes for Case-Insensitive Join
- [ ] Alembic migration creates `UPPER(inst_symbol)` index on `bo_airflow.oracle_position`
- [ ] Alembic migration creates `UPPER(ticker)` index on `bo_airflow.oracle_bloomberg`
- [ ] mako_conflicts query uses index scans (verifiable via EXPLAIN)

### AC3: Parallel Query Execution
- [ ] All 6 count queries run concurrently via `asyncio.gather()`
- [ ] Each concurrent query uses its own `AsyncSession` (per SQLAlchemy requirements)
- [ ] Results are identical to sequential execution

### AC4: Identity Resolution Cached Across Requests
- [ ] `get_by_email()` result cached with TTL (default 300s)
- [ ] Cache keyed on email address
- [ ] Second request for same email within TTL returns cached result without DB queries
- [ ] Cache uses manual TTL dict pattern (consistent with existing `GoogleAdminClient` caching)

### AC5: Summary Counts Cached
- [ ] Dashboard summary dict cached with short TTL (30s)
- [ ] All users receive same cached result within TTL window
- [ ] Cache automatically expires and refreshes

### AC6: No Regressions
- [ ] All existing tests pass
- [ ] New unit tests cover the bug fix (all 6 counts)
- [ ] New unit tests verify caching behavior (hit/miss/expiry)
- [ ] New unit tests verify parallel execution produces correct results

---

## Technical Approach

### Caching Strategy
- **Identity cache**: Manual TTL dict on `GoogleIdentityProvider` (matches existing `_user_cache` pattern in `google.py:28`). Key: email, Value: `(IdentityInfo, timestamp)`, TTL: 300s.
- **Summary cache**: Module-level TTL dict in `pad_service.py`. Key: `"summary"`, Value: `(dict, timestamp)`, TTL: 30s.
- No new pip dependencies required - use the same manual pattern already in the codebase.

### Parallel Query Strategy
- Use `asyncio.gather()` with separate `get_session()` contexts per query
- Each coroutine opens its own `AsyncSession` via the existing `get_session()` factory
- Connection pool (size=10, overflow=20) has capacity for 6 concurrent connections

### Index Strategy
- Expression indexes via Alembic migration using `op.create_index(..., [text("UPPER(column)")])`
- Both target tables are READ-ONLY in `bo_airflow` schema
- Indexes are additive and non-destructive

---

## Out of Scope

- Redis/distributed caching (future enhancement)
- Frontend changes (covered by prior `performance_optimization_20260203` track)
- Refactoring the fuzzy matcher algorithm itself
- Changing the auth flow to skip fuzzy matching (risky - affects identity resolution correctness)
- Connection pool tuning beyond current settings

---

## References

- [SQLAlchemy AsyncSession concurrency](https://github.com/sqlalchemy/sqlalchemy/discussions/9312) - one session per gather task
- [PostgreSQL expression indexes](https://www.postgresql.org/docs/current/indexes-expressional.html)
- [Alembic expression indexes](https://alembic.sqlalchemy.org/en/latest/ops.html)
- [cachetools TTLCache patterns](https://cachetools.readthedocs.io/)
