# Plan: PAD Search (Conflict View) - Detailed Implementation

## Track Summary
Implement a side-by-side conflict search view showing Mako institutional trading vs Employee PA trades, with 3-tier symbol lookup and 30-day risk highlighting.

**Branch**: DSS-4074
**Cannot run tests**: Working from git bundle in isolated environment

---

## Architecture Overview

### Data Flow

```
User Search Input ("AAPL")
         ↓
┌────────────────────────────────────────────────────────────────┐
│                   3-TIER SYMBOL LOOKUP                         │
├────────────────────────────────────────────────────────────────┤
│ Tier 1: OracleBloomberg (isin, sedol, ticker, description)    │
│    └→ Returns inst_symbol list ['AAPL UW', 'AAPL UA']         │
│                                                                 │
│ Tier 2: OracleMapInstSymbol (exch_symbol)                      │
│    └→ Only searched if Tier 1 returns empty                    │
│                                                                 │
│ Tier 3: OracleProduct (description)                            │
│    └→ Only searched if Tier 1 & 2 return empty                 │
└────────────────────────────────────────────────────────────────┘
         ↓
    inst_symbols
         ↓
┌────────────────────────┐     ┌────────────────────────┐
│   LEFT PANEL (Mako)    │     │   RIGHT PANEL (PA)     │
├────────────────────────┤     ├────────────────────────┤
│ ProductUsage           │     │ PADRequest             │
│ + OracleProduct (desc) │     │ + OracleEmployee       │
│ + OraclePortfolioGroup │     │ + OracleDivision       │
│ + OracleCostCentre     │     │                        │
│   (desk display_name)  │     │ WHERE status='approved'│
└────────────────────────┘     └────────────────────────┘
```

### Key Tables (from `src/pa_dealing/db/models/market.py`)

| Table | Schema | Purpose |
|-------|--------|---------|
| `oracle_bloomberg` | bo_airflow | Tier 1 lookup (ISIN, SEDOL, ticker, bloomberg, inst_symbol) |
| `oracle_map_inst_symbol` | bo_airflow | Tier 2 lookup (exch_symbol → inst_symbol) |
| `oracle_product` | bo_airflow | Tier 3 lookup + product descriptions |
| `oracle_product_usage` | bo_airflow | Mako trading activity (company, portfolio_id, last_trade_date) |
| `oracle_portfolio_group` | bo_airflow | Portfolio → rp_portfolio_id mapping |
| `oracle_portfolio_meta_data` | bo_airflow | Cost centre info |
| `oracle_cost_centre` | bo_airflow | Desk display_name (GOLD STANDARD) |
| `pad_request` | padealing | Employee PAD requests |
| `oracle_employee` | bo_airflow | Employee info + division_id |
| `oracle_division` | bo_airflow | Division descriptions |

---

## Phase 1: Backend Service Implementation

### Task 1.1: Create PAD Search Service

**File**: `src/pa_dealing/services/pad_search.py`

```python
"""PAD Search service for conflict view functionality."""

from datetime import datetime, timedelta

from sqlalchemy import or_, select, func
from sqlalchemy.ext.asyncio import AsyncSession

from pa_dealing.db import get_session
from pa_dealing.db.models.market import (
    OracleBloomberg,
    OracleMapInstSymbol,
    OracleProduct,
    ProductUsage,
    OraclePortfolioGroup,
    OraclePortfolioMetaData,
    OracleCostCentre,
)
from pa_dealing.db.models.core import PADRequest
from pa_dealing.db.models.identity import OracleEmployee, OracleDivision


class PADSearchService:
    """Service for PAD Search conflict detection functionality."""

    async def resolve_symbols_waterfall(
        self, session: AsyncSession, query: str
    ) -> list[str]:
        """
        3-tier waterfall to resolve search query to inst_symbols.

        Uses icontains (LIKE %value%) for fuzzy matching.
        Short-circuits: if Tier N returns results, skip lower tiers.
        """
        query_pattern = f"%{query}%"

        # Tier 1: Bloomberg (ISIN, SEDOL, ticker, description)
        stmt = (
            select(OracleBloomberg.inst_symbol)
            .where(
                or_(
                    OracleBloomberg.isin.ilike(query_pattern),
                    OracleBloomberg.sedol.ilike(query_pattern),
                    OracleBloomberg.ticker.ilike(query_pattern),
                    OracleBloomberg.bloomberg.ilike(query_pattern),
                    OracleBloomberg.description.ilike(query_pattern),
                )
            )
            .where(OracleBloomberg.inst_symbol.isnot(None))
            .distinct()
            .limit(100)  # Prevent runaway queries
        )
        result = await session.execute(stmt)
        symbols = [r[0] for r in result.fetchall() if r[0]]
        if symbols:
            return symbols

        # Tier 2: MapInstSymbol (exch_symbol)
        stmt = (
            select(OracleMapInstSymbol.inst_symbol)
            .where(OracleMapInstSymbol.exch_symbol.ilike(query_pattern))
            .distinct()
            .limit(100)
        )
        result = await session.execute(stmt)
        symbols = [r[0] for r in result.fetchall() if r[0]]
        if symbols:
            return symbols

        # Tier 3: Product (description fallback)
        stmt = (
            select(OracleProduct.inst_symbol)
            .where(OracleProduct.description.ilike(query_pattern))
            .distinct()
            .limit(100)
        )
        result = await session.execute(stmt)
        return [r[0] for r in result.fetchall() if r[0]]

    async def search_mako_trading(
        self, session: AsyncSession, query: str
    ) -> list[dict]:
        """
        Search Mako institutional trading activity.

        Returns ProductUsage records with:
        - Description from OracleProduct (subquery)
        - Desk name from join chain: PortfolioGroup → PortfolioMetaData → CostCentre
        """
        # Step 1: Resolve query to inst_symbols
        symbols = await self.resolve_symbols_waterfall(session, query)
        if not symbols:
            return []

        # Step 2: Fetch from ProductUsage with annotations
        # Subquery for description from OracleProduct
        desc_subq = (
            select(OracleProduct.description)
            .where(OracleProduct.inst_symbol == ProductUsage.inst_symbol)
            .correlate(ProductUsage)
            .scalar_subquery()
        )

        # Subquery for desk name (complex join chain)
        # ProductUsage.portfolio_id → PortfolioGroup → PortfolioMetaData → CostCentre
        desk_subq = (
            select(
                func.coalesce(
                    OracleCostCentre.display_name,
                    OraclePortfolioMetaData.pnl_cost_centre_name,
                    OraclePortfolioMetaData.desk,
                    OraclePortfolioGroup.rp_portfolio,
                )
            )
            .select_from(OraclePortfolioGroup)
            .outerjoin(
                OraclePortfolioMetaData,
                OraclePortfolioGroup.rp_portfolio_id == OraclePortfolioMetaData.id
            )
            .outerjoin(
                OracleCostCentre,
                OraclePortfolioMetaData.cost_centre == OracleCostCentre.cost_centre
            )
            .where(OraclePortfolioGroup.portfolio_id == ProductUsage.portfolio_id)
            .correlate(ProductUsage)
            .scalar_subquery()
        )

        # Portfolio name subquery
        portfolio_subq = (
            select(OraclePortfolioGroup.portfolio)
            .where(OraclePortfolioGroup.portfolio_id == ProductUsage.portfolio_id)
            .correlate(ProductUsage)
            .limit(1)
            .scalar_subquery()
        )

        stmt = (
            select(
                ProductUsage.id,
                ProductUsage.company,
                portfolio_subq.label("portfolio"),
                desk_subq.label("desk"),
                ProductUsage.inst_symbol,
                desc_subq.label("description"),
                ProductUsage.inst_type,
                ProductUsage.last_trade_date,
                ProductUsage.last_position_date,
            )
            .where(ProductUsage.inst_symbol.in_(symbols))
            .order_by(ProductUsage.last_trade_date.desc().nullslast())
            .limit(500)  # Reasonable limit
        )

        result = await session.execute(stmt)
        return [
            {
                "id": r.id,
                "company": r.company,
                "portfolio": r.portfolio,
                "desk": r.desk,
                "inst_symbol": r.inst_symbol,
                "description": r.description,
                "inst_type": r.inst_type,
                "last_trade_date": r.last_trade_date.isoformat() if r.last_trade_date else None,
                "last_position_date": r.last_position_date.isoformat() if r.last_position_date else None,
            }
            for r in result.fetchall()
        ]

    async def search_pa_trading(
        self, session: AsyncSession, query: str
    ) -> list[dict]:
        """
        Search approved employee PA trades.

        Searches across: inst_symbol, isin, security_description, employee name
        Only returns status='approved' requests.
        """
        query_pattern = f"%{query}%"

        # Division name subquery
        division_subq = (
            select(OracleDivision.description)
            .select_from(OracleEmployee)
            .outerjoin(OracleDivision, OracleEmployee.division_id == OracleDivision.id)
            .where(OracleEmployee.id == PADRequest.employee_id)
            .correlate(PADRequest)
            .scalar_subquery()
        )

        # Employee name subquery
        emp_name_subq = (
            select(OracleEmployee.mako_id)
            .where(OracleEmployee.id == PADRequest.employee_id)
            .correlate(PADRequest)
            .scalar_subquery()
        )

        stmt = (
            select(
                PADRequest.id,
                PADRequest.reference_id,
                division_subq.label("division"),
                emp_name_subq.label("employee_name"),
                PADRequest.inst_symbol,
                PADRequest.security_name.label("security_description"),
                PADRequest.is_derivative,
                PADRequest.updated_at.label("approved_at"),  # Use updated_at as approval date
            )
            .where(PADRequest.status == "approved")
            .where(
                or_(
                    PADRequest.inst_symbol.ilike(query_pattern),
                    PADRequest.isin.ilike(query_pattern),
                    PADRequest.security_name.ilike(query_pattern),
                    PADRequest.bloomberg_ticker.ilike(query_pattern),
                )
            )
            .order_by(PADRequest.updated_at.desc().nullslast())
            .limit(500)
        )

        result = await session.execute(stmt)
        return [
            {
                "id": r.id,
                "reference_id": r.reference_id,
                "division": r.division,
                "employee_name": r.employee_name,
                "inst_symbol": r.inst_symbol,
                "security_description": r.security_description,
                "is_derivative": r.is_derivative,
                "approved_at": r.approved_at.isoformat() if r.approved_at else None,
            }
            for r in result.fetchall()
        ]


# Singleton
_pad_search_service: PADSearchService | None = None


def get_pad_search_service() -> PADSearchService:
    global _pad_search_service
    if _pad_search_service is None:
        _pad_search_service = PADSearchService()
    return _pad_search_service
```

### Task 1.2: Create API Routes

**File**: `src/pa_dealing/api/routes/pad_search.py`

```python
"""PAD Search API routes for conflict view."""

from fastapi import APIRouter, HTTPException, Query
from starlette.status import HTTP_403_FORBIDDEN

from pa_dealing.api.dependencies import CurrentUserDep
from pa_dealing.api.schemas import APIResponse
from pa_dealing.db import get_session
from pa_dealing.services.pad_search import get_pad_search_service

router = APIRouter(prefix="/pad-search", tags=["pad-search"])


def _require_compliance_or_admin(user) -> None:
    """Ensure user has compliance or admin role."""
    if not user.is_compliance and not user.is_admin:
        raise HTTPException(
            status_code=HTTP_403_FORBIDDEN,
            detail="Compliance or admin role required"
        )


@router.get("/mako-trading", response_model=APIResponse)
async def search_mako_trading(
    user: CurrentUserDep,
    q: str = Query(..., min_length=2, description="Search query (ticker, ISIN, name)"),
):
    """
    Search Mako institutional trading activity.

    Uses 3-tier symbol lookup: Bloomberg → MapInstSymbol → Product
    Returns trading data from ProductUsage with desk names resolved.
    """
    _require_compliance_or_admin(user)

    service = get_pad_search_service()
    async with get_session() as session:
        results = await service.search_mako_trading(session, q)
        return APIResponse(data=results, message=f"Found {len(results)} Mako trading records")


@router.get("/pa-trading", response_model=APIResponse)
async def search_pa_trading(
    user: CurrentUserDep,
    q: str = Query(..., min_length=2, description="Search query (ticker, ISIN, name)"),
):
    """
    Search approved employee PA trades.

    Searches across inst_symbol, isin, security_description.
    Only returns approved requests.
    """
    _require_compliance_or_admin(user)

    service = get_pad_search_service()
    async with get_session() as session:
        results = await service.search_pa_trading(session, q)
        return APIResponse(data=results, message=f"Found {len(results)} PA trading records")
```

### Task 1.3: Register Routes

**File**: `src/pa_dealing/api/routes/__init__.py`

Add to imports:
```python
from .pad_search import router as pad_search_router
```

Add to `__all__`:
```python
__all__ = [
    # ... existing
    "pad_search_router",
]
```

**File**: `src/pa_dealing/api/main.py`

Add import and registration:
```python
from .routes import (
    # ... existing
    pad_search_router,
)

# In create_app():
app.include_router(pad_search_router)
```

---

## Phase 2: Frontend - Types & API Client

### Task 2.1: Add TypeScript Types

**File**: `dashboard/src/types/index.ts`

Add interfaces:
```typescript
// PAD Search types
export interface MakoTradingResult {
  id: number
  company: string | null
  portfolio: string | null
  desk: string | null
  inst_symbol: string
  description: string | null
  inst_type: string | null
  last_trade_date: string | null
  last_position_date: string | null
}

export interface PATradingResult {
  id: number
  reference_id: string | null
  division: string | null
  employee_name: string
  inst_symbol: string | null
  security_description: string | null
  is_derivative: boolean
  approved_at: string | null
}
```

### Task 2.2: Add API Client Methods

**File**: `dashboard/src/api/client.ts`

Add to exports:
```typescript
export const padSearch = {
  searchMakoTrading: async (query: string): Promise<MakoTradingResult[]> => {
    const response = await api.get('/pad-search/mako-trading', { params: { q: query } })
    return response.data
  },

  searchPATrading: async (query: string): Promise<PATradingResult[]> => {
    const response = await api.get('/pad-search/pa-trading', { params: { q: query } })
    return response.data
  },
}
```

---

## Phase 3: Frontend - Page Component

### Task 3.1: Create PADSearch Page

**File**: `dashboard/src/pages/PADSearch.tsx`

```tsx
import { useState, useEffect, useRef } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Search, GripVertical } from 'lucide-react'
import { padSearch } from '@/api/client'
import Table from '@/components/ui/Table'
import Card from '@/components/ui/Card'
import type { MakoTradingResult, PATradingResult } from '@/types'

// Check if date is within last 30 days
function isWithin30Days(dateStr: string | null): boolean {
  if (!dateStr) return false
  const date = new Date(dateStr)
  const thirtyDaysAgo = new Date()
  thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30)
  return date >= thirtyDaysAgo
}

// Format date for display
function formatDate(dateStr: string | null): string {
  if (!dateStr) return '-'
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  })
}

// Date cell with 30-day highlighting
function DateCell({ value }: { value: string | null }) {
  const isRecent = isWithin30Days(value)
  return (
    <span className={isRecent ? 'font-bold text-[#B28C54]' : 'text-[#0E1E3F]'}>
      {formatDate(value)}
    </span>
  )
}

export default function PADSearch() {
  const [query, setQuery] = useState('')
  const [debouncedQuery, setDebouncedQuery] = useState('')
  const [splitRatio, setSplitRatio] = useState(() => {
    const saved = localStorage.getItem('pad-search-split')
    return saved ? parseFloat(saved) : 0.5
  })
  const [isDragging, setIsDragging] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  // Debounce search input (300ms)
  useEffect(() => {
    const timer = setTimeout(() => {
      if (query.length >= 2) {
        setDebouncedQuery(query)
      } else {
        setDebouncedQuery('')
      }
    }, 300)
    return () => clearTimeout(timer)
  }, [query])

  // Parallel queries for both panels
  const { data: makoResults, isLoading: makoLoading } = useQuery({
    queryKey: ['pad-search-mako', debouncedQuery],
    queryFn: () => padSearch.searchMakoTrading(debouncedQuery),
    enabled: debouncedQuery.length >= 2,
  })

  const { data: paResults, isLoading: paLoading } = useQuery({
    queryKey: ['pad-search-pa', debouncedQuery],
    queryFn: () => padSearch.searchPATrading(debouncedQuery),
    enabled: debouncedQuery.length >= 2,
  })

  // Resizer drag handling
  const handleMouseDown = () => setIsDragging(true)

  useEffect(() => {
    if (!isDragging) return

    const handleMouseMove = (e: MouseEvent) => {
      if (!containerRef.current) return
      const rect = containerRef.current.getBoundingClientRect()
      const newRatio = (e.clientX - rect.left) / rect.width
      const clampedRatio = Math.max(0.2, Math.min(0.8, newRatio))
      setSplitRatio(clampedRatio)
    }

    const handleMouseUp = () => {
      setIsDragging(false)
      localStorage.setItem('pad-search-split', splitRatio.toString())
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [isDragging, splitRatio])

  // Mako Trading columns
  const makoColumns = [
    {
      header: 'Company',
      accessor: (row: MakoTradingResult) => (
        <span className="font-bold text-[#0E1E3F]">{row.company || '-'}</span>
      ),
    },
    {
      header: 'Portfolio',
      accessor: (row: MakoTradingResult) => row.portfolio || '-',
    },
    {
      header: 'Desk',
      accessor: (row: MakoTradingResult) => row.desk || '-',
    },
    {
      header: 'Symbol',
      accessor: (row: MakoTradingResult) => (
        <span className="font-mono font-bold text-[#5471DF]">{row.inst_symbol}</span>
      ),
    },
    {
      header: 'Description',
      accessor: (row: MakoTradingResult) => (
        <span className="text-[11px] text-slate-500 truncate max-w-[120px] block">
          {row.description || '-'}
        </span>
      ),
    },
    {
      header: 'Type',
      accessor: (row: MakoTradingResult) => row.inst_type || '-',
    },
    {
      header: 'Last Traded',
      accessor: (row: MakoTradingResult) => <DateCell value={row.last_trade_date} />,
    },
    {
      header: 'Position Date',
      accessor: (row: MakoTradingResult) => <DateCell value={row.last_position_date} />,
    },
  ]

  // PA Trading columns
  const paColumns = [
    {
      header: 'Division',
      accessor: (row: PATradingResult) => row.division || '-',
    },
    {
      header: 'Employee',
      accessor: (row: PATradingResult) => (
        <span className="font-bold text-[#0E1E3F]">{row.employee_name}</span>
      ),
    },
    {
      header: 'Symbol',
      accessor: (row: PATradingResult) => (
        <span className="font-mono font-bold text-[#5471DF]">{row.inst_symbol || '-'}</span>
      ),
    },
    {
      header: 'Description',
      accessor: (row: PATradingResult) => (
        <span className="text-[11px] text-slate-500 truncate max-w-[120px] block">
          {row.security_description || '-'}
        </span>
      ),
    },
    {
      header: 'Derivative',
      accessor: (row: PATradingResult) => (
        <span className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase ${
          row.is_derivative
            ? 'bg-amber-100 text-amber-700'
            : 'bg-[#DBE1F5] text-slate-500'
        }`}>
          {row.is_derivative ? 'Yes' : 'No'}
        </span>
      ),
    },
    {
      header: 'Approved',
      accessor: (row: PATradingResult) => <DateCell value={row.approved_at} />,
    },
  ]

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="page-title">PAD Search</h1>
        <p className="text-slate-500 font-medium">
          Cross-reference employee trades with Mako trading activity
        </p>
      </div>

      {/* Search Bar */}
      <Card noPadding className="bg-white/50">
        <div className="p-4">
          <div className="relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search by ticker, ISIN, SEDOL, or description..."
              className="input pl-12 w-full text-lg"
              autoFocus
            />
          </div>
          {query.length > 0 && query.length < 2 && (
            <p className="text-xs text-slate-400 mt-2">Enter at least 2 characters to search</p>
          )}
        </div>
      </Card>

      {/* Split View */}
      <div
        ref={containerRef}
        className="flex gap-0 min-h-[600px]"
        style={{ cursor: isDragging ? 'col-resize' : 'default' }}
      >
        {/* Left Panel: Mako Trading */}
        <div style={{ width: `${splitRatio * 100}%` }} className="flex flex-col">
          <Card noPadding className="flex-1 flex flex-col">
            <div className="px-4 py-3 bg-[#0E1E3F] text-white rounded-t-xl">
              <h2 className="font-bold">Mako Trading</h2>
              <p className="text-xs text-white/60">Institutional trading activity</p>
            </div>
            <div className="flex-1 overflow-auto">
              <Table
                data={makoResults || []}
                columns={makoColumns}
                keyExtractor={(row) => String(row.id)}
                isLoading={makoLoading && debouncedQuery.length >= 2}
                emptyMessage={
                  debouncedQuery.length < 2
                    ? 'Enter a search term to find Mako trades'
                    : 'No Mako trading activity found'
                }
              />
            </div>
          </Card>
        </div>

        {/* Resizer */}
        <div
          className="w-2 bg-slate-200 hover:bg-[#5471DF] cursor-col-resize flex items-center justify-center transition-colors"
          onMouseDown={handleMouseDown}
        >
          <GripVertical className="w-4 h-4 text-slate-400" />
        </div>

        {/* Right Panel: PA Trading */}
        <div style={{ width: `${(1 - splitRatio) * 100}%` }} className="flex flex-col">
          <Card noPadding className="flex-1 flex flex-col">
            <div className="px-4 py-3 bg-[#5471DF] text-white rounded-t-xl">
              <h2 className="font-bold">PA Account Trading</h2>
              <p className="text-xs text-white/60">Approved employee trades</p>
            </div>
            <div className="flex-1 overflow-auto">
              <Table
                data={paResults || []}
                columns={paColumns}
                keyExtractor={(row) => String(row.id)}
                isLoading={paLoading && debouncedQuery.length >= 2}
                emptyMessage={
                  debouncedQuery.length < 2
                    ? 'Enter a search term to find PA trades'
                    : 'No approved PA trades found'
                }
              />
            </div>
          </Card>
        </div>
      </div>
    </div>
  )
}
```

---

## Phase 4: Frontend - Navigation & Routing

### Task 4.1: Add Route

**File**: `dashboard/src/App.tsx`

Add import:
```tsx
import PADSearch from '@/pages/PADSearch'
```

Add route (inside ProtectedRoute):
```tsx
<Route path="/pad-search" element={<PADSearch />} />
```

### Task 4.2: Add Sidebar Navigation

**File**: `dashboard/src/components/Sidebar.tsx`

Add to navigation items (after existing items, before Settings):
```tsx
{
  path: '/pad-search',
  label: 'PAD Search',
  icon: Search,
  accessLevel: 'compliance', // Or 'all' if managers should see it
}
```

---

## Implementation Checklist

### Backend
- [ ] Create `src/pa_dealing/services/pad_search.py`
  - [ ] `resolve_symbols_waterfall()` - 3-tier lookup
  - [ ] `search_mako_trading()` - ProductUsage with desk resolution
  - [ ] `search_pa_trading()` - Approved PAD requests
- [ ] Create `src/pa_dealing/api/routes/pad_search.py`
  - [ ] GET `/pad-search/mako-trading`
  - [ ] GET `/pad-search/pa-trading`
- [ ] Register routes in `__init__.py` and `main.py`

### Frontend
- [ ] Add types to `types/index.ts`
- [ ] Add API methods to `api/client.ts`
- [ ] Create `pages/PADSearch.tsx`
  - [ ] Debounced search input
  - [ ] Parallel React Query calls
  - [ ] Resizable split view
  - [ ] 30-day highlighting with MAKO Gold
  - [ ] Loading/empty states
- [ ] Add route to `App.tsx`
- [ ] Add nav item to `Sidebar.tsx`

### Styling
- [ ] MAKO Navy (#0E1E3F) for panel headers
- [ ] MAKO Blue (#5471DF) for symbols and right panel
- [ ] MAKO Gold (#B28C54) for 30-day highlighting
- [ ] Montserrat font throughout

---

## Key Architectural Decisions

1. **Parallel API Calls**: Frontend fires both searches simultaneously for perceived speed
2. **Subqueries over JOINs**: Use scalar subqueries for desk names to avoid N+1 and massive joins
3. **Short-circuit Waterfall**: If Tier 1 returns results, skip Tier 2 and 3
4. **Fuzzy Matching**: Use `ilike` (LIKE %value%) for forgiving searches
5. **Client-side Highlighting**: Calculate 30-day window in frontend to keep backend simple
6. **LocalStorage Persistence**: Save split ratio preference
7. **Result Limits**: Cap at 500 results per panel to prevent UI overload

---

## Notes for Implementation

Since we cannot run tests in this isolated environment:

1. **Verify imports**: Ensure all model imports match actual file locations
2. **Check field names**: Confirm `security_name` vs `security_description` on PADRequest
3. **Division lookup**: Verify OracleDivision model exists and has `description` field
4. **Test query syntax**: The subquery approach may need adjustment based on actual SQLAlchemy behavior
5. **API response unwrapping**: Frontend client already handles `{success, data}` unwrapping
