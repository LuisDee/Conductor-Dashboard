# Specification: Project Structure Cleanup, Reorganization, and Documentation Overhaul

## Overview
This track focuses on a comprehensive "deep dive" into the `pa_dealing` project to resolve technical debt accrued during development. The goal is to audit every file, remove or reorganize obsolete artifacts, align the directory structure with industry best practices for Python and TypeScript/React, and establish a robust documentation framework optimized for humans and AI agents.

## Functional Requirements
### 1. Project Audit and Cleanup
- **File Audit:** Evaluate every file in the project root and subdirectories.
- **Obsolete Artifact Removal:** Identify and remove/archive old planning documents (e.g., `IMPLEMENTATION_PLAN_PHASE_2.md`, `REMEDIATION_PLAN.md`, `ORIGINAL_IMPORVEMENT_PLAN.md`), aborted tests, and one-off utility scripts.
- **Configuration Consolidation:** Consolidate root-level configuration files where appropriate (e.g., moving specific agent docs or environment templates).

### 2. Structural Reorganization
- **Python (src/pa_dealing):** Reorganize the backend module to strictly follow best practices (e.g., clean separation of schemas, models, agents, and services).
- **Frontend (dashboard):** Review the TypeScript/React structure in `dashboard/src` and align with modern standards (e.g., feature-based vs. type-based folder structure).
- **Support Directories:** Categorize `scripts/` (e.g., `setup/`, `migrations/`, `maintenance/`) and reorganize `tests/` to mirror the `src` structure.
- **Standards Enforcement:** Implement and standardize linting/formatting (Ruff for Python; ESLint/Prettier for TypeScript) and configure pre-commit hooks to maintain the new standard.

### 3. Documentation Framework (Human & AI)
- **Agent Context:** Consolidate common agent instructions into `AGENTS.md`. Update `GEMINI.md` and any other agent-specific files to act as thin wrappers/overrides for specific agent nuances.
- **Architecture Docs:** Create high-level system architecture documentation (e.g., C4 model or similar) in the `docs/` folder.
- **Developer Guide:** Create a `CONTRIBUTING.md` or comprehensive Developer Guide covering setup, testing, and the newly established structural conventions.

## Non-Functional Requirements
- **Functional Integrity:** The system must remain fully functional throughout the reorganization (verified by the existing E2E suite).
- **Discoverability:** Improve project "readability" for AI agents by using logical naming and consistent structures.

## Acceptance Criteria
- [x] No obsolete planning or temporary files remain in the project root.
- [x] The directory structure follows established best practices for both Python and React.
- [x] All code passes linting/formatting checks (Ruff, ESLint).
- [x] `AGENTS.md` is the primary source of truth for agent context.
- [x] `docs/` contains updated Architecture and Developer guides.

## Out of Scope
- Adding new business features.
- Large-scale refactoring of core business logic (logic remains the same; only structure changes).
