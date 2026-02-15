# Implementation Plan: PostgreSQL Connection Hygiene Remediation

## Phase 1: Configuration & Engine Refactor
**Goal:** Make database connection behavior configurable, observable, and resilient.

- **Task 1.1: Add Settings with Conservative Defaults**
  - **File:** `src/pa_dealing/config/settings.py`
  - **Action:** Add `db_pool_size` (default: 3), `db_max_overflow` (default: 2), and `db_pool_recycle` (default: 1800).
  - **Why:** Lower defaults (Total 5) prevent exhaustion in dev environments. Production can override via env vars.

- **Task 1.2: Identify Connections & Enable Health Checks**
  - **File:** `src/pa_dealing/db/engine.py`
  - **Action:** In `create_engine`:
    - Add `"application_name": "pa-dealing-api"` to `connect_args`.
    - Set `pool_pre_ping=True` (Enable connection health checks, replacing manual keepalives).
    - Set `pool_timeout=30` (Fail fast if pool exhausted).
    - Use `db_pool_recycle` setting.
  - **Why:** Observability + Reliability. `pre_ping` handles transparent reconnection; timeouts prevent infinite hangs.

## Phase 2: Application Shutdown Fix (The "Leak" Fix)
**Goal:** Ensure the API container releases all connections when it stops/restarts.

- **Task 2.1: Implement Async Disposal**
  - **File:** `src/pa_dealing/db/engine.py`
  - **Action:** Ensure `dispose_engine()` uses `await _engine.dispose()` properly.

- **Task 2.2: Call Disposal on Shutdown with Logging**
  - **File:** `src/pa_dealing/api/main.py`
  - **Action:** In `lifespan`:
    ```python
    yield
    await dispose_engine()
    log.info("database_engine_disposed")
    ```
  - **Why:** Explicitly closing the pool prevents "idle" connections. Logging proves it happened.

## Phase 3: Test Suite Fix (The "Flood" Fix)
**Goal:** Prevent the test suite from opening hundreds of connections.

- **Task 3.1: Fix `reset_engine`**
  - **File:** `src/pa_dealing/db/engine.py`
  - **Action:** Implement a disposal check in `reset_engine()`. Warning: Synchronous disposal of async engines is tricky; preferred approach is to await disposal in the test fixture *before* calling reset.

- **Task 3.2: Audit for Rogue Engines**
  - **Action:** Grep `tests/` for `create_engine` or `create_async_engine` to find tests bypassing the shared module. Refactor them to use `db/engine.py`.

- **Task 3.3: Update Test Teardown**
  - **File:** `tests/conftest.py`
  - **Action:** Update fixtures to `await dispose_engine()` explicitly during teardown.

## Phase 4: Server-Side Safeguards (The Safety Net)
**Goal:** Prevent future regressions from taking down the database.

- **Task 4.1: Apply User Limits (Documentation/Ops)**
  - **File:** `docs/deployment/database_tuning.md`
  - **Action:** Document the following safeguards:
    ```sql
    ALTER ROLE pad_app_user SET idle_session_timeout = '300000';  -- 5 min
    ALTER ROLE pad_app_user SET idle_in_transaction_session_timeout = '60000';  -- 1 min
    ALTER ROLE pad_app_user CONNECTION LIMIT 50;
    ```
  - **Why:** Hard limits prevent one bad service from starving the entire cluster.

## Phase 5: Verification
**Goal:** Prove the fixes worked.

- **Task 5.1: Shutdown Test**
  - **Action:** Start API -> Stop API. Verify `pg_stat_activity` count drops to 0. Check logs for "database_engine_disposed".

- **Task 5.2: Test Suite Monitor**
  - **Action:** Run test suite while polling `SELECT count(*), usename, application_name FROM pg_stat_activity GROUP BY usename, application_name;`.
  - **Target:** Connection count should stay stable (< 10) and not grow linearly.
