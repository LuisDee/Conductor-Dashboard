# Spec: Datetime & Timezone Standardization

## Problem Statement
31 instances of deprecated `datetime.utcnow()` across 13 files return naive datetimes causing timezone comparison errors with PostgreSQL `timestamptz` columns. Additionally, critical paths use `datetime.now()` without timezone. This causes incorrect holding period calculations and audit timestamp ambiguity, both regulatory compliance risks.

## Source
- `.autopsy/REVIEW_REPORT.md` - Finding #10
- `.autopsy/ARCHITECTURE_REPORT.md` - Section 5: "CRITICAL: Deprecated datetime.utcnow() Causes Timezone Errors"

## Findings (Verified Against Code)

### datetime.utcnow() - 31 instances across 13 files

**Source files (10 files, 22 calls):**
| File | Count | Context |
|------|-------|---------|
| `services/notification_outbox.py` | 7 | Notification queue management |
| `api/routes/pdf_history.py` | 3 | Analytics date calculations |
| `db/repository.py` | 2 | Holding period calculations |
| `services/rules_engine/service.py` | 2 | Rule update timestamps |
| `services/restricted_instruments.py` | 2 | Instrument update timestamps |
| `api/routes/admin.py` | 1 | Notification retry logic |
| `services/extraction_router.py` | 1 | Trade review timestamps |
| `services/gcs_client.py` | 1 | GCS document timestamp fallback |
| `services/smart_matcher.py` | 1 | Trade matching cutoff date |

**Test files (3 files, 9 calls):**
| File | Count |
|------|-------|
| `tests/unit/test_pdf_history_api.py` | 5 |
| `tests/integration/test_pdf_poller_integration.py` | 4 |

### Timezone imports status
- Only 3 files currently import `from datetime import timezone`
- 1 file imports `from datetime import UTC` (repository.py)

## Requirements
1. Replace all `datetime.utcnow()` with `datetime.now(timezone.utc)` in source files (22 calls)
2. Replace all `datetime.utcnow()` with `datetime.now(timezone.utc)` in test files (9 calls)
3. Add `from datetime import timezone` import where needed
4. Add ruff lint rule to ban `utcnow()` usage
5. Verify ORM models use `server_default=func.now()` (already correct per architecture report)

## Out of Scope
- Fixing all 132 `datetime.now()` without timezone calls (these are mostly in orchestrator/agent code and tests; tracked separately if needed)

## Acceptance Criteria
- [ ] Zero instances of `datetime.utcnow()` in codebase
- [ ] Ruff/lint rule prevents future `utcnow()` usage
- [ ] All holding period calculations use timezone-aware datetimes
- [ ] All existing tests pass
- [ ] No naive/aware datetime comparison warnings
