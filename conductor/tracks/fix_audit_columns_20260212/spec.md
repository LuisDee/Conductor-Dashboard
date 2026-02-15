# Specification: Fix Missing Audit Columns (Trace ID)

## Overview
A `ProgrammingError` (UndefinedColumnError) was reported during trade submission because the `trace_id` column is missing from the `audit_events` table. Investigation revealed that Alembic migration `b79ae701f82b` (Add trace and reference to audit logs) has not been applied to the target environments. This track aims to synchronize the database schema with the SQLAlchemy models across all environments.

## Functional Requirements
- Apply Alembic migration `b79ae701f82b` to all environments (Local Docker and External Dev).
- Ensure the `audit_events` table includes `trace_id` and `trade_reference_id` columns.
- Ensure the `audit_log` table includes `trace_id` and `snapshot_hash` columns (as per the same migration).
- **Update API routes to bind `trade_reference_id` to the log context so it is correctly captured in `audit_events` and `audit_log`.**
- Verify that trade submissions no longer fail with `UndefinedColumnError`.

## Non-Functional Requirements
- **Consistency:** Database schemas must be identical across local and external development environments.
- **Reliability:** Audit events must be correctly persisted with correlation/trace IDs for debugging.

## Acceptance Criteria
- [ ] Migration `b79ae701f82b` is successfully applied to the local database.
- [ ] Migration `b79ae701f82b` is successfully applied to the external dev database.
- [ ] Schema verification (`\d padealing.audit_events` and `\d padealing.audit_log`) confirms new columns exist.
- [ ] Integration tests related to auditing and trade submission pass.
- [ ] Manual trade submission via the UI is successful without database errors.

## Out of Scope
- Refactoring the auditing logic itself.
- Updating historical audit records with missing trace IDs (unless required for schema constraints).
