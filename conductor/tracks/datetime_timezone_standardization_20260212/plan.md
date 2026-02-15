# Implementation Plan: Datetime & Timezone Standardization

## Executive Summary

Replace all deprecated `datetime.utcnow()` calls with timezone-aware `datetime.now(UTC)` across the codebase. This eliminates Python 3.12 deprecation warnings and ensures all timestamps are timezone-aware.

**Scope:**
- 21 calls across 10 source files in `src/pa_dealing/`
- 10 calls across 3 test files in `tests/`
- Add ruff DTZ rules to prevent regression

**Risk Level:** LOW - Mechanical string replacement with identical behavior
**Estimated Duration:** 30-45 minutes
**Breaking Changes:** None (backward compatible)

---

## Phase 1: Enable Ruff Datetime Linting (5 min)

### 1.1 Add DTZ Rules to pyproject.toml

**File:** `/Users/luisdeburnay/work/rules_engine_refactor/pyproject.toml`

**Change:**
```toml
# BEFORE (line 94-105)
[tool.ruff.lint]
select = [
    "E",   # pycodestyle errors
    "F",   # Pyflakes
    "I",   # isort
    "N",   # pep8-naming
    "W",   # pycodestyle warnings
    "UP",  # pyupgrade
    "B",   # flake8-bugbear
    "C4",  # flake8-comprehensions
    "SIM", # flake8-simplify
    "RUF", # Ruff-specific rules
]

# AFTER
[tool.ruff.lint]
select = [
    "E",   # pycodestyle errors
    "F",   # Pyflakes
    "I",   # isort
    "N",   # pep8-naming
    "W",   # pycodestyle warnings
    "UP",  # pyupgrade
    "B",   # flake8-bugbear
    "C4",  # flake8-comprehensions
    "SIM", # flake8-simplify
    "RUF", # Ruff-specific rules
    "DTZ", # flake8-datetimez (forbids utcnow, naive now, etc.)
]
```

**Verification:**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor
ruff check src/ tests/ --select DTZ 2>&1 | grep -c "DTZ003"
# Expected: 31 violations (all utcnow calls)
```

**Why DTZ?**
- `DTZ003`: Forbids `datetime.utcnow()` usage
- `DTZ001`: Forbids naive `datetime.now()` without timezone
- `DTZ005`: Forbids naive `datetime.now().replace()` without timezone
- Prevents future regression after migration

---

## Phase 2: Source Files Migration (15-20 min)

### Strategy
Files fall into two categories:
1. **Already has UTC import** (1 file): Just replace `utcnow()` → `now(UTC)`
2. **Needs UTC import** (9 files): Add import, then replace calls

### 2.1 Files Already Importing UTC

#### File: `src/pa_dealing/db/repository.py` (2 calls)

**Current import (line 3):**
```python
from datetime import UTC, date, datetime, timedelta
```

**Changes needed:**
```python
# Line 682 (inside check_holding_period function)
- current_date = datetime.utcnow()
+ current_date = datetime.now(UTC)

# Line 1124 (inside get_pending_approvals function)
- expiry_threshold = datetime.utcnow()
+ expiry_threshold = datetime.now(UTC)
```

**Verification:**
```bash
grep -n "utcnow" src/pa_dealing/db/repository.py
# Expected: no output
```

---

### 2.2 Files Needing UTC Import (9 files)

#### File: `src/pa_dealing/services/notification_outbox.py` (7 calls - HIGHEST PRIORITY)

**Add import (line 15):**
```python
# BEFORE
from datetime import datetime, timedelta

# AFTER
from datetime import UTC, datetime, timedelta
```

**Replace all 7 calls:**
```python
# Line 63 (queue_notification function)
- next_attempt_at=datetime.utcnow(),
+ next_attempt_at=datetime.now(UTC),

# Line 83 (process_outbox_batch function)
- now = datetime.utcnow()
+ now = datetime.now(UTC)

# Line 104 (process_outbox_batch function)
- entry.sent_at = datetime.utcnow()
+ entry.sent_at = datetime.now(UTC)

# Line 136 (_calculate_next_attempt function)
- return datetime.utcnow()
+ return datetime.now(UTC)

# Line 138 (_calculate_next_attempt function)
- return datetime.utcnow() + timedelta(minutes=backoff_minutes)
+ return datetime.now(UTC) + timedelta(minutes=backoff_minutes)

# Line 170 (_mark_failed function)
- entry.last_attempt_at = datetime.utcnow()
+ entry.last_attempt_at = datetime.now(UTC)

# Line 299 (get_recent_success_count function)
- .where(NotificationOutbox.sent_at >= datetime.utcnow().replace(hour=0, minute=0, second=0))
+ .where(NotificationOutbox.sent_at >= datetime.now(UTC).replace(hour=0, minute=0, second=0))
```

**Verification:**
```bash
grep -n "utcnow" src/pa_dealing/services/notification_outbox.py
# Expected: no output
```

---

#### File: `src/pa_dealing/api/routes/pdf_history.py` (3 calls)

**Add import:**
```python
# Find existing datetime import, update to:
from datetime import UTC, datetime, timedelta
```

**Replace all 3 calls:**
```bash
# Use sed for batch replacement
sed -i '' 's/datetime\.utcnow()/datetime.now(UTC)/g' src/pa_dealing/api/routes/pdf_history.py
```

**Verification:**
```bash
grep -n "utcnow" src/pa_dealing/api/routes/pdf_history.py
# Expected: no output
```

---

#### File: `src/pa_dealing/services/rules_engine/service.py` (2 calls - REGULATORY)

**Add import:**
```python
from datetime import UTC, datetime
```

**Replace both calls:**
```bash
sed -i '' 's/datetime\.utcnow()/datetime.now(UTC)/g' src/pa_dealing/services/rules_engine/service.py
```

**Context:** Lines 168 and 231 - Rule update timestamps for regulatory tracking

---

#### File: `src/pa_dealing/services/restricted_instruments.py` (2 calls)

**Add import:**
```python
from datetime import UTC, datetime
```

**Replace both calls:**
```bash
sed -i '' 's/datetime\.utcnow()/datetime.now(UTC)/g' src/pa_dealing/services/restricted_instruments.py
```

---

#### Remaining Single-Call Files (5 files - 1 call each)

**Batch command for all 5 files:**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Files: admin.py, extraction_router.py, gcs_client.py, smart_matcher.py, config.py
for file in \
  src/pa_dealing/api/routes/admin.py \
  src/pa_dealing/services/extraction_router.py \
  src/pa_dealing/services/gcs_client.py \
  src/pa_dealing/services/smart_matcher.py \
  src/pa_dealing/api/routes/config.py; do

  # Step 1: Add UTC import (find datetime import line, add UTC)
  # Most files have: from datetime import datetime
  # Update to: from datetime import UTC, datetime
  sed -i '' 's/from datetime import datetime/from datetime import UTC, datetime/' "$file"

  # Step 2: Replace utcnow() call
  sed -i '' 's/datetime\.utcnow()/datetime.now(UTC)/g' "$file"

  # Verify
  echo "Checking $file..."
  grep -n "utcnow" "$file" || echo "  ✓ Clean"
done
```

**Manual verification for complex imports:**
Some files may have multi-line imports like:
```python
from datetime import (
    datetime,
    timedelta,
)
```

For these, manually add `UTC,` to the import list.

---

## Phase 3: Test Files Migration (10 min)

### 3.1 Unit Test File

#### File: `tests/unit/test_pdf_history_api.py` (5 calls)

**Add import:**
```python
from datetime import UTC, datetime
```

**Batch replace:**
```bash
sed -i '' 's/datetime\.utcnow()/datetime.now(UTC)/g' tests/unit/test_pdf_history_api.py
grep -n "utcnow" tests/unit/test_pdf_history_api.py
# Expected: no output
```

---

### 3.2 Integration Test Files (2 files)

#### File: `tests/integration/test_pdf_poller_integration.py` (4 calls)

**Add import:**
```python
from datetime import UTC, datetime
```

**Batch replace:**
```bash
sed -i '' 's/datetime\.utcnow()/datetime.now(UTC)/g' tests/integration/test_pdf_poller_integration.py
```

---

#### File: `tests/integration/test_schema_contracts.py` (1 call)

**Add import:**
```python
from datetime import UTC, datetime
```

**Batch replace:**
```bash
sed -i '' 's/datetime\.utcnow()/datetime.now(UTC)/g' tests/integration/test_schema_contracts.py
```

---

## Phase 4: Final Verification (5-10 min)

### 4.1 Grep Verification

```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Should return ZERO source/test files
grep -r "datetime\.utcnow()" src/ tests/
# Expected output: (empty)

# Should return only documentation/plan files
grep -r "datetime\.utcnow()" . --include="*.md" --include="*.txt"
# Expected: only conductor/ and .autopsy/ files
```

### 4.2 Ruff Verification

```bash
# Should return ZERO DTZ003 violations
ruff check src/ tests/ --select DTZ003
# Expected output: "All checks passed!"

# Full DTZ check (may flag naive datetime usage elsewhere - acceptable for now)
ruff check src/ tests/ --select DTZ
```

### 4.3 Import Verification

```bash
# Verify all files have UTC imported
for file in $(grep -l "datetime.now(UTC)" src/pa_dealing/**/*.py); do
  grep -q "from datetime import.*UTC" "$file" || echo "Missing UTC import: $file"
done
# Expected: no output
```

### 4.4 Test Suite Verification

```bash
# Run critical test suites
cd /Users/luisdeburnay/work/rules_engine_refactor

# Unit tests (fast)
pytest tests/unit/test_pdf_history_api.py -v

# Integration tests with datetime logic
pytest tests/integration/test_pdf_poller_integration.py -v
pytest tests/integration/test_schema_contracts.py -v

# Full test suite (if time permits)
pytest tests/ -v --tb=short
```

**Expected:** All tests pass unchanged (datetime.now(UTC) returns equivalent values)

---

## Rollback Strategy

### Simple Git Revert
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Create safety branch first
git checkout -b datetime-migration-backup

# After migration, if issues arise:
git checkout main  # or your working branch
git revert HEAD    # revert the migration commit

# Or if not yet committed:
git checkout -- pyproject.toml src/ tests/
```

### Verification After Rollback
```bash
grep -r "datetime\.utcnow()" src/ tests/ | wc -l
# Expected: 31 (back to original count)
```

---

## Edge Cases & Considerations

### 1. SQLAlchemy Default Values
Some models may use `default=datetime.utcnow` in Column definitions. These are **NOT** included in this migration because:
- They're function references, not function calls: `default=datetime.utcnow` (correct)
- Not `default=datetime.utcnow()` (incorrect - would freeze timestamp)
- Will be handled separately when SQLAlchemy 2.x datetime column defaults are standardized

### 2. pytest Filterwarnings
`pyproject.toml` currently suppresses utcnow warnings from SQLAlchemy and Google libraries (lines 83-86). These can be **removed after migration**:

```toml
# Can be removed after migration completes
filterwarnings = [
    "ignore:utcnow\\(\\) is deprecated:DeprecationWarning:sqlalchemy.*",
    "ignore:utcnow\\(\\) is deprecated:DeprecationWarning:google.*",
    # ... keep other filters
]
```

### 3. Timezone-Aware vs Naive Datetime Mixing
After migration, some code may compare timezone-aware datetimes (from `now(UTC)`) with naive datetimes (from legacy sources). This will raise `TypeError: can't compare offset-naive and offset-aware datetimes`.

**Mitigation:**
- All database `DateTime` columns should store timezone-aware datetimes
- If legacy data exists, add `.replace(tzinfo=UTC)` to convert naive → aware
- Ruff DTZ001 will catch future naive datetime.now() calls

### 4. Datetime.replace() Calls
Line 299 in `notification_outbox.py` uses `.replace(hour=0, minute=0, second=0)` which is safe because:
- `datetime.now(UTC).replace()` preserves timezone info
- Only modifies time components, not timezone

---

## Success Criteria

- [ ] Zero `datetime.utcnow()` calls in `src/` and `tests/` directories
- [ ] Zero DTZ003 violations from `ruff check --select DTZ003`
- [ ] All modified files have `from datetime import UTC` (or `timezone`)
- [ ] Full test suite passes without new failures
- [ ] No timezone-aware/naive comparison errors in logs
- [ ] pyproject.toml has `"DTZ"` in ruff select rules

---

## Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Enable DTZ rules | 5 min | None |
| Phase 2: Source files (10 files) | 15-20 min | Phase 1 |
| Phase 3: Test files (3 files) | 10 min | Phase 2 |
| Phase 4: Verification | 5-10 min | Phase 3 |
| **Total** | **35-45 min** | Sequential |

---

## Post-Migration Tasks

1. **Remove pytest filterwarnings** for utcnow deprecation (lines 83-84 in pyproject.toml)
2. **Document timezone strategy** in `docs/DATETIME_HANDLING.md`:
   - Always use `datetime.now(UTC)` for timestamps
   - Always use timezone-aware datetimes in database models
   - Use `datetime.fromisoformat()` for parsing ISO strings (preserves timezone)
3. **Add pre-commit hook** (optional) to run `ruff check --select DTZ` before commits

---

## Notes

- **Python 3.11 compatibility:** `UTC` constant available in Python 3.11+ (current: 3.11)
- **Alternative for Python 3.10:** Use `from datetime import timezone` and `datetime.now(timezone.utc)`
- **No database migration needed:** All datetime columns already store timezone info (PostgreSQL `TIMESTAMP WITH TIME ZONE`)
- **Docker containers:** No config changes needed, migration is code-only
