# Implementation Plan: Security Confirmation UX & Position Lookup

**Track ID**: `security_confirmation_ux_and_position_lookup_20260122`

---

## Phase 1: Critical Database Fix (Priority 1)
**Goal**: Unblock trade submission by fixing database error

- [x] Task 1.1: Investigate database schema mismatch
  - [x] Read `OraclePosition` model in `src/pa_dealing/db/models/market.py`
  - [x] Verify `last_trade_date` field usage in codebase
  - [x] Check if field is used in `days_since_last_trade` property
  - [x] Confirm field is unused (returns None)

- [x] Task 1.2: Write tests for trade submission
  - [x] Create test in `tests/integration/test_trade_submission.py`
  - [x] Test: Verify trade submission completes without database errors
  - [x] Test: Verify trade creation in database after submission
  - [x] Test: Verify _check_mako_recent_activity doesn't crash

- [x] Task 1.3: Remove unused column from model
  - [x] Remove `last_trade_date: Mapped[datetime | None]` from line 59 in `src/pa_dealing/db/models/market.py`
  - [x] Update `_check_mako_recent_activity()` to stub (return no conflict)
  - [x] Update seed scripts (seed_dev_database.py, seed_data.py)
  - [x] Verify no other code references this field
  - [x] Add documentation comments pointing to Phase 4

- [x] Task 1.4: Verify fix
  - [x] Run integration tests (expect pass)
  - [x] Manual test: Submit trade end-to-end in dev environment
  - [x] Verify trade record created in database
  - [x] Bonus: Fixed selection bug (wrong security selected when user typed "1")
  - [x] Bonus: Implemented semantic ranking (best match always at index 0)

- [x] Task: Conductor - User Manual Verification 'Phase 1: Critical Database Fix' (Protocol in workflow.md)

---

## Phase 2: Symbol Extraction Enhancement (Priority 2) ✅ COMPLETE
**Goal**: Improve UX by cleaning derivative terms from security searches

- [x] Task 2.1: Design regex patterns for term cleaning
  - [x] List all derivative types to strip (calls, puts, options, futures, forwards)
  - [x] List all quantity words to strip (lots, shares, units, contracts)
  - [x] List all price indicators to strip (@ 100, at 50.25)
  - [x] List all action verbs to strip (buy, sell, buying, selling, trade)
  - [x] Design regex patterns for each category

- [x] Task 2.2: Write tests for symbol extraction
  - [x] Create test in `tests/unit/test_symbol_extraction.py`
  - [x] Test: "buying 5 lots of bund calls @ 100" → "bund"
  - [x] Test: "sell 10 shares of AAPL" → "aapl"
  - [x] Test: "trade FGBL futures" → "fgbl"
  - [x] Test: "bund call options" → "bund"
  - [x] Test: "100 lots @ 50" → "" (no symbol)
  - [x] Test: Preserve existing negation word removal
  - [x] Run tests (expect failure - logic not implemented)

- [x] Task 2.3: Implement enhanced term cleaning
  - [x] Open `src/pa_dealing/agents/slack/chatbot.py`
  - [x] Locate `update_draft` function (around line 239-250)
  - [x] Add regex patterns for derivative types
  - [x] Add regex patterns for quantity words
  - [x] Add regex patterns for price indicators
  - [x] Add regex patterns for action verbs
  - [x] Add regex for quantity phrases (e.g., "5 lots of")
  - [x] Keep existing negation word removal
  - [x] Add whitespace cleanup after all removals

- [x] Task 2.4: Verify symbol extraction
  - [x] Run unit tests (expect pass)
  - [x] Lint code: `ruff check --fix .`
  - [x] Format code: `ruff format .`
  - [x] Restart Slack listener to reload chatbot logic

- [x] Task 2.5: Manual UAT testing
  - [x] Test: "buying 5 lots of bund calls @ 100" (should extract "bund")
  - [x] Test: "sell AAPL shares" (should extract "aapl")
  - [x] Test: "FGBL futures" (should extract "fgbl")
  - [x] Verify bot finds correct securities

- [x] Task: Conductor - User Manual Verification 'Phase 2: Symbol Extraction Enhancement' (Protocol in workflow.md)

---

## Phase 3: Simplified Confirmation UX (Priority 3) ✅ COMPLETE
**Goal**: Cleaner, less overwhelming confirmation flow

- [x] Task 3.1: Design new confirmation flow
  - [x] Map out single-match confirmation prompt
  - [x] Map out "no" response handling (show alternatives)
  - [x] Map out direct symbol override handling
  - [x] Define new SYSTEM_PROMPT instructions

- [x] Task 3.2: Write tests for confirmation flow
  - [x] Create test in `tests/integration/test_security_confirmation_flow.py`
  - [x] Test: Single match shows only top result
  - [x] Test: User confirms with "yes"
  - [x] Test: User rejects with "no" (shows alternatives)
  - [x] Test: User types different symbol name (new search)
  - [x] Run tests (expect failure - logic not implemented)

- [x] Task 3.3: Update SYSTEM_PROMPT
  - [x] Open `src/pa_dealing/agents/slack/chatbot.py`
  - [x] Locate SYSTEM_PROMPT (around line 78-93)
  - [x] Change instruction from "present ALL candidates" to "present ONLY top match"
  - [x] Update example format: "Can you confirm you are referring to [TICKER] ([description])?"
  - [x] Update "no" response handling instructions
  - [x] Remove numbered selection instructions

- [x] Task 3.4: Update confirmation logic (if needed)
  - [x] Review confirmation handling code in `update_draft` function
  - [x] Ensure "yes" confirms top match
  - [x] Ensure "no" triggers alternative display
  - [x] Ensure direct symbol name triggers new search

- [x] Task 3.5: Verify confirmation UX
  - [x] Run integration tests (expect pass)
  - [x] Lint and format code
  - [x] Restart Slack listener

- [x] Task 3.6: Manual UAT testing
  - [x] Test: "bund" → Shows "Can you confirm BUND?"
  - [x] Test: Reply "yes" → Continues with BUND
  - [x] Test: Reply "no" → Shows alternatives, asks for clarification
  - [x] Test: Reply different name → Treats as new search

- [x] Task: Conductor - User Manual Verification 'Phase 3: Simplified Confirmation UX' (Protocol in workflow.md)

---

## Phase 4: Position Lookup & Conflict Detection (Priority 4) ✅ COMPLETE
**Goal**: Query firm trading and employee history for compliance risk detection

### Sub-Phase 4.1: Database Schema Verification
- [x] Task 4.1.1: Verify `product_usage` table exists
  - [x] Connect to dev database
  - [x] Query `bo_airflow.product_usage` schema
  - [x] Verify columns: `inst_symbol`, `portfolio`, `last_trade_date`, `last_position_date`
  - [x] Document table relationships

- [x] Task 4.1.2: Verify related tables
  - [x] Verify `bo_airflow.position` table for position_size lookup
  - [x] Verify `bo_airflow.portfolio_meta_data` table
  - [x] Verify `bo_airflow.portfolio_group` table
  - [x] Document join patterns

### Sub-Phase 4.2: Create Pydantic Schema Models
- [x] Task 4.2.1: Write tests for schema models
  - [x] Create test in `tests/unit/test_position_schemas.py`
  - [x] Test: `MakoPositionInfo` model validation
  - [x] Test: `EmployeeTradeRecord` model validation
  - [x] Test: `ConflictRiskResult` model validation
  - [x] Run tests (expect failure - models not defined)

- [x] Task 4.2.2: Implement schema models
  - [x] Open `src/pa_dealing/db/schemas.py`
  - [x] Create `MakoPositionInfo` class with fields:
    - `inst_symbol: str`
    - `portfolio: str | None`
    - `last_trade_date: datetime | None`
    - `last_position_date: datetime | None`
    - `position_size: Decimal | None`
    - `desk_name: str | None`
  - [x] Create `EmployeeTradeRecord` class with fields:
    - `request_id: int`
    - `direction: str`
    - `quantity: int`
    - `estimated_value: Decimal`
    - `approved_date: datetime`
  - [x] Create `ConflictRiskResult` class with fields:
    - `has_conflict: bool`
    - `days_since_mako_trade: int | None`
    - `firm_direction: str`
    - `employee_direction: str`
    - `desk_name: str | None`
    - `employee_trade_count: int`
    - `risk_level: str`
    - `risk_factors: list[str]`

- [x] Task 4.2.3: Verify schema models
  - [x] Run unit tests (expect pass)
  - [x] Lint and format code

### Sub-Phase 4.3: Create Repository Functions
- [x] Task 4.3.1: Write tests for `get_mako_position_info()`
  - [x] Create test in `tests/integration/test_position_lookup.py`
  - [x] Test: Query returns position info when Mako traded security
  - [x] Test: Query returns None when Mako never traded security
  - [x] Test: Verify position_size populated from subquery
  - [x] Test: Verify desk_name populated from joins
  - [x] Run tests (expect failure - function not implemented)

- [x] Task 4.3.2: Implement `get_mako_position_info()`
  - [x] Open `src/pa_dealing/db/repository.py`
  - [x] Create async function `get_mako_position_info(session, inst_symbol)`
  - [x] Query `product_usage` table filtered by `inst_symbol`
  - [x] Subquery `position` table for `position_size` on `last_position_date`
  - [x] Join `portfolio_meta_data` + `portfolio_group` for `desk_name`
  - [x] Return `MakoPositionInfo` or None

- [x] Task 4.3.3: Write tests for `get_employee_trade_history()`
  - [x] Test: Query returns approved trades only
  - [x] Test: Query filters out deleted records (`deleted_at IS NULL`)
  - [x] Test: Query filters by `inst_symbol`
  - [x] Test: Query filters by `employee_id`
  - [x] Test: Query respects `lookback_months` parameter
  - [x] Run tests (expect failure - function not implemented)

- [x] Task 4.3.4: Implement `get_employee_trade_history()`
  - [x] Create async function `get_employee_trade_history(session, employee_id, inst_symbol, lookback_months=6)`
  - [x] Query `pad_request` joined with `pad_approval`
  - [x] Filter: `approval_type = 'compliance'`
  - [x] Filter: `decision = 'approved'`
  - [x] Filter: `deleted_at IS NULL`
  - [x] Filter: `inst_symbol = <param>`
  - [x] Filter: `employee_id = <param>`
  - [x] Filter: `created_at >= NOW() - <lookback_months> months`
  - [x] Return list of `EmployeeTradeRecord`

- [x] Task 4.3.5: Write tests for `calculate_conflict_risk()`
  - [x] Create test in `tests/unit/test_conflict_detection.py`
  - [x] Test: Conflict flagged when Mako traded <30 days ago
  - [x] Test: No conflict when Mako traded >30 days ago
  - [x] Test: No conflict when Mako never traded (None input)
  - [x] Test: Firm direction determined correctly (LONG/SHORT/FLAT)
  - [x] Test: Risk level calculated correctly (high/medium/low/none)
  - [x] Test: Risk factors populated with explanations
  - [x] Run tests (expect failure - function not implemented)

- [x] Task 4.3.6: Implement `calculate_conflict_risk()`
  - [x] Create function `calculate_conflict_risk(mako_info, employee_history, employee_direction)`
  - [x] Calculate `days_since_trade = (CURRENT_DATE - last_trade_date).days`
  - [x] Determine `has_conflict = days_since_trade <= 30`
  - [x] Determine firm direction from `position_size` (>0=LONG, <0=SHORT, 0=FLAT)
  - [x] Calculate `employee_trade_count = len(employee_history)`
  - [x] Determine risk_level logic:
    - "high": conflict + same direction as firm
    - "medium": conflict + opposite direction
    - "low": no conflict but employee has history
    - "none": no conflict and no history
  - [x] Populate `risk_factors` with human-readable explanations
  - [x] Return `ConflictRiskResult`

- [x] Task 4.3.7: Verify repository functions
  - [x] Run all tests (unit + integration)
  - [x] Lint and format code
  - [x] Check test coverage >80%

### Sub-Phase 4.4: Integrate Conflict Detection in Orchestrator
- [x] Task 4.4.1: Write tests for orchestrator integration
  - [x] Create test in `tests/integration/test_orchestrator_conflict.py`
  - [x] Test: Request includes conflict data when conflict exists
  - [x] Test: Request has no conflict data when no conflict
  - [x] Test: Compliance assessment JSON includes position data
  - [x] Run tests (expect failure - integration not implemented)

- [x] Task 4.4.2: Update orchestrator to call position lookup
  - [x] Open `src/pa_dealing/agents/orchestrator/agent.py`
  - [x] Locate `process_pad_request()` function
  - [x] Add call to `get_mako_position_info(session, draft.security_identifier)`
  - [x] Add call to `get_employee_trade_history(session, draft.employee_id, draft.security_identifier)`
  - [x] Add call to `calculate_conflict_risk(mako_info, employee_history, draft.direction)`
  - [x] Store conflict data in draft/request object
  - [x] Add to compliance_assessment JSON field

- [x] Task 4.4.3: Update request schema to include conflict fields
  - [x] Open `src/pa_dealing/db/schemas.py`
  - [x] Add fields to request schema (if not already present):
    - `has_conflict: bool | None`
    - `conflict_level: str | None`
    - `conflict_data: dict | None`

- [x] Task 4.4.4: Verify orchestrator integration
  - [x] Run integration tests (expect pass)
  - [x] Lint and format code
  - [x] Restart API service

### Sub-Phase 4.5: Update Dashboard to Display Conflicts
- [x] Task 4.5.1: Write Playwright tests for conflict display
  - [x] Create test in `tests/e2e/test_conflict_display.py`
  - [x] Test: Conflict badge appears on request card when has_conflict=True
  - [x] Test: No badge when has_conflict=False
  - [x] Test: Detail view shows full conflict information
  - [x] Run tests with `--workers 4` (expect failure - UI not implemented)

- [x] Task 4.5.2: Update RequestCard component
  - [x] Open `dashboard/src/components/RequestCard.tsx`
  - [x] Add conditional rendering for conflict badge
  - [x] Show ⚠️ icon + "Conflict Detected" when `request.has_conflict`
  - [x] Color-code badge based on `conflict_level`

- [x] Task 4.5.3: Update RequestDetail component
  - [x] Open `dashboard/src/components/RequestDetail.tsx`
  - [x] Add Alert/Warning section for conflict details
  - [x] Display: Days since Mako traded
  - [x] Display: Firm direction (LONG/SHORT/FLAT)
  - [x] Display: Employee direction
  - [x] Display: Desk name
  - [x] Display: Employee's prior trade count

- [x] Task 4.5.4: Rebuild and verify dashboard
  - [x] Rebuild dashboard container: `docker compose -f docker/docker-compose.yml build dashboard`
  - [x] Restart dashboard: `docker compose -f docker/docker-compose.yml up -d dashboard`
  - [x] Run Playwright tests with `--workers 4` (expect pass)
  - [x] Rerun full suite to verify no regressions

- [x] Task: Conductor - User Manual Verification 'Phase 4: Position Lookup & Conflict Detection' (Protocol in workflow.md)

---

## Phase 5: Dashboard Startup in UAT Script (Priority 5)
**Goal**: Add dashboard to UAT startup script

- [x] Task 5.1: Write test for dashboard startup
  - [x] Create test script `tests/scripts/test_uat_startup.sh`
  - [x] Test: Verify tmux session has 3 windows (API, Slack, Dashboard)
  - [x] Test: Verify dashboard accessible at http://localhost:3000/
  - [x] Run test (expect failure - dashboard window not in script)

- [x] Task 5.2: Update UAT startup script
  - [x] Open `scripts/ops/run_uat_dev_simple.sh`
  - [x] Add tmux window 2 for dashboard (after line 86)
  - [x] Change directory to `dashboard/`
  - [x] Set `VITE_API_URL=http://localhost:8000`
  - [x] Run `npm install 2>&1 | tail -3` (quiet mode)
  - [x] Run `npm run dev`

- [x] Task 5.3: Update script output
  - [x] Update "Services:" section to include dashboard URL
  - [x] Add line: `🎨 Dashboard: http://localhost:3000`
  - [x] Update "View logs:" section
  - [x] Update window instructions: "0 (API), 1 (Slack), or 2 (Dashboard)"

- [x] Task 5.4: Verify dashboard startup
  - [x] Kill existing UAT session: `tmux kill-session -t pad_uat`
  - [x] Run updated script: `bash scripts/ops/run_uat_dev_simple.sh`
  - [x] Verify 3 tmux windows created
  - [x] Check window 2 shows dashboard dev server running
  - [x] Visit http://localhost:3000/ (expect dashboard to load)

- [x] Task 5.5: Update UAT documentation
  - [x] Open `UAT_QUICK_START.md`
  - [x] Update Pre-Flight Check section to mention dashboard
  - [x] Add dashboard access instructions

- [x] Task: Conductor - User Manual Verification 'Phase 5: Dashboard Startup in UAT Script' (Protocol in workflow.md)

---

## Phase 6: Testing & Documentation
**Goal**: Comprehensive testing and documentation updates

- [x] Task 6.1: Create position lookup integration tests
  - [x] Create `tests/integration/test_position_lookup.py` (if not already created)
  - [x] Test: `get_mako_position_info()` with real database
  - [x] Test: `get_employee_trade_history()` with real database
  - [x] Test: End-to-end trade submission with conflict detection
  - [x] Run tests (expect pass)

- [x] Task 6.2: Update UAT guide with conflict detection tests
  - [x] Open `UAT_SECURITY_CONFIRMATION_FLOW.md`
  - [x] Add new scenario: "Conflict Detection"
  - [x] Steps: Submit trade for security Mako recently traded
  - [x] Expected: Conflict badge appears, details shown in dashboard
  - [x] Validation checklist for conflict display

- [x] Task 6.3: Run full regression suite
  - [x] Run all unit tests: `pytest tests/unit/`
  - [x] Run all integration tests: `pytest tests/integration/`
  - [x] Run E2E tests inside containers
  - [x] Run Playwright tests with `--workers 4`
  - [x] Verify all tests pass
  - [x] Check coverage: >80% required

- [x] Task 6.4: Lint and format all code
  - [x] Run `ruff check --fix .`
  - [x] Run `ruff format .`
  - [x] Fix any linting errors
  - [x] Verify clean output

- [x] Task 6.5: Update CHANGELOG or release notes
  - [x] Document all 5 issues fixed
  - [x] Note breaking changes (if any)
  - [x] Note new features (position lookup, conflict detection)

- [x] Task 6.6: Manual end-to-end UAT testing
  - [x] Issue #1: Submit trade, verify success (no database error)
  - [x] Issue #2: Test "buying 5 lots of bund calls @ 100" → extracts "bund"
  - [x] Issue #3: Test confirmation shows single match
  - [x] Issue #4: Test conflict warning displays in dashboard
  - [x] Issue #5: Test dashboard accessible at http://localhost:3000/
  - [x] Verify all UAT test cases pass

- [x] Task: Conductor - User Manual Verification 'Phase 6: Testing & Documentation' (Protocol in workflow.md)

---

## Files Modified Summary

| File | Changes | Phase |
|------|---------|-------|
| `src/pa_dealing/db/models/market.py` | Remove `last_trade_date` field | Phase 1 |
| `src/pa_dealing/agents/slack/chatbot.py` | Enhanced term cleaning | Phase 2 |
| `src/pa_dealing/agents/slack/chatbot.py` | Simplified SYSTEM_PROMPT | Phase 3 |
| `src/pa_dealing/db/schemas.py` | New: `MakoPositionInfo`, `EmployeeTradeRecord`, `ConflictRiskResult` | Phase 4 |
| `src/pa_dealing/db/repository.py` | New: `get_mako_position_info()`, `get_employee_trade_history()`, `calculate_conflict_risk()` | Phase 4 |
| `src/pa_dealing/agents/orchestrator/agent.py` | Integrate conflict detection | Phase 4 |
| `dashboard/src/components/RequestCard.tsx` | Show conflict badge | Phase 4 |
| `dashboard/src/components/RequestDetail.tsx` | Show conflict details | Phase 4 |
| `scripts/ops/run_uat_dev_simple.sh` | Add dashboard tmux window | Phase 5 |
| `tests/integration/test_position_lookup.py` | New test file | Phase 6 |
| `UAT_SECURITY_CONFIRMATION_FLOW.md` | Add conflict detection test | Phase 6 |

---

## Success Criteria

All tasks completed:
- ✅ All `[ ]` checkboxes marked as `[x]`
- ✅ All tests passing (unit + integration + E2E + Playwright)
- ✅ Code coverage >80%
- ✅ Manual UAT tests pass for all 5 issues
- ✅ No regressions introduced
- ✅ Dashboard loads without errors
- ✅ Trade submissions succeed
- ✅ Conflict detection working correctly
