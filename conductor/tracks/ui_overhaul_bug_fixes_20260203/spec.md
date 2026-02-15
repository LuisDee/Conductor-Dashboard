# Specification: UI Overhaul & Bug Fixes

> This spec is built incrementally. Each phase documents a bug or improvement found during exploratory testing. Once all issues are captured, a plan will be generated.

---

## Phase 1: PAD Search — PA Trading Panel Returns No Results for Known Trades

### Problem

When searching for instruments (e.g. "aapl", "ibgm") on the PAD Search page, the **Mako Trading panel** (left) returns results correctly, but the **PA Trading panel** (right) returns nothing — even though the same instruments are visible on the Holding Period page with confirmed executed trades.

### Root Cause

The PA trading search in `src/pa_dealing/services/pad_search.py` filters strictly by:

```python
.where(PADRequest.status == "approved")
```

However, the PAD request lifecycle progresses through:

```
pending_manager → pending_compliance → pending_smf16 → approved → executed
```

Once a trade is **executed**, its status changes from `"approved"` to `"executed"`. The holding period page correctly queries `status == "executed"`, but the PAD Search only looks at `status == "approved"` — creating a visibility gap where completed trades disappear from the conflict search.

### Impact

Compliance officers cannot see executed trades in the PAD Search conflict view. This defeats the purpose of the page, since the most important trades to cross-reference against Mako activity are the ones that were actually executed, not just approved.

### Expected Fix

1. Update the PA trading search filter in `src/pa_dealing/services/pad_search.py` to include both `approved` and `executed` statuses:
   ```python
   .where(PADRequest.status.in_(["approved", "executed"]))
   ```

2. Add a soft-delete exclusion filter — the `PADRequest` model has a `deleted_at` field that is not currently checked:
   ```python
   .where(PADRequest.deleted_at.is_(None))
   ```

3. Verify the fix returns results for instruments visible on the holding period page.

### Files Affected

- `src/pa_dealing/services/pad_search.py` — `search_pa_trading()` method

---

## Phase 2: PAD Search — UI Tightening & Dynamic Conflict Window

### 2a: Styling Changes

**Search Bar:**
- Change "Mako Trading" header text color to white
- Reduce vertical padding significantly — make it much slimmer
- Reduce border radius to `4px` (less rounded/bubbly)

**Panel Headers:**
- Left panel: Change header text from "Institutional trading activity" to just **"Mako Trading"**. Remove the subtitle entirely.
- Right panel: Remove "Approved employee trades" subtitle. Keep just **"PA Account Trading"**.
- Reduce vertical padding on both headers — compact, just enough to fit the text.
- Reduce border radius to `4px`.

**General:**
- Less padding/margins throughout — tighten everything up
- Sharper corners (`4px` border radius globally, not rounded)

### 2b: Remove 30-Day Risk Zone Legend

Remove the "30-Day Risk Zone" notification/legend card at the bottom of the PAD Search page. The highlighting functionality itself stays — just the explanatory banner goes away.

### 2c: Dynamic Conflict Detection Window

**Current state:** The date highlighting in PAD Search uses a hardcoded 30-day window (`isWithin30Days()` in `PADSearch.tsx`). Meanwhile, the Settings page has a "Mako Trading Lookback" slider that stores `mako_lookback_months` in the `RiskScoringConfig` — but this value is **not connected** to the PAD Search highlighting at all.

**Required changes:**

1. **Settings page** — Change "Mako Trading Lookback" from **months** to **days**:
   - Rename field label to "Mako Trading Lookback (Days)"
   - Change unit from "Mos" to "Days"
   - Default value: **30 days**
   - Slider/input range: 1–365 days
   - Rename the backend field from `mako_lookback_months` to `mako_lookback_days` (or keep the field name but change semantics — decide at plan time)

2. **PAD Search page** — Replace hardcoded 30-day window with the configured value:
   - Fetch `mako_lookback_days` from the risk scoring config on page load
   - Replace `isWithin30Days()` with a dynamic `isWithinLookback(dateStr, lookbackDays)` function
   - If the user changes the setting to e.g. 5 days, only instruments traded within the last 5 days get highlighted — instruments traded 20 days ago would NOT be highlighted

3. **Persistence** — When a user changes the slider on Settings and clicks "Save Changes", the new value should be reflected immediately on PAD Search (on next load/refresh).

### Files Affected

- `dashboard/src/pages/PADSearch.tsx` — styling, remove legend, dynamic lookback
- `dashboard/src/pages/Settings.tsx` — months → days conversion
- `dashboard/src/types/index.ts` — update `RiskScoringConfig` type if field name changes
- `dashboard/src/api/client.ts` — update if field name changes
- `src/pa_dealing/db/models/compliance.py` — default value update
- `src/pa_dealing/api/routes/config.py` — field rename if applicable
- `src/pa_dealing/api/schemas.py` — field rename if applicable

---

## Phase 3: Holding Periods — Dynamic Period & UI Overhaul

### 3a: Dynamic Holding Period from Settings

The holding period page currently hardcodes "30-day" in the subtitle ("Tracking mandatory 30-day ownership periods"). This must become dynamic, driven by the `holding_period_days` setting in compliance config.

**Required changes:**

1. The page subtitle should remove the hardcoded "30-day" — e.g. "Tracking mandatory ownership periods" or dynamically show the configured value.
2. "Days remaining" calculations on each row must use the configured `holding_period_days` value, not a hardcoded 30.
3. Fetch the holding period setting from the risk/compliance config on page load.
4. When a user changes the holding period in Settings and saves, the holding periods page should reflect the new value on next load.

### 3b: Remove Stat Blocks

Remove the three summary cards at the top of the page:
- "Ending Soon"
- "Next 14 Days"
- "Total Active"

Move the total active count to a small inline badge next to the page title, e.g. **"Holding Periods · 5 active"**.

### 3c: Filter Bar

- Reduce vertical padding
- Reduce border radius to `4px` on dropdowns and inputs

### 3d: Table Structure Changes

- **Add new "Description" column** — move security description out of the Instrument column into its own column
- **Instrument column** — show only the ticker/identifier (no description)
- **Remove "Status" column** — only active periods are shown anyway, column is redundant

### 3e: Table Header & Row Styling

**Table Header:**
- Reduce vertical padding to `6px 12px`

**Table Rows:**
- Reduce vertical padding to `6px 12px`
- **Employee column**: Show only the email, remove the name
- **Period End column**: Remove "30-DAY REQUIREMENT" subtitle text, show only the date
- **ISIN column**: Keep "Not Available" fallback text as-is (it's good for this product)

### 3f: Global Theme Rules (Apply Across All Pages)

These rules should be treated as a design system baseline going forward:

**Border Radius:**
- Use `4px` everywhere — inputs, cards, buttons, badges, dropdowns
- Less bubbly, more subtle

**Padding & Spacing:**
- Reduce vertical padding throughout — keep things compact
- Search bars: minimal vertical padding (`2–4px`)
- Table headers: `6px` vertical
- Table rows: `6px` vertical
- Panel/card headers: `6px` vertical
- Filter bars: `10px` vertical

**Headers & Labels:**
- Keep headers single-line where possible
- Flatten the UI — less visual hierarchy, more density

### Files Affected

- `dashboard/src/pages/HoldingPeriods.tsx` — stat block removal, badge, table restructure, dynamic period
- `dashboard/src/pages/Settings.tsx` — ensure holding period days setting exists and is editable
- `dashboard/src/types/index.ts` — holding period config type if needed
- `dashboard/src/api/client.ts` — fetch holding period config
- Global/shared CSS or Tailwind classes — `4px` border radius, compact padding baseline

---

## Phase 4: Mako Conflicts — Correct Count, Position Display & PAD Search Filtering

### 4a: Fix Mako Conflicts Count

**Problem:** The "Mako Conflicts" count currently shows **4497**, which is the number of distinct securities Mako is trading — NOT the number of actual conflicts. A conflict is defined as an **outstanding user who has trades that overlap instruments with Mako**. The correct count is believed to be around 1.

**Expected fix:**
- The conflict count must reflect the number of **distinct employees** (or employee+instrument pairs — clarify at plan time) where a PAD trade overlaps with a Mako-traded instrument.
- Not the count of Mako securities.

### 4b: Position Display — Long/Short with Units

Currently the Mako Conflicts panel shows position as a plain number (e.g. "5"). This should be reformatted to show direction, units, and a signed value:

- Positive position → **"Long 5 units (+5)"** with `(+5)` in **green**
- Negative position → **"Short 50 units (-50)"** with `(-50)` in **red**

Apply this to both:
- **Employee position** column
- **Mako position** column

### 4c: Conflict Type Definitions

The "Conflict Type" column must use these three specific types:

- **Parallel** — employee and Mako are trading the **same security in the same direction** (both long or both short)
- **Opposite** — employee and Mako are trading the **same security in opposite directions** (one long, one short)
- **Restricted Instrument** — employee is trading a security that appears on the **restricted instruments list** (where `is_active = true`) — regardless of Mako position

These should be derived from the position data and the restricted instruments table (Phase 5). A single conflict row could potentially be both Parallel/Opposite AND Restricted if the instrument is on the restricted list.

### 4d: Table Column Changes

- **Keep** the "Conflict Type" column (with definitions from 4c above)
- **Add** a new column (no header text) containing a **"View in PAD Search"** link/button
  - Clicking this navigates to the PAD Search page with pre-applied filters for that user and instrument (see 4e below)

### 4e: PAD Search — Double-Click Filter System

Add an interactive filtering mechanism to both panels on the PAD Search page:

**How it works:**
1. User double-clicks any cell value in either table (e.g. "GLO" in the Company column, or "2026-01-26" in the Last Traded column, or a user name in the PA panel)
2. That value is added as an **active filter** for that column
3. The filter does NOT hide non-matching rows — instead it **sorts matching rows to the top** and **greys out** non-matching rows (they remain visible but visually de-emphasized)
4. Multiple filters can be stacked (e.g. filter by company AND date)

**Filter indicators:**
- When any filter is active, a **"Clear Filters"** button/chip appears near the search bar
- Each active filter should be visible as a removable chip/tag (e.g. `Company: GLO ×`)
- Clicking "Clear Filters" removes all active filters and restores normal display

**Applies to both panels:**
- **Mako Trading (left)**: Can filter on Company, Portfolio, Desk, Symbol, Description, Inst Type, Last Traded, Position Date
- **PA Trading (right)**: Can filter on Division, Employee, Symbol, Description, Approved date

### 4f: "View in PAD Search" Navigation

When clicking "View in PAD Search" from the Mako Conflicts page:

1. Navigate to `/pad-search`
2. Pre-populate the search bar with the instrument symbol
3. Apply two filters automatically:
   - **PA panel**: Employee filter (sort that employee to top, grey out others)
   - **Both panels**: Symbol/instrument filter (sort matching instrument to top)
4. The URL should encode these filters as query parameters so the state is shareable/bookmarkable (e.g. `/pad-search?q=AAPL&employee=john.doe&symbol=AAPL`)

### Files Affected

- Dashboard page for Mako Conflicts (explore at plan time to identify exact file) — count fix, position display, column changes, "View in PAD Search" link
- `dashboard/src/pages/PADSearch.tsx` — double-click filter system, filter chips, "Clear Filters" button, URL query param support
- Backend conflict count endpoint — fix to count actual overlapping users, not distinct Mako securities
- `dashboard/src/types/index.ts` — filter state types
- React Router config — support for query params on `/pad-search`

---

## Phase 5: Restricted Instruments — Deprecate Confluence Sync, New Standalone Page

### 5a: Deprecate Confluence Scraping System

The current system scrapes a Confluence page for restricted instruments using BeautifulSoup HTML parsing. This entire system must be removed:

**Remove from Mako Conflicts page:**
- "Sync Now" button
- "Last Sync" timestamp/status

**Remove from Settings page:**
- "Restricted List Sync" configuration section (Confluence URL, sync interval, etc.)

**Remove from backend:**
- All code responsible for Confluence scraping / HTML parsing (BeautifulSoup)
- Sync scheduler/cron job if one exists
- Confluence-related config fields from the risk/compliance config model
- Any Confluence sync API endpoints

Confluence is **no longer the source of truth** for restricted instruments.

### 5b: New Restricted Instruments Page

Create a dedicated `/restricted-instruments` page (new top-level nav item) that serves as the sole management interface for the restricted list.

**Table columns:**
- **Instrument** (`inst_symbol`) — the Mako internal identifier / ticker
- **ISIN** — International Securities Identification Number
- **Reason** — free text explaining why it's restricted
- **Date Added** — auto-populated when created
- **Updated By** — email of the user who last modified the record
- **Status** — Active / Inactive (only active shown by default, toggle to see all)

### 5c: Instrument Matching Logic

**Critical:** The matching logic must be robust. When checking whether a PAD request conflicts with a restricted instrument:

- If a user enters a **ticker** in the `inst_symbol` field of a restricted instrument, it should still block matching trades — the system must cross-reference through the same waterfall/resolution logic used elsewhere (Bloomberg → MapInstSymbol → Product) to resolve identifiers
- Match on **ISIN** as well — if the restricted instrument has an ISIN, any PAD request with the same ISIN should be blocked
- Match should be case-insensitive
- This needs thorough review at plan time to ensure no gaps in the matching chain

### 5d: Add/Edit via Modal

A modal popup for adding a new restricted instrument:

**Fields:**
- **Instrument** (`inst_symbol`) — required, text input
- **ISIN** — optional, text input (validated as 12-char format if provided)
- **Reason** — required, text area

**Auto-populated (not user-editable):**
- **Date Added** — current timestamp
- **Status** — defaults to "Active"
- **Updated By** — current user's email

### 5e: Remove (Soft Delete)

- Each row has a "Remove" action (button or icon)
- Removing an instrument sets its status to **Inactive** (soft delete — record is preserved)
- Inactive instruments are hidden from the default view but viewable via a toggle/filter
- Removing also logs the `updated_by` email and timestamp

### 5f: Audit Trail

Every modification to the restricted instruments table (add, remove, edit) must be logged to a database audit table:

**Audit record fields:**
- Restricted instrument ID
- Action type (`added`, `removed`, `edited`)
- Changed by (user email)
- Timestamp
- Before/after values (for edits)

This is internal — not exposed in the UI, but queryable for compliance audits.

### Files Affected

- **New:** `dashboard/src/pages/RestrictedInstruments.tsx` — new page with table, modal, toggle
- **New:** `src/pa_dealing/api/routes/restricted_instruments.py` — CRUD endpoints
- **New:** `src/pa_dealing/services/restricted_instruments.py` — service layer
- **New or extend:** DB model for restricted instruments table + audit log table
- `dashboard/src/components/layout/Sidebar.tsx` — add nav item
- `dashboard/src/App.tsx` — add route
- Mako Conflicts page — remove Sync Now / Last Sync UI
- Settings page — remove Restricted List Sync section
- Backend — remove all Confluence scraping code (BeautifulSoup, sync endpoints, scheduler)
- Risk engine / matching logic — update to use new restricted instruments table with robust cross-reference matching
