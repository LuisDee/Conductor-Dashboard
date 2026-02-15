# Trade History Dashboard Page - Specification

## Overview

A new compliance-focused dashboard page that displays all **executed trades** - trades that have been approved, executed, and have a contract note/activity statement uploaded. This provides a complete audit trail of completed personal account dealing transactions.

## Problem Statement

Currently there is no single view showing:
- All trades that have completed the full PAD lifecycle (requested → approved → executed → documented)
- Contract note verification status
- Execution details (actual price, quantity, date from broker confirmation)

Compliance needs visibility into completed trades for:
- Audit trail purposes
- Contract note variance detection
- Historical reporting
- Post-trade compliance reviews

## User Stories

1. **As a Compliance Officer**, I want to see all executed trades in one place, so I can review the complete audit trail
2. **As a Compliance Officer**, I want to filter trades by employee, date range, and instrument, so I can find specific transactions
3. **As a Compliance Officer**, I want to view the original contract note/activity statement for any trade, so I can verify execution details
4. **As a Compliance Officer**, I want to see variance indicators when extracted data differs from the original request
5. **As a Compliance Officer**, I want to see a clear error message (e.g., "Missing GCS path") when a PDF cannot be loaded, so I understand the storage state.

## Functional Requirements

### Page Access
- **Route**: `/trade-history`
- **Access**: Compliance and Admin users only (matches Execution Tracking pattern)
- **Navigation**: Add to sidebar under "Execution Tracking" in Operations section

### Data Source
Join data from:
- `PADRequest` (original request details)
- `PADExecution` (execution details, recorded_by)
- `ContractNoteUpload` (document metadata, verification_status)
- `ParsedTrade` (extracted data from document if available)
- `GCSDocument` (for PDF access)

### Filter Criteria
| Filter | Type | Description |
|--------|------|-------------|
| Date Range | Date picker | Filter by `executed_at` date |
| Employee | Text/Searchable | Filter by employee name (partial match) |
| Instrument | Text | Filter by ticker, ISIN, or security name |
| Verification Status | Dropdown | All / Verified / Pending / Mismatched |

### Table Columns

| Column | Source | Display |
|--------|--------|---------|
| Request ID | `PADRequest.id` | Link to `/requests/{id}` |
| Employee | `PADRequest.employee_name` | Name (email on hover) |
| Instrument | `PADRequest.security_identifier` | Ticker + name |
| ISIN | `PADRequest.isin` | ISIN code |
| Direction | `PADRequest.direction` | BUY (green) / SELL (red) badge |
| Quantity | `PADExecution.execution_quantity` | Formatted number |
| Price | `PADExecution.execution_price` | Currency + value |
| Executed | `PADExecution.executed_at` | Date + time |
| Contract Note | `ContractNoteUpload.verification_status` | View Button (FileSearch icon) |

### Contract Note Viewer
- **Toggle View**: Clicking the "View" button (FileSearch icon) shows/hides inline PDF viewer below the row (accordion pattern).
- **Error States**: If PDF fails to load, display the specific error from the API (e.g., "GCS bucket name not configured", "Blob not found").
- **Decision**: Use inline accordion for quick preview, full modal for detailed review.

### Status Badges (for filtering/meta)

| Status | Color | Meaning |
|--------|-------|---------|
| `verified` | Green | Contract note matches request details |
| `pending` | Blue | Contract note uploaded, awaiting verification |
| `mismatched` | Amber/Red | Variance detected between request and document |
| `missing` | Gray | No contract note uploaded yet |

### Variance Indicators
When `verification_status = 'mismatched'`, show warning icon with tooltip indicating which fields differ:
- Quantity variance
- Price variance
- Date variance
- Direction mismatch

## Technical Requirements

### Backend API

**Endpoint**: `GET /api/dashboard/trade-history`

**Query Parameters**:
```
executed_from: string (YYYY-MM-DD) - optional
executed_to: string (YYYY-MM-DD) - optional
employee_name: string - optional, partial match
security: string - optional, matches ticker/ISIN/name
verification_status: string - optional (verified/pending/mismatched/missing)
page: int - optional, default 1
page_size: int - optional, default 50
```

### Frontend Components

**File**: `dashboard/src/pages/TradeHistory.tsx`
**Icon**: Use `FileSearch` from `lucide-react` for the "View" action.

### Database Query
```sql
SELECT
  pr.id as request_id,
  pr.reference_id,
  pr.employee_id,
  pr.employee_name,
  pr.employee_email,
  pr.security_identifier,
  pr.security_name,
  pr.isin,
  pr.direction,
  pr.quantity as requested_quantity,
  pr.estimated_price,
  pe.execution_quantity,
  pe.execution_price,
  pe.executed_at,
  oe.name as recorded_by_name,
  cnu.id as contract_note_id,
  cnu.original_filename as contract_note_filename,
  cnu.verification_status,
  cnu.verification_metadata,
  cnu.gcs_document_id
FROM pad_request pr
INNER JOIN pad_execution pe ON pe.request_id = pr.id
LEFT JOIN contract_note_upload cnu ON cnu.request_id = pr.id AND cnu.is_active = true
LEFT JOIN bo_airflow.oracle_employee oe ON oe.id = pe.recorded_by_id
WHERE pr.status = 'executed'
ORDER BY pe.executed_at DESC
```

## UI/UX Requirements

### Visual Design
- Match existing dashboard styling (slate backgrounds, blue accents)
- Use consistent table component (`Table` from `@/components/ui/Table`)
- Use `FileSearch` icon for viewing documents.
- Match filter card styling from other pages

## Non-Functional Requirements

- Signed URL error messages must be descriptive (not generic).
- signed GCS URLs (15 min expiry)

## Success Criteria
1. Compliance can view all executed trades with contract notes
2. Filters work correctly and update URL params
3. Contract note PDFs viewable directly from page via `FileSearch` icon.
4. Descriptive error messages shown when PDF fails to load.
5. Page accessible only to compliance/admin users
