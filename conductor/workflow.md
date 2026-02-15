# Development Workflow

## 1. Core Principles
- **Container-First Development:** All testing and execution MUST occur within the Docker environment. Local (host-level) execution is strictly prohibited to ensure consistency.
- **Test-Driven Development (TDD):** Tests are written *before* implementation. No code is committed without passing tests.
- **Continuous E2E Verification:** End-to-end tests (including full backend integration and frontend rendering) must be maintained and executed regularly.
- **Visual Regression Testing:** All web pages and UI components must be verified against their rendered state using Playwright. New UI features require corresponding Playwright tests.

## 2. Testing Protocol
- **Unit Tests:** Run via `pytest` inside the backend container.
- **E2E Tests:** Run full scenario tests to verify system-wide behavior.
- **UI Tests (Playwright):**
    - Execute against the running dashboard container to verify rendering and interaction.
    - **Concurrency:** ALWAYS run with at least 4 workers to ensure performance and identify race conditions.
    - **Fixing Loop:** When fixing failed tests, use the `--last-failed` (or equivalent) flag to only repeat the failing tests.
    - **Final Verification:** Once all targeted failures pass, a full test suite rerun is MANDATORY before considering the task complete.

## 3. Frontend Build Protocol (CRITICAL)
- **Immutable Builds:** The dashboard container serves a static production build. Changes to `src/` are **NOT** reflected in the running container automatically.
- **Rebuild Requirement:** After ANY modification to frontend code (`.tsx`, `.ts`, `.css`), you MUST run:
  `docker compose -f docker/docker-compose.yml build dashboard && docker compose -f docker/docker-compose.yml up -d dashboard`
- **Verify before Test:** UI tests run against stale code are invalid. Always ensure the build timestamp in the container matches your latest edits.
- **Coverage Requirement:** >80% code coverage is required for all new logic.

## 3. Development Loop
For every task:
1. **Pull Latest:** Always start by ensuring your local branch is up-to-date (`git pull`).
2. **Rebuild Containers:** Ensure the Docker environment reflects the latest state (`docker compose build`).
3. **Write Tests:** Create unit or E2E tests that define the expected behavior.
4. Implement: Write the minimal code necessary to pass the tests.
5. Lint & Format (Python): Run `ruff check --fix .` and `ruff format .` to ensure code validity and standard style.
6. Verify (Docker): Run the full test suite inside the containers.
7. Commit: Commit changes with a clear message.
7. **Record:** specific task summary in Git Notes.

## 4. Phase Completion
- At the end of each phase, run the full regression suite (Unit + E2E + Playwright) to ensure no regressions were introduced.

## 5. Track Management (CRITICAL)

### Every Track MUST Have meta.yaml
When creating or updating tracks in `conductor/tracks/`, **EVERY track directory MUST contain a `meta.yaml` file**. This file is read by tooling (e.g., `cbd`) to track progress.

**Required meta.yaml structure:**
```yaml
name: Human-readable track name
status: not_started | in_progress | backlog | complete | blocked
priority: low | medium | high | critical
created: YYYY-MM-DD
completed: YYYY-MM-DD  # Only when status is complete
branch: branch-name    # Optional: associated git branch
tags:
  - relevant
  - tags
```

**Valid status values:**
- `not_started` - Track defined but work hasn't begun
- `in_progress` - Actively being worked on
- `backlog` - Spec complete, ready to start when prioritized
- `complete` - All work finished and verified
- `blocked` - Cannot proceed due to dependency or blocker

### Track Completion Checklist
When completing a track:
1. ✅ Update `meta.yaml` with `status: complete` and `completed: YYYY-MM-DD`
2. ✅ Update `conductor/tracks.md` to mark track as `[x]` complete
3. ✅ Both files MUST be in sync

### Creating New Tracks
Every new track directory must contain at minimum:
- `meta.yaml` - Track metadata (REQUIRED)
- `spec.md` - Problem statement and requirements
- `plan.md` - Implementation plan with tasks

<!-- ARCHITECT:HOOKS — Read architect/hooks/*.md for additional workflow steps -->