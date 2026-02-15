# Action Plan: Complete Dev Database Migration Track

**Status**: Investigation Complete - Ready for Implementation
**Date**: 2026-01-21
**Track Priority**: HIGHEST (CRITICAL)

---

## What We Discovered

✅ **Good News**:
- Dev database IS connected and working
- UAT scenarios passed (as you confirmed)
- 329/329 tests passing locally
- Dual-schema architecture (`padealing` + `bo_airflow`) exists in dev

⚠️ **Issues Found**:
- Alembic migrations have been modifying `bo_airflow` tables (Oracle-synced, should be read-only)
- 2+ phantom columns in our models (`oracle_employee.email`, `oracle_employee.full_name`)
- Dev is 1 migration behind local (missing `fc3b6e7695d9` schema consolidation)
- Schema drift confirmed between our models and dev reality

---

## Remaining Work to Complete Track

Based on the spec's success criteria, here's what's left:

| Requirement | Status | Notes |
|-------------|--------|-------|
| 1. Connect to dev database | ✅ DONE | Working connection |
| 2. Apply migrations to dev | ⚠️ **NEEDS FIX** | One pending, need environment-aware approach |
| 3. 100% test pass rate | ✅ DONE | 329/329 passing |
| 4. 10 UAT scenarios validated | ✅ DONE | User confirmed UAT passed |
| 5. User sign-off on UAT | ⏳ PENDING | Need to document which scenarios were tested |
| 6. Documentation | ⏳ PENDING | Need to document migration approach |
| 7. Active_stocks investigation | ⏳ PENDING | Table exists in dev, need to determine purpose |

---

## Option 1: Minimal Path (Fast Track Completion)

**Goal**: Mark track complete with current state, defer migration fixes to future track

### Steps:

1. **Document UAT Results** (30 min)
   - You confirmed UAT passed - which scenarios (1-10) were tested?
   - Create UAT_RESULTS.md with sign-off
   - Mark spec.md checklist items as complete

2. **Investigate active_stocks Table** (15 min)
   - Quick search: `grep -r "active_stocks" src/ tests/`
   - Decision: Keep or drop? (likely historical, can drop)
   - Document in investigation summary

3. **Document "As-Is" State** (30 min)
   - Accept that dev is 1 migration behind (fc3b6e7695d9 not needed - schemas exist)
   - Document bo_airflow schema drift as known issue
   - Create follow-up track for "Schema Reconciliation & Migration Strategy"

4. **Update Track Metadata** (10 min)
   - Mark track as complete in tracks.md
   - Update metadata.json status
   - Create follow-up track: "Dev Database Schema Reconciliation"

**Total Time**: ~1.5 hours
**Outcome**: Track complete, issues deferred to new track

---

## Option 2: Complete Fix (Thorough Approach)

**Goal**: Fix all identified issues before marking complete

### Steps:

1. **Create Environment-Aware Migration System** (2 hours)
   - Update `alembic/env.py` to detect APP_ENV
   - Add filter to skip bo_airflow operations in dev/prod
   - Test locally with APP_ENV=dev simulation

2. **Create "No-Op" Migration for Dev** (30 min)
   - Migration fc3b6e7695d9 (consolidate_to_dual_schema) is redundant for dev
   - Create conditional version that skips schema creation if already exists
   - Apply to dev: `alembic upgrade head`

3. **Run Full Schema Validation** (1 hour)
   - Extend `/tmp/investigate_dev_schema.py` to check ALL models
   - Compare every SQLAlchemy model column against dev schema
   - Document ALL phantom columns (not just oracle_employee)

4. **Update SQLAlchemy Models** (1-2 hours)
   - Remove or document phantom columns
   - Add comments marking which columns are local-only
   - Update code to use oracle_contact join for email everywhere

5. **Document UAT Results** (30 min)
   - Same as Option 1

6. **Investigate active_stocks** (15 min)
   - Same as Option 1

7. **Create Migration Guide** (1 hour)
   - Document bo_airflow read-only policy
   - Document environment-aware migration approach
   - Create SCHEMA_REFERENCE.md with required joins

**Total Time**: ~6-8 hours
**Outcome**: Track fully complete, no technical debt

---

## Option 3: Hybrid Approach (Recommended)

**Goal**: Fix critical issues now, defer nice-to-haves

### Steps:

#### Phase A: Critical Fixes (Complete Now)

1. **Document Current State** (30 min)
   - Accept dev is 1 migration behind (schemas exist, no action needed)
   - Document that fc3b6e7695d9 should NOT be applied to dev (redundant)
   - Create MIGRATION_STATUS.md explaining state

2. **Document UAT Results** (30 min)
   - Which scenarios were tested?
   - Get user sign-off
   - Mark spec checklist complete

3. **Investigate active_stocks** (15 min)
   - Quick determination: keep or drop

4. **Document Phantom Columns** (15 min)
   - oracle_employee.email ❌ (use oracle_contact)
   - oracle_employee.full_name ❌ (use oracle_contact)
   - Document in KNOWN_ISSUES.md

**Subtotal**: ~1.5 hours

#### Phase B: Future Track (Defer)

5. **Create Follow-Up Track**: "Schema Reconciliation & Migration Hardening"
   - Implement environment-aware migrations
   - Full schema validation (all models)
   - Update models to match dev reality
   - Create migration guide

**Total Time Now**: ~1.5 hours
**Outcome**: Track complete with documented known issues, follow-up track created

---

## Recommendation: **Option 3 (Hybrid)**

**Why**:
- ✅ Application works in dev (UAT passed)
- ✅ Tests pass (329/329)
- ✅ Critical functionality validated
- ⚠️ Schema issues are known, documented, and non-blocking
- ⚠️ Migration strategy needs improvement but not urgent

**What we accomplish**:
- Mark this track complete (unblock dependent tracks)
- Document current state transparently
- Create proper follow-up track for technical debt

**What we defer**:
- Environment-aware migration system
- Full schema validation (all tables)
- Model cleanup for phantom columns
- Comprehensive migration guide

---

## Decision Point

**Question for you**:
1. Which option do you prefer? (1, 2, or 3)
2. For UAT: Which scenarios (1-10 from spec.md) did you test?
3. For active_stocks: Do you know what this table is for? Should we keep it?

**My Recommendation**: Option 3
- Fast path to completion (~1.5 hours)
- Documents current state honestly
- Creates clean follow-up track for improvements
- Unblocks other work (conflict detection, legacy field review)

---

## Next Immediate Actions (Awaiting Your Decision)

**If Option 1 or 3**:
1. Tell me which UAT scenarios you tested (1-10)
2. I'll document UAT results
3. I'll investigate active_stocks
4. I'll create completion documentation
5. We mark track complete and move on

**If Option 2**:
1. I'll start building environment-aware migration system
2. Full schema validation
3. Model updates
4. Then UAT documentation
5. Then mark complete

**Your call!** What would you like to do?
