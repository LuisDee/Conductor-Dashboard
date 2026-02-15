# Spec: Restricted Instruments UI & Sync Controls

## Overview

Enhance the dashboard to display the restricted instruments list and provide controls for sync management. This includes adding a restricted instruments table to the Mako Conflicts page, a "Sync Now" button, configurable sync frequency, and improved status visibility.

## Background

### Current System
- `RestrictedSecurity` table stores restricted instruments (synced from Confluence)
- Sync service runs on a fixed interval (`RESTRICTED_LIST_SYNC_INTERVAL_MINUTES`, default 60)
- Settings page shows basic sync status (status, last sync, count) but is read-only
- No UI to view actual restricted instruments or trigger manual sync
- Sync interval can only be changed via environment variable (requires restart)

### Problems
1. Compliance cannot see which instruments are on the restricted list from the dashboard
2. No way to trigger an immediate sync (must wait for scheduled interval)
3. Cannot adjust sync frequency without redeploying
4. Status display buried in Settings page, not contextually relevant

### Solution
1. Add "Restricted Instruments" section to Mako Conflicts page
2. Add "Sync Now" button with timestamp display
3. Add sync frequency control to Settings page
4. Move/duplicate status display to Mako Conflicts page

---

## Functional Requirements

### FR1: Restricted Instruments Table on Mako Conflicts Page

Add a new section below the existing Mako Conflicts table.

**Table Columns:**
| Column | Description |
|--------|-------------|
| Instrument | `inst_symbol` (primary), with ISIN as sublabel if available |
| Reason | Restriction reason text |
| Added | `added_at` timestamp formatted |
| Status | Badge: Active (green) / Inactive (gray) |

**Features:**
- Filter by search (ticker/ISIN)
- Toggle to show/hide inactive instruments (default: hide)
- Sort by added date (newest first)
- Empty state: "No restricted instruments configured"

**Data Source:**
- New API endpoint: `GET /dashboard/restricted-instruments`
- Returns all `RestrictedSecurity` records with `is_active=True` (or all if filter toggled)

### FR2: Sync Now Button

Add a sync control panel above the restricted instruments table.

**Components:**
1. **Status Card**
   - Sync Status: `SUCCESS` / `FAILURE` / `PENDING` / `UNKNOWN` (color-coded badge)
   - Last Synced: Formatted timestamp, or "Never"
   - Instruments Count: Number badge
   - Sync Frequency: "Every X minutes"

2. **Sync Now Button**
   - Primary action button
   - Disabled while sync in progress
   - Shows spinner during sync
   - On success: Toast "Sync completed - X instruments updated"
   - On failure: Toast error with message

**API Endpoint:**
- `POST /config/restricted-list-sync` - Trigger immediate sync
- Returns: `{ success: bool, message: str, stats: { added, updated, removed } }`

### FR3: Configurable Sync Frequency

Add sync frequency control to Settings page (in the Restricted List Sync card).

**Control:**
- Dropdown or input: 15 / 30 / 60 / 120 / 240 minutes
- Save updates `ComplianceConfig` table (new key: `restricted_list_sync_interval_minutes`)
- On save: Reschedule the APScheduler job dynamically

**Implementation:**
- Store in `compliance_config` table (not env var) for runtime changes
- Settings priority: DB config > env var > default (60)
- Scheduler reads from config on job execution

### FR4: API Endpoints

#### `GET /dashboard/restricted-instruments`
```typescript
Response: {
  data: [
    {
      id: number;
      inst_symbol: string;
      isin: string | null;
      reason: string | null;
      is_active: boolean;
      added_at: string;  // ISO timestamp
    }
  ]
}
```

Query params:
- `include_inactive: bool` (default: false)
- `search: string` (optional, filters by inst_symbol or isin)

#### `POST /config/restricted-list-sync`
```typescript
Request: {}  // No body required

Response: {
  success: boolean;
  message: string;
  data: {
    added: number;
    updated: number;
    removed: number;
    total: number;
  }
}
```

#### `PUT /config/restricted-list-sync-interval`
```typescript
Request: {
  interval_minutes: number;  // 15, 30, 60, 120, 240
}

Response: {
  success: boolean;
  message: string;
  data: {
    interval_minutes: number;
    next_sync: string;  // ISO timestamp
  }
}
```

### FR5: Dashboard API Client Updates

Add to `dashboard/src/api/client.ts`:

```typescript
export const config = {
  // ... existing methods

  getRestrictedInstruments: async (params?: {
    include_inactive?: boolean;
    search?: string;
  }): Promise<RestrictedInstrument[]> => { ... },

  triggerRestrictedListSync: async (): Promise<{
    added: number;
    updated: number;
    removed: number;
    total: number;
  }> => { ... },

  updateSyncInterval: async (intervalMinutes: number): Promise<{
    interval_minutes: number;
    next_sync: string;
  }> => { ... },
};
```

---

## UI Design

### Mako Conflicts Page Layout

```
┌─────────────────────────────────────────────────────────┐
│ Mako Conflicts                                          │
│ Internal trade correlation and position conflict        │
│ monitoring                                              │
├─────────────────────────────────────────────────────────┤
│ [Filters: Security | Employee]              [Reset]     │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Employee | Instrument | Emp Pos | Mako Pos | Type   │ │
│ │ ...                                                 │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                         │
│ ═══════════════════════════════════════════════════════ │
│                                                         │
│ Restricted Instruments                                  │
│ Instruments prohibited from personal trading            │
│                                                         │
│ ┌──────────────────────────────────┐                   │
│ │ Status: ● SUCCESS    Last Sync: 2 mins ago          │
│ │ Instruments: 47      Frequency: Every 60 mins       │
│ │                               [🔄 Sync Now]         │
│ └──────────────────────────────────┘                   │
│                                                         │
│ [Search...]                    [☐ Show Inactive]       │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Instrument    | Reason                    | Added   │ │
│ │ AAPL          | Insider investigation     | Jan 15  │ │
│ │ US0378331005  |                           |         │ │
│ │ TSLA          | Pending acquisition       | Jan 10  │ │
│ │ ...                                                 │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Settings Page - Enhanced Sync Card

```
┌─────────────────────────────────────────────────────────┐
│ 🔄 Restricted List Sync                                 │
│ Status of integration with Confluence                   │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Status           │ ● SUCCESS                        │ │
│ │ Last Sync        │ Jan 29, 2026 14:30:00           │ │
│ │ Instruments      │ 47                               │ │
│ │ Sync Frequency   │ [Every 60 minutes ▼]            │ │
│ │ Error            │ (none)                           │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                         │
│                               [🔄 Sync Now]             │
└─────────────────────────────────────────────────────────┘
```

---

## Technical Implementation

### Files to Modify

**Backend:**
| File | Changes |
|------|---------|
| `src/pa_dealing/api/routes/dashboard.py` | Add `GET /dashboard/restricted-instruments` |
| `src/pa_dealing/api/routes/config.py` | Add `POST /config/restricted-list-sync`, `PUT /config/restricted-list-sync-interval` |
| `src/pa_dealing/services/restricted_list_sync.py` | Add `get_restricted_instruments()`, expose sync trigger |
| `src/pa_dealing/agents/monitoring/scheduler.py` | Dynamic interval reloading |

**Frontend:**
| File | Changes |
|------|---------|
| `dashboard/src/pages/MakoConflicts.tsx` | Add restricted instruments section |
| `dashboard/src/pages/Settings.tsx` | Add sync interval control, sync button |
| `dashboard/src/api/client.ts` | Add new API methods |
| `dashboard/src/types/index.ts` | Add `RestrictedInstrument` type |

### Files to Create

| File | Purpose |
|------|---------|
| `dashboard/src/components/RestrictedInstrumentsSection.tsx` | Reusable section component |
| `dashboard/src/components/SyncStatusCard.tsx` | Sync status + controls card |

---

## Non-Functional Requirements

### NFR1: Performance
- Restricted instruments query should be paginated if count > 100
- Sync status should poll every 30 seconds (or use WebSocket later)

### NFR2: Authorization
- All endpoints require compliance/admin role
- Sync trigger is audit-logged

### NFR3: Error Handling
- Sync failures show clear error message
- UI remains functional if sync service unavailable
- Stale data warning if last sync > 2 hours ago

### NFR4: Mako Brand Colors
- Success badge: `#2C5F2D`
- Error badge: `#B85042`
- Warning badge: `#B28C54`
- Primary button: `#5471DF`

---

## Acceptance Criteria

### AC1: Restricted Instruments Display
- [ ] Restricted instruments table visible on Mako Conflicts page
- [ ] Table shows inst_symbol, ISIN, reason, added date, status
- [ ] Search filter works
- [ ] Show inactive toggle works

### AC2: Sync Now Functionality
- [ ] Sync Now button triggers immediate sync
- [ ] Button disabled during sync with spinner
- [ ] Success/failure toast displayed
- [ ] Status updates after sync completes

### AC3: Sync Interval Configuration
- [ ] Dropdown to select sync interval in Settings
- [ ] Save persists to database
- [ ] Scheduler respects new interval without restart
- [ ] Current interval displayed in UI

### AC4: Status Visibility
- [ ] Sync status card on Mako Conflicts page
- [ ] Shows: status, last sync time, instrument count, frequency
- [ ] Warning if sync failed or data stale

### AC5: Authorization
- [ ] All endpoints require compliance/admin role
- [ ] Sync trigger audit logged

---

## Out of Scope

- Real-time WebSocket updates (use polling for now)
- Add/edit restricted instruments from UI (managed in Confluence)
- Historical sync logs in UI
- Slack notifications for sync failures

---

## Test Plan

### Unit Tests
1. Test `get_restricted_instruments()` returns correct data
2. Test sync trigger endpoint returns stats
3. Test interval update persists and schedules correctly

### Integration Tests
1. Test full sync flow with mock Confluence
2. Test UI displays instruments correctly
3. Test sync button triggers and updates UI

### Manual UAT
1. Navigate to Mako Conflicts, verify restricted instruments section
2. Click Sync Now, verify sync runs and status updates
3. Change sync interval in Settings, verify new schedule
4. Verify Mako brand colors used throughout

---

## Implementation Sequence

1. **Phase 1: Backend APIs**
   - Add `GET /dashboard/restricted-instruments`
   - Add `POST /config/restricted-list-sync`
   - Add `PUT /config/restricted-list-sync-interval`
   - Add dynamic scheduler interval reloading

2. **Phase 2: Frontend - Mako Conflicts Page**
   - Create `RestrictedInstrumentsSection` component
   - Create `SyncStatusCard` component
   - Integrate into Mako Conflicts page

3. **Phase 3: Frontend - Settings Page**
   - Add sync interval dropdown
   - Add Sync Now button
   - Wire up API calls

4. **Phase 4: Polish & Testing**
   - Add loading states
   - Add error handling
   - Write tests
   - Manual UAT
