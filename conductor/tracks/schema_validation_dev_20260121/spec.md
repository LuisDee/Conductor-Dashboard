# Specification: Schema Validation Against Dev Database

## Overview

Validate that our SQLAlchemy models and code queries match the actual schema structure in dev database. Prevents "hallucinating" columns that don't exist and ensures joins align with production data.

**Priority**: Medium - Quality assurance
**Type**: Validation & Documentation
**Dependencies**: None
**Trigger**: Recent discovery that `oracle_employee.email` and `oracle_employee.full_name` don't exist in dev

---

## Context

### Problem Statement

During test debugging, we discovered our local SQLAlchemy models define columns that **don't exist in dev**:
- `oracle_employee.email` - ❌ Doesn't exist in dev
- `oracle_employee.full_name` - ❌ Doesn't exist in dev

This caused:
1. Code that worked locally to fail against dev
2. False assumptions about data availability
3. Test failures that were hard to diagnose

**Root Cause**: Schema drift between local models and dev reality.

### Dev Database Environment

- **Host**: `uk02vddb004.uk.makoglobal.com`
- **Database**: `backoffice_db`
- **Schemas**: `bo_airflow` (Oracle sync), `padealing` (application)
- **Access**: `pad_app_user` / `padd_app_pass`

---

## Objectives

1. **Schema Introspection**: Query dev database to get actual table schemas
2. **Model Validation**: Compare SQLAlchemy models against dev schema
3. **Query Validation**: Verify all SQL queries reference real columns
4. **Documentation**: Document schema differences and required joins
5. **Prevention**: Create automated validation script for future changes

---

## Functional Requirements

### FR-1: Schema Introspection Script

Create `scripts/validate_schema_against_dev.py` that:
- Connects to dev database
- Queries `information_schema.columns` for all tables in `bo_airflow` and `padealing`
- Extracts: table_name, column_name, data_type, is_nullable, column_default
- Saves to JSON: `schema_snapshots/dev_schema_YYYYMMDD.json`

### FR-2: Model Comparison

Compare SQLAlchemy models against dev schema:
- Read model definitions from `src/pa_dealing/db/models/`
- Extract defined columns (using SQLAlchemy metadata)
- Compare against dev schema snapshot
- Report:
  - ✅ Columns that match
  - ⚠️ Columns in model but NOT in dev (false assumptions)
  - ℹ️ Columns in dev but NOT in model (unused fields)

### FR-3: Query Validation

Scan codebase for SQL queries and validate:
- All `text()` queries in `src/pa_dealing/`
- Extract table references and column names
- Check against dev schema
- Report queries that reference non-existent columns

### FR-4: Required Joins Documentation

Document critical joins required by dev schema:
- **Email Resolution**: MUST join `oracle_contact` (employee.email doesn't exist)
- **Name Resolution**: TBD (full_name doesn't exist, may need oracle_contact or derivation)
- Other joins as discovered

### FR-5: Automated Validation in CI/CD

Add to CI/CD pipeline:
- Run schema validation on every PR
- Fail if new queries reference non-existent columns
- Update schema snapshot monthly (or when dev schema changes)

---

## Discovered Schema Differences

### oracle_employee (bo_airflow schema)

**Columns that DON'T exist in dev** (but defined in local model):
- ❌ `email` - Email comes from oracle_contact only
- ❌ `full_name` - Name may come from oracle_contact or be NULL

**Columns that DO exist in dev** (validated):
- ✅ `id` (bigint, primary key)
- ✅ `mako_id` (text, unique)
- ✅ `manager_id` (bigint, foreign key)
- ✅ `manager` (text, legacy mako_id reference)
- ✅ `status` (text)
- ✅ `company` (text)
- ✅ `cost_centre` (text)
- ✅ `start_date` (timestamp)
- ✅ `end_date` (timestamp)
- ✅ `boffin_group` (text)
- ✅ `employee_type_id` (bigint)
- ... (many Oracle-specific fields)

### oracle_contact (bo_airflow schema)

**Critical fields** (validated in dev):
- ✅ `id` (bigint, primary key)
- ✅ `employee_id` (bigint, foreign key to oracle_employee)
- ✅ `contact_group_id` (bigint) - **Filter: 2 = Work contacts**
- ✅ `contact_type_id` (bigint) - **Filter: 5 = Email type**
- ✅ `email` (text) - **Source of truth for employee emails**
- ✅ `forename` (text)
- ✅ `surname` (text)

**Required Join Pattern**:
```sql
FROM bo_airflow.oracle_employee e
LEFT JOIN bo_airflow.oracle_contact c ON (
    e.id = c.employee_id
    AND c.contact_group_id = 2  -- Work Group
    AND c.contact_type_id = 5   -- Email Type
)
```

**Statistics from Dev** (2026-01-21):
- Total active employees: Unknown (query returned 0, needs investigation)
- Employees with contact emails (group=2, type=5): 481
- Email coverage: Validated working for Alex and Luis

---

## Acceptance Criteria

1. [ ] Schema introspection script created and tested against dev
2. [ ] Dev schema snapshot saved to `schema_snapshots/dev_schema_20260121.json`
3. [ ] Model comparison report generated
4. [ ] All phantom columns documented
5. [ ] All SQL queries validated against dev schema
6. [ ] Required joins documented in `docs/SCHEMA_REFERENCE.md`
7. [ ] CI/CD validation added (optional, future enhancement)
8. [ ] SQLAlchemy models updated to match dev reality

---

## Out of Scope

- Modifying dev database schema
- Changing Oracle sync processes
- Adding columns to dev that only exist locally
- Backfilling missing data in dev

---

## Success Metrics

- Zero queries that reference non-existent columns
- All model columns exist in dev OR documented as local-only
- Automated validation prevents future drift
- Team confidence in schema alignment

---

## Implementation Notes

### Script Template

```python
# scripts/validate_schema_against_dev.py
import asyncio
import asyncpg
import json
from pathlib import Path

async def introspect_dev_schema():
    conn = await asyncpg.connect(
        host='uk02vddb004.uk.makoglobal.com',
        database='backoffice_db',
        user='pad_app_user',
        password='padd_app_pass'
    )

    # Get all columns from bo_airflow and padealing schemas
    rows = await conn.fetch("""
        SELECT
            table_schema,
            table_name,
            column_name,
            data_type,
            is_nullable,
            column_default
        FROM information_schema.columns
        WHERE table_schema IN ('bo_airflow', 'padealing')
        ORDER BY table_schema, table_name, ordinal_position
    """)

    # Save snapshot
    snapshot = {"timestamp": "...", "tables": {...}}
    Path("schema_snapshots").mkdir(exist_ok=True)
    with open("schema_snapshots/dev_schema_20260121.json", "w") as f:
        json.dump(snapshot, f, indent=2)

    await conn.close()
```

### Validation Rules

1. **Model columns**: All `Mapped[]` fields must exist in dev OR be marked `local_only=True`
2. **Raw SQL queries**: All columns in `text()` queries must exist in dev
3. **Joins**: All FK relationships must match dev schema

---

## References

- Dev database specs: `conductor/tracks/dev_db_migration_validation_20260120/spec.md`
- Oracle employee model: `src/pa_dealing/db/models/core.py`
- Oracle contact model: `src/pa_dealing/db/models/core.py`
- Recent fix: GoogleIdentityProvider now uses oracle_contact join for email resolution

---

**Created**: 2026-01-21
**Status**: Specification complete, ready for implementation
