# Implementation Plan: Import Path Standardization

## Executive Summary

Replace all `from src.pa_dealing` imports with `from pa_dealing` across the codebase. The `src/` prefix is unnecessary and causes confusion since the package is installed via `pip install -e .` which makes `pa_dealing` directly importable.

**Scope:**
- 58 imports across 13 source files in `src/pa_dealing/`
- 68 imports across 20 test files in `tests/`
- Add ruff configuration to prevent regression

**Risk Level:** LOW - Mechanical find/replace, verified by test suite
**Estimated Duration:** 25-35 minutes
**Breaking Changes:** None (both import paths currently work)

---

## Phase 1: Verify Package Configuration (5 min)

### 1.1 Understand Current Setup

**File:** `/Users/luisdeburnay/work/rules_engine_refactor/pyproject.toml`

**Current configuration (lines 73-74):**
```toml
[tool.hatch.build.targets.wheel]
packages = ["src/pa_dealing", "scripts"]
```

**This means:**
- Package is installed from `src/pa_dealing/` directory
- After `pip install -e .`, Python can import via `from pa_dealing ...`
- The `src/` prefix in imports is **redundant** and **incorrect**

**ruff isort config (missing but needed):**
```toml
# Currently absent - need to add:
[tool.ruff.lint.isort]
known-first-party = ["pa_dealing"]
```

### 1.2 Verify Import Resolution

**Test in Docker environment:**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Start Docker container with current code
docker compose up -d postgres
docker compose run --rm backend bash

# Inside container:
python3 -c "from pa_dealing.db.models import PADRequest; print('✓ Direct import works')"
python3 -c "from src.pa_dealing.db.models import PADRequest; print('✗ Should not work')"
# Expected: First succeeds, second fails (or succeeds if src/ in sys.path by accident)

# Check installed package
pip list | grep pa-dealing
# Expected: pa-dealing 0.1.0 (editable install)

exit
```

**Test locally:**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor
python3 -c "import sys; print('\n'.join(sys.path))"
# Check if 'src/' is in path

python3 -c "from pa_dealing.db.models import PADRequest; print('✓ Works')"
```

---

## Phase 2: Source Files Migration (10-12 min)

### Strategy
All 13 source files use **lazy imports** inside function bodies (not top-level). This pattern is used to avoid circular import issues.

### 2.1 Files with Most Imports (Priority Order)

#### File: `src/pa_dealing/agents/orchestrator/agent.py` (13 imports)

**All lazy imports inside function bodies.**

**Batch replace command:**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

sed -i '' 's/from src\.pa_dealing/from pa_dealing/g' \
  src/pa_dealing/agents/orchestrator/agent.py

# Verify
grep -n "from src\." src/pa_dealing/agents/orchestrator/agent.py
# Expected: no output
```

**Example changes:**
```python
# BEFORE (inside functions)
from src.pa_dealing.agents.orchestrator.risk_scoring import score_request
from src.pa_dealing.db.repository import get_employee_info

# AFTER
from pa_dealing.agents.orchestrator.risk_scoring import score_request
from pa_dealing.db.repository import get_employee_info
```

---

#### File: `src/pa_dealing/agents/slack/handlers.py` (12 imports)

**Batch replace:**
```bash
sed -i '' 's/from src\.pa_dealing/from pa_dealing/g' \
  src/pa_dealing/agents/slack/handlers.py

grep -n "from src\." src/pa_dealing/agents/slack/handlers.py
# Expected: no output
```

---

#### File: `src/pa_dealing/agents/slack/chatbot.py` (9 imports)

**Batch replace:**
```bash
sed -i '' 's/from src\.pa_dealing/from pa_dealing/g' \
  src/pa_dealing/agents/slack/chatbot.py

grep -n "from src\." src/pa_dealing/agents/slack/chatbot.py
# Expected: no output
```

**Lines affected:** 761, 1293, 1600, 1714, 1723, 1732, 1876, 1889, 2262

---

#### File: `src/pa_dealing/services/pad_service.py` (9 imports)

**Batch replace:**
```bash
sed -i '' 's/from src\.pa_dealing/from pa_dealing/g' \
  src/pa_dealing/services/pad_service.py

grep -n "from src\." src/pa_dealing/services/pad_service.py
# Expected: no output
```

---

#### File: `src/pa_dealing/agents/monitoring/jobs.py` (4 imports)

**Batch replace:**
```bash
sed -i '' 's/from src\.pa_dealing/from pa_dealing/g' \
  src/pa_dealing/agents/monitoring/jobs.py

grep -n "from src\." src/pa_dealing/agents/monitoring/jobs.py
# Expected: no output
```

---

### 2.2 Batch Process Remaining Source Files (8 files)

**All remaining files (1-2 imports each):**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Process all in one command
for file in \
  src/pa_dealing/agents/orchestrator/risk_scoring_service.py \
  src/pa_dealing/agents/orchestrator/risk_scoring.py \
  src/pa_dealing/api/routes/requests.py \
  src/pa_dealing/api/routes/config.py \
  src/pa_dealing/api/routes/audit.py \
  src/pa_dealing/api/routes/dashboard.py \
  src/pa_dealing/audit/logger.py \
  src/pa_dealing/utils/reference_id.py; do

  echo "Processing $file..."
  sed -i '' 's/from src\.pa_dealing/from pa_dealing/g' "$file"

  # Verify
  grep -n "from src\." "$file" && echo "  ✗ Still has src. imports!" || echo "  ✓ Clean"
done
```

---

### 2.3 Verify All Source Files

```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Should return ZERO results
grep -r "from src\.pa_dealing" src/pa_dealing/
# Expected output: (empty)

# Count successful migrations
grep -r "from pa_dealing" src/pa_dealing/ | wc -l
# Expected: 58 (all imports converted)
```

---

## Phase 3: Test Files Migration (8-10 min)

### 3.1 Batch Process All Test Files

**All 20 test files in one pass:**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Get all test files with src. imports
grep -rl "from src\.pa_dealing" tests/ > /tmp/test_files_to_fix.txt

# Process each file
while IFS= read -r file; do
  echo "Processing $file..."
  sed -i '' 's/from src\.pa_dealing/from pa_dealing/g' "$file"
done < /tmp/test_files_to_fix.txt

# Verify
grep -r "from src\.pa_dealing" tests/
# Expected: no output
```

**Breakdown by test type:**

#### Unit Tests (11 files, ~40 imports)
- `test_google_identity_provider.py` (10 imports)
- `test_orchestrator.py` (4 imports)
- `test_api_authorization.py` (4 imports)
- `test_derivative_detection.py` (3 imports)
- `test_approval_expiry.py` (3 imports)
- `test_currency_endpoint.py` (2 imports)
- `test_risk_scoring_config.py` (2 imports)
- `test_mar_compliance.py` (1 import)
- `test_risk_scoring.py` (1 import)
- `test_insider_info.py` (1 import)
- `test_declaration_flow.py` (1 import)

#### Integration Tests (5 files, ~30 imports)
- `test_slack_mock.py` (11 imports)
- `test_risk_scoring_integration.py` (5 imports)
- `test_messy_data.py` (3 imports)
- `test_document_errors.py` (1 import)
- `test_interactive_updates.py` (1 import)

#### E2E Tests (4 files, ~18 imports)
- `test_uat_scenarios.py` (8 imports)
- `test_e2e_scenarios.py` (4 imports)
- `test_slack_interaction.py` (2 imports)
- `test_concurrency.py` (1 import)

---

### 3.2 Verify Test Files

```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Should return ZERO results
grep -r "from src\.pa_dealing" tests/
# Expected output: (empty)

# Count successful migrations
grep -r "from pa_dealing" tests/ | grep "^[^#]*from pa_dealing" | wc -l
# Expected: ~68 imports (excluding comments)
```

---

## Phase 4: Add Lint Guard (5 min)

### 4.1 Update pyproject.toml

**File:** `/Users/luisdeburnay/work/rules_engine_refactor/pyproject.toml`

**Add isort configuration (after line 106):**
```toml
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
ignore = []

# ADD THIS SECTION:
[tool.ruff.lint.isort]
known-first-party = ["pa_dealing"]
# This ensures ruff knows pa_dealing is a first-party package

# ADD THIS SECTION (alternative: use banned-api when available in ruff)
# For now, rely on grep in CI/CD to catch violations
```

**Alternative: Wait for ruff banned-api support**
Ruff does not yet support banned imports (like flake8-banned-api). When available, add:
```toml
[tool.ruff.lint.flake8-banned-api]
banned-modules = [
    { name = "src.pa_dealing", msg = "Use 'from pa_dealing' instead of 'from src.pa_dealing'" }
]
```

---

### 4.2 Add Pre-commit Check

**Create:** `/Users/luisdeburnay/work/rules_engine_refactor/.git/hooks/pre-commit` (optional)

```bash
#!/bin/bash
# Pre-commit hook to prevent src.pa_dealing imports

if git diff --cached --name-only | xargs grep -l "from src\.pa_dealing" 2>/dev/null; then
  echo "❌ ERROR: Found 'from src.pa_dealing' imports in staged files"
  echo "   Use 'from pa_dealing' instead"
  exit 1
fi

exit 0
```

**Make executable:**
```bash
chmod +x /Users/luisdeburnay/work/rules_engine_refactor/.git/hooks/pre-commit
```

---

## Phase 5: Verification (5-8 min)

### 5.1 Grep Verification

```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Check source files
echo "Checking src/ directory..."
grep -r "from src\.pa_dealing" src/ && echo "❌ FAILED" || echo "✓ PASSED"

# Check test files
echo "Checking tests/ directory..."
grep -r "from src\.pa_dealing" tests/ && echo "❌ FAILED" || echo "✓ PASSED"

# Check scripts (should be clean already)
echo "Checking scripts/ directory..."
grep -r "from src\.pa_dealing" scripts/ && echo "⚠ Found in scripts" || echo "✓ Clean"

# Allow in documentation
echo "Checking documentation (informational only)..."
grep -r "from src\.pa_dealing" . --include="*.md" | wc -l
# Expected: some occurrences in .autopsy/, conductor/, md/ (acceptable)
```

### 5.2 Ruff Verification

```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Check import sorting
ruff check src/ tests/ --select I
# Expected: May show some isort warnings, but no src.pa_dealing imports

# Full lint check
ruff check src/ tests/
# Expected: No new errors introduced by import changes
```

### 5.3 Import Resolution Test

**Create temporary test script:**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

cat > /tmp/test_imports.py <<'EOF'
#!/usr/bin/env python3
"""Test that all imports resolve correctly."""

print("Testing direct pa_dealing imports...")
try:
    from pa_dealing.db.models import PADRequest
    from pa_dealing.db.repository import get_employee_info
    from pa_dealing.agents.orchestrator.agent import get_orchestrator_agent
    from pa_dealing.services.pad_service import PADService
    print("✓ All imports successful")
except ImportError as e:
    print(f"✗ Import failed: {e}")
    exit(1)

print("\nTesting that src.pa_dealing imports FAIL...")
try:
    from src.pa_dealing.db.models import PADRequest  # noqa
    print("✗ src.pa_dealing import should NOT work!")
    exit(1)
except ImportError:
    print("✓ src.pa_dealing correctly rejected")

print("\n✓ All import tests passed!")
EOF

python3 /tmp/test_imports.py
```

### 5.4 Test Suite Verification

**Run full test suite:**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Quick unit tests first
pytest tests/unit/ -v --tb=short -x
# Expected: All pass (no import errors)

# Integration tests
pytest tests/integration/ -v --tb=short -x
# Expected: All pass (no import errors)

# E2E tests (may require services running)
docker compose up -d postgres slack-mock
pytest tests/e2e/ -v --tb=short
# Expected: All pass

# Full suite
pytest tests/ -v
# Expected: All tests pass with no new failures
```

**If any test fails with ImportError:**
```bash
# Debug which file has the issue
pytest tests/ -v 2>&1 | grep "ImportError.*src\.pa_dealing"
# Fix remaining files manually
```

### 5.5 Docker Container Verification

**Build and test in Docker:**
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Rebuild backend container
docker compose build backend

# Run tests in container
docker compose run --rm backend pytest tests/ -v --tb=short

# Start services and check logs for import errors
docker compose up -d
docker compose logs backend | grep -i "importerror\|modulenotfound"
# Expected: no import errors

docker compose down
```

---

## Rollback Strategy

### Simple Git Revert
```bash
cd /Users/luisdeburnay/work/rules_engine_refactor

# Create safety branch first
git checkout -b import-migration-backup

# After migration, if issues arise:
git checkout main  # or your working branch
git revert HEAD    # revert the migration commit

# Or if not yet committed:
git checkout -- src/ tests/ pyproject.toml
```

### Emergency Rollback Command
```bash
# Revert all files to use src.pa_dealing again
cd /Users/luisdeburnay/work/rules_engine_refactor

find src/ tests/ -type f -name "*.py" -exec sed -i '' 's/from pa_dealing/from src.pa_dealing/g' {} +

# Verify rollback
grep -r "from src\.pa_dealing" src/ tests/ | wc -l
# Expected: ~126 (back to original count)
```

---

## Edge Cases & Considerations

### 1. Lazy Imports (Function-Level Imports)

**Why lazy imports are used:**
```python
# BEFORE (causes circular import)
from pa_dealing.agents.slack.ui import build_blocks

def handler():
    return build_blocks()

# AFTER (avoids circular import)
def handler():
    from pa_dealing.agents.slack.ui import build_blocks
    return build_blocks()
```

**Impact:** None. The migration only changes the import path, not the import location.

### 2. Scripts Directory

**Check if scripts/ has src. imports:**
```bash
grep -r "from src\.pa_dealing" scripts/
# Expected: possibly some occurrences
```

**If found, fix them:**
```bash
find scripts/ -type f -name "*.py" -exec sed -i '' 's/from src\.pa_dealing/from pa_dealing/g' {} +
```

### 3. Jupyter Notebooks

**Check .ipynb files:**
```bash
find . -name "*.ipynb" -exec grep -l "from src\.pa_dealing" {} +
# If found, manually edit (JSON format makes sed risky)
```

### 4. CI/CD Configuration

**Check GitHub Actions / CI config:**
```bash
grep -r "src\.pa_dealing" .github/ .gitlab-ci.yml Jenkinsfile 2>/dev/null
# Expected: likely none, but verify
```

---

## Success Criteria

- [ ] Zero `from src.pa_dealing` imports in `src/` directory
- [ ] Zero `from src.pa_dealing` imports in `tests/` directory
- [ ] Zero `from src.pa_dealing` imports in `scripts/` directory
- [ ] `ruff check --select I` passes with no import errors
- [ ] Full test suite passes without new failures
- [ ] Docker containers start without import errors
- [ ] pyproject.toml has `[tool.ruff.lint.isort]` configuration
- [ ] Pre-commit hook (optional) prevents future violations

---

## Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Verify package config | 5 min | None |
| Phase 2: Source files (13 files) | 10-12 min | Phase 1 |
| Phase 3: Test files (20 files) | 8-10 min | Phase 2 |
| Phase 4: Add lint guard | 5 min | Phase 3 |
| Phase 5: Verification | 5-8 min | Phase 4 |
| **Total** | **33-40 min** | Sequential |

---

## Post-Migration Tasks

1. **Update documentation:**
   - Add import guidelines to `docs/CONTRIBUTING.md`
   - Document package structure in `docs/ARCHITECTURE.md`

2. **Update IDE configurations:**
   - `.vscode/settings.json`: Ensure `python.analysis.extraPaths` includes `src/`
   - PyCharm: Mark `src/pa_dealing` as "Sources Root"

3. **Add to CI/CD:**
   ```yaml
   # .github/workflows/lint.yml
   - name: Check for src. imports
     run: |
       if grep -r "from src\.pa_dealing" src/ tests/; then
         echo "ERROR: Found 'from src.pa_dealing' imports"
         exit 1
       fi
   ```

4. **Clean up pyproject.toml** (optional):
   Consider moving to flat layout in future:
   ```toml
   # Future: Move src/pa_dealing/ → pa_dealing/
   packages = ["pa_dealing", "scripts"]
   ```

---

## Notes

- **Both import styles currently work** because Python may have `src/` in sys.path by accident
- **After migration, only `from pa_dealing` will work** (cleaner, more Pythonic)
- **No database changes needed** - this is pure Python import refactoring
- **No Docker config changes needed** - package installation unchanged
- **No API changes** - internal import paths don't affect external APIs
