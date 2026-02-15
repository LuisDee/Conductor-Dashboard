# Specification: PAD Search (Conflict View)

## Overview

Replicate the "Conflict Search" functionality from the legacy compliance portal. This provides a unified side-by-side view of Mako institutional trading activity and Employee Personal Account Dealing (PAD) trades, allowing compliance officers to identify potential conflicts of interest (e.g., front-running) via a single search bar.

**Priority**: High
**Type**: Feature Implementation
**Branch**: DSS-4074

## Background

### Current System
The legacy compliance portal has a dedicated "PA Dealing Search" page that provides:
- Split-view layout with two synchronized panels
- Mako Trading (Left): Institutional activity from `ProductUsage`
- PA Account Trading (Right): Employee approved trades from `PersonalAccountDealing`
- Single search bar that updates both panels simultaneously
- 30-day "Risk Zone" highlighting for recent trades

### Problems / Gap
1. Our current PA Dealing dashboard lacks a unified conflict search view
2. Compliance cannot easily cross-reference employee trades against firm trading activity
3. No visual highlighting for trades within the 30-day risk window

### Solution
Implement a new "PAD Search" page accessible as a top-level navigation item that replicates the legacy split-view search functionality using our existing data models and MAKO design system.

---

## Functional Requirements

### FR-1: Navigation & Page Structure

**FR-1.1: Top-Level Navigation**
- Add "PAD Search" as a new top-level navigation item in the sidebar
- Position after existing compliance-related items
- Icon: Search or magnifying glass icon

**FR-1.2: Page Layout**
```
┌─────────────────────────────────────────────────────────────────┐
│ PAD Search                                                       │
│ Cross-reference employee trades with Mako trading activity      │
├─────────────────────────────────────────────────────────────────┤
│ [🔍 Search by ticker, ISIN, SEDOL, or description...         ]  │
├─────────────────────────────────────────────────────────────────┤
│                              │                                   │
│   MAKO TRADING (LEFT)        │     PA ACCOUNT TRADING (RIGHT)   │
│                              │                                   │
│   ┌──────────────────────┐   │   ┌──────────────────────────┐   │
│   │ Company | Portfolio  │   │   │ Division | Name          │   │
│   │ Desk    | Symbol     │   │   │ Symbol   | Description   │   │
│   │ Desc    | Type       │   │   │ Is Deriv | Last Traded   │   │
│   │ Last Trade | Pos Date│   │   │                          │   │
│   └──────────────────────┘   │   └──────────────────────────┘   │
│                              │                                   │
│         [← RESIZER →]        │                                   │
└─────────────────────────────────────────────────────────────────┘
```

### FR-2: Unified Search Interface

**FR-2.1: Search Bar**
- Full-width search input at top of page
- Placeholder: "Search by ticker, ISIN, SEDOL, or description..."
- Debounced input (300ms) to prevent excessive API calls
- Single string search (case-insensitive "contains" matching)

**FR-2.2: Parallel Search Execution**
- When user types, trigger BOTH panel searches simultaneously
- Both tables show loading spinners independently
- Tables update as their respective API calls complete

**FR-2.3: Resizable Split View**
- Draggable vertical divider between panels
- Default: 50/50 split
- Persist user's resize preference in localStorage

### FR-3: Mako Trading Panel (Left)

**FR-3.1: Data Source**
- Primary: `bo_airflow.oracle_product_usage` (aliased as `ProductUsage`)
- Joined with `oracle_product` for descriptions
- Joined with `oracle_portfolio_group` → `oracle_cost_centre` for desk names

**FR-3.2: 3-Tier Lookup (Symbol Resolution)**
The search input is translated to `inst_symbol` values using a waterfall approach:

| Tier | Table | Fields Searched |
|------|-------|-----------------|
| 1 | `oracle_bloomberg` | isin, sedol, ticker, description |
| 2 | `oracle_map_inst_symbol` | exch_symbol, description |
| 3 | `oracle_product` | description |

- Search uses `icontains` (SQL `LIKE %value%`) - case-insensitive fuzzy matching
- Once symbols are resolved, fetch trading data from `ProductUsage`
- The lookup tier does NOT affect which fields are displayed

**FR-3.3: Table Columns**

| Column | Source | Notes |
|--------|--------|-------|
| Company | `ProductUsage.company` | FK to Company table |
| Portfolio | `ProductUsage.portfolio` | FK to Portfolio table |
| Desk | `OraclePortfolioGroup` → `OracleCostCentre.display_name` | Annotated via subquery |
| Symbol | `ProductUsage.inst_symbol` | Mako internal identifier |
| Description | `OracleProduct.description` | Joined via inst_symbol |
| Inst Type | `ProductUsage.inst_type` | Single character code |
| Last Traded | `ProductUsage.last_trade_date` | **Highlight if < 30 days** |
| Position Date | `ProductUsage.last_position_date` | **Highlight if < 30 days** |

**FR-3.4: Data Scope**
- Show ALL historical data matching the resolved symbols
- No date filter on the query itself
- The 30-day threshold is visual only (highlighting)

### FR-4: PA Account Trading Panel (Right)

**FR-4.1: Data Source**
- Primary: `pad_request` table
- Joined with `oracle_employee` for employee details
- Joined with `oracle_division` for division name

**FR-4.2: Row Filter Logic**
- **Status**: Only show `status = 'approved'` requests
- We do NOT have an `is_deleted` field - no filter needed for this

**FR-4.3: Search Logic**
Search across these fields in `pad_request`:
- `inst_symbol`
- `isin`
- `security_description`
- Employee name (via join)

**FR-4.4: Table Columns**

| Column | Source | Notes |
|--------|--------|-------|
| Division | `OracleDivision.description` | Via `oracle_employee.division_id` |
| Name | `OracleEmployee.forename + surname` | Full name concatenation |
| Symbol | `pad_request.inst_symbol` | Mako internal identifier |
| Description | `pad_request.security_description` | User-provided description |
| Is Derivative | `pad_request.is_derivative` | Boolean flag (Y/N display) |
| Last Traded | `pad_request.approved_at` | **Highlight if < 30 days** |

### FR-5: Visual Compliance Highlighting

**FR-5.1: 30-Day Risk Zone**
- Any date in "Last Traded" or "Position Date" columns within the last 30 days MUST be displayed in **Bold** and **MAKO Gold (#B28C54)** text
- This applies to BOTH panels

**FR-5.2: Empty States**
- Left panel: "No Mako trading activity found for this search"
- Right panel: "No approved PA trades found for this search"
- Initial state (no search): "Enter a search term to find conflicts"

### FR-6: API Endpoints

**FR-6.1: Mako Trading Search**
```
GET /api/pad-search/mako-trading?q={search_term}

Response: {
  data: [
    {
      company: string;
      portfolio: string;
      desk: string | null;
      inst_symbol: string;
      description: string | null;
      inst_type: string | null;
      last_trade_date: string | null;  // ISO date
      last_position_date: string | null;  // ISO date
    }
  ]
}
```

**FR-6.2: PA Trading Search**
```
GET /api/pad-search/pa-trading?q={search_term}

Response: {
  data: [
    {
      division: string | null;
      employee_name: string;
      inst_symbol: string | null;
      security_description: string | null;
      is_derivative: boolean;
      last_traded_date: string | null;  // ISO date
    }
  ]
}
```

---

## Technical Requirements

### TR-1: Backend - New Search Service

**File**: `src/pa_dealing/services/pad_search.py`

```python
class PADSearchService:
    """Service for PAD Search conflict detection."""

    async def search_mako_trading(
        self, session: AsyncSession, query: str
    ) -> list[MakoTradingResult]:
        """
        3-tier symbol lookup then fetch from ProductUsage.
        """
        # Step 1: Resolve query to inst_symbols via waterfall
        symbols = await self._resolve_symbols(session, query)

        # Step 2: Fetch trading data for those symbols
        return await self._fetch_mako_trading(session, symbols)

    async def search_pa_trading(
        self, session: AsyncSession, query: str
    ) -> list[PATradingResult]:
        """
        Search approved PAD requests.
        """
        # Search across inst_symbol, isin, description, employee name
        pass

    async def _resolve_symbols(
        self, session: AsyncSession, query: str
    ) -> list[str]:
        """
        3-tier waterfall: Bloomberg -> MapInstSymbol -> Product
        """
        # Tier 1: Bloomberg (ISIN, SEDOL, ticker, description)
        symbols = await self._search_bloomberg(session, query)
        if symbols:
            return symbols

        # Tier 2: MapInstSymbol (exch_symbol)
        symbols = await self._search_map_inst_symbol(session, query)
        if symbols:
            return symbols

        # Tier 3: Product (description fallback)
        return await self._search_product(session, query)
```

### TR-2: Backend - API Routes

**File**: `src/pa_dealing/api/routes/pad_search.py`

```python
router = APIRouter(prefix="/pad-search", tags=["pad-search"])

@router.get("/mako-trading")
async def search_mako_trading(
    q: str = Query(..., min_length=2),
    user: CurrentUser = Depends(get_current_user),
):
    """Search Mako institutional trading activity."""
    _require_compliance_or_admin(user)

    service = get_pad_search_service()
    async with get_session() as session:
        results = await service.search_mako_trading(session, q)
        return APIResponse(data=results)

@router.get("/pa-trading")
async def search_pa_trading(
    q: str = Query(..., min_length=2),
    user: CurrentUser = Depends(get_current_user),
):
    """Search approved employee PA trades."""
    _require_compliance_or_admin(user)

    service = get_pad_search_service()
    async with get_session() as session:
        results = await service.search_pa_trading(session, q)
        return APIResponse(data=results)
```

### TR-3: Frontend - New Page Component

**File**: `dashboard/src/pages/PADSearch.tsx`

Key components:
- `PADSearchPage` - Main page container
- `SearchBar` - Debounced search input
- `MakoTradingPanel` - Left table
- `PATradingPanel` - Right table
- `ResizableSplitView` - Draggable divider wrapper

### TR-4: Frontend - API Client

**File**: `dashboard/src/api/client.ts`

```typescript
export const padSearch = {
  searchMakoTrading: async (query: string): Promise<MakoTradingResult[]> => {
    const response = await api.get('/pad-search/mako-trading', { params: { q: query } });
    return response.data;
  },

  searchPATrading: async (query: string): Promise<PATradingResult[]> => {
    const response = await api.get('/pad-search/pa-trading', { params: { q: query } });
    return response.data;
  },
};
```

---

## Non-Functional Requirements

### NFR-1: Performance
- Parallelize backend requests for the two panels
- Debounce search input (300ms delay)
- Backend queries should complete within 2 seconds
- Add database indexes on search fields if missing

### NFR-2: Styling
- Adhere to MAKO Brand Guidelines
- Colors: Navy (#0E1E3F), Blue (#5471DF), Gold (#B28C54)
- Typography: Montserrat font family
- Tables: Use existing `Table.tsx` component patterns

### NFR-3: Authorization
- All endpoints require `compliance` or `admin` role
- Unauthorized users see 403 error

### NFR-4: Responsiveness
- Minimum viewport width: 1024px
- Graceful degradation on smaller screens (stack panels vertically)

---

## Acceptance Criteria

### AC-1: Navigation
- [ ] "PAD Search" appears as a top-level navigation item
- [ ] Page loads without errors

### AC-2: Search Functionality
- [ ] Searching for a ticker (e.g., "AAPL") updates both tables simultaneously
- [ ] Search is case-insensitive
- [ ] Partial matches work (e.g., "APP" matches "APPLE INC")
- [ ] Empty search clears both tables

### AC-3: Mako Trading Panel
- [ ] Correctly resolves symbols via 3-tier lookup
- [ ] Displays company, portfolio, desk, symbol, description, type, dates
- [ ] Shows all historical data (not date-filtered)

### AC-4: PA Trading Panel
- [ ] Only displays requests with status = 'approved'
- [ ] Shows division, name, symbol, description, is_derivative, last traded
- [ ] Employee names display correctly (forename + surname)

### AC-5: Visual Highlighting
- [ ] Dates within last 30 days are bold and gold (#B28C54)
- [ ] Highlighting applies to both panels

### AC-6: Split View
- [ ] Resizer allows adjusting panel widths
- [ ] Resize preference persists across page refreshes
- [ ] Layout doesn't break at edge cases (90/10 split)

### AC-7: Authorization
- [ ] Unauthenticated users cannot access the page
- [ ] Non-compliance users get 403 error on API calls

---

## Out of Scope

1. **Pagination**: Initial implementation shows all results (add if performance issues)
2. **Export to CSV**: Not included in v1
3. **Saved Searches**: Not included in v1
4. **Real-time updates**: No WebSocket push for new trades
5. **Advanced filters**: Beyond search term (date range, status, etc.)

---

## Dependencies

**Required Data Sources** (already available):
- `bo_airflow.oracle_bloomberg` - Symbol lookup tier 1
- `bo_airflow.oracle_map_inst_symbol` - Symbol lookup tier 2
- `bo_airflow.oracle_product` - Symbol lookup tier 3 + descriptions
- `bo_airflow.oracle_product_usage` - Mako trading data
- `bo_airflow.oracle_portfolio_group` - Portfolio to desk mapping
- `bo_airflow.oracle_cost_centre` - Desk names
- `public.pad_request` - Employee PAD requests
- `bo_airflow.oracle_employee` - Employee details
- `bo_airflow.oracle_division` - Division names

---

## Reference Implementation

The legacy system implementation can be found at:
- **Backend (API/Filters)**: `compliance_portal/pages/pa_dealing/rest.py`, `filters.py`
- **Frontend (UI)**: `compliance_portal/templates/pa_dealing/search/searchTables.js`, `searchFunctions.js`
- **Serializers**: `compliance_portal/pages/pa_dealing/serializers.py`
- **Models**: `database_models/models/tables/table_models.py`

Key architectural patterns from legacy:
1. **Annotate & Subquery**: Use subqueries instead of massive JOINs
2. **Parallel API Calls**: Frontend fires both searches simultaneously
3. **Rosetta Stone Strategy**: Keep transaction tables lean, use mapping tables for text search
