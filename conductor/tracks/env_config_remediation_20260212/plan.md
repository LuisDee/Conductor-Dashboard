# Implementation Plan: Environment Configuration & Code Robustness Remediation

## Phase 1: Code Integrity & Logic Robustness
**Goal:** Fix the internal logic errors in the settings and authentication layers.

- **Task 1.1: Fix Settings Decorators**
  - **File:** `src/pa_dealing/config/settings.py`
  - **Action:** Add `@property` decorator to `is_development()` and `is_production()`.
  - **Why:** In the current state, `settings.is_development` returns a *method object* rather than a boolean. While Python treats non-None objects as `True`, this is extremely fragile and breaks any logic that expects a boolean or tries to serialize the object.
  - **Verification:** `python -c "assert isinstance(get_settings().is_development, bool)"`

- **Task 1.2: Standardize Development Environment Detection**
  - **File:** `src/pa_dealing/config/settings.py`
  - **Action:** Ensure `is_development` property correctly evaluates `ENVIRONMENT` against all variants: `dev`, `development`, and `local`.
  - **File:** `src/pa_dealing/api/auth.py`
  - **Action:** Update all `if settings.environment == "development"` checks to use `if settings.is_development`.
  - **Why:** Our `.env.dev` uses `ENVIRONMENT=dev`, but the code was looking for `development`, causing auto-authentication to fail silently.

## Phase 2: Configuration Sovereignty & service Synchronization
**Goal:** Ensure all Docker services see the same configuration and prevent silent schema reversions.

- **Task 2.1: Synchronize docker-compose.yml**
  - **File:** `docker/docker-compose.yml`
  - **Action:** Perform a vertical audit of all services: `api`, `slack-listener`, `monitoring`, `pdf-poller`, `graph-email-poller`.
  - **Requirements:** Every service MUST have the following environment variables mapped:
    - `REFERENCE_SCHEMA`
    - `STATE_SCHEMA`
    - `ENVIRONMENT`
    - `DATABASE_URL`
    - `ORACLE_BACKOFFICE_URL` (where enrichment is used)
  - **Why:** Currently, some services (like the poller) were missing `REFERENCE_SCHEMA`, causing them to default to `bo_airflow` while the API was looking at `ldeburna`, creating a split-brain state.

- **Task 2.2: Standardize Fallback Logic**
  - **Action:** Change `${VAR:-bo_airflow}` to `${VAR:-ldeburna}` in `docker-compose.yml` for the reference schema, OR remove the fallback entirely to ensure we fail fast if the environment isn't set.
  - **Why:** The previous `${VAR:-bo_airflow}` fallback was silently "undoing" our migration whenever the shell environment was lost.

## Phase 3: Application Critical Fixes
**Goal:** Restore functionality to services broken by the migration or latent bugs.

- **Task 3.1: Fix Graph Email Poller Crash**
  - **File:** `src/pa_dealing/services/graph_email_poller.py`
  - **Action:** Update the message processing loop to use `message.message_id` instead of `message.id`.
  - **Why:** The `MessageInfo` dataclass uses `message_id`. This is a hard crash preventing all email ingestion.

## Phase 4: Cleanup & Verification
**Goal:** Prove the system is stable and clean.

- **Task 4.1: Purge Bytecode Cache**
  - **Action:** Run `find . -name "__pycache__" -exec rm -rf {} +` across the project root and mounted volumes.
  - **Why:** Stale `.pyc` files can cause the Python interpreter to execute old logic even after the `.py` file has been changed.

- **Task 4.2: Full Stack Cold Restart**
  - **Action:** `docker compose -f docker/docker-compose.yml down` followed by `docker compose -f docker/docker-compose.yml --env-file .env.dev up -d`.
  - **Verification:** 
    - `docker exec pad_api env | grep SCHEMA` (Verify ldeburna)
    - `docker exec pad_graph_email_poller env | grep SCHEMA` (Verify ldeburna)
    - `curl http://localhost:8000/api/auth/me` (Verify auth_status: ok)
