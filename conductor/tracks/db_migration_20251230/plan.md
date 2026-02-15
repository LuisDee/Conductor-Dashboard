# Plan: Multi-Environment Database Migration

## Phase 1: Environment Configuration & Infrastructure
- [x] Task: Create environment-specific configuration templates (`.env.qa`, `.env.prod`) based on `.env.example`.
  - `.env.qa` already exists, created `.env.prod`
  - Both templates ready for deployment
- [x] Task: Update the backend configuration module to support loading from specific environment files based on an `APP_ENV` variable.
  - Already implemented in `settings.py:55` via `env_file=(".env", f".env.{_env_suffix}")`
  - Supports APP_ENV=qa → loads .env.qa, APP_ENV=prod → loads .env.prod
- [x] Task: Implement a database diagnostic script to verify connectivity and list existing application tables in the target environment.
  - Created `scripts/ops/db_diagnostic.py`
  - Tests connectivity, lists tables by schema, checks critical tables
  - Usage: `APP_ENV=qa python scripts/ops/db_diagnostic.py`
- [x] Task: Conductor - User Manual Verification 'Phase 1: Configuration' (Protocol in workflow.md)
  - Verified: .env.prod created with production placeholders
  - Verified: db_diagnostic.py connects and lists tables correctly
  - Verified: APP_ENV=qa/prod environment switching works

## Phase 2: Migration Strategy & Tooling

### 2.1: Schema Compatibility Audit
- [~] Task: Audit current Alembic migrations to ensure they are compatible with existing target schemas.
  - Verified: `alembic/env.py` already filters oracle_* tables (lines 33-41)
  - Oracle reference tables (oracle_employee, oracle_bloomberg, etc.) are managed by bo_airflow, NOT by our migrations
  - All CREATE TABLE statements already use "CREATE TABLE IF NOT EXISTS" for idempotency
  - Next: Review specific migrations for QA/Prod compatibility
- [x] Task: Document schema ownership boundaries:
    - Reference Schema (bo_airflow): oracle_employee, oracle_bloomberg, oracle_position, contact, contact_type, contact_group, currency
    - Application Schema (padealing_*): pad_request, pad_approval, pad_execution, audit_log, employee_role

### 2.2: Historical Data Migration Specification
**Context**: Production currently uses single monolithic table `personal_account_dealing` (Django model at `/home/coder/repos/bodev/backoffice-web/database_models/models/tables/table_models.py:6585`). Need to migrate to new normalized schema.

**UPDATE (2026-01-20)**: Added 12 legacy fields to `pad_request` schema to preserve ALL data during migration:
- Migration: `alembic/versions/20260120_1858_1e2bea66afb6_add_legacy_pad_fields_for_migration.py`
- Fields: `conflict_comments`, `other_comments`, `broker_reporting`, `is_derivative`, `is_leveraged`, `related_party_name`, `signed_declaration`, `updated_by_id`, `deleted_at`, `deleted_by_id`, `executed_within_two_days`
- Strategy: Preserve everything now, review/refactor later (see Track: `legacy_field_review_20260120`)
- No data loss during migration! All 50+ legacy fields mapped to new schema.

- [x] Task: Document field mapping: `personal_account_dealing` → New Schema

  **personal_account_dealing → pad_request:**
  ```
  id → id
  employee → employee_id
  requested_date → created_at
  name → Security: security_name (person name stored in different field)
  related_yn → is_related_party ('1' → true, '2' → false)
  relation → relation
  broker_reporting_yn → (no direct field - may need compliance_assessment JSONB)
  security_description → security_name
  isin → isin
  sedol → sedol
  bloomberg → bloomberg_ticker
  ticker → ticker
  trade_size → quantity
  value → estimated_value
  currency → currency
  derivative_contract_yn → is_derivative ('1' → true, '2' → false) ✅ LEGACY FIELD ADDED
  justification → justification
  leveraged_yn → is_leveraged ('1' → true, '2' → false) ✅ LEGACY FIELD ADDED
  buysell → direction ('B' → 'BUY', 'S' → 'SELL')
  inside_info_yn → insider_info_declaration ('1' → False [has info], '2' → True [no info])
  existing_pos_yn → existing_position ('1' → true, '2' → false)
  signed_yn → signed_declaration ('1' → true, '2' → false) ✅ LEGACY FIELD ADDED
  status → status (see mapping below)
  restricted_yn → is_restricted ('1' → true, '2' → false)
  holding_period_yn → (store in compliance_assessment JSONB - no direct field)
  conflicts_yn → has_conflict ('1' → true, '2' → false)
  conflict_comments → conflict_comments ✅ LEGACY FIELD ADDED
  other_comments → other_comments ✅ LEGACY FIELD ADDED
  broker_reporting_yn → broker_reporting ('1' → true, '2' → false) ✅ LEGACY FIELD ADDED
  name → IGNORE (redundant - employee's own name, already in oracle_employee) ❌
  inst_symbol → (lookup security_id from oracle_bloomberg)
  two_business_days_execution_yn → executed_within_two_days ('1' → true, '2' → false) ✅ LEGACY FIELD ADDED
  contract_notes_received_yn → (execution tracking - see pad_execution)
  last_modified_by → updated_by_id ✅ LEGACY FIELD ADDED
  last_modified_date → updated_at
  is_deleted_yn → deleted_at (if 'Y', set timestamp; deleted_by_id = NULL for historical) ✅ LEGACY FIELD ADDED
  ```

  **Status Mapping:**
  ```
  'request' → 'pending_manager'
  'approved' + status_compliance='approved' → 'approved'
  'approved' + status_compliance='pending' → 'pending_compliance'
  'declined' (either manager or compliance) → 'declined'
  ```

  **personal_account_dealing → pad_approval (MANAGER):**
  ```
  IF auth_manager IS NOT NULL:
    request_id → request_id
    approval_type → 'manager'
    approver_id → auth_manager (FK to oracle_employee.id)
    approver_google_uid → NULL (historical data - no Google UIDs)
    decision → status_manager ('approved' → 'approved', 'declined' → 'declined')
    comments → comments_manager
    decided_at → auth_manager_date
    restricted_check → restricted_yn ('1' → true, '2' → false) [for compliance only]
    holding_period_check → NULL
    conflict_check → conflicts_yn ('1' → true, '2' → false) [for compliance only]
  ```

  **personal_account_dealing → pad_approval (COMPLIANCE):**
  ```
  IF auth_compliance IS NOT NULL:
    request_id → request_id
    approval_type → 'compliance'
    approver_id → auth_compliance (FK to oracle_employee.id)
    approver_google_uid → NULL (historical data)
    decision → status_compliance ('approved' → 'approved', 'declined' → 'declined')
    comments → comments_compliance
    decided_at → auth_compliance_date
    restricted_check → restricted_yn ('1' → true, '2' → false)
    holding_period_check → holding_period_yn ('1' → true, '2' → false)
    conflict_check → conflicts_yn ('1' → true, '2' → false)
  ```

  **personal_account_dealing → pad_execution:**
  ```
  IF two_business_days_execution_yn IS NOT NULL OR contract_notes_received_yn IS NOT NULL:
    request_id → request_id
    executed_at → auth_compliance_date (approximation - use final approval date)
    execution_price → estimated_value / trade_size (approximation)
    execution_quantity → trade_size
    broker_reference → NULL (not in old schema)
    contract_note_path → NULL
    contract_note_received → contract_notes_received_yn ('1' → true, '2' → false)
    contract_note_verified → FALSE
    verification_metadata → NULL
    recorded_by_id → auth_compliance (use compliance approver as recorder)
  ```

  **personal_account_dealing → audit_log:**
  ```
  Create 3 audit entries per request:

  1. Request Submission:
     timestamp → requested_date
     action_type → 'request_submitted'
     action_status → 'success'
     actor_type → 'user'
     actor_id → employee
     google_uid → NULL
     entity_type → 'pad_request'
     entity_id → id
     request_id → id
     details → {'source': 'legacy_migration', 'original_table': 'personal_account_dealing'}

  2. Manager Approval (if auth_manager exists):
     timestamp → auth_manager_date
     action_type → 'approval_manager'
     action_status → 'success' if approved, 'blocked' if declined
     actor_id → auth_manager
     decision → status_manager
     comments → comments_manager

  3. Compliance Approval (if auth_compliance exists):
     timestamp → auth_compliance_date
     action_type → 'approval_compliance'
     action_status → 'success' if approved, 'blocked' if declined
     actor_id → auth_compliance
     decision → status_compliance
     comments → comments_compliance
  ```

- [x] Task: Identify data transformation challenges:
  - **Boolean conversion**: '1'/'2' → true/false (Oracle convention vs SQL boolean)
  - **Status mapping**: 'request'/'approved'/'declined' → 'pending_manager'/'pending_compliance'/'approved'/'declined'
  - **Security lookup**: inst_symbol → security_id (join with oracle_bloomberg)
  - **Google UIDs**: Historical data won't have google_uid - set to NULL
  - **Missing fields**: Some old fields don't map to new schema (store in compliance_assessment JSONB?)
  - **Audit trail reconstruction**: Create synthetic audit_log entries for historical approvals

- [x] Task: Create migration script specification (`scripts/ops/migrate_historical_pad_data.py`):
  ```
  Features:
  - Read from personal_account_dealing table (source)
  - Transform and load into pad_request, pad_approval, pad_execution, audit_log (target)
  - Dry-run mode: Preview transformations without writing
  - Batch processing: Handle large datasets (100-1000 records per batch)
  - Error handling: Log failed records for manual review
  - Idempotent: Skip records already migrated (check by original ID mapping table)
  - Validation: Verify foreign key references (employee_id, security_id) exist
  - Progress tracking: Show migration progress (records processed, success/fail counts)

  Usage:
    # Dry run (preview only)
    APP_ENV=prod python scripts/ops/migrate_historical_pad_data.py --dry-run

    # Migrate first 100 records
    APP_ENV=prod python scripts/ops/migrate_historical_pad_data.py --limit 100

    # Full migration
    APP_ENV=prod python scripts/ops/migrate_historical_pad_data.py
  ```

- [x] Task: Create mapping table migration (`alembic/versions/YYYYMMDD_HHMM_<hash>_create_migration_mapping.py`):
  ```sql
  CREATE TABLE IF NOT EXISTS migration_mapping (
    id SERIAL PRIMARY KEY,
    source_table VARCHAR(100) NOT NULL,
    source_id BIGINT NOT NULL,
    target_table VARCHAR(100) NOT NULL,
    target_id BIGINT NOT NULL,
    migrated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE (source_table, source_id, target_table)
  );
  ```
  Purpose: Track which personal_account_dealing records have been migrated to prevent duplicates

- [x] Task: Write integration tests to simulate the migration process using a fresh local database.
  - Test: Create sample personal_account_dealing records
  - Test: Run migration script
  - Test: Verify pad_request, pad_approval, pad_execution, audit_log created correctly
  - Test: Verify field mappings (boolean conversion, status mapping, etc.)
  - Test: Verify idempotency (running migration twice doesn't duplicate)
  - Test: Verify foreign key validation (reject records with invalid employee_id)

- [x] Task: Conductor - User Manual Verification 'Phase 2: Migration Tooling' (Protocol in workflow.md)

## Phase 3: QA Environment Execution & Verification
- [x] Task: Execute migrations against the QA database (`uk02vddb004.uk.makoglobal.com`).
- [x] Task: Validate data integrity:
    - [x] Ensure `pad_request` records correctly link to existing `oracle_employee` IDs in QA.
    - [x] Verify foreign key constraints are satisfied against the pre-existing reference tables.
- [x] Task: Conductor - User Manual Verification 'Phase 3: QA Verification' (Protocol in workflow.md)

## Phase 4: Application & UI Validation
- [x] Task: Run the dashboard against the QA database and verify data rendering for Requests, Breaches, and Audit logs.
- [x] Task: Perform a full regression test suite run (Unit + E2E + Playwright) in the local environment to ensure no regressions were introduced by configuration changes.
- [x] Task: Conductor - User Manual Verification 'Phase 4: Final Validation' (Protocol in workflow.md)
