# Track: Generic Bug Fixes and Tweaks

**ID**: generic_bugfixes_and_tweaks_20260206
**Priority**: Medium
**Tags**: bugfix, maintenance, iterative

## Overview
This is an iterative track dedicated to addressing small bugs, UI tweaks, and general system maintenance as they arise. Instead of creating a new track for every minor adjustment, we will append to this track's plan and specification over time.

## Current Items for Discussion
### 1. Hide SMF16 Required: None
- **Context**: Slack proposed trade summary notification.
- **Goal**: Omit the "SMF16 Required" field if the value is "None" to reduce clutter.
- **Proposed Fix**: Conditional logic in `build_trade_summary_blocks`.

### 2. TypeError: SimplifiedRiskScorer.score_request() unexpected keyword argument 'resolution_outcome'
- **Context**: Trade request submission in Slack.
- **Goal**: Fix the crash preventing trade submissions.
- **Diagnosis**: Discrepancy between code on disk and code in memory. The `slack-listener` container was likely running a stale version of the `risk_scoring.py` module after recent instrument resolution changes.
- **Proposed Fix**: Ensure all backend containers are restarted after significant logic changes. Clear `__pycache__` to prevent stale bytecode execution.

### 3. External Resolution Hijacking Internal Nomenclature
- **Context**: Instrument lookup (search_instruments).
- **Goal**: Ensure internal symbols (like "BUND") are found even if an external provider returns unrelated results (like "BMHS").
- **Diagnosis**: Tier 0 (External) was returning early if any results were found from EODHD, even if those results weren't in our DB (Outcome 2). This prevented internal tiers from being checked.
- **Proposed Fix**: Only return early from External Resolution if high-confidence matches (Outcome 1 - External + Internal) are found. Otherwise, collect external results as a secondary fallback and proceed to internal DB tiers.

### 4. Urgent Blocker: 500 Internal Server Error on Approval/Decline
- **Context**: Dashboard pending approvals / decline actions.
- **Goal**: Restore the ability to approve or decline requests.
- **Diagnosis**: `AttributeError: 'PADRequest' object has no attribute 'ticker'`. This was caused by the dropping of the `ticker` column in migration `c98f6aae7452` while the repository layer (`_request_to_info`) still attempts to access it.
- **Proposed Fix**: Update all repository and service layer references to use `inst_symbol` or `bloomberg_ticker` instead of the now-deleted `ticker` column. Maintain backward compatibility in Pydantic schemas where necessary.
