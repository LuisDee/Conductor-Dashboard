# Implementation Plan: Migrate Email Ingestion State to ldeburna

## Phase 1: Investigation & Decision
- **Task:** Verify permissions to create tables in `ldeburna`.
    - *Action:* Run a test script to `CREATE TABLE ldeburna.test_table (...)`.
- **Decision:** If we cannot create tables in `ldeburna` (read-only perms), we must move `email_ingestion_state` to `padealing` (our app schema) and update the code to look there.
    - *Note:* Moving to `padealing` is architecturally cleaner if we own the process. If Airflow needs it, we grant Airflow access to `padealing`.

## Phase 2: Table Creation (The "Migration")
Assuming we decide to create it in `ldeburna` (to match `bo_airflow` structure):
- **Task:** Create `scripts/ops/create_email_state_table.py` that connects as `pad_app_user` and executes the DDL for `email_ingestion_state` in the target schema defined by env var.
    - *Why script vs Alembic?* Alembic is configured to manage `padealing`. Managing a table in `ldeburna` via Alembic might be tricky if we don't "own" the schema. A dedicated ops script is safer for this specific infrastructure setup task.

## Phase 3: Configuration Update
- **Task:** Update `.env.dev`:
    - Set `STATE_SCHEMA=ldeburna`.
- **Task:** Update `docker-compose.yml` to ensure `STATE_SCHEMA` is passed (already done, but verify).

## Phase 4: Verification
- **Task:** Restart containers.
- **Task:** Run email ingestion tests or trigger a poll.
- **Task:** Verify `ldeburna.email_ingestion_state` is populated.

## Phase 5: Cleanup
- **Task:** Remove any `bo_airflow` fallback logic from the codebase if it exists (e.g., in defaults).
