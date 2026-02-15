# Track Brief: Critical Data Integrity Bug Fixes

**Goal**: Fix 8 verified logic bugs causing data corruption, incorrect compliance decisions, and broken audit trails.

**Source**: `.autopsy/REVIEW_REPORT.md` + `.autopsy/ARCHITECTURE_REPORT.md` - all verified against code.

## Scope
Backend: PDF double-decode, breach auto-resolution, FX rate fallback, risk routing, orphan recovery timeout, audit ActionTypes.
Frontend: range slider validation, boolean select types.

## Key Files
- `src/pa_dealing/services/graph_email_poller.py` (PDF)
- `src/pa_dealing/services/pad_service.py` (breach + audit)
- `src/pa_dealing/services/currency_service.py` (FX)
- `src/pa_dealing/agents/orchestrator/risk_scoring.py` (routing)
- `src/pa_dealing/services/pdf_poller.py` (orphan timeout)
- `dashboard/src/components/ui/DualRangeSlider.tsx` (slider)
- `dashboard/src/pages/NewRequest.tsx` (boolean selects)

## Effort Estimate
S-M (1-2 weeks) - mostly 1-line fixes with tests; risk routing needs business decision
