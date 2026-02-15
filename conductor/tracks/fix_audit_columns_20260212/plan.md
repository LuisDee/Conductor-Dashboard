# Implementation Plan: Fix Missing Audit Columns (Trace ID)

## Phase 1: Database Migration & Schema Sync
- [ ] Task: Apply Alembic migration `b79ae701f82b` to the local Docker database.
    - [ ] Run `docker compose exec backend alembic upgrade head`.
- [ ] Task: Apply Alembic migration `b79ae701f82b` to the external development database (`env.dev`).
    - [ ] Ensure correct environment variables are set for the migration.
- [ ] Task: Verify schema changes in both environments.
    - [ ] Confirm `trace_id` and `trade_reference_id` exist in `padealing.audit_events`.
    - [ ] Confirm `trace_id` and `snapshot_hash` exist in `padealing.audit_log`.
- [ ] Task: Conductor - User Manual Verification 'Database Migration & Schema Sync' (Protocol in workflow.md)

## Phase 2: Integration Testing & Verification
- [ ] Task: Implement `trade_reference_id` binding in `src/pa_dealing/api/routes/requests.py`.
- [ ] Task: Execute backend integration tests for auditing and request processing.
    - [ ] Run `pytest tests/integration/test_auditing.py` (or equivalent).
- [ ] Task: Perform manual end-to-end verification.
    - [ ] Submit a trade via the Slack chatbot or Dashboard UI.
    - [ ] Confirm the request processes successfully without `UndefinedColumnError`.
    - [ ] Verify the audit entry contains a valid `trace_id`.
- [ ] Task: Conductor - User Manual Verification 'Integration Testing & Verification' (Protocol in workflow.md)

## Phase 3: Final Regression & Cleanup
- [ ] Task: Run the full regression suite (Unit + E2E + Playwright) as per the Workflow.
- [ ] Task: Update `meta.yaml` to mark the track as complete.
- [ ] Task: Conductor - User Manual Verification 'Final Regression & Cleanup' (Protocol in workflow.md)
