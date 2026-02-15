# Track Brief: Datetime & Timezone Standardization

**Goal**: Replace all deprecated `datetime.utcnow()` with `datetime.now(timezone.utc)` and add lint guard.

**Source**: `.autopsy/ARCHITECTURE_REPORT.md` Section 5 - verified: 31 instances across 13 files.

## Scope
10 source files (22 calls) + 3 test files (9 calls). Add lint rule to prevent regression.

## Key Files
- `services/notification_outbox.py` (7 calls - largest)
- `api/routes/pdf_history.py` (3 calls)
- `db/repository.py` (2 calls - holding period)
- `services/rules_engine/service.py` (2 calls)
- `services/restricted_instruments.py` (2 calls)
- 5 more files with 1 call each

## Effort Estimate
S (< 1 week) - find-replace with test verification
