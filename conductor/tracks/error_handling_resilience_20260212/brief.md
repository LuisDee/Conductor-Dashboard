# Track Brief: Error Handling & Resilience Hardening

**Goal**: Fix cascading failure patterns, type safety violations, credential leaking, and silent data loss.

**Source**: `.autopsy/REVIEW_REPORT.md` HIGH findings - all verified against code.

## Scope
6 categories: asyncio.gather error handling, API error handler types, credential leaking in logs, unprotected flush() calls, silent failure patterns, bare except blocks.

## Key Files
- `services/pad_service.py` (asyncio.gather, flush)
- `dashboard/src/api/client.ts` (error handler type)
- `services/graph_client.py` (credential leaking)
- `services/trade_document_processor.py` (silent failures, flush)
- `services/pdf_poller.py` (bare except)
- `services/restricted_instruments.py` (flush)

## Effort Estimate
M (1-2 weeks) - multiple files, needs careful testing of error paths
