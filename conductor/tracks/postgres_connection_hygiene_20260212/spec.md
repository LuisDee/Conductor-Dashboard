# Specification: PostgreSQL Connection Hygiene Remediation

## Goal
Eliminate database connection leaks, ensure proper resource cleanup on shutdown, and improve observability by identifying connections in `pg_stat_activity`.

## Problem Statement
The application currently suffers from:
1.  **Connection Leaks:** Connections remain open after application shutdown and between test runs.
2.  **Lack of Observability:** Connections appear as "unknown" in database monitoring tools because `application_name` is not set.
3.  **Hardcoded Configuration:** Pool settings (size, overflow) are hardcoded, preventing tuning without code changes.

## Requirements

### 1. Proper Shutdown
- The `dispose_engine()` method must be called explicitly when the FastAPI application shuts down (`lifespan` handler).
- Background workers (email poller, scheduler) must ensure they release their DB resources on stop.

### 2. Test Isolation
- The `reset_engine()` helper used in tests must explicitly **dispose** of the old engine before creating a new one, preventing connection pile-up during test suite execution.

### 3. Connection Identification
- All database connections must set `application_name` in `connect_args`.
- Format: `pa-dealing-<service>` (e.g., `pa-dealing-api`, `pa-dealing-worker`).

### 4. Configuration
- Expose `DB_POOL_SIZE` and `DB_MAX_OVERFLOW` as environment variables in `Settings`.
- Set sensible defaults (Pool: 5, Overflow: 10).
- Enable `pool_recycle` (1800s) to prevent using stale connections closed by firewalls.

## Deliverables
- [ ] Refactored `src/pa_dealing/db/engine.py` with configurable pool settings and `application_name`.
- [ ] Updated `src/pa_dealing/api/main.py` to call `dispose_engine()` on shutdown.
- [ ] Updated `tests/conftest.py` and `reset_engine()` to properly close connections.
- [ ] Verified clean shutdown logs ("database_engine_disposed").
