# Spec: Fix PA Search Division Lookup

## Problem Statement

The PA Search page displays incorrect division data for employees. Instead of
showing the canonical department name (e.g., "Trading", "Risk", "Technology"),
it shows the raw `boffin_group` field from `oracle_employee` — a semicolon-delimited
string of ALL group memberships (e.g., "developer;accounting;backoffice;brokerage;...").

### Root Cause

The `search_pa_trading()` service method in `pad_search.py` resolves division using:

```python
COALESCE(oracle_employee.boffin_group, oracle_employee.cost_centre)
```

This is incorrect. The correct lookup chain (already used by `risk_scoring_service.py`)
is:

```
oracle_employee.division_id → oracle_department.id → oracle_department.name
```

### Current State
- `pad_search.py` uses `COALESCE(boffin_group, cost_centre)` — wrong source
- `OracleEmployee` model in `core.py` does not map `division_id` — column exists in DB but is unmapped
- `risk_scoring_service.py` already uses the correct `division_id → oracle_department` lookup via raw SQL
- Frontend Division filter chip shows the full raw semicolon string
- Frontend `dblClickCell` passes the raw multi-value string as the filter value

### Expected State
- `pad_search.py` joins `oracle_employee.division_id → oracle_department.name` to get division
- `OracleEmployee` model maps `division_id` so it's available via ORM
- Division column shows clean department names (e.g., "Trading", "Technology")
- Division filter works correctly (clean single values, no semicolons)
- Fallback to `boffin_group` first value if `division_id` is NULL

## Functional Requirements

### FR-1: Add division_id to OracleEmployee Model
- Map the existing `division_id` database column in the SQLAlchemy model (`core.py`)
- Column type: `BigInteger`, nullable, foreign key to `oracle_department.id`

### FR-2: Update PA Search Division Subquery
- Replace `COALESCE(boffin_group, cost_centre)` with a join to `oracle_department`
- Use `oracle_employee.division_id → oracle_department.name` as the primary source
- Fallback: if `division_id` is NULL or no matching department, fall back to
  first semicolon-delimited value from `boffin_group`, then `cost_centre`
- The SQL pattern already exists in `risk_scoring_service.py:_lookup_department_name()`

### FR-3: Update Unit Tests
- Update existing PAD search tests to reflect the new division lookup
- Add test cases for: division_id present, division_id NULL (fallback), both NULL

## Out of Scope
- Creating an ORM model for `oracle_department` (raw SQL subquery is sufficient,
  consistent with existing risk_scoring_service.py pattern)
- Frontend changes (clean department names eliminate the filter bug automatically)
- Alembic migrations (column already exists in database)
- Changes to risk_scoring_service.py (already correct)

## Acceptance Criteria
- [ ] PA Search Division column shows canonical department names
- [ ] Division filter chip shows clean single value (e.g., "Trading")
- [ ] Fallback works when division_id is NULL
- [ ] Unit tests pass with >80% coverage on changed code
- [ ] No regression in existing PAD search functionality
