# Specification: Dashboard UI Overhaul & Role-Based Views

## Objective
Enhance the user experience and situational awareness of the PA Dealing Dashboard through role-based sidebar restructuring, real-time activity feeds, and trend indicators.

## Requirements

### 1. Sidebar Restructuring
The sidebar will be divided into three primary sections based on the user's role:

- **Section 1: Contextual Header**
    - **Regular Users:** "MY PA DEALING"
        - Dashboard
        - New Request
        - My Requests
        - Holding Periods
        - Trade History
        - Execution Tracking
    - **Compliance Users:** "OPERATIONS"
        - Dashboard
        - Requests (was My Requests)
        - Holding Periods
        - Trade History
        - Execution Tracking
        - Breaches
        - Mako Conflicts
        - Restricted List
        - PAD Search

- **Section 2: Shared Tools**
    - **Both Roles:** "ADMIN & TOOLS"
        - PDF Ingestion
        - Audit Log
        - Reports
        - AI Accuracy
        - Settings
        - System Health

### 2. "ALL / MY" Filter Toggle (Compliance)
Compliance users need a way to switch between their own data and everyone's data on core listing pages.

- **Target Pages:** Requests, Holding Periods, Trade History, Execution Tracking.
- **Component:** A prominent toggle or tab group at the top of the content area.
- **States:** "ALL" (default for compliance) and "MY" (filtered by current user's `employee_id`).

### 3. Dashboard Activity Feed
Replace the "Quick Actions" section with a live activity feed.

- **Content:** Last 10 system actions from the `AuditLog`.
- **Actions:** Requests submitted, approvals granted, breaches resolved, PDFs uploaded, executions recorded.
- **UX:** Chronological list with timestamps and action badges.

### 4. Summary Card Trend Indicators
Summary cards should show deltas from "yesterday" to provide quick insight into whether metrics are improving or worsening.

- **Example:** "19 pending, ↑3 from yesterday"
- **Implementation:** Backend will calculate counts for the current state vs. the state 24 hours ago.

## Technical Design

### Frontend
- **Sidebar:** Refactor `Sidebar.tsx` to group items into sections and use conditional logic for headers.
- **Filter Toggle:** Create `ViewFilterToggle.tsx` using Tailwind styling.
- **Activity Feed:** Create `ActivityFeed.tsx` fetching from a new `/api/dashboard/activity` endpoint.
- **StatCard:** Update `StatCard.tsx` to accept and render `trend` data.

### Backend
- **Activity Endpoint:** Add `GET /api/dashboard/activity` fetching top 10 `AuditLog` entries.
- **Summary Trends:** Update `PADService.get_dashboard_summary_counts` to perform dual-count queries (now vs. 24h ago).
- **Trade History Filter:** Update `get_trade_history` to support `employee_id` filtering.

## Success Criteria
- Sidebar headers change correctly based on user role.
- Compliance users can toggle between personal and global data on listing pages.
- "Quick Actions" are removed from the dashboard.
- Dashboard shows a functional activity feed of recent actions.
- Summary cards display trend deltas (e.g., +3).
