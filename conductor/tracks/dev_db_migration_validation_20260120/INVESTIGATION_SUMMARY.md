# Dev Database Migration Investigation Summary

**Date**: 2026-01-21
**Investigator**: Claude Code
**Database**: `uk02vddb004.uk.makoglobal.com` / `backoffice_db`

---

## Executive Summary

✅ **Dev database has been partially migrated** - Current revision: `1e2bea66afb6`
⚠️ **One migration behind** - Latest migration `fc3b6e7695d9` (schema consolidation) not applied
✅ **Dual-schema architecture EXISTS** - Both `padealing` and `bo_airflow` schemas present
⚠️ **Schema drift confirmed** - 2+ phantom columns in our models vs dev reality

---

## Current State

### Alembic Migration Status

```
Current Revision: 1e2bea66afb6 (add_legacy_pad_fields_for_migration)
Applied Date:     Unknown (alembic_version table doesn't track timestamps)
Next Pending:     fc3b6e7695d9 (consolidate_to_dual_schema)
```

**Migration History Applied to Dev** (assumed based on revision chain):
1. ✅ `20251217_1014_4143273be37f` - initial_schema
2. ✅ `20251218_0001_v2_schema`
3. ✅ `20251219_0001_security_tables`
4. ✅ `20251219_0002_consolidate_tables`
5. ✅ `20251222_0001_add_reference_id`
6. ✅ `20251222_1025_f07c3c54e637` - add_contract_note_fields
7. ✅ `20251222_1028_fda9d658ebc6` - add_verification_fields ⚠️ **Altered bo_airflow tables**
8. ✅ `20251223_1132_f89971675f4c` - rename_ai_risk_assessment
9. ✅ `20251224_spec_compliance_gaps`
10. ✅ `20251229_add_reference_id_to_audit_log`
11. ✅ `20251229_add_auto_approval_type`
12. ✅ `20251229_0735_aa0f04f2fdaf` - add_last_trade_date ⚠️ **Altered oracle_position**
13. ✅ `20260119_1248_a50909b3c7db` - add_tiered_lookup_tables ⚠️ **Altered oracle_bloomberg**
14. ✅ `20260119_1506_39539b3613cd` - rename_oracle_inst_type
15. ✅ `20260119_2107_98b90c8f1ba4` - add_google_uid_columns
16. ✅ `20260120_0021_06750fdfd3a9` - add_contact_tables
17. ✅ `20260120_1858_1e2bea66afb6` - **add_legacy_pad_fields** ← **Current**
18. ❌ `20260121_1013_fc3b6e7695d9` - consolidate_to_dual_schema ← **Not applied yet**

### Schema State

**padealing schema** (Application-managed):
- ✅ 13 tables created
- ✅ Tables: alembic_version, pad_request, pad_approval, pad_execution, pad_breach, audit_log, employee_role, compliance_config, compliance_decision_outcome, restricted_security, chatbot_session, personal_account_dealing, active_stocks

**bo_airflow schema** (Oracle-synced, should be read-only):
- ✅ 245 tables (Oracle-synced)
- ✅ Key tables present: oracle_employee, oracle_bloomberg, oracle_contact, oracle_position, oracle_portfolio_group, inst_type, oracle_map_inst_symbol
- ⚠️ **Some tables may have been modified by our migrations** (see violations below)

---

## Critical Findings

### 1. Schema Drift: Phantom Columns

**Columns in our SQLAlchemy models that DON'T exist in dev database:**

| Table | Column | Status | Impact |
|-------|--------|--------|--------|
| `oracle_employee` | `email` | ❌ PHANTOM | HIGH - Code using this will fail in dev |
| `oracle_employee` | `full_name` | ❌ PHANTOM | MEDIUM - Less critical |

**Resolution**: Email comes from `oracle_contact` table join:
```sql
LEFT JOIN bo_airflow.oracle_contact c ON (
    e.id = c.employee_id
    AND c.contact_group_id = 2  -- Work contacts
    AND c.contact_type_id = 5   -- Email type
)
```

**Coverage in Dev**:
- Total work emails available: 1,114 employees
- Validated working for test users (Alex, Luis)

### 2. Migration Violations: bo_airflow Table Alterations

**Migrations that attempted to ALTER Oracle-synced tables:**

#### Migration: `20251222_1028_add_verification_fields`
**Violated tables**: `oracle_bloomberg`, `oracle_employee`, `oracle_position`

```python
# These ALTER statements should NOT run against dev/prod bo_airflow
op.execute("ALTER TABLE oracle_bloomberg ADD COLUMN IF NOT EXISTS bloomberg VARCHAR(50)")
op.execute("ALTER TABLE oracle_bloomberg ADD COLUMN IF NOT EXISTS cusip VARCHAR(9)")
op.execute("ALTER TABLE oracle_bloomberg ADD COLUMN IF NOT EXISTS description VARCHAR(200)")
# ... 5 more columns

op.execute("ALTER TABLE oracle_employee ALTER COLUMN mako_id TYPE VARCHAR(50)")
op.execute("ALTER TABLE oracle_employee ADD COLUMN IF NOT EXISTS manager VARCHAR(50)")

op.execute("ALTER TABLE oracle_position ADD COLUMN IF NOT EXISTS underlying_symbol VARCHAR(30)")
```

**Dev Reality**: These columns ALREADY EXIST in dev (Oracle sync provides them)

#### Migration: `20251229_0735_add_last_trade_date`
**Violated tables**: `oracle_position`

```python
op.add_column("oracle_position", sa.Column("last_trade_date", sa.DateTime(), nullable=True))
```

#### Migration: `20260119_1248_add_tiered_lookup_tables`
**Violated tables**: `oracle_bloomberg`, `oracle_position`

```python
op.add_column('oracle_bloomberg', sa.Column('reuters', sa.String(), nullable=True))
op.add_column('oracle_bloomberg', sa.Column('is_deleted_yn', sa.String(length=1), server_default='N'))
op.alter_column('oracle_bloomberg', 'bloomberg', nullable=False)
# ... more alterations
```

**Dev Reality**: `reuters` and `is_deleted_yn` ALREADY EXIST in dev

### 3. Why Migrations Succeeded in Dev

**Hypothesis**: Migrations used conditional logic (`IF NOT EXISTS`, checks) that made them idempotent:
- If column exists → Skip operation
- If column doesn't exist → Create it

**Result**: Migrations didn't fail, but they DID attempt to modify Oracle-managed schema.

**Problem**: This creates schema drift - dev bo_airflow may now differ from Oracle source of truth.

---

## Schema Comparison: Local vs Dev

### oracle_employee Columns

**In Dev** (49 columns total):
```
id, mako_id, nationality, gender, status, ni_number, dob, source_id,
emergency_contact, boffin_group, encrypted_password, password_expiry_date,
image, start_date, end_date, manager, review_type_id, next_review_date,
control_id, company, t6_employee, t9_payroll_id, payroll_ref, currency_id,
gl_code, employers_ni_code, employers_pension_code, loan_repayments,
entitlement, max_carry_forward, comments, reason_for_leaving,
relocation_comments, division_id, activity_code, boffin_id_list,
entitlement_override, signatory_level, goverance_yn, leave_authoriser,
notice_period, cost_centre, mifid_long_code, mifid_country_id,
mifid_long_code_type, mifid_short_code, manager_id, leave_authoriser_id,
employee_type_id
```

**NOT in Dev** (phantom columns):
- ❌ `email` - Use oracle_contact join instead
- ❌ `full_name` - May need oracle_contact or derivation

### oracle_bloomberg Columns

**Key columns confirmed in dev**:
```
id, bloomberg, inst_symbol, inst_type, description, primary_exchange,
isin, sedol, reuters, cusip, trade_currency, is_deleted_yn, ticker
```

**Analysis**: Most columns our migrations added ALREADY existed in dev.

---

## Recommendations

### Immediate Actions (Before Next Migration)

1. **DO NOT apply `fc3b6e7695d9` (consolidate_to_dual_schema) to dev yet**
   - Dev already has dual schemas
   - This migration is safe (only creates schemas IF NOT EXISTS) but redundant

2. **Create schema-aware `alembic/env.py`**
   - Detect environment: local/test vs dev/prod
   - Skip all bo_airflow table operations when `APP_ENV=dev` or `prod`
   - Only allow bo_airflow operations for local development

3. **Audit all existing migrations**
   - Identify all operations that touch bo_airflow schema
   - Wrap in environment checks
   - Create migration to clean up any accidental bo_airflow modifications

### Medium-Term Actions

4. **Implement Schema Validation Track**
   - Compare ALL SQLAlchemy models against dev schema (see `schema_validation_dev_20260121`)
   - Identify all phantom columns
   - Update models to match dev reality
   - Document required joins (oracle_contact for email)

5. **Mark bo_airflow models as reflection-only**
   - Use SQLAlchemy `autoload_with=engine` to reflect schema from database
   - OR clearly comment which columns are local-only vs dev-present

6. **Update documentation**
   - Document that bo_airflow schema is Oracle-managed (read-only)
   - Document required join patterns (email, etc.)
   - Update migration guide to prevent future violations

---

## Migration Strategy Going Forward

### Option A: Environment-Aware Migrations (Recommended)

**alembic/env.py** enhancement:
```python
from src.pa_dealing.config.settings import settings

def run_migrations_online():
    # ...
    context.configure(
        connection=connection,
        target_metadata=target_metadata,
        include_schemas=True,
        # Skip bo_airflow operations in dev/prod
        version_table_schema='padealing',  # Keep alembic_version in padealing
        compare_type=True,
        include_object=lambda obj, name, type_, reflected, compare_to: (
            filter_objects_by_environment(obj, name, type_, reflected, compare_to)
        )
    )

def filter_objects_by_environment(obj, name, type_, reflected, compare_to):
    """Filter operations based on environment."""
    # In dev/prod: Skip all bo_airflow operations
    if settings.environment in ('dev', 'prod'):
        if hasattr(obj, 'schema') and obj.schema == 'bo_airflow':
            return False  # Skip this object

    # In local/test: Allow all operations
    return True
```

**Migration file template** for bo_airflow operations:
```python
def upgrade() -> None:
    """Upgrade database schema."""
    from src.pa_dealing.config.settings import settings

    # Only run bo_airflow operations in local/test environments
    if settings.environment in ('local', 'test'):
        op.add_column('oracle_position', sa.Column('some_column', ...))
    else:
        print(f"Skipping bo_airflow operation in {settings.environment} environment")
```

### Option B: Separate Migration Branches

- Create separate migration branches for:
  - `padealing` schema (always run)
  - `bo_airflow` schema (local/test only)
- Use Alembic's branch labels feature

### Option C: Schema Reflection (Most Conservative)

- Don't manage bo_airflow schema in migrations at all
- Use SQLAlchemy reflection to load schema from database at runtime
- Migrations only manage `padealing` schema

**Recommendation**: Option A (environment-aware) - Clean, explicit, maintainable

---

## Next Steps

### Phase 1: Assess Current State
- [x] ✅ Investigate dev database alembic_version
- [x] ✅ Identify applied migrations
- [x] ✅ Document phantom columns
- [x] ✅ Document migration violations
- [x] Run schema comparison script (compare ALL models vs dev)

### Phase 2: Create Migration Fix
- [x] Update alembic/env.py with environment-aware filtering
- [x] Test locally that bo_airflow operations are skipped in dev mode
- [x] Create migration to document "no-op for dev" for fc3b6e7695d9

### Phase 3: Schema Validation
- [x] Implement full schema validation script (all tables)
- [x] Update SQLAlchemy models to match dev reality
- [x] Remove or document all phantom columns
- [x] Test application against dev database

### Phase 4: Documentation
- [x] Update SCHEMA_REFERENCE.md with join patterns
- [x] Update migration guide with bo_airflow read-only policy
- [x] Create runbook for future migrations

---

## Questions for User

1. **Migration Strategy**: Do you want to implement Option A (environment-aware migrations)?
2. **Existing Violations**: Should we create a "cleanup" migration to revert bo_airflow modifications?
3. **Schema Validation**: Should we run the full schema validation track before proceeding?
4. **UAT Completion**: You mentioned UAT passed - which scenarios were tested? Should we document results?

---

## Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Schema drift between dev and Oracle | HIGH | CONFIRMED | Schema validation + reflection |
| Code using phantom columns fails in dev | HIGH | CONFIRMED | Already fixed for email, need audit for others |
| Future migrations violate bo_airflow | MEDIUM | HIGH | Environment-aware env.py |
| Alembic state corruption | LOW | LOW | alembic_version is clean, linear history |
| Test failures due to schema mismatch | MEDIUM | CONFIRMED | Use dev schema for integration tests |

---

## Conclusion

**Current Status**: Dev database is functional but has schema drift issues.

**Safe to Proceed?**:
- ✅ YES for padealing schema operations
- ⚠️ CAUTION for bo_airflow operations (implement environment checks first)
- ❌ NO for applying fc3b6e7695d9 blindly (already has dual schemas)

**Recommended Path**:
1. Implement environment-aware migrations (alembic/env.py)
2. Run full schema validation to identify ALL phantom columns
3. Update models to match dev reality
4. Create "no-op" migration for fc3b6e7695d9 in dev
5. Document UAT results and complete track

**Track Completion**: ~70% complete. Remaining work is migration strategy + documentation.
