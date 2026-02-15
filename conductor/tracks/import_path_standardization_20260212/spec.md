# Spec: Import Path Standardization

## Problem Statement
127 instances of `from src.pa_dealing...` imports mixed with 715 instances of `from pa_dealing...`. The `src/__init__.py` file makes both valid in development, but only `from pa_dealing...` works in production wheel installations. This causes random import failures across environments.

## Source
- `.autopsy/ARCHITECTURE_REPORT.md` - Section 5: "CRITICAL: Inconsistent Import Paths Break Module Resolution"

## Findings (Verified Against Code)

### Import counts
- `from src.pa_dealing...`: 127 instances (39 in 13 source files, 88 in 20 test files)
- `from pa_dealing...`: 715 instances (dominant pattern)
- `src/__init__.py`: EXISTS (empty marker file, enables `src` as importable package)
- `pyproject.toml`: `packages = ["src/pa_dealing", "scripts"]`

### Source files using `from src.pa_dealing` (13 files)
- `pad_service.py` (lazy imports)
- `agents/orchestrator/agent.py`, `risk_scoring_service.py`, `risk_scoring.py`
- `agents/slack/handlers.py`, `chatbot.py`
- `agents/monitoring/jobs.py`
- `api/routes/requests.py`, `config.py`, `audit.py`, `dashboard.py`
- `audit/logger.py`
- `utils/reference_id.py`

### Test files using `from src.pa_dealing` (20 files, 88 instances)
- Unit tests: 11 files (~40 imports)
- Integration tests: 5 files (~30 imports)
- E2E tests: 4 files (~18 imports)

## Requirements
1. Remove `src/__init__.py`
2. Replace all `from src.pa_dealing` with `from pa_dealing` across all files
3. Update `pyproject.toml` package config if needed
4. Add ruff/flake8 lint rule to prevent `from src.` imports
5. Run full test suite to verify no broken imports

## Acceptance Criteria
- [ ] `src/__init__.py` removed
- [ ] Zero instances of `from src.pa_dealing` in codebase
- [ ] Lint rule prevents future `from src.` imports
- [ ] All tests pass in Docker environment
- [ ] `pyproject.toml` package config verified
