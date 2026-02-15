# Implementation Plan: Operations Dashboard Redesign

**Track:** dashboard_redesign_20260209
**Status:** In Progress (Phase 7 remaining)
**Branch:** DSS-4074

---

## Phase 1: Database Layer — `audit_events` Table ✅
- [x] Create AuditEvent SQLAlchemy model (`src/pa_dealing/db/models/audit_event.py`)
  - BigInteger id, String(50) event_type/target_type, BigInteger actor_user_id/target_id
  - JSONB payload with server_default="{}"
  - DateTime(timezone=True) created_at with server_default=func.now()
  - 4 indexes: type+created DESC, created DESC, target_type+target_id, payload GIN
  - `__table_args__` with `schema="padealing"`
- [x] Register model in `db/models/__init__.py` and `db/__init__.py`
- [x] Create Alembic migration (`alembic/versions/20260209_1200_add_audit_events_table.py`)
  - down_revision='58b9120a773a'
  - Creates table with all 4 indexes; downgrade drops cleanly
- [x] Add 5 repository functions to `db/repository.py`:
  - `insert_audit_event` — INSERT + flush, returns model
  - `get_recent_audit_events` — Recent Activity query with category filter mapping
  - `get_audit_event_by_id` — Single row for modal
  - `get_audit_event_stats` — COUNT + GROUP BY aggregation
  - `get_audit_event_sparkline` — Daily counts with zero-fill for 7 days

## Phase 2: Backend — Audit Event Writes ✅
- [x] Add `from pa_dealing.db.repository import insert_audit_event` to pad_service.py
- [x] Instrument `_submit_request_with_session` — writes `request_submitted` event
- [x] Instrument `_submit_request_with_session` — writes `auto_approved` event when initial_status == "auto_approved"
- [x] Instrument `approve_request` — writes `manager_approved` event (approval_type == "manager")
- [x] Instrument `approve_request` — writes `compliance_approved` event (approval_type == "compliance"/"smf16")
- [x] Instrument `decline_request` — writes `request_rejected` event with rejection_reason
- [x] Instrument `record_execution` — writes `breach_detected` event after breach flush
- [x] Instrument `record_execution` — writes `execution_confirmed` event after successful execution
- [x] Instrument `update_compliance_setting` (dashboard.py) — writes `config_change` event with old/new values
- [x] All writes use session.flush() (not commit) — shares caller's transaction for rollback safety

## Phase 3: Backend — New/Modified Dashboard Endpoints ✅
- [x] Enhanced `/dashboard/summary` with 3-tier role scoping:
  - Compliance/Admin/SMF16 → scope_type="all" (no employee filter)
  - Manager → scope_type="manager" (own + subordinates via manager_id)
  - Standard → scope_type="own" (employee_id filter)
- [x] Enhanced `/dashboard/activity` with event_type filter param (approvals/breaches/config/null)
- [x] New endpoint `GET /dashboard/activity/{event_id}` — single audit event for modal
- [x] New endpoint `GET /dashboard/sparklines` — 7-day sparkline data (compliance only, 403 for others)
- [x] New endpoint `GET /dashboard/request-statistics?days=30|90` — compliance only
- [x] Added `get_enhanced_dashboard_summary` service method with sub-counts, deadline info, manager scope
- [x] Added `get_sparkline_data` with audit_events fallback to state-based queries
- [x] Added `get_request_statistics` with outcome distribution and avg approval time delta
- [x] Added `get_activity_events` with enrichment (title, description, actor_name, dot_color)
- [x] Made execution tracking deadline configurable via ComplianceConfig (replaced hardcoded timedelta(days=2))

## Phase 4: Frontend — Dashboard Redesign ✅
- [x] Created `Sparkline.tsx` — Pure SVG polyline + gradient fill, no axes/labels
- [x] Created `DashboardTile.tsx` — Unified metric/CTA tile with accent colors, sub-badges, sparkline
- [x] Created `ActivityDetailModal.tsx` — Read-only modal using existing Modal component
- [x] Created `RecentActivityPanel.tsx` — Filter chips (All/Approvals/Breaches/Config), clickable rows, relative timestamps
- [x] Created `RequestStatisticsPanel.tsx` — 30/90-day toggle, 2x2 stat cards, outcome distribution bar, legend
- [x] Added TypeScript types to `types/index.ts`: EnhancedDashboardSummary, SparklineData, SparklineResponse, RequestStatistics, AuditEventItem
- [x] Added API client methods to `client.ts`: getSparklines, getRequestStatistics, getActivityEvents, getActivityEventDetail
- [x] Rewrote `Dashboard.tsx` with 3-tier role-aware layout:
  - Standard/Manager: 4-col grid (New Request green CTA, My Requests blue, Pending Approvals amber, Execution Tracking cyan)
  - Compliance: 4-col grid (Pending Approvals amber, Active Breaches red, Execution Tracking cyan, Conflicts purple) + divider + 2-col bottom panels
- [x] "Mako Conflicts" renamed to "Conflicts"

## Phase 5: Test User "Tibi Eris" ✅
- [x] Added to DEV_USERS in `dashboard/src/lib/devUser.ts` (email: teris@mako.com, role: employee)
- [x] Added oracle_employee record in `tests/conftest.py` (status='Active', manager=swilliam)
- [x] Added oracle_contact record mapping teris@mako.com
- [x] Added 3 sample PADRequests (pending_manager AAPL, pending_compliance MSFT, approved GOOGL)
- [x] Added `trade_execution_deadline_days` and `contract_note_deadline_days` to compliance config seeds
- [x] Added `padealing.audit_events` to cleanup tables list

## Phase 6: Write Test Suites ✅
- [x] `tests/unit/test_dashboard_redesign.py` — 21 pytest tests:
  - Data scoping: standard (own), manager (subordinates), compliance (all), cross-user leakage
  - Tile data: sub-count summation, new-today delta, configurable deadlines, near-deadline flagging
  - Sparklines: 7 data points, fallback when audit_events empty
  - Request statistics: 30-day, 90-day, outcome distribution sums, zero-total safety
  - Structure: required keys, sub-field presence
  - Activity: enrichment (title, color, description), config change format
- [x] `tests/unit/test_audit_events.py` — 24 pytest tests:
  - Audit writes: manager_approved, compliance_approved, request_rejected, config_change
  - Payload completeness: 7 event types with all required fields validated
  - Transaction safety: flush not commit, rollback discards events
  - Dashboard reads: summary, activity, sparklines, statistics don't write events
  - Repository: filter categories, sparkline zero-fill, missing ID returns None
- [x] `dashboard/tests/dashboard_redesign.spec.ts` — 27 Playwright tests:
  - Standard user: 4 tiles, no compliance UI, page title, navigation
  - Manager: same 4 tiles, no compliance UI
  - Compliance: 4 compliance tiles, sparklines, divider, both panels, "Conflicts" rename
  - Activity: filter chips, single-select, modal open/close, read-only modal
  - Statistics: 30/90 day toggle, revert
  - Dev switcher: Tibi Eris visibility, role switching
  - API authorization: standard blocked from compliance endpoints

## Phase 7: Run Tests & Patch Bugs
- [ ] Run pytest suite: `pytest tests/unit/test_dashboard_redesign.py tests/unit/test_audit_events.py -v`
- [ ] Fix any failing pytest tests
- [ ] Run Playwright suite: `npx playwright test dashboard/tests/dashboard_redesign.spec.ts`
- [ ] Fix any failing Playwright tests
- [ ] Run full regression: `pytest tests/ -v --timeout=120`
- [ ] Run full Playwright regression: `npx playwright test`
- [ ] Verify Alembic migration applies cleanly: `alembic upgrade head`
- [ ] Verify Alembic downgrade works: `alembic downgrade -1` then `alembic upgrade head`

---

## Files Summary

### NEW files created:
| File | Purpose |
|------|---------|
| `src/pa_dealing/db/models/audit_event.py` | AuditEvent SQLAlchemy model |
| `alembic/versions/20260209_1200_add_audit_events_table.py` | Alembic migration |
| `dashboard/src/components/dashboard/DashboardTile.tsx` | Unified metric/CTA tile component |
| `dashboard/src/components/dashboard/Sparkline.tsx` | SVG sparkline component |
| `dashboard/src/components/dashboard/RecentActivityPanel.tsx` | Activity panel with filters + clickable rows |
| `dashboard/src/components/dashboard/ActivityDetailModal.tsx` | Event detail modal (read-only) |
| `dashboard/src/components/dashboard/RequestStatisticsPanel.tsx` | Stats panel with period toggle + distribution bar |
| `tests/unit/test_dashboard_redesign.py` | Pytest: data scoping, tile data, sparkline, statistics |
| `tests/unit/test_audit_events.py` | Pytest: audit event writes, rollback, payload completeness |
| `dashboard/tests/dashboard_redesign.spec.ts` | Playwright: role rendering, interactions, dev switcher |

### EXISTING files modified:
| File | Changes |
|------|---------|
| `src/pa_dealing/db/models/__init__.py` | Added AuditEvent import + __all__ entry |
| `src/pa_dealing/db/__init__.py` | Added AuditEvent import + __all__ entry |
| `src/pa_dealing/db/repository.py` | Added 5 audit_event query functions |
| `src/pa_dealing/services/pad_service.py` | Added audit writes in 7 handlers; 6 new dashboard methods; configurable deadline |
| `src/pa_dealing/api/routes/dashboard.py` | Added config_change audit write; 3-tier /summary; enhanced /activity; 3 new endpoints |
| `dashboard/src/types/index.ts` | Added 5 new TypeScript interfaces |
| `dashboard/src/api/client.ts` | Added 4 new API methods to dashboard object |
| `dashboard/src/pages/Dashboard.tsx` | Full rewrite: 3-tier role rendering, new tiles, compliance panels |
| `dashboard/src/lib/devUser.ts` | Added Tibi Eris to DEV_USERS array |
| `tests/conftest.py` | Added teris employee/contact/requests; deadline configs; audit_events cleanup |
