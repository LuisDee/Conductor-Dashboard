# Implementation Plan: Project Structure Cleanup, Reorganization, and Documentation Overhaul

## Phase 1: Audit & Root Cleanup
- [x] Task: Common Sense Code Review & Redundancy Check
    - [x] Subtask: **Prompt Strategy:** Act as a Senior Tech Lead. Perform a manual review of `src/pa_dealing` and `scripts` to identify:
        1. **Duplication:** Multiple files/functions doing the same thing (e.g., redundant DB helpers).
        2. **Inconsistency:** Mixed patterns (e.g., config loading, error handling).
        3. **Cruft:** "Bolted on" code that doesn't fit the architecture.
    - [x] Subtask: Create a brief "Refactoring Opportunities" list to address during reorganization.
- [x] Task: Project-wide File Audit
    - [x] Subtask: Identify all `.md` and `.txt` files in the root AND all subdirectories for deletion or archiving.
    - [x] Subtask: Move legacy planning files (`IMPLEMENTATION_PLAN.md`, `REMEDIATION_PLAN.md`, etc.) to `conductor/archive/`.
    - [x] Subtask: Audit `scripts/` directory and remove one-off utilities that are no longer functional or needed.
- [x] Task: Infrastructure Standardization
    - [x] Subtask: Standardize `ruff` configuration in `pyproject.toml`.
    - [x] Subtask: Review and update `dashboard/eslint.config.js` or `.eslintrc` for modern React best practices.
- [x] Task: Validation (Phase 1)
    - [x] Subtask: Run a smoke test (e.g., `pytest tests/test_api_authorization.py`) to ensure no critical files were deleted.
    - [x] Subtask: Verify `scripts/` still run (e.g., execute a setup script in dry-run mode).
- [x] Task: Conductor - User Manual Verification 'Phase 1: Audit & Root Cleanup' (Protocol in workflow.md)

## Phase 2: Structural Reorganization (Python & Support)
- [x] Task: Reorganize Backend (`src/pa_dealing`)
    - [x] Subtask: Audit internal module structure; ensure clear boundaries between `db`, `api`, `agents`, and `config`.
    - [x] Subtask: Move `alembic.ini` and `alembic/` to a standard location if required.
    - [x] Subtask: Update unit test imports to reflect new structure.
- [x] Task: Reorganize `tests/` and `scripts/`
    - [x] Subtask: Structure `tests/` to mirror `src/pa_dealing/` (e.g., `tests/unit/`, `tests/integration/`, `tests/e2e/`).
    - [x] Subtask: Categorize `scripts/` into functional subdirectories (e.g., `db/`, `dev/`, `maintenance/`).
- [x] Task: Validation (Phase 2)
    - [x] Subtask: **First:** Run a single unit test (e.g., `pytest tests/unit/test_config.py`) to verify basic imports. Fix if failed.
    - [x] Subtask: **Second:** Run the full Unit Test suite. Fix if failed.
    - [x] Subtask: **Third:** Run a single E2E test (e.g., `test_e2e_scenarios.py`) to verify integration. Fix if failed.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Structural Reorganization' (Protocol in workflow.md)

## Phase 3: Structural Reorganization (Frontend)
- [x] Task: Dashboard Structure Alignment
    - [x] Subtask: Review `dashboard/src/` and propose/implement a feature-based or standard React folder structure.
    - [x] Subtask: Ensure path aliases (e.g., `@/components`) are configured in `tsconfig.json` and `vite.config.ts`.
- [x] Task: Validation (Phase 3)
    - [x] Subtask: **First:** Build the frontend (`npm run build`) to ensure no compilation errors. Fix if failed.
    - [x] Subtask: **Second:** Run a single Playwright test (e.g., `tests/audit_all_pages.spec.ts`) to verify rendering. Fix if failed.
    - [x] Subtask: **Third:** Run the full Playwright test suite.
- [x] Task: Conductor - User Manual Verification 'Phase 3: Structural Reorganization (Frontend)' (Protocol in workflow.md)

## Phase 4: Documentation Overhaul
- [x] Task: Consolidate Agent Context
    - [x] Subtask: Create/Update `AGENTS.md` with common system-wide context and rules.
    - [x] Subtask: Update `.gemini/GEMINI.md` to reference `AGENTS.md` for shared knowledge.
- [x] Task: Architecture & Developer Docs
    - [x] Subtask: Create `docs/ARCHITECTURE.md` with system diagrams and data flow.
    - [x] Subtask: Create `docs/DEVELOPER_GUIDE.md` (or `CONTRIBUTING.md`) covering the new structure and linting rules.
- [x] Task: Conductor - User Manual Verification 'Phase 4: Documentation Overhaul' (Protocol in workflow.md)

## Phase 5: Final Validation & Standards (COMPLETED)
- [x] Task: Global Lint & Format
    - [x] Subtask: Run `ruff check --fix` and `ruff format` on all Python code.
    - [x] Subtask: Run `npm run lint` and format on the dashboard.
- [x] Task: Regression Testing (Full)
    - [x] Subtask: Execute full Unit Test suite (Final Check).
    - [x] Subtask: Execute full E2E test suite (Final Check).
    - [x] Subtask: Execute full Playwright test suite (Final Check).
- [x] Task: Conductor - User Manual Verification 'Phase 5: Final Validation' (Protocol in workflow.md)
