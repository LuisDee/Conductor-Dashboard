# Track Brief: God Module Decomposition

**Goal**: Split 5 god modules (>2,300 LOC each) into focused modules and extract business logic from Slack handlers.

**Source**: `.autopsy/ARCHITECTURE_REPORT.md` Section 5 - verified LOC counts.

## Scope
Phase 1: Split repository.py (2,953 LOC) into 4 domain repositories. Phase 2: Split pad_service.py (2,748 LOC) into 4 lifecycle services. Phase 3: Extract business logic from handlers.py (3,192 LOC) to service layer.

## Key Files
- `db/repository.py` (2,953 LOC -> 4 modules)
- `services/pad_service.py` (2,748 LOC -> 4 modules)
- `agents/slack/handlers.py` (3,192 LOC -> slim handlers + services)

## Dependencies
- Should follow `critical_data_integrity_bugs_20260212` (fixes in pad_service.py)
- Should follow `error_handling_resilience_20260212` (fixes in pad_service.py)

## Effort Estimate
L (1-3 months) - large refactoring with comprehensive test verification
