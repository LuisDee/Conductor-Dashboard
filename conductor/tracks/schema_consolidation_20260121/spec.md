# Database Schema Consolidation

## Overview

Consolidate local development databases to match the dev/prod dual-schema architecture (`padealing` + `bo_airflow`), eliminating the current split between two separate PostgreSQL containers and ensuring all tests run against a consistent schema layout.

## Problem Statement

1. **Two separate local databases** exist:
   - `pad_db` on port 5432 (used by Docker API stack)
   - `pa_dealing_postgres` on port 5433 (used by pytest)

2. **Neither matches dev/prod architecture**:
   - Both use single `public` schema
   - Dev/prod uses `padealing` schema for application tables
   - Dev/prod uses `bo_airflow` schema for Oracle sync tables

3. **Critical table missing**:
   - `oracle_contact` must be in `bo_airflow` schema
   - Code queries `bo_airflow.oracle_contact` but table doesn't exist
   - Causes HTTP 500 errors on all API requests requiring identity resolution

4. **Integration tests fail**:
   - 9 audit retrieval tests fail with "relation bo_airflow.oracle_contact does not exist"

## Current State

### Local Databases
| Container | Port | Schemas | Tables |
|-----------|------|---------|--------|
| pad_db | 5432 | public only | 23 tables |
| pa_dealing_postgres | 5433 | public only | 28 tables |

### Dev Database
| Schema | Tables | Purpose |
|--------|--------|---------|
| padealing | 13 | Application data (read-write) |
| bo_airflow | 245 | Oracle sync data (read-only) |

## Functional Requirements

### FR1: Single Database
- Remove `pa_dealing_postgres` container (port 5433)
- Use only `pad_db` (port 5432) for all local development
- Remove or archive root `docker-compose.yml`

### FR2: Dual-Schema Architecture
- Create `padealing` schema for application tables
- Create `bo_airflow` schema for Oracle reference tables
- Match dev/prod schema layout exactly

### FR3: Table Placement

**padealing schema:**
- pad_request, pad_approval, pad_execution, pad_breach
- audit_log, chatbot_session
- compliance_config, compliance_decision_outcome
- employee_role, restricted_security
- personal_account_dealing, active_stocks
- alembic_version

**bo_airflow schema:**
- oracle_contact, oracle_employee
- oracle_bloomberg, oracle_exchange
- oracle_map_inst_symbol, oracle_portfolio
- oracle_position, oracle_product
- inst_type, currency
- contact, contact_group, contact_type
- mako_position (test data only)

### FR4: SQLAlchemy Model Updates
- Update `__table_args__` for each model to specify correct schema
- Use `schema='padealing'` for application models
- Use `schema='bo_airflow'` for Oracle reference models

### FR5: Alembic Migration
- Create migration to establish dual-schema architecture
- Handle existing data migration if needed
- Ensure idempotent execution

### FR6: Test Fixture Updates
- Update `tests/conftest.py` to work with new schemas
- Remove dynamic schema creation (no longer needed)
- Update table references throughout test files

### FR7: Docker Initialization
- Update `docker/docker-compose.yml` database initialization
- Create init script to set up schemas on fresh database
- Ensure migrations run correctly in container

## Acceptance Criteria

1. [ ] Only one PostgreSQL container running locally (`pad_db` on 5432)
2. [ ] `\dn` command shows `padealing` and `bo_airflow` schemas
3. [ ] All 9 audit integration tests pass (no more 500 errors)
4. [ ] All existing unit tests continue to pass (267+ tests)
5. [ ] `.env` and `.env.local` point to single database on port 5432
6. [ ] Root `docker-compose.yml` removed or clearly marked as deprecated
7. [ ] Docker API container starts successfully with new schema
8. [ ] SQLAlchemy models have correct schema assignments

## Out of Scope

- Syncing actual production data from Oracle (bo_airflow will have test seed data only)
- Changes to dev/prod infrastructure
- Performance optimization
- Adding new tables beyond what's needed for PA Dealing

## Technical Notes

### Dev Database Connection (for reference)
```
Host: uk02vddb004.uk.makoglobal.com
Port: 5432
Database: backoffice_db
User: pad_app_user
Schemas: padealing, bo_airflow
```

### Key Files to Modify
- `src/pa_dealing/db/models/core.py` - Model schema definitions
- `docker/docker-compose.yml` - Database service configuration
- `tests/conftest.py` - Test fixture schema setup
- `alembic/versions/` - New migration file
- `.env`, `.env.local` - Database connection strings
