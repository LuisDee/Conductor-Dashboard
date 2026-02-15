# Implementation Plan: Dashboard UI Overhaul & Role-Based Views

## Phase 1: Sidebar Restructuring
- [ ] Refactor `dashboard/src/components/layout/Sidebar.tsx`
    - [ ] Define `MY_PA_DEALING_ITEMS` and `OPERATIONS_ITEMS`
    - [ ] Update `navItems` configuration
    - [ ] Implement conditional header rendering ("MY PA DEALING" vs "OPERATIONS")
    - [ ] Group bottom items under "ADMIN & TOOLS"
- [ ] Verify sidebar visibility for both Regular and Compliance roles

## Phase 2: Backend Activity & Trends
- [ ] Add `ActionType` variants if missing for activity feed
- [ ] Implement `PADService.get_recent_activity()` in `src/pa_dealing/services/pad_service.py`
- [ ] Add `GET /api/dashboard/activity` endpoint in `src/pa_dealing/api/routes/dashboard.py`
- [ ] Update `PADService.get_dashboard_summary_counts`
    - [ ] Implement yesterday's count logic for all 6 metrics
    - [ ] Return updated data structure with `current` and `delta` fields
- [ ] Update `get_trade_history` in `src/pa_dealing/db/repository.py` to support `employee_id`

## Phase 3: Dashboard Overhaul (Frontend)
- [ ] Update `types/index.ts` with new `DashboardSummary` structure
- [ ] Modify `dashboard/src/pages/Dashboard.tsx`
    - [ ] Remove Quick Actions section
    - [ ] Create and integrate `ActivityFeed` component
    - [ ] Update `StatCard` to render trend indicators
- [ ] Verify data fetching and rendering accuracy

## Phase 4: "ALL / MY" Filter Integration
- [ ] Create `dashboard/src/components/ui/ViewFilterToggle.tsx`
- [ ] Integrate toggle into `MyRequests.tsx` (and handle label rename)
- [ ] Integrate toggle into `HoldingPeriods.tsx`
- [ ] Integrate toggle into `TradeHistory.tsx`
- [ ] Integrate toggle into `ExecutionTracking.tsx`
- [ ] Ensure all pages correctly pass `employee_id` to API client

## Phase 5: Verification & Polishing
- [ ] Manual walkthrough of all roles and views
- [ ] Regression testing of core listing pages
- [ ] UI/UX polishing (spacing, animations for trend arrows)
