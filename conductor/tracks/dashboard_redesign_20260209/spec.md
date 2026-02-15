# Track Specification: Operations Dashboard Redesign

## 1. Goal
Redesign the Operations Dashboard with three-tier role-aware views (standard, manager, compliance), a new `audit_events` table for compliance activity tracking, sparkline charts, two compliance-only bottom panels (Recent Activity + Request Statistics), a new test user, and comprehensive test suites.

## 2. Problem Summary

**Current Limitations:**
1. Dashboard has binary role split (ops vs standard user) — no manager-scoped view
2. No audit trail for compliance-critical actions (approvals, rejections, breaches, config changes)
3. No real-time activity feed for compliance officers
4. No sparkline trend indicators on dashboard tiles
5. No request statistics panel for compliance reporting
6. Execution tracking deadlines are hardcoded (not configurable)
7. "Mako Conflicts" label is confusing — should be shortened to "Conflicts"
8. Standard users see compliance-specific UI elements they don't need

**User Impact:**
- Managers cannot see subordinate data without compliance privileges
- Compliance officers lack real-time visibility into system activity
- No configurable deadlines for execution tracking
- Standard users get cluttered UI with irrelevant compliance features

## 3. Scope

### In Scope
- Three-tier role-based dashboard: standard (own data), manager (own + subordinates), compliance (all data)
- New `audit_events` table (SQLAlchemy model + Alembic migration)
- Inline audit event writes in 8 existing action handlers (same transaction for rollback safety)
- SVG sparkline component for compliance tiles
- Recent Activity panel with filter chips (All/Approvals/Breaches/Config) and clickable rows
- Activity Detail modal (read-only — no action buttons)
- Request Statistics panel with 30/90-day toggle and outcome distribution bar
- New test user "Tibi Eris" (standard user for testing low-privilege view)
- Configurable execution deadlines (trade_execution_deadline_days, contract_note_deadline_days)
- Comprehensive pytest + Playwright test suites

### Out of Scope
- Email notifications for audit events
- Audit event export/download
- Historical backfill of audit events
- Changes to sidebar navigation structure
- Holding period tile (removed from dashboard, remains in sidebar nav only)

## 4. User Stories

### Standard User
- **As an Employee,** I want to see my own PA dealing requests and pending approvals on the dashboard so I can track my compliance status
- **As an Employee,** I want a "New Request" CTA tile so I can quickly submit a new pre-clearance request

### Manager
- **As a Manager,** I want to see my own requests plus my direct reports' pending approvals so I can manage my team's compliance
- **As a Manager,** I want sub-badges showing "awaiting manager" vs "awaiting compliance" counts so I know where bottlenecks are

### Compliance Officer
- **As a Compliance Officer,** I want to see all users' data across the organization on my dashboard so I can monitor compliance
- **As a Compliance Officer,** I want sparkline trends on each tile so I can spot anomalies at a glance
- **As a Compliance Officer,** I want a Recent Activity feed with filter chips so I can monitor specific event types
- **As a Compliance Officer,** I want a Request Statistics panel with 30/90-day views so I can report on compliance metrics
- **As a Compliance Officer,** I want to click an activity row to see full event details in a read-only modal

## 5. Technical Requirements

- AuditEvent model: BigInteger id, String(50) event_type/target_type, BigInteger actor/target ids, JSONB payload, TIMESTAMPTZ created_at
- 4 database indexes: type+created DESC, created DESC, target_type+target_id, payload GIN
- Audit writes share caller's transaction (session.flush, not commit) for rollback safety
- 8 instrumented action handlers: request_submitted, auto_approved, manager_approved, compliance_approved, request_rejected, execution_confirmed, breach_detected, config_change
- Three-tier scoping via manager_id hierarchy: compliance=all, manager=own+subordinates, standard=own
- Two configurable deadlines via ComplianceConfig: trade_execution_deadline_days (default 2), contract_note_deadline_days (default 30)
- Sparkline fallback: state-based queries when audit_events table is empty (fresh deployment)
- Frontend: React 18, TypeScript, TanStack React Query, Tailwind CSS, MAKO design system

## 6. Acceptance Criteria

**Database & Backend:**
- [x] AuditEvent model exists with correct columns, types, and 4 indexes
- [x] Alembic migration creates `padealing.audit_events` table; downgrade drops cleanly
- [x] AuditEvent registered in models/__init__.py and db/__init__.py
- [x] Repository functions: insert, query-by-type, query-by-id, time-window stats, sparkline daily counts
- [x] Audit writes instrumented in all 8 action handlers, sharing same transaction
- [x] Payloads match spec — each event type contains all required fields
- [x] Enhanced summary returns sub-counts (awaiting_manager, awaiting_compliance, my_pending_requests)
- [x] Manager scoping: manager sees own + subordinates; standard user sees own only
- [x] Sparkline endpoint returns 7-day daily data for 4 compliance metrics
- [x] Request statistics endpoint returns correct 30/90-day aggregations with outcome distribution
- [x] Activity endpoint supports event_type filter param for chip filtering
- [x] Activity detail endpoint returns single event by ID for modal
- [x] Both deadlines configurable via ComplianceConfig, not hardcoded

**Frontend:**
- [x] DashboardTile component supports metric and CTA variants with correct accent colors
- [x] Sparkline SVG component renders polyline + gradient fill, no axes/labels
- [x] Dashboard.tsx renders 3 distinct layouts: standard (4 tiles), manager (4 tiles), compliance (4 tiles + divider + 2 panels)
- [x] Standard/Manager tiles: New Request (green CTA), My Requests (blue), Pending Approvals (amber), Execution Tracking (cyan)
- [x] Compliance tiles: Pending Approvals (amber), Active Breaches (red), Execution Tracking (cyan), Conflicts (purple) — all with sparklines
- [x] No compliance UI for standard/manager: no divider, no bottom panels, no sparklines
- [x] Recent Activity Panel has filter chips (All/Approvals/Breaches/Config), clickable rows, read-only modal
- [x] Request Statistics Panel has 30/90-day toggle, 4 stat cards, outcome distribution bar
- [x] "Mako Conflicts" renamed to "Conflicts" in compliance tiles

**Test User & Tests:**
- [x] Tibi Eris added to dev switcher (devUser.ts) and test seed data (conftest.py) with sample requests
- [ ] Pytest suite passes: data scoping (3 roles), tile data accuracy, audit event writes (8 types), rollback safety, sparkline data, request statistics
- [ ] Playwright suite passes: role-based rendering (3 roles), filter interactions, modal open/close, statistics toggle, dev switcher
- [x] All code syntactically valid — Python passes syntax check; TypeScript follows existing patterns

## 7. Testing Requirements

- **Pytest (`tests/unit/test_dashboard_redesign.py`)**: 21 tests covering data scoping, tile data, sparklines, request statistics
- **Pytest (`tests/unit/test_audit_events.py`)**: 24 tests covering audit writes, payload completeness, rollback safety, read-only dashboard checks
- **Playwright (`dashboard/tests/dashboard_redesign.spec.ts`)**: 27 tests covering role-based rendering, filter interactions, modal, statistics toggle, dev switcher, API authorization

## 8. Timeline

- Phase 1 (Database layer): 0.5 day
- Phase 2 (Audit event writes): 0.5 day
- Phase 3 (API endpoints): 1 day
- Phase 4 (Frontend redesign): 1 day
- Phase 5 (Tibi Eris test user): 0.25 day
- Phase 6 (Test suites): 0.5 day
- Phase 7 (Run tests & patch bugs): 0.5 day

**Total:** ~4 days
