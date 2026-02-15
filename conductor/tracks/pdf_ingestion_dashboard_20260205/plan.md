# PDF Ingestion History Dashboard

## Research: Industry Best Practices (2025-2026)

Based on web research into dashboard design, IDP platforms, and document processing UIs:

### Dashboard Design Principles
*Sources: [UXPin](https://www.uxpin.com/studio/blog/dashboard-design-principles/), [DesignRush](https://www.designrush.com/agency/ui-ux-design/dashboard/trends/dashboard-design-principles), [Qlik](https://www.qlik.com/us/dashboard-examples/dashboard-design)*

1. **5-Second Rule**: User should understand the most critical info in 5 seconds
2. **Minimalist Design**: Uncluttered interfaces, essential elements only
3. **Real-Time Data**: 2025+ dashboards demand real-time or near-real-time updates
4. **Accessibility**: High-contrast schemes, keyboard navigation, screen reader support
5. **Progressive Disclosure**: Show summary first, drill-down for details

### Intelligent Document Processing (IDP) UI Patterns
*Sources: [Docsumo](https://www.docsumo.com/solutions/intelligent-document-processing-platform), [Nanonets](https://nanonets.com/buyers-guide/best-intelligent-document-processing-software), [ScaleHub](https://scalehub.com/2025-idp-guide/)*

1. **Excel-like Data Tables**: Familiar spreadsheet view for extracted data
2. **Visual Validation UI**: Quick resolution of inaccuracies/exceptions
3. **Side-by-Side View**: PDF on left, extracted data on right (critical for QA)
4. **Confidence Highlighting**: Visual markers on low-confidence extractions
5. **No-Code Configuration**: Point-and-click for non-technical users

### Confidence Score Visualization
*Sources: [Microsoft Azure Doc Intelligence](https://learn.microsoft.com/en-us/azure/ai-services/document-intelligence/concept/accuracy-confidence), [Veryfi](https://faq.veryfi.com/en/articles/5571597-confidence-score-explained), [LEADTOOLS](https://www.leadtools.com/help/sdk/v20/dh/to/ocr-confidence-reporting.html)*

1. **Threshold-Based Coloring**:
   - HIGH (>90%): Green
   - MEDIUM (70-90%): Yellow/Gold
   - LOW (<70%): Red (requires review)
2. **Field-Level Scores**: Show confidence per extracted field, not just document
3. **Visual Markers**: Color-coded highlighting in extracted text
4. **Don't Rely Blindly**: Different use cases need different thresholds

### Audit Trail & Compliance
*Sources: [MetricStream](https://www.metricstream.com/learn/compliance-dashboard.html), [Datadog](https://www.datadoghq.com/product/audit-trail/), [DocuWare](https://start.docuware.com/blog/document-management/audit-trails)*

1. **Time-Stamped Events**: Every action logged with timestamp
2. **Color-Coded Urgency**: Quick visual identification of issues
3. **Drill-Down Capability**: Summary → detailed event log
4. **Export for Audits**: One-click report generation

### Pipeline Monitoring KPIs
*Sources: [Telmai](https://www.telm.ai/blog/the-12-data-pipeline-metrics-that-matter-most/), [Metaplane](https://www.metaplane.dev/blog/data-quality-metrics-for-data-warehouses)*

1. **Throughput**: Documents/hour, trades/day
2. **Latency**: Avg processing time, P95 processing time
3. **Success Rate**: (Successful / Total) × 100
4. **Error Rate**: Failed extractions / Total attempts
5. **Field Coverage**: % of documents with each field populated

### Side-by-Side PDF Comparison View
*Sources: [Adobe Acrobat](https://helpx.adobe.com/acrobat/using/compare-documents.html), [PDiff](https://www.csci.de/en/pdiff/), [Draftable](https://www.draftable.com/compare)*

**Key Pattern**: Split-screen with:
- Left panel: Original PDF (scrollable, zoomable)
- Right panel: Extracted data table with confidence indicators
- Synchronized scrolling optional
- Highlight extraction regions on PDF when hovering over fields

---

## Overview

Create a new dashboard page that provides full visibility into the PDF ingestion pipeline, showing all documents processed by the system with their extraction results, confidence scores, and matched trades.

## User Value

- **Compliance**: See all PDFs processed and verify extraction accuracy
- **Operations**: Monitor pipeline health, identify failures, retry failed documents
- **Debugging**: View raw PDF alongside extracted fields to identify extraction issues
- **Audit**: Complete history of all document processing with timestamps

## Data Sources

### Primary Tables
| Table | Schema | Purpose |
|-------|--------|---------|
| `gcs_document` | padealing | Core document record with status, source, paths |
| `parsed_trade` | padealing | Extracted trade data with confidence scores |
| `email_ingestion_state` | bo_airflow | Email-level tracking (attachments, trade counts) |
| `document_processing_log` | padealing | Event timeline for each document |

### Key Fields Available

**Document Info:**
- `original_filename`, `archive_gcs_path`
- `source` (mailbox_poller, graph_polling, manual_upload)
- `email_source` (broker, user, manual, unknown)
- `sender_email`, `received_at`, `processed_at`
- `status` (pending, processing, parsed, failed, rejected)
- `error_message`, `retry_count`

**Extraction Results (per trade):**
- `ticker`, `isin`, `sedol`, `cusip`, `security_description`
- `direction`, `quantity`, `price`, `notional`, `proceeds`
- `trade_date`, `settlement_date`
- `account_holder`, `broker`, `currency`
- `confidence_score` (0-1)
- `raw_extracted_data` (full JSON from LLM)

**Matching Info:**
- `match_status` (pending, matched, unmatched, manual_review)
- `matched_employee_id`, `match_method`, `match_confidence`
- `request_id` (linked PAD request if matched)

---

## Implementation Plan

### Phase 1: Backend API Endpoints

**File:** `src/pa_dealing/api/routes/pdf_history.py`

#### 1.1 List Documents Endpoint
```
GET /api/pdf-history
```

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `page` | int | Page number (default: 1) |
| `page_size` | int | Items per page (default: 20, max: 100) |
| `source` | string | Filter: mailbox_poller, graph_polling, manual_upload |
| `status` | string | Filter: pending, processing, parsed, failed |
| `date_from` | date | Filter: received after |
| `date_to` | date | Filter: received before |
| `search` | string | Search filename or sender email |
| `has_errors` | bool | Filter to only failed/errored documents |

**Response:**
```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": "uuid",
        "filename": "contract_note_aapl.pdf",
        "source": "graph_polling",
        "source_display": "Email (Graph API)",
        "sender_email": "luis@mako.com",
        "status": "parsed",
        "status_badge": { "text": "Parsed", "color": "green" },
        "received_at": "2026-02-05T21:41:53Z",
        "processed_at": "2026-02-05T21:42:10Z",
        "processing_time_seconds": 17,
        "trade_count": 1,
        "avg_confidence": 0.92,
        "error_message": null,
        "retry_count": 0,
        "archive_path": "gs://bucket/archive/..."
      }
    ],
    "pagination": {
      "page": 1,
      "page_size": 20,
      "total_items": 156,
      "total_pages": 8
    }
  }
}
```

#### 1.2 Document Detail Endpoint
```
GET /api/pdf-history/{document_id}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "document": {
      "id": "uuid",
      "filename": "contract_note_aapl.pdf",
      "source": "graph_polling",
      "sender_email": "luis@mako.com",
      "status": "parsed",
      "received_at": "2026-02-05T21:41:53Z",
      "processed_at": "2026-02-05T21:42:10Z",
      "archive_gcs_path": "gs://...",
      "error_message": null,
      "worker_id": "poller-coder-abc123"
    },
    "trades": [
      {
        "id": 1234,
        "ticker": "AAPL",
        "isin": "US0378331005",
        "direction": "BUY",
        "quantity": 5,
        "price": 235.50,
        "trade_date": "2026-01-25",
        "account_holder": "Luis de Burnay-Bastos",
        "broker": "Interactive Brokers",
        "confidence_score": 0.95,
        "confidence_display": { "level": "HIGH", "color": "green" },
        "match_status": "matched",
        "matched_request_id": 456,
        "match_method": "name_fuzzy"
      }
    ],
    "raw_extraction": {
      "ticker": "AAPL",
      "isin": "US0378331005",
      "direction": "BUY",
      "quantity": 5,
      "price": 235.50,
      "confidence": "high",
      "document_type": "CONTRACT_NOTE",
      "review_reasons": []
    },
    "processing_timeline": [
      { "event": "received", "timestamp": "2026-02-05T21:41:53Z" },
      { "event": "processing_started", "timestamp": "2026-02-05T21:41:54Z" },
      { "event": "parsing_completed", "timestamp": "2026-02-05T21:42:08Z" },
      { "event": "trades_extracted", "timestamp": "2026-02-05T21:42:08Z", "details": { "count": 1 } },
      { "event": "archived", "timestamp": "2026-02-05T21:42:10Z" }
    ]
  }
}
```

#### 1.3 PDF Viewer Endpoint
```
GET /api/pdf-history/{document_id}/pdf
```

Returns signed GCS URL (existing pattern from `/documents/{id}/pdf`).

#### 1.4 Summary Statistics Endpoint
```
GET /api/pdf-history/stats
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_documents": 156,
    "by_status": {
      "parsed": 142,
      "failed": 8,
      "processing": 2,
      "pending": 4
    },
    "by_source": {
      "graph_polling": 120,
      "mailbox_poller": 30,
      "manual_upload": 6
    },
    "today": {
      "processed": 8,
      "failed": 1,
      "trades_extracted": 15
    },
    "avg_confidence": 0.87,
    "avg_processing_time_seconds": 12.5,
    "failure_rate_percent": 5.1
  }
}
```

#### 1.5 Retry Failed Document Endpoint
```
POST /api/pdf-history/{document_id}/retry
```

Resets status to `pending` for reprocessing (if `status='failed'` and `retry_count < 3`).

---

### Phase 2: API Response Schemas

**File:** `src/pa_dealing/api/schemas.py` (additions)

```python
class PDFHistoryItem(BaseModel):
    id: UUID
    filename: str
    source: str
    source_display: str
    sender_email: str | None
    status: str
    status_badge: dict  # { text, color }
    received_at: datetime
    processed_at: datetime | None
    processing_time_seconds: float | None
    trade_count: int
    avg_confidence: float | None
    error_message: str | None
    retry_count: int

class ExtractedTradeDetail(BaseModel):
    id: int
    ticker: str | None
    isin: str | None
    sedol: str | None
    security_description: str | None
    direction: str | None
    quantity: Decimal | None
    price: Decimal | None
    notional: Decimal | None
    trade_date: date | None
    account_holder: str | None
    broker: str | None
    confidence_score: float | None
    confidence_display: dict  # { level: HIGH/MEDIUM/LOW, color }
    match_status: str | None
    matched_request_id: int | None
    match_method: str | None

class PDFHistoryDetail(BaseModel):
    document: PDFHistoryItem
    trades: list[ExtractedTradeDetail]
    raw_extraction: dict  # Full raw_extracted_data JSON
    processing_timeline: list[dict]

class PDFHistoryStats(BaseModel):
    total_documents: int
    by_status: dict[str, int]
    by_source: dict[str, int]
    today: dict[str, int]
    avg_confidence: float
    avg_processing_time_seconds: float
    failure_rate_percent: float
```

---

### Phase 3: Frontend Components

**File:** `dashboard/src/pages/PDFHistory.tsx`

#### 3.1 Main Page Layout
```
┌─────────────────────────────────────────────────────────────────┐
│  PDF Ingestion History                              [Refresh]   │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │ Total    │ │ Parsed   │ │ Failed   │ │ Avg Conf │           │
│  │   156    │ │   142    │ │    8     │ │   87%    │           │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │
├─────────────────────────────────────────────────────────────────┤
│  Filters: [Status ▾] [Source ▾] [Date From] [Date To] [Search] │
├─────────────────────────────────────────────────────────────────┤
│  Filename          │ Source   │ Status  │ Trades │ Conf │ Date │
│  ─────────────────────────────────────────────────────────────  │
│  contract_aapl.pdf │ Email    │ ✓ Parsed│   1    │ 95%  │ 2/5  │
│  statement.pdf     │ Manual   │ ✓ Parsed│   3    │ 88%  │ 2/5  │
│  note_fgbl.pdf     │ Email    │ ✗ Failed│   0    │  -   │ 2/4  │
│  ...               │          │         │        │      │      │
├─────────────────────────────────────────────────────────────────┤
│  Page 1 of 8                              [◀ Prev] [Next ▶]    │
└─────────────────────────────────────────────────────────────────┘
```

#### 3.2 Document Detail Modal/Page
```
┌─────────────────────────────────────────────────────────────────┐
│  contract_note_aapl.pdf                          [View PDF] [X] │
├─────────────────────────────────────────────────────────────────┤
│  Document Info                                                  │
│  ───────────────────────────────────────                        │
│  Source: Email (Graph Polling)                                  │
│  Sender: luis@mako.com                                          │
│  Received: Feb 5, 2026 9:41 PM                                  │
│  Processed: Feb 5, 2026 9:42 PM (17 seconds)                    │
│  Status: ✓ Parsed                                               │
├─────────────────────────────────────────────────────────────────┤
│  Extracted Trades                                               │
│  ───────────────────────────────────────                        │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ AAPL (US0378331005) │ BUY │ 5 @ $235.50 │ 2026-01-25   │   │
│  │ Confidence: 95% (HIGH)                                   │   │
│  │ Account: Luis de Burnay-Bastos                          │   │
│  │ Match: ✓ Matched to PAD #456 (name_fuzzy)               │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  [▾ Raw Extraction Data]  (collapsible JSON viewer)            │
│  {                                                              │
│    "ticker": "AAPL",                                            │
│    "isin": "US0378331005",                                      │
│    "confidence": "high",                                        │
│    ...                                                          │
│  }                                                              │
├─────────────────────────────────────────────────────────────────┤
│  Processing Timeline                                            │
│  ───────────────────────────────────────                        │
│  ● 9:41:53 PM - Received                                        │
│  ● 9:41:54 PM - Processing started                              │
│  ● 9:42:08 PM - Parsing completed                               │
│  ● 9:42:08 PM - 1 trade extracted                               │
│  ● 9:42:10 PM - Archived to GCS                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 3.3 Side-by-Side PDF Comparison View (Research-Driven)

**Key UX Pattern** (from IDP industry research):
```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Document Review: contract_note_aapl.pdf                        [Close] [X] │
├────────────────────────────────┬────────────────────────────────────────────┤
│                                │  Extracted Data                            │
│   ┌────────────────────────┐   │  ───────────────────────────────           │
│   │                        │   │  Ticker      AAPL         ● 98%            │
│   │   [PDF VIEWER]         │   │  ISIN        US0378331005 ● 95%            │
│   │                        │   │  Direction   BUY          ● 99%            │
│   │   contract_note_aapl   │   │  Quantity    5            ● 97%            │
│   │                        │   │  Price       $235.50      ● 94%            │
│   │   - Zoom controls      │   │  Trade Date  2026-01-25   ● 92%            │
│   │   - Page navigation    │   │  Account     Luis de B... ● 78% ⚠         │
│   │   - Full screen        │   │  Broker      IB           ● 85%            │
│   │                        │   │                                            │
│   └────────────────────────┘   │  Overall Confidence: 92% (HIGH)            │
│                                │                                            │
│   Page 1 of 1                  │  [▾ Raw JSON]  [▾ Processing Log]          │
└────────────────────────────────┴────────────────────────────────────────────┘
```

**Implementation:**
- Left panel: PDF.js viewer with signed GCS URL
- Right panel: Extracted fields with per-field confidence indicators
- Confidence colors: GREEN (>90%), GOLD (70-90%), RED (<70%)
- Hover on field → highlight region in PDF (future enhancement)
- Collapsible sections for raw JSON and processing timeline

---

### Phase 4: Navigation & Routing

**File:** `dashboard/src/App.tsx`
- Add route: `/pdf-history`

**File:** `dashboard/src/components/layout/Sidebar.tsx`
- Add nav item: "PDF History" under Documents section
- Icon: `DocumentTextIcon` or similar

---

### Phase 5: Pipeline Performance Analytics

#### 5.1 Summary Stats Cards (Top of Page)
```
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ Total PDFs   │ │ Success Rate │ │ Avg Conf.    │ │ Avg Process  │
│    156       │ │    91.7%     │ │    87%       │ │   12.5s      │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘

┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ Today        │ │ This Week    │ │ Failed Today │ │ Pending      │
│    12        │ │    84        │ │     1        │ │     3        │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
```

#### 5.2 Pipeline Throughput Chart
- Line chart: documents processed per hour/day
- Overlay: success vs failed over time
- Identify busy periods and bottlenecks

#### 5.3 Processing Time Distribution
```
Processing Time (seconds)
│  ████████████████████████  0-5s   (45%)
│  ██████████████            5-15s  (32%)
│  ██████                    15-30s (15%)
│  ██                        30-60s (5%)
│  █                         60s+   (3%)
```

#### 5.4 Confidence Score Distribution
```
Extraction Confidence
│  HIGH (>90%)    ████████████████████  (68%)
│  MEDIUM (70-90%) ██████████           (24%)
│  LOW (<70%)      ███                  (8%)
```

#### 5.5 Source Breakdown
```
Document Sources (Last 30 Days)
┌─────────────────────────────────────┐
│  █████████████████  Graph Polling   │ 120 (77%)
│  ██████             Mailbox Poller  │  30 (19%)
│  ██                 Manual Upload   │   6 (4%)
└─────────────────────────────────────┘
```

#### 5.6 Failure Analysis Dashboard
```
Failed Documents by Error Type
┌────────────────────────────────────────────┐
│  LLM Extraction Error     ████████  (5)    │
│  Invalid PDF Format       ████      (3)    │
│  No Trades Found          ███       (2)    │
│  Timeout                  ██        (1)    │
│  GCS Upload Failed        █         (1)    │
└────────────────────────────────────────────┘

Recent Failures (expandable list with retry buttons)
```

#### 5.7 Email Ingestion Stats (Graph API)
```
Email Pipeline Metrics (from bo_airflow.email_ingestion_state)
┌─────────────────────────────────────────────────────────────────┐
│  Emails Processed    │  142  │  Total emails with PDFs ingested │
│  Attachments Found   │  213  │  Total PDF attachments           │
│  Trades Extracted    │  287  │  Total trades from emails        │
│  Avg Attach/Email    │  1.5  │  Attachments per email          │
│  Avg Trades/Email    │  2.0  │  Trades per email               │
│  Last Poll           │  2min │  Time since last Graph poll     │
│  Poll Success Rate   │  100% │  Successful poll cycles         │
└─────────────────────────────────────────────────────────────────┘
```

#### 5.8 User Matching Performance
```
Trade Matching Breakdown
┌─────────────────────────────────────────────────────────────────┐
│  ✓ Auto-Matched (email)     ██████████████████   72%           │
│  ✓ Auto-Matched (name)      ████████             18%           │
│  ⚠ Manual Review Required   ███                   7%           │
│  ✗ Unmatched                █                     3%           │
└─────────────────────────────────────────────────────────────────┘

Match Method Distribution:
  - email_exact: 145 trades
  - name_exact: 32 trades
  - name_fuzzy: 20 trades
  - manual: 15 trades
  - unmatched: 8 trades
```

#### 5.9 Extraction Field Coverage
```
Field Extraction Rate (% of documents with field populated)
┌─────────────────────────────────────────────────────────────────┐
│  Ticker/Symbol        ████████████████████████████  95%        │
│  Direction (BUY/SELL) ████████████████████████████  94%        │
│  Quantity             ████████████████████████████  93%        │
│  Price                ████████████████████████████  92%        │
│  Trade Date           ██████████████████████████    88%        │
│  ISIN                 ████████████████████          72%        │
│  Account Holder       ███████████████               65%        │
│  Broker               █████████████                 58%        │
│  Settlement Date      ███████████                   48%        │
└─────────────────────────────────────────────────────────────────┘
```

#### 5.10 Real-Time Pipeline Status
```
Pipeline Health
┌─────────────────────────────────────────────────────────────────┐
│  Graph Poller        ● Running    Last: 2 min ago              │
│  Mailbox Poller      ● Running    Last: 5 min ago              │
│  Trade Processor     ● Healthy    Queue: 0 pending             │
│  GCS Archiver        ● Healthy    Last archive: 3 min ago      │
│  LLM (Gemini)        ● Available  Avg latency: 4.2s            │
└─────────────────────────────────────────────────────────────────┘
```

#### 5.11 Time-Series Analytics Endpoint
```
GET /api/pdf-history/analytics
```

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `period` | string | hour, day, week, month |
| `from_date` | date | Start of range |
| `to_date` | date | End of range |

**Response:**
```json
{
  "success": true,
  "data": {
    "throughput": [
      { "timestamp": "2026-02-05T00:00:00Z", "processed": 12, "failed": 1 },
      { "timestamp": "2026-02-05T01:00:00Z", "processed": 8, "failed": 0 }
    ],
    "confidence_trend": [
      { "timestamp": "2026-02-05T00:00:00Z", "avg_confidence": 0.88 }
    ],
    "processing_time_trend": [
      { "timestamp": "2026-02-05T00:00:00Z", "avg_seconds": 11.2, "p95_seconds": 28.5 }
    ],
    "source_breakdown": {
      "graph_polling": 120,
      "mailbox_poller": 30,
      "manual_upload": 6
    },
    "error_breakdown": {
      "extraction_failed": 5,
      "invalid_pdf": 3,
      "no_trades": 2
    },
    "match_breakdown": {
      "email_exact": 145,
      "name_exact": 32,
      "name_fuzzy": 20,
      "manual": 15,
      "unmatched": 8
    },
    "field_coverage": {
      "ticker": 0.95,
      "direction": 0.94,
      "quantity": 0.93,
      "price": 0.92,
      "trade_date": 0.88,
      "isin": 0.72,
      "account_holder": 0.65,
      "broker": 0.58
    }
  }
}
```

#### 5.12 Confidence Breakdown Chart
- Pie chart showing confidence distribution (HIGH/MEDIUM/LOW)
- Helps identify extraction quality trends

#### 5.13 Export Functionality
- Export document list to CSV
- Export extraction results for audit
- Export analytics data for reporting

---

## File Changes Summary

### New Files
| File | Purpose |
|------|---------|
| `src/pa_dealing/api/routes/pdf_history.py` | Backend API endpoints |
| `dashboard/src/pages/PDFHistory.tsx` | Main page component |
| `dashboard/src/components/PDFHistoryTable.tsx` | Document list table |
| `dashboard/src/components/PDFDetailModal.tsx` | Detail view modal |
| `dashboard/src/components/PDFViewer.tsx` | Embedded PDF viewer |
| `dashboard/src/components/ExtractionDataView.tsx` | JSON viewer for raw data |

### Modified Files
| File | Change |
|------|--------|
| `src/pa_dealing/api/main.py` | Register pdf_history router |
| `src/pa_dealing/api/schemas.py` | Add PDF history schemas |
| `dashboard/src/App.tsx` | Add route |
| `dashboard/src/components/layout/Sidebar.tsx` | Add nav item |

---

---

## Page Template Specification (MAKO Design System Compliance)

Based on research of existing pages (PADSearch, Dashboard, MyRequests, AuditLog), the PDF History page MUST follow these patterns for visual consistency:

### Layout Structure
```
Root Container (space-y-3 gap)
├── Header Section
│   ├── Page Title (page-title class)
│   └── Subtitle (text-slate-500, font-medium, text-[13px])
├── Stats Cards Row (grid-cols-4)
├── Filters Card (bg-white/50, border-dashed)
├── Main Table (Card noPadding + Table component)
└── Pagination Footer (inside Card)
```

### CSS Classes & Tokens

| Element | Classes |
|---------|---------|
| Page wrapper | `<div className="space-y-3">` |
| Title | `<h1 className="page-title">` |
| Subtitle | `<p className="text-slate-500 font-medium text-[13px]">` |
| Filter card | `<Card noPadding className="bg-white/50 border-dashed rounded-[4px]">` |
| Filter grid | `grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-2` |
| Filter label | `<label className="label flex items-center gap-2">` |
| Filter input | `<input className="input" />` |
| Reset button | `btn btn-secondary w-full text-xs uppercase tracking-widest font-black h-[42px]` |
| Table wrapper | `<Card noPadding>` |
| Pagination | `px-3 py-2 border-t border-slate-100 flex items-center justify-between bg-white/30 rounded-b-[4px]` |

### MAKO Color Tokens
| Token | Value | Usage |
|-------|-------|-------|
| `--navy` | #0E1E3F | Primary text, titles |
| `--blue` | #5471DF | Links, primary actions |
| `--gold` | #B28C54 | Warnings, highlights |
| `--light-blue` | #DBE1F5 | Table headers, secondary buttons |
| `--success` | #2C5F2D | Success badges |
| `--error` | #B85042 | Error badges |

### Font Stack
- Family: Montserrat (all text)
- Weights: 400 (body), 500 (subtitle), 600 (labels), 700 (titles), 800 (stat values)
- Title size: 18px
- Body size: 14px
- Small/label: 11-13px
- Mono for IDs: font-mono

### Table Column Patterns
```tsx
// Text column
{ header: 'Name', accessor: (row) => <span className="font-bold text-[#0E1E3F]">{row.name}</span> }

// Link column
{ header: 'ID', accessor: (row) => <Link className="font-mono font-black text-[#5471DF]">#{row.id}</Link> }

// Date column
{ header: 'Date', accessor: (row) => (
  <div className="flex flex-col">
    <span className="text-[#0E1E3F] font-bold text-xs">{formatDate(row.date)}</span>
    <span className="text-[10px] text-slate-400 font-mono">{formatTime(row.date)}</span>
  </div>
)}

// Status column
{ header: 'Status', accessor: (row) => <StatusBadge status={row.status} /> }

// Small text
{ header: 'Type', accessor: (row) => <span className="text-[11px] text-slate-500 uppercase font-semibold">{row.type}</span> }
```

### API Integration Pattern
```tsx
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

const { data, isLoading, error } = useQuery({
  queryKey: ['pdf-history', filters, page],  // Include filters in key!
  queryFn: () => pdfHistoryApi.list({ ...filters, page }),
})
```

### Loading State
Pass `isLoading={isLoading}` to Table component → renders 5 skeleton rows automatically.

### Error State
```tsx
if (error) {
  return (
    <Card accentColor="error">
      <div className="flex items-center gap-3" style={{ color: '#B85042' }}>
        <AlertTriangle className="w-5 h-5" />
        <p style={{ fontFamily: 'Montserrat', fontWeight: 600 }}>Error: {error.message}</p>
      </div>
    </Card>
  )
}
```

---

## Testing Plan

1. **Unit Tests:**
   - API endpoint response schemas
   - Filter/pagination logic
   - Confidence level calculation

2. **Integration Tests:**
   - Full flow: document → API → frontend render
   - PDF signed URL generation
   - Retry failed document flow

3. **Manual Testing:**
   - Verify all filter combinations work
   - Test PDF viewer with various file types
   - Test with documents in all status states
   - Verify raw extraction data displays correctly

---

## Success Criteria

- [ ] Can view list of all ingested PDFs with status
- [ ] Can filter by source, status, date range
- [ ] Can see extracted trade details for each document
- [ ] Can view raw extraction JSON for debugging
- [ ] Can view original PDF inline
- [ ] Can retry failed documents
- [ ] Stats show pipeline health metrics
- [ ] Confidence scores displayed clearly
- [ ] Processing timeline shows audit trail
- [ ] **Red error banner** for trades with no matching PAD request
- [ ] **Side-by-side PDF viewer** for LLM extraction validation
- [ ] **Resolution workflow** for unmatched trades

---

## Phase 6: Side-by-Side PDF Viewer & Unmatched Trade Resolution

### Problem Statement (User Feedback)

The current detail modal shows extracted trades with `match_status = "manual_review"` but:
1. **No way to see the actual PDF** to verify what the LLM extracted
2. **No clear error message** when trades were extracted but no PAD request matched
3. **No resolution workflow** - users can't take action on unmatched trades

### Requirements

#### Must Have
1. **View PDF from the interface** - Click to open PDF in a viewer
2. **Red error message** when trades are extracted but no PAD request matched
3. **Side-by-side view** - PDF on left, extracted fields on right
4. **Manual review workflow** - All trades need human verification at this stage

#### Behavior Rules
- **Has extracted trades + no match** → Show RED error: "No matching PAD request found"
- **No extracted trades** → No error (document may not be a contract note)
- **Has match** → Show linked PAD request with clickable link

---

### 6.1 Add Error State to Document Table

**File**: `dashboard/src/pages/PDFHistory.tsx`

In the "Match" column of the document table, show clear error states:

```tsx
// Current: just shows "manual_review" text
// New: Show red error if trades exist but no match

function MatchStatusCell({ trade }: { trade: ExtractedTradeDetail }) {
  if (trade.match_status === 'manual_review') {
    return (
      <span className="text-[#B85042] font-semibold flex items-center gap-1">
        <AlertTriangle className="w-4 h-4" />
        No matching PAD request
      </span>
    );
  }

  if (trade.match_status === 'matched' && trade.matched_request_id) {
    return (
      <Link
        to={`/requests/${trade.matched_request_id}`}
        className="text-[#5471DF] font-mono font-bold hover:underline"
      >
        PAD-{trade.matched_request_id}
      </Link>
    );
  }

  return <span className="text-slate-400">—</span>;
}
```

---

### 6.2 Add "View PDF" Button to Each Row

**File**: `dashboard/src/pages/PDFHistory.tsx`

Add a document icon button that opens the PDF:

```tsx
// API function in dashboard/src/api/client.ts
export const pdfHistory = {
  // ... existing functions ...
  getPdfUrl: async (documentId: string): Promise<{ url: string }> => {
    const response = await apiClient.get(`/documents/${documentId}/pdf`);
    return response.data;
  },
};

// In table row actions column:
const handleViewPdf = async (documentId: string) => {
  try {
    const { url } = await pdfHistory.getPdfUrl(documentId);
    window.open(url, '_blank');
  } catch (error) {
    toast.error('Failed to load PDF');
  }
};

// Table column:
{
  header: '',
  accessor: (row) => (
    <button
      onClick={() => handleViewPdf(row.id)}
      title="View PDF"
      className="p-1 hover:bg-slate-100 rounded"
    >
      <FileText className="w-4 h-4 text-[#5471DF]" />
    </button>
  ),
  width: 40,
}
```

---

### 6.3 Side-by-Side Detail Modal

**File**: `dashboard/src/pages/PDFHistory.tsx`

Replace the current detail modal with a split-view layout:

```
┌────────────────────────────────────────────────────────────────────────┐
│ contract_note_aapl.pdf                                         [X]    │
├─────────────────────────────────┬──────────────────────────────────────┤
│                                 │                                      │
│      PDF VIEWER                 │  ⚠️ NO MATCHING PAD REQUEST FOUND   │
│      (iframe with signed URL)   │  ────────────────────────────────   │
│                                 │                                      │
│  ┌───────────────────────┐      │  Extracted Trade #1                  │
│  │                       │      │  ├─ Ticker: AAPL                    │
│  │    [PDF Content]      │      │  ├─ Direction: BUY                   │
│  │                       │      │  ├─ Quantity: 5                      │
│  │  Native browser       │      │  ├─ Price: USD 235.50                │
│  │  PDF controls         │      │  ├─ Trade Date: 25 Jan 2026          │
│  │  (zoom, page nav)     │      │  └─ Confidence: 57% (LOW) ⚠️        │
│  │                       │      │                                      │
│  └───────────────────────┘      │  Account Holder (from PDF):          │
│                                 │  "John Smith"                        │
│                                 │                                      │
│                                 │  Match Attempt:                      │
│                                 │  ❌ No approved PAD request found    │
│                                 │     for AAPL BUY by this user        │
│                                 │                                      │
│    ◄────── draggable ──────►    │  [▾ Raw JSON] [▾ Timeline]          │
│                                 │                                      │
└─────────────────────────────────┴──────────────────────────────────────┘
```

**Implementation approach** (reuse PADSearch.tsx split-view pattern):

```tsx
function PDFDetailModal({ document, onClose }: Props) {
  const [splitPosition, setSplitPosition] = useState(50);
  const [isDragging, setIsDragging] = useState(false);
  const [pdfUrl, setPdfUrl] = useState<string | null>(null);

  // Fetch signed URL on mount
  useEffect(() => {
    pdfHistory.getPdfUrl(document.id).then(({ url }) => setPdfUrl(url));
  }, [document.id]);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg w-[95vw] h-[90vh] flex flex-col">
        {/* Header */}
        <div className="px-4 py-3 border-b flex justify-between items-center bg-[#0E1E3F] text-white rounded-t-lg">
          <h2 className="font-bold">{document.filename}</h2>
          <button onClick={onClose}><X className="w-5 h-5" /></button>
        </div>

        {/* Split view content */}
        <div className="flex-1 flex overflow-hidden">
          {/* Left: PDF Viewer */}
          <div style={{ width: `${splitPosition}%` }} className="h-full">
            {pdfUrl ? (
              <iframe src={pdfUrl} className="w-full h-full border-0" />
            ) : (
              <div className="flex items-center justify-center h-full">
                <Spinner /> Loading PDF...
              </div>
            )}
          </div>

          {/* Draggable divider */}
          <div
            className="w-1 bg-slate-200 hover:bg-[#5471DF] cursor-col-resize"
            onMouseDown={() => setIsDragging(true)}
          />

          {/* Right: Extracted data */}
          <div style={{ width: `${100 - splitPosition}%` }} className="h-full overflow-y-auto p-4">
            <ExtractedDataPanel document={document} />
          </div>
        </div>
      </div>
    </div>
  );
}
```

---

### 6.4 Extracted Data Panel with Error States

**New component**: `dashboard/src/components/ExtractedTradeCard.tsx`

```tsx
interface ExtractedTradeCardProps {
  trade: ExtractedTradeDetail;
  index: number;
}

function ExtractedTradeCard({ trade, index }: ExtractedTradeCardProps) {
  const hasNoMatch = trade.match_status === 'manual_review';

  return (
    <div className="border rounded-lg p-4 mb-4">
      {/* ERROR BANNER - Red if no match */}
      {hasNoMatch && (
        <div className="bg-red-50 border border-red-200 text-[#B85042] p-3 rounded mb-4">
          <div className="flex items-center gap-2 font-semibold">
            <AlertTriangle className="w-5 h-5" />
            No matching PAD request found
          </div>
          <p className="text-sm mt-1">
            Could not find an approved PAD request for {trade.ticker} {trade.direction}
          </p>
        </div>
      )}

      {/* SUCCESS BANNER - If matched */}
      {trade.match_status === 'matched' && trade.matched_request_id && (
        <div className="bg-green-50 border border-green-200 text-[#2C5F2D] p-3 rounded mb-4">
          <div className="flex items-center justify-between">
            <span className="font-semibold flex items-center gap-2">
              <CheckCircle className="w-5 h-5" />
              Matched to PAD Request
            </span>
            <Link
              to={`/requests/${trade.matched_request_id}`}
              className="text-[#5471DF] font-mono font-bold hover:underline"
            >
              View PAD-{trade.matched_request_id} →
            </Link>
          </div>
          <p className="text-sm mt-1">
            Match method: {trade.match_method}
          </p>
        </div>
      )}

      <h3 className="font-bold text-[#0E1E3F] mb-3">
        Extracted Trade #{index + 1}
      </h3>

      {/* Structured fields */}
      <dl className="grid grid-cols-2 gap-2 text-sm">
        <dt className="text-slate-500">Ticker</dt>
        <dd className="font-mono font-bold">{trade.ticker || '—'}</dd>

        <dt className="text-slate-500">ISIN</dt>
        <dd className="font-mono">{trade.isin || '—'}</dd>

        <dt className="text-slate-500">Direction</dt>
        <dd className={trade.direction === 'BUY' ? 'text-green-600' : 'text-red-600'}>
          {trade.direction || '—'}
        </dd>

        <dt className="text-slate-500">Quantity</dt>
        <dd>{trade.quantity || '—'}</dd>

        <dt className="text-slate-500">Price</dt>
        <dd>{trade.currency} {trade.price || '—'}</dd>

        <dt className="text-slate-500">Trade Date</dt>
        <dd>{trade.trade_date ? formatDate(trade.trade_date) : '—'}</dd>

        <dt className="text-slate-500">Confidence</dt>
        <dd className={getConfidenceColor(trade.confidence_score)}>
          {Math.round((trade.confidence_score || 0) * 100)}%
          ({trade.confidence_display?.level || 'UNKNOWN'})
          {trade.confidence_display?.level === 'LOW' && ' ⚠️'}
        </dd>
      </dl>

      {/* Account holder section */}
      <div className="mt-4 pt-4 border-t">
        <h4 className="text-slate-500 text-sm mb-1">Account Holder (extracted from PDF)</h4>
        <p className="font-mono">{trade.account_holder || 'Not extracted'}</p>
      </div>
    </div>
  );
}

function getConfidenceColor(score: number | null): string {
  if (!score) return 'text-slate-400';
  if (score >= 0.9) return 'text-[#2C5F2D] font-semibold';
  if (score >= 0.7) return 'text-[#B28C54] font-semibold';
  return 'text-[#B85042] font-semibold';
}
```

---

### 6.5 Files to Modify

| File | Changes |
|------|---------|
| `dashboard/src/pages/PDFHistory.tsx` | Add View PDF button, split-view modal, error states |
| `dashboard/src/api/client.ts` | Add `getPdfUrl()` function |
| `dashboard/src/components/ExtractedTradeCard.tsx` | New component for structured trade display with error banners |

### 6.6 Reusable Patterns

| Pattern | Source |
|---------|--------|
| Draggable split view | `dashboard/src/pages/PADSearch.tsx` (lines 89-140) |
| Signed URL endpoint | `GET /documents/{document_id}/pdf` (already exists) |
| Error banner styling | MAKO error color: `#B85042`, bg: `bg-red-50` |
| Success banner styling | MAKO success color: `#2C5F2D`, bg: `bg-green-50` |

---

### 6.7 Verification

1. **View PDF works**: Click View PDF button → PDF opens in new tab or modal
2. **Error message shows**: Trade with `manual_review` status shows red error banner
3. **Side-by-side modal**: Click document row → Modal shows PDF left, fields right
4. **Fields readable**: Extracted data displayed in structured format (not JSON dump)
5. **Link works**: If matched, PAD request link navigates correctly
6. **Confidence colors**: HIGH=green, MEDIUM=gold, LOW=red with warning icon
