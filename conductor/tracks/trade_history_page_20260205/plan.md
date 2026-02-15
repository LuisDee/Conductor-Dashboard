# Trade History Dashboard Page - Implementation Plan

## Phase 1: Backend API

### 1.1 Add Response Schema

**File**: `src/pa_dealing/api/schemas.py`

Add new Pydantic model:

```python
class TradeHistoryItem(BaseModel):
    """Executed trade with contract note verification status."""

    request_id: int
    reference_id: str | None
    employee_id: int
    employee_name: str
    employee_email: str
    security_identifier: str
    security_name: str | None
    isin: str | None
    direction: str

    # Original request values
    requested_quantity: int
    estimated_price: Decimal | None
    currency: str | None

    # Execution values
    execution_quantity: int
    execution_price: Decimal
    executed_at: datetime
    recorded_by_name: str | None

    # Contract note status
    contract_note_id: int | None
    contract_note_filename: str | None
    verification_status: Literal["verified", "pending", "mismatched", "missing"]
    verification_metadata: dict | None

    # For PDF access
    gcs_document_id: str | None

    model_config = ConfigDict(from_attributes=True)


class TradeHistoryResponse(BaseModel):
    """Paginated trade history response."""

    items: list[TradeHistoryItem]
    total: int
    page: int
    page_size: int
    has_more: bool
```

### 1.2 Add Repository Method

**File**: `src/pa_dealing/db/repository.py`

Add method to `PADRepository`:

```python
async def get_trade_history(
    self,
    *,
    executed_from: date | None = None,
    executed_to: date | None = None,
    employee_name: str | None = None,
    security: str | None = None,
    verification_status: str | None = None,
    page: int = 1,
    page_size: int = 50,
) -> tuple[list[dict], int]:
    """Get executed trades with contract note verification status."""

    # Build query joining PADRequest, PADExecution, ContractNoteUpload
    query = (
        select(
            PADRequest.id.label("request_id"),
            PADRequest.reference_id,
            PADRequest.employee_id,
            PADRequest.employee_name,
            PADRequest.employee_email,
            PADRequest.security_identifier,
            PADRequest.security_name,
            PADRequest.isin,
            PADRequest.direction,
            PADRequest.quantity.label("requested_quantity"),
            PADRequest.estimated_price,
            PADRequest.currency,
            PADExecution.execution_quantity,
            PADExecution.execution_price,
            PADExecution.executed_at,
            OracleEmployee.name.label("recorded_by_name"),
            ContractNoteUpload.id.label("contract_note_id"),
            ContractNoteUpload.original_filename.label("contract_note_filename"),
            ContractNoteUpload.verification_status,
            ContractNoteUpload.verification_metadata,
            func.cast(ContractNoteUpload.gcs_document_id, String).label("gcs_document_id"),
        )
        .select_from(PADRequest)
        .join(PADExecution, PADExecution.request_id == PADRequest.id)
        .outerjoin(
            ContractNoteUpload,
            and_(
                ContractNoteUpload.request_id == PADRequest.id,
                ContractNoteUpload.is_active == True,
            ),
        )
        .outerjoin(OracleEmployee, OracleEmployee.id == PADExecution.recorded_by_id)
        .where(PADRequest.status == "executed")
    )

    # Apply filters
    if executed_from:
        query = query.where(func.date(PADExecution.executed_at) >= executed_from)
    if executed_to:
        query = query.where(func.date(PADExecution.executed_at) <= executed_to)
    if employee_name:
        query = query.where(PADRequest.employee_name.ilike(f"%{employee_name}%"))
    if security:
        query = query.where(
            or_(
                PADRequest.security_identifier.ilike(f"%{security}%"),
                PADRequest.isin.ilike(f"%{security}%"),
                PADRequest.security_name.ilike(f"%{security}%"),
            )
        )
    if verification_status:
        if verification_status == "missing":
            query = query.where(ContractNoteUpload.id.is_(None))
        else:
            query = query.where(ContractNoteUpload.verification_status == verification_status)

    # Count total
    count_query = select(func.count()).select_from(query.subquery())
    total = await self.session.scalar(count_query) or 0

    # Apply pagination and ordering
    query = query.order_by(PADExecution.executed_at.desc())
    query = query.offset((page - 1) * page_size).limit(page_size)

    result = await self.session.execute(query)
    rows = result.mappings().all()

    # Transform rows, handling missing contract notes
    items = []
    for row in rows:
        item = dict(row)
        if item["contract_note_id"] is None:
            item["verification_status"] = "missing"
        items.append(item)

    return items, total
```

### 1.3 Add API Endpoint

**File**: `src/pa_dealing/api/routes/dashboard.py`

Add new endpoint:

```python
@router.get("/trade-history", response_model=APIResponse)
async def get_trade_history(
    service: PADServiceDep,
    user: CurrentUserDep,
    executed_from: str | None = Query(None, description="Filter from date (YYYY-MM-DD)"),
    executed_to: str | None = Query(None, description="Filter to date (YYYY-MM-DD)"),
    employee_name: str | None = Query(None, description="Filter by employee name"),
    security: str | None = Query(None, description="Filter by ticker/ISIN/security name"),
    verification_status: str | None = Query(
        None,
        description="Filter by verification status",
        enum=["verified", "pending", "mismatched", "missing"],
    ),
    page: int = Query(1, ge=1, description="Page number"),
    page_size: int = Query(50, ge=1, le=100, description="Items per page"),
):
    """Get historical record of executed trades with contract note verification status."""
    _require_compliance_or_admin(user)

    # Parse dates
    from_date = datetime.strptime(executed_from, "%Y-%m-%d").date() if executed_from else None
    to_date = datetime.strptime(executed_to, "%Y-%m-%d").date() if executed_to else None

    items, total = await service.repository.get_trade_history(
        executed_from=from_date,
        executed_to=to_date,
        employee_name=employee_name,
        security=security,
        verification_status=verification_status,
        page=page,
        page_size=page_size,
    )

    return APIResponse(
        data={
            "items": items,
            "total": total,
            "page": page,
            "page_size": page_size,
            "has_more": (page * page_size) < total,
        }
    )
```

---

## Phase 2: Frontend Types & API Client

### 2.1 Add TypeScript Types

**File**: `dashboard/src/types/index.ts`

Add after `ExecutionTracking` interface:

```typescript
export interface TradeHistoryItem {
  request_id: number;
  reference_id: string | null;
  employee_id: number;
  employee_name: string;
  employee_email: string;
  security_identifier: string;
  security_name: string | null;
  isin: string | null;
  direction: string;

  // Original request values
  requested_quantity: number;
  estimated_price: number | null;
  currency: string | null;

  // Execution values
  execution_quantity: number;
  execution_price: number;
  executed_at: string;
  recorded_by_name: string | null;

  // Contract note status
  contract_note_id: number | null;
  contract_note_filename: string | null;
  verification_status: 'verified' | 'pending' | 'mismatched' | 'missing';
  verification_metadata: {
    quantity_variance?: number;
    price_variance?: number;
    date_variance_days?: number;
    direction_match?: boolean;
  } | null;

  // For PDF access
  gcs_document_id: string | null;
}

export interface TradeHistoryResponse {
  items: TradeHistoryItem[];
  total: number;
  page: number;
  page_size: number;
  has_more: boolean;
}
```

### 2.2 Add API Client Method

**File**: `dashboard/src/api/client.ts`

Add to `dashboard` export object:

```typescript
getTradeHistory: async (params?: {
  executed_from?: string;
  executed_to?: string;
  employee_name?: string;
  security?: string;
  verification_status?: string;
  page?: number;
  page_size?: number;
}): Promise<TradeHistoryResponse> => {
  try {
    const response = await api.get<TradeHistoryResponse>('/dashboard/trade-history', {
      params: cleanParams(params as Record<string, unknown>),
    });
    return response.data;
  } catch (error) {
    return handleError(error as AxiosError);
  }
},
```

Also add method for getting contract note PDF URL (reuse documents endpoint):

```typescript
getContractNotePdf: async (gcsDocumentId: string): Promise<{ url: string; expires_at: string }> => {
  try {
    const response = await api.get<{ url: string; expires_at: string }>(`/documents/${gcsDocumentId}/pdf`);
    return response.data;
  } catch (error) {
    return handleError(error as AxiosError);
  }
},
```

---

## Phase 3: Trade History Page Component

### 3.1 Create Page Component

**File**: `dashboard/src/pages/TradeHistory.tsx`

```typescript
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import {
  Search, FileText, Calendar, ChevronDown, ChevronUp,
  AlertTriangle, CheckCircle2, Clock, FileWarning, Eye, ExternalLink
} from 'lucide-react'
import { dashboard, pdfHistory } from '@/api/client'
import Table from '@/components/ui/Table'
import Card from '@/components/ui/Card'
import StatusBadge from '@/components/ui/StatusBadge'
import type { TradeHistoryItem } from '@/types'

// Format helpers (reuse from other pages)
function formatDate(dateStr: string | null): string { ... }
function formatCurrency(value: number | null, currency: string | null): string { ... }

// Verification status badge
function VerificationBadge({ status }: { status: TradeHistoryItem['verification_status'] }) {
  const config = {
    verified: { text: 'Verified', color: 'green' },
    pending: { text: 'Pending', color: 'blue' },
    mismatched: { text: 'Variance', color: 'gold' },
    missing: { text: 'Missing', color: 'gray' },
  }
  const { text, color } = config[status]
  return <StatusBadge status={text} variant={color} />
}

// Expandable row for contract note preview
function ContractNotePreview({ item }: { item: TradeHistoryItem }) {
  const [pdfUrl, setPdfUrl] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (!item.gcs_document_id) return
    setLoading(true)
    pdfHistory.getPdfUrl(item.gcs_document_id)
      .then(res => setPdfUrl(res.url))
      .finally(() => setLoading(false))
  }, [item.gcs_document_id])

  if (!item.gcs_document_id) {
    return (
      <div className="p-4 bg-slate-50 text-slate-500 text-sm">
        No contract note uploaded for this trade.
      </div>
    )
  }

  return (
    <div className="p-4 bg-slate-50 border-t">
      {/* Variance indicators */}
      {item.verification_status === 'mismatched' && item.verification_metadata && (
        <div className="mb-3 p-3 bg-amber-50 border border-amber-200 rounded-[4px]">
          <div className="flex items-center gap-2 text-amber-700 font-bold text-sm mb-2">
            <AlertTriangle className="w-4 h-4" />
            Variance Detected
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs">
            {item.verification_metadata.quantity_variance !== undefined && (
              <div>Quantity: {item.verification_metadata.quantity_variance > 0 ? '+' : ''}{item.verification_metadata.quantity_variance}</div>
            )}
            {item.verification_metadata.price_variance !== undefined && (
              <div>Price: {item.verification_metadata.price_variance > 0 ? '+' : ''}{item.verification_metadata.price_variance.toFixed(2)}%</div>
            )}
            {item.verification_metadata.date_variance_days !== undefined && (
              <div>Date: {item.verification_metadata.date_variance_days} days</div>
            )}
          </div>
        </div>
      )}

      {/* PDF iframe */}
      {loading ? (
        <div className="h-[400px] flex items-center justify-center">
          <div className="animate-spin w-6 h-6 border-2 border-[#5471DF] border-t-transparent rounded-full" />
        </div>
      ) : pdfUrl ? (
        <div className="relative">
          <iframe src={pdfUrl} className="w-full h-[400px] border rounded-[4px]" />
          <a
            href={pdfUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="absolute top-2 right-2 px-2 py-1 bg-white/90 rounded-[4px] text-xs font-bold flex items-center gap-1 hover:bg-white"
          >
            <ExternalLink className="w-3 h-3" />
            Open
          </a>
        </div>
      ) : (
        <div className="h-[400px] flex items-center justify-center text-slate-500">
          Unable to load PDF
        </div>
      )}
    </div>
  )
}

export default function TradeHistory() {
  const [filters, setFilters] = useState({
    executed_from: '',
    executed_to: '',
    employee_name: '',
    security: '',
    verification_status: '',
  })
  const [page, setPage] = useState(1)
  const [expandedRow, setExpandedRow] = useState<number | null>(null)
  const pageSize = 50

  const { data, isLoading } = useQuery({
    queryKey: ['trade-history', filters, page],
    queryFn: () => dashboard.getTradeHistory({
      executed_from: filters.executed_from || undefined,
      executed_to: filters.executed_to || undefined,
      employee_name: filters.employee_name || undefined,
      security: filters.security || undefined,
      verification_status: filters.verification_status || undefined,
      page,
      page_size: pageSize,
    }),
  })

  const items = data?.items || []
  const total = data?.total || 0
  const hasMore = data?.has_more || false

  // Table columns
  const columns = [
    {
      header: 'Request',
      accessor: (row: TradeHistoryItem) => (
        <Link to={`/requests/${row.request_id}`} className="text-[#5471DF] font-bold hover:underline">
          #{row.request_id}
        </Link>
      ),
    },
    {
      header: 'Employee',
      accessor: (row: TradeHistoryItem) => (
        <div>
          <p className="font-medium text-[#0E1E3F]">{row.employee_name}</p>
          <p className="text-[10px] text-slate-400">{row.employee_email}</p>
        </div>
      ),
    },
    {
      header: 'Instrument',
      accessor: (row: TradeHistoryItem) => (
        <div>
          <p className="font-mono font-bold text-[#5471DF]">{row.security_identifier}</p>
          {row.security_name && <p className="text-[10px] text-slate-500 truncate max-w-[150px]">{row.security_name}</p>}
        </div>
      ),
    },
    {
      header: 'ISIN',
      accessor: (row: TradeHistoryItem) => (
        <span className="font-mono text-[11px]">{row.isin || '-'}</span>
      ),
    },
    {
      header: 'Direction',
      accessor: (row: TradeHistoryItem) => (
        <StatusBadge
          status={row.direction}
          variant={row.direction === 'BUY' ? 'green' : 'red'}
        />
      ),
    },
    {
      header: 'Quantity',
      accessor: (row: TradeHistoryItem) => (
        <span className="font-mono">{row.execution_quantity.toLocaleString()}</span>
      ),
    },
    {
      header: 'Price',
      accessor: (row: TradeHistoryItem) => (
        <span className="font-mono">{formatCurrency(row.execution_price, row.currency)}</span>
      ),
    },
    {
      header: 'Executed',
      accessor: (row: TradeHistoryItem) => (
        <span className="text-[11px] text-slate-600">{formatDate(row.executed_at)}</span>
      ),
    },
    {
      header: 'Contract Note',
      accessor: (row: TradeHistoryItem) => (
        <div className="flex items-center gap-2">
          <VerificationBadge status={row.verification_status} />
          {row.gcs_document_id && (
            <button
              onClick={(e) => {
                e.stopPropagation()
                setExpandedRow(expandedRow === row.request_id ? null : row.request_id)
              }}
              className="p-1 hover:bg-slate-100 rounded-[4px] text-slate-500 hover:text-[#5471DF]"
              title="View Contract Note"
            >
              {expandedRow === row.request_id ? (
                <ChevronUp className="w-4 h-4" />
              ) : (
                <ChevronDown className="w-4 h-4" />
              )}
            </button>
          )}
        </div>
      ),
    },
  ]

  // Render rows with expansion
  const renderRow = (row: TradeHistoryItem, rowContent: React.ReactNode) => (
    <>
      {rowContent}
      {expandedRow === row.request_id && (
        <tr>
          <td colSpan={columns.length}>
            <ContractNotePreview item={row} />
          </td>
        </tr>
      )}
    </>
  )

  return (
    <div className="space-y-4">
      {/* Header */}
      <div>
        <h1 className="page-title">Trade History</h1>
        <p className="text-slate-500 font-medium text-[13px]">
          Complete audit trail of executed trades with contract note verification
        </p>
      </div>

      {/* Filters */}
      <Card noPadding className="bg-white/50">
        <div className="px-4 py-3 grid grid-cols-1 md:grid-cols-5 gap-3">
          {/* Date Range */}
          <div className="flex gap-2">
            <input
              type="date"
              value={filters.executed_from}
              onChange={(e) => { setFilters(f => ({ ...f, executed_from: e.target.value })); setPage(1); }}
              className="flex-1 px-2 py-2 text-[13px] rounded-[4px] border border-[rgba(14,30,63,0.2)] focus:outline-none focus:border-[#5471DF]"
              placeholder="From"
            />
            <input
              type="date"
              value={filters.executed_to}
              onChange={(e) => { setFilters(f => ({ ...f, executed_to: e.target.value })); setPage(1); }}
              className="flex-1 px-2 py-2 text-[13px] rounded-[4px] border border-[rgba(14,30,63,0.2)] focus:outline-none focus:border-[#5471DF]"
              placeholder="To"
            />
          </div>

          {/* Employee */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
            <input
              type="text"
              value={filters.employee_name}
              onChange={(e) => { setFilters(f => ({ ...f, employee_name: e.target.value })); setPage(1); }}
              placeholder="Employee..."
              className="pl-10 w-full py-2 text-[13px] rounded-[4px] border border-[rgba(14,30,63,0.2)] focus:outline-none focus:border-[#5471DF]"
            />
          </div>

          {/* Security */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
            <input
              type="text"
              value={filters.security}
              onChange={(e) => { setFilters(f => ({ ...f, security: e.target.value })); setPage(1); }}
              placeholder="Instrument..."
              className="pl-10 w-full py-2 text-[13px] rounded-[4px] border border-[rgba(14,30,63,0.2)] focus:outline-none focus:border-[#5471DF]"
            />
          </div>

          {/* Status */}
          <select
            value={filters.verification_status}
            onChange={(e) => { setFilters(f => ({ ...f, verification_status: e.target.value })); setPage(1); }}
            className="px-3 py-2 text-[13px] rounded-[4px] border border-[rgba(14,30,63,0.2)] focus:outline-none focus:border-[#5471DF]"
          >
            <option value="">All Statuses</option>
            <option value="verified">Verified</option>
            <option value="pending">Pending</option>
            <option value="mismatched">Variance</option>
            <option value="missing">Missing</option>
          </select>

          {/* Clear */}
          {(filters.executed_from || filters.executed_to || filters.employee_name || filters.security || filters.verification_status) && (
            <button
              onClick={() => { setFilters({ executed_from: '', executed_to: '', employee_name: '', security: '', verification_status: '' }); setPage(1); }}
              className="text-[11px] font-bold uppercase text-slate-400 hover:text-red-500"
            >
              Clear Filters
            </button>
          )}
        </div>
      </Card>

      {/* Table */}
      <Card noPadding>
        <div className="px-3 py-[6px] bg-[#0E1E3F] text-white rounded-t-[4px] flex items-center justify-between">
          <h2 className="font-bold text-xs">Executed Trades</h2>
          <span className="text-[10px] text-slate-300">{total} total</span>
        </div>
        <Table
          data={items}
          columns={columns}
          keyExtractor={(row) => row.request_id.toString()}
          isLoading={isLoading}
          emptyMessage="No executed trades found"
          renderRow={renderRow}
        />
        {/* Pagination */}
        {total > pageSize && (
          <div className="px-3 py-2 bg-slate-50 border-t flex items-center justify-between text-[11px]">
            <span className="text-slate-500">
              Showing {(page - 1) * pageSize + 1}-{Math.min(page * pageSize, total)} of {total}
            </span>
            <div className="flex gap-2">
              <button
                onClick={() => setPage(p => Math.max(1, p - 1))}
                disabled={page === 1}
                className="px-3 py-1 rounded-[4px] bg-white border border-slate-200 disabled:opacity-50 hover:bg-slate-50"
              >
                Previous
              </button>
              <button
                onClick={() => setPage(p => p + 1)}
                disabled={!hasMore}
                className="px-3 py-1 rounded-[4px] bg-white border border-slate-200 disabled:opacity-50 hover:bg-slate-50"
              >
                Next
              </button>
            </div>
          </div>
        )}
      </Card>
    </div>
  )
}
```

---

## Phase 4: Navigation & Routing

### 4.1 Add Sidebar Navigation

**File**: `dashboard/src/components/layout/Sidebar.tsx`

Add import:
```typescript
import { History } from 'lucide-react'
```

Add to `navItems` array after "Execution Tracking":
```typescript
{ path: '/trade-history', label: 'Trade History', icon: <History className="w-5 h-5" />, access: 'compliance' },
```

### 4.2 Add Route

**File**: `dashboard/src/App.tsx`

Add import:
```typescript
import TradeHistory from '@/pages/TradeHistory'
```

Add route after Execution Tracking:
```typescript
<Route path="/trade-history" element={
  <ProtectedRoute access="compliance">
    <TradeHistory />
  </ProtectedRoute>
} />
```

---

## Phase 5: Contract Note Viewer Integration

### 5.1 Add Expandable Row Support to Table Component

**File**: `dashboard/src/components/ui/Table.tsx`

If not already supported, add `renderRow` prop:
```typescript
interface TableProps<T> {
  // ... existing props
  renderRow?: (row: T, rowContent: React.ReactNode) => React.ReactNode
}
```

### 5.2 Enhance Contract Note Preview

Already included in Phase 3. Enhancements:
- Add variance breakdown display
- Add link to PDF History page for full extraction details
- Add "View in PAD Request" link

---

## Phase 6: Tests

### 6.1 Backend Unit Test

**File**: `tests/unit/test_trade_history_api.py`

```python
import pytest
from datetime import datetime, timedelta
from httpx import AsyncClient

pytestmark = pytest.mark.asyncio


async def test_trade_history_requires_compliance(client: AsyncClient, normal_user_headers):
    """Trade history endpoint requires compliance role."""
    response = await client.get("/api/dashboard/trade-history", headers=normal_user_headers)
    assert response.status_code == 403


async def test_trade_history_returns_executed_trades(client: AsyncClient, compliance_headers, executed_trade):
    """Trade history returns executed trades with verification status."""
    response = await client.get("/api/dashboard/trade-history", headers=compliance_headers)
    assert response.status_code == 200
    data = response.json()["data"]
    assert "items" in data
    assert "total" in data


async def test_trade_history_filters_by_date(client: AsyncClient, compliance_headers, executed_trade):
    """Trade history can filter by execution date range."""
    today = datetime.utcnow().strftime("%Y-%m-%d")
    response = await client.get(
        f"/api/dashboard/trade-history?executed_from={today}",
        headers=compliance_headers,
    )
    assert response.status_code == 200


async def test_trade_history_filters_by_verification_status(client: AsyncClient, compliance_headers):
    """Trade history can filter by contract note verification status."""
    response = await client.get(
        "/api/dashboard/trade-history?verification_status=missing",
        headers=compliance_headers,
    )
    assert response.status_code == 200
```

### 6.2 Frontend E2E Test

**File**: `dashboard/tests/trade_history.spec.ts`

```typescript
import { test, expect } from '@playwright/test';

test.describe('Trade History Page', () => {
  test.beforeEach(async ({ page }) => {
    page.on('console', (msg) => {
      if (msg.type() === 'error' && !msg.text().includes('favicon')) {
        console.error(`Console error: ${msg.text()}`);
      }
    });
  });

  test('page loads correctly for compliance users', async ({ page }) => {
    await page.goto('/trade-history', { waitUntil: 'networkidle' });

    // Should show page title
    await expect(page.locator('h1')).toContainText(/Trade History/i);
  });

  test('displays filter controls', async ({ page }) => {
    await page.goto('/trade-history', { waitUntil: 'networkidle' });

    // Check for filter elements
    await expect(page.locator('input[type="date"]').first()).toBeVisible();
    await expect(page.locator('input[placeholder*="Employee"]')).toBeVisible();
    await expect(page.locator('input[placeholder*="Instrument"]')).toBeVisible();
    await expect(page.locator('select')).toBeVisible();
  });

  test('displays executed trades table', async ({ page }) => {
    await page.goto('/trade-history', { waitUntil: 'networkidle' });

    // Check table header
    const tableHeader = page.locator('.bg-\\[\\#0E1E3F\\]').filter({ hasText: /Executed Trades/i });
    await expect(tableHeader).toBeVisible();

    // Check for column headers
    await expect(page.locator('th', { hasText: /Request/i }).first()).toBeVisible();
    await expect(page.locator('th', { hasText: /Employee/i }).first()).toBeVisible();
    await expect(page.locator('th', { hasText: /Contract Note/i }).first()).toBeVisible();
  });

  test('contract note viewer expands on click', async ({ page }) => {
    await page.goto('/trade-history', { waitUntil: 'networkidle' });

    // Find expand button (chevron) if any trades exist
    const expandButton = page.locator('button[title="View Contract Note"]').first();

    if (await expandButton.isVisible()) {
      await expandButton.click();

      // Should show expanded content
      await page.waitForSelector('iframe, [class*="bg-slate-50"]');
    }
  });

  test('verification status badge displays correctly', async ({ page }) => {
    await page.goto('/trade-history', { waitUntil: 'networkidle' });

    // Look for status badges
    const badges = page.locator('span').filter({ hasText: /Verified|Pending|Variance|Missing/i });
    const count = await badges.count();

    console.log(`[Trade History] Found ${count} verification status badges`);
  });
});
```

---

## Verification Checklist

### Phase 1 (Backend)
- [ ] Schema added to `schemas.py`
- [ ] Repository method added
- [ ] API endpoint works with auth
- [ ] Filters work correctly
- [ ] Pagination works

### Phase 2 (Types & Client)
- [ ] TypeScript types added
- [ ] API client method added
- [ ] TypeScript compiles without errors

### Phase 3 (Page Component)
- [ ] Page renders
- [ ] Filters update query
- [ ] Table displays data
- [ ] Contract note preview expands
- [ ] Variance indicators display

### Phase 4 (Navigation)
- [ ] Sidebar link visible to compliance
- [ ] Route works
- [ ] Access control enforced

### Phase 5 (Viewer)
- [ ] PDF loads in iframe
- [ ] Open in new tab works
- [ ] Variance metadata displays

### Phase 6 (Tests)
- [ ] Backend tests pass
- [ ] Frontend tests pass
- [ ] No TypeScript errors
- [ ] Build succeeds
