# Plan: Fix PA Search Division Lookup

## Phase 1: Backend Model Update

- [x] **1.1** `src/pa_dealing/db/models/core.py` — Add `division_id` mapped column
  to `OracleEmployee` model (`BigInteger`, nullable). Column already exists in DB,
  this just exposes it to the ORM.

## Phase 2: Update PA Search Division Query

- [x] **2.1** `src/pa_dealing/services/pad_search.py` — Replace the division subquery.
  Change from `COALESCE(boffin_group, cost_centre)` to a subquery that joins
  `oracle_employee.division_id → oracle_department.id` and selects `oracle_department.name`.
  Use the reference schema from settings (same as `risk_scoring_service.py`).
- [x] **2.2** `src/pa_dealing/services/pad_search.py` — Add COALESCE fallback: if
  `division_id` is NULL or no matching department row, fall back to first value from
  `boffin_group` (split on `;`), then `cost_centre`. Use SQL `COALESCE(department_subq,
  SPLIT_PART(boffin_group, ';', 1), cost_centre)`.

## Phase 3: Tests

- [x] **3.1** Write unit tests: division_id present → returns department name
- [x] **3.2** Write unit tests: division_id NULL → falls back to first boffin_group value
- [x] **3.3** Write unit tests: both NULL → falls back to cost_centre
- [x] **3.4** Run full regression suite (`./scripts/test-runner.sh unit`)
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: Verify & Deploy

- [x] **4.1** Rebuild dashboard container, verify Division column shows clean names
- [x] **4.2** Verify Division double-click filter creates clean chip (e.g., "Division: Trading")
- [x] **4.3** Remove debug console.logs if any remain from dimming bug track
- [x] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)

## Phase 5: Cross-Panel Filter Fix (Symbol + Description)

- [x] **5.1** Add `CROSS_PANEL_COLUMNS` constant for Symbol and Description
  - `dashboard/src/pages/PADSearch.tsx` line 54
- [x] **5.2** Fix cross-panel filtering in `sortedMako` memo
  - Include PA panel filters for Symbol/Description columns
- [x] **5.3** Fix cross-panel filtering in `sortedPA` memo
  - Include Mako panel filters for Symbol/Description columns
- [x] **5.4** Manual verification: click Symbol in right panel, verify left panel dims non-matching rows
- [x] Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)
