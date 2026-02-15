# Implementation Plan - Generic Bug Fixes and Tweaks

This plan is iterative and will be updated as new bug fixes and tweaks are identified.

## Phase 1: Logging Visibility (Completed)
- [x] Unignore `scripts/` directory in `.gitignore`.
- [x] Enhance `motherlogs.py` (renamed to `pad-main-logs.py`) with more service colors.
- [x] Rename `motherlogs.py` to `pad-main-logs.py`.
- [x] Verify `graph-email-poller` and `outbox_worker` log visibility.

## Phase 2: Ongoing Maintenance (Completed)
- [x] Hide "SMF16 Required: None" from Slack proposed trade summary.
- [x] Fix `TypeError: SimplifiedRiskScorer.score_request() unexpected keyword argument 'resolution_outcome'`.
- [x] Fix External Resolution hijacking internal symbols (e.g., "bund" -> "BMHS").
- [x] Add "None of these" handling to chatbot disambiguation list.
- [x] Fix Test Infrastructure (model registration + conftest conflicts).
- [x] Update Instrument Lookup documentation and skill with new tiered logic.

## Phase 3: Monitoring & New Papercuts (Current)
- [x] Investigate `oracle_product` (Tier 3) empty state in dev (Issue 6 from URGENT_TODOS). **Verified: Table has 7,112 rows in Dev. Tier 3 is healthy.**
- [x] Move market price overwrite handling to a new dedicated track: `price_resolution_strategy_20260208`.
- [ ] Monitor system for new "papercuts".

## Phase 4: Urgent Blocker - `ticker` AttributeError (Completed)
- [x] Update `_request_to_info` in `repository.py` to use `inst_symbol` and map back to `ticker` for schemas.
- [x] Update `get_requests_by_status` in `repository.py` to remove `req.ticker`.
- [x] Update `submit_pad_request` in `repository.py` to remove `ticker` assignment.
- [x] Update `trade_document_processor.py` to use `request.inst_symbol`.
- [x] Update `document_processor/agent.py` to use `inst_symbol`.
- [x] Verify fix by running `scripts/test_decline_45.py` via API (or similar endpoint verification).
- [x] Fix regressions in `test_trade_document_processor.py`, `test_document_agent.py`, `test_pdf_history_api.py`, `test_identity_cache.py`, and UI redesign tests.
