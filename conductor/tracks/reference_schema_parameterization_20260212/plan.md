# Implementation Plan: Reference Schema Parameterization & ldeburna Migration

## Phase 1: Diagnostic & Audit (Safety First)

### 1.1 Column Parity Audit [COMPLETED]
- **Finding:** Verified that `ldeburna` contains all core reference tables with matching columns.
- **Exception:** `email_ingestion_state` is MISSING in `ldeburna`.
- **Action:** Retain `bo_airflow` as a secondary schema or confirm if `email_ingestion_state` should be migrated by the Airflow team.

### 1.2 Permission Verification
Verify that `pad_app_user` has `SELECT`, `USAGE`, and `REFERENCES` on `ldeburna`.
- **Command:** Run verification SQL from `docs/deployment/database_permissions.md` targetting `ldeburna`.

## Phase 2: Code Parameterization (Refactoring)

### 2.1 Refactor SQLAlchemy Base & Models
Inject dynamic schema support into `Base` using a `SchemaMixin`.
- **File:** `src/pa_dealing/db/models/base.py`
  - Implement `ReferenceSchemaMixin` that reads `settings.reference_schema`.
- **File:** `src/pa_dealing/db/models/market.py`
  - Update all `Oracle*` models to inherit from `ReferenceSchemaMixin`.
- **File:** `src/pa_dealing/db/models/core.py`
  - Update `OracleEmployee`, `OracleContact`, `OracleEmployeeLog`, `OraclePortfolio`.
- **File:** `src/pa_dealing/db/models/email_ingestion.py`
  - **Special Case:** Handle `EmailIngestionState`. If it must stay in `bo_airflow`, hardcode it or add a `LEGACY_STATE_SCHEMA` setting.

### 2.2 Refactor Foreign Key Definitions
Convert static strings in `ForeignKey` to dynamic references.
- **Files:** `src/pa_dealing/db/models/pad.py`, `compliance.py`, `document.py`.
- **Pattern:** `ForeignKey(f"{get_settings().reference_schema}.oracle_employee.id")`.

### 2.3 Refactor Raw SQL Fragments
Remove hardcoded `bo_airflow.` from strings.
- **File:** `src/pa_dealing/db/sql_fragments.py`
  - Use f-strings: `FROM {get_settings().reference_schema}.oracle_employee`.
- **Inventory Check:** Grep and replace in:
  - `src/pa_dealing/db/repository.py`
  - `src/pa_dealing/identity/postgres.py`
  - `src/pa_dealing/identity/provider_google.py`
  - `src/pa_dealing/identity/fuzzy_matcher.py`

### 2.4 CI Guardrail
Add a ruff rule or a grep-based check to prevent regression.
- **Task:** Update `scripts/lint.sh` to fail if `bo_airflow.` is found in `src/`.

## Phase 3: Database Migration (The Big Switch)

### 3.1 Create Migration: Repoint Foreign Keys
Generate a new Alembic migration: `repoint_reference_schema_ldeburna`.
- **Logic:**
  1. Detect existing FKs on `pad_request`, `pad_approval`, `pad_execution`, `restricted_security`, `audit_log`, `pad_breach`.
  2. Drop FKs pointing to `bo_airflow`.
  3. Add FKs pointing to `ldeburna` as `NOT VALID`.
  4. Perform `VALIDATE CONSTRAINT` in a separate transaction (or outside transaction if possible in Alembic).

### 3.2 Repoint Environment
- **File:** `.env.dev`
  - Set `REFERENCE_SCHEMA=ldeburna`.

## Phase 4: Verification & Testing

### 4.1 Unit & Integration Tests
Run core identity and market tests using the new schema.
- **Command:** `./scripts/test-runner.sh fast`
- **Focus:** `tests/test_identity_resolution.py`, `tests/test_instrument_lookup.py`.

### 4.2 Cross-Schema Search Path Validation
- **File:** `src/pa_dealing/db/engine.py`
  - Verify `search_path` correctly includes `padealing`, `ldeburna`, and `public`.

### 4.3 Manual UAT
Verify the dashboard loads data correctly in the Dev environment.

## Phase 5: Cleanup

### 5.1 Update Documentation
- **File:** `docs/tooling/identity-resolution.md`
- **File:** `docs/tooling/instrument-lookup.md`
- **File:** `docs/deployment/database_permissions.md`

### 5.2 Archive Audit Script
Move `scripts/ops/audit_reference_schema.py` to `scripts/ops/archive/` or delete it.
