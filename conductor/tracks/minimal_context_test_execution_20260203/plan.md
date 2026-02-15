# Implementation Plan: Minimal-Context Test Execution System

## Phase 1: Discovery ✅
- [x] Analyze project test configuration (pytest, playwright).
- [x] Identify existing test markers and scripts.

## Phase 2: Implementation ✅
- [x] Create `scripts/test-runner.sh`.
- [x] Create `scripts/test-bg.sh`.
- [x] Create `scripts/test-status.sh`.

## Phase 3: Gemini Skill ✅
- [x] Create `.gemini/skills/test-runner/SKILL.md`.
- [x] Create `.gemini/skills/test-runner/references/test-modes.md`.
- [x] Update project `GEMINI.md` with critical mandates.

## Phase 4: Verification ✅
- [x] Rigorously test synchronous execution modes (`fast`, `lint`, `typecheck`).
- [x] Verify background execution and status polling.
- [x] Verify failure extraction logic efficiency.
