# Track Brief: Import Path Standardization

**Goal**: Standardize all imports to `from pa_dealing...` and remove `src/__init__.py`.

**Source**: `.autopsy/ARCHITECTURE_REPORT.md` Section 5 - verified: 127 `from src.pa_dealing` instances.

## Scope
13 source files (39 imports) + 20 test files (88 imports). Remove `src/__init__.py`. Add lint guard.

## Key Files
- `src/__init__.py` (remove)
- 13 source files with lazy `from src.pa_dealing` imports
- 20 test files with `from src.pa_dealing` imports
- `pyproject.toml` (verify package config)

## Effort Estimate
S (< 1 week) - mostly find-replace with test verification
