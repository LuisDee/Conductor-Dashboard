# Implementation Plan: Database Schema Consolidation

## Phase 1: Analysis & Preparation

### 1.1 Document Current State
- [x] Task: Export current table list from pad_db (port 5432)
- [x] Task: Export current table list from pa_dealing_postgres (port 5433)
- [x] Task: Document all SQLAlchemy models and their current schema settings
- [x] Task: Identify all code references to schema-qualified table names

### 1.2 Create Schema Mapping
- [x] Task: Create definitive list of tables for `padealing` schema
- [x] Task: Create definitive list of tables for `bo_airflow` schema
- [x] Task: Verify mapping against dev database structure
- [x] Task: Document any tables that exist locally but not in dev (test-only tables)

### 1.3 Backup & Safety
- [x] Task: Create backup of pad_db current state
- [x] Task: Document rollback procedure
- [x] Task: Create git branch for schema consolidation work

- [x] Task: Conductor - User Manual Verification 'Phase 1: Analysis & Preparation' (Protocol in workflow.md)

---

## Phase 2: Database Infrastructure Changes

### 2.1 Stop Secondary Database
- [x] Task: Stop pa_dealing_postgres container
- [x] Task: Update any scripts referencing port 5433
- [x] Task: Archive root docker-compose.yml (rename to docker-compose.yml.archived)

### 2.2 Create Database Init Script
- [x] Task: Create `docker/db/init-schemas.sql` script with:
  - CREATE SCHEMA IF NOT EXISTS padealing
  - CREATE SCHEMA IF NOT EXISTS bo_airflow
  - GRANT appropriate permissions
- [x] Task: Update `docker/docker-compose.yml` to mount init script
- [x] Task: Test init script on fresh database container

### 2.3 Update Environment Configuration
- [x] Task: Update `.env` to use DATABASE_SCHEMA=padealing and REFERENCE_SCHEMA=bo_airflow
- [x] Task: Update `.env.local` with same schema settings
- [x] Task: Update `docker/docker-compose.yml` environment variables for API container
- [x] Task: Verify settings.py reads schema configuration correctly

- [x] Task: Conductor - User Manual Verification 'Phase 2: Database Infrastructure Changes' (Protocol in workflow.md)

---

## Phase 3: SQLAlchemy Model Updates

### 3.1 Update Application Models (padealing schema)
- [x] Task: Update PADRequest model - add `__table_args__ = {'schema': 'padealing'}`
- [x] Task: Update PADApproval model schema
- [x] Task: Update PADExecution model schema
- [x] Task: Update PADBreach model schema
- [x] Task: Update AuditLog model schema
- [x] Task: Update ChatbotSession model schema
- [x] Task: Update ComplianceConfig model schema
- [x] Task: Update ComplianceDecisionOutcome model schema
- [x] Task: Update EmployeeRole model schema
- [x] Task: Update RestrictedSecurity model schema
- [x] Task: Update PersonalAccountDealing model schema (if exists)
- [x] Task: Update ActiveStocks model schema (if exists)

### 3.2 Update Reference Models (bo_airflow schema)
- [x] Task: Verify OracleContact model has `schema='bo_airflow'` (already set)
- [x] Task: Update OracleEmployee model - add `schema='bo_airflow'`
- [x] Task: Update OracleBloomberg model schema
- [x] Task: Update OracleExchange model schema
- [x] Task: Update OracleMapInstSymbol model schema
- [x] Task: Update OraclePortfolio model schema
- [x] Task: Update OraclePosition model schema
- [x] Task: Update OracleProduct model schema
- [x] Task: Update InstType model schema
- [x] Task: Update Currency model schema
- [x] Task: Update Contact model schema (if used)
- [x] Task: Update ContactGroup model schema
- [x] Task: Update ContactType model schema
- [x] Task: Update MakoPosition model schema (if exists)

### 3.3 Update Query References
- [x] Task: Search for hardcoded schema references in provider_google.py
- [x] Task: Update any raw SQL queries to use parameterized schema names
- [x] Task: Verify all JOIN queries use correct schema-qualified names

- [x] Task: Conductor - User Manual Verification 'Phase 3: SQLAlchemy Model Updates' (Protocol in workflow.md)

---

## Phase 4: Alembic Migration

### 4.1 Create Migration File
- [x] Task: Generate new Alembic migration: `alembic revision -m "consolidate_to_dual_schema"`
- [x] Task: Write upgrade() function:
  - Create padealing schema
  - Create bo_airflow schema
  - Move/recreate tables in correct schemas
- [x] Task: Write downgrade() function for rollback capability
- [x] Task: Handle alembic_version table location

### 4.2 Test Migration
- [x] Task: Test migration on fresh database (docker compose down -v, up)
- [x] Task: Test migration on existing database with data
- [x] Task: Verify all tables exist in correct schemas after migration
- [x] Task: Verify foreign key relationships still work

- [x] Task: Conductor - User Manual Verification 'Phase 4: Alembic Migration' (Protocol in workflow.md)

---

## Phase 5: Test Infrastructure Updates

### 5.1 Update conftest.py
- [x] Task: Remove dynamic `CREATE SCHEMA IF NOT EXISTS bo_airflow` (now in DB init)
- [x] Task: Update fixture table references to use schema-qualified names
- [x] Task: Update seed data INSERT statements for padealing schema tables
- [x] Task: Update seed data INSERT statements for bo_airflow schema tables
- [x] Task: Update cleanup/teardown to handle both schemas

### 5.2 Update Test Files
- [x] Task: Search all test files for hardcoded table references
- [x] Task: Update test_google_identity_provider.py table references
- [x] Task: Update test_unauthorized_access.py if needed
- [x] Task: Update integration test files for schema-qualified queries
- [x] Task: Update any mock configurations for schema names

### 5.3 Update pytest Configuration
- [x] Task: Update pytest.ini or pyproject.toml if DATABASE_URL needs changing
- [x] Task: Ensure tests use port 5432 (pad_db) not 5433
- [x] Task: Remove any test-specific database configuration for port 5433

- [x] Task: Conductor - User Manual Verification 'Phase 5: Test Infrastructure Updates' (Protocol in workflow.md)

---

## Phase 6: Docker Stack Updates

### 6.1 Rebuild Containers
- [x] Task: Stop all PA Dealing containers
- [x] Task: Remove old database volume: `docker volume rm docker_postgres_data`
- [x] Task: Rebuild API container: `docker compose -f docker/docker-compose.yml build api`
- [x] Task: Start fresh stack: `docker compose -f docker/docker-compose.yml up -d`

### 6.2 Verify Container Database
- [x] Task: Connect to pad_db and verify schemas exist
- [x] Task: Verify tables are in correct schemas
- [x] Task: Verify API container can query bo_airflow.oracle_contact
- [x] Task: Test API health endpoint

### 6.3 Seed Test Data
- [x] Task: Ensure oracle_employee has test data in bo_airflow schema
- [x] Task: Ensure oracle_contact has test data with email mappings
- [x] Task: Verify identity resolution works (email → employee_id)

- [x] Task: Conductor - User Manual Verification 'Phase 6: Docker Stack Updates' (Protocol in workflow.md)

---

## Phase 7: Verification & Testing

### 7.1 Run Unit Tests
- [x] Task: Run full pytest suite: `poetry run pytest`
- [x] Task: Verify all previously passing tests still pass
- [x] Task: Fix any schema-related test failures

### 7.2 Run Integration Tests
- [x] Task: Run audit retrieval tests: `poetry run pytest tests/integration/test_audit_retrieval.py -v`
- [x] Task: Verify all 9 audit tests now pass (no more 500 errors)
- [x] Task: Run full integration test suite

### 7.3 Manual Verification
- [x] Task: Test API endpoint manually: `curl http://localhost:8000/health`
- [x] Task: Test request submission via API
- [x] Task: Verify audit trail is created correctly
- [x] Task: Check database schemas match expected layout

### 7.4 Final Validation
- [x] Task: Run complete test suite: `poetry run pytest -v`
- [x] Task: Verify test count matches or exceeds previous (267+ tests)
- [x] Task: Document any tests that were modified or removed

- [x] Task: Conductor - User Manual Verification 'Phase 7: Verification & Testing' (Protocol in workflow.md)

---

## Phase 8: Cleanup & Documentation

### 8.1 Remove Deprecated Files
- [x] Task: Delete or archive `docker-compose.yml` from project root
- [x] Task: Remove `scripts/db/init_db.sql` if no longer needed
- [x] Task: Clean up any orphaned database configuration files

### 8.2 Update Documentation
- [x] Task: Update README.md with new database setup instructions
- [x] Task: Document the dual-schema architecture
- [x] Task: Update any developer setup guides

### 8.3 Final Commit
- [x] Task: Stage all changes
- [x] Task: Create comprehensive commit message
- [x] Task: Push to branch

- [x] Task: Conductor - User Manual Verification 'Phase 8: Cleanup & Documentation' (Protocol in workflow.md)

---

## Summary

| Phase | Tasks | Focus Area |
|-------|-------|------------|
| 1 | 11 | Analysis & Preparation |
| 2 | 11 | Database Infrastructure |
| 3 | 19 | SQLAlchemy Models |
| 4 | 7 | Alembic Migration |
| 5 | 10 | Test Infrastructure |
| 6 | 9 | Docker Stack |
| 7 | 11 | Verification |
| 8 | 7 | Cleanup |
| **Total** | **85** | |

## Risk Mitigation

1. **Data Loss**: Backup database before starting; keep archived docker-compose.yml
2. **Test Failures**: Run tests after each phase; fix issues before proceeding
3. **Rollback**: Alembic downgrade() provides rollback; git branch preserves code state
4. **Container Issues**: Document exact docker commands; test on fresh containers

## Dependencies

- Phase 2 must complete before Phase 3 (schemas must exist for models)
- Phase 3 must complete before Phase 4 (models define migration targets)
- Phase 4 must complete before Phase 5 (migration creates schema structure)
- Phase 6 depends on Phases 2-5 (all code changes needed for containers)
- Phase 7 validates all previous phases
