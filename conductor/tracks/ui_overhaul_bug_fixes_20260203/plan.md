# Implementation Plan: UI Overhaul & Bug Fixes

> Derived from spec.md. Each phase maps to a spec phase. Tasks are ordered by dependency.

---

## Phase 1: PAD Search — Fix PA Trading Status Filter ✅

- [x] **1.1** `src/pa_dealing/services/pad_search.py` line 257 — Change `.where(PADRequest.status == "approved")` to `.where(PADRequest.status.in_(["approved", "executed"]))`
- [x] **1.2** `src/pa_dealing/services/pad_search.py` — Add `.where(PADRequest.deleted_at.is_(None))` soft-delete exclusion immediately after the status filter
- [x] **1.3** Verify search returns results for instruments visible on the holding period page (unit tests updated with new status filter + soft-delete exclusion tests)

---

## Phase 2: PAD Search — UI Tightening & Dynamic Conflict Window ✅

### 2a: Styling (all in `dashboard/src/pages/PADSearch.tsx`)

- [x] **2.1** Search bar card — reduce vertical padding to `py-1`, reduce border radius to `rounded-[4px]`
- [x] **2.2** Left panel header (line 240) — change text from "Institutional trading activity" subtitle to just **"Mako Trading"** title only. Remove subtitle line 242. Change `rounded-t-xl` to `rounded-t-[4px]`. Reduce padding to `px-3 py-1.5`
- [x] **2.3** Right panel header (line 279) — remove subtitle. Keep title "PA Account Trading". Same padding/radius changes as left panel
- [x] **2.4** Table wrappers, cards, and legend — replace all `rounded-xl` / `rounded-lg` with `rounded-[4px]`
- [x] **2.5** General padding reduction — reduce `space-y-6` to `space-y-3`, reduce `gap-4` to `gap-2` where appropriate throughout the page
- [x] **2.6** Search input (line 218) — reduce to `py-1 text-sm` instead of `text-lg`

### 2b: Remove 30-Day Risk Zone Legend

- [x] **2.7** `dashboard/src/pages/PADSearch.tsx` — Delete the legend Card block (lines 306-316)

### 2c: Dynamic Conflict Window — Backend

- [x] **2.8** `src/pa_dealing/db/models/compliance.py` line 71 — Change default from `"mako_lookback_months": 3` to `"mako_lookback_days": 30`
- [x] **2.9** `src/pa_dealing/api/schemas.py` — In `RiskScoringConfigResponse` (line ~316) and `RiskScoringConfigUpdate` (line ~341), rename `mako_lookback_months: int` to `mako_lookback_days: int`. Update validation range from `1-24` to `1-365`
- [x] **2.10** `src/pa_dealing/api/routes/config.py` — In `update_risk_scoring_config()`, update the field reference from `mako_lookback_months` to `mako_lookback_days` (the `set_value()` call and audit change tracking)
- [x] **2.11** `src/pa_dealing/api/routes/config.py` — In `_config_to_response()`, handle migration: if `mako_lookback_months` exists but `mako_lookback_days` doesn't, convert `months * 30` to days for backward compat

### 2c: Dynamic Conflict Window — Frontend

- [x] **2.12** `dashboard/src/types/index.ts` — In `RiskScoringConfig` (line 324) and `RiskScoringConfigUpdate` (line 336), rename `mako_lookback_months` to `mako_lookback_days`
- [x] **2.13** `dashboard/src/pages/Settings.tsx` — Rename state variable `lookbackMonths` → `lookbackDays`. Update label from "Mako Trading Lookback" to "Mako Trading Lookback (Days)". Change unit from "Mos" to "Days". Update input range from `min=1 max=24` to `min=1 max=365`. Initialize from `riskConfig.mako_lookback_days`. Update the save payload key.
- [x] **2.14** `dashboard/src/pages/PADSearch.tsx` — Add `useQuery` call to fetch risk scoring config (`config.getRiskScoring()`). Extract `mako_lookback_days` (default 30). Replace `isWithin30Days()` with `isWithinLookback(dateStr, lookbackDays)` that uses the dynamic value. Update the `DateCell` component to accept `lookbackDays` prop.
- [x] **2.15** `dashboard/src/api/client.ts` — If field name changed in config update payload, update accordingly (types auto-propagate)

---

## Phase 3: Holding Periods — Dynamic Period & UI Overhaul ✅

### 3a: Dynamic Holding Period

- [x] **3.1** `dashboard/src/pages/HoldingPeriods.tsx` — Uses settings query for dynamic holding_period_days value
- [x] **3.2** `dashboard/src/pages/HoldingPeriods.tsx` — Update subtitle to "Tracking mandatory ownership periods" (removed hardcoded "30-day")

### 3b: Remove Stat Blocks

- [x] **3.3** `dashboard/src/pages/HoldingPeriods.tsx` — Removed 3-column stat cards grid (Ending Soon, Next 14 Days, Total Active)
- [x] **3.4** `dashboard/src/pages/HoldingPeriods.tsx` — Added inline count badge next to title: "Holding Periods · N active"

### 3c: Filter Bar

- [x] **3.5** `dashboard/src/pages/HoldingPeriods.tsx` — Filter card: reduced padding, rounded-[4px] on inputs/dropdowns

### 3d: Table Structure

- [x] **3.6** `dashboard/src/pages/HoldingPeriods.tsx` — Split into separate "Instrument" (ticker) and "Description" (security_name) columns
- [x] **3.7** `dashboard/src/pages/HoldingPeriods.tsx` — Removed "Status" column (redundant)

### 3e: Table Header & Row Styling

- [x] **3.8** `dashboard/src/pages/HoldingPeriods.tsx` — Employee column: shows only email, removed name
- [x] **3.9** `dashboard/src/pages/HoldingPeriods.tsx` — Period End column: removed requirement subtitle, shows only date
- [x] **3.10** Compact padding applied throughout page (space-y-3, gap-2)

### 3f: Global Theme Rules

- [x] **3.11** Audited all pages and components for `rounded-xl`, `rounded-lg`, `rounded-2xl` → replaced with `rounded-[4px]` across: Card.tsx, Table.tsx, SearchableSelect.tsx, SecuritySearchInput.tsx, Sidebar.tsx, index.css, Settings, Breaches, ExecutionTracking, AuditLog, Reports, MakoConflicts
- [x] **3.12** Table component header `borderRadius` changed from `12px 12px 0 0` to `4px 4px 0 0`; Card base styles updated in CSS and component
- [x] **3.13** Filter bars standardized to `px-3 py-[10px]` with `gap-2` across Breaches, ExecutionTracking, AuditLog, Reports; page-level spacing reduced to `space-y-3`

---

## Phase 4: Mako Conflicts — Count Fix, Position Display & PAD Search Filtering ✅

### 4a: Fix Conflict Count

- [x] **4.1** `src/pa_dealing/services/pad_service.py` — Fixed `mako_conflicts` count: replaced `COUNT(ProductUsage.inst_symbol.distinct())` with proper join of employee positions (executed PADRequests) against MakoPosition, counting only actual overlaps where employee has non-zero net position

### 4b: Position Display

- [x] **4.2** `dashboard/src/pages/MakoConflicts.tsx` — Updated "Employee Position" column: shows "Long N (+N)" in green or "Short N (-N)" in red
- [x] **4.3** `dashboard/src/pages/MakoConflicts.tsx` — Updated "Mako Position" column with same Long/Short formatting
- [x] **4.4** `dashboard/src/types/index.ts` — Fixed type: `mako_position: number` (was string)

### 4c: Conflict Type Logic

- [x] **4.5** `src/pa_dealing/services/pad_service.py` — Replaced hardcoded `"same_security"` with dynamic conflict types: `"restricted_instrument"` (on restricted list), `"parallel"` (same direction), `"opposite"` (opposing directions)
- [x] **4.6** `dashboard/src/pages/MakoConflicts.tsx` — Conflict type badges: red for restricted, amber for parallel, blue for opposite

### 4d: "View in PAD Search" Column

- [x] **4.7** `dashboard/src/pages/MakoConflicts.tsx` — Added "PAD Search" link column navigating to `/pad-search?q={inst_symbol}&employee={employee_name}`

### 4e: PAD Search — Double-Click Filter System

- [x] **4.8** `dashboard/src/pages/PADSearch.tsx` — Added filter state with `useSearchParams` URL param initialization on mount
- [x] **4.9** `dashboard/src/pages/PADSearch.tsx` — Double-click handler on table cells adds `{panel, column, value}` filter
- [x] **4.10** `dashboard/src/pages/PADSearch.tsx` — Client-side filter sorts matching rows to top, non-matching get `opacity-40`
- [x] **4.11** `dashboard/src/pages/PADSearch.tsx` — Filter chip UI with removable tags and "Clear All" button
- [x] **4.12** `dashboard/src/pages/PADSearch.tsx` — Filter state synced to URL via `useSearchParams` (replace mode)
- [x] **4.13** `dashboard/src/pages/PADSearch.tsx` — Auto-triggers search when `q` param present on mount from navigation

---

## Phase 5: Restricted Instruments — Deprecate Confluence, New Page ✅

### 5a: Deprecate Confluence Sync — Backend

- [x] **5.1** `src/pa_dealing/services/restricted_list_sync.py` — Delete entire file
- [x] **5.2** `src/pa_dealing/integrations/confluence_client.py` — Delete entire file
- [x] **5.3** `src/pa_dealing/agents/monitoring/jobs.py` — Remove `check_restricted_list_sync()` job and `JobType.RESTRICTED_LIST_SYNC`
- [x] **5.4** `src/pa_dealing/agents/monitoring/scheduler.py` — Remove restricted list sync job registration from `start()` and `reschedule_restricted_list_sync()`
- [x] **5.5** `src/pa_dealing/api/routes/config.py` — Remove endpoints: `GET /restricted-list-sync-status`, `POST /restricted-list-sync`, `PUT /restricted-list-sync-interval`
- [x] **5.6** `src/pa_dealing/config/settings.py` — Remove Confluence-related settings: `confluence_url`, `confluence_username`, `confluence_api_token`, `restricted_list_page_id`, `restricted_list_space`, `restricted_list_title`, `restricted_list_sync_interval_minutes`
- [x] **5.7** `.env.example` — Remove `CONFLUENCE_*` and `RESTRICTED_LIST_SYNC_INTERVAL_MINUTES` vars
- [x] **5.8** `pyproject.toml` — Remove `atlassian-python-api` and `beautifulsoup4` dependencies (verified no other code uses them)

### 5a: Deprecate Confluence Sync — Frontend

- [x] **5.9** `dashboard/src/components/RestrictedInstrumentsSection.tsx` — Deleted entirely (replaced by standalone page). Also deleted `SyncStatusCard.tsx`.
- [x] **5.10** `dashboard/src/pages/Settings.tsx` — Removed "Restricted List Sync" card, sync state variables, and sync status query
- [x] **5.11** `dashboard/src/pages/MakoConflicts.tsx` — Removed `RestrictedInstrumentsSection` import and usage
- [x] **5.12** `dashboard/src/api/client.ts` — Removed `config.getRestrictedListSyncStatus()`, `config.triggerRestrictedListSync()`, `config.updateSyncInterval()` methods
- [x] **5.13** `dashboard/src/types/index.ts` — Removed `SyncStatus` interface
- [x] **5.14** Removed test files: `tests/unit/test_confluence_client.py`, `tests/unit/test_restricted_list_sync.py`, `tests/integration/test_confluence_sync.py`

### 5b: New Restricted Instruments DB Model & Audit Table

- [x] **5.15** `src/pa_dealing/db/models/compliance.py` — Extended `RestrictedSecurity` model with `updated_by` (String) and `updated_at` (DateTime, onupdate)
- [x] **5.16** `src/pa_dealing/db/models/compliance.py` — Created `RestrictedSecurityAuditLog` model with id, restricted_security_id (FK), action, changed_by, timestamp, before_values (JSONB), after_values (JSONB)
- [x] **5.17** Created manual Alembic migration `20260203_1800_add_restricted_security_audit.py`

### 5c: New Restricted Instruments Backend Service

- [x] **5.18** Create `src/pa_dealing/services/restricted_instruments.py` with:
  - `list_restricted_instruments(session, include_inactive, search)` — existing logic from dashboard route
  - `add_restricted_instrument(session, inst_symbol, isin, reason, user_email)` — insert + audit log
  - `remove_restricted_instrument(session, id, user_email)` — set `is_active=False` + audit log
  - `get_restricted_instrument(session, id)` — single item lookup

### 5d: New Restricted Instruments API Routes

- [x] **5.19** Create `src/pa_dealing/api/routes/restricted_instruments.py` with:
  - `GET /restricted-instruments` — list (with search, include_inactive params) — move from dashboard route
  - `POST /restricted-instruments` — add new (body: inst_symbol, isin, reason) — compliance auth required
  - `DELETE /restricted-instruments/{id}` — soft-delete (set inactive) — compliance auth required
- [x] **5.20** Register new router in the FastAPI app
- [x] **5.21** Remove old `GET /dashboard/restricted-instruments` endpoint from `src/pa_dealing/api/routes/dashboard.py` (lines 278-313)

### 5e: Instrument Matching Logic

- [x] **5.22** Review and document the matching chain: `check_restricted_list_comprehensive()` in `repository.py:443-531` already performs multi-identifier OR-based matching (inst_symbol, ISIN, bloomberg). Identity resolution happens upstream via `resolve_instrument_identity()` before the restricted check is called.
- [x] **5.23** ISIN matching already implemented: exact case-insensitive match via `func.upper(RestrictedSecurity.isin) == isin.upper()` at `repository.py:470`
- [x] **5.24** Case-insensitive matching already implemented: all comparisons use `func.upper()` normalization

### 5f: New Restricted Instruments Page — Frontend

- [x] **5.25** Create `dashboard/src/pages/RestrictedInstruments.tsx`:
  - Table with columns: Instrument, ISIN, Reason, Date Added, Updated By, Status
  - Search bar with debounce
  - "Show Inactive" toggle
  - "Add Instrument" button opening a modal
  - Each row has a "Remove" action (sets inactive)
  - Follow global theme: `4px` border radius, compact padding
- [x] **5.26** Create add modal component with fields: Instrument (required), ISIN (optional, validated 12-char), Reason (required). Auto-populated: date, status=active, updated_by=current user email
- [x] **5.27** `dashboard/src/api/client.ts` — Add new `restrictedInstruments` API namespace: `list()`, `add()`, `remove()`
- [x] **5.28** `dashboard/src/types/index.ts` — Update `RestrictedInstrument` type to include `updated_by: string | null` and `updated_at: string | null`

### 5g: Navigation & Routing

- [x] **5.29** `dashboard/src/components/layout/Sidebar.tsx` — Add nav item: `{ path: '/restricted-instruments', label: 'Restricted List', icon: <ShieldAlert />, access: 'compliance' }`. Place after Mako Conflicts in the Operations section.
- [x] **5.30** `dashboard/src/App.tsx` — Add route: `<Route path="/restricted-instruments" element={<ProtectedRoute access="compliance"><RestrictedInstruments /></ProtectedRoute>} />`
- [x] **5.31** Remove `RestrictedInstrumentsSection` component import from `MakoConflicts.tsx` (the inline section on that page is replaced by the standalone page)
- [x] **5.32** Delete `dashboard/src/components/RestrictedInstrumentsSection.tsx` and `dashboard/src/components/SyncStatusCard.tsx`

---

## Execution Order & Dependencies

```
Phase 1 (standalone — no dependencies)
  ↓
Phase 2 (depends on Phase 1 for PA results to verify)
  ↓
Phase 3 (depends on Phase 2c for the mako_lookback_days field rename)
  ↓
Phase 5 (standalone from Phases 1-3, but should be done before Phase 4)
  ↓
Phase 4 (depends on Phase 5 for RestrictedSecurity table used in conflict type logic)
```

**Recommended implementation sequence:**
1. Phase 1 — quick fix, immediate value
2. Phase 2a + 2b — styling, no backend changes
3. Phase 2c — backend field rename + frontend wiring
4. Phase 3 — holding periods overhaul
5. Phase 5a-5d — deprecate Confluence, build new restricted instruments backend
6. Phase 5e-5g — restricted instruments frontend + matching logic
7. Phase 4a-4c — conflict count fix + position display + conflict types
8. Phase 4d-4e — PAD Search filter system + "View in PAD Search" navigation
9. Phase 3f — global theme audit (final pass)
