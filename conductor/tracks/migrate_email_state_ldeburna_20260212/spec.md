# Specification: Migrate Email Ingestion State to ldeburna

## Goal
Eliminate the final dependency on `bo_airflow` by migrating the `email_ingestion_state` table to the `ldeburna` schema (or any configured `STATE_SCHEMA`). This ensures the application can run in a completely isolated environment (like `ldeburna`) without any cross-schema dependencies on legacy `bo_airflow`.

## Background
Currently, the application uses a hybrid configuration:
- Reference Data (Employees, Instruments): `ldeburna` (via `REFERENCE_SCHEMA`)
- Email State: `bo_airflow` (via `STATE_SCHEMA`)

This was a temporary measure because `ldeburna` lacked the `email_ingestion_state` table. To achieve full isolation, we must create this table in the target schema.

## Requirements
1.  **Table Creation:** The `email_ingestion_state` table must exist in the configured `STATE_SCHEMA` (e.g., `ldeburna`).
2.  **Schema Parameterization:** The table creation mechanism (Alembic) must support creating this table in a dynamic schema, or we must use a script if Alembic is restricted.
3.  **Code Update:** The application configuration must default `STATE_SCHEMA` to match `REFERENCE_SCHEMA` (or be explicitly set to `ldeburna`) so that NO traffic goes to `bo_airflow`.
4.  **Verification:** Confirm that email ingestion works correctly when pointing purely to `ldeburna`.

## Constraints
- `ldeburna` is a "read-only reference schema" in concept, but we evidently need write access to it for this state table, OR we need to move this state table to `padealing` (application schema).
- **Decision Point:** Should `email_ingestion_state` live in `padealing` (app-owned) or `ldeburna` (reference)? 
    - *Analysis:* The table is "shared state with Airflow". If Airflow reads/writes it, it's usually in `bo_airflow`. If we are moving away from `bo_airflow`, where does the *new* Airflow process look? 
    - *Assumption for this track:* We are replicating the `bo_airflow` structure into `ldeburna` for testing/isolation. So we will create it in `ldeburna`.

## Deliverables
- [ ] Alembic migration or SQL script to create `email_ingestion_state` in the target schema.
- [ ] Update `.env.dev` to set `STATE_SCHEMA=ldeburna`.
- [ ] Verification of full decoupling.
