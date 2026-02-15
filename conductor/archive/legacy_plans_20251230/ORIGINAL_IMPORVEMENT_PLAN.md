# PA Dealing Dashboard - Comprehensive UX Improvements Plan

## Overview
Complete implementation of UX improvements including human-readable reference IDs, request detail page, filtering on all pages, audit log fixes, and comprehensive testing.

## Priority Order
1. **Human-Readable Reference IDs** - Foundation for all other features
2. **Request Detail Page** - Central hub for viewing request lifecycle
3. **Filtering on All Pages** - Consistent UX across dashboard
4. **Audit Log Improvements** - Fix bugs, improve readability
5. **Testing** - E2E, unit, and Playwright tests

---

## Phase 1: Human-Readable Reference IDs

### 1.1 Database Model (DONE)
- [x] Added `reference_id` column to `PADRequest` model (`db/models.py:236`)

### 1.2 Reference ID Generation
**File:** `src/pa_dealing/utils/reference_id.py` (created)

Format: `MAKO_ID-YYMMDD-TICKER` (e.g., `SKEMP-241222-AAPL`)
- User part: First 8 chars of mako_id (uppercase)
- Date part: YYMMDD format
- Security part: Ticker or first 6 chars of security name
- Collision handling: Adds `-2`, `-3` suffix for same-day duplicates

### 1.3 Generate on Request Creation
**File:** `src/pa_dealing/agents/database/tools.py`
- Update `submit_pad_request()` function (line ~901)
- Call `get_next_sequence()` to handle duplicates
- Set `reference_id` on PADRequest before saving

### 1.4 Update API Responses
**Files to modify:**
- `src/pa_dealing/api/schemas.py` - Add `reference_id: str | None` to all response models
- `src/pa_dealing/services/pad_service.py` - Include `reference_id` in all dict returns
- `src/pa_dealing/agents/database/schemas.py` - Add to `PADRequestInfo`

**Response models to update:**
- `PADRequestResponse`
- `PendingApprovalResponse`
- `ExecutionTrackingItem`
- `HoldingPeriodItem`

### 1.5 Update Frontend Types
**File:** `dashboard/src/types/index.ts`
- Add `reference_id: string` to `PADRequest`, `ExecutionTracking`, `HoldingPeriod`

### 1.6 Display Reference ID in UI
**Files to modify:**
- `dashboard/src/pages/PendingApprovals.tsx` - Show reference_id in ID column
- `dashboard/src/pages/ExecutionTracking.tsx` - Show reference_id
- `dashboard/src/pages/HoldingPeriods.tsx` - Show reference_id
- `dashboard/src/pages/AuditLog.tsx` - Show reference_id for entity

### 1.7 Database Migration
- Add column if not exists: `ALTER TABLE pad_request ADD COLUMN IF NOT EXISTS reference_id VARCHAR(50) UNIQUE`
- Create index: `CREATE INDEX IF NOT EXISTS ix_pad_request_reference_id ON pad_request(reference_id)`

---

## Phase 2: Request Detail Page

### 2.1 Backend API Endpoint
**File:** `src/pa_dealing/api/routes/requests.py`

New endpoint: `GET /api/requests/{request_id}/detail`

Returns comprehensive data:
```python
{
    "request": {...},  # Full PADRequest with reference_id
    "employee": {...},  # Employee details
    "security": {...},  # Security details
    "approvals": [...],  # List of PADApproval records with approver names
    "execution": {...},  # PADExecution if exists
    "breaches": [...],  # Related PADBreach records
    "audit_trail": [...],  # Filtered audit log entries for this request
}
```

### 2.2 Service Method
**File:** `src/pa_dealing/services/pad_service.py`

New method: `get_request_detail(request_id: int) -> dict`
- Fetches request with all relationships
- Fetches audit log entries for this entity
- Aggregates into comprehensive response

### 2.3 Frontend Route
**File:** `dashboard/src/App.tsx`
- Add route: `<Route path="/requests/:id" element={<RequestDetail />} />`

### 2.4 Request Detail Page Component
**File:** `dashboard/src/pages/RequestDetail.tsx` (new)

Layout:
```
+------------------------------------------+
| Request SKEMP-241222-AAPL                |
| Status: [Approved]  Risk: [MEDIUM]       |
+------------------------------------------+
|                    |                     |
| Request Info       | Security Info       |
| - Employee         | - Ticker/Bloomberg  |
| - Direction/Qty    | - ISIN/SEDOL        |
| - Value/Currency   | - Description       |
| - Justification    |                     |
+------------------------------------------+
| Approval Timeline                        |
| [Manager] jsmith - Approved - 10:30am    |
| [Compliance] skemp - Approved - 11:45am  |
+------------------------------------------+
| Execution Details (if executed)          |
| Price: $150.25  Qty: 100  Broker: XYZ123 |
+------------------------------------------+
| Audit Trail                              |
| [table of audit entries for this request]|
+------------------------------------------+
```

### 2.5 Add Links from Other Pages
- PendingApprovals: Click row or ID to navigate
- ExecutionTracking: Click request_id to navigate
- HoldingPeriods: Click request_id to navigate
- AuditLog: Click entity_id to navigate (if entity_type=pad_request)

---

## Phase 3: Filtering on All Pages

### 3.1 Backend Filter Parameters
**Files to modify:**

`src/pa_dealing/api/routes/dashboard.py`:
- `GET /pending-approvals?status=&employee_id=&risk_level=&start_date=&end_date=`
- `GET /breaches?severity=&breach_type=&employee_id=&resolved=`
- `GET /execution-tracking?employee_id=&security=&overdue_only=`
- `GET /holding-periods?employee_id=&ending_within_days=`
- `GET /mako-conflicts?employee_id=&security=`

`src/pa_dealing/services/pad_service.py`:
- Update each method to accept filter parameters
- Use conditional WHERE clause pattern from audit log

### 3.2 Frontend Filter Components

**Pattern to follow:** AuditLog.tsx filter implementation

**Files to modify:**

`dashboard/src/pages/PendingApprovals.tsx`:
```typescript
const [filters, setFilters] = useState({
  status: '',      // pending_manager, pending_compliance, pending_smf16
  risk_level: '',  // LOW, MEDIUM, HIGH
  employee: '',    // Employee name search
  start_date: '',
  end_date: '',
})
```

`dashboard/src/pages/Breaches.tsx`:
```typescript
const [filters, setFilters] = useState({
  severity: '',     // LOW, MEDIUM, HIGH, CRITICAL
  breach_type: '',  // holding_period, unauthorized, etc.
  resolved: '',     // true, false, all
})
```

`dashboard/src/pages/ExecutionTracking.tsx`:
```typescript
const [filters, setFilters] = useState({
  urgency: '',  // overdue, expiring_soon, all
  employee: '',
})
```

`dashboard/src/pages/HoldingPeriods.tsx`:
```typescript
const [filters, setFilters] = useState({
  ending_within: '',  // 7, 14, 30, all
  employee: '',
})
```

`dashboard/src/pages/MakoConflicts.tsx`:
```typescript
const [filters, setFilters] = useState({
  employee: '',
  security: '',
})
```

### 3.3 API Client Updates
**File:** `dashboard/src/api/client.ts`
- Update each dashboard method to accept filter params
- Use `cleanParams()` helper to remove empty values

---

## Phase 4: Audit Log Improvements

### 4.1 Fix Employee Filter Bug
**File:** `dashboard/src/pages/AuditLog.tsx`
- Line 173-184: Both "Actor (Email)" and "Employee (Name)" filters update `filters.actor`
- Fix: Create separate `employee_name` filter field
- Update API to accept `employee_name` parameter

### 4.2 Format Details Nicely
**File:** `dashboard/src/pages/AuditLog.tsx`
- Replace raw JSON with formatted display
- Create `AuditDetails` component that renders common patterns:
  - Security info: ticker, direction, quantity
  - Approval info: approver, decision, comments
  - Execution info: price, quantity, broker
  - Status changes: old_status → new_status

### 4.3 Add Request Timeline View
When filtering by entity_id (request), show timeline visualization:
- Vertical timeline with icons for each action
- Color-coded by action type
- Expandable details

### 4.4 Backend: Add employee_name Filter
**File:** `src/pa_dealing/services/pad_service.py`
- Update `search_audit_log()` to accept `employee_name` param
- Join to employees table to filter by name

---

## Phase 5: Testing

### 5.1 Database Migration Test
**File:** `tests/test_migrations.py` (new)
- Test reference_id column exists
- Test index exists
- Test unique constraint

### 5.2 Reference ID Generation Tests
**File:** `tests/test_reference_id.py` (new)
```python
class TestReferenceIdGeneration:
    def test_basic_format(self):
        ref = generate_reference_id("skemp", ticker="AAPL")
        assert ref == "SKEMP-YYMMDD-AAPL"

    def test_collision_handling(self):
        # Same user, same day, same security
        ref1 = generate_reference_id("skemp", ticker="AAPL", sequence=0)
        ref2 = generate_reference_id("skemp", ticker="AAPL", sequence=1)
        assert ref2 == ref1 + "-2"

    def test_no_ticker_uses_name(self):
        ref = generate_reference_id("skemp", security_name="Apple Inc")
        assert "APPLE" in ref
```

### 5.3 API Tests for Reference ID
**File:** `tests/test_e2e_api.py` (update)
```python
class TestReferenceId:
    def test_submit_returns_reference_id(self, client):
        response = client.post("/requests", ...)
        assert "reference_id" in response.json()["data"]
        assert response.json()["data"]["reference_id"].startswith("SKEMP-")

    def test_get_request_includes_reference_id(self, client):
        # Create request, then fetch
        response = client.get(f"/requests/{request_id}")
        assert "reference_id" in response.json()["data"]

    def test_pending_approvals_includes_reference_id(self, client):
        response = client.get("/dashboard/pending-approvals")
        for item in response.json()["data"]:
            assert "reference_id" in item
```

### 5.4 Request Detail API Tests
**File:** `tests/test_e2e_api.py` (update)
```python
class TestRequestDetail:
    def test_get_request_detail(self, client):
        response = client.get(f"/requests/{request_id}/detail")
        assert response.status_code == 200
        data = response.json()["data"]
        assert "request" in data
        assert "employee" in data
        assert "approvals" in data
        assert "audit_trail" in data

    def test_request_detail_404(self, client):
        response = client.get("/requests/999999/detail")
        assert response.status_code == 404
```

### 5.5 Filter API Tests
**File:** `tests/test_e2e_api.py` (update)
```python
class TestFiltering:
    def test_pending_approvals_filter_by_status(self, client):
        response = client.get("/dashboard/pending-approvals?status=pending_manager")
        # All items should have status=pending_manager

    def test_pending_approvals_filter_by_risk(self, client):
        response = client.get("/dashboard/pending-approvals?risk_level=HIGH")
        # All items should have risk_level=HIGH

    def test_breaches_filter_by_severity(self, client):
        response = client.get("/dashboard/breaches?severity=CRITICAL")
```

### 5.6 Playwright Tests
**File:** `dashboard/tests/pages.spec.ts` (update)

```typescript
test.describe('Request Detail Page', () => {
  test('displays request details correctly', async ({ page }) => {
    // First get a request ID from pending approvals
    await page.goto('/pending-approvals');
    const firstRow = page.locator('table tbody tr').first();
    const requestId = await firstRow.locator('td').first().textContent();

    // Navigate to detail page
    await firstRow.click();
    await page.waitForURL(/\/requests\/\d+/);

    // Verify sections exist
    await expect(page.locator('text=/Request Info/i')).toBeVisible();
    await expect(page.locator('text=/Approval Timeline/i')).toBeVisible();
    await expect(page.locator('text=/Audit Trail/i')).toBeVisible();
  });

  test('reference_id is displayed', async ({ page }) => {
    await page.goto('/pending-approvals');
    // Check reference_id format is visible (e.g., SKEMP-241222-AAPL)
    await expect(page.locator('text=/[A-Z]+-\\d{6}-[A-Z]+/')).toBeVisible();
  });
});

test.describe('Filtering', () => {
  test('pending approvals can filter by risk level', async ({ page }) => {
    await page.goto('/pending-approvals');

    // Find and use risk filter
    await page.selectOption('select[name="risk_level"]', 'HIGH');

    // Wait for filter to apply
    await page.waitForTimeout(500);

    // Verify results are filtered
    const riskBadges = page.locator('.badge:has-text("HIGH")');
    const count = await riskBadges.count();
    // All visible risk badges should be HIGH
  });

  test('breaches can filter by severity', async ({ page }) => {
    await page.goto('/breaches');
    await page.selectOption('select[name="severity"]', 'CRITICAL');
    // Verify filtering works
  });
});
```

---

## Implementation Order

### Day 1: Reference IDs (Foundation)
1. Complete reference_id generation in `submit_pad_request()`
2. Update all API response schemas
3. Update service methods to return reference_id
4. Update frontend types
5. Update frontend display
6. Write tests

### Day 2: Request Detail Page
1. Create backend endpoint and service method
2. Create frontend page component
3. Add route
4. Add navigation links from other pages
5. Write tests

### Day 3: Filtering
1. Update backend endpoints with filter params
2. Update service methods
3. Update frontend pages with filter UI
4. Update API client
5. Write tests

### Day 4: Audit Log + Final Testing
1. Fix employee filter bug
2. Format audit details nicely
3. Run full test suite
4. Fix any issues
5. Final verification

---

## Files to Create
- `dashboard/src/pages/RequestDetail.tsx`
- `tests/test_reference_id.py`

## Files to Modify
### Backend
- `src/pa_dealing/agents/database/tools.py` - Generate reference_id
- `src/pa_dealing/api/schemas.py` - Add reference_id to response models
- `src/pa_dealing/api/routes/requests.py` - Add detail endpoint
- `src/pa_dealing/api/routes/dashboard.py` - Add filter params
- `src/pa_dealing/services/pad_service.py` - Add methods, update returns
- `src/pa_dealing/agents/database/schemas.py` - Add reference_id

### Frontend
- `dashboard/src/types/index.ts` - Add reference_id
- `dashboard/src/api/client.ts` - Add filter params
- `dashboard/src/App.tsx` - Add route
- `dashboard/src/pages/PendingApprovals.tsx` - Filters + reference_id display
- `dashboard/src/pages/Breaches.tsx` - Filters
- `dashboard/src/pages/ExecutionTracking.tsx` - Filters + reference_id
- `dashboard/src/pages/HoldingPeriods.tsx` - Filters + reference_id
- `dashboard/src/pages/MakoConflicts.tsx` - Filters
- `dashboard/src/pages/AuditLog.tsx` - Fix bug + format details
- `dashboard/src/components/Layout.tsx` - Add nav for request detail (if needed)

### Tests
- `tests/test_e2e_api.py` - Add reference_id and filter tests
- `dashboard/tests/pages.spec.ts` - Add detail page and filter tests

---

## Success Criteria
1. All requests have human-readable reference_id (format: SKEMP-241222-AAPL)
2. Reference IDs displayed in all tables and detail views
3. Request detail page shows complete lifecycle
4. All pages have working filters
5. Audit log employee filter bug fixed
6. All existing tests pass
7. New tests for reference_id, detail page, and filters pass
8. Playwright tests verify UI changes
